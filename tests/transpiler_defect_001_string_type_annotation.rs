#![allow(missing_docs)]
//! TRANSPILER-DEFECT-001: String Type Annotations Don't Auto-Convert
//!
//! **Problem**: String literals with String type annotations don't auto-convert
//! **Discovered**: 2025-10-07 (LANG-COMP-007 session)
//! **Severity**: HIGH
//!
//! Expected: `let name: String = "Alice"` should work
//! Actual: Compilation fails with "expected String, found &str"
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

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Simple string literal with String type annotation
///
/// This test should PASS now that transpiler fix is implemented.
#[test]
fn test_defect_001_green_string_literal_with_type_annotation() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let name: String = "Alice";
    println(name);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Compile should succeed (currently fails)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // Currently FAILS: "expected String, found &str"
}

/// Test 2: Multiple string variables with String type annotations
#[test]
fn test_defect_001_green_multiple_string_annotations() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let first: String = "Alice";
    let last: String = "Smith";
    let full = first + " " + last;
    println(full);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 3: String type annotation in function parameter
#[test]
fn test_defect_001_green_function_parameter_string_annotation() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun greet(name: String) {
    println("Hello, " + name);
}

fun main() {
    greet("Alice");
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 4: String type annotation with f-string
#[test]
fn test_defect_001_green_fstring_with_string_annotation() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let greeting: String = f"Hello, world!";
    println(greeting);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 5: Workaround validation - manual .`to_string()` should still work
#[test]
fn test_defect_001_workaround_manual_to_string() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let name: String = "Alice".to_string();
    println(name);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // This workaround should work NOW (validation test)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 6: Type inference without annotation should still work
#[test]
fn test_defect_001_baseline_type_inference_works() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let name = "Alice";
    println(name);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Type inference should work NOW (baseline test)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document all failing cases
#[test]
fn test_defect_001_red_phase_summary() {
    // This test documents the RED phase state
    println!("TRANSPILER-DEFECT-001 RED Phase:");
    println!("- 4 tests created that WILL FAIL when un-ignored");
    println!("- 2 baseline tests that pass NOW (workaround + type inference)");
    println!();
    println!("Expected failures:");
    println!("1. String literal with String type annotation");
    println!("2. Multiple string variables with annotations");
    println!("3. Function parameter with String annotation");
    println!("4. F-string with String annotation");
    println!();
    println!("Next: GREEN phase - fix transpiler to auto-convert");
}
