//! Simulation engine — WAT/WASM creatures, tick loop.

pub mod abi;
pub mod energy_ledger;
pub mod events;
pub mod food;
pub mod guest_mem;
pub mod hex;
pub mod host;
pub mod sandbox;
pub mod sim_config;
pub mod vm;
pub mod wat;
pub mod world_tile;

pub use abi::{
    Payload, ACTION_ENERGY, CORPSE_ENERGY, ENERGY_PER_OPCODE, ENERGY_SCALE, OPCODES_PER_TICK,
    PAYLOAD_SIZE,
};
pub use energy_ledger::EnergyLedger;
pub use events::{CreatureAction, DeathReason, InboxDelivery, TickResult, WorldEvent};
pub use host::PendingAction;
pub use sandbox::{
    run_sandbox, SandboxBench, SandboxFrame, SandboxRequest, SandboxResult, SandboxTile,
};
pub use sim_config::SimConfig;
pub use terrarium_test_spec::{
    parse_tests, AssertionResult, Diagnostic as TestDiagnostic, ParsedTests, TestSpec,
};
pub use vm::{run_tick, validate_external_broadcast, validate_external_signal, Creature, Signal};
pub use wat::{compile_wat, wat_move_once, WatError, WAT_IDLE, WAT_MOVE_FWD_LOOP};
pub use world_tile::{WorldTile, WorldTiles};

pub const TICK_HZ: u32 = 2;
