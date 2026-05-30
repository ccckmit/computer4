#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;
const MAX_LINE: usize = 1024;
const MAX_FIELDS: usize = 64;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let mut program = "{ print $0 }";
    let mut file_count = 0;
    let mut files: [&str; 16] = [""; 16];

    for i in 1..=args.args_len() {
        if let Some(arg) = args.get_str(i) {
            if file_count == 0 && !arg.starts_with('-') {
                program = arg;
            } else if arg.starts_with("-") {
            } else {
                if file_count < 16 {
                    files[file_count] = arg;
                    file_count += 1;
                }
            }
        }
    }

    if file_count == 0 {
        return awk_stdin(program);
    }

    let mut exit_code = 0;
    for i in 0..file_count {
        if !awk_file(program, files[i]) {
            exit_code = 1;
        }
    }

    exit_code
}

fn awk_file(program: &str, file: &str) -> bool {
    let mut fd = match open(file, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => {
            eprintln!("awk: {}: No such file or directory", file);
            return false;
        }
    };

    let mut buf = [0u8; BUF_SIZE];
    let mut line = [0u8; MAX_LINE];
    let mut pos = 0;

    loop {
        let n = match fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for idx in 0..n {
            let b = buf[idx];
            if b == b'\n' || pos >= MAX_LINE - 1 {
                if pos > 0 {
                    line[pos] = 0;
                    let s = unsafe { core::str::from_utf8_unchecked(&line[..pos]) };
                    execute_action(s, program);
                }
                pos = 0;
            } else {
                line[pos] = b;
                pos += 1;
            }
        }
    }

    let _ = close(fd);
    true
}

fn awk_stdin(program: &str) -> u32 {
    let mut buf = [0u8; BUF_SIZE];
    let mut line = [0u8; MAX_LINE];
    let mut pos = 0;

    loop {
        let n = match Fd::STDIN.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for idx in 0..n {
            let b = buf[idx];
            if b == b'\n' || pos >= MAX_LINE - 1 {
                if pos > 0 {
                    line[pos] = 0;
                    let s = unsafe { core::str::from_utf8_unchecked(&line[..pos]) };
                    execute_action(s, program);
                }
                pos = 0;
            } else {
                line[pos] = b;
                pos += 1;
            }
        }
    }

    0
}

fn execute_action(line: &str, program: &str) {
    let fields = split_fields(line);

    let has_pattern = program.starts_with('/');

    if has_pattern {
        let end = program.find('/').unwrap_or(0);
        if end > 0 {
            let pattern = &program[1..end];
            if !line.contains(pattern) {
                return;
            }
        }
    }

    let action = find_action(program);

    if action.contains("print $") {
        let mut field_idx = 0usize;
        let mut collecting = false;
        let mut num_buf = [0u8; 10];
        let mut num_len = 0;

        for c in action.chars() {
            if c == '$' {
                collecting = true;
                num_len = 0;
            } else if collecting && c.is_ascii_digit() {
                if num_len < 10 {
                    num_buf[num_len] = c as u8;
                    num_len += 1;
                }
            } else if collecting && !c.is_ascii_digit() {
                if num_len > 0 {
                    num_buf[num_len] = 0;
                    field_idx = parse_field_index(&num_buf[..num_len]);
                    if field_idx > 0 && field_idx <= fields.len() as usize {
                        print!("{}", fields[field_idx - 1]);
                    }
                }
                collecting = false;
                num_len = 0;
                if c != ' ' && c != '\t' && c != '$' {
                    print!("{}", c);
                }
            } else if collecting {
                if num_len > 0 {
                    num_buf[num_len] = 0;
                    field_idx = parse_field_index(&num_buf[..num_len]);
                    if field_idx > 0 && field_idx <= fields.len() as usize {
                        print!("{}", fields[field_idx - 1]);
                    }
                }
                collecting = false;
                num_len = 0;
            } else {
                if c == '"' {
                    continue;
                }
                print!("{}", c);
            }
        }

        if collecting && num_len > 0 {
            num_buf[num_len] = 0;
            field_idx = parse_field_index(&num_buf[..num_len]);
            if field_idx > 0 && field_idx <= fields.len() as usize {
                print!("{}", fields[field_idx - 1]);
            }
        }
    } else {
        let mut in_quote = false;
        for c in action.chars() {
            if c == '"' {
                in_quote = !in_quote;
            } else if !in_quote && c == '$' {
            } else if !in_quote {
                print!("{}", c);
            }
        }
    }
    println!();
}

fn find_action(program: &str) -> &str {
    for (i, c) in program.chars().enumerate() {
        if c == '{' {
            for (j, d) in program.chars().enumerate().skip(i + 1) {
                if d == '}' {
                    return &program[i + 1..j];
                }
            }
        }
    }
    "{ print $0 }"
}

fn parse_field_index(buf: &[u8]) -> usize {
    let mut idx = 0usize;
    for &b in buf {
        if b >= b'0' && b <= b'9' {
            idx = idx * 10 + (b - b'0') as usize;
        }
    }
    idx
}

fn split_fields(line: &str) -> [&str; MAX_FIELDS] {
    let mut fields = [""; MAX_FIELDS];
    let mut count = 0;
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut start = 0;

    while start < len && count < MAX_FIELDS {
        while start < len && (bytes[start] == b' ' || bytes[start] == b'\t') {
            start += 1;
        }
        if start >= len {
            break;
        }
        let mut end = start;
        while end < len && bytes[end] != b' ' && bytes[end] != b'\t' {
            end += 1;
        }
        fields[count] = unsafe { core::str::from_utf8_unchecked(&bytes[start..end]) };
        count += 1;
        start = end;
    }

    let mut result = [""; MAX_FIELDS];
    for i in 0..count {
        result[i] = fields[i];
    }
    result
}