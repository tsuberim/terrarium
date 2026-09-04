#![no_std]

use strategy_hunter::{host, scavenger_tick};

#[no_mangle]
pub extern "C" fn main() {
    loop {
        scavenger_tick();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { host::sleep() };
    loop {}
}
