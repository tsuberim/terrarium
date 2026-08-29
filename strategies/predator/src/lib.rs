#![no_std]

use strategy_hunter::{host, tick as hunt, KIND_CREATURE};

#[no_mangle]
pub extern "C" fn tick() {
    hunt(KIND_CREATURE);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { host::sleep() };
    loop {}
}
