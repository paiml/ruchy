/// P0-LINT: False Positive TDD Tests
/// These tests define the expected behavior for variable usage tracking
/// to eliminate false positives in the lint tool
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_fstring_interpolation_variable_usage() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test that variables used in f-string interpolations are recognized as used
    let code = r#"
let name = "World"
let greeting = f"Hello, {name}!"
println(greeting)
"#;

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_multiple_fstring_interpolations() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test multiple variables in f-strings
    let code = r#"
let first = "John"
let last = "Doe"
let age = 30
let message = f"Name: {first} {last}, Age: {age}"
println(message)
"#;

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_nested_fstring_expressions() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test expressions within f-string interpolations
    let code = r#"
let x = 5
let y = 10
let result = f"Sum: {x + y}, Product: {x * y}"
println(result)
"#;

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_function_parameter_usage_in_body() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test that function parameters used in the body are recognized
    let code = r"
fn calculate(x, y) {
    return x + y
}

let result = calculate(10, 20)
println(result)
";

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_function_parameter_in_conditional() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test parameters used in conditionals
    let code = r"
fn is_positive(n) {
    if n > 0 {
        return true
    } else {
        return false
    }
}

println(is_positive(5))
";

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_function_parameter_in_fstring() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test parameters used in f-strings within functions
    let code = r#"
fn greet(name) {
    let message = f"Hello, {name}!"
    return message
}

println(greet("Alice"))
"#;

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_lambda_parameter_usage() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test lambda parameters are recognized as used
    let code = r"
let add = |x, y| x + y
let result = add(3, 4)
println(result)
";

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_complex_fstring_with_method_calls() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test method calls on variables in f-strings
    let code = r#"
let items = [1, 2, 3]
let count = items.len()
let message = f"There are {count} items"
println(message)
"#;

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_actual_unused_variable_detection() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test that genuinely unused variables are still detected
    let code = r"
let unused = 42
let used = 10
println(used)
";

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Warning - unused variable: unused",
        ));
}

#[test]
fn test_unused_function_parameter_detection() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Test that genuinely unused parameters are NOT reported as errors
    // because they might be part of a public API
    let code = r"
fn process(x, y) {
    return x * 2  // y is not used
}

println(process(5, 10))
";

    fs::write(&file_path, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}
