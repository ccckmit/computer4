#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) -> u32 {
    let mut sources: [&str; 16] = [""; 16];
    let mut dest = "";
    let mut mode: u16 = 0o755;
    let mut src_count = 0;

    let mut i = 1;
    while i <= args.args_len() {
        if let Some(arg) = args.get_str(i) {
            if arg == "-m" {
                i += 1;
                if i <= args.args_len() {
                    if let Some(m) = args.get_str(i) {
                        mode = parse_mode(m);
                    }
                }
            } else {
                if src_count == 0 || (src_count > 0 && !dest.is_empty()) {
                    if src_count < 16 {
                        sources[src_count] = arg;
                        src_count += 1;
                    }
                } else {
                    dest = arg;
                }
            }
        }
        i += 1;
    }

    if src_count == 0 || dest.is_empty() {
        eprintln!("install: missing file operand");
        return 1;
    }

    if src_count > 1 {
        if mkdir(dest).is_err() {
            eprintln!("install: cannot create directory '{}'", dest);
            return 1;
        }
    }

    for idx in 0..src_count {
        let src = sources[idx];
        let dst = dest;

        match copy_file(src, dst) {
            Ok(_) => {
                chmod(dst, mode).ok();
            }
            Err(e) => {
                eprintln!("install: cannot copy '{}' to '{}': {:?}", src, dst, e);
                return 1;
            }
        }
    }

    0
}

fn parse_mode(s: &str) -> u16 {
    let mut mode: u16 = 0;
    for c in s.chars() {
        match c {
            '0'..='7' => {
                mode = mode * 8 + (c as u16 - '0' as u16);
            }
            _ => {}
        }
    }
    mode
}

fn copy_file(src: &str, dst: &str) -> Result<(), Errno> {
    let mut src_fd = open(src, OpenFlag::READ_ONLY)?;
    let mut dst_fd = open(dst, OpenFlag::CREATE | OpenFlag::WRITE_ONLY | OpenFlag::TRUNCATE)?;

    let mut buf = [0u8; 4096];
    loop {
        let n = match src_fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => {
                let _ = close(src_fd);
                let _ = close(dst_fd);
                return Err(Errno::EIO);
            }
        };
        if write(dst_fd, &buf[..n]).is_err() {
            let _ = close(src_fd);
            let _ = close(dst_fd);
            return Err(Errno::EIO);
        }
    }

    let _ = close(src_fd);
    let _ = close(dst_fd);
    Ok(())
}