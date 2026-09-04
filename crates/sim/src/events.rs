use crate::abi::Payload;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathReason {
    EnergyFloor,
    OutOfEnergy,
    OutOfGas,
    EmptyProgram,
    InvalidProgram,
    WasmTrap,
    OutOfVision,
    BadDirection,
    SpawnEnergyTooLow,
    SignalUnknownTarget,
    SignalOutOfRange,
    Suicide,
    SpawnFailed,
    SignalFailed,
    Killed,
    Eaten,
}

impl DeathReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnergyFloor => "energy_floor",
            Self::OutOfEnergy => "out_of_energy",
            Self::OutOfGas => "out_of_gas",
            Self::EmptyProgram => "empty_program",
            Self::InvalidProgram => "invalid_program",
            Self::WasmTrap => "wasm_trap",
            Self::OutOfVision => "out_of_vision",
            Self::BadDirection => "bad_direction",
            Self::SpawnEnergyTooLow => "spawn_energy_too_low",
            Self::SignalUnknownTarget => "signal_unknown_target",
            Self::SignalOutOfRange => "signal_out_of_range",
            Self::Suicide => "suicide",
            Self::SpawnFailed => "spawn_failed",
            Self::SignalFailed => "signal_failed",
            Self::Killed => "killed",
            Self::Eaten => "eaten",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "energy_floor" => Self::EnergyFloor,
            "out_of_energy" => Self::OutOfEnergy,
            "out_of_gas" => Self::OutOfGas,
            "empty_program" => Self::EmptyProgram,
            "invalid_program" => Self::InvalidProgram,
            "wasm_trap" => Self::WasmTrap,
            "out_of_vision" => Self::OutOfVision,
            "bad_direction" => Self::BadDirection,
            "spawn_energy_too_low" => Self::SpawnEnergyTooLow,
            "signal_unknown_target" => Self::SignalUnknownTarget,
            "signal_out_of_range" => Self::SignalOutOfRange,
            "suicide" => Self::Suicide,
            "spawn_failed" => Self::SpawnFailed,
            "signal_failed" => Self::SignalFailed,
            "killed" => Self::Killed,
            "eaten" => Self::Eaten,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InboxDelivery {
    pub creature_id: u64,
    pub sender: u64,
    pub payload: Payload,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldEvent {
    Signal {
        from_id: u64,
        from_x: i32,
        from_y: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        to_id: Option<u64>,
        broadcast: bool,
    },
    Death {
        creature_id: u64,
        owner_uid: String,
        x: i32,
        y: i32,
        reason: DeathReason,
        facing: u8,
        energy: i64,
        health: i32,
        max_health: i32,
    },
    Spawn {
        creature_id: u64,
        parent_id: u64,
        parent_x: i32,
        parent_y: i32,
        x: i32,
        y: i32,
    },
    Hit {
        actor_id: u64,
        victim_id: u64,
        x: i32,
        y: i32,
        damage: i32,
        victim_health: i32,
    },
    Eat {
        actor_id: u64,
        x: i32,
        y: i32,
        energy: i64,
        tile_kind: i64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CreatureAction {
    Move {
        creature_id: u64,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },
    Rotate {
        creature_id: u64,
        from_facing: u8,
        to_facing: u8,
    },
    Eat {
        creature_id: u64,
        x: i32,
        y: i32,
    },
    Hit {
        creature_id: u64,
        x: i32,
        y: i32,
    },
}

#[derive(Clone, Debug, Default)]
pub struct TickResult {
    pub events: Vec<WorldEvent>,
    pub actions: Vec<CreatureAction>,
    pub destroyed: i64,
    pub free_minted: i64,
    pub credit_payouts: Vec<(String, i64)>,
    pub tiles_dirty: HashSet<(i32, i32)>,
    pub inbox_deliveries: Vec<InboxDelivery>,
}
