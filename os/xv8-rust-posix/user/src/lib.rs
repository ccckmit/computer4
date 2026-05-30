#![no_std]
#![feature(alloc)]

extern crate alloc;

use core::panic::PanicInfo;

#[macro_use]
mod io;
mod args;
mod line;
mod syscall;

pub use kernel::abi::*;

pub use args::*;
pub use io::{Read, Stderr, Stdin, Stdout, Write};
pub use line::LineEditor;
pub use syscall::*;

unsafe extern "Rust" {
    fn main(args: Args);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
fn _start() -> ! {
    unsafe {
        let args = Args::from_stack();
        main(args);
        exit(0);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("! {}", info);
    exit(1)
}