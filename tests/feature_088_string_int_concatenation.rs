// RED Phase Test for Feature #88: String + Integer automatic concatenation
//
// This addresses 4 failing book examples in Chapter 5 (Control Flow)
// that attempt to concatenate strings with integers.
//
// Error: "Type error: Cannot add string and integer"
//
// Expected behavior: Automatically convert integers to strings during
// concatenation, matching behavior of Python, JavaScript, Ruby, etc.
//
// EXTREME TDD Methodology:
// 1. RED: Create comprehensive failing tests (this file)
// 2. GREEN: Implement auto-conversion in Add operation
// 3. REFACTOR: Add property tests for all numeric types

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED: Test basic string + integer concatenation
///
/// This is the most common use case from the book examples.
#[test]
fn test_feature_088_string_plus_integer_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat.ruchy");

    let code = r#"
fun main() {
    let result = "Count: " + 42;
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    // RED: Currently fails with "Type error: Cannot add string and integer"
    // GREEN: Should output "Count: 42"
    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Count: 42"),
        "Expected 'Count: 42', got: {stdout}"
    );
}

/// RED: Test integer + string concatenation (reversed order)
///
/// Some languages support this, others don't. Let's support both orders.
#[test]
fn test_feature_088_integer_plus_string() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_rev.ruchy");

    let code = r#"
fun main() {
    let result = 42 + " items";
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("42 items"),
        "Expected '42 items', got: {stdout}"
    );
}

/// RED: Test string + negative integer
///
/// Edge case: negative numbers should include the minus sign.
#[test]
fn test_feature_088_string_plus_negative_integer() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_neg.ruchy");

    let code = r#"
fun main() {
    let result = "Temperature: " + -5;
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Temperature: -5"),
        "Expected 'Temperature: -5', got: {stdout}"
    );
}

/// RED: Test string + float concatenation
///
/// Floats should also be automatically converted to strings.
#[test]
fn test_feature_088_string_plus_float() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_float.ruchy");

    let code = r#"
fun main() {
    let result = "Price: $" + 19.99;
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Price: $19.99"),
        "Expected 'Price: $19.99', got: {stdout}"
    );
}

/// RED: Test multiple concatenations in sequence
///
/// Book examples often chain multiple concatenations.
#[test]
fn test_feature_088_multiple_concatenations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_multi.ruchy");

    let code = r#"
fun main() {
    let result = "x = " + 10 + ", y = " + 20;
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("x = 10, y = 20"),
        "Expected 'x = 10, y = 20', got: {stdout}"
    );
}

/// RED: Test concatenation in loop (from book ch05 examples)
///
/// This is the actual failing pattern from the book.
#[test]
fn test_feature_088_concatenation_in_loop() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_loop.ruchy");

    let code = r#"
fun main() {
    let mut i = 0;
    while i < 3 {
        println("Iteration: " + i);
        i = i + 1;
    }
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Iteration: 0"),
        "Expected 'Iteration: 0', got: {stdout}"
    );
    assert!(
        stdout.contains("Iteration: 1"),
        "Expected 'Iteration: 1', got: {stdout}"
    );
    assert!(
        stdout.contains("Iteration: 2"),
        "Expected 'Iteration: 2', got: {stdout}"
    );
}

/// RED: Test boolean concatenation
///
/// Booleans should also convert to "true"/"false" strings.
#[test]
fn test_feature_088_string_plus_boolean() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_bool.ruchy");

    let code = r#"
fun main() {
    let result = "Is ready: " + true;
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Is ready: true"),
        "Expected 'Is ready: true', got: {stdout}"
    );
}

/// GREEN: Verify actual string + string still works
///
/// This is a sanity check - existing functionality shouldn't break.
#[test]
fn test_feature_088_string_plus_string_still_works() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_concat_strings.ruchy");

    let code = r#"
fun main() {
    let result = "Hello, " + "World!";
    println(result);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Hello, World!"),
        "Expected 'Hello, World!', got: {stdout}"
    );
}

/// GREEN: Verify numeric addition still works
///
/// Integer + Integer should still perform mathematical addition, not concatenation.
#[test]
fn test_feature_088_integer_plus_integer_still_adds() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_add_ints.ruchy");

    let code = r"
fun main() {
    let result = 10 + 20;
    println(result);
}
";

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("30"), "Expected '30', got: {stdout}");
    assert!(!stdout.contains("1020"), "Should not concatenate integers");
}
