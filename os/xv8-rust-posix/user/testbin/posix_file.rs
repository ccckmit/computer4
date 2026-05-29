#![no_std]
#![no_main]

use user::*;

/// Open for reading+writing, create + truncate.
const O_CREATE_RW: usize = OpenFlag::CREATE | OpenFlag::READ_WRITE | OpenFlag::TRUNCATE;

/// lseek SEEK_SET / SEEK_CUR / SEEK_END on a regular file.
fn test_lseek() {
    let fd = open("/pf_lseek", O_CREATE_RW).expect("lseek: create");
    write(fd, b"hello, world!").expect("lseek: write");
    close(fd).expect("lseek: close");

    let fd = open("/pf_lseek", OpenFlag::READ_ONLY).expect("lseek: open");

    // SEEK_END → size = 13
    let pos = lseek(fd, 0, 2).expect("lseek: SEEK_END");
    assert_eq!(pos, 13, "lseek SEEK_END");
    assert_eq!(read(fd, &mut [0u8; 1]).expect("lseek: read at end"), 0, "read at end");

    // SEEK_SET → 0
    let pos = lseek(fd, 0, 0).expect("lseek: SEEK_SET");
    assert_eq!(pos, 0, "lseek SEEK_SET");

    // SEEK_CUR → 5
    let pos = lseek(fd, 5, 1).expect("lseek: SEEK_CUR +5");
    assert_eq!(pos, 5, "lseek SEEK_CUR");

    // read remainder
    let mut buf = [0u8; 8];
    let n = read(fd, &mut buf).expect("lseek: read after cur");
    assert_eq!(n, 8, "read after seek");
    assert_eq!(&buf, b", world!", "data after seek");

    close(fd).expect("lseek: close");
    unlink("/pf_lseek").expect("lseek: unlink");
}

/// lseek on a pipe must fail with ESPIPE.
fn test_lseek_pipe() {
    let (r, _w) = pipe().expect("lseek_pipe: pipe");
    assert_eq!(
        lseek(r, 0, 0),
        Err(Errno::ESPIPE),
        "lseek on pipe must fail"
    );
}

/// getdents reads directory entries.
fn test_getdents() {
    mkdir("/pf_dir").expect("getdents: mkdir");
    let fd = open("/pf_dir", OpenFlag::READ_ONLY).expect("getdents: open dir");

    let mut buf = [0u8; 64];
    let total = getdents(fd, &mut buf).expect("getdents");
    assert!(total >= 32, "getdents: should read at least 2 entries"); // . and ..
    let mut found_dot = false;
    let mut found_dotdot = false;
    let entsize = 16;
    for off in (0..total).step_by(entsize) {
        let _inum = u16::from_ne_bytes([buf[off], buf[off + 1]]);
        let _name_end = buf[off + 2..off + 16].iter().position(|&c| c == 0).unwrap_or(14);
        let name = core::str::from_utf8(&buf[off + 2..off + 2 + _name_end]).unwrap_or("");
        if name == "." { found_dot = true; }
        if name == ".." { found_dotdot = true; }
    }
    assert!(found_dot, "getdents: missing '.'");
    assert!(found_dotdot, "getdents: missing '..'");

    close(fd).expect("getdents: close");
    unlink("/pf_dir").expect("getdents: unlink");
}

/// symlink + readlink roundtrip
fn test_symlink_readlink() {
    let fd = open("/pf_target", O_CREATE_RW).expect("symlink: create target");
    write(fd, b"target data").expect("symlink: write target");
    close(fd).expect("symlink: close target");

    symlink("/pf_target", "/pf_link").expect("symlink");

    let mut buf = [0u8; 32];
    let n = readlink("/pf_link", &mut buf).expect("readlink");
    assert_eq!(n, 10, "readlink len");
    assert_eq!(&buf[..n], b"/pf_target", "readlink content");

    unlink("/pf_link").expect("symlink: unlink link");
    unlink("/pf_target").expect("symlink: unlink target");
}

/// access checks file existence
fn test_access() {
    let fd = open("/pf_access", O_CREATE_RW).expect("access: create");
    close(fd).expect("access: close");

    access("/pf_access", 0).expect("access: F_OK should work");
    assert_eq!(access("/pf_nonexistent", 0), Err(Errno::ENOENT), "access: ENOENT");

    unlink("/pf_access").expect("access: unlink");
}

/// fcntl F_DUPFD duplicates a file descriptor
fn test_fcntl_dupfd() {
    let fd = open("/pf_dupfd", O_CREATE_RW).expect("dupfd: create");
    write(fd, b"dupfd test").expect("dupfd: write");
    close(fd).expect("dupfd: close");

    let fd = open("/pf_dupfd", OpenFlag::READ_ONLY).expect("dupfd: reopen");

    // F_DUPFD with arg=0 should allocate the lowest fd (might be 3-15 range)
    let newfd = fcntl(fd, F_DUPFD, 0).expect("dupfd: F_DUPFD");
    assert!(newfd > 0, "dupfd: new fd should be positive");

    // both should share the same offset
    let mut buf = [0u8; 5];
    let n = read(fd, &mut buf).expect("dupfd: read original");
    assert_eq!(&buf[..n], b"dupfd", "original read");

    let n = read(Fd::from_raw(newfd), &mut buf).expect("dupfd: read dup");
    assert_eq!(&buf[..n], b" test", "dup read (offset shared)");

    close(fd).expect("dupfd: close orig");
    close(Fd::from_raw(newfd)).expect("dupfd: close dup");

    unlink("/pf_dupfd").expect("dupfd: unlink");
}

/// dup2 duplicates a file descriptor to a specific target number
fn test_dup2() {
    let fd = open("/pf_dup2", O_CREATE_RW).expect("dup2: create");
    write(fd, b"dup2 data").expect("dup2: write");
    close(fd).expect("dup2: close");

    let fd = open("/pf_dup2", OpenFlag::READ_ONLY).expect("dup2: reopen");

    let newfd = dup2(fd, 7).expect("dup2");
    assert_eq!(newfd, 7, "dup2: should return newfd");

    let mut buf = [0u8; 9];
    let n = read(fd, &mut buf).expect("dup2: read original");
    assert_eq!(&buf[..n], b"dup2 data", "dup2 original data");

    // reading from dup'd fd should show same content (shared offset already advanced)
    // re-seek to 0 on original
    lseek(fd, 0, 0).expect("dup2: seek orig");
    let n = read(Fd::from_raw(7), &mut buf).expect("dup2: read dup");
    assert_eq!(&buf[..n], b"dup2 data", "dup2 dup data");

    close(fd).expect("dup2: close orig");
    close(Fd::from_raw(7)).expect("dup2: close dup");

    unlink("/pf_dup2").expect("dup2: unlink");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_lseek();
    test_lseek_pipe();
    test_getdents();
    test_symlink_readlink();
    test_access();
    test_fcntl_dupfd();
    test_dup2();
    println!("_posix_file: all 7 tests passed");
}
