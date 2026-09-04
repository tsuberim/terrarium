//! Ephemeral in-memory sim runs for preview / test harness.

use serde::{Deserialize, Serialize};
use terrarium_test_spec::{
    evaluate_assertions, AssertionResult, FrameSnapshot, TestSpec, TileKind, TilePlacement,
};

use crate::abi::Payload;
use crate::energy_ledger::EnergyLedger;
use crate::events::{CreatureAction, DeathReason, WorldEvent};
use crate::hex;
use crate::host;
use crate::sim_config::SimConfig;
use crate::vm::{run_tick, Creature};
use crate::world_tile::{WorldTile, WorldTiles};

const MAX_TICKS: u64 = 500;
const MAX_WASM_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxTile {
    pub x: i32,
    pub y: i32,
    pub kind: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxFrame {
    pub tick: u64,
    pub x: i32,
    pub y: i32,
    pub facing: u8,
    pub energy: i64,
    pub health: i32,
    pub alive: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<CreatureAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<WorldEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxBench {
    pub ticks_run: u64,
    pub start_energy: i64,
    pub end_energy: i64,
    pub total_spent: i64,
    pub per_tick_avg: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxResult {
    pub ok: bool,
    pub alive: bool,
    pub ticks_run: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_reason: Option<DeathReason>,
    pub frames: Vec<SandboxFrame>,
    pub tiles: Vec<SandboxTile>,
    pub bench: SandboxBench,
    pub test_passed: bool,
    pub assertions: Vec<AssertionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct SandboxRequest<'a> {
    pub wasm: &'a [u8],
    pub spec: TestSpec,
    pub config: Option<SimConfig>,
}

const SANDBOX_ID: u64 = 0x5341_4E44; // 'SAND'

pub fn run_sandbox(req: SandboxRequest<'_>) -> SandboxResult {
    host::clear_all_vm_cache();
    if req.wasm.is_empty() || req.wasm.len() > MAX_WASM_BYTES {
        return fail_result("invalid wasm size", 0);
    }

    if host::load_module(host::wasm_engine(), req.wasm).is_none() {
        return fail_result("invalid wasm module", 0);
    }

    let ticks = req.spec.ticks.clamp(1, MAX_TICKS);
    let start_energy = req.spec.start_energy;
    let config = req.config.clone().unwrap_or_else(sandbox_config);
    let mut tiles = spec_tiles(&req.spec, &config);
    let mut creatures = vec![Creature {
        id: SANDBOX_ID,
        x: 0,
        y: 0,
        energy: start_energy,
        health: config.max_health,
        max_health: config.max_health,
        owner_uid: "sandbox".into(),
        owner_id: SANDBOX_ID,
        parent_id: None,
        wasm: req.wasm.to_vec(),
        code: "// sandbox".into(),
        alive: true,
        inbox: vec![],
        death_reason: None,
        born_tick: 0,
        facing: req.spec.facing,
        init: Payload::default(),
    }];

    let mut frames = Vec::with_capacity(ticks as usize);
    let mut ledger = EnergyLedger::default();
    let mut ticks_run = 0u64;

    for tick in 1..=ticks {
        ticks_run = tick;
        let tick_result = run_tick(
            &mut creatures,
            &mut tiles,
            &mut ledger,
            &config,
            tick,
            &mut vec![],
        );
        let creature = creatures
            .iter()
            .find(|c| c.id == SANDBOX_ID)
            .cloned()
            .unwrap_or_else(|| dead_creature(&config));

        frames.push(SandboxFrame {
            tick,
            x: creature.x,
            y: creature.y,
            facing: creature.facing,
            energy: creature.energy,
            health: creature.health,
            alive: creature.alive,
            actions: tick_result.actions,
            events: tick_result.events,
        });

        if !creature.alive {
            break;
        }
    }

    let end_creature = creatures.iter().find(|c| c.id == SANDBOX_ID);
    let alive = end_creature.map(|c| c.alive).unwrap_or(false);
    let death_reason = end_creature.and_then(|c| c.death_reason);
    let end_energy = end_creature.map(|c| c.energy).unwrap_or(0);
    let total_spent = (start_energy - end_energy).max(0);
    let per_tick_avg = if ticks_run > 0 {
        total_spent / ticks_run as i64
    } else {
        0
    };

    let snapshots: Vec<FrameSnapshot> = frames
        .iter()
        .map(|f| FrameSnapshot {
            tick: f.tick,
            x: f.x,
            y: f.y,
            facing: f.facing,
            energy: f.energy,
            alive: f.alive,
        })
        .collect();
    let assertions = evaluate_assertions(&req.spec.assertions, &snapshots);
    let test_passed = assertions.iter().all(|a| a.passed);

    SandboxResult {
        ok: true,
        alive,
        ticks_run,
        death_reason,
        tiles: tiles_public(&tiles),
        bench: SandboxBench {
            ticks_run,
            start_energy,
            end_energy,
            total_spent,
            per_tick_avg,
        },
        frames,
        test_passed,
        assertions,
        error: None,
    }
}

fn fail_result(msg: &str, start_energy: i64) -> SandboxResult {
    SandboxResult {
        ok: false,
        alive: false,
        ticks_run: 0,
        death_reason: Some(DeathReason::InvalidProgram),
        frames: vec![],
        tiles: vec![],
        bench: empty_bench(start_energy, 0),
        test_passed: false,
        assertions: vec![],
        error: Some(msg.into()),
    }
}

fn dead_creature(config: &SimConfig) -> Creature {
    Creature {
        id: SANDBOX_ID,
        x: 0,
        y: 0,
        energy: 0,
        health: 0,
        max_health: config.max_health,
        owner_uid: "sandbox".into(),
        owner_id: SANDBOX_ID,
        parent_id: None,
        wasm: vec![],
        code: String::new(),
        alive: false,
        inbox: vec![],
        death_reason: Some(DeathReason::EnergyFloor),
        born_tick: 0,
        facing: 0,
        init: Payload::default(),
    }
}

fn sandbox_config() -> SimConfig {
    SimConfig {
        max_active_food: 0,
        food_spawn_interval: u64::MAX,
        ..SimConfig::default()
    }
}

fn spec_tiles(spec: &TestSpec, config: &SimConfig) -> WorldTiles {
    let mut tiles = WorldTiles::new();
    for placement in &spec.tiles {
        let (x, y) = match placement {
            TilePlacement::At { x, y, .. } => (*x, *y),
            TilePlacement::Ahead { facing, .. } => hex::neighbor(0, 0, *facing).unwrap_or((1, 0)),
        };
        let kind = match placement {
            TilePlacement::At { kind, .. } | TilePlacement::Ahead { kind, .. } => kind,
        };
        insert_tile(&mut tiles, x, y, kind, config);
    }
    tiles
}

fn insert_tile(tiles: &mut WorldTiles, x: i32, y: i32, kind: &TileKind, config: &SimConfig) {
    let tile = match kind {
        TileKind::Solid => WorldTile::Solid,
        TileKind::Food { energy } => WorldTile::Food {
            energy: energy.unwrap_or(config.food_nominal_energy),
        },
        TileKind::Corpse { energy } => WorldTile::Corpse {
            energy: energy.unwrap_or(config.corpse_energy),
            death_reason: DeathReason::EnergyFloor,
        },
    };
    tiles.insert((x, y), tile);
}

fn tiles_public(tiles: &WorldTiles) -> Vec<SandboxTile> {
    let mut out: Vec<_> = tiles
        .iter()
        .map(|((x, y), tile)| match tile {
            WorldTile::Solid => SandboxTile {
                x: *x,
                y: *y,
                kind: 1,
                energy: None,
            },
            WorldTile::Corpse { energy, .. } => SandboxTile {
                x: *x,
                y: *y,
                kind: 3,
                energy: Some(*energy),
            },
            WorldTile::Food { energy } => SandboxTile {
                x: *x,
                y: *y,
                kind: 4,
                energy: Some(*energy),
            },
        })
        .collect();
    out.sort_by_key(|t| (t.x, t.y));
    out
}

fn empty_bench(start: i64, ticks: u64) -> SandboxBench {
    SandboxBench {
        ticks_run: ticks,
        start_energy: start,
        end_energy: start,
        total_spent: 0,
        per_tick_avg: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_wat;
    use crate::wat::WAT_MOVE_FWD_LOOP;
    use terrarium_test_spec::parse_tests;
    use terrarium_test_spec::TestSpec;

    fn runner_open_spec() -> TestSpec {
        parse_tests(
            r#"
#[terrarium::test]
fn open_field() {
    run_ticks(5);
    assert!(alive());
}
"#,
        )
        .tests
        .into_iter()
        .next()
        .unwrap()
    }

    fn runner_wall_spec() -> TestSpec {
        parse_tests(
            r#"
#[terrarium::test]
fn wall_blocked() {
    tile_ahead(solid());
    run_ticks(3);
    assert_eq!(x(), 0);
}
"#,
        )
        .tests
        .into_iter()
        .next()
        .unwrap()
    }

    #[test]
    fn runner_moves_in_open_sandbox() {
        let wasm = compile_wat(WAT_MOVE_FWD_LOOP).unwrap();
        let res = run_sandbox(SandboxRequest {
            wasm: &wasm,
            spec: runner_open_spec(),
            config: None,
        });
        assert!(res.ok);
        assert!(res.alive);
        assert!(res.test_passed);
        assert!(res.frames.last().unwrap().x > 0);
        assert!(res.bench.total_spent > 0);
    }

    #[test]
    fn wall_ahead_blocks_runner() {
        let wasm = compile_wat(WAT_MOVE_FWD_LOOP).unwrap();
        let res = run_sandbox(SandboxRequest {
            wasm: &wasm,
            spec: runner_wall_spec(),
            config: None,
        });
        assert!(res.test_passed);
        assert!(res.frames.iter().all(|f| f.x == 0));
    }
}
