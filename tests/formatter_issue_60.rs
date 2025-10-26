// Issue #60: ruchy fmt incorrectly transforms fun → fn
//
// Problem: The formatter outputs `fn` (Rust syntax) instead of `fun` (Ruchy syntax)
// Impact: Breaks ruchyruchy bootstrap code which uses `fun` consistently
//
// Extreme TDD: RED phase - tests written FIRST to demonstrate bug

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Function declaration should preserve `fun` keyword
#[test]
fn test_format_preserves_fun_keyword() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.ruchy");

    // Write file with correct `fun` syntax
    fs::write(&file, "fun greet(name) { println(\"Hello, {}\", name) }").unwrap();

    // Format the file
    ruchy_cmd()
        .arg("fmt")
        .arg(&file)
        .assert()
        .success();

    // Read back and verify `fun` is preserved (not converted to `fn`)
    let content = fs::read_to_string(&file).unwrap();
    assert!(
        content.contains("fun greet"),
        "Formatter should preserve 'fun' keyword, but got: {}",
        content
    );
    assert!(
        !content.contains("fn greet"),
        "Formatter incorrectly converted 'fun' to 'fn': {}",
        content
    );
}

// Test 2: Multiple functions should all preserve `fun`
#[test]
fn test_format_multiple_functions_preserve_fun() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.ruchy");

    fs::write(
        &file,
        r#"
fun add(a, b) { a + b }
fun multiply(x, y) { x * y }
fun greet(name) { println("Hello, {}", name) }
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&file)
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();

    // Verify all functions use `fun` (none use `fn`)
    assert_eq!(
        content.matches("fun ").count(),
        3,
        "Expected 3 'fun' keywords, got: {}",
        content
    );
    assert_eq!(
        content.matches("fn ").count(),
        0,
        "Formatter incorrectly output 'fn' keyword: {}",
        content
    );
}

// Test 3: Nested functions should preserve `fun`
#[test]
fn test_format_nested_functions_preserve_fun() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.ruchy");

    fs::write(
        &file,
        r#"
fun outer() {
    fun inner(x) { x + 1 }
    inner(42)
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&file)
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();

    assert!(
        content.contains("fun outer"),
        "Outer function should use 'fun': {}",
        content
    );
    assert!(
        content.contains("fun inner"),
        "Inner function should use 'fun': {}",
        content
    );
    assert!(
        !content.contains("fn "),
        "Formatter incorrectly used 'fn': {}",
        content
    );
}

// Test 4: Functions with type annotations should preserve `fun`
#[test]
fn test_format_typed_functions_preserve_fun() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.ruchy");

    fs::write(
        &file,
        "fun add(a: Int, b: Int) -> Int { a + b }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&file)
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();

    assert!(
        content.contains("fun add"),
        "Typed function should use 'fun': {}",
        content
    );
    assert!(
        !content.contains("fn add"),
        "Formatter incorrectly used 'fn': {}",
        content
    );
}

// Test 5: Anonymous functions should preserve `fun` (if applicable)
#[test]
fn test_format_anonymous_functions_preserve_fun() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.ruchy");

    fs::write(
        &file,
        r#"
let double = fun(x) { x * 2 }
let result = double(21)
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&file)
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();

    // Check that if `fun` was used in input, it's preserved in output
    if content.contains("fun(") || content.contains("fun (") {
        assert!(
            !content.contains("fn(") && !content.contains("fn ("),
            "Formatter incorrectly converted 'fun' to 'fn': {}",
            content
        );
    }
}

// Test 6: Issue #60 - Real-world ruchyruchy bootstrap code pattern
#[test]
fn test_format_ruchyruchy_pattern() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.ruchy");

    // Pattern from ruchyruchy bootstrap
    fs::write(
        &file,
        r#"
struct Compiler {
    source: String
}

impl Compiler {
    fun new(source: String) -> Compiler {
        Compiler { source }
    }

    fun compile(self) -> Result {
        // compilation logic
        Ok(())
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&file)
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();

    // Both impl methods should use `fun`
    assert!(
        content.contains("fun new"),
        "Method 'new' should use 'fun': {}",
        content
    );
    assert!(
        content.contains("fun compile"),
        "Method 'compile' should use 'fun': {}",
        content
    );
    assert_eq!(
        content.matches("fn ").count(),
        0,
        "Formatter incorrectly used 'fn' instead of 'fun': {}",
        content
    );
}
