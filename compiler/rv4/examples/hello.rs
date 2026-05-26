#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

const MSG: &[u8] = b"Hello, World!\n";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") MSG.as_ptr(),
            in("a1") MSG.len(),
            in("a7") 2,
        );
    }
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") 0,
            in("a7") 0,
            options(noreturn)
        );
    }
}
