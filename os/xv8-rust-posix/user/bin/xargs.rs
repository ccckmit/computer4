#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) -> u32 {
    eprintln!("xargs: not fully implemented");
    0
}