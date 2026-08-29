use crate::isa::{self, op, tile, STACK_MAX};
use crate::world_tile::{place_corpse, sense_kind, set_cell, blocks_movement, WorldTile, WorldTiles};
use crate::CORPSE_ENERGY;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    Continue,
    Sleep,
    Halt,
    Dead,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub energy: i64,
    pub owner_uid: String,
    pub bytecode: Vec<u8>,
    pub pc: usize,
    pub stack: Vec<i32>,
    pub alive: bool,
}

const INSTRUCTION_COST: i64 = 1;
const MOVE_EXTRA: i64 = 1;
const DIG_EXTRA: i64 = 1;
const PLACE_EXTRA: i64 = 1;

pub fn tick_creature(creature: &mut Creature, world: &mut WorldView<'_>) -> StepOutcome {
    if !creature.alive || creature.energy <= CORPSE_ENERGY {
        creature.alive = false;
        return StepOutcome::Dead;
    }

    loop {
        match step(creature, world) {
            StepOutcome::Continue => {
                if !creature.alive {
                    return StepOutcome::Dead;
                }
                continue;
            }
            other => return other,
        }
    }
}

fn step(creature: &mut Creature, world: &mut WorldView<'_>) -> StepOutcome {
    let Some(opcode) = creature.bytecode.get(creature.pc).copied() else {
        return StepOutcome::Halt;
    };

    if opcode != op::SLEEP && creature.energy < INSTRUCTION_COST {
        return StepOutcome::Sleep;
    }

    match opcode {
        op::HALT => StepOutcome::Halt,
        op::SLEEP => {
            creature.pc += 1;
            StepOutcome::Sleep
        }
        op::MOVE => {
            if !pay(creature, INSTRUCTION_COST + MOVE_EXTRA) {
                return StepOutcome::Sleep;
            }
            let Some(dir) = read_dir(creature) else {
                return StepOutcome::Sleep;
            };
            world.try_move(creature.id.as_str(), dir);
            creature.pc += 2;
            StepOutcome::Continue
        }
        op::DIG => {
            if !pay(creature, INSTRUCTION_COST + DIG_EXTRA) {
                return StepOutcome::Sleep;
            }
            let Some(dir) = read_dir(creature) else {
                return StepOutcome::Sleep;
            };
            if let Some((x, y)) = adjacent(creature.x, creature.y, dir) {
                world.set_tile(x, y, tile::EMPTY);
            }
            creature.pc += 2;
            StepOutcome::Continue
        }
        op::PLACE => {
            if !pay(creature, INSTRUCTION_COST + PLACE_EXTRA) {
                return StepOutcome::Sleep;
            }
            let Some(dir) = read_dir(creature) else {
                return StepOutcome::Sleep;
            };
            if let Some((x, y)) = adjacent(creature.x, creature.y, dir) {
                if world.tile_at(x, y) == tile::EMPTY && world.creature_at(x, y).is_none() {
                    world.set_tile(x, y, tile::SOLID);
                }
            }
            creature.pc += 2;
            StepOutcome::Continue
        }
        op::EAT => {
            if !pay(creature, INSTRUCTION_COST) {
                return StepOutcome::Sleep;
            }
            let Some(dir) = read_dir(creature) else {
                return StepOutcome::Sleep;
            };
            if let Some((x, y)) = adjacent(creature.x, creature.y, dir) {
                if let Some(gained) = world.take_corpse_energy(x, y) {
                    creature.energy += gained;
                }
            }
            creature.pc += 2;
            StepOutcome::Continue
        }
        op::SENSE => {
            if !pay(creature, INSTRUCTION_COST) {
                return StepOutcome::Sleep;
            }
            let Some(dir) = read_dir(creature) else {
                return StepOutcome::Sleep;
            };
            let kind = adjacent(creature.x, creature.y, dir)
                .map(|(x, y)| world.sense_at(x, y))
                .unwrap_or(tile::EMPTY);
            if push(creature, kind).is_err() {
                return StepOutcome::Sleep;
            }
            creature.pc += 2;
            StepOutcome::Continue
        }
        op::ENERGY => {
            if !pay(creature, INSTRUCTION_COST) {
                return StepOutcome::Sleep;
            }
            if push(creature, creature.energy.clamp(0, i32::MAX as i64) as i32).is_err() {
                return StepOutcome::Sleep;
            }
            creature.pc += 1;
            StepOutcome::Continue
        }
        op::POP => {
            if !pay(creature, INSTRUCTION_COST) {
                return StepOutcome::Sleep;
            }
            if pop(creature).is_err() {
                return StepOutcome::Sleep;
            }
            creature.pc += 1;
            StepOutcome::Continue
        }
        op::DUP => {
            if !pay(creature, INSTRUCTION_COST) {
                return StepOutcome::Sleep;
            }
            let Some(&top) = creature.stack.last() else {
                return StepOutcome::Sleep;
            };
            if push(creature, top).is_err() {
                return StepOutcome::Sleep;
            }
            creature.pc += 1;
            StepOutcome::Continue
        }
        op::PUSH => {
            if !pay(creature, INSTRUCTION_COST) {
                return StepOutcome::Sleep;
            }
            let Some(value) = read_i16(creature, creature.pc + 1) else {
                return StepOutcome::Sleep;
            };
            if push(creature, value as i32).is_err() {
                return StepOutcome::Sleep;
            }
            creature.pc += 3;
            StepOutcome::Continue
        }
        op::JMP => jump(creature, false, false),
        op::JZ => jump(creature, true, true),
        op::JNZ => jump(creature, true, false),
        op::EQ => binop(creature, |a, b| i32::from(a == b)),
        op::LT => binop(creature, |a, b| i32::from(a < b)),
        op::ADD => binop(creature, |a, b| a.wrapping_add(b)),
        op::SUB => binop(creature, |a, b| b.wrapping_sub(a)),
        op::SUICIDE => {
            creature.alive = false;
            StepOutcome::Dead
        }
        _ => StepOutcome::Sleep,
    }
}

fn jump(creature: &mut Creature, cond: bool, on_zero: bool) -> StepOutcome {
    if !pay(creature, INSTRUCTION_COST) {
        return StepOutcome::Sleep;
    }
    if cond {
        let top = match pop(creature) {
            Ok(v) => v,
            Err(()) => return StepOutcome::Sleep,
        };
        if (top == 0) != on_zero {
            creature.pc += 3;
            return StepOutcome::Continue;
        }
    }
    let Some(offset) = read_i16(creature, creature.pc + 1) else {
        return StepOutcome::Sleep;
    };
    let next = creature.pc as i32 + 3 + offset as i32;
    if next < 0 || next as usize >= creature.bytecode.len() {
        return StepOutcome::Sleep;
    }
    creature.pc = next as usize;
    StepOutcome::Continue
}

fn binop(creature: &mut Creature, f: fn(i32, i32) -> i32) -> StepOutcome {
    if !pay(creature, INSTRUCTION_COST) {
        return StepOutcome::Sleep;
    }
    let Some(a) = pop(creature).ok() else {
        return StepOutcome::Sleep;
    };
    let Some(b) = pop(creature).ok() else {
        return StepOutcome::Sleep;
    };
    if push(creature, f(a, b)).is_err() {
        return StepOutcome::Sleep;
    }
    creature.pc += 1;
    StepOutcome::Continue
}

fn pay(creature: &mut Creature, cost: i64) -> bool {
    if creature.energy < cost {
        return false;
    }
    creature.energy -= cost;
    if creature.energy <= CORPSE_ENERGY {
        creature.alive = false;
    }
    true
}

fn push(creature: &mut Creature, value: i32) -> Result<(), ()> {
    if creature.stack.len() >= STACK_MAX {
        return Err(());
    }
    creature.stack.push(value);
    Ok(())
}

fn pop(creature: &mut Creature) -> Result<i32, ()> {
    creature.stack.pop().ok_or(())
}

fn read_dir(creature: &Creature) -> Option<u8> {
    creature.bytecode.get(creature.pc + 1).copied()
}

fn read_i16(creature: &Creature, pc: usize) -> Option<i16> {
    let bytes = creature.bytecode.get(pc..pc + 2)?;
    Some(i16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn adjacent(x: i32, y: i32, dir: u8) -> Option<(i32, i32)> {
    Some(match dir {
        isa::dir::N => (x, y - 1),
        isa::dir::E => (x + 1, y),
        isa::dir::S => (x, y + 1),
        isa::dir::W => (x - 1, y),
        _ => return None,
    })
}

pub struct WorldView<'a> {
    positions: &'a mut Vec<(String, i32, i32)>,
    tiles: &'a mut WorldTiles,
}

impl<'a> WorldView<'a> {
    pub fn new(
        positions: &'a mut Vec<(String, i32, i32)>,
        tiles: &'a mut WorldTiles,
    ) -> Self {
        Self { positions, tiles }
    }

    fn pos_index(&self, id: &str) -> Option<usize> {
        self.positions.iter().position(|(cid, _, _)| cid == id)
    }

    pub fn creature_at(&self, x: i32, y: i32) -> Option<&str> {
        self.positions
            .iter()
            .find(|(_, px, py)| *px == x && *py == y)
            .map(|(id, _, _)| id.as_str())
    }

    pub fn tile_at(&self, x: i32, y: i32) -> i32 {
        sense_kind(self.tiles, x, y, false)
    }

    pub fn sense_at(&self, x: i32, y: i32) -> i32 {
        sense_kind(
            self.tiles,
            x,
            y,
            self.creature_at(x, y).is_some(),
        )
    }

    pub fn set_tile(&mut self, x: i32, y: i32, kind: i32) {
        set_cell(self.tiles, x, y, kind);
    }

    fn take_corpse_energy(&mut self, x: i32, y: i32) -> Option<i64> {
        match self.tiles.get(&(x, y)).copied() {
            Some(WorldTile::Corpse { energy }) => {
                self.tiles.remove(&(x, y));
                Some(energy)
            }
            _ => None,
        }
    }

    pub fn try_move(&mut self, id: &str, dir: u8) {
        let Some(idx) = self.pos_index(id) else {
            return;
        };
        let (x, y) = (self.positions[idx].1, self.positions[idx].2);
        let Some((nx, ny)) = adjacent(x, y, dir) else {
            return;
        };
        if blocks_movement(self.tiles, nx, ny) {
            return;
        }
        if self
            .positions
            .iter()
            .any(|(cid, px, py)| cid != id && *px == nx && *py == ny)
        {
            return;
        }
        self.positions[idx].1 = nx;
        self.positions[idx].2 = ny;
    }
}

pub fn run_tick(creatures: &mut Vec<Creature>, tiles: &mut WorldTiles) {
    let mut positions: Vec<(String, i32, i32)> = creatures
        .iter()
        .filter(|c| c.alive)
        .map(|c| (c.id.clone(), c.x, c.y))
        .collect();

    let mut dead: Vec<(i32, i32)> = Vec::new();

    for creature in creatures.iter_mut().filter(|c| c.alive) {
        let idx = positions.iter().position(|(id, _, _)| id == &creature.id);
        if let Some(i) = idx {
            creature.x = positions[i].1;
            creature.y = positions[i].2;
        }
        let mut world = WorldView::new(&mut positions, tiles);
        tick_creature(creature, &mut world);
        if let Some(i) = positions.iter().position(|(id, _, _)| id == &creature.id) {
            creature.x = positions[i].1;
            creature.y = positions[i].2;
        }
        if !creature.alive {
            dead.push((creature.x, creature.y));
        }
    }

    creatures.retain(|c| c.alive);

    for (x, y) in dead {
        place_corpse(tiles, x, y, CORPSE_ENERGY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_tile::WorldTiles;
    use crate::assemble;

    #[test]
    fn all_examples_assemble() {
        for example in crate::EXAMPLE_PROGRAMS {
            assemble(example.code).unwrap_or_else(|err| {
                panic!("example `{}` failed at line {}: {}", example.id, err.line, err.message)
            });
        }
    }

    #[test]
    fn tunnel_east_moves_over_ten_ticks() {
        let code = crate::EXAMPLE_PROGRAMS
            .iter()
            .find(|e| e.id == "tunnel")
            .unwrap()
            .code;
        let mut creatures = vec![Creature {
            id: "a".into(),
            x: 0,
            y: 0,
            energy: 10_000,
            owner_uid: "u".into(),
            bytecode: assemble(code).unwrap(),
            pc: 0,
            stack: vec![],
            alive: true,
        }];
        let mut tiles = WorldTiles::new();
        for _ in 0..10 {
            run_tick(&mut creatures, &mut tiles);
        }
        assert_eq!(creatures[0].x, 10);
        assert_eq!(creatures[0].y, 0);
    }

    #[test]
    fn idle_does_not_move() {
        let code = crate::EXAMPLE_PROGRAMS
            .iter()
            .find(|e| e.id == "idle")
            .unwrap()
            .code;
        let mut creatures = vec![Creature {
            id: "a".into(),
            x: 3,
            y: 4,
            energy: 100,
            owner_uid: "u".into(),
            bytecode: assemble(code).unwrap(),
            pc: 0,
            stack: vec![],
            alive: true,
        }];
        let mut tiles = WorldTiles::new();
        for _ in 0..5 {
            run_tick(&mut creatures, &mut tiles);
        }
        assert_eq!(creatures[0].x, 3);
        assert_eq!(creatures[0].y, 4);
    }
}
