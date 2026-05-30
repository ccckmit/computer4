#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let mut n = 10;
    let mut cmd_index = 1;
    let mut has_n_option = false;

    for i in 1..=args.args_len() {
        if let Some(arg) = args.get_str(i) {
            if arg == "-n" && !has_n_option {
                if i + 1 <= args.args_len() {
                    if let Some(n_str) = args.get_str(i + 1) {
                        n = n_str.parse().unwrap_or(10);
                        cmd_index = i + 2;
                        has_n_option = true;
                    }
                }
            } else if !arg.starts_with('-') && i > 1 {
                break;
            }
        }
    }

    if cmd_index > args.args_len() {
        match nice(n as isize) {
            Ok(v) => println!("{}", v),
            Err(_) => println!("0"),
        }
        exit(0);
    }

    let cmd = args.get_str(cmd_index).unwrap();
    let mut argv: [&str; 16] = [""; 16];
    let mut argc = 0;

    for i in cmd_index..=args.args_len() {
        if let Some(s) = args.get_str(i) {
            if argc < 16 {
                argv[argc] = s;
                argc += 1;
            }
        }
    }

    let _ = nice(n as isize);
    exec(cmd, &argv[..argc]);

    eprintln!("nice: {}: not found", cmd);
    exit(1);
}