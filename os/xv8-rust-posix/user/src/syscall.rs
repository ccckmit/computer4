pub mod raw {
    use core::arch::asm;

    use kernel::abi::{Ioctl, Stat, Syscall, Termios};

    #[inline(always)]
    fn syscall0(syscall: Syscall) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                lateout("a0") ret,
            );
        }
        ret
    }

    #[inline(always)]
    fn syscall1(syscall: Syscall, a0: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                inlateout("a0") a0 as isize => ret,
            );
        }
        ret
    }

    #[inline(always)]
    fn syscall2(syscall: Syscall, a0: usize, a1: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                inlateout("a0") a0 as isize => ret,
                in("a1") a1,
            );
        }
        ret
    }

    #[inline(always)]
    fn syscall3(syscall: Syscall, a0: usize, a1: usize, a2: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                inlateout("a0") a0 as isize => ret,
                in("a1") a1,
                in("a2") a2,
            );
        }
        ret
    }

    #[inline(always)]
    fn syscall5(syscall: Syscall, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                inlateout("a0") a0 as isize => ret,
                in("a1") a1,
                in("a2") a2,
                in("a3") a3,
                in("a4") a4,
            );
        }
        ret
    }

    #[inline(always)]
    fn syscall6(syscall: Syscall, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                inlateout("a0") a0 as isize => ret,
                in("a1") a1,
                in("a2") a2,
                in("a3") a3,
                in("a4") a4,
                in("a5") a5,
            );
        }
        ret
    }

    pub fn fork() -> isize {
        syscall0(Syscall::Fork)
    }

    pub fn exit(code: usize) -> ! {
        syscall1(Syscall::Exit, code);
        unreachable!();
    }

    pub fn wait(status: *mut usize) -> isize {
        syscall1(Syscall::Wait, status as usize)
    }

    pub fn pipe(fds: *mut usize) -> isize {
        syscall1(Syscall::Pipe, fds as usize)
    }

    pub fn read(fd: usize, buf: *mut u8, len: usize) -> isize {
        syscall3(Syscall::Read, fd, buf as usize, len)
    }

    pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        syscall3(Syscall::Write, fd, buf as usize, len)
    }

    pub fn kill(pid: usize, sig: usize) -> isize {
        syscall2(Syscall::Kill, pid, sig)
    }

    pub fn exec(path: *const u8, argv: *const *const u8) -> isize {
        syscall2(Syscall::Exec, path as usize, argv as usize)
    }

    pub fn fstat(fd: usize, stat: *mut Stat) -> isize {
        syscall2(Syscall::Fstat, fd, stat as usize)
    }

    pub fn chdir(path: *const u8) -> isize {
        syscall1(Syscall::Chdir, path as usize)
    }

    pub fn dup(fd: usize) -> isize {
        syscall1(Syscall::Dup, fd)
    }

    pub fn getpid() -> isize {
        syscall0(Syscall::Getpid)
    }

    pub fn sbrk(n: usize) -> isize {
        syscall1(Syscall::Sbrk, n)
    }

    pub fn sleep(ticks: usize) -> isize {
        syscall1(Syscall::Sleep, ticks)
    }

    pub fn uptime() -> isize {
        syscall0(Syscall::Uptime)
    }

    pub fn open(path: *const u8, flags: usize) -> isize {
        syscall2(Syscall::Open, path as usize, flags)
    }

    pub fn close(fd: usize) -> isize {
        syscall1(Syscall::Close, fd)
    }

    pub fn mknod(path: *const u8, major: usize, minor: usize) -> isize {
        syscall3(Syscall::Mknod, path as usize, major, minor)
    }

    pub fn unlink(path: *const u8) -> isize {
        syscall1(Syscall::Unlink, path as usize)
    }

    pub fn link(old: *const u8, new: *const u8) -> isize {
        syscall2(Syscall::Link, old as usize, new as usize)
    }

    pub fn mkdir(path: *const u8) -> isize {
        syscall1(Syscall::Mkdir, path as usize)
    }

    pub fn poweroff(code: u32) -> ! {
        syscall1(Syscall::Poweroff, code as usize);
        unreachable!();
    }

    pub fn ioctl(fd: usize, cmd: usize, arg: usize) -> isize {
        syscall3(Syscall::Ioctl, fd, cmd, arg)
    }

    pub fn tcgetattr(fd: usize, termios: *mut Termios) -> isize {
        syscall3(Syscall::Ioctl, fd, Ioctl::TCGETS, termios as usize)
    }

    pub fn tcsetattr(fd: usize, termios: *const Termios) -> isize {
        syscall3(Syscall::Ioctl, fd, Ioctl::TCSETS, termios as usize)
    }

    pub fn socket(port: u16) -> isize {
        syscall1(Syscall::Socket, port as usize)
    }

    pub fn send(
        fd: usize,
        buf: *const u8,
        len: usize,
        dest_ip: *const u8,
        dest_port: u16,
    ) -> isize {
        syscall5(
            Syscall::Send,
            fd,
            buf as usize,
            len,
            dest_ip as usize,
            dest_port as usize,
        )
    }

    pub fn receive(
        fd: usize,
        buf: *mut u8,
        len: usize,
        src_ip: *mut u8,
        src_port: *mut u16,
    ) -> isize {
        syscall5(
            Syscall::Receive,
            fd,
            buf as usize,
            len,
            src_ip as usize,
            src_port as usize,
        )
    }

    pub fn random(buf: *mut u8, len: usize) -> isize {
        syscall2(Syscall::Random, buf as usize, len)
    }

    pub fn sigaction(sig: usize, act: *const u8, oldact: *mut u8) -> isize {
        syscall3(Syscall::Sigaction, sig, act as usize, oldact as usize)
    }

    pub fn sigprocmask(how: usize, set: *const u8, oldset: *mut u8) -> isize {
        syscall3(Syscall::Sigprocmask, how, set as usize, oldset as usize)
    }

    pub fn sigpending(set: *mut u8) -> isize {
        syscall1(Syscall::Sigpending, set as usize)
    }

    pub fn sigsuspend(mask: usize) -> isize {
        syscall1(Syscall::Sigsuspend, mask)
    }

    pub fn lseek(fd: usize, offset: isize, whence: usize) -> isize {
        syscall3(Syscall::Lseek, fd, offset as usize, whence)
    }

    pub fn truncate(path: *const u8) -> isize {
        syscall1(Syscall::Truncate, path as usize)
    }

    pub fn ftruncate(fd: usize, len: usize) -> isize {
        syscall2(Syscall::Ftruncate, fd, len)
    }

    pub fn getdents(fd: usize, buf: *mut u8, len: usize) -> isize {
        syscall3(Syscall::Getdents, fd, buf as usize, len)
    }

    pub fn symlink(target: *const u8, linkpath: *const u8) -> isize {
        syscall2(Syscall::Symlink, target as usize, linkpath as usize)
    }

    pub fn readlink(path: *const u8, buf: *mut u8, len: usize) -> isize {
        syscall3(Syscall::Readlink, path as usize, buf as usize, len)
    }

    pub fn access(path: *const u8, mode: usize) -> isize {
        syscall2(Syscall::Access, path as usize, mode)
    }

    pub fn fcntl(fd: usize, cmd: usize, arg: usize) -> isize {
        syscall3(Syscall::Fcntl, fd, cmd, arg)
    }

    pub fn dup2(oldfd: usize, newfd: usize) -> isize {
        syscall2(Syscall::Dup2, oldfd, newfd)
    }

    pub fn mmap(addr: usize, length: usize, prot: usize, flags: usize, fd: usize, offset: usize) -> isize {
        syscall6(Syscall::Mmap, addr, length, prot, flags, fd, offset)
    }

    pub fn munmap(addr: usize, length: usize) -> isize {
        syscall2(Syscall::Munmap, addr, length)
    }

    pub fn mprotect(addr: usize, length: usize, prot: usize) -> isize {
        syscall3(Syscall::Mprotect, addr, length, prot)
    }

    pub fn setsid() -> isize {
        syscall0(Syscall::SetSid)
    }

    pub fn getpgid(pid: usize) -> isize {
        syscall1(Syscall::GetPgid, pid)
    }

    pub fn getppid() -> isize {
        syscall0(Syscall::GetPpid)
    }

    pub fn nice(inc: isize) -> isize {
        syscall1(Syscall::Nice, inc as usize)
    }

    pub fn clock_gettime(clock_id: u32, tp: usize) -> isize {
        syscall2(Syscall::ClockGetTime, clock_id as usize, tp)
    }

    pub fn nanosleep(req: usize, rem: usize) -> isize {
        syscall2(Syscall::NanoSleep, req, rem)
    }

    pub fn chmod(path: usize, mode: u16) -> isize {
        syscall2(Syscall::Chmod, path, mode as usize)
    }

    pub fn chown(path: usize, uid: u32, gid: u32) -> isize {
        syscall3(Syscall::Chown, path, uid as usize, gid as usize)
    }

    pub fn umask(mask: u16) -> isize {
        syscall1(Syscall::Umask, mask as usize)
    }

    pub fn getuid() -> isize {
        syscall0(Syscall::GetUid)
    }

    pub fn getgid() -> isize {
        syscall0(Syscall::GetGid)
    }

    pub fn setuid(uid: u32) -> isize {
        syscall1(Syscall::SetUid, uid as usize)
    }

    pub fn setgid(gid: u32) -> isize {
        syscall1(Syscall::SetGid, gid as usize)
    }
}

use kernel::abi::{MAXPATH, Stat, Errno, SigAction, Timespec, Termios};
use kernel::abi::{CLOCK_MONOTONIC, CLOCK_REALTIME};

/// A file descriptor returned by or passed to syscalls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fd(usize);

impl Fd {
    pub const STDIN: Fd = Fd(0);
    pub const STDOUT: Fd = Fd(1);
    pub const STDERR: Fd = Fd(2);

    /// Returns the raw file descriptor number.
    pub fn as_raw(&self) -> usize {
        self.0
    }

    /// Creates an `Fd` from a raw file descriptor number.
    pub fn from_raw(fd: usize) -> Self {
        Self(fd)
    }
}

impl core::fmt::Display for Fd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated path suitable for passing to syscalls.
///
/// Guarantees that the inner string is shorter than `MAXPATH` and contains no
/// embedded null bytes, so it can be safely null-terminated on the stack.
#[derive(Debug, Clone, Copy)]
struct Path<'a>(&'a str);

impl<'a> Path<'a> {
    fn new(s: &'a str) -> Result<Self, Errno> {
        if s.len() >= MAXPATH || s.bytes().any(|b| b == 0) {
            return Err(Errno::ENAMETOOLONG);
        }
        Ok(Self(s))
    }

    /// Creates a null-terminated C-string buffer on the stack.
    fn as_cpath(&self) -> [u8; MAXPATH] {
        let mut buf = [0u8; MAXPATH];
        buf[..self.0.len()].copy_from_slice(self.0.as_bytes());
        buf
    }
}

/// Converts a raw signed syscall return into `Result`, treating negative values as error codes.
#[inline(always)]
fn check(ret: isize) -> Result<usize, Errno> {
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(Errno::from((-ret) as u16))
    }
}

/// Converts a raw syscall return into `Result<(), Errno>`.
#[inline(always)]
fn check_unit(ret: isize) -> Result<(), Errno> {
    check(ret).map(|_| ())
}

/// Validates a path string and creates a C-compatible path buffer.
fn validate_path(path: &str) -> Result<[u8; MAXPATH], Errno> {
    Ok(Path::new(path)?.as_cpath())
}

pub fn fork() -> Result<usize, Errno> {
    check(raw::fork())
}

pub fn exit(code: usize) -> ! {
    raw::exit(code)
}

pub fn exit_with_msg(msg: &str) -> ! {
    eprintln!("{}", msg);
    exit(1);
}

pub fn wait(status: &mut usize) -> Result<usize, Errno> {
    check(raw::wait(status as *mut usize))
}

pub fn pipe() -> Result<(Fd, Fd), Errno> {
    let mut fds = [0usize; 2];
    check_unit(raw::pipe(fds.as_mut_ptr()))?;
    Ok((Fd(fds[0]), Fd(fds[1])))
}

pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize, Errno> {
    check(raw::read(fd.as_raw(), buf.as_mut_ptr(), buf.len()))
}

pub fn write(fd: Fd, buf: &[u8]) -> Result<usize, Errno> {
    check(raw::write(fd.as_raw(), buf.as_ptr(), buf.len()))
}

pub fn kill(pid: usize, sig: usize) -> Result<(), Errno> {
    check_unit(raw::kill(pid, sig))
}

/// Replaces the current process image with the program at `path`.
///
/// `argv` contains the argument strings. This function packs them into a contiguous
/// stack buffer with null terminators and builds the pointer array expected by the kernel.
///
/// Returns `Errno` because if `exec` returns at all, it failed.

pub fn exec(path: &str, argv: &[&str]) -> Errno {
    let cpath = match validate_path(path) {
        Ok(cpath) => cpath,
        Err(e) => return e,
    };

    const MAX_ARGV: usize = 16;
    const BUF_SIZE: usize = 512;

    let mut buf = [0u8; BUF_SIZE];
    let mut ptrs: [*const u8; MAX_ARGV + 1] = [core::ptr::null(); MAX_ARGV + 1];
    let mut offset = 0;

    for (i, arg) in argv.iter().enumerate().take(MAX_ARGV) {
        ptrs[i] = buf[offset..].as_ptr();
        buf[offset..offset + arg.len()].copy_from_slice(arg.as_bytes());
        // buf is zeroed, so the byte after the arg is already a null terminator
        offset += arg.len() + 1;
    }
    // ptrs is already null-terminated (initialized to null)

    let ret = raw::exec(cpath.as_ptr(), ptrs.as_ptr());
    // exec only returns on failure
    Errno::from((-ret) as u16)
}

pub fn fstat(fd: Fd, stat: &mut Stat) -> Result<(), Errno> {
    check_unit(raw::fstat(fd.as_raw(), stat as *mut Stat))
}

pub fn chdir(path: &str) -> Result<(), Errno> {
    let cpath = validate_path(path)?;
    check_unit(raw::chdir(cpath.as_ptr()))
}

pub fn dup(fd: Fd) -> Result<Fd, Errno> {
    check(raw::dup(fd.as_raw())).map(Fd)
}

pub fn getpid() -> usize {
    raw::getpid() as usize
}

pub fn sbrk(n: isize) -> Result<usize, Errno> {
    check(raw::sbrk(n as usize))
}

pub fn sleep(ticks: usize) -> Result<(), Errno> {
    check_unit(raw::sleep(ticks))
}

pub fn uptime() -> usize {
    raw::uptime() as usize
}

pub fn open(path: &str, flags: usize) -> Result<Fd, Errno> {
    let cpath = validate_path(path)?;
    check(raw::open(cpath.as_ptr(), flags)).map(Fd)
}

pub fn close(fd: Fd) -> Result<(), Errno> {
    check_unit(raw::close(fd.as_raw()))
}

pub fn mknod(path: &str, major: usize, minor: usize) -> Result<(), Errno> {
    let cpath = validate_path(path)?;
    check_unit(raw::mknod(cpath.as_ptr(), major, minor))
}

pub fn unlink(path: &str) -> Result<(), Errno> {
    let cpath = validate_path(path)?;
    check_unit(raw::unlink(cpath.as_ptr()))
}

pub fn link(old: &str, new: &str) -> Result<(), Errno> {
    let cold = validate_path(old)?;
    let cnew = validate_path(new)?;
    check_unit(raw::link(cold.as_ptr(), cnew.as_ptr()))
}

pub fn mkdir(path: &str) -> Result<(), Errno> {
    let cpath = validate_path(path)?;
    check_unit(raw::mkdir(cpath.as_ptr()))
}

pub fn poweroff(code: u32) -> ! {
    raw::poweroff(code)
}

pub fn ioctl(fd: Fd, cmd: usize, arg: usize) -> Result<usize, Errno> {
    check(raw::ioctl(fd.as_raw(), cmd, arg))
}

pub fn socket(port: u16) -> Result<Fd, Errno> {
    check(raw::socket(port)).map(Fd)
}

pub fn send(fd: Fd, buf: &[u8], dest_ip: &[u8; 4], dest_port: u16) -> Result<usize, Errno> {
    check(raw::send(
        fd.as_raw(),
        buf.as_ptr(),
        buf.len(),
        dest_ip.as_ptr(),
        dest_port,
    ))
}

pub fn receive(
    fd: Fd,
    buf: &mut [u8],
    src_ip: &mut [u8; 4],
    src_port: &mut u16,
) -> Result<usize, Errno> {
    check(raw::receive(
        fd.as_raw(),
        buf.as_mut_ptr(),
        buf.len(),
        src_ip.as_mut_ptr(),
        src_port as *mut u16,
    ))
}

pub fn random(buf: &mut [u8]) -> Result<(), Errno> {
    check_unit(raw::random(buf.as_mut_ptr(), buf.len()))
}

pub fn sigaction(sig: usize, act: Option<&SigAction>, oldact: Option<&mut SigAction>) -> Result<(), Errno> {
    let act_ptr = act.map_or(core::ptr::null::<SigAction>(), |a| a as *const SigAction);
    let oldact_ptr = oldact.map_or(core::ptr::null_mut::<SigAction>(), |a| a as *mut SigAction);
    check_unit(raw::sigaction(
        sig,
        act_ptr as *const u8,
        oldact_ptr as *mut u8,
    ))
}

pub fn sigprocmask(how: usize, set: Option<&u32>, oldset: Option<&mut u32>) -> Result<(), Errno> {
    let set_ptr = set.map_or(core::ptr::null::<u32>(), |s| s as *const u32);
    let oldset_ptr = oldset.map_or(core::ptr::null_mut::<u32>(), |s| s as *mut u32);
    check_unit(raw::sigprocmask(
        how,
        set_ptr as *const u8,
        oldset_ptr as *mut u8,
    ))
}

pub fn sigpending() -> Result<u32, Errno> {
    let mut set: u32 = 0;
    check_unit(raw::sigpending((&mut set) as *mut u32 as *mut u8))?;
    Ok(set)
}

pub fn sigsuspend(_mask: u32) -> Result<(), Errno> {
    Err(Errno::EINTR)
}

pub fn lseek(fd: Fd, offset: isize, whence: usize) -> Result<usize, Errno> {
    check(raw::lseek(fd.as_raw(), offset, whence))
}

pub fn truncate(path: &str) -> Result<(), Errno> {
    let cpath = validate_path(path)?;
    check_unit(raw::truncate(cpath.as_ptr()))
}

pub fn ftruncate(fd: Fd, len: usize) -> Result<(), Errno> {
    check_unit(raw::ftruncate(fd.as_raw(), len))
}

pub fn getdents(fd: Fd, buf: &mut [u8]) -> Result<usize, Errno> {
    check(raw::getdents(fd.as_raw(), buf.as_mut_ptr(), buf.len()))
}

pub fn symlink(target: &str, linkpath: &str) -> Result<(), Errno> {
    let ctarget = validate_path(target)?;
    let clinkpath = validate_path(linkpath)?;
    check_unit(raw::symlink(ctarget.as_ptr(), clinkpath.as_ptr()))
}

pub fn readlink(path: &str, buf: &mut [u8]) -> Result<usize, Errno> {
    let cpath = validate_path(path)?;
    check(raw::readlink(cpath.as_ptr(), buf.as_mut_ptr(), buf.len()))
}

pub fn access(path: &str, mode: usize) -> Result<(), Errno> {
    let cpath = validate_path(path)?;
    check_unit(raw::access(cpath.as_ptr(), mode))
}

pub fn fcntl(fd: Fd, cmd: usize, arg: usize) -> Result<usize, Errno> {
    check(raw::fcntl(fd.as_raw(), cmd, arg))
}

pub fn dup2(oldfd: Fd, newfd: usize) -> Result<usize, Errno> {
    check(raw::dup2(oldfd.as_raw(), newfd))
}

/// Maps `length` bytes starting at `addr` with the given `prot` and `flags`.
///
/// Returns the mapped address on success.
pub fn mmap(addr: *const u8, length: usize, prot: usize, flags: usize, fd: usize, offset: usize) -> Result<*mut u8, Errno> {
    check(raw::mmap(addr as usize, length, prot, flags, fd, offset)).map(|a| a as *mut u8)
}

/// Unmaps `length` bytes starting at `addr`.
pub fn munmap(addr: *const u8, length: usize) -> Result<(), Errno> {
    check_unit(raw::munmap(addr as usize, length))
}

/// Changes protection for `length` bytes starting at `addr`.
pub fn mprotect(addr: *const u8, length: usize, prot: usize) -> Result<(), Errno> {
    check_unit(raw::mprotect(addr as usize, length, prot))
}

// mmap protection flags
pub const PROT_NONE: usize = 0x0;
pub const PROT_READ: usize = 0x1;
pub const PROT_WRITE: usize = 0x2;
pub const PROT_EXEC: usize = 0x4;

// mmap flags
pub const MAP_SHARED: usize = 0x01;
pub const MAP_PRIVATE: usize = 0x02;
pub const MAP_FIXED: usize = 0x10;
pub const MAP_ANONYMOUS: usize = 0x20;

/// Creates a new session. The calling process becomes the session leader.
pub fn setsid() -> Result<usize, Errno> {
    check(raw::setsid())
}

/// Returns the process group ID of the process with the given PID.
/// If `pid` is 0, returns the calling process's PGID.
pub fn getpgid(pid: usize) -> Result<usize, Errno> {
    check(raw::getpgid(pid))
}

/// Returns the parent process ID of the calling process.
pub fn getppid() -> Result<usize, Errno> {
    check(raw::getppid())
}

/// Changes the nice value of the calling process by `inc`.
/// Returns the new nice value.
pub fn nice(inc: isize) -> Result<isize, Errno> {
    check(raw::nice(inc)).map(|v| (v as isize) - 20)
}

/// Gets the time of the given clock.
pub fn clock_gettime(clock_id: u32, tp: &mut Timespec) -> Result<(), Errno> {
    check_unit(raw::clock_gettime(clock_id, tp as *mut _ as usize))
}

/// Sleeps with nanosecond precision.
/// Returns `EINTR` if interrupted by a signal, with remaining time written to `rem`.
pub fn nanosleep(req: &Timespec, rem: &mut Timespec) -> Result<(), Errno> {
    check_unit(raw::nanosleep(req as *const _ as usize, rem as *mut _ as usize))
}

/// Changes permissions of a file.
pub fn chmod(path: &str, mode: u16) -> Result<(), Errno> {
    check_unit(raw::chmod(path.as_ptr() as usize, mode))
}

/// Changes owner and group of a file.
pub fn chown(path: &str, uid: u32, gid: u32) -> Result<(), Errno> {
    check_unit(raw::chown(path.as_ptr() as usize, uid, gid))
}

/// Sets the file creation mask and returns the previous mask.
pub fn umask(mask: u16) -> u16 {
    raw::umask(mask) as u16
}

/// Returns the real user ID of the calling process.
pub fn getuid() -> u32 {
    raw::getuid() as u32
}

/// Returns the real group ID of the calling process.
pub fn getgid() -> u32 {
    raw::getgid() as u32
}

/// Sets the user ID of the calling process.
pub fn setuid(uid: u32) -> Result<(), Errno> {
    check_unit(raw::setuid(uid))
}

/// Sets the group ID of the calling process.
pub fn setgid(gid: u32) -> Result<(), Errno> {
    check_unit(raw::setgid(gid))
}

/// Gets terminal attributes for the given fd.
pub fn tcgetattr(fd: Fd, termios: &mut Termios) -> Result<(), Errno> {
    check_unit(raw::tcgetattr(fd.as_raw(), termios as *mut Termios))
}

/// Sets terminal attributes for the given fd.
pub fn tcsetattr(fd: Fd, termios: &Termios) -> Result<(), Errno> {
    check_unit(raw::tcsetattr(fd.as_raw(), termios as *const Termios))
}
