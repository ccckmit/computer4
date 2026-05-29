use alloc::vec;

use crate::memlayout::QEMU_POWER;
use crate::proc::{self, Channel, Pid, current_proc};
use crate::rng::rand_bytes;
use crate::signal::{self, SigAction, NSIG, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK};
use crate::syscall::{Errno, SyscallArgs};
use crate::trap::TICKS;
use crate::vm::VA;

pub fn sys_exit(args: &SyscallArgs) -> ! {
    let n = args.get_int(0);
    proc::exit(n);
}

pub fn sys_getpid(args: &SyscallArgs) -> Result<usize, Errno> {
    let pid = args.proc().inner.lock().pid;
    Ok(*pid)
}

pub fn sys_fork(_args: &SyscallArgs) -> Result<usize, Errno> {
    match log!(proc::fork()) {
        Ok(pid) => Ok(*pid),
        Err(_) => Err(Errno::EAGAIN),
    }
}

pub fn sys_wait(args: &SyscallArgs) -> Result<usize, Errno> {
    let addr = args.get_addr(0);
    match proc::wait(addr) {
        Some(pid) => Ok(*pid),
        None => err!(Errno::ECHILD),
    }
}

pub fn sys_sbrk(args: &SyscallArgs) -> Result<usize, Errno> {
    let size = args.get_int(0);
    let addr = args.proc().data().size;

    match unsafe { log!(proc::grow(size, size >= 0)) } {
        Ok(_) => Ok(addr),
        Err(_) => Err(Errno::ENOMEM),
    }
}

pub fn sys_sleep(args: &SyscallArgs) -> Result<usize, Errno> {
    let duration = args.get_int(0).max(0) as usize;

    let mut ticks = TICKS.lock();
    let ticks0 = *ticks;

    while *ticks - ticks0 < duration {
        if current_proc().is_killed() {
            return Err(Errno::EINTR);
        }

        ticks = proc::sleep(Channel::Ticks, ticks);
    }

    Ok(0)
}

pub fn sys_kill(args: &SyscallArgs) -> Result<usize, Errno> {
    let pid = args.get_int(0);
    let sig = args.get_raw(1) as u32;

    // Safety: kernel will return an error if the process does not exist.
    if sig >= NSIG as u32 {
        return Err(Errno::EINVAL);
    }

    if proc::kill(unsafe { Pid::from_usize(pid as usize) }, sig) {
        Ok(0)
    } else {
        Err(Errno::ESRCH)
    }
}

pub fn sys_uptime(_args: &SyscallArgs) -> Result<usize, Errno> {
    let ticks = *TICKS.lock();
    Ok(ticks)
}

pub fn sys_random(args: &SyscallArgs) -> Result<usize, Errno> {
    let dest_buf = args.get_addr(0);
    let len = args.get_int(1) as usize;

    let mut src_buf = vec![0u8; len];
    rand_bytes(&mut src_buf);

    try_log!(proc::copy_to_user(&src_buf, dest_buf).map_err(|_| Errno::EFAULT));

    Ok(0)
}

pub fn sys_poweroff(args: &SyscallArgs) -> ! {
    let code = match args.get_int(0) as u32 {
        0 => 0x5555,
        c => (c << 16) | 0x3333,
    };

    println!("! powering off...");

    unsafe { *(QEMU_POWER as *mut u32) = code };

    unreachable!("poweroff failed");
}
