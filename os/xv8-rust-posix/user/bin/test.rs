#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let result = test_expr(&args);
    if !result {
        exit(1);
    }
    0
}

fn test_expr(args: &Args) -> bool {
    let argc = args.args_len();

    if argc == 1 {
        return false;
    }

    if argc == 2 {
        let s = args.get_str(1).unwrap();
        return !s.is_empty();
    }

    if argc == 3 {
        let op = args.get_str(2).unwrap();
        let s = args.get_str(1).unwrap();
        match op {
            "-n" => return !s.is_empty(),
            "-z" => return s.is_empty(),
            _ => return false,
        }
    }

    if argc >= 4 {
        let a = args.get_str(1).unwrap();
        let op = args.get_str(2).unwrap();
        let b = args.get_str(3).unwrap();

        match op {
            "=" | "==" => return a == b,
            "!=" => return a != b,
            "-eq" => {
                return compare_i64(a, b, |x, y| x == y);
            }
            "-ne" => {
                return compare_i64(a, b, |x, y| x != y);
            }
            "-lt" => {
                return compare_i64(a, b, |x, y| x < y);
            }
            "-le" => {
                return compare_i64(a, b, |x, y| x <= y);
            }
            "-gt" => {
                return compare_i64(a, b, |x, y| x > y);
            }
            "-ge" => {
                return compare_i64(a, b, |x, y| x >= y);
            }
            _ => return false,
        }
    }

    false
}

fn compare_i64<F>(a: &str, b: &str, f: F) -> bool
where
    F: Fn(i64, i64) -> bool,
{
    if let (Some(ai), Some(bi)) = (parse_i64(a), parse_i64(b)) {
        f(ai, bi)
    } else {
        false
    }
}

fn parse_i64(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut start = 0;
    let mut negative = false;

    if bytes[0] == b'-' {
        negative = true;
        start = 1;
    }

    if start >= bytes.len() {
        return None;
    }

    let mut result: i64 = 0;
    for &b in &bytes[start..] {
        if b < b'0' || b > b'9' {
            return None;
        }
        result = result * 10 + (b - b'0') as i64;
    }

    if negative {
        result = -result;
    }

    Some(result)
}