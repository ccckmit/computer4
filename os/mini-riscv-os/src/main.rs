#![no_std]
#![no_main]

mod os_code;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}