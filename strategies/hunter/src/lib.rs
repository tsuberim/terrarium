//! Shared hunter logic (dev-authored, compiled to WASM via `scripts/build-strategies.sh`).

#![no_std]

pub const KIND_CREATURE: i32 = 2;
pub const KIND_CORPSE: i32 = 3;

const VISION: i32 = 5;

const ADJACENT: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

pub mod host {
    #[link(wasm_import_module = "terrarium")]
    extern "C" {
        pub fn sleep();
        pub fn sense_at(dx: i32, dy: i32) -> i32;
        #[link_name = "move"]
        pub fn step(dir: i32) -> i32;
        pub fn eat(dir: i32) -> i32;
        pub fn random_byte() -> i32;
    }
}

fn dir_of(dx: i32, dy: i32) -> i32 {
    if dy < 0 {
        0
    } else if dx > 0 {
        1
    } else if dy > 0 {
        2
    } else {
        3
    }
}

fn step_toward(dx: i32, dy: i32) {
    unsafe {
        if dx > 0 {
            host::step(1);
        } else if dx < 0 {
            host::step(3);
        } else if dy > 0 {
            host::step(2);
        } else if dy < 0 {
            host::step(0);
        }
    }
}

fn ring_at(d: i32, i: i32) -> (i32, i32) {
    match i {
        0 => (d, 0),
        1 => (0, -d),
        2 => (-d, 0),
        3 => (0, d),
        4 => (d, d),
        5 => (d, -d),
        6 => (-d, -d),
        _ => (-d, d),
    }
}

/// Pick N/E/S/W from the sim RNG and step once.
pub fn wander() {
    let dir = unsafe { host::random_byte() } & 3;
    unsafe {
        host::step(dir as i32);
        host::sleep();
    }
}

/// Scan vision for `target` kind, eat if adjacent else step toward nearest seen.
pub fn tick(target: i32) {
    for (dx, dy) in ADJACENT {
        if unsafe { host::sense_at(dx, dy) } == target {
            unsafe {
                host::eat(dir_of(dx, dy));
                host::sleep();
            }
            return;
        }
    }

    let mut d = 1;
    while d <= VISION {
        let mut i = 0;
        while i < 8 {
            let (dx, dy) = ring_at(d, i);
            if unsafe { host::sense_at(dx, dy) } == target {
                step_toward(dx, dy);
                unsafe { host::sleep() };
                return;
            }
            i += 1;
        }
        d += 1;
    }

    wander();
}
