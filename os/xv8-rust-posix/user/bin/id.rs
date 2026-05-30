#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let uid = getuid();
    let gid = getgid();

    println!("uid={} gid={}", uid, gid);
}