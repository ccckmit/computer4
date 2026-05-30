#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 512;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Mount /proc using source path only (kernel uses arg0 for mount point)
    print!("mount /proc ... ");
    let ret = raw::mount(b"/proc\0".as_ptr(), b"\0".as_ptr(), b"\0".as_ptr(), 0);
    if ret == 0 {
        println!("ok");
        passed += 1;
    } else {
        println!("FAILED (ret={})", ret);
        failed += 1;
    }

    // Test 2: Mount /dev
    print!("mount /dev ... ");
    let ret = raw::mount(b"/dev\0".as_ptr(), b"\0".as_ptr(), b"\0".as_ptr(), 0);
    if ret == 0 {
        println!("ok");
        passed += 1;
    } else {
        println!("FAILED (ret={})", ret);
        failed += 1;
    }

    // Test 3: Open and read /proc/self/status via VFS
    print!("read /proc/self/status ... ");
    match open("/proc/self/status", 0) {
        Ok(fd) => {
            let mut buf = [0u8; BUF_SIZE];
            match read(fd, &mut buf) {
                Ok(n) => {
                    let s = core::str::from_utf8(&buf[..n]).unwrap_or("");
                    if s.contains("procfs") && s.contains("Pid:") {
                        println!("ok ({} bytes)", n);
                        passed += 1;
                    } else {
                        println!("FAILED (unexpected content)");
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("FAILED (read error: {:?})", e);
                    failed += 1;
                }
            }
            let _ = close(fd);
        }
        Err(e) => {
            println!("FAILED (open error: {:?})", e);
            failed += 1;
        }
    }

    // Test 4: Open /dev via VFS
    print!("open /dev ... ");
    match open("/dev", 0) {
        Ok(fd) => {
            println!("ok (fd={})", fd.as_raw());
            passed += 1;
            let _ = close(fd);
        }
        Err(e) => {
            println!("FAILED (open error: {:?})", e);
            failed += 1;
        }
    }

    // Test 5: Umount /proc
    print!("umount /proc ... ");
    let ret = raw::umount(b"/proc\0".as_ptr());
    if ret == 0 {
        println!("ok");
        passed += 1;
    } else {
        println!("FAILED (ret={})", ret);
        failed += 1;
    }

    // Test 6: Umount /dev
    print!("umount /dev ... ");
    let ret = raw::umount(b"/dev\0".as_ptr());
    if ret == 0 {
        println!("ok");
        passed += 1;
    } else {
        println!("FAILED (ret={})", ret);
        failed += 1;
    }

    println!(
        "\n{} tests: {} passed, {} failed\n",
        env!("CARGO_PKG_NAME"),
        passed,
        failed
    );

    if failed > 0 {
        exit(1);
    }
}