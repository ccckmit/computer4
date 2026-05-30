#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        eprintln!("unlink: missing operand");
        exit(1);
    }

    let path = args.get_str(1).unwrap();

    match unlink(path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("unlink: cannot unlink '{}': {:?}", path, e);
            exit(1);
        }
    }
}