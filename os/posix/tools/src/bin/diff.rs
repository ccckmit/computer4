use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        eprintln!("diff: invalid option -- '{}'", args[i]);
        std::process::exit(1);
    }

    if i + 2 > args.len() {
        eprintln!("usage: diff file1 file2");
        std::process::exit(1);
    }

    let path1 = Path::new(&args[i]);
    let path2 = Path::new(&args[i + 1]);

    let text1 = fs::read_to_string(path1).unwrap_or_else(|e| {
        eprintln!("diff: {}: {}", path1.display(), e);
        std::process::exit(1);
    });
    let text2 = fs::read_to_string(path2).unwrap_or_else(|e| {
        eprintln!("diff: {}: {}", path2.display(), e);
        std::process::exit(1);
    });

    let lines1: Vec<&str> = text1.lines().collect();
    let lines2: Vec<&str> = text2.lines().collect();

    // Simple line-by-line comparison
    let mut first_diff = true;
    for i in 0..lines1.len().max(lines2.len()) {
        let l1 = lines1.get(i).copied().unwrap_or("");
        let l2 = lines2.get(i).copied().unwrap_or("");
        if l1 != l2 {
            if first_diff {
                println!("--- {}", path1.display());
                println!("+++ {}", path2.display());
                first_diff = false;
            }
            println!("@@ -{} +{} @@", i + 1, i + 1);
            if i < lines1.len() {
                println!("-{}", l1);
            }
            if i < lines2.len() {
                println!("+{}", l2);
            }
        }
    }

    if first_diff {
        // Identical
    } else {
        std::process::exit(1);
    }
}
