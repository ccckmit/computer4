#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        eprintln!("nohup: missing operand");
        exit(1);
    }

    let cmd = args.get_str(1).unwrap();
    exec(cmd, &[cmd]);
    eprintln!("nohup: {}: not found", cmd);
    exit(1);
}