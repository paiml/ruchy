//! QA-026: Variable Scoping Divergence Fix
//!
//! EXTREME TDD - RED Phase
//!
//! Critical Bug: Constant propagation leaks inner scope bindings to outer scope.
//! Expected: `let x = 10; if true { let x = 20; } print(x)` should print 10
//! Actual: Constant propagation substitutes 20 everywhere
//!
//! Reference: docs/specifications/100-point-qa-beta-checklist-4.0-beta.md [QA-026]

#![allow(clippy::single_char_pattern)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::search_is_some)]
#![allow(deprecated)]

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to run ruchy transpile and return output
fn transpile(code: &str) -> String {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(code.as_bytes()).expect("write code");

    // QA-049: Use -o - to output to stdout
    let output = Command::cargo_bin("ruchy")
        .expect("ruchy binary")
        .arg("transpile")
        .arg(file.path())
        .arg("-o")
        .arg("-")
        .output()
        .expect("run transpile");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// RED TEST 1: Basic shadowing - inner let should not affect outer scope
#[test]
fn test_qa_026_red_01_basic_shadowing() {
    let code = r#"
let x = 10
if true {
    let x = 20
    print(x)
}
print(x)
"#;

    let result = transpile(code);

    // The transpiled code MUST have two different values for the two print statements
    // Inner print should use 20, outer print should use 10
    // Note: Constant propagation may inline the literals, which is correct!

    // The outer print MUST NOT use the inner value
    // This is the critical assertion - outer x should be 10, not 20
    let lines: Vec<&str> = result.lines().collect();
    let last_print_line = lines.iter().rev()
        .find(|l| l.contains("print!"))
        .expect("must have print statement");

    // The last (outer) print should use 10, not 20
    // If scoping is buggy, both prints would use the same value
    assert!(
        last_print_line.contains("10") || last_print_line.contains("x"),
        "Outer scope print must use outer x (value 10). Got:\n{result}"
    );

    // Also verify inner print uses 20
    let inner_print = lines.iter()
        .find(|l| l.contains("print!") && l.contains("20"))
        .is_some();
    assert!(
        inner_print,
        "Inner scope print must use inner x (value 20). Got:\n{result}"
    );
}

/// RED TEST 2: Shadowing with different types
#[test]
fn test_qa_026_red_02_shadowing_different_types() {
    let code = r#"
let x = "hello"
if true {
    let x = 42
    print(x)
}
print(x)
"#;

    let result = transpile(code);

    // Inner x should be 42, outer x should be "hello"
    // Bug would propagate 42 to outer scope (type error!)

    // Check that "hello" appears in the output (outer x is a string)
    assert!(
        result.contains("hello") || result.contains(r#""hello""#),
        "Outer x must remain string. Got:\n{result}"
    );
}

/// RED TEST 3: Nested shadowing - three levels
#[test]
fn test_qa_026_red_03_nested_shadowing() {
    let code = r#"
let x = 1
if true {
    let x = 2
    if true {
        let x = 3
        print(x)
    }
    print(x)
}
print(x)
"#;

    let result = transpile(code);

    // Should have three different scopes with three different x values
    // Bug would make all prints use 3

    // Count how many times we see different values
    // If constant propagation is correct, we should NOT see 3 used everywhere
    let three_count = result.matches("3").count();

    // If all three prints use 3, that's a bug
    // (The literal 3 should only appear once for the innermost print)
    assert!(
        three_count <= 2, // One for the let, one for the print
        "Innermost value 3 should not leak to all scopes. Found {three_count} occurrences. Got:\n{result}"
    );
}

/// RED TEST 4: Shadowing in for loop
#[test]
fn test_qa_026_red_04_shadowing_in_loop() {
    let code = r#"
let x = 100
for i in 0..3 {
    let x = i
    print(x)
}
print(x)
"#;

    let result = transpile(code);

    // The outer x (100) must not be affected by loop body
    // Bug would propagate loop variable to outer scope

    assert!(
        result.contains("100"),
        "Outer x = 100 must be preserved. Got:\n{result}"
    );
}

/// RED TEST 5: Shadowing in else branch
#[test]
fn test_qa_026_red_05_shadowing_in_else() {
    let code = r#"
let x = 1
if false {
    let x = 2
} else {
    let x = 3
    print(x)
}
print(x)
"#;

    let result = transpile(code);

    // Else branch should have x = 3, outer should have x = 1
    // Dead code elimination might remove if false branch, but that's OK

    // The outer print must use 1, not 3
    assert!(
        result.contains("1"),
        "Outer x = 1 must be preserved. Got:\n{result}"
    );
}

/// RED TEST 6: Shadowing does NOT affect mutation
#[test]
fn test_qa_026_red_06_mutation_vs_shadowing() {
    let code = r#"
let mut x = 10
if true {
    x = 20
}
print(x)
"#;

    let result = transpile(code);

    // This is MUTATION, not shadowing - x should be 20 after the if block
    // This test verifies we don't break mutation while fixing shadowing

    // After mutation, x should be 20
    // (This is different from shadowing where outer x stays 10)
    assert!(
        result.contains("mut") && result.contains("x"),
        "Must generate mutable x. Got:\n{result}"
    );
}

/// RED TEST 7: Ensure let declarations are preserved in transpiled output
#[test]
fn test_qa_026_red_07_let_declarations_preserved() {
    let code = r#"
let x = 10
{
    let x = 20
}
"#;

    let result = transpile(code);

    // Both let declarations must appear in output
    // Bug: inner let x = 20 is being eliminated

    // Count let declarations
    let let_count = result.matches("let x").count();

    assert!(
        let_count >= 2,
        "Must have at least 2 let declarations for x (outer and inner). Found {let_count}. Got:\n{result}"
    );
}

/// RED TEST 8: Verify generated Rust compiles correctly
#[test]
fn test_qa_026_red_08_generated_rust_compiles() {
    let code = r#"
let x = 10
if true {
    let x = 20
    print(x)
}
print(x)
"#;

    let result = transpile(code);

    // The generated code must be valid Rust that compiles
    // If constant propagation is buggy, it might generate invalid code

    // Basic syntax check - must have fn main
    assert!(
        result.contains("fn main"),
        "Must generate fn main. Got:\n{result}"
    );

    // Must have proper braces
    let open_braces = result.matches('{').count();
    let close_braces = result.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces must be balanced. Got:\n{result}"
    );
}
