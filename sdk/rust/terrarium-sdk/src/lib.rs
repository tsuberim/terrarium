//! Terrarium creature SDK for `wasm32-unknown-unknown`.
//!
//! Player code is body-only in the editor; the compile worker wraps it in `main` with prelude imported.

#![no_std]

pub mod host;

pub mod tile {
    pub const EMPTY: i32 = 0;
    pub const SOLID: i32 = 1;
    pub const CREATURE: i32 = 2;
    pub const CORPSE: i32 = 3;
    pub const FOOD: i32 = 4;
}

const SENSE_SIZE: usize = 24;

static mut SENSE_BUF: [u8; SENSE_SIZE] = [0; SENSE_SIZE];

fn read_i32_at(offset: usize) -> i32 {
    unsafe {
        let bytes: *const [u8; 4] = SENSE_BUF[offset..].as_ptr().cast();
        i32::from_le_bytes(*bytes)
    }
}

pub fn sleep() {
    unsafe { host::sleep() };
}

pub fn energy() -> i64 {
    unsafe { host::energy() }
}

pub fn move_forward() -> i32 {
    unsafe { host::step(0) }
}

pub fn eat_forward() -> i32 {
    unsafe { host::eat(0) }
}

pub fn rotate(delta: i32) -> i32 {
    unsafe { host::rotate(delta) }
}

/// Sense hex offset `(dq, dr)`. Returns tile kind or `-1` if out of vision.
pub fn sense_kind(dq: i32, dr: i32) -> i32 {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(SENSE_BUF) as *mut u8 as i32;
        if host::sense(dq, dr, ptr) == 0 {
            return -1;
        }
        read_i32_at(0)
    }
}

pub fn random_byte() -> u8 {
    unsafe { host::random_byte() as u8 }
}

pub fn uptime() -> i32 {
    unsafe { host::uptime() }
}

pub mod prelude {
    pub use crate::{eat_forward, energy, move_forward, rotate, sense_kind, sleep, tile, uptime};
}
