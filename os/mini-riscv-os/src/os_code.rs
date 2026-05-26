use core::arch::asm;
use core::ptr;
use core::sync::atomic::{AtomicI32, Ordering, spin_loop_hint};

const MAX_TASK: usize = 2;
const STACK_SIZE: usize = 1024;

const UART: usize = 0x10000000;
const UART_THR: *mut u8 = (UART + 0x00) as *mut u8;
const UART_LSR: *mut u8 = (UART + 0x05) as *mut u8;
const UART_LSR_EMPTY_MASK: u8 = 0x40;

const CLINT: usize = 0x2000000;
const CLINT_MTIME: usize = CLINT + 0xBFF8;
const fn clint_mtimecmp(hartid: usize) -> usize { CLINT + 0x4000 + 4 * hartid }

const MSTATUS_MIE: usize = 1 << 3;
const MIE_MTIE: usize = 1 << 7;

#[repr(C)]
#[derive(Copy, Clone)]
struct Context {
    ra: u32, sp: u32, s0: u32, s1: u32, s2: u32, s3: u32,
    s4: u32, s5: u32, s6: u32, s7: u32, s8: u32, s9: u32,
    s10: u32, s11: u32,
}

static mut TASK_STACK: [[u8; STACK_SIZE]; MAX_TASK] = [[0; STACK_SIZE]; MAX_TASK];
static mut CTX_TASKS: [Context; MAX_TASK] = [Context {
    ra: 0, sp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0,
    s5: 0, s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
}; MAX_TASK];
static mut CTX_OS: Context = Context { ra: 0, sp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0 };
static mut CTX_NOW: *mut Context = ptr::null_mut();
static mut TASK_TOP: usize = 0;
static mut SHARED_COUNTER: i32 = 0;
static mut TIMER_COUNT: i32 = 0;
static mut TIMER_SCRATCH: [u32; 32] = [0; 32];

struct Mutex {
    lock: AtomicI32,
}

impl Mutex {
    pub const fn new() -> Self { Mutex { lock: AtomicI32::new(0) } }
    #[inline]
    pub fn lock(&self) {
        loop {
            while self.lock.load(Ordering::Relaxed) != 0 { core::hint::spin_loop(); }
            if self.lock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() { return; }
        }
    }
    #[inline]
    pub fn unlock(&self) { self.lock.store(0, Ordering::Release); }
}

static M: Mutex = Mutex::new();

fn putc(c: u8) {
    unsafe {
        while (*UART_LSR & UART_LSR_EMPTY_MASK) == 0 {}
        UART_THR.write_volatile(c);
    }
}

fn puts(s: &[u8]) {
    for &c in s { if c == 0 { break } putc(c); }
}

fn puti(mut n: i32) {
    if n < 0 { putc(b'-'); n = -n; }
    let mut digits = [0u8; 10];
    let mut i = 0;
    loop {
        digits[i] = (n % 10) as u8;
        i += 1;
        n /= 10;
        if n == 0 { break; }
    }
    while i > 0 { i -= 1; putc(b'0' + digits[i]); }
}

fn delay(count: i32) {
    let mut c = count * 50000;
    while c > 0 { c -= 1; }
}

fn r_mhartid() -> usize { let x: usize; unsafe { asm!("csrr {0}, mhartid", out(reg) x) }; x }
fn r_mstatus() -> usize { let x: usize; unsafe { asm!("csrr {0}, mstatus", out(reg) x) }; x }
fn w_mstatus(x: usize) { unsafe { asm!("csrw mstatus, {0}", in(reg) x) } }
fn w_mtvec(x: usize) { unsafe { asm!("csrw mtvec, {0}", in(reg) x) } }
fn w_mie(x: usize) { unsafe { asm!("csrw mie, {0}", in(reg) x) } }
fn r_mie() -> usize { let x: usize; unsafe { asm!("csrr {0}, mie", out(reg) x) }; x }
fn w_mscratch(x: usize) { unsafe { asm!("csrw mscratch, {0}", in(reg) x) } }

fn get_mtime() -> u64 {
    unsafe {
        let low = *(CLINT_MTIME as *const u32);
        let high = *((CLINT_MTIME + 4) as *const u32);
        ((high as u64) << 32) | (low as u64)
    }
}

fn set_mtimecmp(hartid: usize, val: u64) {
    unsafe {
        let hi = clint_mtimecmp(hartid) + 4;
        let lo = clint_mtimecmp(hartid);
        *(lo as *mut u32) = 0xFFFFFFFF;
        *(hi as *mut u32) = (val >> 32) as u32;
        *(lo as *mut u32) = (val & 0xFFFFFFFF) as u32;
    }
}

extern "C" {
    fn sys_switch(ctx_old: *mut Context, ctx_new: *mut Context);
    fn sys_timer();
}

unsafe fn task_switch_to(i: usize) {
    CTX_NOW = &mut CTX_TASKS[i];
    sys_switch(&mut CTX_OS as *mut Context, &mut CTX_TASKS[i] as *mut Context);
}

unsafe fn task_switch_back() {
    let ctx = CTX_NOW;
    CTX_NOW = &mut CTX_OS;
    sys_switch(ctx, &mut CTX_OS as *mut Context);
}

unsafe fn task_create(task: unsafe extern "C" fn()) -> i32 {
    let i = TASK_TOP;
    TASK_TOP += 1;
    CTX_TASKS[i].ra = task as u32;
    CTX_TASKS[i].sp = &TASK_STACK[i][STACK_SIZE - 1] as *const u8 as u32;
    i as i32
}

#[no_mangle]
pub unsafe extern "C" fn timer_handler() {
    puts(b"timer_handler: ");
    puti(TIMER_COUNT);
    puts(b"\n");
    TIMER_COUNT += 1;
    task_switch_back();
}

unsafe extern "C" fn user_task0() {
    puts(b"Task0: Created!\n");
    loop {
        M.lock();
        puts(b"Task0: counter before = ");
        puti(SHARED_COUNTER);
        puts(b"\n");
        SHARED_COUNTER += 1;
        puts(b"Task0: counter after  = ");
        puti(SHARED_COUNTER);
        puts(b"\n");
        M.unlock();
        delay(1000);
    }
}

unsafe extern "C" fn user_task1() {
    puts(b"Task1: Created!\n");
    loop {
        M.lock();
        puts(b"Task1: counter before = ");
        puti(SHARED_COUNTER);
        puts(b"\n");
        SHARED_COUNTER += 1;
        puts(b"Task1: counter after  = ");
        puti(SHARED_COUNTER);
        puts(b"\n");
        M.unlock();
        delay(1000);
    }
}

unsafe fn timer_init() {
    let id = r_mhartid();
    let interval: u64 = 20000000;
    set_mtimecmp(id, get_mtime() + interval);
    TIMER_SCRATCH[3] = clint_mtimecmp(id) as u32;
    TIMER_SCRATCH[4] = interval as u32;
    w_mscratch(TIMER_SCRATCH.as_mut_ptr() as usize);
    w_mtvec(sys_timer as usize);
    w_mstatus(r_mstatus() | MSTATUS_MIE);
    w_mie(r_mie() | MIE_MTIE);
}

#[no_mangle]
pub unsafe extern "C" fn rust_main() -> ! {
    puts(b"OS start\n");
    timer_init();
    task_create(user_task0);
    task_create(user_task1);

    let mut current_task: i32 = 0;
    loop {
        puts(b"OS: Activate next task\n");
        task_switch_to(current_task as usize);
        puts(b"OS: Back to OS\n\n");
        current_task = (current_task + 1) % TASK_TOP as i32;
    }
}