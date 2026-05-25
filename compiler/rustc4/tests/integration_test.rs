use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn compile_and_run(source: &str) -> String {
    let ir = rustc4::compile(source);

    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir();
    let ir_path = dir.join(format!("test_rustc4_{}.ll", id));

    std::fs::write(&ir_path, &ir).unwrap();

    // Validate with llc
    let llc_result = Command::new("llc")
        .args(&["-o", "/dev/null", ir_path.to_str().unwrap()])
        .output()
        .expect("llc executable not found");

    if !llc_result.status.success() {
        let stderr = String::from_utf8_lossy(&llc_result.stderr);
        std::fs::remove_file(&ir_path).ok();
        panic!("llc validation failed:\nIR:\n{}\n\nError:\n{}", ir, stderr);
    }

    // Run with lli
    let lli_output = Command::new("lli")
        .arg(ir_path.to_str().unwrap())
        .output()
        .expect("lli executable not found");

    std::fs::remove_file(&ir_path).ok();

    if !lli_output.status.success() {
        let stderr = String::from_utf8_lossy(&lli_output.stderr);
        panic!("lli execution failed:\nIR:\n{}\n\nError:\n{}", ir, stderr);
    }

    String::from_utf8_lossy(&lli_output.stdout).to_string()
}

#[test]
fn test_add() {
    let source = r#"
fn add(x: i32, y: i32) -> i32 {
    return x + y;
}

fn main() -> i32 {
    let a: i32 = 3;
    let b: i32 = 4;
    let c: i32 = add(a, b);
    print_int(c);
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "7");
}

#[test]
fn test_fact() {
    let source = r#"
fn main() -> i32 {
    let n: i32 = 5;
    let mut i: i32 = 1;
    let mut result: i32 = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    print_int(result);
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "120");
}

#[test]
fn test_fib() {
    let source = r#"
fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

fn main() -> i32 {
    print_int(fib(10));
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "55");
}

#[test]
fn test_gcd() {
    let source = r#"
fn gcd(a: i32, b: i32) -> i32 {
    let mut x: i32 = a;
    let mut y: i32 = b;
    while y != 0 {
        let t: i32 = y;
        y = x % y;
        x = t;
    }
    return x;
}

fn main() -> i32 {
    print_int(gcd(48, 18));
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "6");
}

#[test]
fn test_is_prime() {
    let source = r#"
fn is_prime(n: i32) -> i32 {
    if n < 2 {
        return 0;
    }
    let mut i: i32 = 2;
    while i * i <= n {
        if n % i == 0 {
            return 0;
        }
        i = i + 1;
    }
    return 1;
}

fn main() -> i32 {
    print_int(is_prime(17));
    print_int(is_prime(4));
    print_int(is_prime(2));
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "1\n0\n1");
}

#[test]
fn test_if_else() {
    let source = r#"
fn main() -> i32 {
    let x: i32 = 10;
    if x > 5 {
        print_int(1);
    } else {
        print_int(0);
    }
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_while_loop() {
    let source = r#"
fn main() -> i32 {
    let mut i: i32 = 0;
    while i < 3 {
        print_int(i);
        i = i + 1;
    }
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "0\n1\n2");
}

#[test]
fn test_nested_if() {
    let source = r#"
fn main() -> i32 {
    let x: i32 = 42;
    if x > 100 {
        print_int(0);
    } else if x > 10 {
        print_int(1);
    } else {
        print_int(2);
    }
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_unary_neg() {
    let source = r#"
fn main() -> i32 {
    let x: i32 = 5;
    let y: i32 = -x;
    print_int(y);
    return 0;
}
"#;
    let output = compile_and_run(source);
    assert_eq!(output.trim(), "-5");
}
