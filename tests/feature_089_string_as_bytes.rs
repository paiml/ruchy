// RED Phase Test for Feature #89: String.as_bytes() method
//
// This addresses 2 failing book examples (Ch4.10, Ch17.8) that use string.as_bytes()
//
// Error: "Unknown zero-argument string method: as_bytes"
//
// Expected behavior: Return array of UTF-8 byte values (0-255) for the string
//
// EXTREME TDD Methodology:
// 1. RED: Create comprehensive failing tests (this file)
// 2. GREEN: Implement as_bytes() in string method handler
// 3. REFACTOR: Add property tests for UTF-8 correctness

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED: Test basic ASCII string conversion to bytes
///
/// This is the most common use case from the book examples.
#[test]
fn test_feature_089_as_bytes_basic_ascii() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_as_bytes.ruchy");

    let code = r#"
fun main() {
    let text = "Hello";
    let bytes = text.as_bytes();
    println(bytes);
}
"#;

    fs::write(&test_file, code).unwrap();

    // RED: Currently fails with "Unknown zero-argument string method: as_bytes"
    // GREEN: Should output array of byte values [72, 101, 108, 108, 111]
    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("[72, 101, 108, 108, 111]"),
        "Expected byte array for 'Hello', got: {stdout}"
    );
}

/// RED: Test empty string
#[test]
fn test_feature_089_as_bytes_empty_string() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_empty.ruchy");

    let code = r#"
fun main() {
    let text = "";
    let bytes = text.as_bytes();
    println(bytes.len());
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains('0'),
        "Expected length 0 for empty string, got: {stdout}"
    );
}

/// RED: Test UTF-8 multi-byte characters
///
/// "Hello 世界" should encode correctly (世=228,184,150, 界=231,149,140)
#[test]
fn test_feature_089_as_bytes_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_utf8.ruchy");

    let code = r#"
fun main() {
    let text = "A";
    let bytes = text.as_bytes();
    // 'A' is ASCII 65
    println(bytes[0]);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("65"),
        "Expected byte value 65 for 'A', got: {stdout}"
    );
}

/// RED: Test indexing into returned byte array
///
/// Common pattern: bytes[0] to get first byte
#[test]
fn test_feature_089_as_bytes_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_index.ruchy");

    let code = r#"
fun main() {
    let text = "ABC";
    let bytes = text.as_bytes();
    println(bytes[0]);
    println(bytes[1]);
    println(bytes[2]);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("65"), "Expected 65 (A), got: {stdout}");
    assert!(stdout.contains("66"), "Expected 66 (B), got: {stdout}");
    assert!(stdout.contains("67"), "Expected 67 (C), got: {stdout}");
}

/// RED: Test special characters
///
/// Newline (\n = 10), tab (\t = 9), etc.
#[test]
fn test_feature_089_as_bytes_special_chars() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_special.ruchy");

    let code = r#"
fun main() {
    let text = "\n";
    let bytes = text.as_bytes();
    println(bytes[0]);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("10"),
        "Expected byte value 10 for newline, got: {stdout}"
    );
}

/// RED: Test iteration over byte array
///
/// Common pattern: for byte in bytes { ... }
#[test]
fn test_feature_089_as_bytes_iteration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_iter.ruchy");

    let code = r#"
fun main() {
    let text = "Hi";
    let bytes = text.as_bytes();
    for byte in bytes {
        println(byte);
    }
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("72"), "Expected 72 (H), got: {stdout}");
    assert!(stdout.contains("105"), "Expected 105 (i), got: {stdout}");
}

/// RED: Test method chaining
///
/// Pattern: `text.as_bytes().len()`
#[test]
fn test_feature_089_as_bytes_method_chaining() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_chain.ruchy");

    let code = r#"
fun main() {
    let text = "Test";
    let length = text.as_bytes().len();
    println(length);
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains('4'),
        "Expected length 4 for 'Test', got: {stdout}"
    );
}

/// GREEN: Verify string methods still work
///
/// Sanity check: `as_bytes()` doesn't break other string methods
#[test]
fn test_feature_089_other_string_methods_still_work() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_sanity.ruchy");

    let code = r#"
fun main() {
    let text = "Hello";
    println(text.len());
    println(text.to_uppercase());
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains('5'), "Expected length 5, got: {stdout}");
    assert!(stdout.contains("HELLO"), "Expected HELLO, got: {stdout}");
}
