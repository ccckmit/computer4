use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rustc4 <input.rs> [output.ir]");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        let mut o = input_path.clone();
        if o.ends_with(".rs") {
            o.truncate(o.len() - 3);
        }
        o + ".ir"
    };

    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };

    let ir = rustc4::compile(&source);
    match fs::write(&output_path, &ir) {
        Ok(_) => {
            eprintln!("Generated: {}", output_path);
        }
        Err(e) => {
            eprintln!("Error writing output file: {}", e);
            process::exit(1);
        }
    }
}
