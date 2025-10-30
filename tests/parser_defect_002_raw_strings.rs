//! DEFECT-PARSER-002: Raw String Literals Not Supported
//!
//! ROOT CAUSE: Parser/lexer doesn't recognize r#"..."# syntax
//! Error: "Expected `RightBrace`, found Let"
//!
//! Impact: 15 failing book examples (highest frequency parser error)
//!
//! RED Phase Tests - Following EXTREME TDD

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_parser_002_raw_string_minimal() {
    // Minimal raw string literal - DEFECT-PARSER-002
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    // Use r## to allow r# inside
    let code = r##"
fun main() {
    let x = r#"test"#;
    println(x);
}
"##;

    fs::write(&source, code).expect("Failed to write test file");

    // Should parse successfully (RED phase - will fail)
    ruchy_cmd()
        .arg("check")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_parser_002_raw_string_with_quotes() {
    // Raw string containing quotes - common use case
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r##"
fun main() {
    let json = r#"{"name": "Alice", "age": 25}"#;
    println(json);
}
"##;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("check")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_parser_002_raw_string_multiline() {
    // Raw string with multiline content
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r##"
fun main() {
    let data = r#"[
        {"name": "Alice"},
        {"name": "Bob"}
    ]"#;
    println(data);
}
"##;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("check")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_parser_002_raw_string_with_backslashes() {
    // Raw string with backslashes (no escaping)
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r##"
fun main() {
    let path = r#"C:\Users\Alice\Documents\file.txt"#;
    println(path);
}
"##;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("check")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_parser_002_regular_string_still_works() {
    // Control test: regular strings should still work
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let x = "normal string";
    println(x);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("check")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_parser_002_book_example_dataframe() {
    // Real failing book example from ch18-dataframes
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r##"
fun test_dataframe_from_json() {
    let json_data = r#"[
        {"name": "Alice", "age": 25, "active": true},
        {"name": "Bob", "age": 30, "active": false}
    ]"#;

    let df = DataFrame::from_json(json_data);
    println("Loaded {} rows from JSON", df.rows());
}
"##;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("check")
        .arg(&source)
        .assert()
        .success();
}
