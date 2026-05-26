use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = args.get(1).map(|s| s.as_str()).unwrap_or("127.0.0.1:8080");
    let root = args.get(2).map(|s| s.as_str()).unwrap_or("public");

    if let Err(e) = web4server::run(addr, Path::new(root)) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
