#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("dirname: missing operand");
        exit(1);
    }

    for i in 1..=args.args_len() {
        if let Some(path) = args.get_str(i) {
            let result = dirname(path);
            println!("{}", result);
        }
    }
}

fn dirname(path: &str) -> &str {
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

    if start == 0 {
        return ".";
    }

    let mut result_end = start;
    while result_end > 0 && bytes[result_end - 1] == b'/' {
        result_end -= 1;
    }

    if result_end == 0 {
        return "/";
    }

    unsafe {
        core::str::from_utf8_unchecked(&bytes[..result_end])
    }
}