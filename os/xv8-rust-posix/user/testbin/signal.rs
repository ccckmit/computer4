#![no_std]
#![no_main]

use user::*;

/// Test SIGKILL terminates a process.
fn test_kill_sigkill() {
    let pid = fork().expect("fork");
    if pid == 0 {
        // Child: loop forever
        loop {
            sleep(1).ok();
        }
    }

    // Parent: kill child with SIGKILL
    kill(pid, 9).expect("kill SIGKILL");

    let mut code = 0;
    let reaped = wait(&mut code).expect("wait after kill");
    assert_eq!(reaped, pid, "reaped wrong pid");
    assert_eq!(code as isize, -1, "killed child must exit with -1");
}

/// Test SIG_IGN: process should survive signal.
fn test_sig_ignore() {
    let pid = fork().expect("fork");
    if pid == 0 {
        // Child: ignore SIGTERM
        let act = SigAction {
            handler: SIG_IGN,
            flags: 0,
            mask: 0,
        };
        sigaction(15, Some(&act), None).expect("sigaction SIG_IGN");

        // Signal ourselves
        kill(getpid(), 15).expect("kill self SIGTERM");

        // Should still be alive
        println!("SIGTERM ignored, alive");
        exit(0);
    }

    let mut code = 0;
    let reaped = wait(&mut code).expect("wait");
    assert_eq!(reaped, pid, "reaped wrong pid");
    assert_eq!(code, 0, "child should exit 0 after ignoring SIGTERM");
}

/// Test SIGALRM with custom handler.
fn test_sigalrm_custom() {
    static mut HANDLER_CALLED: bool = false;

    extern "C" fn alarm_handler(_sig: usize) {
        unsafe {
            HANDLER_CALLED = true;
        }
    }

    let pid = fork().expect("fork");
    if pid == 0 {
        let act = SigAction {
            handler: alarm_handler as *const () as usize,
            flags: 0,
            mask: 0,
        };
        sigaction(14, Some(&act), None).expect("sigaction SIGALRM");

        // Send SIGALRM to self
        kill(getpid(), 14).expect("kill self SIGALRM");

        // The handler was called, which set HANDLER_CALLED
        unsafe {
            assert!(HANDLER_CALLED, "SIGALRM handler was not called");
        }
        exit(0);
    }

    let mut code = 0;
    let reaped = wait(&mut code).expect("wait");
    assert_eq!(reaped, pid, "reaped wrong pid");
    assert_eq!(code, 0, "child with SIGALRM handler should exit 0");
}

/// Test sigprocmask blocks signal delivery.
fn test_sig_block() {
    let pid = fork().expect("fork");
    if pid == 0 {
        // Block SIGTERM
        let set: u32 = 1 << (15 - 1); // SIGTERM bit
        sigprocmask(SIG_BLOCK as usize, Some(&set), None).expect("sigprocmask BLOCK");

        // Send SIGTERM to self — should be pending, not delivered
        kill(getpid(), 15).expect("kill self SIGTERM");

        // Check it's pending
        let pending = sigpending().expect("sigpending");
        assert!(
            (pending & set) != 0,
            "SIGTERM should be pending after block"
        );

        // Unblock — should now be delivered and kill us
        sigprocmask(SIG_UNBLOCK as usize, Some(&set), None).ok();

        // If we reach here, signal was not delivered
        exit(1);
    }

    let mut code = 0;
    let reaped = wait(&mut code).expect("wait");
    assert_eq!(reaped, pid, "reaped wrong pid");
    // After unblock, SIGTERM default terminates, so exit code should be -1
    assert_eq!(code as isize, -1, "child should be killed by unblocked SIGTERM");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_kill_sigkill();
    test_sig_ignore();
    test_sigalrm_custom();
    test_sig_block();
    println!("_signal: all 4 tests passed");
}
