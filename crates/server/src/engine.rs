use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use sqlx::SqlitePool;
use terrarium_sim::{
    host::PendingAction, run_tick, validate_external_broadcast, validate_external_signal,
    vm::Creature, EnergyLedger, Payload, SimConfig, WorldTile, WorldTiles,
};
use tokio::sync::broadcast;

use crate::config::Config;
use crate::control::ControlRegistry;
use crate::persist::{load_creatures, load_ledger, load_tiles, PersistSnapshot};
use crate::seed::seed_ecosystem;
use crate::wire::{build_creature_delta, build_tile_delta, creature_public, tiles_public};
pub use crate::wire::{CreaturePublic, TilePublic, WorldMessage};

const DEFAULT_PERSIST_EVERY: u64 = 10;
const BROADCAST_CAPACITY: usize = 4096;

pub struct CreatureInfo {
    pub alive: bool,
}

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
    external_actions: Mutex<Vec<(u64, PendingAction)>>,
    control: RwLock<Option<Arc<ControlRegistry>>>,
}

impl WorldEngine {
    pub async fn bootstrap(db: SqlitePool, config: &Config) -> anyhow::Result<Arc<Self>> {
        let mut creatures = load_creatures(&db).await?;
        let mut tiles = load_tiles(&db).await?;
        let mut ledger = load_ledger(&db).await?;
        let sim_config = SimConfig::default();

        if creatures.is_empty() && config.seed_ecosystem {
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
            external_actions: Mutex::new(Vec::new()),
            control: RwLock::new(None),
        }))
    }

    pub fn attach_control(&self, control: Arc<ControlRegistry>) {
        *self.control.write() = Some(control);
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

    pub fn creature_owned_by(&self, creature_id: u64, owner_uid: &str) -> Option<CreatureInfo> {
        let state = self.inner.read();
        state
            .creatures
            .iter()
            .find(|c| c.id == creature_id && c.owner_uid == owner_uid)
            .map(|c| CreatureInfo { alive: c.alive })
    }

    pub fn inject_control_signal(
        &self,
        from_id: u64,
        target: u64,
        payload: Payload,
    ) -> Result<(), &'static str> {
        let state = self.inner.read();
        validate_external_signal(&state.creatures, from_id, target, &self.sim_config.read())?;
        drop(state);
        self.external_actions
            .lock()
            .push((from_id, PendingAction::Signal { target, payload }));
        Ok(())
    }

    pub fn inject_control_broadcast(
        &self,
        from_id: u64,
        payload: Payload,
    ) -> Result<(), &'static str> {
        let state = self.inner.read();
        validate_external_broadcast(&state.creatures, from_id)?;
        drop(state);
        self.external_actions
            .lock()
            .push((from_id, PendingAction::Broadcast { payload }));
        Ok(())
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

        let before: HashMap<u64, (i32, i32, i64, i32, i32, u8)> = state
            .creatures
            .iter()
            .map(|c| (c.id, (c.x, c.y, c.energy, c.health, c.max_health, c.facing)))
            .collect();

        let mut creatures = std::mem::take(&mut state.creatures);
        let mut tiles = std::mem::take(&mut state.tiles);
        let mut ledger = state.ledger.clone();
        let config = self.sim_config.read().clone();
        let mut external = std::mem::take(&mut *self.external_actions.lock());
        let tick_result = run_tick(
            &mut creatures,
            &mut tiles,
            &mut ledger,
            &config,
            tick,
            &mut external,
        );
        state.creatures = creatures;
        state.tiles = tiles;
        state.ledger = ledger;
        state.tick = tick;

        let (creatures_upsert, creatures_remove) = build_creature_delta(&before, &state.creatures);
        let (tiles_upsert, tiles_remove) = build_tile_delta(&tick_result.tiles_dirty, &state.tiles);
        let message = WorldMessage::tick_delta(
            tick,
            creatures_upsert,
            creatures_remove.clone(),
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

        if let Some(control) = self.control.read().clone() {
            for delivery in &tick_result.inbox_deliveries {
                control.notify_recv(delivery.creature_id, delivery.sender, delivery.payload);
            }
            for id in &creatures_remove {
                control.notify_detached(*id, "creature_died");
            }
        }

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
    use terrarium_sim::{compile_wat, vm::Creature, wat::WAT_MOVE_FWD_LOOP, Payload};

    use super::*;
    use crate::config::Config;
    use crate::persist::persist_world;

    const TEST_CREATURE_ID: u64 = 10001;

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

    async fn seed_creature(engine: &WorldEngine, id: u64, x: i32, y: i32, code: &str, energy: i64) {
        let wasm = compile_wat(code).unwrap();
        sqlx::query("INSERT INTO accounts (firebase_uid, credits) VALUES ('tester', 1000) ON CONFLICT DO NOTHING")
            .execute(&engine.db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO creatures (id, owner_uid, owner_id, x, y, energy, health, max_health, code, bytecode, born_tick, facing, pc, stack) VALUES (?, 'tester', ?, ?, ?, ?, 100, 100, ?, ?, 0, 0, 0, x'')",
        )
        .bind(id as i64)
        .bind(id as i64)
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
                id,
                owner_uid: "tester".into(),
                owner_id: id,
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
                init: Payload::default(),
            })
            .unwrap();
    }

    const MOVE_EAST: &str = WAT_MOVE_FWD_LOOP;

    #[tokio::test]
    async fn main_moves_creature_in_memory() {
        let engine = test_engine().await;
        seed_creature(&engine, TEST_CREATURE_ID, 0, 0, MOVE_EAST, 100_000_000).await;

        engine.tick_step();

        let WorldMessage::Delta {
            creatures_upsert, ..
        } = engine.full_delta();
        let c = creatures_upsert
            .iter()
            .find(|c| c.id == TEST_CREATURE_ID)
            .expect("creature");
        assert_eq!(c.x, 1);
    }

    #[tokio::test]
    async fn checkpoint_persists_to_db() {
        let engine = test_engine().await;
        seed_creature(&engine, TEST_CREATURE_ID, 0, 0, MOVE_EAST, 100_000_000).await;

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

        let (x,): (i64,) = sqlx::query_as("SELECT x FROM creatures WHERE id = ?")
            .bind(TEST_CREATURE_ID as i64)
            .fetch_one(&engine.db)
            .await
            .unwrap();
        assert!(x >= 8, "expected tunnel progress, x={x}");
    }
}
