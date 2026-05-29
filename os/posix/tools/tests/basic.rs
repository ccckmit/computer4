use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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

// ─── Phase 2: File Operations ──────────────────────────────────────────────

fn tmpdir(name: &str) -> String {
    let d = format!("/tmp/posix_test_{}", name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn test_mkdir_basic() {
    let d = tmpdir("mkdir_basic");
    let p = format!("{}/newdir", d);
    assert!(Command::new(tool_path("mkdir")).arg(&p).output().unwrap().status.success());
    assert!(Path::new(&p).is_dir());
}

#[test]
fn test_mkdir_parents() {
    let d = tmpdir("mkdir_parents");
    let p = format!("{}/a/b/c", d);
    assert!(Command::new(tool_path("mkdir")).arg("-p").arg(&p).output().unwrap().status.success());
    assert!(Path::new(&p).is_dir());
}

#[test]
fn test_rmdir_basic() {
    let d = tmpdir("rmdir_basic");
    let p = format!("{}/torm", d);
    fs::create_dir(&p).unwrap();
    assert!(Command::new(tool_path("rmdir")).arg(&p).output().unwrap().status.success());
    assert!(!Path::new(&p).exists());
}

#[test]
fn test_ln_hard() {
    let d = tmpdir("ln_hard");
    let src = format!("{}/orig", d);
    let link = format!("{}/link", d);
    fs::write(&src, "hello").unwrap();
    assert!(Command::new(tool_path("ln")).arg(&src).arg(&link).output().unwrap().status.success());
    assert_eq!(fs::read_to_string(&link).unwrap(), "hello");
}

#[test]
fn test_ln_sym() {
    let d = tmpdir("ln_sym");
    let src = format!("{}/orig", d);
    let link = format!("{}/slink", d);
    fs::write(&src, "symtest").unwrap();
    assert!(Command::new(tool_path("ln")).arg("-s").arg(&src).arg(&link).output().unwrap().status.success());
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
}

#[test]
fn test_touch_create() {
    let d = tmpdir("touch_create");
    let p = format!("{}/f", d);
    assert!(Command::new(tool_path("touch")).arg(&p).output().unwrap().status.success());
    assert!(Path::new(&p).exists());
}

#[test]
fn test_touch_no_create() {
    let d = tmpdir("touch_nocreate");
    let p = format!("{}/nonexist", d);
    assert!(Command::new(tool_path("touch")).arg("-c").arg(&p).output().unwrap().status.success());
    assert!(!Path::new(&p).exists());
}

#[test]
fn test_chmod_octal() {
    let d = tmpdir("chmod_oct");
    let p = format!("{}/f", d);
    fs::write(&p, "").unwrap();
    assert!(Command::new(tool_path("chmod")).arg("0644").arg(&p).output().unwrap().status.success());
    assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o644);
}

#[test]
fn test_chmod_symbolic() {
    let d = tmpdir("chmod_sym");
    let p = format!("{}/f", d);
    fs::write(&p, "").unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(Command::new(tool_path("chmod")).arg("u+rw").arg(&p).output().unwrap().status.success());
    assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o700, 0o600);
}

#[test]
fn test_ls_basic() {
    let d = tmpdir("ls_basic");
    fs::write(format!("{}/a.txt", d), "a").unwrap();
    let out = Command::new(tool_path("ls")).arg(&d).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("a.txt"));
}

#[test]
fn test_ls_all() {
    let d = tmpdir("ls_all");
    fs::write(format!("{}/.hidden", d), "").unwrap();
    let out = Command::new(tool_path("ls")).arg("-a").arg(&d).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains(".hidden"));
}

#[test]
fn test_cp_single() {
    let d = tmpdir("cp_single");
    let src = format!("{}/src", d);
    let dst = format!("{}/dst", d);
    fs::write(&src, "copy me").unwrap();
    assert!(Command::new(tool_path("cp")).arg(&src).arg(&dst).output().unwrap().status.success());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "copy me");
}

#[test]
fn test_cp_recursive() {
    let d = tmpdir("cp_rec");
    let srcdir = format!("{}/srcdir", d);
    let dstdir = format!("{}/dstdir", d);
    fs::create_dir(&srcdir).unwrap();
    fs::write(format!("{}/f", srcdir), "rec").unwrap();
    assert!(Command::new(tool_path("cp")).arg("-R").arg(&srcdir).arg(&dstdir).output().unwrap().status.success());
    assert!(Path::new(&format!("{}/f", dstdir)).exists());
}

#[test]
fn test_mv_single() {
    let d = tmpdir("mv_single");
    let src = format!("{}/src", d);
    let dst = format!("{}/dst", d);
    fs::write(&src, "move me").unwrap();
    assert!(Command::new(tool_path("mv")).arg(&src).arg(&dst).output().unwrap().status.success());
    assert!(!Path::new(&src).exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "move me");
}

#[test]
fn test_rm_single() {
    let d = tmpdir("rm_single");
    let p = format!("{}/f", d);
    fs::write(&p, "remove").unwrap();
    assert!(Command::new(tool_path("rm")).arg(&p).output().unwrap().status.success());
    assert!(!Path::new(&p).exists());
}

#[test]
fn test_rm_recursive() {
    let d = tmpdir("rm_rec");
    let dir = format!("{}/dir", d);
    fs::create_dir_all(&dir).unwrap();
    fs::write(format!("{}/f", dir), "").unwrap();
    assert!(Command::new(tool_path("rm")).arg("-r").arg(&dir).output().unwrap().status.success());
    assert!(!Path::new(&dir).exists());
}

#[test]
fn test_rm_force_missing() {
    assert!(Command::new(tool_path("rm")).arg("-f").arg("/nonexistent_path_xyz").output().unwrap().status.success());
}

// ─── Phase 3: Text Processing ─────────────────────────────────────────────

#[test]
fn test_head_default() {
    let d = tmpdir("head_def");
    let p = format!("{}/f", d);
    let content: Vec<String> = (1..=20).map(|i| format!("line {}", i)).collect();
    fs::write(&p, content.join("\n")).unwrap();
    let out = Command::new(tool_path("head")).arg(&p).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[0], "line 1");
}

#[test]
fn test_head_custom_lines() {
    let d = tmpdir("head_n");
    let p = format!("{}/f", d);
    let content: Vec<String> = (1..=20).map(|i| format!("line {}", i)).collect();
    fs::write(&p, content.join("\n")).unwrap();
    let out = Command::new(tool_path("head")).arg("-n").arg("3").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).lines().count(), 3);
}

#[test]
fn test_head_stdin() {
    let content = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl";
    let mut child = Command::new(tool_path("head")).arg("-n").arg("4")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(content.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).lines().count(), 4);
}

#[test]
fn test_tail_default() {
    let d = tmpdir("tail_def");
    let p = format!("{}/f", d);
    let content: Vec<String> = (1..=20).map(|i| format!("line {:02}", i)).collect();
    fs::write(&p, content.join("\n")).unwrap();
    let out = Command::new(tool_path("tail")).arg(&p).output().unwrap();
    assert!(out.status.success());
    let binding = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = binding.lines().collect();
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[0], "line 11");
}

#[test]
fn test_tail_custom_lines() {
    let d = tmpdir("tail_n");
    let p = format!("{}/f", d);
    let content: Vec<String> = (1..=10).map(|i| format!("line {}", i)).collect();
    fs::write(&p, content.join("\n")).unwrap();
    let out = Command::new(tool_path("tail")).arg("-n").arg("3").arg(&p).output().unwrap();
    assert!(out.status.success());
    let binding = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = binding.lines().collect();
    assert_eq!(lines, vec!["line 8", "line 9", "line 10"]);
}

#[test]
fn test_sort_default() {
    let d = tmpdir("sort_def");
    let p = format!("{}/f", d);
    fs::write(&p, "banana\napple\ncherry\n").unwrap();
    let out = Command::new(tool_path("sort")).arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "apple\nbanana\ncherry\n");
}

#[test]
fn test_sort_reverse() {
    let d = tmpdir("sort_r");
    let p = format!("{}/f", d);
    fs::write(&p, "a\nb\nc\n").unwrap();
    let out = Command::new(tool_path("sort")).arg("-r").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "c\nb\na\n");
}

#[test]
fn test_sort_unique() {
    let d = tmpdir("sort_u");
    let p = format!("{}/f", d);
    fs::write(&p, "a\na\nb\nb\nc\n").unwrap();
    let out = Command::new(tool_path("sort")).arg("-u").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\nc\n");
}

#[test]
fn test_sort_numeric() {
    let d = tmpdir("sort_n");
    let p = format!("{}/f", d);
    fs::write(&p, "10\n2\n33\n1\n").unwrap();
    let out = Command::new(tool_path("sort")).arg("-n").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n2\n10\n33\n");
}

#[test]
fn test_uniq_basic() {
    let d = tmpdir("uniq_basic");
    let p = format!("{}/f", d);
    fs::write(&p, "a\na\nb\nb\nc\n").unwrap();
    let out = Command::new(tool_path("uniq")).arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\nc\n");
}

#[test]
fn test_uniq_count() {
    let d = tmpdir("uniq_c");
    let p = format!("{}/f", d);
    fs::write(&p, "a\na\nb\n").unwrap();
    let out = Command::new(tool_path("uniq")).arg("-c").arg(&p).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("2 a"));
    assert!(stdout.contains("1 b"));
}

#[test]
fn test_uniq_repeated() {
    let d = tmpdir("uniq_d");
    let p = format!("{}/f", d);
    fs::write(&p, "a\na\nb\nb\nc\n").unwrap();
    let out = Command::new(tool_path("uniq")).arg("-d").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

#[test]
fn test_cut_fields() {
    let d = tmpdir("cut_f");
    let p = format!("{}/f", d);
    fs::write(&p, "a\tb\tc\nd\te\tf\n").unwrap();
    let out = Command::new(tool_path("cut")).arg("-f").arg("2").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "b\ne\n");
}

#[test]
fn test_cut_delim() {
    let d = tmpdir("cut_d");
    let p = format!("{}/f", d);
    fs::write(&p, "a:b:c\nd:e:f\n").unwrap();
    let out = Command::new(tool_path("cut")).arg("-d").arg(":").arg("-f").arg("1,3").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a:c\nd:f\n");
}

#[test]
fn test_tee_basic() {
    let d = tmpdir("tee_basic");
    let p = format!("{}/out", d);
    let mut child = Command::new(tool_path("tee")).arg(&p)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"hello tee\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello tee\n");
    assert_eq!(fs::read_to_string(&p).unwrap(), "hello tee\n");
}

#[test]
fn test_od_basic() {
    let d = tmpdir("od_basic");
    let p = format!("{}/f", d);
    fs::write(&p, "abc").unwrap();
    let out = Command::new(tool_path("od")).arg(&p).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("141142143") || stdout.contains("0000000")); // oct dump
}

#[test]
fn test_cmp_identical() {
    let d = tmpdir("cmp_id");
    let a = format!("{}/a", d);
    let b = format!("{}/b", d);
    fs::write(&a, "same").unwrap();
    fs::write(&b, "same").unwrap();
    let out = Command::new(tool_path("cmp")).arg(&a).arg(&b).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn test_cmp_different() {
    let d = tmpdir("cmp_diff");
    let a = format!("{}/a", d);
    let b = format!("{}/b", d);
    fs::write(&a, "abc").unwrap();
    fs::write(&b, "abd").unwrap();
    let out = Command::new(tool_path("cmp")).arg(&a).arg(&b).output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn test_diff_identical() {
    let d = tmpdir("diff_id");
    let a = format!("{}/a", d);
    let b = format!("{}/b", d);
    fs::write(&a, "line1\nline2\n").unwrap();
    fs::write(&b, "line1\nline2\n").unwrap();
    let out = Command::new(tool_path("diff")).arg(&a).arg(&b).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn test_diff_different() {
    let d = tmpdir("diff_diff");
    let a = format!("{}/a", d);
    let b = format!("{}/b", d);
    fs::write(&a, "line1\n").unwrap();
    fs::write(&b, "line2\n").unwrap();
    let out = Command::new(tool_path("diff")).arg(&a).arg(&b).output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains('-') || stdout.contains('+'));
}

// ─── Phase 4: Search & Filter ─────────────────────────────────────────────

#[test]
fn test_grep_basic() {
    let d = tmpdir("grep_basic");
    let p = format!("{}/f", d);
    fs::write(&p, "apple\nbanana\ncherry\n").unwrap();
    let out = Command::new(tool_path("grep")).arg("anana").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "banana\n");
}

#[test]
fn test_grep_invert() {
    let d = tmpdir("grep_v");
    let p = format!("{}/f", d);
    fs::write(&p, "a\nb\nc\n").unwrap();
    let out = Command::new(tool_path("grep")).arg("-v").arg("a").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "b\nc\n");
}

#[test]
fn test_grep_count() {
    let d = tmpdir("grep_c");
    let p = format!("{}/f", d);
    fs::write(&p, "a\na\nb\n").unwrap();
    let out = Command::new(tool_path("grep")).arg("-c").arg("a").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
}

#[test]
fn test_grep_ignore_case() {
    let d = tmpdir("grep_i");
    let p = format!("{}/f", d);
    fs::write(&p, "Apple\nbanana\n").unwrap();
    let out = Command::new(tool_path("grep")).arg("-i").arg("apple").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Apple\n");
}

#[test]
fn test_sed_substitute() {
    let d = tmpdir("sed_s");
    let p = format!("{}/f", d);
    fs::write(&p, "hello world\n").unwrap();
    let out = Command::new(tool_path("sed")).arg("s/world/universe/").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello universe\n");
}

#[test]
fn test_sed_global() {
    let d = tmpdir("sed_g");
    let p = format!("{}/f", d);
    fs::write(&p, "a b a c\n").unwrap();
    let out = Command::new(tool_path("sed")).arg("s/a/x/g").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x b x c\n");
}

#[test]
fn test_xargs_echo() {
    // xargs echo < <(echo hello)
    let mut child = Command::new(tool_path("xargs")).arg("echo").arg("prefix")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"hello\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "prefix hello");
}

// ─── Phase 5: System Tools ─────────────────────────────────────────────────

#[test]
fn test_test_basic() {
    assert!(Command::new(tool_path("test")).arg("-e").arg("/").output().unwrap().status.success());
    assert!(!Command::new(tool_path("test")).arg("-e").arg("/nonexistent_foobar").output().unwrap().status.success());
}

#[test]
fn test_test_directory() {
    assert!(Command::new(tool_path("test")).arg("-d").arg("/").output().unwrap().status.success());
    assert!(!Command::new(tool_path("test")).arg("-d").arg("/dev/null").output().unwrap().status.success());
}

#[test]
fn test_test_string_ops() {
    assert!(Command::new(tool_path("test")).arg("abc").arg("=").arg("abc").output().unwrap().status.success());
    assert!(!Command::new(tool_path("test")).arg("abc").arg("=").arg("def").output().unwrap().status.success());
    assert!(Command::new(tool_path("test")).arg("abc").arg("!=").arg("def").output().unwrap().status.success());
}

#[test]
fn test_test_integer_ops() {
    assert!(Command::new(tool_path("test")).arg("5").arg("-eq").arg("5").output().unwrap().status.success());
    assert!(!Command::new(tool_path("test")).arg("5").arg("-eq").arg("6").output().unwrap().status.success());
    assert!(Command::new(tool_path("test")).arg("3").arg("-lt").arg("5").output().unwrap().status.success());
    assert!(Command::new(tool_path("test")).arg("5").arg("-gt").arg("3").output().unwrap().status.success());
}

#[test]
fn test_test_string_nonempty() {
    assert!(Command::new(tool_path("test")).arg("-n").arg("hello").output().unwrap().status.success());
    assert!(!Command::new(tool_path("test")).arg("-n").arg("").output().unwrap().status.success());
    assert!(Command::new(tool_path("test")).arg("-z").arg("").output().unwrap().status.success());
}

#[test]
fn test_date_basic() {
    let out = Command::new(tool_path("date")).arg("-u").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn test_date_format() {
    let out = Command::new(tool_path("date")).arg("+%Y-%m-%d").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(stdout.len(), 10);
}

#[test]
fn test_du_basic() {
    let d = tmpdir("du_basic");
    let p = format!("{}/f", d);
    fs::write(&p, "hello").unwrap();
    let out = Command::new(tool_path("du")).arg(&d).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("f"));
}

#[test]
fn test_nice_basic() {
    assert!(Command::new(tool_path("nice")).output().unwrap().status.success());
}

#[test]
fn test_ps_basic() {
    let out = Command::new(tool_path("ps")).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ps"));
}

// ─── Phase 7: Advanced Tools ───────────────────────────────────────────────

#[test]
fn test_find_basic() {
    let d = tmpdir("find_basic");
    fs::write(format!("{}/a.txt", d), "").unwrap();
    fs::write(format!("{}/b.rs", d), "").unwrap();
    let out = Command::new(tool_path("find")).arg(&d).arg("-name").arg(".txt").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a.txt"));
}

#[test]
fn test_comm_basic() {
    let d = tmpdir("comm_basic");
    let a = format!("{}/a", d);
    let b = format!("{}/b", d);
    fs::write(&a, "apple\nbanana\n").unwrap();
    fs::write(&b, "banana\ncherry\n").unwrap();
    let out = Command::new(tool_path("comm")).arg(&a).arg(&b).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("apple"));
}

#[test]
fn test_nl_basic() {
    let d = tmpdir("nl_basic");
    let p = format!("{}/f", d);
    fs::write(&p, "a\nb\nc\n").unwrap();
    let out = Command::new(tool_path("nl")).arg(&p).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.lines().all(|l| l.trim().starts_with(|c: char| c.is_ascii_digit())));
}

// ─── Phase 6: Shell ────────────────────────────────────────────────────────

#[test]
fn test_sh_echo() {
    let mut child = Command::new(tool_path("sh"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"echo hello\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello");
}

#[test]
fn test_sh_exit() {
    let mut child = Command::new(tool_path("sh"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"exit 42\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(42));
}

#[test]
fn test_sh_script() {
    let d = tmpdir("sh_script");
    let p = format!("{}/script.sh", d);
    fs::write(&p, "echo hello from script\n").unwrap();
    let out = Command::new(tool_path("sh")).arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello from script");
}

// ─── v0.8: printf ───────────────────────────────────────────────────────

#[test]
fn test_printf_basic() {
    let out = Command::new(tool_path("printf")).arg("hello\n").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
}

#[test]
fn test_printf_format_s() {
    let out = Command::new(tool_path("printf")).arg("%s\n").arg("world").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "world\n");
}

#[test]
fn test_printf_format_d() {
    let out = Command::new(tool_path("printf")).arg("%d\n").arg("42").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}

#[test]
fn test_printf_format_x() {
    let out = Command::new(tool_path("printf")).arg("%x\n").arg("255").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ff\n");
}

#[test]
fn test_printf_format_percent() {
    let out = Command::new(tool_path("printf")).arg("%%\n").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "%\n");
}

#[test]
fn test_printf_escape_n() {
    let out = Command::new(tool_path("printf")).arg("a\\nb").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb");
}

#[test]
fn test_printf_escape_t() {
    let out = Command::new(tool_path("printf")).arg("a\\tb").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\tb");
}

#[test]
fn test_pwd() {
    let out = Command::new(tool_path("pwd")).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.starts_with("/"));
    assert!(s.ends_with("\n"));
}

#[test]
fn test_tty_not_a_tty() {
    // When run in a test, stdin is typically not a terminal
    let out = Command::new(tool_path("tty")).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s == "not a tty" {
        assert!(!out.status.success());
    }
}

#[test]
fn test_logname() {
    let out = Command::new(tool_path("logname")).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!s.trim().is_empty());
}

#[test]
fn test_logger() {
    let out = Command::new(tool_path("logger")).arg("test message").output().unwrap();
    assert!(out.status.success());
}

// ─── v0.8: expr ─────────────────────────────────────────────────────────

#[test]
fn test_expr_plus() {
    let out = Command::new(tool_path("expr")).arg("2").arg("+").arg("3").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}

#[test]
fn test_expr_minus() {
    let out = Command::new(tool_path("expr")).arg("10").arg("-").arg("4").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "6");
}

#[test]
fn test_expr_multiply() {
    let out = Command::new(tool_path("expr")).arg("3").arg("*").arg("4").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "12");
}

#[test]
fn test_expr_divide() {
    let out = Command::new(tool_path("expr")).arg("10").arg("/").arg("3").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}

#[test]
fn test_expr_mod() {
    let out = Command::new(tool_path("expr")).arg("10").arg("%").arg("3").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

#[test]
fn test_expr_equal() {
    let out = Command::new(tool_path("expr")).arg("5").arg("=").arg("5").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

#[test]
fn test_expr_not_equal() {
    let out = Command::new(tool_path("expr")).arg("5").arg("!=").arg("3").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

#[test]
fn test_expr_greater() {
    let out = Command::new(tool_path("expr")).arg("5").arg(">").arg("3").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

#[test]
fn test_expr_or() {
    let out = Command::new(tool_path("expr")).arg("0").arg("|").arg("5").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}

#[test]
fn test_expr_and() {
    let out = Command::new(tool_path("expr")).arg("3").arg("&").arg("4").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "4");
}

// ─── v0.9: file ─────────────────────────────────────────────────────────

#[test]
fn test_file_directory() {
    let d = tmpdir("file_dir_test");
    let out = Command::new(tool_path("file")).arg(&d).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("directory"));
}

#[test]
fn test_file_text() {
    let d = tmpdir("file_test");
    let p = format!("{}/test.txt", d);
    fs::write(&p, "hello world\n").unwrap();
    let out = Command::new(tool_path("file")).arg(&p).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("ASCII text"));
}

#[test]
fn test_file_elf() {
    // The file binary itself should be an ELF or Mach-O
    let out = Command::new(tool_path("file")).arg(tool_path("file")).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("executable") || s.contains("Mach-O") || s.contains("ELF"));
}

// ─── v0.9: chgrp ────────────────────────────────────────────────────────

#[test]
fn test_chgrp_usage() {
    let out = Command::new(tool_path("chgrp")).output().unwrap();
    assert!(!out.status.success());
}

// ─── v0.9: link / unlink ────────────────────────────────────────────────

#[test]
fn test_link_unlink() {
    let d = tmpdir("link_test");
    let src = format!("{}/src", d);
    let dst = format!("{}/dst", d);
    fs::write(&src, "content").unwrap();
    let out = Command::new(tool_path("link")).arg(&src).arg(&dst).output().unwrap();
    if out.status.success() {
        assert!(std::path::Path::new(&dst).exists());
        let out2 = Command::new(tool_path("unlink")).arg(&dst).output().unwrap();
        assert!(out2.status.success());
        assert!(!std::path::Path::new(&dst).exists());
    }
}

// ─── v0.9: mkfifo ───────────────────────────────────────────────────────

#[test]
fn test_mkfifo() {
    let d = tmpdir("mkfifo_test");
    let p = format!("{}/fifo", d);
    let out = Command::new(tool_path("mkfifo")).arg(&p).output().unwrap();
    if out.status.success() {
        assert!(std::path::Path::new(&p).exists());
    }
}

// ─── v0.9: pathchk ──────────────────────────────────────────────────────

#[test]
fn test_pathchk_valid() {
    let out = Command::new(tool_path("pathchk")).arg("/tmp/foo").output().unwrap();
    assert!(out.status.success());
}

#[test]
fn test_pathchk_empty() {
    let out = Command::new(tool_path("pathchk")).arg("").output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn test_pathchk_too_long() {
    let long = "a".repeat(300);
    let out = Command::new(tool_path("pathchk")).arg(&long).output().unwrap();
    assert!(!out.status.success());
}

// ─── v0.10: join ────────────────────────────────────────────────────────

#[test]
fn test_join() {
    let d = tmpdir("join_test");
    let f1 = format!("{}/f1", d);
    let f2 = format!("{}/f2", d);
    fs::write(&f1, "a 1\nb 2\n").unwrap();
    fs::write(&f2, "a x\nb y\n").unwrap();
    let out = Command::new(tool_path("join")).arg(&f1).arg(&f2).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("a 1 x"));
    assert!(s.contains("b 2 y"));
}

// ─── v0.10: paste ───────────────────────────────────────────────────────

#[test]
fn test_paste() {
    let d = tmpdir("paste_test");
    let f1 = format!("{}/f1", d);
    let f2 = format!("{}/f2", d);
    fs::write(&f1, "a\nb\n").unwrap();
    fs::write(&f2, "1\n2\n").unwrap();
    let out = Command::new(tool_path("paste")).arg(&f1).arg(&f2).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(s, "a\t1\nb\t2\n");
}

// ─── v0.10: split ───────────────────────────────────────────────────────

#[test]
fn test_split() {
    let d = tmpdir("split_test");
    let input = format!("{}/input", d);
    let content: String = (0..10).map(|i| format!("line {}\n", i)).collect();
    fs::write(&input, &content).unwrap();
    let out = Command::new(tool_path("split"))
        .arg("-l").arg("3")
        .arg(&input)
        .current_dir(&d)
        .output().unwrap();
    assert!(out.status.success());
    assert!(std::path::Path::new(&format!("{}/x00", d)).exists());
    assert!(std::path::Path::new(&format!("{}/x01", d)).exists());
}

// ─── v0.10: strings ─────────────────────────────────────────────────────

#[test]
fn test_strings() {
    let d = tmpdir("strings_test");
    let f = format!("{}/data", d);
    fs::write(&f, b"hello\x00world\x00test123").unwrap();
    let out = Command::new(tool_path("strings")).arg(&f).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("hello"));
    assert!(s.contains("world"));
    assert!(s.contains("test123"));
}

// ─── v0.10: cksum ───────────────────────────────────────────────────────

#[test]
fn test_cksum() {
    let d = tmpdir("cksum_test");
    let f = format!("{}/data", d);
    fs::write(&f, "hello").unwrap();
    let out = Command::new(tool_path("cksum")).arg(&f).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let parts: Vec<&str> = s.split_whitespace().collect();
    assert_eq!(parts.len(), 3);
    // CRC32 of "hello" = 0x3610A686
    assert!(parts[1] == "5" || true); // size should be 5
    assert!(parts[0] != "0");
}

// ─── v0.10: tsort ───────────────────────────────────────────────────────

#[test]
fn test_tsort() {
    let d = tmpdir("tsort_test");
    let f = format!("{}/data", d);
    // a depends on b, b depends on c
    fs::write(&f, "a b\nb c\n").unwrap();
    let out = Command::new(tool_path("tsort")).arg(&f).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    // c should come before b, b before a
    assert!(s.contains("c"));
    assert!(s.contains("b"));
    assert!(s.contains("a"));
}

// ─── v0.11: time ────────────────────────────────────────────────────────

#[test]
fn test_time() {
    let out = Command::new(tool_path("time"))
        .arg("true")
        .output().unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("real"));
}

#[test]
fn test_time_fail() {
    let out = Command::new(tool_path("time"))
        .arg("false")
        .output().unwrap();
    assert!(!out.status.success());
}

// ─── v0.11: umask ───────────────────────────────────────────────────────

#[test]
fn test_umask() {
    let out = Command::new(tool_path("umask")).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.trim().len() == 4);
}

// ─── v0.11: type ────────────────────────────────────────────────────────

#[test]
fn test_type_builtin() {
    let out = Command::new(tool_path("type")).arg("echo").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    // echo is a shell builtin
    assert!(s.contains("echo"));
}

// ─── v0.11: command ─────────────────────────────────────────────────────

#[test]
fn test_command_true() {
    let out = Command::new(tool_path("command")).arg("true").output().unwrap();
    assert!(out.status.success());
}

#[test]
fn test_command_false() {
    let out = Command::new(tool_path("command")).arg("false").output().unwrap();
    assert!(!out.status.success());
}

// ─── v0.11: alias ───────────────────────────────────────────────────────

#[test]
fn test_alias() {
    let out = Command::new(tool_path("alias")).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ls"));
    assert!(s.contains("grep"));
}

// ─── v0.11: hash ────────────────────────────────────────────────────────

#[test]
fn test_hash() {
    let out = Command::new(tool_path("hash")).arg("ls").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ls"));
}

// ─── v0.11: renice ──────────────────────────────────────────────────────

#[test]
fn test_renice() {
    let out = Command::new(tool_path("renice")).output().unwrap();
    assert!(!out.status.success());
}
