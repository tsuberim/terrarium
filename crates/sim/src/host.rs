//! Host syscalls for creature WASM modules (ABI v2).

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::{Mutex, OnceLock};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use wasmtime::{Caller, Config, Engine, Error, Linker, Memory, Module, Store, TypedFunc};

use crate::abi::{self, action, Payload};
use crate::energy_ledger::EnergyLedger;
use crate::events::DeathReason;
use crate::guest_mem::{self, refresh_creature_memory};
use crate::hex;
use crate::sim_config::SimConfig;
use crate::vm::{mark_dead, Creature, Snapshot};
use crate::world_tile::WorldTiles;

struct YieldOnce {
    done: bool,
}

impl YieldOnce {
    fn new() -> Self {
        Self { done: false }
    }
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done {
            Poll::Ready(())
        } else {
            self.done = true;
            Poll::Pending
        }
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => {}
        }
    }
}

fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(std::ptr::null(), &VTABLE),
        |_| {},
        |_| {},
        |_| {},
    );
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SliceEnd {
    None,
    Action,
}

enum PollOutcome {
    Suspended,
    Halted,
    Trap(Error),
}

const FUEL_YIELD_CHUNK: u64 = 512;

fn fuel_yield_interval(budget: u64) -> u64 {
    if budget <= FUEL_YIELD_CHUNK {
        1
    } else {
        FUEL_YIELD_CHUNK
    }
}

fn prepare_think_fuel(store: &mut Store<HostState>, budget: u64) -> Result<(), Error> {
    store.fuel_async_yield_interval(Some(fuel_yield_interval(budget)))?;
    store.set_fuel(budget.saturating_add(1))?;
    Ok(())
}

fn poll_main_slice(
    run: Pin<&mut (dyn Future<Output = std::result::Result<(), Error>> + Send)>,
) -> PollOutcome {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    match run.poll(&mut cx) {
        Poll::Ready(Ok(())) => PollOutcome::Halted,
        Poll::Ready(Err(err)) => PollOutcome::Trap(err),
        Poll::Pending => PollOutcome::Suspended,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    Move {
        rel: u8,
    },
    Rotate {
        delta: i32,
    },
    Dig {
        rel: u8,
    },
    Place {
        rel: u8,
    },
    Eat {
        rel: u8,
    },
    Hit {
        rel: u8,
    },
    Spawn {
        rel: u8,
        energy: i64,
        owner_id: u64,
        child_init: Payload,
    },
    Signal {
        target: u64,
        payload: Payload,
    },
    Broadcast {
        payload: Payload,
    },
}

#[derive(Debug, Default, Clone)]
pub struct ThinkResult {
    pub action: Option<PendingAction>,
    pub suicide: bool,
}

impl ThinkResult {
    pub fn tick_busy(&self) -> bool {
        self.suicide || self.action.is_some()
    }

    pub fn take_action(&mut self, action: PendingAction) -> bool {
        if self.tick_busy() {
            return false;
        }
        self.action = Some(action);
        true
    }

    pub fn mark_halted(&mut self) {
        self.suicide = true;
    }

    pub fn effective_facing(&self, facing: u8) -> u8 {
        match &self.action {
            Some(PendingAction::Rotate { delta }) => (facing as i32 + delta).rem_euclid(6) as u8,
            _ => facing,
        }
    }
}

pub struct HostState {
    pub(crate) creature: *mut Creature,
    pub(crate) snapshot: *const Snapshot,
    pub(crate) tiles: *const WorldTiles,
    pub(crate) config: *const SimConfig,
    pub(crate) ledger: *mut EnergyLedger,
    pub tick: u64,
    pub rng_seed: u64,
    pub rng_calls: u64,
    pub memory: Option<Memory>,
    pub result: ThinkResult,
    slice_end: SliceEnd,
}

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

    #[allow(dead_code)]
    fn tiles(&self) -> &WorldTiles {
        unsafe { &*self.tiles }
    }

    fn config(&self) -> &SimConfig {
        unsafe { &*self.config }
    }

    fn ledger(&mut self) -> &mut EnergyLedger {
        unsafe { &mut *self.ledger }
    }

    fn take_world_action(&mut self, action: PendingAction) -> bool {
        if !self.result.take_action(action) {
            return false;
        }
        self.slice_end = SliceEnd::Action;
        true
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
}

pub fn payload_to_action(payload: &Payload) -> Result<PendingAction, Error> {
    let needs_rel = matches!(
        payload.tag(),
        action::MOVE | action::DIG | action::PLACE | action::EAT | action::HIT | action::SPAWN
    );
    if needs_rel && !abi::rel::valid(payload.rel()) {
        return Err(Error::msg("bad direction"));
    }
    match payload.tag() {
        action::MOVE => Ok(PendingAction::Move { rel: payload.rel() }),
        action::ROTATE => Ok(PendingAction::Rotate {
            delta: payload.a() as i32,
        }),
        action::DIG => Ok(PendingAction::Dig { rel: payload.rel() }),
        action::PLACE => Ok(PendingAction::Place { rel: payload.rel() }),
        action::EAT => Ok(PendingAction::Eat { rel: payload.rel() }),
        action::HIT => Ok(PendingAction::Hit { rel: payload.rel() }),
        action::SPAWN => {
            let energy = payload.a() as i64;
            Ok(PendingAction::Spawn {
                rel: payload.rel(),
                energy,
                owner_id: payload.spawn_owner_id(),
                child_init: payload.child_init_from_spawn(),
            })
        }
        action::SIGNAL => Ok(PendingAction::Signal {
            target: payload.a(),
            payload: *payload,
        }),
        action::BROADCAST => Ok(PendingAction::Broadcast { payload: *payload }),
        action::NONE => Err(Error::msg("empty action")),
        _ => Err(Error::msg("unknown action tag")),
    }
}

fn action_extra(host: &HostState, tag: u32) -> i64 {
    let config = host.config();
    match tag {
        action::MOVE => config.move_extra,
        action::DIG => config.dig_extra,
        action::PLACE => config.place_extra,
        action::HIT => config.hit_extra,
        action::ROTATE => config.rotate_extra,
        _ => 0,
    }
}

pub fn link_host(linker: &mut Linker<HostState>) -> Result<(), Error> {
    linker.func_wrap("terrarium", "rand", |mut caller: Caller<'_, HostState>| {
        Ok(next_random(caller.data_mut()))
    })?;

    linker.func_wrap("terrarium", "recv", |mut caller: Caller<'_, HostState>| {
        let sig = caller.data_mut().creature().inbox.first().cloned();
        if let Some(sig) = sig {
            if let Some(memory) = caller.data().memory {
                let mut buf = [0u8; abi::RECV_MSG_SIZE];
                buf[..8].copy_from_slice(&sig.from_id.to_le_bytes());
                buf[8..].copy_from_slice(&sig.payload.bytes);
                let base = abi::ABI_RECV as usize;
                let end = base + buf.len();
                let data = memory.data_mut(&mut caller);
                if end <= data.len() {
                    data[base..end].copy_from_slice(&buf);
                }
            }
            caller.data_mut().creature().inbox.remove(0);
            Ok(1_i32)
        } else {
            Ok(0_i32)
        }
    })?;

    linker.func_wrap_async(
        "terrarium",
        "act",
        |mut caller: Caller<'_, HostState>, (): ()| {
            Box::new(async move { apply_act(&mut caller).await })
        },
    )?;

    Ok(())
}

fn creature_rng_seed(id: u64, tick: u64) -> u64 {
    tick.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(id)
}

fn mix64(x: u64) -> u64 {
    let mut x = x;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x
}

fn next_random(host: &mut HostState) -> u64 {
    host.rng_calls = host.rng_calls.wrapping_add(1);
    mix64(host.rng_seed.wrapping_add(host.rng_calls))
}

fn read_action_payload(caller: &Caller<'_, HostState>) -> Option<Payload> {
    let memory = caller.data().memory?;
    let base = abi::ABI_ACTION as usize;
    let end = base + abi::PAYLOAD_SIZE;
    let data = memory.data(caller);
    if end > data.len() {
        return None;
    }
    let mut bytes = [0u8; abi::PAYLOAD_SIZE];
    bytes.copy_from_slice(&data[base..end]);
    Some(Payload { bytes })
}

async fn apply_act(caller: &mut Caller<'_, HostState>) -> Result<i32, Error> {
    if caller.data().result.tick_busy() {
        return Ok(-1);
    }
    let Some(payload) = read_action_payload(caller) else {
        return Ok(-1);
    };
    if payload.tag() == 0 {
        return Ok(-1);
    }
    let pending = match payload_to_action(&payload) {
        Ok(p) => p,
        Err(_) => return Ok(-1),
    };
    let extra = action_extra(caller.data(), payload.tag());
    if extra > 0 {
        caller.data_mut().pay_action(extra)?;
    }
    if payload.tag() == action::SPAWN {
        let energy = payload.a() as i64;
        let floor = caller.data().config().corpse_energy;
        if energy <= floor {
            mark_dead(caller.data_mut().creature(), DeathReason::SpawnEnergyTooLow);
            return Err(Error::msg("spawn energy"));
        }
    }
    if payload.tag() == action::SIGNAL {
        let target = payload.a();
        let snapshot = caller.data().snapshot();
        let Some(&(tx, ty)) = snapshot.positions.get(&target) else {
            mark_dead(
                caller.data_mut().creature(),
                DeathReason::SignalUnknownTarget,
            );
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
    }
    caller.data_mut().take_world_action(pending);
    YieldOnce::new().await;
    Ok(0)
}

fn in_sig_range(sx: i32, sy: i32, tx: i32, ty: i32, config: &SimConfig) -> bool {
    hex::in_range(tx - sx, ty - sy, config.r_sig)
}

fn gas_budget(creature: &Creature, config: &SimConfig) -> u64 {
    if config.energy_per_opcode <= 0 {
        return config.opcodes_per_tick;
    }
    let affordable = (creature.energy / config.energy_per_opcode).max(0) as u64;
    config.opcodes_per_tick.min(affordable)
}

fn charge_opcode_gas(store: &mut Store<HostState>, opcodes_used: u64, config: &SimConfig) {
    let cost = (opcodes_used as i64).saturating_mul(config.energy_per_opcode);
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

fn trap_death_reason(err: &Error) -> DeathReason {
    if err.downcast_ref::<wasmtime::Trap>() == Some(&wasmtime::Trap::OutOfFuel) {
        return DeathReason::OutOfGas;
    }
    let msg = err.to_string();
    if msg.contains("all fuel consumed") || msg.contains("out of fuel") {
        return DeathReason::OutOfGas;
    }
    for needle in [
        "out of energy",
        "energy floor",
        "bad direction",
        "spawn energy",
        "unknown target",
        "out of signal range",
    ] {
        if msg.contains(needle) {
            return match needle {
                "out of energy" => DeathReason::OutOfEnergy,
                "energy floor" => DeathReason::EnergyFloor,
                "bad direction" => DeathReason::BadDirection,
                "spawn energy" => DeathReason::SpawnEnergyTooLow,
                "unknown target" => DeathReason::SignalUnknownTarget,
                "out of signal range" => DeathReason::SignalOutOfRange,
                _ => DeathReason::WasmTrap,
            };
        }
    }
    DeathReason::WasmTrap
}

type MainFiber = Pin<Box<dyn Future<Output = std::result::Result<(), Error>> + Send>>;

struct CachedVm {
    store: Store<HostState>,
    main_fn: TypedFunc<(), ()>,
    main_run: Option<MainFiber>,
}

fn stash_main_run(
    run: Pin<Box<dyn Future<Output = std::result::Result<(), Error>> + Send + '_>>,
) -> MainFiber {
    unsafe { std::mem::transmute(run) }
}

thread_local! {
    static VM_CACHE_EPOCH: Cell<u64> = const { Cell::new(0) };
}

static VM_CACHE: OnceLock<Mutex<HashMap<String, CachedVm>>> = OnceLock::new();

fn vm_cache_key(creature: &Creature, wasm: &[u8]) -> String {
    let epoch = VM_CACHE_EPOCH.with(|e| e.get());
    let mut hasher = DefaultHasher::new();
    wasm.hash(&mut hasher);
    format!(
        "{epoch}:{}:{:p}:{:x}",
        creature.id,
        creature,
        hasher.finish()
    )
}

fn host_state_for_tick(
    creature: &mut Creature,
    snapshot: &Snapshot,
    tiles: &WorldTiles,
    config: &SimConfig,
    ledger: &mut EnergyLedger,
    tick: u64,
    memory: Option<Memory>,
) -> HostState {
    HostState {
        creature: creature as *mut Creature,
        snapshot: snapshot as *const Snapshot,
        tiles: tiles as *const WorldTiles,
        config: config as *const SimConfig,
        ledger: ledger as *mut EnergyLedger,
        tick,
        rng_seed: creature_rng_seed(creature.id, tick),
        rng_calls: 0,
        memory,
        result: ThinkResult::default(),
        slice_end: SliceEnd::None,
    }
}

pub fn clear_vm_cache() {
    VM_CACHE_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
}

pub fn clear_all_vm_cache() {
    VM_CACHE_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
    if let Some(cache) = VM_CACHE.get() {
        let mut cache = cache.lock().unwrap();
        for (_, mut vm) in cache.drain() {
            vm.main_run.take();
        }
    }
}

#[allow(clippy::too_many_arguments)]
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

    let key = vm_cache_key(creature, &creature.wasm);
    let cache = VM_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().unwrap();

    if !cache.contains_key(&key) {
        let mut linker = Linker::new(engine);
        if link_host(&mut linker).is_err() {
            mark_dead(creature, DeathReason::InvalidProgram);
            return ThinkResult::default();
        }
        let host = host_state_for_tick(creature, snapshot, tiles, config, ledger, tick, None);
        let mut store = Store::new(engine, host);
        let Ok(instance) = block_on(linker.instantiate_async(&mut store, module)) else {
            mark_dead(creature, DeathReason::InvalidProgram);
            return ThinkResult::default();
        };
        let memory = instance.get_memory(&mut store, "memory");
        let main_fn = match instance.get_typed_func::<(), ()>(&mut store, "main") {
            Ok(f) => f,
            Err(_) => {
                mark_dead(creature, DeathReason::InvalidProgram);
                return ThinkResult::default();
            }
        };
        store.data_mut().memory = memory;
        guest_mem::write_creature_init(&mut store, &creature.init);
        cache.insert(
            key.clone(),
            CachedVm {
                store,
                main_fn,
                main_run: None,
            },
        );
    }

    let cached = cache.get_mut(&key).unwrap();
    let mut run = cached.main_run.take();
    let store_ptr = std::ptr::addr_of_mut!(cached.store);

    unsafe {
        *(*store_ptr).data_mut() = host_state_for_tick(
            creature,
            snapshot,
            tiles,
            config,
            ledger,
            tick,
            cached.store.data().memory,
        );
    }

    let budget = gas_budget(creature, config);
    if budget == 0 {
        if let Some(run) = run {
            cached.main_run = Some(stash_main_run(run));
        }
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

    if let Err(err) = prepare_think_fuel(&mut cached.store, budget) {
        if let Some(run) = run {
            cached.main_run = Some(stash_main_run(run));
        }
        mark_dead(creature, DeathReason::InvalidProgram);
        let _ = err;
        return ThinkResult::default();
    }

    refresh_creature_memory(&mut cached.store);

    unsafe {
        if run.is_none() {
            run = Some(Box::pin(cached.main_fn.call_async(&mut *store_ptr, ())));
        }
    }
    let mut run = run.expect("main fiber started");

    let chunk = fuel_yield_interval(budget);
    let mut opcodes_used = 0_u64;
    let mut gas_trap_suspend = false;
    let poll = loop {
        if opcodes_used >= budget {
            break PollOutcome::Suspended;
        }
        let poll = poll_main_slice(run.as_mut());
        let has_action = unsafe { (*store_ptr).data().slice_end == SliceEnd::Action };
        match poll {
            PollOutcome::Suspended if has_action => break poll,
            PollOutcome::Suspended => {
                opcodes_used = opcodes_used.saturating_add(chunk).min(budget);
            }
            PollOutcome::Trap(err)
                if err.downcast_ref::<wasmtime::Trap>() == Some(&wasmtime::Trap::OutOfFuel) =>
            {
                gas_trap_suspend = true;
                break PollOutcome::Suspended;
            }
            _ => break poll,
        }
    };
    let suspended = matches!(poll, PollOutcome::Suspended);
    match &poll {
        PollOutcome::Halted => unsafe {
            (*store_ptr).data_mut().result.mark_halted();
        },
        PollOutcome::Trap(err) => unsafe {
            let c = (*store_ptr).data_mut().creature();
            if c.death_reason.is_some() {
                c.alive = false;
            } else {
                mark_dead(c, trap_death_reason(err));
            }
        },
        PollOutcome::Suspended => {}
    }

    let stash_fiber = suspended && !gas_trap_suspend;
    if stash_fiber {
        cached.main_run = Some(stash_main_run(run));
    } else {
        drop(run);
    }

    charge_opcode_gas(&mut cached.store, opcodes_used, config);

    let result = cached.store.data().result.clone();
    if !cached.store.data().creature_ref().alive
        && cached.store.data().creature_ref().death_reason.is_none()
    {
        mark_dead(cached.store.data_mut().creature(), DeathReason::EnergyFloor);
    }

    result
}

pub fn wasm_engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut cfg = Config::new();
        cfg.async_support(true);
        cfg.consume_fuel(true);
        cfg.async_stack_size(4 << 20);
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
    fn payload_move_roundtrip() {
        let mut p = Payload::default();
        p.set_tag(action::MOVE);
        p.set_rel(abi::rel::FWD);
        let a = payload_to_action(&p).unwrap();
        assert!(matches!(a, PendingAction::Move { rel: 0 }));
    }
}
