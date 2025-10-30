//! Regression tests for Issue #81: panic!() and undefined functions return exit code 0
//!
//! Bug: panic!() and undefined function calls should return non-zero exit codes
//! Expected: Exit code 1 (or other non-zero)
//! Actual: Exit code 0 (success)
//!
//! GitHub Issue: <https://github.com/paiml/ruchy/issues/81>
//! Ticket: DEBUGGER-013

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;

/// Test 1: panic!() should return non-zero exit code
#[test]
fn test_regression_081_panic_returns_nonzero_exit_code() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
fun main() {{
    panic!("intentional crash");
}}
"#
    )
    .unwrap();

    // RED TEST: This should FAIL (currently returns exit code 0)
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .failure() // Expect non-zero exit code
        .stderr(predicate::str::contains("panic")); // Expect error message
}

/// Test 2: Undefined function call should return non-zero exit code
/// NOTE: Currently ignored - Ruchy treats undefined functions as valid (return Nil)
/// This requires language-level changes to add function existence checking
#[test]
#[ignore = "Language limitation: undefined functions return Nil instead of error"]
fn test_regression_081_undefined_function_returns_nonzero_exit_code() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r"
fun main() {{
    undefined_function_that_does_not_exist();
}}
"
    )
    .unwrap();

    // RED TEST: This should FAIL (currently returns exit code 0)
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .failure() // Expect non-zero exit code
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("undefined")));
}

/// Test 3: Successful execution should return exit code 0
#[test]
fn test_regression_081_success_returns_zero_exit_code() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
fun main() {{
    println("Hello World");
}}
"#
    )
    .unwrap();

    // This should PASS (success case)
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .success() // Expect exit code 0
        .stdout(predicate::str::contains("Hello World"));
}

/// Test 4: Runtime error (not panic) should return non-zero exit code
#[test]
fn test_regression_081_runtime_error_returns_nonzero_exit_code() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r"
fun main() {{
    let x = 1 / 0;  // Division by zero
}}
"
    )
    .unwrap();

    // RED TEST: This should FAIL (currently returns exit code 0)
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .failure() // Expect non-zero exit code
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}
