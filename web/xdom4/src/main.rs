use std::env;
use std::fs;
use std::process;
use xdom4::{parse, to_string};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: xdom4 <file.xml>");
        process::exit(1);
    }
    let content = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("error reading {}: {}", args[1], e);
        process::exit(1);
    });
    match parse(&content) {
        Ok(doc) => println!("{}", to_string(&doc)),
        Err(e) => {
            eprintln!("parse error: {}", e);
            process::exit(1);
        }
    }
}
