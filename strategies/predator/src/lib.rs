#![no_std]

use strategy_hunter::{host, predator_tick};

#[no_mangle]
pub extern "C" fn tick() {
    predator_tick();
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { host::sleep() };
    loop {}
}
