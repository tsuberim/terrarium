//! Shared hunter logic (dev-authored, compiled to WASM via `scripts/build-strategies.sh`).

#![no_std]

pub const KIND_CREATURE: i32 = 2;
pub const KIND_CORPSE: i32 = 3;
pub const KIND_FOOD: i32 = 4;

/// Prey alarm — attracts hawks and scavengers.
pub const SIG_ALARM: i32 = 0x01;
/// Predator hunt ping — marks active chase.
pub const SIG_HUNT: i32 = 0x02;

/// Bud a clone when energy exceeds this (5× corpse floor).
pub const SPAWN_THRESHOLD: i64 = 5_000_000;
/// Energy transferred to each child — must stay above corpse floor to survive.
pub const SPAWN_GIFT: i32 = 2_000_000;

const VISION: i32 = 5;
const SENSE_SIZE: usize = 24;
const RECV_SIZE: usize = 36;
/// Host actions per sim tick (move/eat/hit/spawn each count as one).
const ACTIONS_PER_TICK: u32 = 2;

/// Hex neighbors: E, NE, NW, W, SW, SE offsets (q, r).
const ADJACENT: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

static mut SENSE_BUF: [u8; SENSE_SIZE] = [0; SENSE_SIZE];
static mut RECV_BUF: [u8; RECV_SIZE] = [0; RECV_SIZE];

pub mod host {
    #[link(wasm_import_module = "terrarium")]
    extern "C" {
        pub fn sleep();
        pub fn sense(dq: i32, dr: i32, ptr: i32) -> i32;
        pub fn recv(ptr: i32) -> i32;
        pub fn pos_x() -> i32;
        pub fn pos_y() -> i32;
        pub fn facing() -> i32;
        pub fn rotate(delta: i32) -> i32;
        #[link_name = "move"]
        pub fn step(_forward: i32) -> i32;
        pub fn eat(dir: i32) -> i32;
        pub fn hit(dir: i32) -> i32;
        pub fn signal_broadcast(byte: i32) -> i32;
        pub fn random_byte() -> i32;
        pub fn energy() -> i64;
        pub fn spawn(dir: i32, energy: i32) -> i32;
    }
}

fn read_i32(buf: *const u8, off: usize) -> i32 {
    unsafe {
        i32::from_le_bytes([
            *buf.add(off),
            *buf.add(off + 1),
            *buf.add(off + 2),
            *buf.add(off + 3),
        ])
    }
}

fn read_i64(buf: *const u8, off: usize) -> i64 {
    unsafe {
        i64::from_le_bytes([
            *buf.add(off),
            *buf.add(off + 1),
            *buf.add(off + 2),
            *buf.add(off + 3),
            *buf.add(off + 4),
            *buf.add(off + 5),
            *buf.add(off + 6),
            *buf.add(off + 7),
        ])
    }
}

fn sense_at(dq: i32, dr: i32) -> (i32, i64) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(SENSE_BUF) as *mut u8 as i32;
        host::sense(dq, dr, ptr);
        let b = core::ptr::addr_of!(SENSE_BUF).cast();
        (read_i32(b, 0), read_i64(b, 8))
    }
}

fn sense_kind(dq: i32, dr: i32) -> i32 {
    sense_at(dq, dr).0
}

pub fn is_edible(kind: i32) -> bool {
    kind == KIND_CORPSE || kind == KIND_FOOD
}

fn dir_of(dq: i32, dr: i32) -> i32 {
    let mut i = 0;
    while i < 6 {
        if ADJACENT[i].0 == dq && ADJACENT[i].1 == dr {
            return i as i32;
        }
        i += 1;
    }
    0
}

fn relative_dir(dq: i32, dr: i32) -> i32 {
    let abs = dir_of(dq, dr);
    let facing = unsafe { host::facing() };
    ((abs - facing) % 6 + 6) % 6
}

fn turn_to_abs(abs: i32) {
    unsafe {
        let facing = host::facing();
        let rel = ((abs - facing) % 6 + 6) % 6;
        if rel == 0 {
            return;
        }
        let delta = if rel <= 3 { rel } else { rel - 6 };
        host::rotate(delta);
    }
}

fn step_forward() {
    unsafe {
        host::step(0);
    }
}

fn step_toward(dq: i32, dr: i32) {
    let abs = dir_of(dq, dr);
    let rel = relative_dir(dq, dr);
    if rel == 0 {
        step_forward();
    } else {
        turn_to_abs(abs);
    }
}

fn step_toward_cell(fq: i32, fr: i32) {
    unsafe {
        step_toward(fq - host::pos_x(), fr - host::pos_y());
    }
}

fn step_axial(q: i32, r: i32, dir: i32) -> (i32, i32) {
    match dir {
        0 => (q + 1, r),
        1 => (q + 1, r - 1),
        2 => (q, r - 1),
        3 => (q - 1, r),
        4 => (q - 1, r + 1),
        _ => (q, r + 1),
    }
}

fn finish_tick() {
    unsafe { host::sleep() };
}

fn run_actions(mut step: impl FnMut() -> bool) {
    let mut n = 0u32;
    while n < ACTIONS_PER_TICK {
        if !step() {
            break;
        }
        n += 1;
    }
    finish_tick();
}

fn act_adjacent(dq: i32, dr: i32, strike: bool) {
    let abs = dir_of(dq, dr);
    if unsafe { host::facing() } != abs {
        turn_to_abs(abs);
        return;
    }
    unsafe {
        if strike {
            host::hit(0);
        } else {
            host::eat(0);
        }
    }
}

fn wander_step() -> bool {
    let abs = unsafe { host::random_byte() % 6 } as i32;
    if unsafe { host::facing() } != abs {
        turn_to_abs(abs);
        return true;
    }
    step_forward();
    true
}

/// Clone self on a random empty adjacent hex when well-fed.
fn maybe_clone() -> bool {
    unsafe {
        if host::energy() <= SPAWN_THRESHOLD {
            return false;
        }
        let start = (host::random_byte() % 6) as i32;
        let mut i = 0;
        while i < 6 {
            let dir = (start + i) % 6;
            let (dq, dr) = ADJACENT[dir as usize];
            if sense_kind(dq, dr) == 0 {
                if unsafe { host::facing() } != dir {
                    turn_to_abs(dir);
                    finish_tick();
                    return true;
                }
                host::spawn(0, SPAWN_GIFT);
                finish_tick();
                return true;
            }
            i += 1;
        }
        false
    }
}

fn follow_signal(byte: i32) -> bool {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RECV_BUF) as *mut u8 as i32;
        if host::recv(ptr) == 0 {
            return false;
        }
        let b = core::ptr::addr_of!(RECV_BUF).cast();
        if read_i32(b, 12) != byte {
            return false;
        }
        step_toward_cell(read_i32(b, 4), read_i32(b, 8));
        true
    }
}

/// Eat adjacent food, else step toward the richest corpse or food in vision.
fn seek_food(corpse: bool, food: bool, wander_if_empty: bool) -> bool {
    for (dq, dr) in ADJACENT {
        let (kind, _) = sense_at(dq, dr);
        let ok = (corpse && kind == KIND_CORPSE) || (food && kind == KIND_FOOD);
        if ok {
            act_adjacent(dq, dr, false);
            return true;
        }
    }

    let mut best_energy = 0i64;
    let mut best_q = 0i32;
    let mut best_r = 0i32;
    let mut found = false;

    let mut d = 1;
    while d <= VISION {
        let mut q = d;
        let mut r = 0;
        let walk = [2i32, 3, 4, 5, 0, 1];
        let mut face = 0;
        while face < 6 {
            let mut s = 0;
            while s < d {
                let (kind, energy) = sense_at(q, r);
                let ok = (corpse && kind == KIND_CORPSE) || (food && kind == KIND_FOOD);
                if ok && energy > best_energy {
                    best_energy = energy;
                    best_q = q;
                    best_r = r;
                    found = true;
                }
                (q, r) = step_axial(q, r, walk[face as usize]);
                s += 1;
            }
            face += 1;
        }
        d += 1;
    }

    if found {
        step_toward(best_q, best_r);
        return true;
    }

    if wander_if_empty {
        return wander_step();
    }
    false
}

/// One hunt step toward `target`; `strike` hits live creatures, else eats.
fn hunt_step(target: i32, strike: bool) -> bool {
    for (dq, dr) in ADJACENT {
        if sense_kind(dq, dr) == target {
            act_adjacent(dq, dr, strike);
            return true;
        }
    }

    let mut d = 1;
    while d <= VISION {
        let mut q = d;
        let mut r = 0;
        let walk = [2i32, 3, 4, 5, 0, 1];
        let mut face = 0;
        while face < 6 {
            let mut s = 0;
            while s < d {
                if sense_kind(q, r) == target {
                    if strike && target == KIND_CREATURE {
                        unsafe {
                            host::signal_broadcast(SIG_HUNT);
                        }
                    }
                    step_toward(q, r);
                    return true;
                }
                (q, r) = step_axial(q, r, walk[face as usize]);
                s += 1;
            }
            face += 1;
        }
        d += 1;
    }

    wander_step()
}

fn flee_from(dq: i32, dr: i32) -> bool {
    if sense_kind(dq, dr) != KIND_CREATURE {
        return false;
    }
    let away = (dir_of(dq, dr) + 3) % 6;
    if unsafe { host::facing() } != away {
        turn_to_abs(away);
        return true;
    }
    step_forward();
    unsafe {
        host::signal_broadcast(SIG_ALARM);
    }
    true
}

fn predator_step() -> bool {
    if seek_food(true, true, false) {
        return true;
    }
    hunt_step(KIND_CREATURE, true)
}

fn scavenger_step() -> bool {
    if follow_signal(SIG_ALARM) {
        return true;
    }
    seek_food(true, true, true)
}

fn prey_step() -> bool {
    for (dq, dr) in ADJACENT {
        if flee_from(dq, dr) {
            return true;
        }
    }
    if seek_food(false, true, false) {
        return true;
    }
    wander_step()
}

fn hawk_step() -> bool {
    if follow_signal(SIG_ALARM) {
        return true;
    }
    seek_food(true, true, true)
}

/// Graze food and corpses, then hunt live prey.
pub fn predator_tick() {
    if maybe_clone() {
        return;
    }
    run_actions(|| predator_step());
}

/// Rush prey alarms, else forage corpses and food.
pub fn scavenger_tick() {
    if maybe_clone() {
        return;
    }
    run_actions(|| scavenger_step());
}

/// Flee predators; graze food when safe.
pub fn prey_tick() {
    if maybe_clone() {
        return;
    }
    run_actions(|| prey_step());
}

/// Rush prey alarms, else forage like a scavenger.
pub fn hawk_tick() {
    if maybe_clone() {
        return;
    }
    run_actions(|| hawk_step());
}
