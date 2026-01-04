#![allow(missing_docs)]
//! TRANSPILER-DEFECT-013: Vec/Array indexing with String return type
//!
//! **Problem**: Functions with `-> String` return type that return Vec[index] get E0308
//! **Discovered**: 2025-10-31 (Remaining E0308 errors after DEFECT-012)
//! **Severity**: HIGH (2+ E0308 errors remaining in reaper project)
//!
//! Expected: `fun get() -> String { data[0] }` should compile
//! Actual: E0308 - expected String, found &str
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Tests Should FAIL ====================

/// Test 1: Direct Vec index return with String type
#[test]
fn test_defect_013_vec_index_return() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_first() -> String {
    let items = vec!["one", "two", "three"];
    items[0]
}

fun main() {
    let first = get_first();
    println!("{}", first);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // Should compile with .to_string() wrapper
}

/// Test 2: Vec index in variable return with String type
#[test]
fn test_defect_013_vec_index_variable_return() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_element() -> String {
    let data = vec!["alpha", "beta", "gamma"];
    let elem = data[0];
    elem
}

fun main() {
    let e = get_element();
    println!("{}", e);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // Should compile with .to_string() wrapper
}

/// Test 3: Baseline - `String::from` with Vec index should work
#[test]
fn test_defect_013_baseline_string_from_vec() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_converted() -> String {
    let items = vec!["test1", "test2"];
    String::from(items[0])
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
