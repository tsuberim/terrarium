//! Creature runtime — one WASM tick per sim tick, think → act → world-tick.

use std::collections::HashMap;

use uuid::Uuid;

use crate::abi::tile;
use crate::hex;
use crate::abi::corpse_yield_energy;
use crate::energy_ledger::EnergyLedger;
use crate::energy_nodes::try_spawn_nodes;
use crate::events::{DeathReason, TickResult, WorldEvent};
use crate::host::{self, PendingAction, ThinkResult};
use crate::sim_config::SimConfig;
use crate::world_tile::{place_corpse, sense_kind, set_cell, blocks_movement, WorldTile, WorldTiles};

#[derive(Clone, Debug)]
pub struct Signal {
    pub from_id: String,
    pub from_x: i32,
    pub from_y: i32,
    pub byte: u8,
    pub broadcast: bool,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub energy: i64,
    pub health: i32,
    pub max_health: i32,
    pub owner_uid: String,
    pub parent_id: Option<String>,
    pub wasm: Vec<u8>,
    pub code: String,
    pub alive: bool,
    pub inbox: Vec<Signal>,
    pub death_reason: Option<DeathReason>,
    /// Sim tick when the creature was deployed or spawned.
    pub born_tick: u64,
}

pub(crate) fn mark_dead(creature: &mut Creature, reason: DeathReason) {
    creature.alive = false;
    if creature.death_reason.is_none() {
        creature.death_reason = Some(reason);
    }
}

fn death_event(creature: &Creature) -> WorldEvent {
    WorldEvent::Death {
        creature_id: creature.id.clone(),
        owner_uid: creature.owner_uid.clone(),
        x: creature.x,
        y: creature.y,
        reason: creature.death_reason.unwrap_or(DeathReason::WasmTrap),
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub positions: HashMap<String, (i32, i32)>,
    pub id_at: HashMap<(i32, i32), String>,
    pub energy: HashMap<String, i64>,
    pub health: HashMap<String, i32>,
    pub max_health: HashMap<String, i32>,
}

pub fn run_tick(
    creatures: &mut Vec<Creature>,
    tiles: &mut WorldTiles,
    ledger: &mut EnergyLedger,
    config: &SimConfig,
    tick: u64,
) -> TickResult {
    let destroyed_before = ledger.destroyed;
    let free_before = ledger.free_minted;
    let mut result = TickResult::default();
    let snapshot = build_snapshot(creatures);
    let engine = host::wasm_engine();

    let mut pending: Vec<(usize, ThinkResult)> = Vec::new();
    let mut suicide_ids: HashMap<String, ()> = HashMap::new();

    for (i, creature) in creatures.iter_mut().enumerate() {
        if !creature.alive {
            continue;
        }
        if creature.energy <= config.corpse_energy {
            mark_dead(creature, DeathReason::EnergyFloor);
            result.events.push(death_event(creature));
            continue;
        }
        let think = think_once(engine, creature, &snapshot, tiles, config, ledger, tick);
        if !creature.alive {
            result.events.push(death_event(creature));
            continue;
        }
        if think.suicide {
            suicide_ids.insert(creature.id.clone(), ());
        }
        pending.push((i, think));
    }

    for (i, think) in &pending {
        if think.suicide || !think.actions.is_empty() {
            continue;
        }
        let c = &mut creatures[*i];
        if c.health >= c.max_health {
            continue;
        }
        let cost = config.health_regen_cost;
        if c.energy <= config.corpse_energy + cost {
            continue;
        }
        c.energy -= cost;
        ledger.record_destroy(cost);
        c.health = (c.health + config.health_regen).min(c.max_health);
    }

    let mut move_targets: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (i, think) in &pending {
        for action in &think.actions {
            if let PendingAction::Move { dir } = action {
                let c = &creatures[*i];
                if let Some((nx, ny)) = hex::neighbor(c.x, c.y, *dir) {
                    move_targets.entry((nx, ny)).or_default().push(*i);
                }
            }
        }
    }

    let mut blocked_moves: HashMap<usize, bool> = HashMap::new();
    for (cell, ids) in &move_targets {
        if ids.len() > 1 && ids.iter().any(|id| *id != ids[0]) {
            for id in ids {
                blocked_moves.insert(*id, true);
            }
            continue;
        }
        let i = ids[0];
        let c = &creatures[i];
        if blocks_movement(tiles, cell.0, cell.1) {
            blocked_moves.insert(i, true);
            continue;
        }
        if snapshot.id_at.contains_key(cell) {
            let occupant = snapshot.id_at.get(cell).unwrap();
            if occupant != &c.id {
                blocked_moves.insert(i, true);
            }
        }
    }

    let mut spawns: Vec<(usize, PendingAction)> = Vec::new();
    let mut signals: Vec<(usize, PendingAction)> = Vec::new();
    let mut eats: Vec<(usize, u8)> = Vec::new();
    let mut hits: Vec<(usize, u8)> = Vec::new();

    for (i, think) in pending {
        let c = &mut creatures[i];
        if think.suicide {
            mark_dead(c, DeathReason::Suicide);
            continue;
        }

        for action in think.actions {
            match action {
                PendingAction::Move { dir } => {
                    if *blocked_moves.get(&i).unwrap_or(&false) {
                        continue;
                    }
                    if let Some((nx, ny)) = hex::neighbor(c.x, c.y, dir) {
                        c.x = nx;
                        c.y = ny;
                    }
                }
                PendingAction::Dig { dir } => {
                    if let Some((x, y)) = hex::neighbor(c.x, c.y, dir) {
                        set_cell(tiles, x, y, tile::EMPTY);
                    }
                }
                PendingAction::Place { dir } => {
                    if let Some((x, y)) = hex::neighbor(c.x, c.y, dir) {
                        if sense_kind(tiles, x, y, false) == tile::EMPTY
                            && !snapshot.id_at.contains_key(&(x, y))
                        {
                            set_cell(tiles, x, y, tile::SOLID);
                        }
                    }
                }
                PendingAction::Eat { dir } => eats.push((i, dir)),
                PendingAction::Hit { dir } => hits.push((i, dir)),
                PendingAction::Spawn { .. } => spawns.push((i, action)),
                PendingAction::SignalTo { .. } | PendingAction::SignalBroadcast { .. } => {
                    signals.push((i, action))
                }
            }
        }
    }

    for (i, dir) in eats {
        if !creatures[i].alive {
            continue;
        }
        let Some((x, y)) = hex::neighbor(creatures[i].x, creatures[i].y, dir) else {
            continue;
        };
        match tiles.get(&(x, y)).copied() {
            Some(WorldTile::Corpse { energy, .. }) => {
                tiles.remove(&(x, y));
                creatures[i].energy += energy;
                result.events.push(WorldEvent::Eat {
                    actor_id: creatures[i].id.clone(),
                    x,
                    y,
                    energy,
                });
            }
            Some(WorldTile::EnergyNode { energy }) if energy > 0 => {
                tiles.remove(&(x, y));
                creatures[i].energy += energy;
                result.events.push(WorldEvent::Eat {
                    actor_id: creatures[i].id.clone(),
                    x,
                    y,
                    energy,
                });
            }
            _ => {}
        }
    }

    for (i, dir) in hits {
        if !creatures[i].alive {
            continue;
        }
        let Some((x, y)) = hex::neighbor(creatures[i].x, creatures[i].y, dir) else {
            continue;
        };
        if tiles.get(&(x, y)).is_some() {
            continue;
        }
        let Some(victim_idx) = creatures.iter().position(|v| v.alive && v.x == x && v.y == y) else {
            continue;
        };
        if victim_idx == i {
            continue;
        }
        creatures[victim_idx].health -= config.hit_damage;
        let victim_health = creatures[victim_idx].health;
        let victim_id = creatures[victim_idx].id.clone();
        result.events.push(WorldEvent::Hit {
            actor_id: creatures[i].id.clone(),
            victim_id,
            x,
            y,
            damage: config.hit_damage,
            victim_health,
        });
        if creatures[victim_idx].health <= 0 {
            mark_dead(&mut creatures[victim_idx], DeathReason::Killed);
            result.events.push(death_event(&creatures[victim_idx]));
        }
    }

    for (i, action) in spawns {
        let PendingAction::Spawn { dir, energy } = action else {
            continue;
        };
        let px = creatures[i].x;
        let py = creatures[i].y;
        let Some((x, y)) = hex::neighbor(px, py, dir) else {
            mark_dead(&mut creatures[i], DeathReason::SpawnFailed);
            continue;
        };
        if snapshot.id_at.contains_key(&(x, y))
            || blocks_movement(tiles, x, y)
            || sense_kind(tiles, x, y, false) != tile::EMPTY
        {
            mark_dead(&mut creatures[i], DeathReason::SpawnFailed);
            continue;
        }
        let parent_id = creatures[i].id.clone();
        let owner_uid = creatures[i].owner_uid.clone();
        let wasm = creatures[i].wasm.clone();
        let code = creatures[i].code.clone();
        if creatures[i].energy < energy {
            mark_dead(&mut creatures[i], DeathReason::OutOfEnergy);
            continue;
        }
        creatures[i].energy -= energy;
        let child_id = Uuid::new_v4().to_string();
        result.events.push(WorldEvent::Spawn {
            creature_id: child_id.clone(),
            parent_id: parent_id.clone(),
            x,
            y,
        });
        creatures.push(Creature {
            id: child_id,
            x,
            y,
            energy,
            health: config.max_health,
            max_health: config.max_health,
            owner_uid,
            parent_id: Some(parent_id),
            wasm,
            code,
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: tick,
        });
    }

    let mut deliveries: Vec<(String, Signal)> = Vec::new();
    for (i, action) in signals {
        let sender_id = creatures[i].id.clone();
        match action {
            PendingAction::SignalTo {
                to_id,
                byte,
                from_x,
                from_y,
            } => {
                let Some(&(tx, ty)) = snapshot.positions.get(&to_id) else {
                    mark_dead(&mut creatures[i], DeathReason::SignalUnknownTarget);
                    continue;
                };
                let c = &creatures[i];
                if !in_sig_range(c.x, c.y, tx, ty, config) {
                    mark_dead(&mut creatures[i], DeathReason::SignalOutOfRange);
                    continue;
                }
                let sig = Signal {
                    from_id: sender_id.clone(),
                    from_x,
                    from_y,
                    byte,
                    broadcast: false,
                };
                deliveries.push((to_id.clone(), sig));
                result.events.push(WorldEvent::Signal {
                    from_id: sender_id,
                    from_x,
                    from_y,
                    to_id: Some(to_id),
                    byte,
                    broadcast: false,
                });
            }
            PendingAction::SignalBroadcast {
                byte,
                from_x,
                from_y,
            } => {
                let sender = &creatures[i];
                let sx = sender.x;
                let sy = sender.y;
                for (cid, &(tx, ty)) in &snapshot.positions {
                    if cid == &sender.id {
                        continue;
                    }
                    if !in_sig_range(sx, sy, tx, ty, config) {
                        continue;
                    }
                    deliveries.push((
                        cid.clone(),
                        Signal {
                            from_id: sender_id.clone(),
                            from_x,
                            from_y,
                            byte,
                            broadcast: true,
                        },
                    ));
                }
                result.events.push(WorldEvent::Signal {
                    from_id: sender_id,
                    from_x,
                    from_y,
                    to_id: None,
                    byte,
                    broadcast: true,
                });
            }
            _ => {}
        }
    }

    for (to_id, sig) in deliveries {
        if let Some(c) = creatures.iter_mut().find(|c| c.id == to_id && c.alive) {
            if c.inbox.len() >= config.signal_inbox_cap {
                c.inbox.remove(0);
            }
            c.inbox.push(sig);
        }
    }

    for creature in creatures.iter() {
        if creature.alive {
            continue;
        }
        if suicide_ids.contains_key(&creature.id) {
            result.events.push(death_event(creature));
            continue;
        }
        if !result.events.iter().any(|e| {
            matches!(e, WorldEvent::Death { creature_id, .. } if creature_id == &creature.id)
        }) {
            result.events.push(death_event(creature));
        }
    }

    let mut dead: Vec<(i32, i32, i64, i64, DeathReason)> = Vec::new();
    for creature in creatures.iter() {
        if creature.alive || suicide_ids.contains_key(&creature.id) {
            continue;
        }
        let yield_energy = corpse_yield_energy(creature.energy);
        ledger.record_destroy(creature.energy.saturating_sub(yield_energy));
        dead.push((
            creature.x,
            creature.y,
            yield_energy,
            creature.energy,
            creature.death_reason.unwrap_or(DeathReason::WasmTrap),
        ));
    }

    creatures.retain(|c| c.alive);

    for (x, y, energy, _orig, reason) in dead {
        place_corpse(tiles, x, y, energy, reason);
    }

    try_spawn_nodes(
        ledger,
        tiles,
        &creatures.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>(),
        tick,
        config,
    );

    result.destroyed = ledger.destroyed - destroyed_before;
    result.free_minted = ledger.free_minted - free_before;
    result
}

fn build_snapshot(creatures: &[Creature]) -> Snapshot {
    let mut positions = HashMap::new();
    let mut id_at = HashMap::new();
    let mut energy = HashMap::new();
    let mut health = HashMap::new();
    let mut max_health = HashMap::new();
    for c in creatures.iter().filter(|c| c.alive) {
        positions.insert(c.id.clone(), (c.x, c.y));
        id_at.insert((c.x, c.y), c.id.clone());
        energy.insert(c.id.clone(), c.energy);
        health.insert(c.id.clone(), c.health);
        max_health.insert(c.id.clone(), c.max_health);
    }
    Snapshot {
        positions,
        id_at,
        energy,
        health,
        max_health,
    }
}

fn in_sig_range(sx: i32, sy: i32, tx: i32, ty: i32, config: &SimConfig) -> bool {
    crate::hex::in_range(tx - sx, ty - sy, config.r_sig)
}

fn think_once(
    engine: &wasmtime::Engine,
    creature: &mut Creature,
    snapshot: &Snapshot,
    tiles: &WorldTiles,
    config: &SimConfig,
    ledger: &mut EnergyLedger,
    tick: u64,
) -> ThinkResult {
    let Some(module) = host::cached_module(engine, &creature.wasm) else {
        mark_dead(creature, DeathReason::InvalidProgram);
        return ThinkResult::default();
    };
    host::run_creature_tick(engine, &module, creature, snapshot, tiles, config, ledger, tick)
}

pub fn adjacent(q: i32, r: i32, dir: u8) -> Option<(i32, i32)> {
    hex::neighbor(q, r, dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_wat;
    use crate::world_tile::WorldTiles;

    #[test]
    fn all_examples_compile() {
        for example in crate::EXAMPLE_PROGRAMS {
            compile_wat(example.code).unwrap_or_else(|err| {
                panic!("example `{}` failed: {err}", example.id)
            });
        }
    }

    #[test]
    fn one_tick_moves_east() {
        const CODE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (func (export "tick")
    i32.const 0
    call $move
    drop
    call $sleep)
)
"#;
        let mut creatures = vec![Creature {
            id: "a".into(),
            x: 0,
            y: 0,
            energy: 1_000_000_000,
            health: 100,
            max_health: 100,
            owner_uid: "u".into(),
            parent_id: None,
            wasm: compile_wat(CODE).unwrap(),
            code: CODE.into(),
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: 0,
        }];
        let mut tiles = WorldTiles::new();
        let mut ledger = EnergyLedger::default();
        let config = SimConfig::default();
        run_tick(&mut creatures, &mut tiles, &mut ledger, &config, 1);
        assert_eq!(creatures[0].x, 1);
    }
}
