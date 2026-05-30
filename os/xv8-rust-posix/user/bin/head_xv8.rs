#![no_std]
#![no_main]

use user::*;

const DEFAULT_LINES: usize = 10;
const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let argc = args.args_len();

    let mut lines = DEFAULT_LINES;
    let mut start_file = 1;

    if argc > 0 && args.get_str(1).unwrap().starts_with('-') {
        let opt = args.get_str(1).unwrap();
        if opt.starts_with("-n") {
            let n_str = &opt[2..];
            if n_str.is_empty() {
                if argc > 2 {
                    lines = parse_num(args.get_str(2).unwrap());
                    start_file = 3;
                }
            } else {
                lines = parse_num(n_str);
                start_file = 2;
            }
        }
    }

    if start_file > argc {
        print_stdin(lines);
        return;
    }

    for i in start_file..=argc {
        let path = args.get_str(i).unwrap();
        let mut fd = match open(path, OpenFlag::READ_ONLY) {
            Ok(fd) => fd,
            Err(_) => {
                println!("head: cannot open '{}': No such file or directory", path);
                continue;
            }
        };

        let mut buf = [0u8; BUF_SIZE];
        let mut line_count = 0;

        loop {
            if line_count >= lines {
                break;
            }
            let n = match fd.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            for j in 0..n {
                if buf[j] == b'\n' {
                    line_count += 1;
                    if line_count >= lines {
                        print!("{}", buf[j + 1] as char);
                        break;
                    }
                }
                print!("{}", buf[j] as char);
            }
        }
        let _ = close(fd);
    }
}

fn print_stdin(lines: usize) {
    let mut buf = [0u8; BUF_SIZE];
    let mut line_count = 0;

    loop {
        let mut fd = Fd::STDIN;
        let n = match fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        for j in 0..n {
            print!("{}", buf[j] as char);
            if buf[j] == b'\n' {
                line_count += 1;
                if line_count >= lines {
                    return;
                }
            }
        }
    }
}

fn parse_num(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.bytes() {
        if c >= b'0' && c <= b'9' {
            n = n * 10 + (c - b'0') as usize;
        } else {
            break;
        }
    }
    n
}