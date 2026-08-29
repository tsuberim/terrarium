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
    pub hit_extra: i64,
    pub signal_inbox_cap: usize,
    pub max_health: i32,
    pub hit_damage: i32,
    pub health_regen: i32,
    pub health_regen_cost: i64,
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
            hit_extra: ENERGY_SCALE,
            signal_inbox_cap: 8,
            max_health: 100,
            hit_damage: 34,
            health_regen: 5,
            health_regen_cost: ENERGY_SCALE,
        }
    }
}

impl SimConfig {
    pub fn in_hex_range(&self, dq: i32, dr: i32, r: i32) -> bool {
        crate::hex::in_range(dq, dr, r)
    }
}
