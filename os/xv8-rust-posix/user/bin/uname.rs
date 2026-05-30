#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    let mut all = false;

    if args.args_len() == 1 {
        all = true;
    } else {
        for i in 1..=args.args_len() {
            if let Some(arg) = args.get_str(i) {
                match arg {
                    "-a" | "--all" => {
                        all = true;
                    }
                    "-s" | "--kernel-name" => {
                        print!("xv8 ");
                    }
                    "-n" | "--nodename" => {
                        print!("xv8 ");
                    }
                    "-r" | "--kernel-release" => {
                        print!("1.0 ");
                    }
                    "-v" | "--version" => {
                        print!("v1.2 ");
                    }
                    "-m" | "--machine" => {
                        print!("riscv64 ");
                    }
                    _ => {
                        eprintln!("uname: invalid option '{}'", arg);
                        exit(1);
                    }
                }
            }
        }
    }

    if all {
        print!("xv8 xv8 1.0 v1.2 riscv64");
    }

    println!();
}