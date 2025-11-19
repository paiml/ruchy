#![allow(missing_docs)]
//! Regression tests for Issue #80: Ruchy doesn't support stdin input
//!
//! Bug: `ruchy run -` should read from stdin (Unix convention)
//! Expected: Code from stdin is executed
//! Actual: Error: -: No such file or directory
//!
//! GitHub Issue: <https://github.com/paiml/ruchy/issues/80>
//! Ticket: DEBUGGER-013

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

/// Test 1: `ruchy run -` should execute code from stdin
#[test]
fn test_regression_080_stdin_with_dash_argument() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg("-")
        .write_stdin("fun main() { println(\"Hello from stdin\"); }")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from stdin"));
}

/// Test 2: `ruchy run -` should handle syntax errors in stdin
#[test]
fn test_regression_080_stdin_syntax_error() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg("-")
        .write_stdin("fun main() { invalid syntax }")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

/// Test 3: `ruchy run -` should handle empty stdin gracefully
/// Note: Empty programs are syntax errors in Ruchy, which is reasonable
#[test]
fn test_regression_080_stdin_empty() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg("-")
        .write_stdin("")
        .assert()
        .failure() // Empty program is a syntax error
        .stderr(predicate::str::contains("Empty program"));
}

/// Test 4: `ruchy -e` should still work (not affected by stdin support)
#[test]
fn test_regression_080_eval_flag_still_works() {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg("println(\"Hello from -e\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from -e"));
}

/// Test 5: `ruchy run <file>` should still work (not affected by stdin support)
#[test]
fn test_regression_080_file_argument_still_works() {
    use tempfile::NamedTempFile;
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "fun main() {{ println(\"Hello from file\"); }}").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from file"));
}
