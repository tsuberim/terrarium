//! SQLite load/persist for world state.

use std::collections::HashSet;

use sqlx::SqlitePool;
use terrarium_sim::{
    compile_wat, vm::Creature, wat::WAT_IDLE, DeathReason, EnergyLedger, Payload, WorldTile,
    WorldTiles,
};

pub(crate) struct PersistSnapshot {
    pub creatures: Vec<Creature>,
    pub tiles: WorldTiles,
    pub ledger: EnergyLedger,
}

pub async fn load_creatures(db: &SqlitePool) -> anyhow::Result<Vec<Creature>> {
    let rows = sqlx::query_as::<
        _,
        (
            i64,
            String,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            Option<Vec<u8>>,
            String,
            i64,
            i64,
        ),
    >(
        "SELECT id, owner_uid, owner_id, x, y, energy, health, max_health, bytecode, code, born_tick, facing FROM creatures ORDER BY id",
    )
    .fetch_all(db)
    .await?;

    let idle = compile_wat(WAT_IDLE)?;

    rows.into_iter()
        .map(
            |(
                id,
                owner_uid,
                owner_id,
                x,
                y,
                energy,
                health,
                max_health,
                wasm_blob,
                code,
                born_tick,
                facing,
            )| {
                let wasm = match wasm_blob.filter(|b| !b.is_empty()) {
                    Some(b) => b,
                    None => compile_wat(&code).unwrap_or_else(|_| idle.clone()),
                };
                let id = id as u64;
                let owner_id = if owner_id == 0 { id } else { owner_id as u64 };
                Ok(Creature {
                    id,
                    owner_uid,
                    owner_id,
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
                    init: Payload::default(),
                })
            },
        )
        .collect()
}

pub async fn load_ledger(db: &SqlitePool) -> anyhow::Result<EnergyLedger> {
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

pub async fn load_tiles(db: &SqlitePool) -> anyhow::Result<WorldTiles> {
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

    let alive_ids: HashSet<i64> = creatures.iter().map(|c| c.id as i64).collect();
    let db_ids = sqlx::query_scalar::<_, i64>("SELECT id FROM creatures")
        .fetch_all(&mut *tx)
        .await?;

    for id in db_ids {
        if !alive_ids.contains(&id) {
            sqlx::query("DELETE FROM creatures WHERE id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }

    for (i, creature) in creatures.iter().enumerate() {
        let slot = -(i as i64 + 1);
        sqlx::query("UPDATE creatures SET x = ?, y = ? WHERE id = ?")
            .bind(slot)
            .bind(slot)
            .bind(creature.id as i64)
            .execute(&mut *tx)
            .await?;
    }

    for creature in creatures {
        sqlx::query(
            "UPDATE creatures SET x = ?, y = ?, energy = ?, health = ?, max_health = ?, facing = ?, owner_id = ?, bytecode = COALESCE(bytecode, ?), born_tick = ? WHERE id = ?",
        )
        .bind(creature.x as i64)
        .bind(creature.y as i64)
        .bind(creature.energy)
        .bind(creature.health as i64)
        .bind(creature.max_health as i64)
        .bind(creature.facing as i64)
        .bind(creature.owner_id as i64)
        .bind(&creature.wasm)
        .bind(creature.born_tick as i64)
        .bind(creature.id as i64)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("DELETE FROM world_tiles")
        .execute(&mut *tx)
        .await?;
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

pub async fn credit_payout(db: &SqlitePool, uid: &str, amount: i64) -> anyhow::Result<()> {
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
