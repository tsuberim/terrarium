#![no_std]

mod user;

#[no_mangle]
pub extern "C" fn main() {
    user::main();
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    terrarium_sdk::sleep();
    loop {}
}
