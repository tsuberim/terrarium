//! Host syscalls for creature WASM modules.

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::{Mutex, OnceLock};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use uuid::Uuid;
use wasmtime::{Caller, Config, Engine, Error, Linker, Module, Store, TypedFunc};

use crate::abi::{RECV_STRUCT_SIZE, SENSE_STRUCT_SIZE};
use crate::energy_ledger::EnergyLedger;
use crate::events::DeathReason;
use crate::sim_config::SimConfig;
use crate::vm::{mark_dead, Creature, Signal, Snapshot};
use crate::world_tile::{sense_kind, WorldTile, WorldTiles};

/// One poll pending, one poll ready — yields the wasmtime fiber for this sim slice.
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

impl std::fmt::Debug for PollOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Suspended => write!(f, "Suspended"),
            Self::Halted => write!(f, "Halted"),
            Self::Trap(err) => write!(f, "Trap({err})"),
        }
    }
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
    Move,
    Dig {
        dir: u8,
    },
    Place {
        dir: u8,
    },
    Eat {
        dir: u8,
    },
    Hit {
        dir: u8,
    },
    Rotate {
        delta: i32,
    },
    Spawn {
        dir: u8,
        energy: i64,
    },
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

#[derive(Debug, Default, Clone)]
pub struct ThinkResult {
    pub action: Option<PendingAction>,
    pub suicide: bool,
}

impl ThinkResult {
    pub fn tick_busy(&self) -> bool {
        self.suicide || self.action.is_some()
    }

    /// Reserve the sole action slot for this tick. Returns false if already taken.
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
    creature: *mut Creature,
    snapshot: *const Snapshot,
    tiles: *const WorldTiles,
    config: *const SimConfig,
    ledger: *mut EnergyLedger,
    tick: u64,
    rng_seed: u64,
    rng_calls: u64,
    pub result: ThinkResult,
    slice_end: SliceEnd,
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

    fn can_sense(&self, dq: i32, dr: i32) -> bool {
        let config = self.config();
        if !config.in_hex_range(dq, dr, config.r_vis) {
            return false;
        }
        // Adjacent cells are always sensed (flee, eat, hit) regardless of facing.
        if crate::hex::distance(0, 0, dq, dr) <= 1 {
            return true;
        }
        let facing = self.creature_ref().facing;
        crate::hex::in_fov(facing, dq, dr, config.vis_half_arc)
    }

    fn require_forward(d: i32) -> Result<(), Error> {
        if d != 0 {
            return Err(Error::msg(
                "only forward (0) is allowed; use rotate to face the target first",
            ));
        }
        Ok(())
    }

    fn effective_facing(result: &ThinkResult, facing: u8) -> u8 {
        result.effective_facing(facing)
    }
}

pub fn link_host(linker: &mut Linker<HostState>) -> Result<(), wasmtime::Error> {
    linker.func_wrap(
        "terrarium",
        "sleep",
        |_caller: Caller<'_, HostState>| Ok(()),
    )?;

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

    linker.func_wrap("terrarium", "facing", |caller: Caller<'_, HostState>| {
        let facing = caller.data().creature_ref().facing;
        Ok(i32::from(HostState::effective_facing(
            &caller.data().result,
            facing,
        )))
    })?;

    linker.func_wrap(
        "terrarium",
        "sense",
        |mut caller: Caller<'_, HostState>, dq: i32, dr: i32, ptr: i32| {
            if !caller.data().can_sense(dq, dr) {
                return Ok(0_i32);
            }
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
            let mut orientation = -1i32;
            if let Some(id) = snapshot.id_at.get(&(x, y)) {
                energy = snapshot.energy.get(id).copied().unwrap_or(0);
                health = snapshot.health.get(id).copied().unwrap_or(0);
                max_health = snapshot.max_health.get(id).copied().unwrap_or(0);
                orientation = snapshot.facing.get(id).copied().unwrap_or(0) as i32;
            } else if let Some(WorldTile::Corpse { energy: e, .. }) = tiles.get(&(x, y)) {
                energy = *e;
            } else if let Some(WorldTile::Food { energy: e }) = tiles.get(&(x, y)) {
                energy = *e;
            }
            write_sense_struct(
                &mut caller,
                ptr,
                kind,
                energy,
                health,
                max_health,
                orientation,
            )?;
            Ok(1)
        },
    )?;

    linker.func_wrap(
        "terrarium",
        "random_byte",
        |mut caller: Caller<'_, HostState>| Ok(i32::from(next_random(caller.data_mut()))),
    )?;

    linker.func_wrap_async(
        "terrarium",
        "move",
        |mut caller: Caller<'_, HostState>, (d,): (i32,)| {
            Box::new(async move {
                HostState::require_forward(d)?;
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let extra = caller.data().config().move_extra;
                caller.data_mut().pay_action(extra)?;
                caller.data_mut().take_world_action(PendingAction::Move);
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "dig",
        |mut caller: Caller<'_, HostState>, (d,): (i32,)| {
            Box::new(async move {
                HostState::require_forward(d)?;
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let dir = caller.data().creature_ref().facing;
                let extra = caller.data().config().dig_extra;
                caller.data_mut().pay_action(extra)?;
                caller
                    .data_mut()
                    .take_world_action(PendingAction::Dig { dir });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "place",
        |mut caller: Caller<'_, HostState>, (d,): (i32,)| {
            Box::new(async move {
                HostState::require_forward(d)?;
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let dir = caller.data().creature_ref().facing;
                let extra = caller.data().config().place_extra;
                caller.data_mut().pay_action(extra)?;
                caller
                    .data_mut()
                    .take_world_action(PendingAction::Place { dir });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "eat",
        |mut caller: Caller<'_, HostState>, (d,): (i32,)| {
            Box::new(async move {
                HostState::require_forward(d)?;
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let dir = caller.data().creature_ref().facing;
                caller
                    .data_mut()
                    .take_world_action(PendingAction::Eat { dir });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "hit",
        |mut caller: Caller<'_, HostState>, (d,): (i32,)| {
            Box::new(async move {
                HostState::require_forward(d)?;
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let dir = caller.data().creature_ref().facing;
                let extra = caller.data().config().hit_extra;
                caller.data_mut().pay_action(extra)?;
                caller
                    .data_mut()
                    .take_world_action(PendingAction::Hit { dir });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "rotate",
        |mut caller: Caller<'_, HostState>, (delta,): (i32,)| {
            Box::new(async move {
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let extra = caller.data().config().rotate_extra;
                caller.data_mut().pay_action(extra)?;
                caller
                    .data_mut()
                    .take_world_action(PendingAction::Rotate { delta });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "spawn",
        |mut caller: Caller<'_, HostState>, (d, energy): (i32, i32)| {
            Box::new(async move {
                HostState::require_forward(d)?;
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let dir = caller.data().creature_ref().facing;
                let energy = energy as i64;
                let floor = caller.data().config().corpse_energy;
                if energy <= floor {
                    mark_dead(caller.data_mut().creature(), DeathReason::SpawnEnergyTooLow);
                    return Err(Error::msg("spawn energy"));
                }
                caller
                    .data_mut()
                    .take_world_action(PendingAction::Spawn { dir, energy });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "signal_broadcast",
        |mut caller: Caller<'_, HostState>, (byte,): (i32,)| {
            Box::new(async move {
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let (x, y) = {
                    let c = caller.data().creature_ref();
                    (c.x, c.y)
                };
                caller
                    .data_mut()
                    .take_world_action(PendingAction::SignalBroadcast {
                        byte: byte as u8,
                        from_x: x,
                        from_y: y,
                    });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap_async(
        "terrarium",
        "signal_to",
        |mut caller: Caller<'_, HostState>, (ptr, byte): (i32, i32)| {
            Box::new(async move {
                if caller.data().result.tick_busy() {
                    return Ok(0_i32);
                }
                let id_bytes = read_bytes(&mut caller, ptr, 16)?;
                let to_id = uuid_from_bytes(&id_bytes).ok_or_else(|| Error::msg("bad uuid"))?;
                let snapshot = caller.data().snapshot();
                let Some(&(tx, ty)) = snapshot.positions.get(&to_id) else {
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
                caller
                    .data_mut()
                    .take_world_action(PendingAction::SignalTo {
                        to_id,
                        byte: byte as u8,
                        from_x: sx,
                        from_y: sy,
                    });
                YieldOnce::new().await;
                Ok(0_i32)
            })
        },
    )?;

    linker.func_wrap(
        "terrarium",
        "recv",
        |mut caller: Caller<'_, HostState>, ptr: i32| {
            let sig = caller.data_mut().creature().inbox.first().cloned();
            if let Some(sig) = sig {
                write_recv_struct(&mut caller, ptr, &sig)?;
                caller.data_mut().creature().inbox.remove(0);
                Ok(1)
            } else {
                write_i32(&mut caller, ptr, 0)?;
                Ok(0)
            }
        },
    )?;

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
    orientation: i32,
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
    data[ptr + 4..ptr + 8].copy_from_slice(&orientation.to_le_bytes());
    data[ptr + 8..ptr + 16].copy_from_slice(&energy.to_le_bytes());
    data[ptr + 16..ptr + 20].copy_from_slice(&health.to_le_bytes());
    data[ptr + 20..ptr + 24].copy_from_slice(&max_health.to_le_bytes());
    Ok(())
}

fn write_recv_struct(
    caller: &mut Caller<'_, HostState>,
    ptr: i32,
    sig: &Signal,
) -> Result<(), Error> {
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

fn host_trap_reason(msg: &str) -> DeathReason {
    match msg {
        "out of energy" => DeathReason::OutOfEnergy,
        "energy floor" => DeathReason::EnergyFloor,
        "out of vision" => DeathReason::OutOfVision,
        "bad direction" => DeathReason::BadDirection,
        "move: only forward" => DeathReason::BadDirection,
        "spawn energy" => DeathReason::SpawnEnergyTooLow,
        "unknown target" => DeathReason::SignalUnknownTarget,
        "out of signal range" => DeathReason::SignalOutOfRange,
        _ => DeathReason::WasmTrap,
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
        "out of vision",
        "bad direction",
        "move: only forward",
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

type MainFiber = Pin<Box<dyn Future<Output = std::result::Result<(), Error>> + Send>>;

struct CachedVm {
    store: Store<HostState>,
    main_fn: TypedFunc<(), ()>,
    /// `call_async` future borrows `store`; take before any other `store` use.
    main_run: Option<MainFiber>,
}

/// Stash a fiber that borrows our `Store` until the next tick resumes it.
///
/// SAFETY: callers must not touch `CachedVm::store` while `main_run` is `Some`.
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
) -> HostState {
    HostState {
        creature: creature as *mut Creature,
        snapshot: snapshot as *const Snapshot,
        tiles: tiles as *const WorldTiles,
        config: config as *const SimConfig,
        ledger: ledger as *mut EnergyLedger,
        tick,
        rng_seed: creature_rng_seed(&creature.id, tick),
        rng_calls: 0,
        result: ThinkResult::default(),
        slice_end: SliceEnd::None,
    }
}

/// Invalidate cached WASM stores for this thread (tests bump an epoch; no global drain).
pub fn clear_vm_cache() {
    VM_CACHE_EPOCH.with(|e| e.set(e.get().wrapping_add(1)));
}

/// Drop all cached WASM stores (sandbox reset).
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
        let host = host_state_for_tick(creature, snapshot, tiles, config, ledger, tick);
        let mut store = Store::new(engine, host);
        let Ok(instance) = block_on(linker.instantiate_async(&mut store, module)) else {
            mark_dead(creature, DeathReason::InvalidProgram);
            return ThinkResult::default();
        };
        let main_fn = match instance.get_typed_func::<(), ()>(&mut store, "main") {
            Ok(f) => f,
            Err(_) => {
                mark_dead(creature, DeathReason::InvalidProgram);
                return ThinkResult::default();
            }
        };
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
        *(*store_ptr).data_mut() =
            host_state_for_tick(creature, snapshot, tiles, config, ledger, tick);
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

    unsafe {
        if let Err(err) = prepare_think_fuel(&mut *store_ptr, budget) {
            if let Some(run) = run {
                cached.main_run = Some(stash_main_run(run));
            }
            mark_dead(creature, DeathReason::InvalidProgram);
            let _ = err;
            return ThinkResult::default();
        }

        if run.is_none() {
            run = Some(Box::pin(cached.main_fn.call_async(&mut *store_ptr, ())));
        }
    }
    let mut run = run.expect("main fiber started");

    // SAFETY: `run` holds the fiber's `store` borrow; use `store_ptr` only while polling.
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
    use std::sync::OnceLock;
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
    use crate::energy_ledger::EnergyLedger;
    use crate::vm::Creature;
    use crate::world_tile::WorldTiles;

    #[test]
    fn fuel_yield_pure_wasm_loop() {
        const SPIN: &str = r#"
(module
  (func (export "main")
    loop $l
      i32.const 1
      drop
      br $l
    end)
)
"#;
        let config = SimConfig {
            opcodes_per_tick: 64,
            energy_per_opcode: 1,
            ..SimConfig::default()
        };
        let mut creatures = vec![Creature {
            id: "spin".into(),
            energy: 10_000_000,
            wasm: crate::compile_wat(SPIN).unwrap(),
            code: SPIN.into(),
            alive: true,
            health: 100,
            max_health: 100,
            owner_uid: "u".into(),
            parent_id: None,
            inbox: vec![],
            death_reason: None,
            born_tick: 0,
            facing: 0,
            x: 0,
            y: 0,
        }];
        clear_vm_cache();
        crate::vm::run_tick(
            &mut creatures,
            &mut WorldTiles::new(),
            &mut EnergyLedger::default(),
            &config,
            1,
        );
        assert_eq!(
            creatures.len(),
            1,
            "{:?}",
            creatures.first().and_then(|c| c.death_reason)
        );
    }

    #[test]
    fn think_result_single_action_slot() {
        let mut result = ThinkResult::default();
        assert!(result.take_action(PendingAction::Move));
        assert!(!result.take_action(PendingAction::Rotate { delta: 1 }));
        assert_eq!(result.action, Some(PendingAction::Move));
    }

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
            slice_end: SliceEnd::None,
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
            slice_end: SliceEnd::None,
        };
        assert_eq!(first, next_random(&mut again));
        assert_ne!(first, second);
    }
}
