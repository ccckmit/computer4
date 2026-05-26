use js4::{run, tokenize};

#[test]
fn test_tokenizer_numbers() {
    let tokens = tokenize("let x = 42;");
    assert_eq!(tokens.len(), 6);
}

#[test]
fn test_tokenizer_string() {
    let tokens = tokenize("let s = \"hello\";");
    assert_eq!(tokens.len(), 6);
}

#[test]
fn test_run_let() {
    let result = run("let x = 42;");
    assert!(result.is_ok());
}

#[test]
fn test_run_expression() {
    let result = run("1 + 2");
    assert!(result.is_ok());
}

#[test]
fn test_run_function() {
    let result = run("function add(a, b) { return a + b; }");
    assert!(result.is_ok());
}

#[test]
fn test_run_if() {
    let result = run("if (true) { let x = 1; }");
    assert!(result.is_ok());
}

#[test]
fn test_run_while() {
    let result = run("let i = 0; while (i < 5) { i = i + 1; }");
    assert!(result.is_ok());
}

#[test]
fn test_run_try_catch() {
    let result = run("try { throw \"error\"; } catch (e) { let x = 1; }");
    assert!(result.is_ok());
}

#[test]
fn test_break_continue() {
    let result = run("let sum = 0; let i = 0; while (i < 10) { i = i + 1; if (i == 3) { continue; } if (i == 8) { break; } sum = sum + i; }");
    assert!(result.is_ok());
}

#[test]
fn test_array_operations() {
    let result = run("let arr = [1, 2, 3]; arr.push(4);");
    assert!(result.is_ok());
}

#[test]
fn test_object_operations() {
    let result = run("let obj = { name: \"test\" }; obj.age = 25;");
    assert!(result.is_ok());
}