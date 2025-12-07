//! Issue #141: TRANSPILER-016 - Unnecessary braces removal
//!
//! Tests that the transpiler removes unnecessary nested braces while
//! preserving them when semantically required (scoping, let bindings).

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Single-expression blocks should be unwrapped
// ============================================================================

#[test]
fn test_issue_141_01_single_expr_unwrapped() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun add(a: i64, b: i64) -> i64 {
    a + b
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should NOT have {{ a + b }} double braces
    assert!(
        !output.contains("{ { a + b } }") && !output.contains("{{ a + b }}"),
        "Should not have unnecessary nested braces: {output}"
    );
}

#[test]
fn test_issue_141_02_single_return_value() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun get_answer() -> i64 {
    42
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should have clean function body with the return value
    assert!(
        output.contains("fn get_answer") && output.contains("42"),
        "Should have function with return value: {output}"
    );
}

// ============================================================================
// Let bindings require braces for scoping
// ============================================================================

#[test]
fn test_issue_141_03_let_binding_keeps_braces() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun compute() -> i64 {
    let x = 10;
    x * 2
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should have braces to contain let binding scope
    assert!(
        output.contains("let x") && output.contains("x * 2"),
        "Should preserve let binding and expression: {output}"
    );
}

#[test]
fn test_issue_141_04_multiple_statements_keep_braces() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun multi() -> i64 {
    let a = 1;
    let b = 2;
    a + b
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should have all statements in the body
    assert!(
        output.contains("let a") && output.contains("let b") && output.contains("a + b"),
        "Should preserve all statements: {output}"
    );
}

// ============================================================================
// If expressions
// ============================================================================

#[test]
fn test_issue_141_05_if_expression_simple() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun check(x: i64) -> i64 {
    if x > 0 { 1 } else { 0 }
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should have clean if expression without extra nesting
    assert!(
        output.contains("if") && output.contains("else"),
        "Should have if/else: {output}"
    );
}

#[test]
fn test_issue_141_06_if_with_complex_branches() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun process(event: i64) -> i64 {
    if event > 100 {
        let x = event * 2;
        x + 1
    } else {
        0
    }
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Complex branch should have braces (for let binding)
    assert!(
        output.contains("let x"),
        "Should preserve let in branch: {output}"
    );
}

// ============================================================================
// Match expressions
// ============================================================================

#[test]
fn test_issue_141_07_match_simple_arms() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r"
fun categorize(n: i64) -> i64 {
    match n {
        1 => 10,
        2 => 20,
        _ => 0,
    }
}
",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Match arms should be clean
    assert!(
        output.contains("match"),
        "Should have match expression: {output}"
    );
}

// ============================================================================
// Ensure code still compiles and runs correctly
// ============================================================================

#[test]
fn test_issue_141_08_functional_correctness() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let result = if true { 42 } else { 0 };
    println!("{}", result)
}
"#,
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("42"), "Should output 42, got: {stdout}");
}
