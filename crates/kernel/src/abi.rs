//! Host ABI constants shared by the WASM runtime.

pub mod dir {
    pub const N: i32 = 0;
    pub const E: i32 = 1;
    pub const S: i32 = 2;
    pub const W: i32 = 3;
}

pub mod tile {
    pub const EMPTY: i32 = 0;
    pub const SOLID: i32 = 1;
    pub const CREATURE: i32 = 2;
    pub const CORPSE: i32 = 3;
}

pub const RECV_STRUCT_SIZE: i32 = 36;

/// Base energy unit — values are in millions (corpse floor = 1M).
pub const ENERGY_SCALE: i64 = 100_000;

pub const CORPSE_ENERGY: i64 = 10 * ENERGY_SCALE;

/// Max WASM opcodes (including host imports) per creature per sim tick.
pub const OPCODES_PER_TICK: u64 = 10_000;
/// In-game energy charged per opcode executed.
pub const ENERGY_PER_OPCODE: i64 = 1;
