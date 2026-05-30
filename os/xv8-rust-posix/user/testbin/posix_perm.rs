#![no_std]
#![no_main]

use user::*;
use user::raw;

fn test_getuid_gid() {
    let uid = getuid();
    let gid = getgid();
    assert_eq!(uid, 0, "default uid should be root (0)");
    assert_eq!(gid, 0, "default gid should be root (0)");
}

fn test_setuid_getuid() {
    setuid(42).expect("setuid");
    assert_eq!(getuid(), 42);
    setuid(0).expect("setuid back to root");
    assert_eq!(getuid(), 0);
}

fn test_setgid_getgid() {
    setgid(100).expect("setgid");
    assert_eq!(getgid(), 100);
    setgid(0).expect("setgid back to root");
    assert_eq!(getgid(), 0);
}

fn test_umask_default() {
    let old = umask(0o027);
    assert_eq!(old, 0o022, "default umask should be 0o022");
    let restored = umask(old);
    assert_eq!(restored, 0o027);
}

fn cstr(s: &str) -> *const u8 {
    s.as_ptr() as *const u8
}

fn fd_byname(name: &str) -> usize {
    let r = raw::open(name.as_ptr(), OpenFlag::READ_ONLY as usize);
    assert!(r >= 0, "open for stat");
    r as usize
}

fn test_chmod_chown() {
    let path = cstr("/test_perm\0");
    let fd = raw::open(path, OpenFlag::CREATE as usize | OpenFlag::READ_WRITE as usize);
    assert!(fd >= 0, "open for chmod test failed");
    raw::close(fd as usize);

    let mut st = Stat::default();
    let r = raw::fstat(fd_byname("/test_perm\0"), &mut st);
    assert!(r >= 0, "fstat");
    assert!(st.mode & 0o777 > 0, "file should have permissions");

    // Use raw wrappers for paths with explicit null
    let r2 = raw::chmod(cstr("/test_perm\0") as usize, 0o700);
    assert!(r2 >= 0, "chmod");
    let mut st2 = Stat::default();
    let r = raw::fstat(fd_byname("/test_perm\0"), &mut st2);
    assert!(r >= 0, "fstat after chmod");
    assert_eq!(st2.mode & 0o777, 0o700, "chmod should set 0700");

    let r3 = raw::chown(cstr("/test_perm\0") as usize, 42, 100);
    assert!(r3 >= 0, "chown");
    let mut st3 = Stat::default();
    let r = raw::fstat(fd_byname("/test_perm\0"), &mut st3);
    assert!(r >= 0, "fstat after chown");
    assert_eq!(st3.uid, 42, "chown should set uid to 42");
    assert_eq!(st3.gid, 100, "chown should set gid to 100");

    raw::unlink(cstr("/test_perm\0"));
}

fn test_new_file_inherits_uid_gid() {
    setuid(99).expect("setuid for test");
    setgid(99).expect("setgid for test");

    // Verify process uid/gid were actually set
    assert_eq!(getuid(), 99, "process uid should be 99");
    assert_eq!(getgid(), 99, "process gid should be 99");

    // Also check umask isn't interfering
    let saved = umask(0o000);

    let path = cstr("/test_inherit\0");
    let fd = raw::open(path, OpenFlag::CREATE as usize | OpenFlag::READ_WRITE as usize);
    assert!(fd >= 0, "open for inherit test failed");
    raw::close(fd as usize);

    let mut st = Stat::default();
    let r = raw::fstat(fd_byname("/test_inherit\0"), &mut st);
    assert!(r >= 0, "fstat");
    assert_eq!(st.uid, 99, "new file should inherit uid");
    assert_eq!(st.gid, 99, "new file should inherit gid");

    raw::unlink(cstr("/test_inherit\0"));
    setuid(0).expect("setuid back");
    setgid(0).expect("setgid back");
    umask(saved);
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("posix_perm tests:\n");

    test_getuid_gid();
    println!("  test_getuid_gid ... ok");

    test_setuid_getuid();
    println!("  test_setuid_getuid ... ok");

    test_setgid_getgid();
    println!("  test_setgid_getgid ... ok");

    test_umask_default();
    println!("  test_umask_default ... ok");

    test_chmod_chown();
    println!("  test_chmod_chown ... ok");

    test_new_file_inherits_uid_gid();
    println!("  test_new_file_inherits_uid_gid ... ok");

    println!("\nall posix_perm tests passed");
}
