#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 1 {
        return;
    }

    let fmt = args.get_str(1).unwrap();
    let mut arg_index = 2;
    let mut i = 0;

    while i < fmt.len() {
        let c = fmt.as_bytes()[i] as char;

        if c != '%' {
            print!("{}", c);
            i += 1;
            continue;
        }

        i += 1;
        if i >= fmt.len() {
            print!("%");
            break;
        }

        let spec = fmt.as_bytes()[i] as char;

        if spec == '%' {
            print!("%");
            i += 1;
            continue;
        }

        i += 1;

        match spec {
            'c' => {
                if arg_index <= args.args_len() {
                    if let Some(s) = args.get_str(arg_index) {
                        if let Some(&b) = s.as_bytes().first() {
                            print!("{}", b as char);
                        }
                    }
                    arg_index += 1;
                }
            }
            's' => {
                if arg_index <= args.args_len() {
                    if let Some(s) = args.get_str(arg_index) {
                        print!("{}", s);
                    }
                    arg_index += 1;
                }
            }
            'd' | 'i' | 'u' => {
                if arg_index <= args.args_len() {
                    if let Some(s) = args.get_str(arg_index) {
                        if let Some(n) = parse_int(s) {
                            print_int(n);
                        }
                    }
                    arg_index += 1;
                }
            }
            'x' | 'X' => {
                if arg_index <= args.args_len() {
                    if let Some(s) = args.get_str(arg_index) {
                        if let Some(n) = parse_int(s) {
                            print_hex(n as u64, spec == 'X');
                        }
                    }
                    arg_index += 1;
                }
            }
            'o' => {
                if arg_index <= args.args_len() {
                    if let Some(s) = args.get_str(arg_index) {
                        if let Some(n) = parse_int(s) {
                            print_oct(n as u64);
                        }
                    }
                    arg_index += 1;
                }
            }
            _ => {
                print!("%{}", spec);
            }
        }
    }

    println!();
}

fn parse_int(s: &str) -> Option<i64> {
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

fn print_hex(n: u64, uppercase: bool) {
    if n == 0 {
        print!("0");
        return;
    }

    let hex_chars = b"0123456789abcdef0123456789ABCDEF";
    let offset = if uppercase { 16 } else { 0 };

    let mut digits = [0u8; 16];
    let mut len = 0;
    let mut n = n;

    while n > 0 {
        digits[len] = hex_chars[(n & 0xF) as usize + offset];
        len += 1;
        n >>= 4;
    }

    let mut i = len;
    while i > 0 {
        i -= 1;
        print!("{}", digits[i] as char);
    }
}

fn print_oct(n: u64) {
    if n == 0 {
        print!("0");
        return;
    }

    let mut digits = [0u8; 22];
    let mut len = 0;
    let mut n = n;

    while n > 0 {
        digits[len] = ((n & 7) as u8).wrapping_add(b'0');
        len += 1;
        n >>= 3;
    }

    let mut i = len;
    while i > 0 {
        i -= 1;
        print!("{}", digits[i] as char);
    }
}