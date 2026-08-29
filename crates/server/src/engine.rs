use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::Serialize;
use sqlx::SqlitePool;
use terrarium_kernel::{
    assemble, run_tick, vm::Creature, WorldTile, WorldTiles, TICK_HZ,
};
use tokio::sync::broadcast;

use crate::config::Config;

const DEFAULT_PERSIST_EVERY: u64 = 10;
const BROADCAST_CAPACITY: usize = 4096;

#[derive(Clone, Serialize)]
pub struct CreaturePublic {
    pub id: String,
    pub x: i64,
    pub y: i64,
    pub energy: i64,
    pub owner_uid: String,
}

#[derive(Clone, Serialize)]
pub struct TilePublic {
    pub x: i64,
    pub y: i64,
    pub kind: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<i64>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldMessage {
    Snapshot {
        tick: u64,
        deploy_cost: i64,
        creatures: Vec<CreaturePublic>,
        tiles: Vec<TilePublic>,
    },
    Delta {
        tick: u64,
        creatures_upsert: Vec<CreaturePublic>,
        creatures_remove: Vec<String>,
        tiles_upsert: Vec<TilePublic>,
        tiles_remove: Vec<[i64; 2]>,
    },
}

struct PersistSnapshot {
    creatures: Vec<Creature>,
    tiles: WorldTiles,
}

struct TickStep {
    message: WorldMessage,
    persist: Option<PersistSnapshot>,
}
struct SimState {
    tick: u64,
    creatures: Vec<Creature>,
    tiles: WorldTiles,
}

pub struct WorldEngine {
    inner: RwLock<SimState>,
    pub db: SqlitePool,
    deploy_cost: i64,
    persist_every: u64,
    events: broadcast::Sender<WorldMessage>,
}

impl WorldEngine {
    pub async fn bootstrap(db: SqlitePool, config: &Config) -> anyhow::Result<Arc<Self>> {
        let creatures = load_creatures(&db).await?;
        let tiles = load_tiles(&db).await?;
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
            }),
            db,
            deploy_cost: config.deploy_cost,
            persist_every,
            events,
        }))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WorldMessage> {
        self.events.subscribe()
    }

    pub fn deploy_cost(&self) -> i64 {
        self.deploy_cost
    }

    pub fn snapshot(&self) -> WorldMessage {
        let state = self.inner.read();
        WorldMessage::Snapshot {
            tick: state.tick,
            deploy_cost: self.deploy_cost,
            creatures: state.creatures.iter().map(creature_public).collect(),
            tiles: tiles_public(&state.tiles),
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

    pub fn insert_creature(&self, creature: Creature) -> anyhow::Result<()> {
        let public = creature_public(&creature);
        let mut state = self.inner.write();
        state.creatures.push(creature);
        let msg = WorldMessage::Delta {
            tick: state.tick,
            creatures_upsert: vec![public],
            creatures_remove: vec![],
            tiles_upsert: vec![],
            tiles_remove: vec![],
        };
        drop(state);
        let _ = self.events.send(msg);
        Ok(())
    }

    pub fn tick_step(&self) -> TickStep {
        let mut state = self.inner.write();
        let tick = state.tick + 1;

        let before: HashMap<String, (i32, i32, i64)> = state
            .creatures
            .iter()
            .map(|c| (c.id.clone(), (c.x, c.y, c.energy)))
            .collect();
        let before_tiles = state.tiles.clone();

        let mut creatures = std::mem::take(&mut state.creatures);
        let mut tiles = std::mem::take(&mut state.tiles);
        run_tick(&mut creatures, &mut tiles);
        state.creatures = creatures;
        state.tiles = tiles;
        state.tick = tick;

        let message = build_delta(tick, &before, &state.creatures, &before_tiles, &state.tiles);
        let persist = if tick % self.persist_every == 0 {
            Some(PersistSnapshot {
                creatures: state.creatures.clone(),
                tiles: state.tiles.clone(),
            })
        } else {
            None
        };
        drop(state);

        TickStep { message, persist }
    }
}

fn build_delta(
    tick: u64,
    before: &HashMap<String, (i32, i32, i64)>,
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
            .map(|(x, y, e)| *x != c.x || *y != c.y || *e != c.energy)
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
        creatures_upsert,
        creatures_remove,
        tiles_upsert,
        tiles_remove,
    }
}

fn creature_public(c: &Creature) -> CreaturePublic {
    CreaturePublic {
        id: c.id.clone(),
        x: c.x as i64,
        y: c.y as i64,
        energy: c.energy,
        owner_uid: c.owner_uid.clone(),
    }
}

fn tile_public(pos: (i32, i32), tile: WorldTile) -> TilePublic {
    let (kind, energy) = match tile {
        WorldTile::Solid => (1, None),
        WorldTile::Corpse { energy } => (3, Some(energy)),
    };
    TilePublic {
        x: pos.0 as i64,
        y: pos.1 as i64,
        kind,
        energy,
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

const IDLE_BYTECODE_SOURCE: &str = "loop:\n  sleep\n  jmp loop\n";

async fn load_creatures(db: &SqlitePool) -> anyhow::Result<Vec<Creature>> {
    let rows = sqlx::query_as::<_, (String, String, i64, i64, i64, Option<Vec<u8>>, i64, Option<Vec<u8>>, String)>(
        "SELECT id, owner_uid, x, y, energy, bytecode, pc, stack, code FROM creatures ORDER BY id",
    )
    .fetch_all(db)
    .await?;

    let idle = assemble(IDLE_BYTECODE_SOURCE)?;

    rows.into_iter()
        .map(|(id, owner_uid, x, y, energy, bytecode, pc, stack, code)| {
            let bytecode = match bytecode.filter(|b| !b.is_empty()) {
                Some(b) => b,
                None => assemble(&code).unwrap_or_else(|_| idle.clone()),
            };
            Ok(Creature {
                id,
                owner_uid,
                x: x as i32,
                y: y as i32,
                energy,
                bytecode,
                pc: pc as usize,
                stack: decode_stack(stack.as_deref().unwrap_or(&[])),
                alive: true,
            })
        })
        .collect()
}

async fn load_tiles(db: &SqlitePool) -> anyhow::Result<WorldTiles> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, Option<i64>)>(
        "SELECT x, y, kind, energy FROM world_tiles",
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(x, y, kind, energy)| {
            let pos = (x as i32, y as i32);
            Some((
                pos,
                match kind {
                    1 => WorldTile::Solid,
                    3 => WorldTile::Corpse {
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
            "UPDATE creatures SET x = ?, y = ?, energy = ?, pc = ?, stack = ?, bytecode = COALESCE(bytecode, ?) WHERE id = ?",
        )
        .bind(creature.x as i64)
        .bind(creature.y as i64)
        .bind(creature.energy)
        .bind(creature.pc as i64)
        .bind(encode_stack(&creature.stack))
        .bind(&creature.bytecode)
        .bind(&creature.id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("DELETE FROM world_tiles").execute(&mut *tx).await?;
    for ((x, y), tile) in tiles {
        let (kind, energy): (i64, Option<i64>) = match tile {
            WorldTile::Solid => (1, None),
            WorldTile::Corpse { energy } => (3, Some(*energy)),
        };
        sqlx::query("INSERT INTO world_tiles (x, y, kind, energy) VALUES (?, ?, ?, ?)")
            .bind(*x as i64)
            .bind(*y as i64)
            .bind(kind)
            .bind(energy)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

fn encode_stack(stack: &[i32]) -> Vec<u8> {
    stack.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn decode_stack(bytes: &[u8]) -> Vec<i32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

pub fn spawn_tick_loop(engine: Arc<WorldEngine>) {
    let (persist_tx, mut persist_rx) = tokio::sync::mpsc::channel::<PersistSnapshot>(2);
    let db = engine.db.clone();

    tokio::spawn(async move {
        while let Some(snapshot) = persist_rx.recv().await {
            if let Err(err) = persist_world(&db, &snapshot.creatures, &snapshot.tiles).await {
                tracing::error!(error = %err, "checkpoint failed");
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
    use terrarium_kernel::{assemble, vm::Creature, EXAMPLE_PROGRAMS};

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
                faucet_max: 1000,
                deploy_cost: 100,
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
        let bytecode = assemble(code).unwrap();
        sqlx::query("INSERT INTO accounts (firebase_uid, credits) VALUES ('tester', 1000) ON CONFLICT DO NOTHING")
            .execute(&engine.db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO creatures (id, owner_uid, x, y, energy, code, bytecode, pc, stack) VALUES (?, 'tester', ?, ?, ?, ?, ?, 0, x'')",
        )
        .bind(id)
        .bind(x as i64)
        .bind(y as i64)
        .bind(energy)
        .bind(code)
        .bind(&bytecode)
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
                bytecode,
                pc: 0,
                stack: vec![],
                alive: true,
            })
            .unwrap();
    }

    #[tokio::test]
    async fn tick_moves_creature_in_memory() {
        let engine = test_engine().await;
        let code = EXAMPLE_PROGRAMS
            .iter()
            .find(|e| e.id == "tunnel")
            .unwrap()
            .code;
        seed_creature(&engine, "a", 0, 0, code, 1000).await;

        engine.tick_step();

        let WorldMessage::Snapshot { creatures, .. } = engine.snapshot() else {
            panic!("expected snapshot");
        };
        assert_eq!(creatures[0].x, 1);
    }

    #[tokio::test]
    async fn checkpoint_persists_to_db() {
        let engine = test_engine().await;
        let code = EXAMPLE_PROGRAMS
            .iter()
            .find(|e| e.id == "tunnel")
            .unwrap()
            .code;
        seed_creature(&engine, "a", 0, 0, code, 1000).await;

        for _ in 0..10 {
            let step = engine.tick_step();
            if let Some(snapshot) = step.persist {
                persist_world(&engine.db, &snapshot.creatures, &snapshot.tiles)
                    .await
                    .unwrap();
            }
        }

        let (x,): (i64,) = sqlx::query_as("SELECT x FROM creatures WHERE id = 'a'")
            .fetch_one(&engine.db)
            .await
            .unwrap();
        assert_eq!(x, 10);
    }
}
