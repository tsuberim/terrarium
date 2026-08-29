//! Semantic tests for the WASM runtime.

use crate::compile_wat;
use crate::energy_ledger::EnergyLedger;
use crate::examples::{BEACON, IDLE, KAMIKAZE, PREDATOR, PREY, RUNNER, SCAVENGER};
use crate::sim_config::SimConfig;
use crate::vm::{run_tick, Creature};
use crate::world_tile::{WorldTile, WorldTiles};
use crate::DeathReason;
use crate::TickResult;

fn default_config() -> SimConfig {
    SimConfig::default()
}

fn tick_world(
    creatures: &mut Vec<Creature>,
    tiles: &mut WorldTiles,
    config: &SimConfig,
    tick: u64,
) -> TickResult {
    let mut ledger = EnergyLedger::default();
    run_tick(creatures, tiles, &mut ledger, config, tick)
}

fn creature_at(x: i32, y: i32, code: &'static str) -> Creature {
    let config = default_config();
    Creature {
        id: "c".into(),
        x,
        y,
        energy: 4_000_000,
        health: config.max_health,
        max_health: config.max_health,
        owner_uid: "u".into(),
        parent_id: None,
        wasm: compile_wat(code).unwrap(),
        code: code.into(),
        alive: true,
        inbox: vec![],
        death_reason: None,
        born_tick: 0,
    }
}

fn empty_tiles() -> WorldTiles {
    WorldTiles::new()
}

fn run(code: &'static str, x: i32, y: i32, tiles: &mut WorldTiles) -> Creature {
    let mut creatures = vec![creature_at(x, y, code)];
    tick_world(&mut creatures, tiles, &default_config(), 1);
    creatures.remove(0)
}

#[test]
fn empty_wasm_kills() {
    let mut creatures = vec![Creature {
        wasm: vec![],
        ..creature_at(0, 0, IDLE)
    }];
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 1);
    assert!(creatures.is_empty());
}

#[test]
fn blocked_move_costs_energy_but_stays_put() {
    let mut tiles = empty_tiles();
    tiles.insert((1, 0), WorldTile::Solid);
    let before = 10_000_000_i64;
    let mut creatures = vec![Creature {
        energy: before,
        ..creature_at(0, 0, RUNNER)
    }];
    tick_world(&mut creatures, &mut tiles, &default_config(), 1);
    assert_eq!(creatures[0].x, 0);
    assert!(creatures[0].energy < before);
    assert!(creatures[0].energy >= before - 200_000);
}

#[test]
fn opcode_gas_charges_energy() {
    let config = SimConfig {
        opcodes_per_tick: 64,
        energy_per_opcode: 1,
        ..SimConfig::default()
    };
    let before = 10_000_000_i64;
    let mut creatures = vec![Creature {
        energy: before,
        ..creature_at(0, 0, IDLE)
    }];
    tick_world(&mut creatures, &mut WorldTiles::new(), &config, 1);
    let spent = before - creatures[0].energy;
    assert!(spent > 0 && spent < 20, "idle should cost a few opcodes, spent={spent}");
}

#[test]
fn out_of_gas_kills() {
    let config = SimConfig {
        opcodes_per_tick: 2,
        energy_per_opcode: 1,
        ..SimConfig::default()
    };
    let mut creatures = vec![creature_at(0, 0, PREY)];
    tick_world(&mut creatures, &mut WorldTiles::new(), &config, 1);
    assert!(creatures.is_empty(), "prey exceeds 2-opcode budget");
}

#[test]
fn out_of_gas_records_reason() {
    let config = SimConfig {
        opcodes_per_tick: 2,
        energy_per_opcode: 1,
        ..SimConfig::default()
    };
    let mut creatures = vec![creature_at(0, 0, PREY)];
    let result = tick_world(&mut creatures, &mut WorldTiles::new(), &config, 1);
    assert!(creatures.is_empty(), "prey exceeds 2-opcode budget");
    assert_eq!(
        result.events,
        vec![crate::WorldEvent::Death {
            creature_id: "c".into(),
            owner_uid: "u".into(),
            x: 0,
            y: 0,
            reason: crate::DeathReason::OutOfGas,
        }]
    );
}

#[test]
fn predator_survives_scan_on_empty_world() {
    let mut creatures = vec![creature_at(0, 0, PREDATOR)];
    let result = tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 1);
    assert_eq!(creatures.len(), 1, "predator should survive full vision scan + wander");
    assert!(!result.events.iter().any(|e| matches!(e, crate::WorldEvent::Death { .. })));
}

#[test]
fn predator_chases_distant_prey() {
    let mut creatures = vec![
        creature_at(0, 0, PREDATOR),
        Creature {
            id: "prey".into(),
            x: 3,
            y: 0,
            ..creature_at(0, 0, IDLE)
        },
    ];
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 1);
    let pred = creatures.iter().find(|c| c.id == "c").expect("predator alive");
    assert_eq!(pred.x, 2);
}

#[test]
fn corpse_yield_is_eighty_percent() {
    use crate::abi::corpse_yield_energy;
    assert_eq!(corpse_yield_energy(5_000_000), 4_000_000);
    assert_eq!(corpse_yield_energy(1_000_000), 800_000);
}

#[test]
fn death_leaves_corpse_with_eighty_percent_energy() {
    let config = SimConfig {
        hit_damage: 100,
        ..default_config()
    };
    let mut tiles = empty_tiles();
    let prey_energy = 5_000_000;
    let mut creatures = vec![
        creature_at(0, 0, PREDATOR),
        Creature {
            id: "prey".into(),
            x: 1,
            y: 0,
            energy: prey_energy,
            health: 100,
            max_health: 100,
            ..creature_at(0, 0, IDLE)
        },
    ];
    tick_world(&mut creatures, &mut tiles, &config, 1);
    match tiles.get(&(1, 0)) {
        Some(WorldTile::Corpse { energy, .. }) => {
            use crate::abi::corpse_yield_energy;
            assert!(*energy <= corpse_yield_energy(prey_energy));
            assert!(*energy >= corpse_yield_energy(prey_energy) - 10_000);
        }
        other => panic!("expected corpse at prey cell, got {other:?}"),
    }
}

#[test]
fn predator_kills_adjacent_prey() {
    let config = SimConfig {
        hit_damage: 100,
        ..default_config()
    };
    let mut creatures = vec![
        creature_at(0, 0, PREDATOR),
        Creature {
            id: "prey".into(),
            x: 1,
            y: 0,
            energy: 5_000_000,
            health: 100,
            max_health: 100,
            ..creature_at(0, 0, IDLE)
        },
    ];
    tick_world(&mut creatures, &mut WorldTiles::new(), &config, 1);
    assert_eq!(creatures.len(), 1);
    assert!(creatures.iter().all(|c| c.id != "prey"));
}

#[test]
fn idle_regen_costs_energy() {
    let config = default_config();
    let mut creatures = vec![Creature {
        health: 50,
        max_health: 100,
        energy: 10_000_000,
        ..creature_at(0, 0, IDLE)
    }];
    let before = creatures[0].energy;
    tick_world(&mut creatures, &mut WorldTiles::new(), &config, 1);
    assert_eq!(creatures[0].health, 50 + config.health_regen);
    assert!(creatures[0].energy <= before - config.health_regen_cost);
}

#[test]
fn moving_creature_does_not_regen() {
    let config = default_config();
    let mut creatures = vec![Creature {
        health: 50,
        max_health: 100,
        ..creature_at(0, 0, RUNNER)
    }];
    tick_world(&mut creatures, &mut WorldTiles::new(), &config, 1);
    assert_eq!(creatures[0].health, 50);
}

#[test]
fn eat_ignores_adjacent_creature() {
    let mut creatures = vec![
        creature_at(0, 0, RUNNER),
        Creature {
            id: "other".into(),
            x: 1,
            y: 0,
            ..creature_at(0, 0, IDLE)
        },
    ];
    const EATER: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "eat" (func $eat (param i32) (result i32)))
  (func (export "tick")
    i32.const 0
    call $eat
    drop
    call $sleep)
)
"#;
    creatures[0].wasm = compile_wat(EATER).unwrap();
    creatures[0].code = EATER.into();
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 1);
    assert_eq!(creatures.len(), 2);
    assert!(creatures.iter().any(|c| c.id == "other"));
}

#[test]
fn scavenger_chases_distant_corpse() {
    let mut tiles = empty_tiles();
    tiles.insert(
        (4, 0),
        WorldTile::Corpse {
            energy: 1_000_000,
            death_reason: DeathReason::EnergyFloor,
        },
    );
    let mut creatures = vec![creature_at(0, 0, SCAVENGER)];
    tick_world(&mut creatures, &mut tiles, &default_config(), 1);
    assert_eq!(creatures[0].x, 2);
}

#[test]
fn prey_flees_east_threat() {
    let mut creatures = vec![
        creature_at(0, 0, PREY),
        Creature {
            id: "pred".into(),
            x: 1,
            y: 0,
            ..creature_at(0, 0, IDLE)
        },
    ];
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 1);
    let prey = creatures.iter().find(|c| c.id == "c").expect("prey alive");
    assert_eq!(prey.x, -2);
}

#[test]
fn suicide_credits_human_owner() {
    const SUICIDE_NOW: &str = r#"
(module
  (import "terrarium" "suicide" (func $suicide))
  (func (export "tick") (call $suicide))
)
"#;
    let mut tiles = empty_tiles();
    let mut creatures = vec![Creature {
        owner_uid: "human".into(),
        energy: 2_000_000,
        wasm: compile_wat(SUICIDE_NOW).unwrap(),
        code: SUICIDE_NOW.into(),
        ..creature_at(0, 0, IDLE)
    }];
    let result = tick_world(&mut creatures, &mut tiles, &default_config(), 1);
    assert!(creatures.is_empty());
    assert_eq!(result.credit_payouts.len(), 1);
    assert_eq!(result.credit_payouts[0].0, "human");
    assert!(result.credit_payouts[0].1 > 1_000_000);
}

#[test]
fn suicide_leaves_no_corpse() {
    let mut tiles = empty_tiles();
    let mut creatures = vec![Creature {
        energy: 2_000_000,
        ..creature_at(5, 5, KAMIKAZE)
    }];
    tick_world(&mut creatures, &mut tiles, &default_config(), 1);
    assert!(creatures.is_empty());
    assert!(!tiles.contains_key(&(5, 5)));
}

#[test]
fn broadcast_delivers_to_neighbor() {
    let mut creatures = vec![
        creature_at(0, 0, BEACON),
        Creature {
            id: "b".into(),
            x: 1,
            y: 0,
            ..creature_at(0, 0, IDLE)
        },
    ];
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 1);
    assert_eq!(creatures[1].inbox.len(), 1);
    assert_eq!(creatures[1].inbox[0].byte, 190);
    assert!(creatures[1].inbox[0].broadcast);
}

#[test]
fn uptime_reports_age_in_ticks() {
    const CODE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "uptime" (func $uptime (result i32)))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (func (export "tick")
    call $uptime
    i32.const 3
    i32.ne
    if (return) end
    i32.const 0
    call $move
    drop
    call $sleep)
)
"#;
    let mut creatures = vec![Creature {
        born_tick: 7,
        wasm: compile_wat(CODE).unwrap(),
        code: CODE.into(),
        ..creature_at(0, 0, IDLE)
    }];
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 10);
    assert_eq!(creatures[0].x, 1);
}

#[test]
fn random_byte_import_works() {
    const CODE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (func (export "tick")
    call $rand
    drop
    call $sleep)
)
"#;
    let mut creatures = vec![Creature {
        wasm: compile_wat(CODE).unwrap(),
        code: CODE.into(),
        ..creature_at(0, 0, IDLE)
    }];
    tick_world(&mut creatures, &mut WorldTiles::new(), &default_config(), 7);
    assert_eq!(creatures.len(), 1);
}

#[test]
fn runner_wanders() {
    let c = run(RUNNER, 0, 0, &mut empty_tiles());
    assert!(c.x != 0 || c.y != 0);
}

#[test]
fn energy_nodes_spawn_when_budget_available() {
    let config = SimConfig {
        node_spawn_interval: 1,
        ..default_config()
    };
    let mut ledger = EnergyLedger {
        destroyed: 500_000,
        free_minted: 0,
    };
    let mut tiles = empty_tiles();
    let mut creatures = vec![creature_at(0, 0, IDLE)];
    run_tick(&mut creatures, &mut tiles, &mut ledger, &config, 10);
    assert!(
        tiles.values().any(|t| matches!(t, WorldTile::EnergyNode { .. })),
        "expected at least one energy node"
    );
    assert!(ledger.free_minted > 0);
}

#[test]
fn eat_energy_node_transfers_energy() {
    let mut tiles = empty_tiles();
    tiles.insert((1, 0), WorldTile::EnergyNode { energy: 250_000 });
    const EATER: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "eat" (func $eat (param i32) (result i32)))
  (func (export "tick")
    i32.const 0
    call $eat
    drop
    call $sleep)
)
"#;
    let before = 5_000_000_i64;
    let mut creatures = vec![Creature {
        energy: before,
        wasm: compile_wat(EATER).unwrap(),
        code: EATER.into(),
        ..creature_at(0, 0, IDLE)
    }];
    tick_world(&mut creatures, &mut tiles, &default_config(), 1);
    assert!(creatures[0].energy > before);
    assert!(creatures[0].energy >= before + 250_000 - 10);
    assert!(!tiles.contains_key(&(1, 0)));
}
