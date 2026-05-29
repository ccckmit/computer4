#![no_std]
#![no_main]

use user::*;
use user::raw;

fn test_errno_values() {
    assert_eq!(Errno::EPERM as u16, 1);
    assert_eq!(Errno::ENOENT as u16, 2);
    assert_eq!(Errno::ESRCH as u16, 3);
    assert_eq!(Errno::EINTR as u16, 4);
    assert_eq!(Errno::EIO as u16, 5);
    assert_eq!(Errno::ENOEXEC as u16, 8);
    assert_eq!(Errno::EBADF as u16, 9);
    assert_eq!(Errno::ECHILD as u16, 10);
    assert_eq!(Errno::EAGAIN as u16, 11);
    assert_eq!(Errno::ENOMEM as u16, 12);
    assert_eq!(Errno::EFAULT as u16, 14);
    assert_eq!(Errno::EEXIST as u16, 17);
    assert_eq!(Errno::EXDEV as u16, 18);
    assert_eq!(Errno::ENOTDIR as u16, 20);
    assert_eq!(Errno::EISDIR as u16, 21);
    assert_eq!(Errno::EINVAL as u16, 22);
    assert_eq!(Errno::ENFILE as u16, 23);
    assert_eq!(Errno::EMFILE as u16, 24);
    assert_eq!(Errno::ENOSPC as u16, 28);
    assert_eq!(Errno::EMLINK as u16, 31);
    assert_eq!(Errno::EPIPE as u16, 32);
    assert_eq!(Errno::ENAMETOOLONG as u16, 36);
    assert_eq!(Errno::ENOSYS as u16, 38);
    assert_eq!(Errno::ENOTEMPTY as u16, 39);
    assert_eq!(Errno::EMSGSIZE as u16, 90);
}

fn test_roundtrip_enoent() {
    let ret = raw::open("/nonexistent_file_for_test\0".as_ptr(), 0);
    assert!(ret < 0, "open nonexistent must return negative");
    assert_eq!(Errno::from((-ret) as u16), Errno::ENOENT);
}

fn test_roundtrip_ebadf() {
    let ret = raw::read(99, core::ptr::null_mut(), 0);
    assert!(ret < 0, "read bad fd must return negative");
    assert_eq!(Errno::from((-ret) as u16), Errno::EBADF);
}

fn test_invalid_syscall_returns_enosys() {
    use core::arch::asm;
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") 99,
            lateout("a0") ret,
        );
    }
    assert!(ret < 0, "invalid syscall must return negative");
    assert_eq!(Errno::from((-ret) as u16), Errno::ENOSYS);
}

fn test_ok_return_is_positive() {
    let pid = raw::getpid();
    assert!(pid > 0, "getpid must return positive");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_errno_values();
    test_roundtrip_enoent();
    test_roundtrip_ebadf();
    test_invalid_syscall_returns_enosys();
    test_ok_return_is_positive();
    println!("_posix_baseline: all 5 tests passed");
}
