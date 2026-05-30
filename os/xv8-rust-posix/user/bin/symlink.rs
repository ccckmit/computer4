#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        eprintln!("symlink: missing operand");
        exit(1);
    }

    if args.args_len() < 3 {
        eprintln!("symlink: missing destination");
        exit(1);
    }

    let target = args.get_str(1).unwrap();
    let linkpath = args.get_str(2).unwrap();

    match symlink(target, linkpath) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("symlink: cannot create symlink '{}' to '{}': {:?}", linkpath, target, e);
            exit(1);
        }
    }
}