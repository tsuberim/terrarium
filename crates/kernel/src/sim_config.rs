use serde::{Deserialize, Serialize};

use crate::abi::{ENERGY_SCALE, ENERGY_PER_OPCODE, OPCODES_PER_TICK, CORPSE_ENERGY};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimConfig {
    pub r_vis: i32,
    pub r_sig: i32,
    pub corpse_energy: i64,
    /// Max opcodes per creature tick (gas limit).
    pub opcodes_per_tick: u64,
    /// Energy cost per opcode executed.
    pub energy_per_opcode: i64,
    pub move_extra: i64,
    pub dig_extra: i64,
    pub place_extra: i64,
    pub signal_inbox_cap: usize,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            r_vis: 5,
            r_sig: 5,
            corpse_energy: CORPSE_ENERGY,
            opcodes_per_tick: OPCODES_PER_TICK,
            energy_per_opcode: ENERGY_PER_OPCODE,
            move_extra: ENERGY_SCALE,
            dig_extra: ENERGY_SCALE,
            place_extra: ENERGY_SCALE,
            signal_inbox_cap: 8,
        }
    }
}

impl SimConfig {
    pub fn in_square(&self, dx: i32, dy: i32, r: i32) -> bool {
        dx.abs() <= r && dy.abs() <= r
    }
}
