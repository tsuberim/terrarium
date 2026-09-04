//! Creature runtime — one WASM tick per sim tick, think → act → world-tick.

use std::collections::HashMap;

use crate::abi::corpse_yield_energy;
use crate::abi::tile;
use crate::abi::{new_creature_id, Payload};
use crate::energy_ledger::EnergyLedger;
use crate::events::{DeathReason, InboxDelivery, TickResult, WorldEvent};
use crate::food::try_spawn_food;
use crate::hex;
use crate::host::{self, PendingAction, ThinkResult};
use crate::sim_config::SimConfig;
use crate::world_tile::{
    blocks_movement, mark_tile, place_corpse, sense_kind, set_cell, TileDirty, WorldTile,
    WorldTiles,
};

#[derive(Clone, Debug)]
pub struct Signal {
    pub from_id: u64,
    pub payload: Payload,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub id: u64,
    pub x: i32,
    pub y: i32,
    pub energy: i64,
    pub health: i32,
    pub max_health: i32,
    pub owner_uid: String,
    /// Owning creature id (visible in guest memory); human account follows owner chain.
    pub owner_id: u64,
    pub parent_id: Option<u64>,
    pub wasm: Vec<u8>,
    pub code: String,
    pub alive: bool,
    pub inbox: Vec<Signal>,
    pub death_reason: Option<DeathReason>,
    pub born_tick: u64,
    pub facing: u8,
    /// Passed to `main(init_ptr)` at birth (zeros on deploy).
    pub init: Payload,
}

pub(crate) fn mark_dead(creature: &mut Creature, reason: DeathReason) {
    creature.alive = false;
    if creature.death_reason.is_none() {
        creature.death_reason = Some(reason);
    }
}

fn death_event(creature: &Creature) -> WorldEvent {
    WorldEvent::Death {
        creature_id: creature.id,
        owner_uid: creature.owner_uid.clone(),
        x: creature.x,
        y: creature.y,
        reason: creature.death_reason.unwrap_or(DeathReason::WasmTrap),
        facing: creature.facing,
        energy: creature.energy,
        health: creature.health,
        max_health: creature.max_health,
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub positions: HashMap<u64, (i32, i32)>,
    pub id_at: HashMap<(i32, i32), u64>,
    pub energy: HashMap<u64, i64>,
    pub health: HashMap<u64, i32>,
    pub max_health: HashMap<u64, i32>,
    pub facing: HashMap<u64, u8>,
}

pub fn run_tick(
    creatures: &mut Vec<Creature>,
    tiles: &mut WorldTiles,
    ledger: &mut EnergyLedger,
    config: &SimConfig,
    tick: u64,
    external_actions: &mut Vec<(u64, PendingAction)>,
) -> TickResult {
    #[cfg(test)]
    if tick == 1 {
        host::clear_vm_cache();
    }
    let destroyed_before = ledger.destroyed;
    let free_before = ledger.free_minted;
    let mut result = TickResult::default();
    let mut tile_dirty = TileDirty::new();
    let snapshot = build_snapshot(creatures);
    let engine = host::wasm_engine();

    let mut pending: Vec<(usize, ThinkResult)> = Vec::new();
    let mut suicide_ids: HashMap<u64, ()> = HashMap::new();

    for (i, creature) in creatures.iter_mut().enumerate() {
        if !creature.alive {
            continue;
        }
        if creature.energy <= config.corpse_energy {
            mark_dead(creature, DeathReason::EnergyFloor);
            result.events.push(death_event(creature));
            continue;
        }
        let mut think = think_once(engine, creature, &snapshot, tiles, config, ledger, tick);
        if !creature.alive {
            result.events.push(death_event(creature));
            continue;
        }
        if !think.tick_busy() {
            if let Some(pos) = external_actions
                .iter()
                .position(|(id, _)| *id == creature.id)
            {
                let (_, action) = external_actions.remove(pos);
                think.action = Some(action);
            }
        }
        if think.suicide {
            suicide_ids.insert(creature.id, ());
        }
        pending.push((i, think));
    }

    for (i, think) in &pending {
        if think.suicide || think.action.is_some() {
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
        if let Some(PendingAction::Move { rel }) = think.action {
            let c = &creatures[*i];
            let dir = hex::abs_dir(c.facing, rel);
            if let Some(cell) = hex::neighbor(c.x, c.y, dir) {
                move_targets.entry(cell).or_default().push(*i);
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

        if let Some(action) = think.action {
            match action {
                PendingAction::Move { rel } => {
                    if *blocked_moves.get(&i).unwrap_or(&false) {
                        continue;
                    }
                    let dir = hex::abs_dir(c.facing, rel);
                    if let Some((nx, ny)) = hex::neighbor(c.x, c.y, dir) {
                        result.actions.push(crate::events::CreatureAction::Move {
                            creature_id: c.id,
                            from_x: c.x,
                            from_y: c.y,
                            to_x: nx,
                            to_y: ny,
                        });
                        c.x = nx;
                        c.y = ny;
                    }
                }
                PendingAction::Rotate { delta } => {
                    let from_facing = c.facing;
                    let to_facing = (c.facing as i32 + delta).rem_euclid(6) as u8;
                    result.actions.push(crate::events::CreatureAction::Rotate {
                        creature_id: c.id,
                        from_facing,
                        to_facing,
                    });
                    c.facing = to_facing;
                }
                PendingAction::Dig { rel } => {
                    let dir = hex::abs_dir(c.facing, rel);
                    if let Some((x, y)) = hex::neighbor(c.x, c.y, dir) {
                        set_cell(tiles, &mut tile_dirty, x, y, tile::EMPTY);
                    }
                }
                PendingAction::Place { rel } => {
                    let dir = hex::abs_dir(c.facing, rel);
                    if let Some((x, y)) = hex::neighbor(c.x, c.y, dir) {
                        if sense_kind(tiles, x, y, false) == tile::EMPTY
                            && !snapshot.id_at.contains_key(&(x, y))
                        {
                            set_cell(tiles, &mut tile_dirty, x, y, tile::SOLID);
                        }
                    }
                }
                PendingAction::Eat { rel } => eats.push((i, rel)),
                PendingAction::Hit { rel } => hits.push((i, rel)),
                PendingAction::Spawn { .. } => spawns.push((i, action)),
                PendingAction::Signal { .. } | PendingAction::Broadcast { .. } => {
                    signals.push((i, action))
                }
            }
        }
    }

    for (i, rel) in eats {
        if !creatures[i].alive {
            continue;
        }
        let dir = hex::abs_dir(creatures[i].facing, rel);
        let Some((x, y)) = hex::neighbor(creatures[i].x, creatures[i].y, dir) else {
            continue;
        };
        match tiles.get(&(x, y)).copied() {
            Some(WorldTile::Corpse { energy, .. }) => {
                mark_tile(&mut tile_dirty, x, y);
                tiles.remove(&(x, y));
                creatures[i].energy += energy;
                result.actions.push(crate::events::CreatureAction::Eat {
                    creature_id: creatures[i].id,
                    x,
                    y,
                });
                result.events.push(WorldEvent::Eat {
                    actor_id: creatures[i].id,
                    x,
                    y,
                    energy,
                    tile_kind: tile::CORPSE as i64,
                });
            }
            Some(WorldTile::Food { energy }) if energy > 0 => {
                mark_tile(&mut tile_dirty, x, y);
                tiles.remove(&(x, y));
                creatures[i].energy += energy;
                result.actions.push(crate::events::CreatureAction::Eat {
                    creature_id: creatures[i].id,
                    x,
                    y,
                });
                result.events.push(WorldEvent::Eat {
                    actor_id: creatures[i].id,
                    x,
                    y,
                    energy,
                    tile_kind: tile::FOOD as i64,
                });
            }
            _ => {}
        }
    }

    for (i, rel) in hits {
        if !creatures[i].alive {
            continue;
        }
        let dir = hex::abs_dir(creatures[i].facing, rel);
        let Some((x, y)) = hex::neighbor(creatures[i].x, creatures[i].y, dir) else {
            continue;
        };
        if tiles.get(&(x, y)).is_some() {
            continue;
        }
        let Some(victim_idx) = creatures
            .iter()
            .position(|v| v.alive && v.x == x && v.y == y)
        else {
            continue;
        };
        if victim_idx == i {
            continue;
        }
        creatures[victim_idx].health -= config.hit_damage;
        let victim_health = creatures[victim_idx].health;
        let victim_id = creatures[victim_idx].id;
        result.actions.push(crate::events::CreatureAction::Hit {
            creature_id: creatures[i].id,
            x,
            y,
        });
        result.events.push(WorldEvent::Hit {
            actor_id: creatures[i].id,
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
        let PendingAction::Spawn {
            rel,
            energy,
            owner_id,
            child_init,
        } = action
        else {
            continue;
        };
        let px = creatures[i].x;
        let py = creatures[i].y;
        let dir = hex::abs_dir(creatures[i].facing, rel);
        let Some((x, y)) = hex::neighbor(px, py, dir) else {
            continue;
        };
        if snapshot.id_at.contains_key(&(x, y))
            || blocks_movement(tiles, x, y)
            || sense_kind(tiles, x, y, false) != tile::EMPTY
        {
            continue;
        }
        let Some(owner_creature) = creatures.iter().find(|c| c.alive && c.id == owner_id) else {
            continue;
        };
        let parent_id = creatures[i].id;
        let parent_x = creatures[i].x;
        let parent_y = creatures[i].y;
        let owner_uid = owner_creature.owner_uid.clone();
        let wasm = creatures[i].wasm.clone();
        let code = creatures[i].code.clone();
        if creatures[i].energy < energy {
            mark_dead(&mut creatures[i], DeathReason::OutOfEnergy);
            continue;
        }
        creatures[i].energy -= energy;
        let child_id = new_creature_id();
        result.events.push(WorldEvent::Spawn {
            creature_id: child_id,
            parent_id,
            parent_x,
            parent_y,
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
            owner_id,
            parent_id: Some(parent_id),
            wasm,
            code,
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: tick,
            facing: creatures[i].facing,
            init: child_init,
        });
    }

    let mut deliveries: Vec<(u64, Signal)> = Vec::new();
    for (i, action) in signals {
        let sender_id = creatures[i].id;
        match action {
            PendingAction::Signal { target, payload } => {
                let Some(&(tx, ty)) = snapshot.positions.get(&target) else {
                    mark_dead(&mut creatures[i], DeathReason::SignalUnknownTarget);
                    continue;
                };
                let c = &creatures[i];
                if !in_sig_range(c.x, c.y, tx, ty, config) {
                    mark_dead(&mut creatures[i], DeathReason::SignalOutOfRange);
                    continue;
                }
                let sig = Signal {
                    from_id: sender_id,
                    payload,
                };
                deliveries.push((target, sig));
                result.events.push(WorldEvent::Signal {
                    from_id: sender_id,
                    from_x: c.x,
                    from_y: c.y,
                    to_id: Some(target),
                    broadcast: false,
                });
            }
            PendingAction::Broadcast { payload } => {
                let sender = &creatures[i];
                let sx = sender.x;
                let sy = sender.y;
                for (cid, &(tx, ty)) in &snapshot.positions {
                    if *cid == sender.id {
                        continue;
                    }
                    if !in_sig_range(sx, sy, tx, ty, config) {
                        continue;
                    }
                    deliveries.push((
                        *cid,
                        Signal {
                            from_id: sender_id,
                            payload,
                        },
                    ));
                }
                result.events.push(WorldEvent::Signal {
                    from_id: sender_id,
                    from_x: sx,
                    from_y: sy,
                    to_id: None,
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
            result.inbox_deliveries.push(InboxDelivery {
                creature_id: to_id,
                sender: sig.from_id,
                payload: sig.payload,
            });
            c.inbox.push(sig);
        }
    }

    let suicide_payouts: Vec<(SuicideRecipient, i64, String)> = creatures
        .iter()
        .filter(|c| suicide_ids.contains_key(&c.id) && c.energy > 0)
        .map(|c| {
            (
                resolve_suicide_recipient(c, creatures),
                c.energy,
                c.owner_uid.clone(),
            )
        })
        .collect();

    for (recipient, energy, owner_uid) in suicide_payouts {
        match recipient {
            SuicideRecipient::Creature(parent_id) => {
                if let Some(parent) = creatures.iter_mut().find(|c| c.id == parent_id && c.alive) {
                    parent.energy += energy;
                } else {
                    result.credit_payouts.push((owner_uid, energy));
                }
            }
            SuicideRecipient::Account(uid) => {
                result.credit_payouts.push((uid, energy));
            }
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
        if !result.events.iter().any(
            |e| matches!(e, WorldEvent::Death { creature_id, .. } if creature_id == &creature.id),
        ) {
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
        place_corpse(tiles, &mut tile_dirty, x, y, energy, reason);
    }

    try_spawn_food(
        ledger,
        tiles,
        &mut tile_dirty,
        &creatures.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>(),
        tick,
        config,
    );

    result.destroyed = ledger.destroyed - destroyed_before;
    result.free_minted = ledger.free_minted - free_before;
    result.tiles_dirty = tile_dirty;
    result
}

fn build_snapshot(creatures: &[Creature]) -> Snapshot {
    let mut positions = HashMap::new();
    let mut id_at = HashMap::new();
    let mut energy = HashMap::new();
    let mut health = HashMap::new();
    let mut max_health = HashMap::new();
    let mut facing = HashMap::new();
    for c in creatures.iter().filter(|c| c.alive) {
        positions.insert(c.id, (c.x, c.y));
        id_at.insert((c.x, c.y), c.id);
        energy.insert(c.id, c.energy);
        health.insert(c.id, c.health);
        max_health.insert(c.id, c.max_health);
        facing.insert(c.id, c.facing);
    }
    Snapshot {
        positions,
        id_at,
        energy,
        health,
        max_health,
        facing,
    }
}

fn in_sig_range(sx: i32, sy: i32, tx: i32, ty: i32, config: &SimConfig) -> bool {
    crate::hex::in_range(tx - sx, ty - sy, config.r_sig)
}

/// Best-effort validation for external control inject (does not kill creature).
pub fn validate_external_signal(
    creatures: &[Creature],
    from_id: u64,
    target: u64,
    config: &SimConfig,
) -> Result<(), &'static str> {
    let sender = creatures
        .iter()
        .find(|c| c.alive && c.id == from_id)
        .ok_or("creature_unavailable")?;
    let snapshot = build_snapshot(creatures);
    let Some(&(tx, ty)) = snapshot.positions.get(&target) else {
        return Err("unknown_target");
    };
    if !in_sig_range(sender.x, sender.y, tx, ty, config) {
        return Err("out_of_range");
    }
    Ok(())
}

pub fn validate_external_broadcast(
    creatures: &[Creature],
    from_id: u64,
) -> Result<(), &'static str> {
    if creatures.iter().any(|c| c.alive && c.id == from_id) {
        Ok(())
    } else {
        Err("creature_unavailable")
    }
}

enum SuicideRecipient {
    Creature(u64),
    Account(String),
}

fn resolve_suicide_recipient(creature: &Creature, creatures: &[Creature]) -> SuicideRecipient {
    let mut parent = creature.parent_id;
    while let Some(id) = parent {
        if creatures.iter().any(|c| c.id == id && c.alive) {
            return SuicideRecipient::Creature(id);
        }
        parent = creatures
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.parent_id);
    }
    SuicideRecipient::Account(creature.owner_uid.clone())
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
    host::run_creature_tick(
        engine, &module, creature, snapshot, tiles, config, ledger, tick,
    )
}

pub fn adjacent(q: i32, r: i32, dir: u8) -> Option<(i32, i32)> {
    hex::neighbor(q, r, dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_wat;
    use crate::wat::{wat_move_once, WAT_MOVE_FWD_LOOP};
    use crate::world_tile::WorldTiles;

    fn test_creature(code: &str, facing: u8) -> Creature {
        Creature {
            id: 1,
            x: 0,
            y: 0,
            energy: 1_000_000_000,
            health: 100,
            max_health: 100,
            owner_uid: "u".into(),
            owner_id: 1,
            parent_id: None,
            wasm: compile_wat(code).unwrap(),
            code: code.into(),
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: 0,
            facing,
            init: Payload::default(),
        }
    }

    #[test]
    fn one_tick_moves_east() {
        let mut creatures = vec![test_creature(WAT_MOVE_FWD_LOOP, 0)];
        let mut tiles = WorldTiles::new();
        let mut ledger = EnergyLedger::default();
        let config = SimConfig::default();
        run_tick(
            &mut creatures,
            &mut tiles,
            &mut ledger,
            &config,
            1,
            &mut vec![],
        );
        assert_eq!(creatures[0].x, 1);
        assert_eq!(creatures[0].facing, 0);
    }

    #[test]
    fn move_forward_is_relative_to_facing() {
        let mut creatures = vec![test_creature(WAT_MOVE_FWD_LOOP, 2)];
        let mut tiles = WorldTiles::new();
        let mut ledger = EnergyLedger::default();
        let config = SimConfig::default();
        run_tick(
            &mut creatures,
            &mut tiles,
            &mut ledger,
            &config,
            1,
            &mut vec![],
        );
        assert_eq!(creatures[0].x, 0);
        assert_eq!(creatures[0].y, -1);
        assert_eq!(creatures[0].facing, 2);
    }

    #[test]
    fn move_non_forward_traps() {
        let mut creatures = vec![test_creature(&wat_move_once(6), 0)];
        let mut tiles = WorldTiles::new();
        let mut ledger = EnergyLedger::default();
        let config = SimConfig::default();
        run_tick(
            &mut creatures,
            &mut tiles,
            &mut ledger,
            &config,
            1,
            &mut vec![],
        );
        assert!(creatures.is_empty());
    }
}
