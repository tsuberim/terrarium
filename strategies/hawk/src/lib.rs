#![no_std]

use strategy_hunter::{host, hawk_tick};

#[no_mangle]
pub extern "C" fn main() {
    loop {
        hawk_tick();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { host::sleep() };
    loop {}
}
