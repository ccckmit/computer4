#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 1 {
        println!("touch: missing file operand");
        exit(1);
    }

    let path = args.get_str(args.args_len()).unwrap();

    match open(path, OpenFlag::CREATE | OpenFlag::WRITE_ONLY) {
        Ok(fd) => {
            let _ = close(fd);
        }
        Err(_) => {
            println!("touch: cannot touch '{}'", path);
            exit(1);
        }
    }
}