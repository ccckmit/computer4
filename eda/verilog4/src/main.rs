use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use verilog4::{parse_file, gen_ruhdl};

const RUHDL_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../ruhdl/src/lib.rs");
const CACHE_DIR: &str = "/tmp/ruhdl_rlib";

fn build_ruhdl_rlib() -> String {
    fs::create_dir_all(CACHE_DIR).ok();
    let rlib_path = format!("{}/libruhdl.rlib", CACHE_DIR);
    if !Path::new(&rlib_path).exists() {
        let status = Command::new("rustc")
            .args(["--crate-type", "lib", "--crate-name", "ruhdl",
                   "--out-dir", CACHE_DIR, RUHDL_SRC, "--edition", "2021"])
            .status()
            .expect("failed to build ruhdl library");
        if !status.success() {
            panic!("ruhdl compilation failed");
        }
    }
    rlib_path
}

fn run_rhdl(input_path: &str) {
    let rlib = build_ruhdl_rlib();
    let stem = Path::new(input_path).file_stem().and_then(|s| s.to_str()).unwrap_or("a.out");
    fs::create_dir_all("/tmp/ruhdl_bin").ok();
    let binary = format!("/tmp/ruhdl_bin/{}", stem);
    let status = Command::new("rustc")
        .args(["--extern", &format!("ruhdl={}", rlib), input_path,
               "-o", &binary, "--edition", "2021"])
        .status()
        .expect("failed to compile ruhdl source");
    if !status.success() {
        panic!("compilation failed");
    }
    let mut child = Command::new(&binary)
        .spawn()
        .expect("failed to execute");
    child.wait().ok();
}

fn convert_verilog(input_path: &str, output_arg: &str) {
    let modules = parse_file(input_path);
    let rust_code = gen_ruhdl(&modules);
    let output_path = if output_arg.ends_with(".rhdl") || output_arg.ends_with(".rs") {
        // treat as output file path
        output_arg.to_string()
    } else {
        // treat as directory
        let input_stem = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        format!("{}/{}.rs", output_arg, input_stem)
    };
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).ok();
        }
    }
    fs::write(&output_path, &rust_code)
        .expect(&format!("Failed to write output: {}", output_path));
    println!("Generated: {}", output_path);
    for m in &modules {
        println!("  module: {}", m.name);
        println!("    ports: {}", m.ports.iter().map(|p| format!("{}: {:?}", p.name, p.direction)).collect::<Vec<_>>().join(", "));
        println!("    items: {}", m.items.len());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage:");
        println!("  verilog4 <input.v> [output.rhdl]   Convert Verilog to ruHDL");
        println!("  verilog4 <input.rhdl>               Compile and run ruHDL");
        println!();
        println!("Examples:");
        println!("  verilog4 adder4_tb.v");
        println!("  verilog4 adder4_tb.v adder4_tb.rhdl");
        println!("  verilog4 adder4_tb.rhdl");
        return;
    }

    let input_path = &args[1];

    if input_path.ends_with(".v") {
        if args.len() > 2 {
            let output_path = &args[2];
            convert_verilog(input_path, output_path);
        } else {
            convert_verilog(input_path, ".");
        }
    } else if input_path.ends_with(".rhdl") || input_path.ends_with(".rs") {
        run_rhdl(input_path);
    } else {
        eprintln!("Unknown file extension (use .v, .rhdl, or .rs)");
    }
}
