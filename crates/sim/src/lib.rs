//! Simulation engine — WAT/WASM creatures, tick loop.

pub mod abi;
pub mod energy_ledger;
pub mod events;
pub mod examples;
pub mod food;
pub mod hex;
pub mod host;
pub mod sandbox;
pub mod sim_config;
pub mod vm;
pub mod wat;
pub mod world_tile;

#[cfg(test)]
mod logic_tests;

pub use abi::{ACTION_ENERGY, CORPSE_ENERGY, ENERGY_PER_OPCODE, ENERGY_SCALE, OPCODES_PER_TICK};
pub use energy_ledger::EnergyLedger;
pub use events::{CreatureAction, DeathReason, TickResult, WorldEvent};
pub use examples::{ExampleProgram, ALL as EXAMPLE_PROGRAMS};
pub use sandbox::{
    run_sandbox, SandboxBench, SandboxFrame, SandboxRequest, SandboxResult, SandboxTile,
};
pub use sim_config::SimConfig;
pub use terrarium_test_spec::{
    parse_tests, AssertionResult, Diagnostic as TestDiagnostic, ParsedTests, TestSpec,
};
pub use vm::{run_tick, Creature, Signal};
pub use wat::{compile_wat, WatError};
pub use world_tile::{WorldTile, WorldTiles};

pub const TICK_HZ: u32 = 2;
