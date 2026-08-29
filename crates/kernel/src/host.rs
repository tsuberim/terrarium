//! Host syscalls for creature WASM modules.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use uuid::Uuid;
use wasmtime::{Caller, Engine, Error, Linker, Module, Store, Config};

use crate::abi::{RECV_STRUCT_SIZE, SENSE_STRUCT_SIZE};
use crate::energy_ledger::EnergyLedger;
use crate::events::DeathReason;
use crate::sim_config::SimConfig;
use crate::vm::{mark_dead, Creature, Signal, Snapshot};
use crate::world_tile::{sense_kind, WorldTile, WorldTiles};

#[derive(Debug, Clone)]
pub enum PendingAction {
    Move { dir: u8 },
    Dig { dir: u8 },
    Place { dir: u8 },
    Eat { dir: u8 },
    Hit { dir: u8 },
    Spawn { dir: u8, energy: i64 },
    SignalTo {
        to_id: String,
        byte: u8,
        from_x: i32,
        from_y: i32,
    },
    SignalBroadcast {
        byte: u8,
        from_x: i32,
        from_y: i32,
    },
}

#[derive(Debug, Default)]
pub struct ThinkResult {
    pub actions: Vec<PendingAction>,
    pub suicide: bool,
}

pub struct HostState {
    creature: *mut Creature,
    snapshot: *const Snapshot,
    tiles: *const WorldTiles,
    config: *const SimConfig,
    ledger: *mut EnergyLedger,
    tick: u64,
    rng_seed: u64,
    rng_calls: u64,
    pub result: ThinkResult,
}

// HostState is only used on the sim thread.
unsafe impl Send for HostState {}

impl HostState {
    fn creature(&mut self) -> &mut Creature {
        unsafe { &mut *self.creature }
    }

    fn creature_ref(&self) -> &Creature {
        unsafe { &*self.creature }
    }

    fn snapshot(&self) -> &Snapshot {
        unsafe { &*self.snapshot }
    }

    fn tiles(&self) -> &WorldTiles {
        unsafe { &*self.tiles }
    }

    fn config(&self) -> &SimConfig {
        unsafe { &*self.config }
    }

    fn ledger(&mut self) -> &mut EnergyLedger {
        unsafe { &mut *self.ledger }
    }

    fn pay_action(&mut self, cost: i64) -> Result<(), Error> {
        let floor = self.config().corpse_energy;
        let energy = {
            let c = self.creature();
            if c.energy < cost {
                mark_dead(c, DeathReason::OutOfEnergy);
                return Err(Error::msg("out of energy"));
            }
            c.energy -= cost;
            c.energy
        };
        self.ledger().record_destroy(cost);
        if energy <= floor {
            mark_dead(self.creature(), DeathReason::EnergyFloor);
            return Err(Error::msg("energy floor"));
        }
        Ok(())
    }

    fn valid_dir(d: i32) -> Result<u8, Error> {
        if (0..crate::abi::dir::COUNT).contains(&d) {
            Ok(d as u8)
        } else {
            Err(Error::msg("bad direction"))
        }
    }

    fn in_vis(&self, dq: i32, dr: i32) -> Result<(), Error> {
        if self.config().in_hex_range(dq, dr, self.config().r_vis) {
            Ok(())
        } else {
            Err(Error::msg("out of vision"))
        }
    }
}

pub fn link_host(linker: &mut Linker<HostState>) -> Result<(), wasmtime::Error> {
    linker.func_wrap("terrarium", "sleep", |_caller: Caller<'_, HostState>| Ok(()))?;

    linker.func_wrap("terrarium", "energy", |caller: Caller<'_, HostState>| {
        Ok(caller.data().creature_ref().energy)
    })?;

    linker.func_wrap("terrarium", "health", |caller: Caller<'_, HostState>| {
        Ok(i64::from(caller.data().creature_ref().health))
    })?;

    linker.func_wrap("terrarium", "pos_x", |caller: Caller<'_, HostState>| {
        Ok(caller.data().creature_ref().x)
    })?;

    linker.func_wrap("terrarium", "pos_y", |caller: Caller<'_, HostState>| {
        Ok(caller.data().creature_ref().y)
    })?;

    linker.func_wrap("terrarium", "uptime", |caller: Caller<'_, HostState>| {
        let born = caller.data().creature_ref().born_tick;
        Ok(caller.data().tick.saturating_sub(born) as i32)
    })?;

    linker.func_wrap("terrarium", "sense", |mut caller: Caller<'_, HostState>, dq: i32, dr: i32, ptr: i32| {
        caller.data_mut().in_vis(dq, dr)?;
        let c = caller.data().creature_ref();
        let x = c.x + dq;
        let y = c.y + dr;
        let snapshot = caller.data().snapshot();
        let tiles = caller.data().tiles();
        let has_creature = snapshot.id_at.contains_key(&(x, y));
        let kind = sense_kind(tiles, x, y, has_creature);
        let mut energy = 0i64;
        let mut health = 0i32;
        let mut max_health = 0i32;
        if let Some(id) = snapshot.id_at.get(&(x, y)) {
            energy = snapshot.energy.get(id).copied().unwrap_or(0);
            health = snapshot.health.get(id).copied().unwrap_or(0);
            max_health = snapshot.max_health.get(id).copied().unwrap_or(0);
        } else if let Some(WorldTile::Corpse { energy: e, .. }) = tiles.get(&(x, y)) {
            energy = *e;
        } else if let Some(WorldTile::EnergyNode { energy: e }) = tiles.get(&(x, y)) {
            energy = *e;
        }
        write_sense_struct(&mut caller, ptr, kind, energy, health, max_health)?;
        Ok(1)
    })?;

    linker.func_wrap("terrarium", "random_byte", |mut caller: Caller<'_, HostState>| {
        Ok(i32::from(next_random(caller.data_mut())))
    })?;

    linker.func_wrap("terrarium", "move", |mut caller: Caller<'_, HostState>, d: i32| {
        let dir = HostState::valid_dir(d)?;
        let extra = caller.data().config().move_extra;
        caller.data_mut().pay_action(extra)?;
        caller.data_mut().result.actions.push(PendingAction::Move { dir });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "dig", |mut caller: Caller<'_, HostState>, d: i32| {
        let dir = HostState::valid_dir(d)?;
        let extra = caller.data().config().dig_extra;
        caller.data_mut().pay_action(extra)?;
        caller.data_mut().result.actions.push(PendingAction::Dig { dir });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "place", |mut caller: Caller<'_, HostState>, d: i32| {
        let dir = HostState::valid_dir(d)?;
        let extra = caller.data().config().place_extra;
        caller.data_mut().pay_action(extra)?;
        caller.data_mut().result.actions.push(PendingAction::Place { dir });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "eat", |mut caller: Caller<'_, HostState>, d: i32| {
        let dir = HostState::valid_dir(d)?;
        caller.data_mut().result.actions.push(PendingAction::Eat { dir });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "hit", |mut caller: Caller<'_, HostState>, d: i32| {
        let dir = HostState::valid_dir(d)?;
        let extra = caller.data().config().hit_extra;
        caller.data_mut().pay_action(extra)?;
        caller.data_mut().result.actions.push(PendingAction::Hit { dir });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "spawn", |mut caller: Caller<'_, HostState>, d: i32, energy: i32| {
        let dir = HostState::valid_dir(d)?;
        let energy = energy as i64;
        let floor = caller.data().config().corpse_energy;
        if energy < floor {
            mark_dead(caller.data_mut().creature(), DeathReason::SpawnEnergyTooLow);
            return Err(Error::msg("spawn energy"));
        }
        caller
            .data_mut()
            .result
            .actions
            .push(PendingAction::Spawn { dir, energy });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "suicide", |mut caller: Caller<'_, HostState>| {
        caller.data_mut().result.suicide = true;
        Ok(())
    })?;

    linker.func_wrap("terrarium", "signal_broadcast", |mut caller: Caller<'_, HostState>, byte: i32| {
        let (x, y) = {
            let c = caller.data().creature_ref();
            (c.x, c.y)
        };
        caller.data_mut().result.actions.push(PendingAction::SignalBroadcast {
            byte: byte as u8,
            from_x: x,
            from_y: y,
        });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "signal_to", |mut caller: Caller<'_, HostState>, ptr: i32, byte: i32| {
        let id_bytes = read_bytes(&mut caller, ptr, 16)?;
        let to_id = uuid_from_bytes(&id_bytes).ok_or_else(|| Error::msg("bad uuid"))?;
        let snapshot = caller.data().snapshot();
        let Some(&(tx, ty)) = snapshot.positions.get(&to_id) else {
            mark_dead(caller.data_mut().creature(), DeathReason::SignalUnknownTarget);
            return Err(Error::msg("unknown target"));
        };
        let (sx, sy) = {
            let c = caller.data().creature_ref();
            (c.x, c.y)
        };
        if !in_sig_range(sx, sy, tx, ty, caller.data().config()) {
            mark_dead(caller.data_mut().creature(), DeathReason::SignalOutOfRange);
            return Err(Error::msg("out of signal range"));
        }
        caller.data_mut().result.actions.push(PendingAction::SignalTo {
            to_id,
            byte: byte as u8,
            from_x: sx,
            from_y: sy,
        });
        Ok(0_i32)
    })?;

    linker.func_wrap("terrarium", "recv", |mut caller: Caller<'_, HostState>, ptr: i32| {
        let sig = caller.data_mut().creature().inbox.first().cloned();
        if let Some(sig) = sig {
            write_recv_struct(&mut caller, ptr, &sig)?;
            caller.data_mut().creature().inbox.remove(0);
            Ok(1)
        } else {
            write_i32(&mut caller, ptr, 0)?;
            Ok(0)
        }
    })?;

    Ok(())
}

fn creature_rng_seed(id: &str, tick: u64) -> u64 {
    let mut h = tick.wrapping_mul(0x9E3779B97F4A7C15);
    for b in id.as_bytes() {
        h = h.wrapping_mul(31).wrapping_add(u64::from(*b));
    }
    h
}

fn mix64(x: u64) -> u64 {
    let mut x = x;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x
}

fn next_random(host: &mut HostState) -> u8 {
    host.rng_calls = host.rng_calls.wrapping_add(1);
    let x = mix64(host.rng_seed.wrapping_add(host.rng_calls));
    (x >> 56) as u8
}

fn in_sig_range(sx: i32, sy: i32, tx: i32, ty: i32, config: &SimConfig) -> bool {
    crate::hex::in_range(tx - sx, ty - sy, config.r_sig)
}

fn uuid_from_bytes(bytes: &[u8]) -> Option<String> {
    let arr: [u8; 16] = bytes.try_into().ok()?;
    let u = Uuid::from_bytes(arr);
    if u.is_nil() {
        return None;
    }
    Some(u.to_string())
}

fn read_bytes(caller: &mut Caller<'_, HostState>, ptr: i32, len: usize) -> Result<Vec<u8>, Error> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| Error::msg("no memory"))?;
    let data = memory.data(&*caller);
    let ptr = ptr as usize;
    let end = ptr.checked_add(len).ok_or_else(|| Error::msg("bad ptr"))?;
    if end > data.len() {
        return Err(Error::msg("oob"));
    }
    Ok(data[ptr..end].to_vec())
}

fn write_i32(caller: &mut Caller<'_, HostState>, ptr: i32, value: i32) -> Result<(), Error> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| Error::msg("no memory"))?;
    let data = memory.data_mut(&mut *caller);
    let ptr = ptr as usize;
    if ptr + 4 > data.len() {
        return Err(Error::msg("oob"));
    }
    data[ptr..ptr + 4].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_sense_struct(
    caller: &mut Caller<'_, HostState>,
    ptr: i32,
    kind: i32,
    energy: i64,
    health: i32,
    max_health: i32,
) -> Result<(), Error> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| Error::msg("no memory"))?;
    let data = memory.data_mut(&mut *caller);
    let ptr = ptr as usize;
    let end = ptr + SENSE_STRUCT_SIZE as usize;
    if end > data.len() {
        return Err(Error::msg("oob"));
    }
    data[ptr..ptr + 4].copy_from_slice(&kind.to_le_bytes());
    data[ptr + 8..ptr + 16].copy_from_slice(&energy.to_le_bytes());
    data[ptr + 16..ptr + 20].copy_from_slice(&health.to_le_bytes());
    data[ptr + 20..ptr + 24].copy_from_slice(&max_health.to_le_bytes());
    Ok(())
}

fn write_recv_struct(caller: &mut Caller<'_, HostState>, ptr: i32, sig: &Signal) -> Result<(), Error> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| Error::msg("no memory"))?;
    let data = memory.data_mut(&mut *caller);
    let ptr = ptr as usize;
    let end = ptr + RECV_STRUCT_SIZE as usize;
    if end > data.len() {
        return Err(Error::msg("oob"));
    }
    let mut buf = vec![0u8; RECV_STRUCT_SIZE as usize];
    buf[0..4].copy_from_slice(&1i32.to_le_bytes());
    buf[4..8].copy_from_slice(&sig.from_x.to_le_bytes());
    buf[8..12].copy_from_slice(&sig.from_y.to_le_bytes());
    buf[12..16].copy_from_slice(&(sig.byte as i32).to_le_bytes());
    buf[16..20].copy_from_slice(&i32::from(sig.broadcast).to_le_bytes());
    if let Ok(u) = Uuid::parse_str(&sig.from_id) {
        buf[20..36].copy_from_slice(u.as_bytes());
    }
    data[ptr..end].copy_from_slice(&buf);
    Ok(())
}

fn gas_budget(creature: &Creature, config: &SimConfig) -> u64 {
    if config.energy_per_opcode <= 0 {
        return config.opcodes_per_tick;
    }
    let affordable = (creature.energy / config.energy_per_opcode).max(0) as u64;
    config.opcodes_per_tick.min(affordable)
}

fn charge_opcode_gas(store: &mut Store<HostState>, budget: u64, config: &SimConfig) {
    let remaining = store.get_fuel().unwrap_or(0);
    let used = budget.saturating_sub(remaining);
    let cost = (used as i64).saturating_mul(config.energy_per_opcode);
    let floor = store.data().config().corpse_energy;
    let energy = {
        let c = store.data_mut().creature();
        c.energy = c.energy.saturating_sub(cost);
        c.energy
    };
    store.data_mut().ledger().record_destroy(cost);
    if energy <= floor {
        mark_dead(store.data_mut().creature(), DeathReason::EnergyFloor);
    }
}

fn host_trap_reason(msg: &str) -> DeathReason {
    match msg {
        "out of energy" => DeathReason::OutOfEnergy,
        "energy floor" => DeathReason::EnergyFloor,
        "out of vision" => DeathReason::OutOfVision,
        "bad direction" => DeathReason::BadDirection,
        "spawn energy" => DeathReason::SpawnEnergyTooLow,
        "unknown target" => DeathReason::SignalUnknownTarget,
        "out of signal range" => DeathReason::SignalOutOfRange,
        _ => DeathReason::WasmTrap,
    }
}

fn trap_death_reason(err: &Error) -> DeathReason {
    let msg = err.to_string();
    if msg.contains("all fuel consumed") || msg.contains("out of fuel") {
        return DeathReason::OutOfGas;
    }
    for needle in [
        "out of energy",
        "energy floor",
        "out of vision",
        "bad direction",
        "spawn energy",
        "unknown target",
        "out of signal range",
    ] {
        if msg.contains(needle) {
            return host_trap_reason(needle);
        }
    }
    DeathReason::WasmTrap
}

pub fn run_creature_tick(
    engine: &Engine,
    module: &Module,
    creature: &mut Creature,
    snapshot: &Snapshot,
    tiles: &WorldTiles,
    config: &SimConfig,
    ledger: &mut EnergyLedger,
    tick: u64,
) -> ThinkResult {
    if creature.wasm.is_empty() {
        mark_dead(creature, DeathReason::EmptyProgram);
        return ThinkResult::default();
    }

    let mut linker: Linker<HostState> = Linker::new(engine);
    if link_host(&mut linker).is_err() {
        mark_dead(creature, DeathReason::InvalidProgram);
        return ThinkResult::default();
    }

    let host = HostState {
        creature: creature as *mut Creature,
        snapshot: snapshot as *const Snapshot,
        tiles: tiles as *const WorldTiles,
        config: config as *const SimConfig,
        ledger: ledger as *mut EnergyLedger,
        tick,
        rng_seed: creature_rng_seed(&creature.id, tick),
        rng_calls: 0,
        result: ThinkResult::default(),
    };

    let mut store = Store::new(engine, host);
    let budget = gas_budget(creature, config);
    if budget == 0 {
        mark_dead(
            creature,
            if creature.energy <= config.corpse_energy {
                DeathReason::EnergyFloor
            } else {
                DeathReason::OutOfGas
            },
        );
        return ThinkResult::default();
    }
    if store.set_fuel(budget).is_err() {
        mark_dead(creature, DeathReason::InvalidProgram);
        return ThinkResult::default();
    }

    let instance = match linker.instantiate(&mut store, module) {
        Ok(i) => i,
        Err(_) => {
            mark_dead(store.data_mut().creature(), DeathReason::InvalidProgram);
            return ThinkResult::default();
        }
    };

    let tick = match instance.get_typed_func::<(), ()>(&mut store, "tick") {
        Ok(f) => f,
        Err(_) => {
            mark_dead(store.data_mut().creature(), DeathReason::InvalidProgram);
            return ThinkResult::default();
        }
    };

    let trap_err = tick.call(&mut store, ()).err();
    charge_opcode_gas(&mut store, budget, config);
    if let Some(err) = trap_err {
        let fuel_left = store.get_fuel().unwrap_or(0);
        let c = store.data_mut().creature();
        if c.death_reason.is_none() {
            let reason = if fuel_left == 0 {
                DeathReason::OutOfGas
            } else {
                trap_death_reason(&err)
            };
            mark_dead(c, reason);
        } else {
            c.alive = false;
        }
    } else if !store.data().creature_ref().alive {
        mark_dead(store.data_mut().creature(), DeathReason::EnergyFloor);
    }

    store.into_data().result
}

pub fn wasm_engine() -> &'static Engine {
    use std::sync::OnceLock;
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut cfg = Config::new();
        cfg.consume_fuel(true);
        Engine::new(&cfg).expect("wasm engine")
    })
}

pub fn load_module(engine: &Engine, wasm: &[u8]) -> Option<Module> {
    Module::new(engine, wasm).ok()
}

pub fn cached_module(engine: &Engine, wasm: &[u8]) -> Option<Module> {
    static CACHE: OnceLock<Mutex<HashMap<Vec<u8>, Module>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().unwrap();
    if let Some(module) = cache.get(wasm) {
        return Some(module.clone());
    }
    let module = load_module(engine, wasm)?;
    cache.insert(wasm.to_vec(), module.clone());
    Some(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_byte_seeded_and_in_range() {
        let mut host = HostState {
            creature: std::ptr::null_mut(),
            snapshot: std::ptr::null(),
            tiles: std::ptr::null(),
            config: std::ptr::null(),
            ledger: std::ptr::null_mut(),
            tick: 99,
            rng_seed: creature_rng_seed("creature-a", 99),
            rng_calls: 0,
            result: ThinkResult::default(),
        };
        let first = next_random(&mut host);
        let second = next_random(&mut host);

        let mut again = HostState {
            creature: std::ptr::null_mut(),
            snapshot: std::ptr::null(),
            tiles: std::ptr::null(),
            config: std::ptr::null(),
            ledger: std::ptr::null_mut(),
            tick: 99,
            rng_seed: creature_rng_seed("creature-a", 99),
            rng_calls: 0,
            result: ThinkResult::default(),
        };
        assert_eq!(first, next_random(&mut again));
        assert_ne!(first, second);
    }
}
