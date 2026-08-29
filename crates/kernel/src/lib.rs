//! Simulation kernel — ISA, assembler, tick loop.

pub mod assemble;
pub mod examples;
pub mod isa;
pub mod vm;
pub mod world_tile;

#[cfg(test)]
mod logic_tests;

pub use assemble::{assemble, AssembleError};
pub use examples::{ExampleProgram, ALL as EXAMPLE_PROGRAMS};
pub use vm::{run_tick, Creature, StepOutcome};
pub use world_tile::{WorldTile, WorldTiles};

pub const TICK_HZ: u32 = 10;
/// Floor energy — creature dies at this level; corpses always hold this much.
pub const CORPSE_ENERGY: i64 = 10;
