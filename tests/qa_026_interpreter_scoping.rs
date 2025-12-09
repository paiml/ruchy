//! QA-026: Interpreter Variable Scoping Tests
//!
//! EXTREME TDD - GREEN Phase
//!
//! Critical Bug: Block expressions did not create new scopes, causing `let` bindings
//! inside blocks to overwrite outer scope variables instead of shadowing them.
//!
//! Expected: `let x = 10; if true { let x = 20 }; println(x)` should print 10
//! Previous: Printed 20 (outer x was overwritten)
//!
//! Reference: docs/specifications/100-point-qa-beta-checklist-4.0-beta.md [QA-026]

#![allow(deprecated)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::if_not_else)]

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to run ruchy and return stdout
fn run_ruchy(code: &str) -> String {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(code.as_bytes()).expect("write code");

    let output = Command::cargo_bin("ruchy")
        .expect("ruchy binary")
        .arg("run")
        .arg(file.path())
        .output()
        .expect("run ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        format!("ERROR: {stderr}\nSTDOUT: {stdout}")
    } else {
        stdout
    }
}

/// GREEN TEST 1: Basic shadowing - inner let should not affect outer scope
#[test]
fn test_qa_026_green_01_basic_shadowing() {
    let code = r#"
let x = 10
if true {
    let x = 20
    println(x)
}
println(x)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 2, "Should have exactly 2 output lines. Got: {result}");
    assert_eq!(lines[0], "20", "Inner print should be 20. Got: {result}");
    assert_eq!(lines[1], "10", "Outer print should be 10. Got: {result}");
}

/// GREEN TEST 2: Shadowing with different types
#[test]
fn test_qa_026_green_02_different_types() {
    let code = r#"
let x = "hello"
if true {
    let x = 42
    println(x)
}
println(x)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 2, "Should have 2 lines. Got: {result}");
    assert_eq!(lines[0], "42", "Inner should be 42. Got: {result}");
    assert_eq!(lines[1], "hello", "Outer should be hello. Got: {result}");
}

/// GREEN TEST 3: Nested shadowing - three levels
#[test]
fn test_qa_026_green_03_nested_shadowing() {
    let code = r#"
let x = 1
if true {
    let x = 2
    if true {
        let x = 3
        println(x)
    }
    println(x)
}
println(x)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 3, "Should have 3 lines. Got: {result}");
    assert_eq!(lines[0], "3", "Innermost should be 3. Got: {result}");
    assert_eq!(lines[1], "2", "Middle should be 2. Got: {result}");
    assert_eq!(lines[2], "1", "Outermost should be 1. Got: {result}");
}

/// GREEN TEST 4: Shadowing in for loop
#[test]
fn test_qa_026_green_04_loop_shadowing() {
    let code = r#"
let x = 100
for i in 0..3 {
    let x = i
    println(x)
}
println(x)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 4, "Should have 4 lines. Got: {result}");
    assert_eq!(lines[0], "0", "Loop iteration 0. Got: {result}");
    assert_eq!(lines[1], "1", "Loop iteration 1. Got: {result}");
    assert_eq!(lines[2], "2", "Loop iteration 2. Got: {result}");
    assert_eq!(lines[3], "100", "Outer x should be 100. Got: {result}");
}

/// GREEN TEST 5: Shadowing in else branch
#[test]
fn test_qa_026_green_05_else_shadowing() {
    let code = r#"
let x = 1
if false {
    let x = 2
    println(x)
} else {
    let x = 3
    println(x)
}
println(x)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 2, "Should have 2 lines. Got: {result}");
    assert_eq!(lines[0], "3", "Else branch should be 3. Got: {result}");
    assert_eq!(lines[1], "1", "Outer should be 1. Got: {result}");
}

/// GREEN TEST 6: Mutation (not shadowing) still works
#[test]
fn test_qa_026_green_06_mutation_works() {
    let code = r#"
let mut x = 10
if true {
    x = 20
}
println(x)
"#;
    let result = run_ruchy(code);

    assert!(
        result.trim() == "20",
        "Mutation should change x to 20. Got: {result}"
    );
}

/// GREEN TEST 7: Standalone block creates scope
#[test]
fn test_qa_026_green_07_standalone_block() {
    let code = r#"
let x = 10
{
    let x = 20
    println(x)
}
println(x)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 2, "Should have 2 lines. Got: {result}");
    assert_eq!(lines[0], "20", "Inner block should be 20. Got: {result}");
    assert_eq!(lines[1], "10", "Outer should be 10. Got: {result}");
}

/// GREEN TEST 8: Multiple variables in nested scopes
#[test]
fn test_qa_026_green_08_multiple_vars() {
    let code = r#"
let x = 1
let y = 2
if true {
    let x = 10
    let y = 20
    println(x)
    println(y)
}
println(x)
println(y)
"#;
    let result = run_ruchy(code);
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), 4, "Should have 4 lines. Got: {result}");
    assert_eq!(lines[0], "10", "Inner x should be 10. Got: {result}");
    assert_eq!(lines[1], "20", "Inner y should be 20. Got: {result}");
    assert_eq!(lines[2], "1", "Outer x should be 1. Got: {result}");
    assert_eq!(lines[3], "2", "Outer y should be 2. Got: {result}");
}
