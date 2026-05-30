#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let mut prev_line = [0u8; 256];
    let mut prev_len = 0;
    let mut first = true;

    let mut fd = Fd::STDIN;
    let mut buf = [0u8; BUF_SIZE];
    let mut line_buf = [0u8; 256];
    let mut pos = 0;

    loop {
        let n = match fd.read(&mut buf) {
            Ok(0) => {
                if pos > 0 {
                    if first || !same_line(&prev_line[..prev_len], &line_buf[..pos]) {
                        print_line(&line_buf[..pos]);
                        prev_line[..pos].copy_from_slice(&line_buf[..pos]);
                        prev_len = pos;
                    }
                }
                break;
            }
            Ok(n) => n,
            Err(_) => break,
        };

        for j in 0..n {
            if buf[j] == b'\n' {
                if first || !same_line(&prev_line[..prev_len], &line_buf[..pos]) {
                    print_line(&line_buf[..pos]);
                    prev_line[..pos].copy_from_slice(&line_buf[..pos]);
                    prev_len = pos;
                    first = false;
                }
                pos = 0;
            } else if pos < 255 {
                line_buf[pos] = buf[j];
                pos += 1;
            }
        }
    }
}

fn same_line(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}

fn print_line(line: &[u8]) {
    for &b in line {
        print!("{}", b as char);
    }
    println!();
}