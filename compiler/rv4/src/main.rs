use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rv4 <elf-file>");
        process::exit(1);
    }

    let path = &args[1];
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("rv4: error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    match rv4::run_elf(&data) {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("rv4: error: {}", e);
            process::exit(1);
        }
    }
}
