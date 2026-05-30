use alloc::vec;

use crate::memlayout::QEMU_POWER;
use crate::proc::{self, Channel, MmapRegion, Pid, current_proc};
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

pub fn sys_mmap(args: &SyscallArgs) -> Result<usize, Errno> {
    use crate::riscv::{pg_round_up, PGSIZE, PTE_W, PTE_X};

    let addr = args.get_addr(0);
    let length = args.get_int(1) as usize;
    let prot = args.get_raw(2) as usize;
    let flags = args.get_raw(3) as usize;
    let fd = args.get_int(4);
    let _offset = args.get_int(5) as usize;

    const MAP_ANONYMOUS: usize = 0x20;

    if length == 0 || prot == 0 {
        return Err(Errno::EINVAL);
    }

    let length = pg_round_up(length);

    if flags & MAP_ANONYMOUS == 0 {
        return Err(Errno::ENOSYS);
    }

    let proc = current_proc();
    let data = unsafe { proc.data_mut() };
    let cur_size = data.size;
    let pagetable = data.pagetable_mut();

    let start = if addr == VA::new(0) {
        pg_round_up(cur_size)
    } else {
        if addr % PGSIZE != 0 {
            return Err(Errno::EINVAL);
        }
        addr.as_usize()
    };

    let xperm = if prot & 0x2 != 0 { PTE_W } else { 0 }
        | if prot & 0x4 != 0 { PTE_X } else { 0 };

    try_log!(pagetable.alloc(start, start + length, xperm).map_err(|_| Errno::ENOMEM));

    data.mmap_regions.push(MmapRegion {
        start,
        len: length,
        prot,
        flags,
    });

    let end = start + length;
    if end > data.size {
        data.size = end;
    }

    Ok(start)
}

pub fn sys_munmap(args: &SyscallArgs) -> Result<usize, Errno> {
    use crate::riscv::PGSIZE;

    let addr = args.get_addr(0);
    let length = args.get_int(1) as usize;

    if addr % PGSIZE != 0 {
        return Err(Errno::EINVAL);
    }

    if length == 0 {
        return Err(Errno::EINVAL);
    }

    let length = if length % PGSIZE != 0 {
        (length / PGSIZE + 1) * PGSIZE
    } else {
        length
    };

    let proc = current_proc();
    let data = unsafe { proc.data_mut() };
    let addr_val = addr.as_usize();
    let tl_end = addr_val + length;

    // Remove matching regions before borrowing pagetable
    data.mmap_regions.retain(|r| {
        let r_end = r.start + r.len;
        tl_end <= r.start || r_end <= addr_val
    });

    let pagetable = data.pagetable_mut();
    pagetable.unmap(addr, length / PGSIZE, true);

    Ok(0)
}

pub fn sys_mprotect(args: &SyscallArgs) -> Result<usize, Errno> {
    use crate::riscv::{pg_round_down, PGSIZE, PTE_U, PTE_R, PTE_W, PTE_X};

    let addr = args.get_addr(0);
    let length = args.get_int(1) as usize;
    let prot = args.get_raw(2) as usize;

    let addr_val = pg_round_down(addr.as_usize());
    if length == 0 {
        return Err(Errno::EINVAL);
    }
    let length = if length % PGSIZE != 0 {
        (length / PGSIZE + 1) * PGSIZE
    } else {
        length
    };

    let mut perm = PTE_U;
    if prot & 0x1 != 0 { perm |= PTE_R; }
    if prot & 0x2 != 0 { perm |= PTE_W; }
    if prot & 0x4 != 0 { perm |= PTE_X; }

    let proc = current_proc();
    let data = unsafe { proc.data_mut() };

    for va in (addr_val..addr_val + length).step_by(PGSIZE) {
        if let Ok(pte) = data.pagetable_mut().0.walk_mut(VA::from(va), false) {
            if pte.is_v() && pte.is_leaf() {
                let pa = pte.as_pa();
                *pte = pa.as_pte() | perm;
            }
        }
    }

    let end = addr_val + length;
    for r in data.mmap_regions.iter_mut() {
        let r_end = r.start + r.len;
        if (addr_val >= r.start && addr_val < r_end) || (r.start >= addr_val && r.start < end) {
            r.prot = prot;
        }
    }

    Ok(0)
}

pub fn sys_setsid(args: &SyscallArgs) -> Result<usize, Errno> {
    let proc = args.proc();
    let mut inner = proc.inner.lock();

    if inner.sid == *inner.pid || inner.pgid == *inner.pid {
        return Err(Errno::EPERM);
    }

    inner.pgid = *inner.pid;
    inner.sid = *inner.pid;

    Ok(*inner.pid)
}

pub fn sys_getpgid(args: &SyscallArgs) -> Result<usize, Errno> {
    let pid_arg = args.get_int(0);

    let pid = if pid_arg == 0 {
        let proc = args.proc();
        *proc.inner.lock().pid
    } else {
        pid_arg as usize
    };

    for proc in crate::proc::PROC_TABLE.iter() {
        let inner = proc.inner.lock();
        if inner.state != crate::proc::ProcState::Unused && *inner.pid == pid {
            return Ok(inner.pgid);
        }
    }

    Err(Errno::ESRCH)
}

pub fn sys_getppid(args: &SyscallArgs) -> Result<usize, Errno> {
    let proc = args.proc();
    let parents = crate::proc::PROC_TABLE.parents.lock();
    if let Some(parent_id) = parents[proc.id] {
        let parent = crate::proc::PROC_TABLE.get(parent_id);
        let parent_pid = *parent.inner.lock().pid;
        Ok(parent_pid)
    } else {
        Ok(0)
    }
}

pub fn sys_nice(args: &SyscallArgs) -> Result<usize, Errno> {
    let inc = args.get_int(0) as i8;

    let proc = args.proc();
    let mut inner = proc.inner.lock();

    let new_nice = (inner.nice as i16 + inc as i16).max(-20).min(19);
    inner.nice = new_nice as i8;

    Ok(new_nice as usize)
}
