#![allow(missing_docs)]
//! TRANSPILER-DEFECT-012: String return type gets &str from string literal
//!
//! **Problem**: Functions with `-> String` return type return &str literals/variables
//! **Discovered**: 2025-10-31 (Remaining E0308 errors in reaper project)
//! **Severity**: CRITICAL (7 E0308 errors remaining)
//!
//! Expected: `fun get_text() -> String { "test" }` should compile
//! Actual: E0308 - expected String, found &str
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Tests Should FAIL ====================

/// Test 1: Direct string literal return with String type
#[test]
fn test_defect_012_red_direct_literal_return() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_message() -> String {
    "Hello World"
}

fun main() {
    let msg = get_message();
    println!("{}", msg);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // RED: Will fail with E0308
}

/// Test 2: String variable return with String type
#[test]
fn test_defect_012_red_variable_return() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_text() -> String {
    let result = "test data";
    result
}

fun main() {
    let text = get_text();
    println!("{}", text);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // RED: Will fail with E0308
}

/// Test 3: Baseline - String::from should work
#[test]
fn test_defect_012_baseline_string_from() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_converted() -> String {
    String::from("works")
}

fun main() {
    let s = get_converted();
    println!("{}", s);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // This should already work
}
