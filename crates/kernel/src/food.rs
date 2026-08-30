use crate::energy_ledger::EnergyLedger;
use crate::sim_config::SimConfig;
use crate::world_tile::{count_food, place_food, WorldTiles};

fn hash_u64(seed: u64) -> u64 {
    let mut h = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}

fn pick_spawn_cell(
    tick: u64,
    attempt: u32,
    occupied: &[(i32, i32)],
    radius: i32,
) -> (i32, i32) {
    let (fq, fr) = if occupied.is_empty() {
        (0, 0)
    } else {
        let n = occupied.len() as i64;
        let mut sq = 0i64;
        let mut sr = 0i64;
        for &(x, y) in occupied {
            sq += i64::from(x);
            sr += i64::from(y);
        }
        ((sq / n) as i32, (sr / n) as i32)
    };

    let h = hash_u64(tick.wrapping_mul(1_000_003).wrapping_add(u64::from(attempt)));
    let ring = (h % (radius as u64).max(1) as u64) as i32 + 1;
    let face = (h >> 16) % 6;
    let step = ((h >> 32) % (ring as u64).max(1) as u64) as i32;

    let mut q = fq;
    let mut r = fr;
    for _ in 0..ring {
        (q, r) = crate::hex::neighbor(q, r, face as u8).unwrap_or((q, r));
    }
    for _ in 0..step {
        (q, r) = crate::hex::neighbor(q, r, ((face + 2) % 6) as u8).unwrap_or((q, r));
    }
    (q, r)
}

fn cell_open(tiles: &WorldTiles, occupied: &[(i32, i32)], q: i32, r: i32) -> bool {
    if tiles.contains_key(&(q, r)) {
        return false;
    }
    !occupied.iter().any(|&(x, y)| x == q && y == r)
}

/// Try to place food tiles using remaining free-mint budget.
pub fn try_spawn_food(
    ledger: &mut EnergyLedger,
    tiles: &mut WorldTiles,
    occupied: &[(i32, i32)],
    tick: u64,
    config: &SimConfig,
) -> u32 {
    if tick % config.food_spawn_interval != 0 {
        return 0;
    }
    if ledger.free_budget() <= 0 {
        return 0;
    }
    if count_food(tiles) >= config.max_active_food {
        return 0;
    }

    let mut spawned = 0u32;

    for attempt in 0..config.food_spawn_attempts {
        if ledger.free_budget() <= 0 {
            break;
        }
        if count_food(tiles) >= config.max_active_food {
            break;
        }
        let (q, r) = pick_spawn_cell(tick, attempt, occupied, config.food_spawn_radius);
        if !cell_open(tiles, occupied, q, r) {
            continue;
        }
        let grant = ledger.try_mint_free(config.food_nominal_energy);
        if grant > 0 {
            place_food(tiles, q, r, grant);
            spawned += 1;
        }
    }

    spawned
}
