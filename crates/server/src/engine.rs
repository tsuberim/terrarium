use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use sqlx::SqlitePool;
use terrarium_sim::{run_tick, vm::Creature, EnergyLedger, SimConfig, WorldTile, WorldTiles};
use tokio::sync::broadcast;

use crate::config::Config;
use crate::persist::{load_creatures, load_ledger, load_tiles, PersistSnapshot};
use crate::seed::seed_ecosystem;
use crate::wire::{build_creature_delta, build_tile_delta, creature_public, tiles_public};
pub use crate::wire::{CreaturePublic, TilePublic, WorldMessage};

const DEFAULT_PERSIST_EVERY: u64 = 10;
const BROADCAST_CAPACITY: usize = 4096;

pub(crate) struct TickStep {
    pub message: WorldMessage,
    pub persist: Option<PersistSnapshot>,
    pub credit_payouts: Vec<(String, i64)>,
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

    pub(crate) fn try_broadcast(&self, msg: WorldMessage) -> bool {
        self.events.send(msg).is_ok()
    }

    pub fn current_tick(&self) -> u64 {
        self.inner.read().tick
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
        let msg = WorldMessage::tick_delta(
            state.tick,
            vec![public],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        drop(state);
        let _ = self.events.send(msg);
        Ok(())
    }

    pub(crate) fn tick_step(&self) -> TickStep {
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

        let mut creatures = std::mem::take(&mut state.creatures);
        let mut tiles = std::mem::take(&mut state.tiles);
        let mut ledger = state.ledger.clone();
        let config = self.sim_config.read().clone();
        let tick_result = run_tick(&mut creatures, &mut tiles, &mut ledger, &config, tick);
        state.creatures = creatures;
        state.tiles = tiles;
        state.ledger = ledger;
        state.tick = tick;

        let (creatures_upsert, creatures_remove) = build_creature_delta(&before, &state.creatures);
        let (tiles_upsert, tiles_remove) = build_tile_delta(&tick_result.tiles_dirty, &state.tiles);
        let message = WorldMessage::tick_delta(
            tick,
            creatures_upsert,
            creatures_remove,
            tiles_upsert,
            tiles_remove,
            tick_result.actions,
            tick_result.events,
        );
        let persist = if tick.is_multiple_of(self.persist_every) {
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

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use terrarium_sim::{compile_wat, vm::Creature};

    use super::*;
    use crate::config::Config;
    use crate::persist::persist_world;

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
                faucet_max: 100 * terrarium_sim::ENERGY_SCALE,
                deploy_cost: 100 * terrarium_sim::ENERGY_SCALE,
                seed_ecosystem: false,
                compile_worker_url: None,
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
        } = engine.full_delta();
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
                persist_world(
                    &engine.db,
                    &snapshot.creatures,
                    &snapshot.tiles,
                    &snapshot.ledger,
                )
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
