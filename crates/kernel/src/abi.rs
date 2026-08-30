//! Host ABI constants shared by the WASM runtime.

/// Absolute world directions (pointy-top axial): E, NE, NW, W, SW, SE.
pub mod dir {
    pub const E: i32 = 0;
    pub const NE: i32 = 1;
    pub const NW: i32 = 2;
    pub const W: i32 = 3;
    pub const SW: i32 = 4;
    pub const SE: i32 = 5;
    pub const COUNT: i32 = 6;
}

/// Relative to body facing for `rotate` only. `move`, `eat`, `hit`, `dig`, `place`, and
/// `spawn` only affect the forward cell (relative 0); use `rotate` to face targets first.
pub mod rel_dir {
    pub const FWD: i32 = 0;
    pub const FWD_R: i32 = 1;
    pub const BACK_R: i32 = 2;
    pub const BACK: i32 = 3;
    pub const BACK_L: i32 = 4;
    pub const FWD_L: i32 = 5;
    pub const COUNT: i32 = 6;
}

pub mod tile {
    pub const EMPTY: i32 = 0;
    pub const SOLID: i32 = 1;
    pub const CREATURE: i32 = 2;
    pub const CORPSE: i32 = 3;
    pub const FOOD: i32 = 4;
}

pub const RECV_STRUCT_SIZE: i32 = 36;

/// Cell snapshot written by `sense` (little-endian, 24 bytes).
pub const SENSE_STRUCT_SIZE: i32 = 24;
pub mod sense_off {
    pub const KIND: i32 = 0;
    /// Creature facing 0–5 when kind=CREATURE; -1 otherwise (was padding before energy).
    pub const ORIENTATION: i32 = 4;
    pub const ENERGY: i32 = 8;
    pub const HEALTH: i32 = 16;
    pub const MAX_HEALTH: i32 = 20;
}

/// Base energy unit — values are in millions (corpse floor = 1M).
pub const ENERGY_SCALE: i64 = 100_000;

pub const CORPSE_ENERGY: i64 = 10 * ENERGY_SCALE;

/// Minimum energy a spawn transfer must exceed (above corpse floor).
pub const SPAWN_MIN_ENERGY: i64 = CORPSE_ENERGY + ACTION_ENERGY;

/// Percent of a creature's remaining energy that becomes a corpse on death.
pub const CORPSE_YIELD_PERCENT: i64 = 80;

pub fn corpse_yield_energy(creature_energy: i64) -> i64 {
    creature_energy.max(0) * CORPSE_YIELD_PERCENT / 100
}

/// Max WASM opcodes (including host imports) per creature per sim tick.
pub const OPCODES_PER_TICK: u64 = 25_000;
/// In-game energy charged per opcode executed.
pub const ENERGY_PER_OPCODE: i64 = 1;
/// Action surcharge (move, hit, dig, place, idle regen) — quarter glim keeps creatures active longer.
pub const ACTION_ENERGY: i64 = ENERGY_SCALE / 4;
