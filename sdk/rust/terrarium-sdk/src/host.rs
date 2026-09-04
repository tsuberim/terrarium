//! Raw host imports — ABI v2 (guest memory + `act()`).

use crate::mem::{write_action, Payload};

#[link(wasm_import_module = "terrarium")]
extern "C" {
    pub fn rand() -> u64;
    pub fn act() -> i32;
    pub fn recv() -> i32;
}

pub fn submit(payload: &Payload) -> i32 {
    write_action(payload);
    unsafe { act() }
}
