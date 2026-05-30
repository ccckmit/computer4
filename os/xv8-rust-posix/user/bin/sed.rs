#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let mut pattern = "";
    let mut replacement = "";
    let mut global = false;
    let mut print_only = false;
    let mut file_index = 1;

    let mut i = 1;
    while i <= args.args_len() {
        if let Some(arg) = args.get_str(i) {
            if arg.starts_with("s/") {
                let parsed = parse_subst_cmd(arg);
                if let Some((p, r, g)) = parsed {
                    pattern = p;
                    replacement = r;
                    global = g;
                }
            }
            i += 1;
            continue;
        }
        i += 1;
    }

    if pattern.is_empty() {
        eprintln!("sed: no substitution specified");
        return 1;
    }

    if args.args_len() < file_index + 1 || args.get_str(file_index + 1) == None {
        return sed_stdin(pattern, replacement, global);
    }

    let mut exit_code = 0;
    for i in (file_index + 1)..=args.args_len() {
        if let Some(file) = args.get_str(i) {
            if !sed_file(pattern, replacement, global, file) {
                exit_code = 1;
            }
        }
    }

    exit_code
}

fn parse_subst_cmd(cmd: &str) -> Option<(&str, &str, bool)> {
    if !cmd.starts_with("s/") || cmd.len() < 4 {
        return None;
    }

    let sep = cmd.chars().nth(2).unwrap();
    let mut parts: [&str; 3] = ["", "", ""];
    let mut part_idx = 0;
    let mut start = 3;

    for (idx, c) in cmd[3..].chars().enumerate() {
        if c == sep {
            parts[part_idx] = &cmd[3..3 + idx];
            part_idx += 1;
            start = 3 + idx + 1;
            if part_idx == 2 {
                break;
            }
        }
    }

    if part_idx < 1 {
        return None;
    }

    let mut global = false;
    let rest = &cmd[start..];
    if rest.ends_with("g") {
        global = true;
    }

    Some((parts[0], parts[1], global))
}

fn sed_file(pattern: &str, replacement: &str, global: bool, file: &str) -> bool {
    let mut fd = match open(file, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => {
            eprintln!("sed: {}: No such file or directory", file);
            return false;
        }
    };

    let mut buf = [0u8; BUF_SIZE];
    let mut line = [0u8; 2048];
    let mut pos = 0;

    loop {
        let n = match fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for idx in 0..n {
            let b = buf[idx];
            if b == b'\n' || pos >= 2047 {
                if pos > 0 {
                    let s = unsafe { core::str::from_utf8_unchecked(&line[..pos]) };
                    let result = substitute(s, pattern, replacement, global);
                    println!("{}", result);
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

fn sed_stdin(pattern: &str, replacement: &str, global: bool) -> u32 {
    let mut buf = [0u8; BUF_SIZE];
    let mut line = [0u8; 2048];
    let mut pos = 0;

    loop {
        let n = match Fd::STDIN.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        for idx in 0..n {
            let b = buf[idx];
            if b == b'\n' || pos >= 2047 {
                if pos > 0 {
                    let s = unsafe { core::str::from_utf8_unchecked(&line[..pos]) };
                    let result = substitute(s, pattern, replacement, global);
                    println!("{}", result);
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

fn substitute(s: &str, pattern: &str, replacement: &str, global: bool) -> AllocStr {
    let mut result = AllocStr { data: [0u8; 2048], len: 0 };
    let mut i = 0;
    let s_bytes = s.as_bytes();
    let p_bytes = pattern.as_bytes();
    let r_bytes = replacement.as_bytes();
    let p_len = p_bytes.len();

    if p_len == 0 {
        for &c in s_bytes {
            if result.len < 2047 {
                result.data[result.len] = c;
                result.len += 1;
            }
        }
        return result;
    }

    loop {
        if i + p_len > s_bytes.len() {
            for j in i..s_bytes.len() {
                if result.len < 2047 {
                    result.data[result.len] = s_bytes[j];
                    result.len += 1;
                }
            }
            break;
        }

        let mut match_len = 0;
        while match_len < p_len && s_bytes[i + match_len] == p_bytes[match_len] {
            match_len += 1;
        }

        if match_len == p_len {
            for &c in r_bytes {
                if result.len < 2047 {
                    result.data[result.len] = c;
                    result.len += 1;
                }
            }
            i += p_len;
            if !global {
                for j in i..s_bytes.len() {
                    if result.len < 2047 {
                        result.data[result.len] = s_bytes[j];
                        result.len += 1;
                    }
                }
                break;
            }
        } else {
            if result.len < 2047 {
                result.data[result.len] = s_bytes[i];
                result.len += 1;
            }
            i += 1;
        }
    }

    result
}

struct AllocStr {
    data: [u8; 2048],
    len: usize,
}

impl core::fmt::Display for AllocStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = unsafe { core::str::from_utf8_unchecked(&self.data[..self.len]) };
        write!(f, "{}", s)
    }
}