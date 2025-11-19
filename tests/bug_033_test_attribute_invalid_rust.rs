#![allow(missing_docs)]
//! BUG-033: @test("description") Transpiles to Invalid Rust
//!
//! **Problem**: @test("description") transpiles to #[test(description)] which is invalid Rust
//! **Discovered**: GitHub Issue #33
//! **Severity**: MEDIUM - Breaks `ruchy property-tests` command
//!
//! **Expected**: Either strip description (#[test]) or use doc comment
//! **Actual**: Generates #[test(description)] which fails Rust compilation
//!
//! **Root Cause**: Transpiler blindly copies attribute arguments without validating
//! Rust's #[test] attribute takes NO arguments
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

/// Test 1: @test with description should compile
#[test]
fn test_bug_033_red_test_with_description() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
@test("simple addition test")
fun test_addition() {
    assert_eq(1 + 1, 2)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // RED: This will FAIL - generates invalid Rust #[test(description)]
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 2: @test without description should work (baseline)
#[test]
fn test_bug_033_baseline_test_without_description() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
@test
fun test_subtraction() {
    assert_eq(2 - 1, 1)
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // This should already work (baseline test)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 3: @test with complex description
#[test]
fn test_bug_033_red_test_with_complex_description() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
@test("tests multiplication with multiple cases and edge conditions")
fun test_multiplication() {
    assert_eq(2 * 3, 6)
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

/// Test 4: Multiple @test functions with descriptions
#[test]
fn test_bug_033_red_multiple_tests() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
@test("test addition")
fun test_add() {
    assert_eq(1 + 1, 2)
}

@test("test subtraction")
fun test_sub() {
    assert_eq(2 - 1, 1)
}

@test
fun test_mult() {
    assert_eq(2 * 3, 6)
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

/// Test 5: Verify transpiled output strips description
#[test]
fn test_bug_033_red_transpiled_output_format() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
@test("my test description")
fun test_example() {
    assert_eq(1, 1)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Transpile and check output
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let transpiled = String::from_utf8_lossy(&output);

    // Should generate #[test] or # [test] (with formatting spaces), NOT #[test(description)]
    assert!(
        transpiled.contains("#[test]") || transpiled.contains("# [test]"),
        "Transpiled code should contain #[test] attribute, found: {transpiled}"
    );

    // Should NOT contain invalid syntax with arguments
    assert!(
        !transpiled.contains("#[test(")
            && !transpiled.contains("#[test (")
            && !transpiled.contains("# [test ("),
        "Transpiled code should NOT contain #[test(description)] - found: {transpiled}"
    );
}

/// Test 6: property-tests command should work
#[test]
fn test_bug_033_red_property_tests_command() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
@test("test with description")
fun test_example() {
    assert_eq(1, 1)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // RED: This is the canonical failing case from the bug report
    ruchy_cmd()
        .arg("property-tests")
        .arg(&source)
        .assert()
        .success();
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document the RED phase
#[test]
fn test_bug_033_red_phase_summary() {
    println!("BUG-033 RED Phase: @test(description) Invalid Rust");
    println!();
    println!("Problem: @test(\"description\") → #[test(description)] (invalid)");
    println!("Impact: Breaks ruchy property-tests command");
    println!();
    println!("Test Suite Created:");
    println!("1. @test with description - compile");
    println!("2. @test without description - baseline");
    println!("3. @test with complex description");
    println!("4. Multiple @test functions");
    println!("5. Verify transpiled output format");
    println!("6. property-tests command works");
    println!();
    println!("Expected Results:");
    println!("- RED Phase: Tests 1, 3-6 FAIL (invalid Rust)");
    println!("- RED Phase: Test 2 PASS (baseline)");
    println!("- GREEN Phase: ALL tests PASS after fix");
    println!();
    println!("Fix Strategy:");
    println!("- Modify format_regular_attribute() to strip args when name == \"test\"");
    println!("- Optional: Add description as doc comment /// <description>");
}
