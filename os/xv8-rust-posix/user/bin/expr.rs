#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        exit(1);
    }

    let result = evaluate_expr(&args);
    if let Some(n) = result {
        print_int(n);
        println!();
    } else {
        exit(1);
    }
}

fn evaluate_expr(args: &Args) -> Option<i64> {
    let mut i = 1;
    let argc = args.args_len();

    if i >= argc {
        return None;
    }

    let first = args.get_str(i).unwrap();
    let mut result: i64 = first.parse().ok()?;

    i += 1;

    while i < argc {
        let op = args.get_str(i).unwrap();
        i += 1;

        if i >= argc {
            return None;
        }

        let arg = args.get_str(i).unwrap();
        i += 1;

        let val: i64 = arg.parse().ok()?;

        match op {
            "+" | "plus" => result = result.wrapping_add(val),
            "-" | "minus" => result = result.wrapping_sub(val),
            "*" | "times" => result = result.wrapping_mul(val),
            "/" | "div" => {
                if val == 0 {
                    return None;
                }
                result /= val;
            }
            "%" | "mod" => {
                if val == 0 {
                    return None;
                }
                result %= val;
            }
            _ => return None,
        }
    }

    Some(result)
}

fn print_int(n: i64) {
    if n == 0 {
        print!("0");
        return;
    }

    let negative = n < 0;
    let mut n = n.abs() as u64;

    let mut digits = [0u8; 20];
    let mut len = 0;

    while n > 0 {
        digits[len] = (n % 10) as u8;
        len += 1;
        n /= 10;
    }

    if negative {
        print!("-");
    }

    let mut i = len;
    while i > 0 {
        i -= 1;
        print!("{}", (digits[i] + b'0') as char);
    }
}