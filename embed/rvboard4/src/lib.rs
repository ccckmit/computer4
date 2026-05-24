#![no_std]
#![allow(internal_features)]

extern "C" {
    fn print(s: *const u8);
}

#[no_mangle]
pub extern "C" fn hello() {
    let msg = b"hello rvboard4\n\0";
    unsafe { print(msg.as_ptr()) };
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}