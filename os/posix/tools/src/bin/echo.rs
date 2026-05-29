fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut no_newline = false;

    if i < args.len() && args[i] == "-n" {
        no_newline = true;
        i += 1;
    }

    if let Some(line) = args[i..].join(" ").strip_suffix('\n') {
        // join already handles the spacing
    }

    let output = args[i..].join(" ");
    if no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
}
