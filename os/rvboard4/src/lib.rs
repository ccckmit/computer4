#![no_std]
#![allow(internal_features)]

extern "C" {
    fn putchar(c: u8);
}

#[no_mangle]
pub extern "C" fn hello() {
    let msg = b"hello rvboard4\n\0";
    let mut i = 0;
    while msg[i] != 0 {
        unsafe { putchar(msg[i]) };
        i += 1;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}