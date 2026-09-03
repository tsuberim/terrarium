//! Ephemeral in-memory sim runs for preview / test harness.

use serde::{Deserialize, Serialize};

use crate::energy_ledger::EnergyLedger;
use crate::events::{CreatureAction, DeathReason, WorldEvent};
use crate::hex;
use crate::host;
use crate::sim_config::SimConfig;
use crate::vm::{run_tick, Creature};
use crate::world_tile::{WorldTile, WorldTiles};

const DEFAULT_START_ENERGY: i64 = 4_000_000;
const MAX_TICKS: u64 = 500;
const MAX_WASM_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxScenario {
    Open,
    FoodAhead,
    WallAhead,
    CorpseAhead,
}

impl SandboxScenario {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "open" | "open_field" => Self::Open,
            "food_ahead" | "food" => Self::FoodAhead,
            "wall_ahead" | "wall" | "wall_blocked" => Self::WallAhead,
            "corpse_ahead" | "corpse" => Self::CorpseAhead,
            _ => return None,
        })
    }

    pub fn parse_or_open(s: &str) -> Self {
        Self::parse(s).unwrap_or(Self::Open)
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct SandboxRequest<'a> {
    pub wasm: &'a [u8],
    pub scenario: SandboxScenario,
    pub ticks: u64,
    pub config: Option<SimConfig>,
    pub start_energy: Option<i64>,
}

pub fn run_sandbox(req: SandboxRequest<'_>) -> SandboxResult {
    if req.wasm.is_empty() || req.wasm.len() > MAX_WASM_BYTES {
        return SandboxResult {
            ok: false,
            alive: false,
            ticks_run: 0,
            death_reason: Some(DeathReason::InvalidProgram),
            frames: vec![],
            tiles: vec![],
            bench: empty_bench(0, 0),
            error: Some("invalid wasm size".into()),
        };
    }

    if host::load_module(host::wasm_engine(), req.wasm).is_none() {
        return SandboxResult {
            ok: false,
            alive: false,
            ticks_run: 0,
            death_reason: Some(DeathReason::InvalidProgram),
            frames: vec![],
            tiles: vec![],
            bench: empty_bench(0, 0),
            error: Some("invalid wasm module".into()),
        };
    }

    let ticks = req.ticks.clamp(1, MAX_TICKS);
    let start_energy = req.start_energy.unwrap_or(DEFAULT_START_ENERGY);
    let config = req.config.clone().unwrap_or_else(sandbox_config);
    let mut tiles = scenario_tiles(req.scenario, &config);
    let mut creatures = vec![Creature {
        id: "sandbox".into(),
        x: 0,
        y: 0,
        energy: start_energy,
        health: config.max_health,
        max_health: config.max_health,
        owner_uid: "sandbox".into(),
        parent_id: None,
        wasm: req.wasm.to_vec(),
        code: "// sandbox".into(),
        alive: true,
        inbox: vec![],
        death_reason: None,
        born_tick: 0,
        facing: 0,
    }];

    let mut frames = Vec::with_capacity(ticks as usize);
    let mut ledger = EnergyLedger::default();
    let mut ticks_run = 0u64;

    for tick in 1..=ticks {
        ticks_run = tick;
        let tick_result = run_tick(&mut creatures, &mut tiles, &mut ledger, &config, tick);
        let creature = creatures
            .iter()
            .find(|c| c.id == "sandbox")
            .cloned()
            .unwrap_or_else(|| Creature {
                id: "sandbox".into(),
                x: 0,
                y: 0,
                energy: 0,
                health: 0,
                max_health: config.max_health,
                owner_uid: "sandbox".into(),
                parent_id: None,
                wasm: vec![],
                code: String::new(),
                alive: false,
                inbox: vec![],
                death_reason: Some(DeathReason::EnergyFloor),
                born_tick: 0,
                facing: 0,
            });

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

    let end_creature = creatures.iter().find(|c| c.id == "sandbox");
    let alive = end_creature.map(|c| c.alive).unwrap_or(false);
    let death_reason = end_creature.and_then(|c| c.death_reason);
    let end_energy = end_creature.map(|c| c.energy).unwrap_or(0);
    let total_spent = (start_energy - end_energy).max(0);
    let per_tick_avg = if ticks_run > 0 {
        total_spent / ticks_run as i64
    } else {
        0
    };

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
        error: None,
    }
}

fn sandbox_config() -> SimConfig {
    SimConfig {
        max_active_food: 0,
        food_spawn_interval: u64::MAX,
        ..SimConfig::default()
    }
}

fn scenario_tiles(scenario: SandboxScenario, config: &SimConfig) -> WorldTiles {
    let mut tiles = WorldTiles::new();
    let ahead = hex::neighbor(0, 0, 0).expect("dir 0");
    match scenario {
        SandboxScenario::Open => {}
        SandboxScenario::FoodAhead => {
            tiles.insert(
                ahead,
                WorldTile::Food {
                    energy: config.food_nominal_energy,
                },
            );
        }
        SandboxScenario::WallAhead => {
            tiles.insert((ahead.0, ahead.1), WorldTile::Solid);
        }
        SandboxScenario::CorpseAhead => {
            tiles.insert(
                ahead,
                WorldTile::Corpse {
                    energy: config.corpse_energy,
                    death_reason: DeathReason::EnergyFloor,
                },
            );
        }
    }
    tiles
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
    use crate::examples::RUNNER;

    #[test]
    fn runner_moves_in_open_sandbox() {
        let wasm = compile_wat(RUNNER).unwrap();
        let res = run_sandbox(SandboxRequest {
            wasm: &wasm,
            scenario: SandboxScenario::Open,
            ticks: 5,
            config: None,
            start_energy: None,
        });
        assert!(res.ok);
        assert!(res.alive);
        assert!(res.frames.last().unwrap().x > 0);
        assert!(res.bench.total_spent > 0);
    }

    #[test]
    fn wall_ahead_blocks_runner() {
        let wasm = compile_wat(RUNNER).unwrap();
        let res = run_sandbox(SandboxRequest {
            wasm: &wasm,
            scenario: SandboxScenario::WallAhead,
            ticks: 3,
            config: None,
            start_energy: None,
        });
        assert!(res.frames.iter().all(|f| f.x == 0));
    }
}
