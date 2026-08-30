use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::Serialize;
use sqlx::SqlitePool;
use terrarium_kernel::{
    compile_wat, food, run_tick, vm::Creature, DeathReason, EnergyLedger, SimConfig,
    WorldEvent, WorldTile, WorldTiles, EXAMPLE_PROGRAMS, TICK_HZ,
};
use tokio::sync::broadcast;

use crate::config::Config;

const DEFAULT_PERSIST_EVERY: u64 = 10;
const BROADCAST_CAPACITY: usize = 4096;
const SEED_OWNER: &str = "__terrarium__";
/// Starting energy for bootstrap creatures (~120 glims); above clone threshold.
const SEED_ENERGY: i64 = 12_000_000;

struct SeedCreature {
    example_id: &'static str,
    id: &'static str,
    x: i32,
    y: i32,
}

const ECOSYSTEM_SEED: &[SeedCreature] = &[
    SeedCreature { example_id: "predator", id: "seed-predator-0", x: 3, y: 0 },
    SeedCreature { example_id: "predator", id: "seed-predator-1", x: -3, y: 0 },
    SeedCreature { example_id: "prey", id: "seed-prey-0", x: 0, y: 3 },
    SeedCreature { example_id: "prey", id: "seed-prey-1", x: 0, y: -3 },
    SeedCreature { example_id: "prey", id: "seed-prey-2", x: 2, y: 2 },
    SeedCreature { example_id: "prey", id: "seed-prey-3", x: -2, y: -2 },
    SeedCreature { example_id: "scavenger", id: "seed-scavenger-0", x: 4, y: -2 },
    SeedCreature { example_id: "scavenger", id: "seed-scavenger-1", x: -4, y: 2 },
    SeedCreature { example_id: "hawk", id: "seed-hawk-0", x: 4, y: 2 },
    SeedCreature { example_id: "hawk", id: "seed-hawk-1", x: -4, y: -2 },
];

#[derive(Clone, Serialize)]
pub struct CreaturePublic {
    pub id: String,
    pub x: i64,
    pub y: i64,
    pub energy: i64,
    pub health: i32,
    pub max_health: i32,
    pub owner_uid: String,
    /// Body facing 0–5 (E, NE, NW, W, SW, SE).
    pub facing: u8,
    /// WASM digest — same program shares the same hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct TilePublic {
    pub x: i64,
    pub y: i64,
    pub kind: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_reason: Option<DeathReason>,
}

#[derive(Clone, Serialize)]
pub struct EnergyLedgerPublic {
    pub destroyed: i64,
    pub free_minted: i64,
    pub free_budget: i64,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldMessage {
    Delta {
        tick: u64,
        /// Full state sync (connect / lag recovery): all entities as upserts, meta populated.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        full: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        deploy_cost: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        corpse_energy: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sim_config: Option<SimConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        energy_ledger: Option<EnergyLedgerPublic>,
        creatures_upsert: Vec<CreaturePublic>,
        creatures_remove: Vec<String>,
        tiles_upsert: Vec<TilePublic>,
        tiles_remove: Vec<[i64; 2]>,
        actions: Vec<terrarium_kernel::CreatureAction>,
        events: Vec<WorldEvent>,
    },
}

struct PersistSnapshot {
    creatures: Vec<Creature>,
    tiles: WorldTiles,
    ledger: EnergyLedger,
}

struct TickStep {
    message: WorldMessage,
    persist: Option<PersistSnapshot>,
    credit_payouts: Vec<(String, i64)>,
}
struct SimState {
    tick: u64,
    creatures: Vec<Creature>,
    tiles: WorldTiles,
    ledger: EnergyLedger,
}

pub struct WorldEngine {
    inner: RwLock<SimState>,
    pub db: SqlitePool,
    deploy_cost: i64,
    persist_every: u64,
    sim_config: RwLock<SimConfig>,
    events: broadcast::Sender<WorldMessage>,
}

impl WorldEngine {
    pub async fn bootstrap(db: SqlitePool, config: &Config) -> anyhow::Result<Arc<Self>> {
        let mut creatures = load_creatures(&db).await?;
        let mut tiles = load_tiles(&db).await?;
        let mut ledger = load_ledger(&db).await?;
        let sim_config = SimConfig::default();

        if creatures.is_empty() && config.seed_ecosystem {
            // Recover from a partial seed (e.g. crash mid-bootstrap) — tiles can
            // remain while creatures were never committed.
            if !tiles.is_empty() {
                sqlx::query("DELETE FROM world_tiles").execute(&db).await?;
                tiles.clear();
            }
            seed_ecosystem(&db, &sim_config, &mut creatures, &mut tiles, &mut ledger).await?;
        }

        let (events, _) = broadcast::channel(BROADCAST_CAPACITY);
        let persist_every = std::env::var("PERSIST_EVERY_TICKS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PERSIST_EVERY);

        Ok(Arc::new(Self {
            inner: RwLock::new(SimState {
                tick: 0,
                creatures,
                tiles,
                ledger,
            }),
            db,
            deploy_cost: config.deploy_cost,
            persist_every,
            sim_config: RwLock::new(SimConfig::default()),
            events,
        }))
    }

    pub fn sim_config(&self) -> SimConfig {
        self.sim_config.read().clone()
    }

    pub fn set_sim_config(&self, config: SimConfig) {
        *self.sim_config.write() = config;
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WorldMessage> {
        self.events.subscribe()
    }

    pub fn deploy_cost(&self) -> i64 {
        self.deploy_cost
    }

    pub fn current_tick(&self) -> u64 {
        self.inner.read().tick
    }

    pub fn public_creatures(&self) -> Vec<CreaturePublic> {
        self.inner
            .read()
            .creatures
            .iter()
            .map(creature_public)
            .collect()
    }

    pub fn world_http(&self) -> (i64, i64, Vec<CreaturePublic>, Vec<TilePublic>) {
        let state = self.inner.read();
        (
            self.deploy_cost,
            self.sim_config.read().corpse_energy,
            state.creatures.iter().map(creature_public).collect(),
            tiles_public(&state.tiles),
        )
    }

    pub fn full_delta(&self) -> WorldMessage {
        let state = self.inner.read();
        let config = self.sim_config.read().clone();
        WorldMessage::Delta {
            tick: state.tick,
            full: true,
            deploy_cost: Some(self.deploy_cost),
            corpse_energy: Some(config.corpse_energy),
            sim_config: Some(config),
            energy_ledger: Some(ledger_public(&state.ledger)),
            creatures_upsert: state.creatures.iter().map(creature_public).collect(),
            creatures_remove: vec![],
            tiles_upsert: tiles_public(&state.tiles),
            tiles_remove: vec![],
            actions: vec![],
            events: vec![],
        }
    }

    pub fn is_deployable(&self, x: i64, y: i64) -> bool {
        let state = self.inner.read();
        let x = x as i32;
        let y = y as i32;
        if state.creatures.iter().any(|c| c.x == x && c.y == y) {
            return false;
        }
        !matches!(state.tiles.get(&(x, y)), Some(WorldTile::Solid))
    }

    pub async fn clear_world(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM creatures")
            .execute(&self.db)
            .await?;
        sqlx::query("DELETE FROM world_tiles")
            .execute(&self.db)
            .await?;
        sqlx::query(
            "INSERT INTO energy_ledger (id, destroyed, free_minted) VALUES (1, 0, 0)
             ON CONFLICT(id) DO UPDATE SET destroyed = 0, free_minted = 0",
        )
        .execute(&self.db)
        .await?;
        let mut state = self.inner.write();
        state.creatures.clear();
        state.tiles.clear();
        state.ledger = EnergyLedger::default();
        drop(state);
        let _ = self.events.send(self.full_delta());
        Ok(())
    }

    pub fn insert_creature(&self, creature: Creature) -> anyhow::Result<()> {
        let public = creature_public(&creature);
        let mut state = self.inner.write();
        state.creatures.push(creature);
        let msg = WorldMessage::Delta {
            tick: state.tick,
            full: false,
            deploy_cost: None,
            corpse_energy: None,
            sim_config: None,
            energy_ledger: None,
            creatures_upsert: vec![public],
            creatures_remove: vec![],
            tiles_upsert: vec![],
            tiles_remove: vec![],
            actions: vec![],
            events: vec![],
        };
        drop(state);
        let _ = self.events.send(msg);
        Ok(())
    }

    pub fn tick_step(&self) -> TickStep {
        let mut state = self.inner.write();
        let tick = state.tick + 1;

        let before: HashMap<String, (i32, i32, i64, i32, i32, u8)> = state
            .creatures
            .iter()
            .map(|c| {
                (
                    c.id.clone(),
                    (c.x, c.y, c.energy, c.health, c.max_health, c.facing),
                )
            })
            .collect();
        let before_tiles = state.tiles.clone();

        let mut creatures = std::mem::take(&mut state.creatures);
        let mut tiles = std::mem::take(&mut state.tiles);
        let mut ledger = state.ledger.clone();
        let config = self.sim_config.read().clone();
        let tick_result = run_tick(&mut creatures, &mut tiles, &mut ledger, &config, tick);
        state.creatures = creatures;
        state.tiles = tiles;
        state.ledger = ledger;
        state.tick = tick;

        let mut message = build_delta(tick, &before, &state.creatures, &before_tiles, &state.tiles);
        if let WorldMessage::Delta { events, actions, .. } = &mut message {
            *events = tick_result.events;
            *actions = tick_result.actions;
        }
        let persist = if tick % self.persist_every == 0 {
            Some(PersistSnapshot {
                creatures: state.creatures.clone(),
                tiles: state.tiles.clone(),
                ledger: state.ledger.clone(),
            })
        } else {
            None
        };
        drop(state);

        TickStep {
            message,
            persist,
            credit_payouts: tick_result.credit_payouts,
        }
    }
}

fn build_delta(
    tick: u64,
    before: &HashMap<String, (i32, i32, i64, i32, i32, u8)>,
    after: &[Creature],
    before_tiles: &WorldTiles,
    after_tiles: &WorldTiles,
) -> WorldMessage {
    let after_ids: HashSet<&str> = after.iter().map(|c| c.id.as_str()).collect();

    let mut creatures_remove = Vec::new();
    for id in before.keys() {
        if !after_ids.contains(id.as_str()) {
            creatures_remove.push(id.clone());
        }
    }

    let mut creatures_upsert = Vec::new();
    for c in after {
        let changed = before
            .get(&c.id)
            .map(|(x, y, e, h, mh, f)| {
                *x != c.x
                    || *y != c.y
                    || *e != c.energy
                    || *h != c.health
                    || *mh != c.max_health
                    || *f != c.facing
            })
            .unwrap_or(true);
        if changed {
            creatures_upsert.push(creature_public(c));
        }
    }

    let mut tiles_upsert = Vec::new();
    let mut tiles_remove = Vec::new();

    for (pos, tile) in after_tiles {
        if before_tiles.get(pos) != Some(tile) {
            tiles_upsert.push(tile_public(*pos, *tile));
        }
    }
    for pos in before_tiles.keys() {
        if !after_tiles.contains_key(pos) {
            tiles_remove.push([pos.0 as i64, pos.1 as i64]);
        }
    }

    WorldMessage::Delta {
        tick,
        full: false,
        deploy_cost: None,
        corpse_energy: None,
        sim_config: None,
        energy_ledger: None,
        creatures_upsert,
        creatures_remove,
        tiles_upsert,
        tiles_remove,
        actions: vec![],
        events: vec![],
    }
}

fn program_hash(wasm: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(wasm);
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

fn creature_public(c: &Creature) -> CreaturePublic {
    CreaturePublic {
        id: c.id.clone(),
        x: c.x as i64,
        y: c.y as i64,
        energy: c.energy,
        health: c.health,
        max_health: c.max_health,
        owner_uid: c.owner_uid.clone(),
        facing: c.facing,
        program_hash: Some(program_hash(&c.wasm)),
    }
}

fn tile_public(pos: (i32, i32), tile: WorldTile) -> TilePublic {
    match tile {
        WorldTile::Solid => TilePublic {
            x: pos.0 as i64,
            y: pos.1 as i64,
            kind: 1,
            energy: None,
            death_reason: None,
        },
        WorldTile::Corpse {
            energy,
            death_reason,
        } => TilePublic {
            x: pos.0 as i64,
            y: pos.1 as i64,
            kind: 3,
            energy: Some(energy),
            death_reason: Some(death_reason),
        },
        WorldTile::Food { energy } => TilePublic {
            x: pos.0 as i64,
            y: pos.1 as i64,
            kind: 4,
            energy: Some(energy),
            death_reason: None,
        },
    }
}

fn ledger_public(ledger: &EnergyLedger) -> EnergyLedgerPublic {
    EnergyLedgerPublic {
        destroyed: ledger.destroyed,
        free_minted: ledger.free_minted,
        free_budget: ledger.free_budget(),
    }
}

fn tiles_public(tiles: &WorldTiles) -> Vec<TilePublic> {
    let mut out: Vec<_> = tiles
        .iter()
        .map(|(pos, tile)| tile_public(*pos, *tile))
        .collect();
    out.sort_by_key(|t| (t.x, t.y));
    out
}

const IDLE_WAT: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (func (export "tick") (call $sleep))
)
"#;

async fn load_creatures(db: &SqlitePool) -> anyhow::Result<Vec<Creature>> {
    let rows = sqlx::query_as::<_, (String, String, i64, i64, i64, i64, i64, Option<Vec<u8>>, String, i64, i64)>(
        "SELECT id, owner_uid, x, y, energy, health, max_health, bytecode, code, born_tick, facing FROM creatures ORDER BY id",
    )
    .fetch_all(db)
    .await?;

    let idle = compile_wat(IDLE_WAT)?;

    rows.into_iter()
        .map(
            |(id, owner_uid, x, y, energy, health, max_health, wasm_blob, code, born_tick, facing)| {
            let wasm = match wasm_blob.filter(|b| !b.is_empty()) {
                Some(b) => b,
                None => compile_wat(&code).unwrap_or_else(|_| idle.clone()),
            };
            Ok(Creature {
                id,
                owner_uid,
                x: x as i32,
                y: y as i32,
                energy,
                health: health as i32,
                max_health: max_health as i32,
                parent_id: None,
                wasm,
                code,
                alive: true,
                inbox: vec![],
                death_reason: None,
                born_tick: born_tick.max(0) as u64,
                facing: (facing as u8).min(5),
            })
        },
        )
        .collect()
}

async fn load_ledger(db: &SqlitePool) -> anyhow::Result<EnergyLedger> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        "SELECT destroyed, free_minted FROM energy_ledger WHERE id = 1",
    )
    .fetch_optional(db)
    .await?;

    Ok(match row {
        Some((destroyed, free_minted)) => EnergyLedger {
            destroyed,
            free_minted,
        },
        None => EnergyLedger::default(),
    })
}

async fn seed_ecosystem(
    db: &SqlitePool,
    sim_config: &SimConfig,
    creatures: &mut Vec<Creature>,
    tiles: &mut WorldTiles,
    ledger: &mut EnergyLedger,
) -> anyhow::Result<()> {
    if ledger.destroyed == 0 && ledger.free_minted == 0 {
        ledger.destroyed = EnergyLedger::BOOTSTRAP_DESTROYED;
        sqlx::query(
            "INSERT INTO energy_ledger (id, destroyed, free_minted) VALUES (1, ?, 0)
             ON CONFLICT(id) DO UPDATE SET destroyed = excluded.destroyed",
        )
        .bind(ledger.destroyed)
        .execute(db)
        .await?;
    }

    sqlx::query(
        "INSERT INTO accounts (firebase_uid, credits) VALUES (?, 0) ON CONFLICT DO NOTHING",
    )
    .bind(SEED_OWNER)
    .execute(db)
    .await?;

    let max_health = sim_config.max_health;
    for spec in ECOSYSTEM_SEED {
        let example = EXAMPLE_PROGRAMS
            .iter()
            .find(|e| e.id == spec.example_id)
            .ok_or_else(|| anyhow::anyhow!("missing example {}", spec.example_id))?;
        let wasm = compile_wat(example.code)?;
        let code = example.code.to_string();

        sqlx::query(
            "INSERT INTO creatures (id, owner_uid, x, y, energy, health, max_health, code, bytecode, born_tick, facing, pc, stack)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, x'')",
        )
        .bind(spec.id)
        .bind(SEED_OWNER)
        .bind(spec.x as i64)
        .bind(spec.y as i64)
        .bind(SEED_ENERGY)
        .bind(max_health as i64)
        .bind(max_health as i64)
        .bind(&code)
        .bind(&wasm)
        .execute(db)
        .await?;

        creatures.push(Creature {
            id: spec.id.into(),
            owner_uid: SEED_OWNER.into(),
            x: spec.x,
            y: spec.y,
            energy: SEED_ENERGY,
            health: max_health,
            max_health,
            parent_id: None,
            wasm,
            code,
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: 0,
            facing: 0,
        });
    }

    let occupied: Vec<(i32, i32)> = creatures.iter().map(|c| (c.x, c.y)).collect();
    for tick in (0..sim_config.food_spawn_interval * 4).step_by(sim_config.food_spawn_interval as usize) {
        food::try_spawn_food(ledger, tiles, &occupied, tick, sim_config);
    }

    for ((x, y), tile) in tiles.iter() {
        if let WorldTile::Food { energy } = tile {
            sqlx::query(
                "INSERT INTO world_tiles (x, y, kind, energy, death_reason) VALUES (?, ?, 4, ?, NULL)
                 ON CONFLICT(x, y) DO UPDATE SET kind = excluded.kind, energy = excluded.energy, death_reason = excluded.death_reason",
            )
            .bind(*x as i64)
            .bind(*y as i64)
            .bind(*energy)
            .execute(db)
            .await?;
        }
    }

    sqlx::query(
        "INSERT INTO energy_ledger (id, destroyed, free_minted) VALUES (1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET destroyed = excluded.destroyed, free_minted = excluded.free_minted",
    )
    .bind(ledger.destroyed)
    .bind(ledger.free_minted)
    .execute(db)
    .await?;

    Ok(())
}

async fn load_tiles(db: &SqlitePool) -> anyhow::Result<WorldTiles> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, Option<i64>, Option<String>)>(
        "SELECT x, y, kind, energy, death_reason FROM world_tiles",
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(x, y, kind, energy, death_reason)| {
            let pos = (x as i32, y as i32);
            Some((
                pos,
                match kind {
                    1 => WorldTile::Solid,
                    3 => WorldTile::Corpse {
                        energy: energy.unwrap_or(0),
                        death_reason: death_reason
                            .as_deref()
                            .and_then(DeathReason::parse)
                            .unwrap_or(DeathReason::WasmTrap),
                    },
                    4 => WorldTile::Food {
                        energy: energy.unwrap_or(0),
                    },
                    _ => return None,
                },
            ))
        })
        .collect())
}

pub async fn persist_world(
    db: &SqlitePool,
    creatures: &[Creature],
    tiles: &WorldTiles,
    ledger: &EnergyLedger,
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    let alive_ids: HashSet<String> = creatures.iter().map(|c| c.id.clone()).collect();
    let db_ids = sqlx::query_scalar::<_, String>("SELECT id FROM creatures")
        .fetch_all(&mut *tx)
        .await?;

    for id in db_ids {
        if !alive_ids.contains(&id) {
            sqlx::query("DELETE FROM creatures WHERE id = ?")
                .bind(&id)
                .execute(&mut *tx)
                .await?;
        }
    }

    for (i, creature) in creatures.iter().enumerate() {
        let slot = -(i as i64 + 1);
        sqlx::query("UPDATE creatures SET x = ?, y = ? WHERE id = ?")
            .bind(slot)
            .bind(slot)
            .bind(&creature.id)
            .execute(&mut *tx)
            .await?;
    }

    for creature in creatures {
        sqlx::query(
            "UPDATE creatures SET x = ?, y = ?, energy = ?, health = ?, max_health = ?, facing = ?, bytecode = COALESCE(bytecode, ?), born_tick = ? WHERE id = ?",
        )
        .bind(creature.x as i64)
        .bind(creature.y as i64)
        .bind(creature.energy)
        .bind(creature.health as i64)
        .bind(creature.max_health as i64)
        .bind(creature.facing as i64)
        .bind(&creature.wasm)
        .bind(creature.born_tick as i64)
        .bind(&creature.id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("DELETE FROM world_tiles").execute(&mut *tx).await?;
    for ((x, y), tile) in tiles {
        match tile {
            WorldTile::Solid => {
                sqlx::query("INSERT INTO world_tiles (x, y, kind, energy, death_reason) VALUES (?, ?, 1, NULL, NULL)")
                    .bind(*x as i64)
                    .bind(*y as i64)
                    .execute(&mut *tx)
                    .await?;
            }
            WorldTile::Corpse {
                energy,
                death_reason,
            } => {
                sqlx::query(
                    "INSERT INTO world_tiles (x, y, kind, energy, death_reason) VALUES (?, ?, 3, ?, ?)",
                )
                .bind(*x as i64)
                .bind(*y as i64)
                .bind(*energy)
                .bind(death_reason.as_str())
                .execute(&mut *tx)
                .await?;
            }
            WorldTile::Food { energy } => {
                sqlx::query(
                    "INSERT INTO world_tiles (x, y, kind, energy, death_reason) VALUES (?, ?, 4, ?, NULL)",
                )
                .bind(*x as i64)
                .bind(*y as i64)
                .bind(*energy)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    sqlx::query(
        "INSERT INTO energy_ledger (id, destroyed, free_minted) VALUES (1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET destroyed = excluded.destroyed, free_minted = excluded.free_minted",
    )
    .bind(ledger.destroyed)
    .bind(ledger.free_minted)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn credit_payout(db: &SqlitePool, uid: &str, amount: i64) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO accounts (firebase_uid, credits) VALUES (?, 0) ON CONFLICT DO NOTHING",
    )
    .bind(uid)
    .execute(db)
    .await?;
    sqlx::query("UPDATE accounts SET credits = credits + ? WHERE firebase_uid = ?")
        .bind(amount)
        .bind(uid)
        .execute(db)
        .await?;
    Ok(())
}

pub fn spawn_tick_loop(engine: Arc<WorldEngine>) {
    let (persist_tx, mut persist_rx) = tokio::sync::mpsc::channel::<PersistSnapshot>(2);
    let (credit_tx, mut credit_rx) = tokio::sync::mpsc::channel::<(String, i64)>(32);
    let db = engine.db.clone();
    let db_persist = db.clone();

    tokio::spawn(async move {
        while let Some(snapshot) = persist_rx.recv().await {
            if let Err(err) = persist_world(&db_persist, &snapshot.creatures, &snapshot.tiles, &snapshot.ledger).await {
                tracing::error!(error = %err, "checkpoint failed");
            }
        }
    });

    tokio::spawn(async move {
        while let Some((uid, amount)) = credit_rx.recv().await {
            if let Err(err) = credit_payout(&db, &uid, amount).await {
                tracing::error!(error = %err, uid = %uid, amount, "credit payout failed");
            }
        }
    });

    std::thread::Builder::new()
        .name("terrarium-sim".into())
        .spawn(move || {
            let period = Duration::from_micros(1_000_000 / TICK_HZ as u64);
            loop {
                let start = Instant::now();

                let step = engine.tick_step();
                if engine.events.send(step.message).is_err() {
                    tracing::debug!("no world subscribers");
                }
                for (uid, amount) in step.credit_payouts {
                    if credit_tx.blocking_send((uid, amount)).is_err() {
                        tracing::error!("credit payout queue closed");
                    }
                }
                if let Some(snapshot) = step.persist {
                    if persist_tx.blocking_send(snapshot).is_err() {
                        tracing::error!("persist queue closed");
                    }
                }

                let elapsed = start.elapsed();
                if elapsed > period {
                    tracing::warn!(
                        overrun_us = elapsed.as_micros(),
                        budget_us = period.as_micros(),
                        "sim tick exceeded budget"
                    );
                }
                std::thread::sleep(period.saturating_sub(elapsed));
            }
        })
        .expect("spawn sim thread");
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use terrarium_kernel::{compile_wat, vm::Creature};

    use super::*;
    use crate::config::Config;

    async fn test_engine() -> Arc<WorldEngine> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        WorldEngine::bootstrap(
            pool,
            &Config {
                listen_addr: "127.0.0.1:0".into(),
                database_url: "sqlite::memory:".into(),
                firebase_project_id: "test".into(),
                faucet_enabled: true,
                faucet_max: 100 * terrarium_kernel::ENERGY_SCALE,
                deploy_cost: 100 * terrarium_kernel::ENERGY_SCALE,
                seed_ecosystem: false,
            },
        )
        .await
        .unwrap()
    }

    async fn seed_creature(
        engine: &WorldEngine,
        id: &str,
        x: i32,
        y: i32,
        code: &str,
        energy: i64,
    ) {
        let wasm = compile_wat(code).unwrap();
        sqlx::query("INSERT INTO accounts (firebase_uid, credits) VALUES ('tester', 1000) ON CONFLICT DO NOTHING")
            .execute(&engine.db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO creatures (id, owner_uid, x, y, energy, health, max_health, code, bytecode, pc, stack) VALUES (?, 'tester', ?, ?, ?, 100, 100, ?, ?, 0, x'')",
        )
        .bind(id)
        .bind(x as i64)
        .bind(y as i64)
        .bind(energy)
        .bind(code)
        .bind(&wasm)
        .execute(&engine.db)
        .await
        .unwrap();
        engine
            .insert_creature(Creature {
                id: id.into(),
                owner_uid: "tester".into(),
                x,
                y,
                energy,
                health: 100,
                max_health: 100,
                parent_id: None,
                wasm,
                code: code.into(),
                alive: true,
                inbox: vec![],
                death_reason: None,
                born_tick: 0,
                facing: 0,
            })
            .unwrap();
    }

    const MOVE_EAST: &str = r#"
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

    #[tokio::test]
    async fn tick_moves_creature_in_memory() {
        let engine = test_engine().await;
        seed_creature(&engine, "a", 0, 0, MOVE_EAST, 100_000_000).await;

        engine.tick_step();

        let WorldMessage::Delta {
            creatures_upsert, ..
        } = engine.full_delta()
        else {
            panic!("expected delta");
        };
        let c = creatures_upsert
            .iter()
            .find(|c| c.id == "a")
            .expect("creature a");
        assert_eq!(c.x, 1);
    }

    #[tokio::test]
    async fn checkpoint_persists_to_db() {
        let engine = test_engine().await;
        seed_creature(&engine, "a", 0, 0, MOVE_EAST, 100_000_000).await;

        for _ in 0..30 {
            let step = engine.tick_step();
            if let Some(snapshot) = step.persist {
                persist_world(&engine.db, &snapshot.creatures, &snapshot.tiles, &snapshot.ledger)
                    .await
                    .unwrap();
            }
        }

        let (x,): (i64,) = sqlx::query_as("SELECT x FROM creatures WHERE id = 'a'")
            .fetch_one(&engine.db)
            .await
            .unwrap();
        assert!(x >= 8, "expected tunnel progress, x={x}");
    }
}
