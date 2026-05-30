// Exports common ABI types and constants for use by userspace programs.
pub use crate::file::{CONSOLE, Ioctl, OpenFlag, Whence};
pub use crate::fs::{DIRSIZE, Dirent, Directory, InodeType, Stat, mode};
pub use crate::net::Ipv4Addr;
pub use crate::param::MAXPATH;
pub use crate::signal::SigAction;
pub use crate::syscall::Errno;
pub use crate::sysfile::{F_DUPFD, F_GETFD, F_SETFD, F_GETFL};

pub use crate::syscall::Syscall;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_BLOCK: u32 = 0;
pub const SIG_UNBLOCK: u32 = 1;
pub const SIG_SETMASK: u32 = 2;
pub const NSIG: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

pub const CLOCK_REALTIME: u32 = 0;
pub const CLOCK_MONOTONIC: u32 = 1;
