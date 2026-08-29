//! Shared hunter logic (dev-authored, compiled to WASM via `scripts/build-strategies.sh`).

#![no_std]

pub const KIND_CREATURE: i32 = 2;
pub const KIND_CORPSE: i32 = 3;

/// Prey alarm — attracts hawks and scavengers.
pub const SIG_ALARM: i32 = 0x01;
/// Predator hunt ping — marks active chase.
pub const SIG_HUNT: i32 = 0x02;

/// Bud a clone when energy exceeds this (10× corpse floor).
pub const SPAWN_THRESHOLD: i64 = 10_000_000;
/// Energy transferred to each child (corpse floor).
pub const SPAWN_GIFT: i32 = 1_000_000;

const VISION: i32 = 5;
const SENSE_SIZE: usize = 24;
const RECV_SIZE: usize = 36;

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
        #[link_name = "move"]
        pub fn step(dir: i32) -> i32;
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

fn sense_kind(dq: i32, dr: i32) -> i32 {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(SENSE_BUF) as *mut u8 as i32;
        host::sense(dq, dr, ptr);
        read_i32(core::ptr::addr_of!(SENSE_BUF).cast(), 0)
    }
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

fn step_toward(dq: i32, dr: i32) {
    unsafe {
        if dq > 0 {
            host::step(0);
        } else if dq < 0 {
            host::step(3);
        } else if dr > 0 {
            host::step(5);
        } else if dr < 0 {
            host::step(2);
        }
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

fn act_adjacent(dir: i32, strike: bool) {
    unsafe {
        if strike {
            host::hit(dir);
        } else {
            host::eat(dir);
        }
        host::sleep();
    }
}

fn wander() {
    let dir = unsafe { host::random_byte() } % 6;
    unsafe {
        host::step(dir as i32);
        host::sleep();
    }
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
                host::spawn(dir, SPAWN_GIFT);
                host::sleep();
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

/// Scan vision for `target` kind; `strike` hits live creatures, else eats corpses.
pub fn tick(target: i32, strike: bool) {
    for (dq, dr) in ADJACENT {
        if sense_kind(dq, dr) == target {
            act_adjacent(dir_of(dq, dr), strike);
            return;
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
                    unsafe { host::sleep() };
                    return;
                }
                (q, r) = step_axial(q, r, walk[face as usize]);
                s += 1;
            }
            face += 1;
        }
        d += 1;
    }

    wander();
}

/// Eat adjacent corpses, then hunt live creatures (hit until dead, eat next tick).
pub fn predator_tick() {
    if maybe_clone() {
        return;
    }
    for (dq, dr) in ADJACENT {
        if sense_kind(dq, dr) == KIND_CORPSE {
            act_adjacent(dir_of(dq, dr), false);
            return;
        }
    }
    tick(KIND_CREATURE, true);
}

/// Rush prey alarms (competition for kills), else scavenge corpses.
pub fn scavenger_tick() {
    if maybe_clone() {
        return;
    }
    if follow_signal(SIG_ALARM) {
        unsafe { host::sleep() };
        return;
    }
    tick(KIND_CORPSE, false);
}

fn flee_from(dq: i32, dr: i32) -> bool {
    if sense_kind(dq, dr) != KIND_CREATURE {
        return false;
    }
    let away = (dir_of(dq, dr) + 3) % 6;
    unsafe {
        host::step(away);
        host::signal_broadcast(SIG_ALARM);
        host::sleep();
    }
    true
}

/// Flee adjacent predators; alarm draws hawks and scavengers.
pub fn prey_tick() {
    if maybe_clone() {
        return;
    }
    for (dq, dr) in ADJACENT {
        if flee_from(dq, dr) {
            return;
        }
    }
    unsafe { host::sleep() };
}

/// Rush prey alarms to compete for the kill.
pub fn hawk_tick() {
    if maybe_clone() {
        return;
    }
    if follow_signal(SIG_ALARM) {
        unsafe { host::sleep() };
        return;
    }
    unsafe { host::sleep() };
}
