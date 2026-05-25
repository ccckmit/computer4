use std::fs;
use std::path::Path;

fn run_ir(path: &str) -> String {
    let source = fs::read_to_string(Path::new(path)).unwrap();
    lli4::interpret(&source)
}

#[test]
fn test_add() {
    let out = run_ir("examples_ll/add.ir");
    assert_eq!(out.trim(), "7");
}

#[test]
fn test_fact() {
    let out = run_ir("examples_ll/fact.ir");
    assert_eq!(out.trim(), "120");
}

#[test]
fn test_fib() {
    let out = run_ir("examples_ll/fib.ir");
    assert_eq!(out.trim(), "55");
}

#[test]
fn test_gcd() {
    let out = run_ir("examples_ll/gcd.ir");
    assert_eq!(out.trim(), "6");
}

#[test]
fn test_if_else() {
    let out = run_ir("examples_ll/if_else.ir");
    assert_eq!(out.trim(), "1");
}

#[test]
fn test_is_prime() {
    let out = run_ir("examples_ll/is_prime.ir");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["1", "0", "1"]);
}

#[test]
fn test_hello_world() {
    let ir = r#"
define i32 @main() {
  %0 = add i32 0, 42
  call void @print_int(i32 %0)
  ret i32 0
}
"#;
    let out = lli4::interpret(ir);
    assert_eq!(out.trim(), "42");
}

#[test]
fn test_multi_block() {
    let ir = r#"
define i32 @main() {
entry:
  %0 = add i32 0, 5
  %1 = add i32 0, 7
  %2 = add i32 %0, %1
  call void @print_int(i32 %2)
  ret i32 0
}
"#;
    let out = lli4::interpret(ir);
    assert_eq!(out.trim(), "12");
}

#[test]
fn test_cond_branch() {
    let ir = r#"
define i32 @main() {
entry:
  %0 = add i32 0, 10
  %1 = add i32 0, 5
  %2 = icmp sgt i32 %0, %1
  br i1 %2, label %then, label %else
then:
  %3 = add i32 0, 1
  call void @print_int(i32 %3)
  ret i32 0
else:
  %4 = add i32 0, 0
  call void @print_int(i32 %4)
  ret i32 0
}
"#;
    let out = lli4::interpret(ir);
    assert_eq!(out.trim(), "1");
}
