#![no_std]
#![no_main]

mod os_code;

pub use os_code::rust_main;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}