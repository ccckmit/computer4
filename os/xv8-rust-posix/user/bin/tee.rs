#![no_std]
#![no_main]

use user::*;

const MAX_FILES: usize = 16;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let mut file_count = 0;
    let mut file_names: [&str; MAX_FILES] = [""; MAX_FILES];

    for i in 1..=args.args_len() {
        if let Some(arg) = args.get_str(i) {
            if !arg.starts_with('-') && file_count < MAX_FILES {
                file_names[file_count] = arg;
                file_count += 1;
            }
        }
    }

    if file_count == 0 {
        let mut buf = [0u8; 4096];
        loop {
            match Fd::STDIN.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    Stdout.write_all(&buf[..n]).ok();
                }
                Err(_) => break,
            }
        }
        return;
    }

    let mut fds: [Option<Fd>; MAX_FILES] = [None; MAX_FILES];
    for i in 0..file_count {
        match open(file_names[i], OpenFlag::CREATE | OpenFlag::WRITE_ONLY | OpenFlag::TRUNCATE) {
            Ok(fd) => fds[i] = Some(fd),
            Err(e) => {
                eprintln!("tee: {}: {:?}", file_names[i], e);
                exit(1);
            }
        }
    }

    let mut buf = [0u8; 4096];
    loop {
        match Fd::STDIN.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                Stdout.write_all(&buf[..n]).ok();
                for i in 0..file_count {
                    if let Some(fd) = fds[i] {
                        write(fd, &buf[..n]).ok();
                    }
                }
            }
            Err(_) => break,
        }
    }

    for i in 0..file_count {
        if let Some(fd) = fds[i] {
            let _ = close(fd);
        }
    }
}