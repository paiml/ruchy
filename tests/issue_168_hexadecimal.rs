//! EXTREME TDD Tests for Issue #168: Hexadecimal number support
//!
//! Feature: Support hex literals like 0xFF, 0x1A2B, 0xDEADBEEF

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// RED PHASE: These tests should FAIL before implementation
// ============================================================================

#[test]
fn test_issue_168_01_basic_hex() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let x = 0xFF;
    println!("{}", x)
}
"#,
    )
    .unwrap();

    // Should run and output 255
    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("255"),
        "Expected 255 from 0xFF, got: {stdout}"
    );
}

#[test]
fn test_issue_168_02_lowercase_hex() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let x = 0xff;
    println!("{}", x)
}
"#,
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("255"),
        "Expected 255 from 0xff, got: {stdout}"
    );
}

#[test]
fn test_issue_168_03_uppercase_hex() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let x = 0x1A2B;
    println!("{}", x)
}
"#,
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("6699"),
        "Expected 6699 from 0x1A2B, got: {stdout}"
    );
}

#[test]
fn test_issue_168_04_hex_arithmetic() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let a = 0x10;
    let b = 0x20;
    let c = a + b;
    println!("{}", c)
}
"#,
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 0x10 = 16, 0x20 = 32, sum = 48
    assert!(
        stdout.contains("48"),
        "Expected 48 from 0x10 + 0x20, got: {stdout}"
    );
}

#[test]
fn test_issue_168_05_hex_transpile() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let x = 0xDEAD;
    println!("{}", x)
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should transpile hex to Rust hex or decimal
    assert!(
        output.contains("0xDEAD") || output.contains("57005"),
        "Expected hex literal in output: {output}"
    );
}

#[test]
fn test_issue_168_06_hex_bitwise_operations() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let mask = 0xFF;
    let value = 0x1234;
    let result = value & mask;
    println!("{}", result)
}
"#,
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 0x1234 & 0xFF = 0x34 = 52
    assert!(
        stdout.contains("52"),
        "Expected 52 from 0x1234 & 0xFF, got: {stdout}"
    );
}
