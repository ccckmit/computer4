#![no_std]
#![no_main]

use user::*;
use user::{mmap, munmap, mprotect, PROT_READ, PROT_WRITE, PROT_NONE, MAP_ANONYMOUS, MAP_PRIVATE};

/// Basic anonymous mmap: allocate, write, read.
fn test_mmap_anon_basic() {
    let len = 4096;
    let addr = mmap(core::ptr::null(), len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0)
        .expect("mmap anon basic");
    assert!(!addr.is_null(), "mmap returned null");

    // Write to the mapped region
    unsafe {
        core::ptr::write_volatile(addr as *mut u8, 0xAB);
        core::ptr::write_volatile(addr.add(1), 0xCD);
        core::ptr::write_volatile(addr.add(4095), 0xEF);
    }

    // Read back
    assert_eq!(unsafe { core::ptr::read_volatile(addr as *const u8) }, 0xAB);
    assert_eq!(unsafe { core::ptr::read_volatile(addr.add(1)) }, 0xCD);
    assert_eq!(unsafe { core::ptr::read_volatile(addr.add(4095)) }, 0xEF);

    // Munmap
    munmap(addr, len).expect("munmap basic");
}

/// mmap with PROT_READ only; writing should cause a fault (kill the process).
fn test_mmap_readonly() {
    let len = 4096;
    let addr = mmap(core::ptr::null(), len, PROT_READ, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0)
        .expect("mmap readonly");
    assert!(!addr.is_null());

    // Read should work
    let val = unsafe { core::ptr::read_volatile(addr as *const u8) };
    assert_eq!(val, 0);

    // Mprotect to add write permission
    mprotect(addr, len, PROT_READ | PROT_WRITE).expect("mprotect add write");

    // Now write should work
    unsafe { core::ptr::write_volatile(addr as *mut u8, 0x42) };
    assert_eq!(unsafe { core::ptr::read_volatile(addr as *const u8) }, 0x42);

    munmap(addr, len).expect("munmap readonly");
}

/// mmap with PROT_NONE should fail (not supported for now)
fn test_mmap_protnone() {
    let len = 4096;
    let result = mmap(core::ptr::null(), len, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0);
    assert!(result.is_err(), "PROT_NONE should fail");
}

/// Multiple mmaps at different addresses
fn test_mmap_multiple() {
    let len = 4096;
    let a1 = mmap(core::ptr::null(), len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0)
        .expect("mmap multi 1");
    let a2 = mmap(core::ptr::null(), len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0)
        .expect("mmap multi 2");

    assert_ne!(a1, a2, "mmaps should return different addresses");

    // Write to both independently
    unsafe {
        core::ptr::write_volatile(a1 as *mut u8, 0x11);
        core::ptr::write_volatile(a2 as *mut u8, 0x22);
    }
    assert_eq!(unsafe { core::ptr::read_volatile(a1 as *const u8) }, 0x11);
    assert_eq!(unsafe { core::ptr::read_volatile(a2 as *const u8) }, 0x22);

    munmap(a1, len).expect("munmap multi 1");
    munmap(a2, len).expect("munmap multi 2");
}

/// Large mmap (multi-page)
fn test_mmap_large() {
    let len = 16384; // 4 pages
    let addr = mmap(core::ptr::null(), len, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, 0, 0)
        .expect("mmap large");
    assert!(!addr.is_null());

    // Touch every page
    for i in 0..4 {
        let p = unsafe { addr.add(i * 4096) };
        unsafe { core::ptr::write_volatile(p, i as u8) };
    }
    for i in 0..4 {
        let p = unsafe { addr.add(i * 4096) };
        assert_eq!(unsafe { core::ptr::read_volatile(p) }, i as u8);
    }

    munmap(addr, len).expect("munmap large");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("posix_mmap tests:\n");

    test_mmap_anon_basic();
    println!("  test_mmap_anon_basic ... ok");

    test_mmap_readonly();
    println!("  test_mmap_readonly ... ok");

    test_mmap_protnone();
    println!("  test_mmap_protnone ... ok");

    test_mmap_multiple();
    println!("  test_mmap_multiple ... ok");

    test_mmap_large();
    println!("  test_mmap_large ... ok");

    println!("\nall mmap tests passed");
}
