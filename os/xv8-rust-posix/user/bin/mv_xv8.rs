#![no_std]
#![no_main]

use user::*;

const BUF_SIZE: usize = 4096;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("mv: missing operand");
        exit(1);
    }

    let src = args.get_str(args.args_len() - 1).unwrap();
    let dst = args.get_str(args.args_len()).unwrap();

    let mut src_fd = match open(src, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => {
            println!("mv: cannot stat '{}': No such file or directory", src);
            exit(1);
        }
    };

    let dst_fd = match open(dst, OpenFlag::CREATE | OpenFlag::WRITE_ONLY | OpenFlag::TRUNCATE) {
        Ok(fd) => fd,
        Err(e) => {
            let _ = close(src_fd);
            println!("mv: cannot create '{}': {:?}", dst, e);
            exit(1);
        }
    };

    let mut buf = [0u8; BUF_SIZE];
    loop {
        match src_fd.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if write(dst_fd, &buf[..n]).is_err() {
                    let _ = close(src_fd);
                    let _ = close(dst_fd);
                    println!("mv: write error");
                    exit(1);
                }
            }
            Err(_) => {
                let _ = close(src_fd);
                let _ = close(dst_fd);
                println!("mv: read error");
                exit(1);
            }
        }
    }

    let _ = close(src_fd);
    let _ = close(dst_fd);
    let _ = unlink(src);
}