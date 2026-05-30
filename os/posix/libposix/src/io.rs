use core::fmt;
#[cfg(target_os = "none")]
use alloc::{string::String, vec::Vec};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    InvalidInput,
    UnexpectedEof,
    BrokenPipe,
    Interrupted,
    Other,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: &'static str,
}

impl Error {
    pub const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Error { kind, message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            let n = self.write(remaining)?;
            remaining = &remaining[n..];
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<()>;
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        platform_read_stdin(buf)
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        platform_write_stdout(buf)
    }
    fn flush(&mut self) -> Result<()> {
        platform_flush_stdout()
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        platform_write_stderr(buf)
    }
    fn flush(&mut self) -> Result<()> {
        platform_flush_stderr()
    }
}

pub fn stdin() -> Stdin { Stdin }
pub fn stdout() -> Stdout { Stdout }
pub fn stderr() -> Stderr { Stderr }

#[derive(Debug)]
pub struct File {
    fd: i32,
}

impl File {
    pub fn open(path: &str) -> Result<Self> {
        platform_open(path)
    }
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        platform_read_fd(self.fd, buf)
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        platform_write_fd(self.fd, buf)
    }
    pub fn close(self) -> Result<()> {
        platform_close_fd(self.fd)
    }
}

#[cfg(unix)]
mod sys {
    use std::io::{self, Read as IoRead, Write as IoWrite};

    pub fn read_stdin(buf: &mut [u8]) -> super::Result<usize> {
        io::stdin().read(buf).map_err(from_std_error)
    }
    pub fn write_stdout(buf: &[u8]) -> super::Result<usize> {
        io::stdout().write(buf).map_err(from_std_error)
    }
    pub fn flush_stdout() -> super::Result<()> {
        io::stdout().flush().map_err(from_std_error)
    }
    pub fn write_stderr(buf: &[u8]) -> super::Result<usize> {
        io::stderr().write(buf).map_err(from_std_error)
    }
    pub fn flush_stderr() -> super::Result<()> {
        io::stderr().flush().map_err(from_std_error)
    }
    pub fn open(path: &str) -> super::Result<super::File> {
        use libc::{open, O_RDONLY};
        let fd = unsafe { open(path.as_ptr() as *const libc::c_char, O_RDONLY) };
        if fd < 0 {
            return Err(super::Error::new(super::ErrorKind::NotFound, "open failed"));
        }
        Ok(super::File { fd })
    }
    pub fn read_fd(fd: i32, buf: &mut [u8]) -> super::Result<usize> {
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "read failed"));
        }
        Ok(n as usize)
    }
    pub fn write_fd(fd: i32, buf: &[u8]) -> super::Result<usize> {
        let n = unsafe { libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "write failed"));
        }
        Ok(n as usize)
    }
    pub fn close_fd(fd: i32) -> super::Result<()> {
        let r = unsafe { libc::close(fd) };
        if r < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "close failed"));
        }
        Ok(())
    }
    pub fn args() -> Vec<String> {
        std::env::args().collect()
    }
    pub fn exit(code: i32) -> ! {
        std::process::exit(code)
    }
    pub fn print(s: &str) {
        print!("{}", s);
    }
    pub fn println(s: &str) {
        println!("{}", s);
    }
    fn from_std_error(e: io::Error) -> super::Error {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => super::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => super::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => super::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput => super::ErrorKind::InvalidInput,
            io::ErrorKind::UnexpectedEof => super::ErrorKind::UnexpectedEof,
            io::ErrorKind::BrokenPipe => super::ErrorKind::BrokenPipe,
            io::ErrorKind::Interrupted => super::ErrorKind::Interrupted,
            _ => super::ErrorKind::Other,
        };
        super::Error { kind, message: "I/O error" }
    }
}

#[cfg(target_os = "none")]
mod sys {
    use crate::Write;

    const STDIN_FD: i32 = 0;
    const STDOUT_FD: i32 = 1;
    const STDERR_FD: i32 = 2;

    #[inline(always)]
    fn sys_read(fd: i32, buf: usize, len: usize) -> isize {
        let ret: isize;
        unsafe {
            core::arch::asm!(
                "li a7, 5",
                "ecall",
                in("a0") fd as usize,
                in("a1") buf,
                in("a2") len,
                lateout("a0") ret,
            );
        }
        ret
    }

    #[inline(always)]
    fn sys_write(fd: i32, buf: usize, len: usize) -> isize {
        let ret: isize;
        unsafe {
            core::arch::asm!(
                "li a7, 16",
                "ecall",
                in("a0") fd as usize,
                in("a1") buf,
                in("a2") len,
                lateout("a0") ret,
            );
        }
        ret
    }

    #[inline(always)]
    fn sys_open(path: usize, len: usize, flags: usize) -> isize {
        let ret: isize;
        unsafe {
            core::arch::asm!(
                "li a7, 15",
                "ecall",
                in("a0") path,
                in("a1") len,
                in("a2") flags,
                lateout("a0") ret,
            );
        }
        ret
    }

    #[inline(always)]
    fn sys_close(fd: i32) {
        let _: isize;
        unsafe {
            core::arch::asm!(
                "li a7, 21",
                "ecall",
                in("a0") fd as usize,
                lateout("a0") _,
            );
        }
    }

    #[inline(always)]
    fn sys_exit(code: i32) -> ! {
        unsafe {
            core::arch::asm!(
                "li a7, 10",
                "ecall",
                in("a0") code,
                options(noreturn)
            );
        }
    }

    fn copy_cstr(ptr: *const u8) -> alloc::vec::Vec<u8> {
        let mut v = alloc::vec::Vec::new();
        unsafe {
            let mut p = ptr;
            loop {
                let b = core::ptr::read(p);
                if b == 0 { break; }
                v.push(b);
                p = p.offset(1);
            }
        }
        v
    }

    fn read_args() -> (usize, *const *const u8) {
        let argc: usize;
        let argv: *const *const u8;
        unsafe {
            core::arch::asm!(
                "mv {0}, a0",
                "mv {1}, a1",
                out(reg) argc,
                out(reg) argv,
            );
        }
        (argc, argv)
    }

    pub fn read_stdin(buf: &mut [u8]) -> super::Result<usize> {
        let n = sys_read(STDIN_FD, buf.as_mut_ptr() as usize, buf.len());
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "stdin read error"));
        }
        Ok(n as usize)
    }

    pub fn write_stdout(buf: &[u8]) -> super::Result<usize> {
        let n = sys_write(STDOUT_FD, buf.as_ptr() as usize, buf.len());
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "stdout write error"));
        }
        Ok(n as usize)
    }

    pub fn flush_stdout() -> super::Result<()> { Ok(()) }

    pub fn write_stderr(buf: &[u8]) -> super::Result<usize> {
        let n = sys_write(STDERR_FD, buf.as_ptr() as usize, buf.len());
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "stderr write error"));
        }
        Ok(n as usize)
    }

    pub fn flush_stderr() -> super::Result<()> { Ok(()) }

    pub fn open(path: &str) -> super::Result<super::File> {
        let path_bytes = path.as_bytes();
        let fd = sys_open(path_bytes.as_ptr() as usize, path_bytes.len(), 0);
        if fd < 0 {
            return Err(super::Error::new(super::ErrorKind::NotFound, "open failed"));
        }
        Ok(super::File { fd: fd as i32 })
    }

    pub fn read_fd(fd: i32, buf: &mut [u8]) -> super::Result<usize> {
        let n = sys_read(fd, buf.as_mut_ptr() as usize, buf.len());
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "read error"));
        }
        Ok(n as usize)
    }

    pub fn write_fd(fd: i32, buf: &[u8]) -> super::Result<usize> {
        let n = sys_write(fd, buf.as_ptr() as usize, buf.len());
        if n < 0 {
            return Err(super::Error::new(super::ErrorKind::Other, "write error"));
        }
        Ok(n as usize)
    }

    pub fn close_fd(fd: i32) -> super::Result<()> {
        sys_close(fd);
        Ok(())
    }

    pub fn args() -> alloc::vec::Vec<alloc::string::String> {
        let (argc, argv) = read_args();
        let mut result = alloc::vec::Vec::new();
        for i in 1..argc {
            unsafe {
                let ptr = *argv.add(i);
                let bytes = copy_cstr(ptr);
                let s = alloc::string::String::from_utf8(bytes)
                    .unwrap_or_else(|_| alloc::string::String::from("?"));
                result.push(s);
            }
        }
        result
    }

    pub fn exit(code: i32) -> ! {
        sys_exit(code)
    }

    pub fn print(s: &str) {
        let mut out = super::Stdout;
        out.write_all(s.as_bytes()).ok();
    }

    pub fn println(s: &str) {
        print(s);
        let mut out = super::Stdout;
        out.write_all(b"\n").ok();
    }
}

#[cfg(windows)]
mod sys {
    use std::io::{self, Read as IoRead, Write as IoWrite};

    pub fn read_stdin(buf: &mut [u8]) -> super::Result<usize> {
        io::stdin().read(buf).map_err(from_std_error)
    }
    pub fn write_stdout(buf: &[u8]) -> super::Result<usize> {
        io::stdout().write(buf).map_err(from_std_error)
    }
    pub fn flush_stdout() -> super::Result<()> {
        io::stdout().flush().map_err(from_std_error)
    }
    pub fn write_stderr(buf: &[u8]) -> super::Result<usize> {
        io::stderr().write(buf).map_err(from_std_error)
    }
    pub fn flush_stderr() -> super::Result<()> {
        io::stderr().flush().map_err(from_std_error)
    }
    pub fn open(_path: &str) -> super::Result<super::File> {
        Err(super::Error::new(super::ErrorKind::Other, "windows not supported"))
    }
    pub fn read_fd(_fd: i32, _buf: &mut [u8]) -> super::Result<usize> {
        Err(super::Error::new(super::ErrorKind::Other, "windows not supported"))
    }
    pub fn write_fd(_fd: i32, _buf: &[u8]) -> super::Result<usize> {
        Err(super::Error::new(super::ErrorKind::Other, "windows not supported"))
    }
    pub fn close_fd(_fd: i32) -> super::Result<()> {
        Err(super::Error::new(super::ErrorKind::Other, "windows not supported"))
    }
    pub fn args() -> Vec<String> {
        std::env::args().collect()
    }
    pub fn exit(code: i32) -> ! {
        std::process::exit(code)
    }
    pub fn print(s: &str) { print!("{}", s); }
    pub fn println(s: &str) { println!("{}", s); }
    fn from_std_error(e: io::Error) -> super::Error {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => super::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => super::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => super::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput => super::ErrorKind::InvalidInput,
            io::ErrorKind::UnexpectedEof => super::ErrorKind::UnexpectedEof,
            io::ErrorKind::BrokenPipe => super::ErrorKind::BrokenPipe,
            io::ErrorKind::Interrupted => super::ErrorKind::Interrupted,
            _ => super::ErrorKind::Other,
        };
        super::Error { kind, message: "I/O error" }
    }
}

fn platform_read_stdin(buf: &mut [u8]) -> Result<usize> { sys::read_stdin(buf) }
fn platform_write_stdout(buf: &[u8]) -> Result<usize> { sys::write_stdout(buf) }
fn platform_flush_stdout() -> Result<()> { sys::flush_stdout() }
fn platform_write_stderr(buf: &[u8]) -> Result<usize> { sys::write_stderr(buf) }
fn platform_flush_stderr() -> Result<()> { sys::flush_stderr() }
fn platform_open(path: &str) -> Result<File> { sys::open(path) }
fn platform_read_fd(fd: i32, buf: &mut [u8]) -> Result<usize> { sys::read_fd(fd, buf) }
fn platform_write_fd(fd: i32, buf: &[u8]) -> Result<usize> { sys::write_fd(fd, buf) }
fn platform_close_fd(fd: i32) -> Result<()> { sys::close_fd(fd) }
pub fn args() -> Vec<String> { sys::args() }
pub fn exit(code: i32) -> ! { sys::exit(code) }
pub fn print(s: &str) { sys::print(s) }
pub fn println(s: &str) { sys::println(s) }