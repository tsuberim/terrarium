//! Host ABI constants shared by the WASM runtime.

/// Pointy-top axial directions: E, NE, NW, W, SW, SE.
pub mod dir {
    pub const E: i32 = 0;
    pub const NE: i32 = 1;
    pub const NW: i32 = 2;
    pub const W: i32 = 3;
    pub const SW: i32 = 4;
    pub const SE: i32 = 5;
    pub const COUNT: i32 = 6;
}

pub mod tile {
    pub const EMPTY: i32 = 0;
    pub const SOLID: i32 = 1;
    pub const CREATURE: i32 = 2;
    pub const CORPSE: i32 = 3;
    pub const NODE: i32 = 4;
}

pub const RECV_STRUCT_SIZE: i32 = 36;

/// Cell snapshot written by `sense` (little-endian).
pub const SENSE_STRUCT_SIZE: i32 = 24;
pub mod sense_off {
    pub const KIND: i32 = 0;
    pub const ENERGY: i32 = 8;
    pub const HEALTH: i32 = 16;
    pub const MAX_HEALTH: i32 = 20;
}

/// Base energy unit — values are in millions (corpse floor = 1M).
pub const ENERGY_SCALE: i64 = 100_000;

pub const CORPSE_ENERGY: i64 = 10 * ENERGY_SCALE;

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
