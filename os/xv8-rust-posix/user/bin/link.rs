#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        eprintln!("link: missing operand");
        exit(1);
    }

    if args.args_len() < 3 {
        eprintln!("link: missing destination");
        exit(1);
    }

    let old = args.get_str(1).unwrap();
    let new = args.get_str(2).unwrap();

    match link(old, new) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("link: cannot link '{}' to '{}': {:?}", old, new, e);
            exit(1);
        }
    }
}