#![no_std]

mod user;

#[no_mangle]
pub extern "C" fn tick() {
    user::tick();
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
