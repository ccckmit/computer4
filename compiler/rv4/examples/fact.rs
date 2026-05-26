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

fn fact(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * fact(n - 1) }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let n = 10;
    let r = fact(n);
    puts(b"fact(");
    putdec(n);
    puts(b") = ");
    putdec(r);
    putchar(b'\n');
    unsafe { core::arch::asm!("ecall", in("a0") 0, in("a7") 0, options(noreturn)); }
}
