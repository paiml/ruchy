#![allow(missing_docs)]
//! BUG-032: `range()` Function Not Transpiled
//!
//! **Problem**: range(start, end) function calls are not transpiled to Rust's (start..end) syntax
//! **Discovered**: GitHub Issue #32
//! **Severity**: HIGH - Blocks compilation to standalone binaries
//!
//! **Expected**: `range(0, 10)` should transpile to `(0..10)`
//! **Actual**: Compiler error: `cannot find function 'range' in this scope`
//!
//! **Works**: Interpreter mode (ruchy run) - has built-in `range()` function
//! **Fails**: Compilation mode (ruchy compile, ruchy fuzz) - no Rust equivalent
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use predicates::prelude::*;
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

/// Test 1: Basic `range()` function call in for loop
///
/// This is the canonical use case from GitHub Issue #32.
#[test]
fn test_bug_032_red_basic_range_in_for_loop() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    for i in range(0, 10) {
        println(i)
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // RED: This will FAIL because range() is not transpiled
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success(); // This assertion will fail in RED phase
}

/// Test 2: `range()` with variable arguments
#[test]
fn test_bug_032_red_range_with_variables() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let start = 5
    let end = 15
    for i in range(start, end) {
        println(i)
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 3: `range()` in variable assignment
#[test]
fn test_bug_032_red_range_assigned_to_variable() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let r = range(1, 5)
    for i in r {
        println(i)
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 4: `range()` in expression context
#[test]
fn test_bug_032_red_range_in_expression() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let count = range(0, 10).count()
    println(count)
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 5: Multiple `range()` calls
#[test]
fn test_bug_032_red_multiple_ranges() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    for i in range(0, 5) {
        for j in range(0, 3) {
            println(i + j)
        }
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 6: `range()` with negative numbers
#[test]
fn test_bug_032_red_range_negative_numbers() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    for i in range(-5, 5) {
        println(i)
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 7: Baseline - range syntax (1..10) already works
#[test]
fn test_bug_032_baseline_range_syntax() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    for i in 0..10 {
        println(i)
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // This should work NOW (baseline test - range syntax already transpiles)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 8: Verify `range()` works in interpreter mode (baseline)
#[test]
fn test_bug_032_baseline_interpreter_mode() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    for i in range(0, 5) {
        println(i)
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // This SHOULD work in interpreter mode (baseline)
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("4"));
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document the RED phase
#[test]
fn test_bug_032_red_phase_summary() {
    println!("BUG-032 RED Phase: range() Function Not Transpiled");
    println!();
    println!("Problem: range(start, end) function calls not transpiled to (start..end)");
    println!("Impact: Blocks compilation to standalone binaries");
    println!();
    println!("Test Suite Created:");
    println!("1. Basic range() in for loop (canonical case)");
    println!("2. range() with variable arguments");
    println!("3. range() assigned to variable");
    println!("4. range() in expression context (.count())");
    println!("5. Multiple/nested range() calls");
    println!("6. range() with negative numbers");
    println!("7. Baseline: range syntax (0..10) - already works");
    println!("8. Baseline: range() in interpreter mode - already works");
    println!();
    println!("Expected Results:");
    println!("- RED Phase: Tests 1-6 FAIL (compilation errors)");
    println!("- RED Phase: Tests 7-8 PASS (baseline validation)");
    println!("- GREEN Phase: ALL tests PASS after fix");
    println!();
    println!("Next Step: Implement try_transpile_range_function() handler");
}
