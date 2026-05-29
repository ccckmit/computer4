use std::process::Command;

/// Find the path to a built tool binary.
fn tool_path(name: &str) -> String {
    // Try CARGO_BIN_EXE_ first (set by cargo for integration tests)
    let var_name = format!("CARGO_BIN_EXE_{}", name);
    if let Ok(path) = std::env::var(&var_name) {
        return path;
    }

    // Fallback: look in target directory relative to crate root
    let target_dir = if cfg!(debug_assertions) { "debug" } else { "release" };
    // Try workspace root (cargo run from workspace)
    let cwd = std::env::current_dir().unwrap();
    let candidates = [
        cwd.join("target").join(target_dir).join(name),
        cwd.join("target").join(target_dir).join(&format!("{}.exe", name)),
        cwd.parent().unwrap().join("target").join(target_dir).join(name),
    ];

    for p in &candidates {
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
    }

    // Last resort: assume it's in PATH
    name.to_string()
}

#[test]
fn test_true_exit_code() {
    let out = Command::new(tool_path("true")).output().unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn test_false_exit_code() {
    let out = Command::new(tool_path("false")).output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn test_echo_default() {
    let out = Command::new(tool_path("echo")).arg("hello").arg("world").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello world\n");
}

#[test]
fn test_echo_no_newline() {
    let out = Command::new(tool_path("echo")).arg("-n").arg("hello").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello");
}

#[test]
fn test_echo_multiple_args() {
    let out = Command::new(tool_path("echo")).arg("a").arg("b").arg("c").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a b c\n");
}

#[test]
fn test_cat_stdin() {
    let mut child = Command::new(tool_path("cat"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"hello\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
}

#[test]
fn test_cat_file() {
    let dir = std::env::temp_dir().join("posix_test_cat");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test.txt");
    std::fs::write(&path, b"line1\nline2\nline3\n").unwrap();

    let out = Command::new(tool_path("cat"))
        .arg(path.to_string_lossy().as_ref())
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "line1\nline2\nline3\n");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_wc_default() {
    let mut child = Command::new(tool_path("wc"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"hello world\nfoo bar baz\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(output.trim().starts_with("2"), "expected 2 lines, got: {}", output);
}

#[test]
fn test_wc_lines_flag() {
    let mut child = Command::new(tool_path("wc"))
        .arg("-l")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"a\nb\nc\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(output.trim().starts_with("3"), "expected 3 lines, got: {}", output);
}

#[test]
fn test_wc_words() {
    let mut child = Command::new(tool_path("wc"))
        .arg("-w")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"one two three four five\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(output.trim().starts_with("5"), "expected 5 words, got: {}", output);
}

#[test]
fn test_wc_chars() {
    let mut child = Command::new(tool_path("wc"))
        .arg("-m")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // "hello" is 5 chars
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"hello").unwrap();
    let out = child.wait_with_output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(output.trim().starts_with("5"), "expected 5 chars, got: {}", output);
}

#[test]
fn test_wc_bytes() {
    let mut child = Command::new(tool_path("wc"))
        .arg("-c")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"hello\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(output.trim().starts_with("6"), "expected 6 bytes, got: {}", output);
}

#[test]
fn test_wc_mutually_exclusive() {
    // -l, -w, -c should be mutually exclusive (last one wins in our impl)
    let mut child = Command::new(tool_path("wc"))
        .arg("-c").arg("-l").arg("-w")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"a b c\nd e f\n").unwrap();
    let out = child.wait_with_output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    // With -w, only word count should show
    assert!(output.trim().starts_with("6"), "expected 6 words, got: {}", output);
}

#[test]
fn test_basename_simple() {
    let out = Command::new(tool_path("basename")).arg("/usr/bin/file.txt").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "file.txt");
}

#[test]
fn test_basename_no_extension() {
    let out = Command::new(tool_path("basename")).arg("/foo/bar/baz").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "baz");
}

#[test]
fn test_basename_with_suffix() {
    let out = Command::new(tool_path("basename")).arg("/dir/file.txt").arg(".txt").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "file");
}

#[test]
fn test_basename_no_path() {
    let out = Command::new(tool_path("basename")).arg("foo").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "foo");
}

#[test]
fn test_dirname_simple() {
    let out = Command::new(tool_path("dirname")).arg("/usr/bin/file.txt").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "/usr/bin");
}

#[test]
fn test_dirname_root() {
    let out = Command::new(tool_path("dirname")).arg("/").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "/");
}

#[test]
fn test_dirname_relative() {
    let out = Command::new(tool_path("dirname")).arg("foo/bar").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "foo");
}

#[test]
fn test_dirname_single() {
    let out = Command::new(tool_path("dirname")).arg("foo").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), ".");
}

#[test]
fn test_sleep_zero() {
    let start = std::time::Instant::now();
    let out = Command::new(tool_path("sleep")).arg("0").output().unwrap();
    let elapsed = start.elapsed();
    assert!(out.status.success());
    assert!(elapsed.as_secs() < 1, "sleep 0 took too long: {:?}", elapsed);
}

#[test]
fn test_sleep_fractional() {
    let out = Command::new(tool_path("sleep")).arg("0.01").output().unwrap();
    assert!(out.status.success());
}

#[test]
fn test_uname_default() {
    let out = Command::new(tool_path("uname")).output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!output.is_empty(), "uname should output something");
}

#[test]
fn test_uname_machine() {
    let out = Command::new(tool_path("uname")).arg("-m").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!output.is_empty(), "uname -m should output machine arch");
}

#[test]
fn test_uname_all() {
    let out = Command::new(tool_path("uname")).arg("-a").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(!output.trim().is_empty(), "uname -a should output something");
    // -a should have multiple fields (space-separated)
    assert!(output.trim().contains(' '), "uname -a should have multiple fields: got '{}'", output.trim());
}

#[test]
fn test_uname_node() {
    let out = Command::new(tool_path("uname")).arg("-n").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!output.is_empty(), "uname -n should output hostname");
}

#[test]
fn test_printenv_var() {
    let out = Command::new(tool_path("printenv")).arg("PATH").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!output.is_empty(), "PATH should be set");
    assert!(output.contains('/'), "PATH should contain slashes");
}

#[test]
fn test_printenv_missing_var() {
    let out = Command::new(tool_path("printenv"))
        .arg("__POSIX_TEST_NONEXISTENT_VAR_12345")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "", "missing var should produce no output");
}

#[test]
fn test_printenv_multiple_vars() {
    let out = Command::new(tool_path("printenv")).arg("HOME").arg("USER").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2, "should print HOME and USER on separate lines");
}

#[test]
fn test_hostname() {
    let out = Command::new(tool_path("hostname")).output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!output.is_empty(), "hostname should not be empty");
}

#[test]
fn test_whoami() {
    let out = Command::new(tool_path("whoami")).output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!output.is_empty(), "whoami should output a username");
}

#[test]
fn test_id_user() {
    let out = Command::new(tool_path("id")).arg("-u").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let uid: u32 = output.parse().expect("id -u should output a numeric UID");
    assert!(uid > 0, "UID should be > 0");
}

#[test]
fn test_id_group() {
    let out = Command::new(tool_path("id")).arg("-g").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let gid: u32 = output.parse().expect("id -g should output a numeric GID");
    assert!(gid > 0, "GID should be > 0");
}

#[test]
fn test_id_real_user() {
    let out = Command::new(tool_path("id")).arg("-ur").output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let uid: u32 = output.parse().expect("id -ur should output numeric UID");
    assert!(uid > 0);
}

#[test]
fn test_env_print() {
    let out = Command::new(tool_path("env")).output().unwrap();
    let output = String::from_utf8_lossy(&out.stdout);
    assert!(output.contains("PATH="), "env should print PATH");
}

#[test]
fn test_env_var_assign() {
    // Test that env with VAR=value sets it for the child
    let out = Command::new(tool_path("env"))
        .arg("TEST_VAR=hello")
        .arg("sh").arg("-c").arg("echo $TEST_VAR")
        .output()
        .unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(output, "hello", "env should pass TEST_VAR=hello");
}

#[test]
fn test_env_unset() {
    // Test -u option removes a variable.
    // Use a custom var to avoid shell defaults.
    let out = Command::new(tool_path("env"))
        .arg("-u").arg("MY_TEST_VAR")
        .env("MY_TEST_VAR", "should_not_appear")
        .arg("sh").arg("-c").arg(r"echo ${MY_TEST_VAR:-unset}")
        .output()
        .unwrap();
    let output = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(output, "unset", "env -u should unset MY_TEST_VAR");
}

#[test]
fn test_yes_default() {
    let mut child = Command::new(tool_path("yes"))
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Read;
    let mut buf = [0u8; 10];
    child.stdout.take().unwrap().read_exact(&mut buf).unwrap();
    child.kill().unwrap();
    assert_eq!(&buf, b"y\ny\ny\ny\ny\n");
}

#[test]
fn test_yes_custom_string() {
    let mut child = Command::new(tool_path("yes")).arg("hello")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Read;
    let mut buf = [0u8; 18];
    child.stdout.take().unwrap().read_exact(&mut buf).unwrap();
    child.kill().unwrap();
    assert_eq!(&buf, b"hello\nhello\nhello\n");
}

#[test]
fn test_yes_multiple_args() {
    let mut child = Command::new(tool_path("yes")).arg("hello").arg("world")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Read;
    let mut buf = [0u8; 24];
    child.stdout.take().unwrap().read_exact(&mut buf).unwrap();
    child.kill().unwrap();
    assert_eq!(&buf, b"hello world\nhello world\n");
}
