#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
extern crate alloc;

pub mod io;
pub mod fmt;
#[cfg(unix)]
pub mod opt;

pub use io::Read;
pub use io::Write;
pub use io::File;
pub use io::{stdin, stdout, stderr};
pub use io::{print, println, exit, args};