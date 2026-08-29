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

/// Max WASM opcodes (including host imports) per creature per sim tick.
pub const OPCODES_PER_TICK: u64 = 10_000;
/// In-game energy charged per opcode executed.
pub const ENERGY_PER_OPCODE: i64 = 1;
