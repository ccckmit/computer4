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

fn fib(n: u64) -> u64 {
    if n <= 1 { return n; }
    let mut a = 0u64;
    let mut b = 1u64;
    let mut i = 2;
    while i <= n {
        let t = a + b;
        a = b;
        b = t;
        i += 1;
    }
    b
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    puts(b"fibonacci(20) = ");
    putdec(fib(20));
    putchar(b'\n');
    unsafe { core::arch::asm!("ecall", in("a0") 0, in("a7") 0, options(noreturn)); }
}
