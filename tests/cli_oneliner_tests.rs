//! Integration tests for CLI one-liner mode

#![allow(clippy::expect_used)] // Tests can use expect

use std::process::Command;
use std::io::Write;

#[test]
fn test_eval_flag_basic() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", "2 + 2"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "4");
}

#[test]
fn test_eval_flag_println() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", r#"println("hello world")"#])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello world");
}

#[test]
fn test_eval_flag_complex_expression() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", "(10 + 5) * 2 - 3"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "27");
}

#[test]
fn test_eval_flag_string_concat() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", r#""hello" + " " + "world""#])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#""hello world""#);
}

#[test]
fn test_eval_flag_boolean() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", "true && false"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "false");
}

#[test]
fn test_eval_flag_if_expression() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", "if 5 > 3 { 100 } else { 200 }"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "100");
}

#[test]
fn test_stdin_pipe() {
    let mut child = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");
    
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(b"42 * 2\n").expect("Failed to write to stdin");
    
    let output = child.wait_with_output().expect("Failed to read output");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "84");
}

#[test]
fn test_json_output_format() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", "42", "--format", "json"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");
}

#[test]
fn test_json_output_string() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", r#""hello""#, "--format", "json"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), r#""hello""#);
}

#[test]
fn test_error_exit_code() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", "undefined_variable"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_script_file_execution() {
    use std::fs;
    let temp_file = "/tmp/test_cli_script.ruchy";
    fs::write(temp_file, "let x = 5\nlet y = 10\nprintln(x + y)").expect("Failed to write test file");
    
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", temp_file])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "15");
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_multiple_print_statements() {
    let output = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "-e", r#"{ println("line1"); println("line2"); 42 }"#])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("line1"));
    assert!(stdout.contains("line2"));
    assert!(stdout.contains("42"));
}