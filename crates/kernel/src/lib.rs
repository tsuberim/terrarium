//! Simulation kernel — WAT/WASM creatures, tick loop.

pub mod abi;
pub mod events;
pub mod examples;
pub mod host;
pub mod sim_config;
pub mod vm;
pub mod wat;
pub mod world_tile;

#[cfg(test)]
mod logic_tests;

pub use events::{DeathReason, TickResult, WorldEvent};
pub use examples::{ExampleProgram, ALL as EXAMPLE_PROGRAMS};
pub use sim_config::SimConfig;
pub use vm::{run_tick, Creature, Signal};
pub use wat::{compile_wat, WatError};
pub use world_tile::{WorldTile, WorldTiles};
pub use abi::{CORPSE_ENERGY, ENERGY_PER_OPCODE, ENERGY_SCALE, OPCODES_PER_TICK};

pub const TICK_HZ: u32 = 10;
