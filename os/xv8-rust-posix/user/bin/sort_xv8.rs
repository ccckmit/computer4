#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 8192;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let argc = args.args_len();
    let mut lines: [[u8; 128]; 1000] = [[0; 128]; 1000];
    let mut line_count = 0;

    for i in 1..=argc {
        let path = args.get_str(i).unwrap_or("");
        let mut fd = if path.is_empty() || path == "-" {
            Fd::STDIN
        } else {
            match open(path, OpenFlag::READ_ONLY) {
                Ok(fd) => fd,
                Err(_) => continue,
            }
        };

        let mut buf = [0u8; BUF_SIZE];
        let mut line_buf = [0u8; 128];
        let mut pos = 0;

        loop {
            let n = match fd.read(&mut buf) {
                Ok(0) => {
                    if pos > 0 && line_count < 1000 {
                        lines[line_count][..pos].copy_from_slice(&line_buf[..pos]);
                        line_count += 1;
                    }
                    break;
                }
                Ok(n) => n,
                Err(_) => break,
            };

            for j in 0..n {
                if buf[j] == b'\n' {
                    if line_count < 1000 {
                        lines[line_count][..pos].copy_from_slice(&line_buf[..pos]);
                        line_buf = [0; 128];
                        line_count += 1;
                    }
                    pos = 0;
                } else if pos < 127 {
                    line_buf[pos] = buf[j];
                    pos += 1;
                }
            }
        }

        if path != "-" && path != "" {
            let _ = close(fd);
        }
    }

    for i in 0..line_count {
        for j in (i + 1)..line_count {
            if lines[i] > lines[j] {
                let temp = lines[i];
                lines[i] = lines[j];
                lines[j] = temp;
            }
        }
    }

    for i in 0..line_count {
        let mut j = 0;
        while lines[i][j] != 0 && j < 128 {
            print!("{}", lines[i][j] as char);
            j += 1;
        }
        println!();
    }
}