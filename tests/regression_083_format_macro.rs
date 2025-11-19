#![allow(missing_docs)]
// Regression tests for GitHub Issue #83: Runtime error when using format! macro
// https://github.com/paiml/ruchy/issues/83
//
// REGRESSION INFO:
// - Working Version: v3.147.6 ✅
// - Broken Versions: v3.147.7, v3.147.8 ❌
// - Error: "Macro 'format!' not yet implemented"
// - Type: Standard library regression
//
// ROOT CAUSE: format! macro was never implemented (NOT a regression - missing feature)
//   - No handler for "format" in ExprKind::Macro/MacroInvocation match arms
//   - Documentation claimed it worked, but code had no implementation
//
// SOLUTION: Implemented format! macro with EXTREME TDD
//   - Added format! handler in interpreter.rs (ExprKind::Macro at line 1279)
//   - Added format! handler in interpreter.rs (ExprKind::MacroInvocation at line 1421)
//   - Supports {} placeholders for values
//   - Supports {:?} placeholders for debug formatting
//   - All 3 regression tests now pass ✅
//
// Test naming convention: test_regression_083_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Test #1: Basic format! macro (minimal reproduction from Issue #83)
/// This is the exact test case reported in the GitHub issue.
#[test]
fn test_regression_083_format_basic() {
    let code = r#"
fun main() {
    let x = 42;
    let msg = format!("Value: {}", x);
    println(msg);
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Value: 42"));
}

/// Test #2: format! with multiple placeholders
/// Verifies that format! works with multiple arguments
#[test]
fn test_regression_083_format_multiple_args() {
    let code = r#"
fun main() {
    let name = "Alice";
    let age = 30;
    let msg = format!("Name: {}, Age: {}", name, age);
    println(msg);
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Name:"))
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Age:"))
        .stdout(predicate::str::contains("30"));
}

/// Test #3: format! with no placeholders
/// Verifies that format! works with static strings
#[test]
fn test_regression_083_format_static_string() {
    let code = r#"
fun main() {
    let msg = format!("Hello, World!");
    println(msg);
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}
