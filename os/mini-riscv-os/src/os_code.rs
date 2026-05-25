#![no_std]
#![no_main]

mod os_code;

#[no_mangle]
pub extern "C" fn main() {
    os_code::rust_main();
}