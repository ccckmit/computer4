use std::env;
use std::fs;
use js4::run;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("js4 - JavaScript Interpreter");
        println!("Usage: js4 <file.js>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };

    match run(&code) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Runtime error: {}", err);
            std::process::exit(1);
        }
    }
}