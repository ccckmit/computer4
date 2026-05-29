use alloc::vec;

use crate::memlayout::QEMU_POWER;
use crate::proc::{self, Channel, Pid, current_proc};
use crate::rng::rand_bytes;
use crate::signal::{SigAction, NSIG, SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK};
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

pub fn sys_sigaction(args: &SyscallArgs) -> Result<usize, Errno> {
    let sig = args.get_raw(0) as u32;
    let act_addr = args.get_addr(1);
    let oldact_addr = args.get_addr(2);
    let null_va = VA::new(0);

    if sig == 0 || sig >= NSIG as u32 {
        return Err(Errno::EINVAL);
    }

    // SIGKILL and SIGSTOP cannot be caught or ignored
    if sig == 9 || sig == 19 {
        return Err(Errno::EINVAL);
    }

    let proc = current_proc();
    let data = unsafe { proc.data_mut() };

    // Return old action if requested
    if oldact_addr != null_va {
        let oldact = SigAction {
            handler: data.sig_handlers[sig as usize],
            flags: data.sig_flags[sig as usize],
            mask: data.sig_masks[sig as usize],
        };
        try_log!(
            proc::copy_to_user(
                unsafe {
                    core::slice::from_raw_parts(
                        &oldact as *const SigAction as *const u8,
                        core::mem::size_of::<SigAction>(),
                    )
                },
                oldact_addr,
            )
            .map_err(|_| Errno::EFAULT)
        );
    }

    // Set new action if provided
    if act_addr != null_va {
        let mut act_buf = [0u8; core::mem::size_of::<SigAction>()];
        try_log!(
            proc::copy_from_user(act_addr, &mut act_buf).map_err(|_| Errno::EFAULT)
        );
        let act: SigAction = unsafe { core::ptr::read_unaligned(act_buf.as_ptr() as *const SigAction) };

        data.sig_handlers[sig as usize] = act.handler;
        data.sig_flags[sig as usize] = act.flags;
        data.sig_masks[sig as usize] = act.mask;
    }

    Ok(0)
}

pub fn sys_sigprocmask(args: &SyscallArgs) -> Result<usize, Errno> {
    let how = args.get_raw(0) as u32;
    let set_addr = args.get_addr(1);
    let oldset_addr = args.get_addr(2);
    let null_va = VA::new(0);

    let proc = current_proc();
    let mut inner = proc.inner.lock();

    // Return old mask if requested
    if oldset_addr != null_va {
        let val = inner.blocked;
        try_log!(
            proc::copy_to_user(
                unsafe { core::slice::from_raw_parts(&val as *const u32 as *const u8, 4) },
                oldset_addr,
            )
            .map_err(|_| Errno::EFAULT)
        );
    }

    // Set new mask if provided
    if set_addr != null_va {
        let mut buf = [0u8; 4];
        try_log!(
            proc::copy_from_user(set_addr, &mut buf).map_err(|_| Errno::EFAULT)
        );
        let set = u32::from_ne_bytes(buf);

        match how {
            SIG_BLOCK => inner.blocked |= set,
            SIG_UNBLOCK => inner.blocked &= !set,
            SIG_SETMASK => inner.blocked = set,
            _ => return Err(Errno::EINVAL),
        }
    }

    Ok(0)
}

pub fn sys_sigpending(args: &SyscallArgs) -> Result<usize, Errno> {
    let set_addr = args.get_addr(0);
    let proc = current_proc();
    let inner = proc.inner.lock();

    let val = inner.pending;
    try_log!(
        proc::copy_to_user(
            unsafe { core::slice::from_raw_parts(&val as *const u32 as *const u8, 4) },
            set_addr,
        )
        .map_err(|_| Errno::EFAULT)
    );

    Ok(0)
}

pub fn sys_sigsuspend(args: &SyscallArgs) -> Result<usize, Errno> {
    let mask = args.get_raw(0) as u32;
    let proc = current_proc();

    // Save and replace blocked mask
    let saved_blocked = {
        let mut inner = proc.inner.lock();
        let saved = inner.blocked;
        inner.blocked = mask;
        saved
    };

    // Check if any unmasked signal is already pending
    let has_pending = {
        let inner = proc.inner.lock();
        (inner.pending & !inner.blocked) != 0
    };

    if !has_pending {
        // Sleep until woken by a signal
        proc::sleep(Channel::Proc(proc.id), proc.inner.lock());
    }

    // Restore blocked mask
    proc.inner.lock().blocked = saved_blocked;

    // sigsuspend always returns EINTR after signal delivery
    Err(Errno::EINTR)
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
