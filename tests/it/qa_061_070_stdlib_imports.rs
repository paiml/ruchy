//! QA-061 through QA-070: Standard Library Import Tests
//!
//! EXTREME TDD - RED Phase
//!
//! Critical Bug: `use std::math` fails with "Undefined variable: math"
//! Root Cause: std.math module not registered in builtin_init.rs
//!
//! Reference: docs/specifications/100-point-qa-beta-checklist-4.0-beta.md [QA-061-070]

#![allow(deprecated)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::if_not_else)]

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to run ruchy and return stdout
fn run_ruchy(code: &str) -> String {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(code.as_bytes()).expect("write code");

    let output = Command::cargo_bin("ruchy")
        .expect("ruchy binary")
        .arg("run")
        .arg(file.path())
        .output()
        .expect("run ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        format!("ERROR: {stderr}\nSTDOUT: {stdout}")
    } else {
        stdout
    }
}

/// RED TEST 1: std::math module should be accessible
#[test]
fn test_qa_065_red_01_math_sqrt() {
    let code = r#"
use std::math
let result = math.sqrt(16.0)
print(result)
"#;
    let result = run_ruchy(code);

    // Should print 4.0 (sqrt of 16)
    assert!(
        result.contains("4") && !result.contains("ERROR"),
        "math.sqrt(16.0) should return 4.0. Got: {result}"
    );
}

/// RED TEST 2: std::math sin function
#[test]
fn test_qa_065_red_02_math_sin() {
    let code = r#"
use std::math
let result = math.sin(0.0)
print(result)
"#;
    let result = run_ruchy(code);

    // sin(0) = 0
    assert!(
        result.contains("0") && !result.contains("ERROR"),
        "math.sin(0.0) should return 0. Got: {result}"
    );
}

/// RED TEST 3: std::math cos function
#[test]
fn test_qa_065_red_03_math_cos() {
    let code = r#"
use std::math
let result = math.cos(0.0)
print(result)
"#;
    let result = run_ruchy(code);

    // cos(0) = 1
    assert!(
        result.contains("1") && !result.contains("ERROR"),
        "math.cos(0.0) should return 1. Got: {result}"
    );
}

/// RED TEST 4: std::math pow function
#[test]
fn test_qa_065_red_04_math_pow() {
    let code = r#"
use std::math
let result = math.pow(2.0, 3.0)
print(result)
"#;
    let result = run_ruchy(code);

    // 2^3 = 8
    assert!(
        result.contains("8") && !result.contains("ERROR"),
        "math.pow(2.0, 3.0) should return 8.0. Got: {result}"
    );
}

/// RED TEST 5: std::math abs function
#[test]
fn test_qa_065_red_05_math_abs() {
    let code = r#"
use std::math
let result = math.abs(-42.0)
print(result)
"#;
    let result = run_ruchy(code);

    // abs(-42) = 42
    assert!(
        result.contains("42") && !result.contains("ERROR"),
        "math.abs(-42.0) should return 42.0. Got: {result}"
    );
}

/// RED TEST 6: std::math floor function
#[test]
fn test_qa_065_red_06_math_floor() {
    let code = r#"
use std::math
let result = math.floor(3.7)
print(result)
"#;
    let result = run_ruchy(code);

    // floor(3.7) = 3
    assert!(
        result.contains("3") && !result.contains("ERROR"),
        "math.floor(3.7) should return 3.0. Got: {result}"
    );
}

/// RED TEST 7: std::math ceil function
#[test]
fn test_qa_065_red_07_math_ceil() {
    let code = r#"
use std::math
let result = math.ceil(3.2)
print(result)
"#;
    let result = run_ruchy(code);

    // ceil(3.2) = 4
    assert!(
        result.contains("4") && !result.contains("ERROR"),
        "math.ceil(3.2) should return 4.0. Got: {result}"
    );
}

/// RED TEST 8: std::math log function (natural log)
#[test]
fn test_qa_065_red_08_math_log() {
    let code = r#"
use std::math
let result = math.log(2.718281828)
print(result)
"#;
    let result = run_ruchy(code);

    // ln(e) ≈ 1
    assert!(
        !result.contains("ERROR"),
        "math.log should not error. Got: {result}"
    );
}

/// RED TEST 9: Direct std access without import
#[test]
fn test_qa_065_red_09_direct_std_access() {
    let code = r#"
let result = std.math.sqrt(9.0)
print(result)
"#;
    let result = run_ruchy(code);

    // Should work with direct access too
    assert!(
        result.contains("3") && !result.contains("ERROR"),
        "std.math.sqrt(9.0) should return 3.0. Got: {result}"
    );
}

/// RED TEST 10: Verify std namespace exists
#[test]
fn test_qa_065_red_10_std_namespace() {
    let code = r#"
print(std)
"#;
    let result = run_ruchy(code);

    // std should be an object containing modules
    assert!(
        !result.contains("Undefined variable: std"),
        "std namespace should exist. Got: {result}"
    );
}
