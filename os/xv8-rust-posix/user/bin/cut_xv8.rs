#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let mut delim = b'\t';
    let mut fields: Option<(usize, usize)> = None;
    let mut start_file = 1;

    if args.args_len() >= 2 {
        let arg1 = args.get_str(1).unwrap();
        if arg1.starts_with("-d") {
            if arg1.len() > 2 {
                delim = arg1.as_bytes()[2];
            }
            if args.args_len() >= 3 {
                let arg2 = args.get_str(2).unwrap();
                if arg2.starts_with("-f") {
                    fields = parse_fields(&arg2[2..]);
                    start_file = 3;
                }
            }
        } else if arg1.starts_with("-f") {
            fields = parse_fields(&arg1[2..]);
            start_file = 2;
        }
    }

    let argc = args.args_len();
    if start_file > argc {
        return;
    }

    let mut buf = [0u8; BUF_SIZE];
    let mut line_buf = [0u8; 2048];

    for i in start_file..=argc {
        let path = args.get_str(i).unwrap();
        let mut fd = match open(path, OpenFlag::READ_ONLY) {
            Ok(fd) => fd,
            Err(_) => continue,
        };

        let mut pos = 0;
        loop {
            let n = match fd.read(&mut buf) {
                Ok(0) => {
                    if pos > 0 {
                        process_line(&line_buf[..pos], fields, delim);
                    }
                    break;
                }
                Ok(n) => n,
                Err(_) => break,
            };

            for j in 0..n {
                if buf[j] == b'\n' {
                    process_line(&line_buf[..pos], fields, delim);
                    pos = 0;
                } else if pos < 2047 {
                    line_buf[pos] = buf[j];
                    pos += 1;
                }
            }
        }
        let _ = close(fd);
    }
}

fn parse_fields(s: &str) -> Option<(usize, usize)> {
    let mut start = 0usize;
    let mut end = 0usize;
    let mut parsing_start = true;

    for c in s.bytes() {
        if c >= b'0' && c <= b'9' {
            if parsing_start {
                start = start * 10 + (c - b'0') as usize;
            } else {
                end = end * 10 + (c - b'0') as usize;
            }
        } else if c == b'-' {
            parsing_start = false;
        }
    }
    if end == 0 {
        end = start;
    }
    Some((start.saturating_sub(1), end.saturating_sub(1)))
}

fn process_line(line: &[u8], fields: Option<(usize, usize)>, delim: u8) {
    let mut field_idx = 0usize;
    let mut field_start = 0usize;
    let mut in_field = false;
    let mut printed = false;
    let (f_start, f_end) = fields.unwrap_or((0, usize::MAX));

    for (i, &b) in line.iter().enumerate() {
        if b == delim || i == line.len() {
            if in_field && field_idx >= f_start && field_idx <= f_end {
                if printed {
                    print!("{}", delim as char);
                }
                for j in field_start..i {
                    print!("{}", line[j] as char);
                }
                printed = true;
            }
            field_idx += 1;
            field_start = i + 1;
            in_field = false;
        } else {
            in_field = true;
        }
    }
    if printed {
        println!();
    }
}