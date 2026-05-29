#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.len() < 2 {
        exit_with_msg("usage: kill pid...");
    }

    for pid_str in args.args_as_str() {
        let pid = pid_str.parse::<usize>().unwrap_or_else(|_| {
            exit_with_msg("kill: invalid pid");
        });
        if kill(pid, 15).is_err() { // SIGTERM
            eprintln!("kill: failed to kill {}", pid);
        }
    }
}
