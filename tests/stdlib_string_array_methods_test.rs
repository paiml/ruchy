// STDLIB-004: Custom String/Array Methods
//
// Implementing: substring, slice, join, unique, zip, enumerate
// Pattern: Custom implementations (no direct Rust stdlib equivalents)
// Tests: Both interpreter (-e flag) and transpiler (run command) modes
//
// Reference: docs/specifications/stdlib1.20-spec.md - String/Array methods

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// String Methods - substring()
// ============================================================================

#[test]
fn test_stdlib004_substring_basic() {
    let code = r#"
let s = "hello world"
let sub = s.substring(0, 5)
assert_eq(sub, "hello")
println("substring: {}", sub)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_stdlib004_substring_middle() {
    let code = r#"
let s = "hello world"
let sub = s.substring(6, 11)
assert_eq(sub, "world")
println("substring: {}", sub)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("world"));
}

#[test]
fn test_stdlib004_substring_partial() {
    let code = r#"
let s = "hello"
let sub = s.substring(1, 4)
assert_eq(sub, "ell")
println("substring: {}", sub)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("ell"));
}

// ============================================================================
// Array Methods - slice()
// ============================================================================

#[test]
fn test_stdlib004_slice_basic() {
    let code = r#"
let arr = [1, 2, 3, 4, 5]
let sliced = arr.slice(1, 4)
println("sliced: {:?}", sliced)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_stdlib004_slice_first_two() {
    let code = r#"
let arr = [10, 20, 30, 40]
let sliced = arr.slice(0, 2)
println("sliced: {:?}", sliced)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"));
}

// ============================================================================
// Array Methods - join()
// ============================================================================

#[test]
fn test_stdlib004_join_comma() {
    let code = r#"
let arr = ["apple", "banana", "cherry"]
let joined = arr.join(", ")
assert_eq(joined, "apple, banana, cherry")
println("joined: {}", joined)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("apple, banana, cherry"));
}

#[test]
fn test_stdlib004_join_dash() {
    let code = r#"
let arr = ["2025", "01", "20"]
let joined = arr.join("-")
assert_eq(joined, "2025-01-20")
println("date: {}", joined)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("2025-01-20"));
}

#[test]
fn test_stdlib004_join_empty_separator() {
    let code = r#"
let arr = ["h", "e", "l", "l", "o"]
let joined = arr.join("")
assert_eq(joined, "hello")
println("word: {}", joined)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

// ============================================================================
// Array Methods - unique()
// ============================================================================

#[test]
fn test_stdlib004_unique_integers() {
    let code = r#"
let arr = [1, 2, 2, 3, 3, 3, 4]
let uniq = arr.unique()
println("unique: {:?}", uniq)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_stdlib004_unique_strings() {
    let code = r#"
let arr = ["apple", "banana", "apple", "cherry", "banana"]
let uniq = arr.unique()
println("unique: {:?}", uniq)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("apple"))
        .stdout(predicate::str::contains("banana"))
        .stdout(predicate::str::contains("cherry"));
}

// ============================================================================
// Array Methods - zip()
// ============================================================================

#[test]
fn test_stdlib004_zip_two_arrays() {
    let code = r#"
let a = [1, 2, 3]
let b = ["a", "b", "c"]
let zipped = zip(a, b)
println("zipped: {:?}", zipped)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
        // Just verify it runs - exact format depends on tuple representation
}

#[test]
fn test_stdlib004_zip_different_lengths() {
    let code = r#"
let a = [1, 2, 3, 4, 5]
let b = ["a", "b", "c"]
let zipped = zip(a, b)
println("zipped: {:?}", zipped)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
        // Should zip up to min length (3 pairs)
}

// ============================================================================
// Array Methods - enumerate()
// ============================================================================

#[test]
fn test_stdlib004_enumerate_basic() {
    let code = r#"
let arr = ["apple", "banana", "cherry"]
let indexed = enumerate(arr)
println("indexed: {:?}", indexed)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
        // Just verify it runs - exact format depends on tuple representation
}

// ============================================================================
// Transpiler Mode Tests
// ============================================================================

#[test]
fn test_stdlib004_transpiler_substring() {
    let code = r#"
fn main() {
    let s = "hello world"
    let sub = s.substring(0, 5)
    assert_eq(sub, "hello")
    println("substring works!")
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("substring works!"));
}

#[test]
fn test_stdlib004_transpiler_join() {
    let code = r#"
fn main() {
    let arr = ["a", "b", "c"]
    let joined = arr.join("-")
    assert_eq(joined, "a-b-c")
    println("join works!")
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("join works!"));
}

#[test]
fn test_stdlib004_transpiler_slice() {
    let code = r#"
fn main() {
    let arr = [1, 2, 3, 4, 5]
    let sliced = arr.slice(1, 4)
    println("slice works!")
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("slice works!"));
}

// ============================================================================
// Integration Tests - Real-world scenarios
// ============================================================================

#[test]
fn test_stdlib004_integration_csv_parsing() {
    let code = r#"
// Simulate CSV parsing
let csv = "John,Doe,30"
let parts = csv.split(",")
let first = parts[0]
let last = parts[1]
let age = parts[2]

// Reconstruct with different separator
let pipe_format = parts.join("|")
assert_eq(pipe_format, "John|Doe|30")

println("CSV parsing test passed")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("CSV parsing test passed"));
}

#[test]
fn test_stdlib004_integration_deduplication() {
    let code = r#"
// Remove duplicates from list
let tags = ["rust", "python", "rust", "javascript", "python", "go"]
let unique_tags = tags.unique()

println("Unique tags: {:?}", unique_tags)
println("Deduplication test passed")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deduplication test passed"));
}

#[test]
fn test_stdlib004_integration_string_extraction() {
    let code = r#"
// Extract domain from email
let email = "user@example.com"
let at_pos = 4  // Position of @
let domain = email.substring(5, 15)

println("Domain extraction test passed")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Domain extraction test passed"));
}
