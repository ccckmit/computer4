#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let mut invert = false;
    let mut pattern: Option<&str> = None;
    let mut file_count = 0;
    let mut files: [&str; 16] = [""; 16];

    let mut i = 1;
    while i <= args.args_len() {
        if let Some(arg) = args.get_str(i) {
            match arg {
                "-v" => invert = true,
                _ => {
                    if pattern.is_none() {
                        pattern = Some(arg);
                    } else if file_count < 16 {
                        files[file_count] = arg;
                        file_count += 1;
                    }
                }
            }
        }
        i += 1;
    }

    let pattern = match pattern {
        Some(p) => p,
        None => {
            eprintln!("grep: pattern not specified");
            return 1;
        }
    };

    if file_count == 0 {
        return grep_stdin(pattern, invert);
    }

    let mut found = false;
    for j in 0..file_count {
        if grep_file(pattern, invert, files[j]) {
            found = true;
        }
    }

    if found { 0 } else { 1 }
}

fn grep_file(pattern: &str, invert: bool, file: &str) -> bool {
    let mut fd = match open(file, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => {
            eprintln!("grep: {}: No such file or directory", file);
            return false;
        }
    };

    let mut buf = [0u8; BUF_SIZE];
    let mut line = [0u8; 1024];
    let mut pos = 0;
    let mut line_no = 0;

    loop {
        let n = match fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for idx in 0..n {
            let b = buf[idx];
            if b == b'\n' {
                if pos > 0 {
                    line_no += 1;
                    let s = unsafe { core::str::from_utf8_unchecked(&line[..pos]) };
                    if matches(s, pattern) != invert {
                        print!("{}:{}:", file, line_no);
                        println!("{}", s);
                    }
                }
                pos = 0;
            } else if pos < 1023 {
                line[pos] = b;
                pos += 1;
            }
        }
    }

    let _ = close(fd);
    false
}

fn grep_stdin(pattern: &str, invert: bool) -> u32 {
    let mut buf = [0u8; BUF_SIZE];
    let mut line = [0u8; 1024];
    let mut pos = 0;
    let mut found = false;

    loop {
        let n = match Fd::STDIN.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for idx in 0..n {
            let b = buf[idx];
            if b == b'\n' {
                if pos > 0 {
                    let s = unsafe { core::str::from_utf8_unchecked(&line[..pos]) };
                    if matches(s, pattern) != invert {
                        println!("{}", s);
                        found = true;
                    }
                }
                pos = 0;
            } else if pos < 1023 {
                line[pos] = b;
                pos += 1;
            }
        }
    }

    if found { 0 } else { 1 }
}

fn matches(haystack: &str, needle: &str) -> bool {
    if needle.len() == 0 {
        return true;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    let nlen = n.len();

    if h.len() < nlen {
        return false;
    }

    let mut result = false;
    let mut i = 0;
    while i <= h.len() - nlen {
        let mut j = 0;
        while j < nlen {
            if h[i + j] != n[j] {
                break;
            }
            j += 1;
        }
        if j == nlen {
            result = true;
            break;
        }
        i += 1;
    }
    result
}