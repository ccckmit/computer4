use core::fmt::Display;

use alloc::string::String;

use crate::file::File;
use crate::fs::FsError;
use crate::net::NetError;
use crate::param::NOFILE;
use crate::proc::{Proc, TrapFrame, current_proc, current_proc_and_data_mut};
use crate::sysfile::*;
use crate::sysnet::*;
use crate::sysproc::*;
use crate::vm::VA;

/// Syscall error codes using POSIX.1-2008 errno values.
///
/// Kernel encodes `-(errno as isize)` in the return register (`a0`).
/// User space decodes negative values back into `Errno` variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Errno {
    EPERM = 1,
    ENOENT = 2,
    ESRCH = 3,
    EINTR = 4,
    EIO = 5,
    ENOEXEC = 8,
    EBADF = 9,
    ECHILD = 10,
    EAGAIN = 11,
    ENOMEM = 12,
    EFAULT = 14,
    EEXIST = 17,
    EXDEV = 18,
    ENOTDIR = 20,
    EISDIR = 21,
    EINVAL = 22,
    ENFILE = 23,
    EMFILE = 24,
    ENOSPC = 28,
    EMLINK = 31,
    EPIPE = 32,
    ENAMETOOLONG = 36,
    ENOSYS = 38,
    ENOTEMPTY = 39,
    EMSGSIZE = 90,
}

impl Errno {
    pub fn code(self) -> u16 {
        self as u16
    }
}

impl From<u16> for Errno {
    fn from(code: u16) -> Self {
        match code {
            1 => Self::EPERM,
            2 => Self::ENOENT,
            3 => Self::ESRCH,
            4 => Self::EINTR,
            5 => Self::EIO,
            8 => Self::ENOEXEC,
            9 => Self::EBADF,
            10 => Self::ECHILD,
            11 => Self::EAGAIN,
            12 => Self::ENOMEM,
            14 => Self::EFAULT,
            17 => Self::EEXIST,
            18 => Self::EXDEV,
            20 => Self::ENOTDIR,
            21 => Self::EISDIR,
            22 => Self::EINVAL,
            23 => Self::ENFILE,
            24 => Self::EMFILE,
            28 => Self::ENOSPC,
            31 => Self::EMLINK,
            32 => Self::EPIPE,
            36 => Self::ENAMETOOLONG,
            38 => Self::ENOSYS,
            39 => Self::ENOTEMPTY,
            90 => Self::EMSGSIZE,
            _ => Self::EINVAL,
        }
    }
}

impl Display for Errno {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Errno::EPERM => write!(f, "operation not permitted"),
            Errno::ENOENT => write!(f, "no such entry"),
            Errno::ESRCH => write!(f, "no such process"),
            Errno::EINTR => write!(f, "interrupted"),
            Errno::EIO => write!(f, "input/output error"),
            Errno::ENOEXEC => write!(f, "exec format error"),
            Errno::EBADF => write!(f, "bad file descriptor"),
            Errno::ECHILD => write!(f, "no child processes"),
            Errno::EAGAIN => write!(f, "resource temporarily unavailable"),
            Errno::ENOMEM => write!(f, "cannot allocate memory"),
            Errno::EFAULT => write!(f, "bad address"),
            Errno::EEXIST => write!(f, "file exists"),
            Errno::EXDEV => write!(f, "cross-device link"),
            Errno::ENOTDIR => write!(f, "not a directory"),
            Errno::EISDIR => write!(f, "is a directory"),
            Errno::EINVAL => write!(f, "invalid argument"),
            Errno::ENFILE => write!(f, "too many open files in system"),
            Errno::EMFILE => write!(f, "too many open files"),
            Errno::ENOSPC => write!(f, "no space left on device"),
            Errno::EMLINK => write!(f, "too many links"),
            Errno::EPIPE => write!(f, "broken pipe"),
            Errno::ENAMETOOLONG => write!(f, "file name too long"),
            Errno::ENOSYS => write!(f, "function not implemented"),
            Errno::ENOTEMPTY => write!(f, "directory not empty"),
            Errno::EMSGSIZE => write!(f, "message too large"),
        }
    }
}

impl From<FsError> for Errno {
    fn from(e: FsError) -> Self {
        match e {
            FsError::OutOfBlock | FsError::OutOfInode => Errno::ENOSPC,
            FsError::OutOfFile | FsError::OutOfPipe => Errno::ENFILE,
            FsError::OutOfRange => Errno::EINVAL,
            FsError::Read | FsError::Write => Errno::EIO,
            FsError::Create => Errno::ENOSPC,
            FsError::Link => Errno::EEXIST,
            FsError::Resolve => Errno::ENOENT,
            FsError::Type => Errno::EINVAL,
            FsError::Copy => Errno::EFAULT,
        }
    }
}

impl From<NetError> for Errno {
    fn from(value: NetError) -> Self {
        match value {
            NetError::NotConfigured => Errno::EPERM,
            NetError::QueueFull => Errno::EAGAIN,
            NetError::TableFull => Errno::ENFILE,
            NetError::OutOfSocket => Errno::EAGAIN,
            NetError::PortInUse => Errno::EEXIST,
            NetError::BadSocket => Errno::EBADF,
            NetError::InvalidAddress => Errno::EINVAL,
            NetError::MalformedPacket => Errno::EINVAL,
            NetError::TransmitFailed => Errno::EIO,
            NetError::Interrupted => Errno::EINTR,
            NetError::RouteNotFound => Errno::ENOENT,
            NetError::PacketTooLarge => Errno::EINVAL,
            NetError::ResourceUnavailable => Errno::EAGAIN,
            NetError::InterfaceNotFound => Errno::ENOENT,
            NetError::ChecksumFailed => Errno::EINVAL,
        }
    }
}

/// Wrapper for extracting typed syscall arguments from trapframe.
pub struct SyscallArgs<'a> {
    trapframe: &'a TrapFrame,
    proc: &'static Proc,
}

impl<'a> SyscallArgs<'a> {
    /// Creates a new SyscallArgs
    fn new(trapframe: &'a TrapFrame, proc: &'static Proc) -> Self {
        Self { trapframe, proc }
    }

    pub fn proc(&self) -> &Proc {
        self.proc
    }

    /// Returns the argument at the given index as a usize.
    pub fn get_raw(&self, index: usize) -> usize {
        match index {
            0 => self.trapframe.a0,
            1 => self.trapframe.a1,
            2 => self.trapframe.a2,
            3 => self.trapframe.a3,
            4 => self.trapframe.a4,
            5 => self.trapframe.a5,
            _ => panic!("invalid syscall argument index {}", index),
        }
    }

    /// Returns the argument at the given index as an isize.
    pub fn get_int(&self, index: usize) -> isize {
        self.get_raw(index) as isize
    }

    /// Returns the argument at the given index as a virtual address.
    ///
    /// Does not check for legality, since `copyin`/`copyout` will do that.
    pub fn get_addr(&self, index: usize) -> VA {
        VA::from(self.get_raw(index))
    }

    /// Fetch the nth word-sized system call argument as a file descriptor and return both the
    /// descriptor and the corresponding `File`.
    pub fn get_file(&self, index: usize) -> Result<(usize, File), Errno> {
        let fd: usize = try_log!(
            self.get_int(index)
                .try_into()
                .or(Err(Errno::EBADF))
        );

        if fd >= NOFILE {
            err!(Errno::EBADF);
        }

        if let Some(file) = &current_proc().data().open_files[fd] {
            return Ok((fd, file.clone()));
        }

        err!(Errno::EBADF);
    }

    /// Fetches a null-terminated string from user space.
    pub fn fetch_string(&self, addr: VA, max: usize) -> Result<String, Errno> {
        let (_proc, data) = current_proc_and_data_mut();

        let mut result = String::with_capacity(max);

        let mut buf = [0u8; 1];
        for i in 0..max {
            try_log!(
                data.pagetable_mut()
                    .copy_from(VA::from(addr.as_usize() + i), &mut buf)
                    .map_err(|_| Errno::EFAULT)
            );

            if buf[0] == 0 {
                return Ok(result);
            }

            result.push(buf[0] as char);
        }

        Ok(result)
    }
}

/// System call numbers
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    Fork = 1,
    Exit = 2,
    Wait = 3,
    Pipe = 4,
    Read = 5,
    Kill = 6,
    Exec = 7,
    Fstat = 8,
    Chdir = 9,
    Dup = 10,
    Getpid = 11,
    Sbrk = 12,
    Sleep = 13,
    Uptime = 14,
    Open = 15,
    Write = 16,
    Mknod = 17,
    Unlink = 18,
    Link = 19,
    Mkdir = 20,
    Close = 21,
    Poweroff = 22,
    Ioctl = 23,
    Socket = 24,
    Send = 25,
    Receive = 26,
    Random = 27,
}

impl TryFrom<usize> for Syscall {
    type Error = Errno;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Syscall::Fork),
            2 => Ok(Syscall::Exit),
            3 => Ok(Syscall::Wait),
            4 => Ok(Syscall::Pipe),
            5 => Ok(Syscall::Read),
            6 => Ok(Syscall::Kill),
            7 => Ok(Syscall::Exec),
            8 => Ok(Syscall::Fstat),
            9 => Ok(Syscall::Chdir),
            10 => Ok(Syscall::Dup),
            11 => Ok(Syscall::Getpid),
            12 => Ok(Syscall::Sbrk),
            13 => Ok(Syscall::Sleep),
            14 => Ok(Syscall::Uptime),
            15 => Ok(Syscall::Open),
            16 => Ok(Syscall::Write),
            17 => Ok(Syscall::Mknod),
            18 => Ok(Syscall::Unlink),
            19 => Ok(Syscall::Link),
            20 => Ok(Syscall::Mkdir),
            21 => Ok(Syscall::Close),
            22 => Ok(Syscall::Poweroff),
            23 => Ok(Syscall::Ioctl),
            24 => Ok(Syscall::Socket),
            25 => Ok(Syscall::Send),
            26 => Ok(Syscall::Receive),
            27 => Ok(Syscall::Random),
            _ => Err(Errno::ENOSYS),
        }
    }
}

type SyscallHandler = fn(&SyscallArgs) -> Result<usize, Errno>;

fn wrap_exit(args: &SyscallArgs) -> Result<usize, Errno> {
    sys_exit(args)
}

fn wrap_poweroff(args: &SyscallArgs) -> Result<usize, Errno> {
    sys_poweroff(args)
}

const SYSCALL_TABLE: [Option<SyscallHandler>; 64] = {
    let mut table: [Option<SyscallHandler>; 64] = [None; 64];
    table[1] = Some(sys_fork);
    table[2] = Some(wrap_exit);
    table[3] = Some(sys_wait);
    table[4] = Some(sys_pipe);
    table[5] = Some(sys_read);
    table[6] = Some(sys_kill);
    table[7] = Some(sys_exec);
    table[8] = Some(sys_fstat);
    table[9] = Some(sys_chdir);
    table[10] = Some(sys_dup);
    table[11] = Some(sys_getpid);
    table[12] = Some(sys_sbrk);
    table[13] = Some(sys_sleep);
    table[14] = Some(sys_uptime);
    table[15] = Some(sys_open);
    table[16] = Some(sys_write);
    table[17] = Some(sys_mknod);
    table[18] = Some(sys_unlink);
    table[19] = Some(sys_link);
    table[20] = Some(sys_mkdir);
    table[21] = Some(sys_close);
    table[22] = Some(wrap_poweroff);
    table[23] = Some(sys_ioctl);
    table[24] = Some(sys_socket);
    table[25] = Some(sys_send);
    table[26] = Some(sys_receive);
    table[27] = Some(sys_random);
    table
};

/// Handle a system call.
///
/// # Safety
/// Called from `usertrap` in `trap.rs`.
#[unsafe(no_mangle)]
pub unsafe fn syscall(trapframe: &mut TrapFrame) {
    let proc = current_proc();
    let args = SyscallArgs::new(trapframe, proc);

    let num = trapframe.a7;
    let result = if num < SYSCALL_TABLE.len() {
        match SYSCALL_TABLE[num] {
            Some(handler) => handler(&args),
            None => Err(Errno::ENOSYS),
        }
    } else {
        Err(Errno::ENOSYS)
    };

    trapframe.a0 = match log!(result) {
        Ok(v) => v,
        Err(error) => {
            #[cfg(debug_assertions)]
            {
                let pid = *proc.inner.lock().pid;
                println!(
                    "! syscall error ({}) from proc {} ({})",
                    error,
                    pid,
                    proc.data().name,
                );
            }
            (-(error.code() as isize)) as usize
        }
    };
}
