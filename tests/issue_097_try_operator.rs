//! Issue #97: Try operator (?) implementation tests
//!
//! Tests the try operator (?) for ergonomic error propagation.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/97>
//! EXTREME TDD: These tests demonstrate the expected behavior

use assert_cmd::Command;
use predicates::prelude::*;

/// Test basic try operator with Err propagation
/// Should propagate Err through ? operator without unwrapping
#[test]
fn test_issue_097_try_operator_err_propagation() {
    let script = r#"
enum MyError {
    Failed(String),
}

fun might_fail() -> Result<i32, MyError> {
    Err(MyError::Failed("test error"))
}

fun use_try_operator() -> Result<i32, MyError> {
    let value = might_fail()?;
    Ok(value + 10)
}

fun main() {
    let result = use_try_operator();
    match result {
        Ok(v) => println("Success: {}", v),
        Err(_) => println("Error occurred"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_097_try_err.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Error occurred"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test try operator with Ok value unwrapping
/// Should unwrap Ok value and continue execution
#[test]
fn test_issue_097_try_operator_ok_unwrapping() {
    let script = r#"
enum MyError {
    Failed(String),
}

fun succeed() -> Result<i32, MyError> {
    Ok(42)
}

fun use_try_operator() -> Result<i32, MyError> {
    let value = succeed()?;
    Ok(value + 10)
}

fun main() {
    let result = use_try_operator();
    match result {
        Ok(v) => println("Success: {}", v),
        Err(_) => println("Error occurred"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_097_try_ok.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Success: 52"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test try operator chaining multiple operations
/// Should short-circuit on first error
#[test]
fn test_issue_097_try_operator_chaining() {
    let script = r#"
enum MyError {
    Failed(String),
}

fun step1() -> Result<i32, MyError> {
    Ok(10)
}

fun step2(x: i32) -> Result<i32, MyError> {
    Err(MyError::Failed("step2 failed"))
}

fun step3(x: i32) -> Result<i32, MyError> {
    Ok(x * 2)
}

fun pipeline() -> Result<i32, MyError> {
    let a = step1()?;
    let b = step2(a)?;
    let c = step3(b)?;
    Ok(c)
}

fun main() {
    let result = pipeline();
    match result {
        Ok(v) => println("Success: {}", v),
        Err(_) => println("Pipeline failed"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_097_try_chain.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Pipeline failed"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test try operator with nested Result types
/// Should work with complex nested structures
#[test]
fn test_issue_097_try_operator_nested_results() {
    let script = r#"
enum Error1 {
    Fail1,
}

enum Error2 {
    Fail2,
}

fun outer() -> Result<i32, Error1> {
    Ok(100)
}

fun use_nested() -> Result<i32, Error1> {
    let val = outer()?;
    Ok(val + 50)
}

fun main() {
    let result = use_nested();
    match result {
        Ok(v) => println("Result: {}", v),
        Err(_) => println("Failed"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_097_try_nested.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 150"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test try operator in expression context
/// Should work as part of larger expressions
#[test]
fn test_issue_097_try_operator_in_expression() {
    let script = r#"
enum MyError {
    Failed,
}

fun get_number() -> Result<i32, MyError> {
    Ok(5)
}

fun compute() -> Result<i32, MyError> {
    let result = get_number()? * 2 + 3;
    Ok(result)
}

fun main() {
    let result = compute();
    match result {
        Ok(v) => println("Computed: {}", v),
        Err(_) => println("Failed"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_097_try_expr.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Computed: 13"));

    std::fs::remove_file(temp_file).unwrap();
}
