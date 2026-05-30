#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.args_len() < 2 {
        println!("env: environment variables not supported");
        return;
    }

    if args.args_len() == 2 {
        let cmd = args.get_str(1).unwrap();
        exec(cmd, &[]);
        eprintln!("env: {}: not found", cmd);
        exit(1);
    }

    eprintln!("env: environment variables not supported");
    exit(1);
}