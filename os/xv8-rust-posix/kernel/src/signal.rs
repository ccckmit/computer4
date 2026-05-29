/// Maximum number of signals
pub const NSIG: usize = 32;

/// POSIX signal numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Signal {
    SIGHUP = 1,
    SIGINT = 2,
    SIGQUIT = 3,
    SIGILL = 4,
    SIGTRAP = 5,
    SIGABRT = 6,
    SIGBUS = 7,
    SIGFPE = 8,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGSEGV = 11,
    SIGUSR2 = 12,
    SIGPIPE = 13,
    SIGALRM = 14,
    SIGTERM = 15,
    SIGCHLD = 17,
    SIGCONT = 18,
    SIGSTOP = 19,
    SIGTSTP = 20,
}

impl Signal {
    pub fn from_raw(sig: u32) -> Option<Self> {
        use Signal::*;
        Some(match sig {
            1 => SIGHUP,
            2 => SIGINT,
            3 => SIGQUIT,
            4 => SIGILL,
            5 => SIGTRAP,
            6 => SIGABRT,
            7 => SIGBUS,
            8 => SIGFPE,
            9 => SIGKILL,
            10 => SIGUSR1,
            11 => SIGSEGV,
            12 => SIGUSR2,
            13 => SIGPIPE,
            14 => SIGALRM,
            15 => SIGTERM,
            17 => SIGCHLD,
            18 => SIGCONT,
            19 => SIGSTOP,
            20 => SIGTSTP,
            _ => return None,
        })
    }

    pub fn raw(self) -> u32 {
        self as u32
    }
}

/// Default action for a signal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigDefault {
    Terminate,
    Ignore,
    Stop,
}

/// Returns the default action for a given signal number (1-indexed).
pub fn default_action(sig: u32) -> SigDefault {
    match sig {
        9 | 1 | 3 | 4 | 5 | 6 | 7 | 8 | 11 | 13 | 15 => SigDefault::Terminate,
        17 | 18 => SigDefault::Ignore,
        19 | 20 => SigDefault::Stop,
        _ => SigDefault::Terminate,
    }
}

/// Signal action (simplified POSIX sigaction)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SigAction {
    /// SIG_DFL=0, SIG_IGN=1, or user-space handler address
    pub handler: usize,
    pub flags: u32,
    pub mask: u32,
}

impl Default for SigAction {
    fn default() -> Self {
        Self {
            handler: 0, // SIG_DFL
            flags: 0,
            mask: 0,
        }
    }
}

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;

/// sigprocmask how values
pub const SIG_BLOCK: u32 = 0;
pub const SIG_UNBLOCK: u32 = 1;
pub const SIG_SETMASK: u32 = 2;

/// Bitmask helpers
pub fn sigbit(sig: u32) -> u32 {
    1 << (sig - 1)
}
