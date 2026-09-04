#![no_std]

use strategy_hunter::{host, prey_tick};

#[no_mangle]
pub extern "C" fn main() {
    loop {
        prey_tick();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { host::sleep() };
    loop {}
}
