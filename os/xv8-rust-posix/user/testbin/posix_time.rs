#![no_std]
#![no_main]

use user::*;

fn test_clock_gettime_monotonic() {
    let mut ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    clock_gettime(CLOCK_MONOTONIC, &mut ts).expect("clock_gettime MONOTONIC");
    assert!(ts.tv_sec > 0 || ts.tv_nsec > 0, "time should be positive");
}

fn test_clock_gettime_realtime() {
    let mut ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    clock_gettime(CLOCK_REALTIME, &mut ts).expect("clock_gettime REALTIME");
    assert!(ts.tv_sec > 0 || ts.tv_nsec > 0, "time should be positive");
}

fn test_clock_gettime_invalid() {
    let mut ts = Timespec { tv_sec: 0, tv_nsec: 0 };
    let ret = clock_gettime(99, &mut ts);
    assert!(ret.is_err(), "invalid clock should return error");
    assert_eq!(ret.unwrap_err(), Errno::EINVAL);
}

fn test_nanosleep_basic() {
    let mut ts = Timespec { tv_sec: 0, tv_nsec: 100_000_000 }; // 100ms
    let mut rem = Timespec { tv_sec: 0, tv_nsec: 0 };
    nanosleep(&ts, &mut rem).expect("nanosleep");
}

fn test_nanosleep_negative_rejected() {
    let ts = Timespec { tv_sec: -1, tv_nsec: 0 };
    let mut rem = Timespec { tv_sec: 0, tv_nsec: 0 };
    let ret = nanosleep(&ts, &mut rem);
    assert!(ret.is_err(), "negative tv_sec should be rejected");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("posix_time tests:\n");

    test_clock_gettime_monotonic();
    println!("  test_clock_gettime_monotonic ... ok");

    test_clock_gettime_realtime();
    println!("  test_clock_gettime_realtime ... ok");

    test_clock_gettime_invalid();
    println!("  test_clock_gettime_invalid ... ok");

    test_nanosleep_basic();
    println!("  test_nanosleep_basic ... ok");

    test_nanosleep_negative_rejected();
    println!("  test_nanosleep_negative_rejected ... ok");

    println!("\nall posix_time tests passed");
}
