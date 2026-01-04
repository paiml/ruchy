#![allow(missing_docs)]
//! TRANSPILER-DEFECT-014: Vec indexing moves non-Copy types (E0507)
//!
//! **Problem**: Vec indexing with non-Copy types causes E0507 move errors
//! **Discovered**: 2025-10-31 (Final errors in reaper project after DEFECT-012/013)
//! **Severity**: HIGH (2 E0507 errors remaining)
//!
//! Expected: `let x = vec_of_structs[0]` should auto-clone
//! Actual: E0507 - cannot move out of index of Vec<T>
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

/// Test 1: Vec indexing with non-Copy struct should auto-clone
#[test]
#[ignore = "RED: This test MUST fail until fix is implemented"]
fn test_defect_014_red_vec_index_non_copy_struct() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
struct Config {
    name: String,
    value: i32,
}

fun get_first_config() -> Config {
    let configs = vec![
        Config { name: String::from("first"), value: 1 },
        Config { name: String::from("second"), value: 2 }
    ];
    configs[0]
}

fun main() {
    let cfg = get_first_config();
    println!("{}", cfg.name);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // RED: Will fail with E0507
}

/// Test 2: Vec indexing in variable assignment should auto-clone
#[test]
#[ignore = "RED: This test MUST fail until fix is implemented"]
fn test_defect_014_red_vec_index_variable_assignment() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
struct Data {
    text: String,
}

fun process_data() {
    let items = vec![
        Data { text: String::from("hello") },
        Data { text: String::from("world") }
    ];

    let first = items[0];
    println!("{}", first.text);
}

fun main() {
    process_data();
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // RED: Will fail with E0507
}

/// Test 3: Baseline - Copy types should work without changes
#[test]
fn test_defect_014_baseline_copy_types_work() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_first_number() -> i32 {
    let numbers = vec![1, 2, 3];
    numbers[0]
}

fun main() {
    let num = get_first_number();
    println!("{}", num);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // This should already work (i32 is Copy)
}

/// Test 4: Baseline - Auto-derived Clone should work with explicit .`clone()`
#[test]
fn test_defect_014_baseline_explicit_clone_works() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
struct Config {
    name: String,
}

fun get_config() -> Config {
    let configs = vec![Config { name: String::from("test") }];
    configs[0].clone()
}

fun main() {
    let cfg = get_config();
    println!("{}", cfg.name);
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
