use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: dirname string");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let parent = match path.parent() {
        Some(p) if p.as_os_str().is_empty() => ".",
        Some(p) => p.to_string_lossy().as_ref(),
        None => ".",
    };

    println!("{}", parent);
}
