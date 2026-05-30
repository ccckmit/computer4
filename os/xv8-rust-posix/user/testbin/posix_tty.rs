#![no_std]
#![no_main]

use user::*;

fn test_tcgetattr() {
    let mut t = Termios {
        c_iflag: 0xFF,
        c_oflag: 0xFF,
        c_cflag: 0xFF,
        c_lflag: 0xFF,
        c_cc: [0; NCCS],
    };
    tcgetattr(Fd::STDIN, &mut t).expect("tcgetattr on stdin");
    assert!(t.c_iflag == 0, "tcgetattr should return kernel defaults");
    assert!(t.c_oflag == 0, "tcgetattr should return kernel defaults");
    assert!(t.c_cflag == 0, "tcgetattr should return kernel defaults");
    assert!(t.c_lflag == 0, "tcgetattr should return kernel defaults");
}

fn test_tcsetattr_tcgetattr_roundtrip() {
    let mut orig = Termios {
        c_iflag: 0xFF,
        c_oflag: 0xFF,
        c_cflag: 0xFF,
        c_lflag: 0xFF,
        c_cc: [0; NCCS],
    };
    tcgetattr(Fd::STDIN, &mut orig).expect("tcgetattr");

    let mut t = orig;
    t.c_iflag = 42;
    t.c_oflag = 43;
    t.c_cflag = 44;
    t.c_lflag = 45;
    tcsetattr(Fd::STDOUT, &t).expect("tcsetattr on stdout");

    let mut t2 = Termios {
        c_iflag: 0xFF,
        c_oflag: 0xFF,
        c_cflag: 0xFF,
        c_lflag: 0xFF,
        c_cc: [0; NCCS],
    };
    tcgetattr(Fd::STDIN, &mut t2).expect("tcgetattr after tcsetattr");
    assert_eq!(t2.c_iflag, 42, "c_iflag should persist");
    assert_eq!(t2.c_oflag, 43, "c_oflag should persist");
    assert_eq!(t2.c_cflag, 44, "c_cflag should persist");
    assert_eq!(t2.c_lflag, 45, "c_lflag should persist");

    tcsetattr(Fd::STDOUT, &orig).expect("tcsetattr restore");
}

fn test_tcgetattr_bad_fd() {
    let mut t = Termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_cc: [0; NCCS],
    };
    let r = tcgetattr(Fd::from_raw(99), &mut t);
    assert!(r.is_err(), "tcgetattr on bad fd should fail");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("posix_tty tests:\n");

    test_tcgetattr();
    println!("  test_tcgetattr ... ok");

    test_tcsetattr_tcgetattr_roundtrip();
    println!("  test_tcsetattr_tcgetattr_roundtrip ... ok");

    test_tcgetattr_bad_fd();
    println!("  test_tcgetattr_bad_fd ... ok");

    println!("\nall posix_tty tests passed");
}
