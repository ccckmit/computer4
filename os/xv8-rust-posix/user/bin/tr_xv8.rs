#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("tr: missing operand");
        exit(1);
    }

    let set1 = args.get_str(1).unwrap();
    let set2 = if args.args_len() >= 3 {
        args.get_str(2).unwrap()
    } else {
        ""
    };

    let mut fd = Fd::STDIN;
    let mut buf = [0u8; BUF_SIZE];
    let mut prev = 0u8;
    let mut in_set1 = false;

    loop {
        let n = match fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for j in 0..n {
            let c = buf[j];
            let mut matched = false;
            let mut output_c = c;

            for (k, sc) in set1.bytes().enumerate() {
                if sc == c {
                    matched = true;
                    if k < set2.len() {
                        output_c = set2.as_bytes()[k];
                    } else if !set2.is_empty() {
                        output_c = set2.as_bytes()[set2.len() - 1];
                    }
                    break;
                }
            }

            if !matched && !set1.contains(c as char) && prev != 0 {
                print!("{}", prev as char);
                prev = 0;
            }

            if matched {
                print!("{}", output_c as char);
                prev = 0;
            } else if c != b'\n' {
                prev = c;
            } else {
                println!();
            }
        }
    }
    if prev != 0 {
        println!();
    }
}