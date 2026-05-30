#![no_std]
#![no_main]

use user::*;

fn test_getpid() {
    let pid = getpid();
    assert!(pid > 0, "getpid should return positive value");
}

fn test_getppid() {
    let ppid = getppid().expect("getppid");
    assert!(ppid > 0, "getppid should return positive (init pid)");
}

fn test_getpgid_self() {
    let pid = getpid();
    let pgid = getpgid(0).expect("getpgid(0)");
    assert_eq!(pgid, pid, "getpgid(0) should return own pid");
}

fn test_setsid() {
    match fork() {
        Ok(0) => {
            // Child
            let old_pgid = getpgid(0).expect("child getpgid before setsid");

            // Create a new session
            let new_sid = setsid().expect("setsid");
            let new_pgid = getpgid(0).expect("child getpgid after setsid");

            assert_eq!(new_sid, new_pgid, "sid should equal pgid after setsid");
            assert_ne!(new_pgid, old_pgid, "pgid should change after setsid");

            // Second setsid should fail (already session leader)
            assert!(setsid().is_err(), "second setsid should fail");

            poweroff(0);
        }
        Ok(_pid) => {
            let mut code = 0;
            wait(&mut code).expect("wait");
            assert_eq!(code, 0, "child should exit with 0");
        }
        Err(_) => panic!("fork failed"),
    }
}

fn test_nice() {
    let old = nice(0).expect("nice(0)");
    assert_eq!(old, 0, "initial nice should be 0");

    let new = nice(5).expect("nice(5)");
    assert_eq!(new, old + 5, "nice should increase by 5");

    let clamped = nice(-100).expect("nice(-100)");
    assert_eq!(clamped, -20, "nice should clamp to -20");

    let clamped2 = nice(100).expect("nice(100)");
    assert_eq!(clamped2, 19, "nice should clamp to 19");
}

fn test_fork_pgid_inherit() {
    match fork() {
        Ok(0) => {
            // Child should inherit parent's pgid
            let pgid = getpgid(0).expect("child getpgid");
            let ppid = getppid().expect("child getppid");
            assert_eq!(pgid, ppid, "child should be in parent's process group");
            poweroff(0);
        }
        Ok(_pid) => {
            let mut code = 0;
            wait(&mut code).expect("wait");
            assert_eq!(code, 0);
        }
        Err(_) => panic!("fork failed"),
    }
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("posix_proc tests:\n");

    test_getpid();
    println!("  test_getpid ... ok");

    test_getppid();
    println!("  test_getppid ... ok");

    test_getpgid_self();
    println!("  test_getpgid_self ... ok");

    test_setsid();
    println!("  test_setsid ... ok");

    test_nice();
    println!("  test_nice ... ok");

    test_fork_pgid_inherit();
    println!("  test_fork_pgid_inherit ... ok");

    println!("\nall proc management tests passed");
    poweroff(0);
}
