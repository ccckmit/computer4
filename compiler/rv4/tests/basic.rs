use std::process::Command;

fn run_example(name: &str) -> (String, i32) {
    let output = Command::new("cargo")
        .args(["run", "--", &format!("examples/{}.o", name)])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run rv4");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, code)
}

#[test]
fn test_hello() { let (s, c) = run_example("hello"); assert_eq!(s, "Hello, World!\n"); assert_eq!(c, 0); }
#[test]
fn test_fact() { let (s, c) = run_example("fact"); assert!(s.contains("fact(10)")); assert_eq!(c, 0); }
#[test]
fn test_fib() { let (s, c) = run_example("fib"); assert!(s.contains("fibonacci(20)")); assert_eq!(c, 0); }
#[test]
fn test_sum() { let (s, c) = run_example("sum"); assert!(s.contains("sum(1..100)")); assert_eq!(c, 0); }
