#![allow(missing_docs)]
// Tests for LINT-008: Variables used in format! macros marked as unused (Issue #8)
// GitHub Issue: https://github.com/paiml/ruchy/issues/8
//
// Test naming convention: test_lint_008_<scenario>

use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test #1: Variable used in format! macro should NOT be marked unused
#[test]
fn test_lint_008_format_macro_single_variable() {
    let code = r#"
fun main() {
    let name = "Alice"
    println(format!("Hello, {}", name))
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(&test_file)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("unused variable: name").not());
}

/// Test #2: Multiple variables in format! macro should NOT be marked unused
#[test]
fn test_lint_008_format_macro_multiple_variables() {
    let code = r#"
fun main() {
    let name = "Alice"
    let age = 25
    println(format!("{} is {} years old", name, age))
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(&test_file)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("unused variable: name").not())
        .stdout(predicate::str::contains("unused variable: age").not());
}

/// Test #3: Format string with expressions (ensure we handle complex cases)
#[test]
fn test_lint_008_format_macro_with_expressions() {
    let code = r#"
fun main() {
    let x = 10
    let y = 32
    println(format!("Sum: {}", x + y))
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(&test_file)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("unused variable: x").not())
        .stdout(predicate::str::contains("unused variable: y").not());
}

/// Test #4: Truly unused variable should still be detected
#[test]
fn test_lint_008_truly_unused_variable_still_detected() {
    let code = r#"
fun main() {
    let unused = 42
    let name = "Alice"
    println(format!("Hello, {}", name))
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(&test_file)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("unused variable: unused"))
        .stdout(predicate::str::contains("unused variable: name").not());
}

/// Test #5: format! return value assigned to variable and used
#[test]
fn test_lint_008_format_result_used() {
    let code = r#"
fun main() {
    let name = "Alice"
    let greeting = format!("Hello, {}", name)
    println(greeting)
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(&test_file)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("unused variable: name").not())
        .stdout(predicate::str::contains("unused variable: greeting").not());
}
