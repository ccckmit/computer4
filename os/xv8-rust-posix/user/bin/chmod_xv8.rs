#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("chmod: missing operand");
        exit(1);
    }

    let mode_str = args.get_str(1).unwrap();
    let path = args.get_str(2).unwrap();

    let mode = match parse_octal(mode_str) {
        Some(m) => m,
        None => {
            println!("chmod: invalid mode: {}", mode_str);
            exit(1);
        }
    };

    match chmod(path, mode as u16) {
        Ok(_) => {}
        Err(e) => {
            println!("chmod: cannot access '{}': {:?}", path, e);
            exit(1);
        }
    }
}

fn parse_octal(s: &str) -> Option<usize> {
    let mut val = 0usize;
    for c in s.bytes() {
        match c {
            b'0'..=b'7' => {
                val = val * 8 + (c - b'0') as usize;
            }
            _ => return None,
        }
    }
    Some(val)
}