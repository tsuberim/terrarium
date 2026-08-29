use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldEvent {
    Signal {
        from_id: String,
        from_x: i32,
        from_y: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        to_id: Option<String>,
        byte: u8,
        broadcast: bool,
    },
    Death {
        creature_id: String,
        owner_uid: String,
        x: i32,
        y: i32,
        reason: DeathReason,
    },
    Spawn {
        creature_id: String,
        parent_id: String,
        x: i32,
        y: i32,
    },
    Hit {
        actor_id: String,
        victim_id: String,
        x: i32,
        y: i32,
        damage: i32,
        victim_health: i32,
    },
    Eat {
        actor_id: String,
        x: i32,
        y: i32,
        energy: i64,
    },
}

#[derive(Clone, Debug, Default)]
pub struct TickResult {
    pub events: Vec<WorldEvent>,
    pub destroyed: i64,
    pub free_minted: i64,
    /// Suicide energy returned to human accounts (uid → glims).
    pub credit_payouts: Vec<(String, i64)>,
}
