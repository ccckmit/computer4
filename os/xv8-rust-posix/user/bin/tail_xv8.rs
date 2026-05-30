#![no_std]
#![no_main]

use user::*;

const DEFAULT_LINES: usize = 10;
const BUF_SIZE: usize = 8192;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let argc = args.args_len();
    if argc < 1 {
        println!("tail: missing operand");
        exit(1);
    }

    let mut lines = DEFAULT_LINES;
    let mut start_file = 1;

    if argc > 1 {
        let opt = args.get_str(1).unwrap();
        if opt.starts_with("-n") {
            let n_str = if opt.len() > 2 { &opt[2..] } else { "10" };
            lines = parse_num(n_str);
            start_file = 2;
        }
    }

    let mut buf = [0u8; BUF_SIZE];

    for i in start_file..=argc {
        let path = args.get_str(i).unwrap();
        let mut fd = match open(path, OpenFlag::READ_ONLY) {
            Ok(fd) => fd,
            Err(_) => {
                println!("tail: cannot open '{}': No such file or directory", path);
                continue;
            }
        };

        let mut content = [0u8; 65536];
        let mut total = 0;
        loop {
            let n = match fd.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            if total + n > 65536 {
                break;
            }
            content[total..total+n].copy_from_slice(&buf[..n]);
            total += n;
        }
        let _ = close(fd);

        if total == 0 {
            continue;
        }

        let mut line_starts: [usize; 1000] = [0; 1000];
        let mut line_count = 0;
        line_starts[0] = 0;
        line_count = 1;

        for j in 0..total {
            if content[j] == b'\n' && line_count < 1000 {
                line_starts[line_count] = j + 1;
                line_count += 1;
            }
        }

        let start = if line_count <= lines {
            0
        } else {
            line_starts[line_count - lines]
        };

        for j in start..total {
            print!("{}", content[j] as char);
        }
    }
}

fn parse_num(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.bytes() {
        if c >= b'0' && c <= b'9' {
            n = n * 10 + (c - b'0') as usize;
        }
    }
    n
}