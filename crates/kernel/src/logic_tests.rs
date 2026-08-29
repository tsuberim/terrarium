//! Semantic tests for the VM — stack order, control flow, world interactions.

use crate::assemble;
use crate::isa::tile;
use crate::vm::{run_tick, Creature};
use crate::world_tile::{WorldTile, WorldTiles};
use crate::CORPSE_ENERGY;

fn creature_at(x: i32, y: i32, bytecode: Vec<u8>) -> Creature {
    Creature {
        id: "c".into(),
        x,
        y,
        energy: 1000,
        owner_uid: "u".into(),
        bytecode,
        pc: 0,
        stack: vec![],
        alive: true,
    }
}

fn run(source: &str, x: i32, y: i32, tiles: &mut WorldTiles) -> Creature {
    let mut creatures = vec![creature_at(x, y, assemble(source).unwrap())];
    run_tick(&mut creatures, tiles);
    creatures.remove(0)
}

#[test]
fn stack_underflow_stalls_not_kills() {
    let mut creatures = vec![creature_at(0, 0, vec![crate::isa::op::POP])];
    run_tick(&mut creatures, &mut WorldTiles::new());
    assert!(creatures[0].alive);
    assert_eq!(creatures[0].pc, 0);
}

#[test]
fn eq_pops_top_second() {
    let code = "push 2\npush 1\neq\n";
    let c = run(code, 0, 0, &mut WorldTiles::new());
    assert_eq!(c.stack, vec![0]);
    assert_eq!(c.pc, assemble(code).unwrap().len());
}

#[test]
fn sub_is_b_minus_a() {
    let code = "push 10\npush 3\nsub\n";
    let c = run(code, 0, 0, &mut WorldTiles::new());
    assert_eq!(c.stack, vec![7]);
}

#[test]
fn blocked_move_costs_energy_but_stays_put() {
    let mut tiles = WorldTiles::new();
    tiles.insert((1, 0), WorldTile::Solid);
    let code = "move e\nsleep\n";
    let before = 100_i64;
    let mut creatures = vec![Creature {
        energy: before,
        ..creature_at(0, 0, assemble(code).unwrap())
    }];
    run_tick(&mut creatures, &mut tiles);
    assert_eq!(creatures[0].x, 0);
    assert_eq!(creatures[0].energy, before - 2);
}

#[test]
fn wall_example_places_solid_north_when_open() {
    let code = crate::EXAMPLE_PROGRAMS
        .iter()
        .find(|e| e.id == "wall")
        .unwrap()
        .code;
    let mut tiles = WorldTiles::new();
    run_tick(&mut creatures_vec(code, 0, 0), &mut tiles);
    assert_eq!(tiles.get(&(0, -1)), Some(&WorldTile::Solid));
}

#[test]
fn wall_example_skips_place_when_already_solid_north() {
    let code = crate::EXAMPLE_PROGRAMS
        .iter()
        .find(|e| e.id == "wall")
        .unwrap()
        .code;
    let mut tiles = WorldTiles::new();
    tiles.insert((0, -1), WorldTile::Solid);
    run_tick(&mut creatures_vec(code, 0, 0), &mut tiles);
    assert_eq!(tiles.len(), 1);
}

#[test]
fn sense_pushes_creature_on_occupied_cell() {
    let mut creatures = vec![
        creature_at(0, 0, assemble("sense e\nsleep\n").unwrap()),
        Creature {
            id: "other".into(),
            x: 1,
            y: 0,
            ..creature_at(0, 0, assemble("sleep\n").unwrap())
        },
    ];
    run_tick(&mut creatures, &mut WorldTiles::new());
    assert_eq!(creatures[0].stack, vec![tile::CREATURE]);
}

#[test]
fn death_leaves_corpse_with_const_energy() {
    let mut tiles = WorldTiles::new();
    let mut creatures = vec![Creature {
        energy: CORPSE_ENERGY + 5,
        bytecode: assemble("suicide\n").unwrap(),
        ..creature_at(5, 5, vec![])
    }];
    run_tick(&mut creatures, &mut tiles);
    assert!(creatures.is_empty());
    assert_eq!(
        tiles.get(&(5, 5)),
        Some(&WorldTile::Corpse { energy: CORPSE_ENERGY })
    );
}

#[test]
fn dies_when_energy_reaches_floor() {
    let mut tiles = WorldTiles::new();
    let mut creatures = vec![Creature {
        energy: CORPSE_ENERGY + 1,
        bytecode: assemble("move e\n").unwrap(),
        ..creature_at(0, 0, vec![])
    }];
    run_tick(&mut creatures, &mut tiles);
    assert!(creatures.is_empty());
    assert_eq!(
        tiles.get(&(1, 0)),
        Some(&WorldTile::Corpse { energy: CORPSE_ENERGY })
    );
}

#[test]
fn eat_transfers_corpse_energy() {
    let mut tiles = WorldTiles::new();
    tiles.insert((1, 0), WorldTile::Corpse { energy: CORPSE_ENERGY });
    let c = run("eat e\nsleep\n", 0, 0, &mut tiles);
    assert_eq!(c.energy, 1000 - 1 + CORPSE_ENERGY);
    assert!(!tiles.contains_key(&(1, 0)));
}

fn creatures_vec(code: &str, x: i32, y: i32) -> Vec<Creature> {
    vec![creature_at(x, y, assemble(code).unwrap())]
}
