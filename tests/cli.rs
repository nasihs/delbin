/// CLI integration tests — written BEFORE implementation (RED phase).
use std::io::Write;
use std::process::{Command, Stdio};

fn run_delbin(dsl: &str, extra_args: &[&str]) -> (i32, String, String) {
    let (code, raw_out, raw_err) = run_delbin_raw(dsl, extra_args);
    (
        code,
        String::from_utf8_lossy(&raw_out).into_owned(),
        String::from_utf8_lossy(&raw_err).into_owned(),
    )
}

fn run_delbin_raw(dsl: &str, extra_args: &[&str]) -> (i32, Vec<u8>, Vec<u8>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_delbin"))
        .arg("-") // read DSL from stdin
        .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn delbin binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(dsl.as_bytes())
        .unwrap();

    let out = child.wait_with_output().unwrap();
    let code = out.status.code().unwrap_or(-1);
    (code, out.stdout, out.stderr)
}

#[test]
fn test_cli_hex_output_to_stdout() {
    let dsl = "@endian = little; struct h @packed { val: u8 = 0xAB; }";
    let (code, stdout, stderr) = run_delbin(dsl, &[]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.trim().eq_ignore_ascii_case("ab"),
        "Expected 'AB', got: '{}'",
        stdout.trim()
    );
}

#[test]
fn test_cli_env_var_substitution() {
    let dsl = "@endian = little; struct h @packed { val: u32 = ${VER}; }";
    let (code, stdout, _) = run_delbin(dsl, &["--env", "VER=42"]);
    assert_eq!(code, 0);
    // 42 as LE u32 = 0x0000_002A
    assert!(
        stdout.trim().eq_ignore_ascii_case("2a000000"),
        "Expected '2A000000', got: '{}'",
        stdout.trim()
    );
}

#[test]
fn test_cli_invalid_dsl_exits_nonzero() {
    let (code, _, stderr) = run_delbin("this is not valid DSL", &[]);
    assert_ne!(code, 0, "invalid DSL should exit non-zero");
    assert!(!stderr.is_empty(), "error message should go to stderr");
}

#[test]
fn test_cli_verbose_prints_warnings_to_stderr() {
    // 0x1FF doesn't fit in u8 → W03002 warning
    let dsl = "@endian = little; struct h @packed { v: u8 = 0x1FF; }";
    let (code, _, stderr) = run_delbin(dsl, &["--verbose"]);
    assert_eq!(code, 0);
    assert!(
        stderr.to_lowercase().contains("w03002") || stderr.to_lowercase().contains("truncat"),
        "Expected truncation warning in stderr, got: {}",
        stderr
    );
}

#[test]
fn test_cli_bin_format_writes_binary() {
    let dsl = "@endian = little; struct h @packed { val: u8 = 0xAB; }";
    let (code, stdout_bytes, _) = run_delbin_raw(dsl, &["--format", "bin"]);
    assert_eq!(code, 0);
    assert_eq!(stdout_bytes, b"\xAB", "binary output should be raw byte 0xAB");
}
