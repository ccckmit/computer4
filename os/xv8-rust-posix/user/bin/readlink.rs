#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        eprintln!("readlink: missing operand");
        exit(1);
    }

    let mut buf = [0u8; 256];

    for i in 1..=args.args_len() {
        if let Some(path) = args.get_str(i) {
            match readlink(path, &mut buf) {
                Ok(n) => {
                    for &b in &buf[..n] {
                        print!("{}", b as char);
                    }
                    println!();
                }
                Err(e) => {
                    eprintln!("readlink: {}: {:?}", path, e);
                    exit(1);
                }
            }
        }
    }
}