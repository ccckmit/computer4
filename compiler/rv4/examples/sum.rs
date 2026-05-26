#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

fn putchar(c: u8) {
    unsafe { core::arch::asm!("ecall", in("a0") c, in("a7") 1); }
}

fn puts(s: &[u8]) {
    unsafe { core::arch::asm!("ecall", in("a0") s.as_ptr(), in("a1") s.len(), in("a7") 2); }
}

fn putdec(mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 20;
    if n == 0 { putchar(b'0'); return; }
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    puts(&buf[i..]);
}

fn sum_to(n: u64) -> u64 {
    let mut s = 0;
    let mut i = 1;
    while i <= n { s += i; i += 1; }
    s
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let n = 100;
    puts(b"sum(1..100) = ");
    putdec(sum_to(n));
    putchar(b'\n');
    unsafe { core::arch::asm!("ecall", in("a0") 0, in("a7") 0, options(noreturn)); }
}
