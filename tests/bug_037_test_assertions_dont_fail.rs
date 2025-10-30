//! BUG-037: Test Assertions Don't Fail Tests (CRITICAL)
//!
//! **Problem**: `ruchy test` reports PASS even when assertions fail
//! **Discovered**: User bug reproduction file
//! **Severity**: CRITICAL - Broken test framework undermines all QA
//!
//! **Expected**: Tests with failing assertions should report as FAILED
//! **Actual**: Tests report as PASSED even with `assert_eq(2`, 3)
//!
//! **Root Cause**: Test runner evaluates file but doesn't execute @test functions
//! Location: src/bin/handlers/handlers_modules/test_helpers.rs:87-89
//! `repl.evaluate_expr_str()` only parses/defines functions, doesn't call them
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

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Failing assertion should cause test to FAIL
#[test]
fn test_bug_037_red_failing_assertion_should_fail() {
    let temp = temp_dir();
    let test_file = temp.path().join("test_fail.ruchy");

    let code = r#"
@test("this assertion should fail")
fun test_failing_assertion() {
    let x = 1 + 1
    assert_eq(x, 3, "Expected 2 to equal 3 - this should fail")
}
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    // RED: Currently this passes when it should FAIL
    // After fix: Should report failure and exit with non-zero
    let output = ruchy_cmd()
        .arg("test")
        .arg(&test_file)
        .assert()
        .failure(); // Should fail because assertion fails

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should contain failure indicator
    assert!(
        stdout.contains("FAILED") || stdout.contains("❌"),
        "Test output should indicate failure, found: {stdout}"
    );
}

/// Test 2: Passing assertion should cause test to PASS
#[test]
fn test_bug_037_baseline_passing_assertion_passes() {
    let temp = temp_dir();
    let test_file = temp.path().join("test_pass.ruchy");

    let code = r#"
@test("this assertion should pass")
fun test_passing_assertion() {
    let x = 1 + 1
    assert_eq(x, 2, "Expected 2 to equal 2")
}
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    // Baseline: Passing tests should succeed
    ruchy_cmd()
        .arg("test")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 3: Multiple test functions - some pass, some fail
#[test]
fn test_bug_037_red_mixed_results() {
    let temp = temp_dir();
    let test_file = temp.path().join("test_mixed.ruchy");

    let code = r#"
@test("this should pass")
fun test_passing() {
    assert_eq(1, 1, "One equals one")
}

@test("this should fail")
fun test_failing() {
    assert_eq(1, 2, "One does not equal two")
}
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    // RED: Should fail because one test fails
    let output = ruchy_cmd()
        .arg("test")
        .arg(&test_file)
        .assert()
        .failure();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show 1 passed, 1 failed
    assert!(
        stdout.contains('1') && (stdout.contains("failed") || stdout.contains("FAILED")),
        "Should show test failure count, found: {stdout}"
    );
}

/// Test 4: Test without assertions should pass
#[test]
fn test_bug_037_baseline_no_assertions() {
    let temp = temp_dir();
    let test_file = temp.path().join("test_simple.ruchy");

    let code = r#"
@test("simple test without assertions")
fun test_simple() {
    let x = 1 + 1
    println(x)
}
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    // Baseline: Tests without assertions should pass
    ruchy_cmd()
        .arg("test")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 5: Verify test functions are actually executed
#[test]
fn test_bug_037_red_test_functions_execute() {
    let temp = temp_dir();
    let test_file = temp.path().join("test_execute.ruchy");
    let output_file = temp.path().join("test_output.txt");

    let code = format!(
        r#"
@test("verify execution")
fun test_writes_file() {{
    fs_write("{}", "test executed")
}}
"#,
        output_file.display().to_string().replace('\\', "\\\\")
    );

    fs::write(&test_file, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("test")
        .arg(&test_file)
        .assert()
        .success();

    // RED: File won't exist because test function never executes
    // After fix: File should exist proving function was called
    assert!(
        output_file.exists(),
        "Test function should execute and create file"
    );
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document the RED phase
#[test]
fn test_bug_037_red_phase_summary() {
    println!("BUG-037 RED Phase: Test Assertions Don't Fail Tests");
    println!();
    println!("Problem: Test runner doesn't execute @test functions");
    println!("Impact: Assertions don't cause test failures");
    println!();
    println!("Root Cause:");
    println!("- run_test_file() calls repl.evaluate_expr_str()");
    println!("- This only parses and defines functions");
    println!("- Test functions are NEVER called");
    println!("- Assertions never execute, so they can't fail");
    println!();
    println!("Test Suite Created:");
    println!("1. Failing assertion should make test fail");
    println!("2. Passing assertion should make test pass (baseline)");
    println!("3. Mixed results should report correctly");
    println!("4. Tests without assertions should pass (baseline)");
    println!("5. Test functions should actually execute");
    println!();
    println!("Expected Results:");
    println!("- RED Phase: Tests 1, 3, 5 FAIL (test framework broken)");
    println!("- RED Phase: Tests 2, 4 PASS (baseline)");
    println!("- GREEN Phase: ALL tests PASS after fix");
    println!();
    println!("Fix Strategy:");
    println!("1. Parse AST to find functions with @test attribute");
    println!("2. For each test function:");
    println!("   a. Execute the function");
    println!("   b. Catch panics/assertions");
    println!("   c. Record pass/fail");
    println!("3. Report results properly");
}
