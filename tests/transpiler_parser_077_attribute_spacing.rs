//! PARSER-077: Attribute Spacing Bug - #[test] Emits with Space as # [test]
//!
//! **Problem**: #[test] attributes transpile with incorrect spacing: # [test] instead of #[test]
//! **Discovered**: 2025-10-23 (GitHub Issue #58 investigation)
//! **Severity**: HIGH - Breaks Rust compilation, #[test] is invalid syntax
//!
//! Expected: `#[test]` with no space
//! Actual: `# [test]` with space between # and [
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use predicates::prelude::*;
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

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Simple #[test] attribute should transpile without spaces
///
/// This is the PRIMARY test for PARSER-077
#[test]
fn test_parser_077_red_simple_test_attribute() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
#[test]
fun foo() {
    42
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Transpile and check output contains #[test] without space
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .output()
        .expect("Failed to execute ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // CRITICAL ASSERTIONS:
    // 1. Must contain #[test] (no space)
    // 2. Must NOT contain # [test] (with space)

    assert!(
        stdout.contains("#[test]"),
        "Expected #[test] without space, but got: {}",
        stdout
    );

    assert!(
        !stdout.contains("# [test]"),
        "Found # [test] with space (BUG!), output: {}",
        stdout
    );
}

/// Test 2: Multiple #[test] attributes
#[test]
fn test_parser_077_red_multiple_test_attributes() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
#[test]
fun test_one() { 1 }

#[test]
fun test_two() { 2 }

#[test]
fun test_three() { 3 }
"#;

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .output()
        .expect("Failed to execute ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // All three #[test] attributes must have correct spacing
    let test_count = stdout.matches("#[test]").count();
    let bad_test_count = stdout.matches("# [test]").count();

    assert_eq!(
        test_count, 3,
        "Expected 3 #[test] attributes, found {}, output: {}",
        test_count, stdout
    );

    assert_eq!(
        bad_test_count, 0,
        "Found {} # [test] with spaces (BUG!), output: {}",
        bad_test_count, stdout
    );
}

/// Test 3: #[derive(...)] attribute should also have correct spacing
#[test]
fn test_parser_077_red_derive_attribute() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
#[derive(Debug, Clone)]
struct Point {
    x: i32,
    y: i32
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .output()
        .expect("Failed to execute ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // #[derive(...)] must have correct spacing
    assert!(
        stdout.contains("#[derive("),
        "Expected #[derive( without space, but got: {}",
        stdout
    );

    assert!(
        !stdout.contains("# [derive("),
        "Found # [derive( with space (BUG!), output: {}",
        stdout
    );
}

/// Test 4: Compile full example - ensures transpiled code is valid Rust
#[test]
fn test_parser_077_red_compile_with_test_attribute() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
#[test]
fun test_addition() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // This should compile successfully if spacing is correct
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 5: Edge case - attribute at start of file (no leading whitespace)
#[test]
fn test_parser_077_red_attribute_at_file_start() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    // No newline before #[test] - edge case
    let code = r#"#[test]
fun foo() { 42 }
"#;

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .output()
        .expect("Failed to execute ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("#[test]"),
        "Expected #[test] without space, but got: {}",
        stdout
    );

    assert!(
        !stdout.contains("# [test]"),
        "Found # [test] with space (BUG!), output: {}",
        stdout
    );
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document all failing cases
#[test]
fn test_parser_077_red_phase_summary() {
    println!("PARSER-077 RED Phase:");
    println!("- 5 tests created that WILL FAIL due to spacing bug");
    println!("");
    println!("Expected failures:");
    println!("1. Simple #[test] attribute has space: # [test]");
    println!("2. Multiple #[test] attributes all have spaces");
    println!("3. #[derive(...)] attribute has space: # [derive(");
    println!("4. Compile fails because # [test] is invalid Rust syntax");
    println!("5. Edge case: attribute at file start also has space");
    println!("");
    println!("Root Cause: TokenStream spacing issue in transpiler");
    println!("Next: GREEN phase - fix TokenStream generation with Spacing::Joint");
}
