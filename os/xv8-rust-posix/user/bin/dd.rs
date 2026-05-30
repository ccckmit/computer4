#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 8192;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let mut input_path: &str = "";
    let mut output_path: &str = "";
    let mut count: usize = 0;

    for i in 1..=args.args_len() {
        if let Some(arg) = args.get_str(i) {
            if arg.starts_with("if=") {
                input_path = &arg[3..];
            } else if arg.starts_with("of=") {
                output_path = &arg[3..];
            } else if arg.starts_with("count=") {
                count = parse_num(&arg[6..]);
            }
        }
    }

    if input_path.is_empty() || output_path.is_empty() {
        eprintln!("dd: missing input or output file");
        return 1;
    }

    let mut srcfd = match open(input_path, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("dd: {}: {:?}", input_path, e);
            return 1;
        }
    };

    let mut dstfd = match open(output_path, OpenFlag::CREATE | OpenFlag::WRITE_ONLY | OpenFlag::TRUNCATE) {
        Ok(fd) => fd,
        Err(e) => {
            let _ = close(srcfd);
            eprintln!("dd: {}: {:?}", output_path, e);
            return 1;
        }
    };

    let mut buf = [0u8; BUF_SIZE];
    let mut total = 0;

    if count > 0 {
        for _ in 0..count {
            let n = match srcfd.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            if write(dstfd, &buf[..n]).is_err() {
                break;
            }
            total += n;
        }
    } else {
        loop {
            let n = match srcfd.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            if write(dstfd, &buf[..n]).is_err() {
                break;
            }
            total += n;
        }
    }

    let _ = close(srcfd);
    let _ = close(dstfd);

    println!("{} bytes copied", total);
    0
}

fn parse_num(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.chars() {
        if c.is_ascii_digit() {
            n = n * 10 + (c as usize - '0' as usize);
        }
    }
    n
}