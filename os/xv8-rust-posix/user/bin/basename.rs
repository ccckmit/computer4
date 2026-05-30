#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("basename: missing operand");
        exit(1);
    }

    for i in 1..=args.args_len() {
        if let Some(path) = args.get_str(i) {
            let result = basename(path);
            println!("{}", result);
        }
    }
}

fn basename(path: &str) -> &str {
    if path.is_empty() {
        return ".";
    }

    let bytes = path.as_bytes();
    let len = bytes.len();

    let mut end = len;
    while end > 0 && bytes[end - 1] == b'/' {
        end -= 1;
    }

    if end == 0 {
        return "/";
    }

    let mut start = end;
    while start > 0 && bytes[start - 1] != b'/' {
        start -= 1;
    }

    unsafe {
        core::str::from_utf8_unchecked(&bytes[start..end])
    }
}