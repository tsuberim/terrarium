//! Bootstrap ecosystem seed data.

use sqlx::SqlitePool;
use terrarium_kernel::{
    compile_wat, food, vm::Creature, EnergyLedger, SimConfig, WorldTile, WorldTiles,
    EXAMPLE_PROGRAMS,
};

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

pub async fn seed_ecosystem(
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
    let mut tile_dirty = terrarium_kernel::world_tile::TileDirty::new();
    for tick in (0..sim_config.food_spawn_interval * 4).step_by(sim_config.food_spawn_interval as usize) {
        food::try_spawn_food(ledger, tiles, &mut tile_dirty, &occupied, tick, sim_config);
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
