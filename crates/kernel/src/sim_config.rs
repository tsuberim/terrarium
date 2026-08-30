use serde::{Deserialize, Serialize};

use crate::abi::{ACTION_ENERGY, ENERGY_SCALE, ENERGY_PER_OPCODE, OPCODES_PER_TICK, CORPSE_ENERGY};

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
    pub rotate_extra: i64,
    /// Frontal vision half-width in hex direction steps (1 = ±60°, 120° total).
    pub vis_half_arc: i32,
    pub signal_inbox_cap: usize,
    pub max_health: i32,
    pub hit_damage: i32,
    pub health_regen: i32,
    pub health_regen_cost: i64,
    /// Energy reserved from budget when a food tile spawns.
    pub food_nominal_energy: i64,
    pub food_spawn_interval: u64,
    pub food_spawn_attempts: u32,
    pub max_active_food: u32,
    pub food_spawn_radius: i32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            r_vis: 5,
            r_sig: 5,
            corpse_energy: CORPSE_ENERGY,
            opcodes_per_tick: OPCODES_PER_TICK,
            energy_per_opcode: ENERGY_PER_OPCODE,
            move_extra: ACTION_ENERGY,
            dig_extra: ACTION_ENERGY,
            place_extra: ACTION_ENERGY,
            hit_extra: ACTION_ENERGY,
            rotate_extra: ACTION_ENERGY,
            vis_half_arc: 1,
            signal_inbox_cap: 8,
            max_health: 100,
            hit_damage: 34,
            health_regen: 5,
            health_regen_cost: ACTION_ENERGY,
            food_nominal_energy: ENERGY_SCALE * 2,
            food_spawn_interval: 5,
            food_spawn_attempts: 6,
            max_active_food: 80,
            food_spawn_radius: 32,
        }
    }
}

impl SimConfig {
    pub fn in_hex_range(&self, dq: i32, dr: i32, r: i32) -> bool {
        crate::hex::in_range(dq, dr, r)
    }
}
