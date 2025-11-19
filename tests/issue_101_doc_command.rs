//! Issue #101: ruchy doc command implementation tests
//!
//! Tests the `ruchy doc` command for documentation generation.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/101>
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_issue_101_doc_simple_function() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "simple.ruchy",
        r"
/// Adds two numbers together
fun add(x, y) {
    x + y
}
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated documentation"));

    // Output directory should exist
    assert!(
        output_dir.exists(),
        "Documentation directory should be created"
    );
}

#[test]
fn test_issue_101_doc_with_doc_comments() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "documented.ruchy",
        r"
/// Calculates the square of a number
///
/// # Arguments
/// * `n` - The number to square
///
/// # Returns
/// The square of the input number
fun square(n) {
    n * n
}
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.exists());
}

#[test]
fn test_issue_101_doc_multiple_functions() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "multi.ruchy",
        r"
/// Add two numbers
fun add(x, y) { x + y }

/// Subtract two numbers
fun sub(x, y) { x - y }

/// Multiply two numbers
fun mul(x, y) { x * y }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.exists());
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_issue_101_doc_markdown_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "test.ruchy",
        r"
/// Test function
fun test() { 42 }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--format")
        .arg("markdown")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Check for markdown file
    let md_file = output_dir.join("test.md");
    assert!(
        md_file.exists() || output_dir.join("index.md").exists(),
        "Markdown documentation should be created"
    );
}

#[test]
fn test_issue_101_doc_html_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "test.ruchy",
        r"
/// Test function
fun test() { 42 }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--format")
        .arg("html")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Check for HTML file
    let html_file = output_dir.join("test.html");
    assert!(
        html_file.exists() || output_dir.join("index.html").exists(),
        "HTML documentation should be created"
    );
}

#[test]
fn test_issue_101_doc_json_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "test.ruchy",
        r"
/// Test function
fun test() { 42 }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Check for JSON file
    let json_file = output_dir.join("test.json");
    assert!(
        json_file.exists() || output_dir.join("docs.json").exists(),
        "JSON documentation should be created"
    );
}

// ============================================================================
// CONTENT VALIDATION TESTS
// ============================================================================

#[test]
fn test_issue_101_doc_markdown_content() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "content.ruchy",
        r#"
/// Greets a person by name
fun greet(name) {
    println!("Hello, {}", name)
}
"#,
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--format")
        .arg("markdown")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Read generated markdown and verify content
    let md_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();

    assert!(
        !md_files.is_empty(),
        "Should generate at least one .md file"
    );

    let content = fs::read_to_string(md_files[0].path()).unwrap();
    assert!(
        content.contains("greet") || content.contains("Greets"),
        "Documentation should contain function name or description"
    );
}

#[test]
fn test_issue_101_doc_json_structure() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "json_test.ruchy",
        r"
/// Test function
fun test() { 42 }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Find and parse JSON file
    let json_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    assert!(
        !json_files.is_empty(),
        "Should generate at least one .json file"
    );

    let content = fs::read_to_string(json_files[0].path()).unwrap();
    assert!(
        content.contains('{') && content.contains('}'),
        "JSON should be valid structure"
    );
}

// ============================================================================
// OPTION TESTS
// ============================================================================

#[test]
fn test_issue_101_doc_private_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "private.ruchy",
        r"
/// Public function
fun public_fn() { 1 }

// Private function (no doc comment)
fun private_fn() { 2 }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--private")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.exists());
}

#[test]
fn test_issue_101_doc_verbose_mode() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "verbose.ruchy",
        r"
/// Test
fun test() { 42 }
",
    );
    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--verbose")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Parsing").or(predicate::str::contains("Generating")));
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_101_doc_missing_file() {
    ruchy_cmd()
        .arg("doc")
        .arg("nonexistent_xyz_12345.ruchy")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

#[test]
fn test_issue_101_doc_invalid_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun test() { 42 }");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--format")
        .arg("invalid_format_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("format").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_101_doc_syntax_error() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.ruchy", "fun bad( { }"); // Invalid syntax

    let output_dir = temp.path().join("docs");

    ruchy_cmd()
        .arg("doc")
        .arg(&file)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("parse")));
}
