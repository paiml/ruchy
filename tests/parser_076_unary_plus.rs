//! PARSER-076: Unary plus operator (+x)
//!
//! GitHub Issue: #58 (Part 1/4)
//! Bug: "Unexpected token: Plus" when parsing +42 or +(expr)
//! Root Cause: Parser only handles unary minus, not unary plus
//! Fix: Add Token::Plus case to prefix expression parsing (identity operator)

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic unary plus with literal
#[test]
fn test_parser_076_unary_plus_literal() {
    let code = r#"
let x = +42
x
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Test 2: Unary plus with expression
#[test]
fn test_parser_076_unary_plus_expression() {
    let code = r#"
let x = +(10 - 5)
x
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

// Test 3: Unary plus with float
#[test]
fn test_parser_076_unary_plus_float() {
    let code = r#"
let x = +3.14
x
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14"));
}

// Test 4: Unary plus vs binary plus
#[test]
fn test_parser_076_unary_vs_binary_plus() {
    let code = r#"
let a = +10
let b = 5
let sum = a + b  // binary plus
sum
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
}

// Test 5: Double unary plus (identity of identity)
#[test]
fn test_parser_076_double_unary_plus() {
    let code = r#"
let x = + + 42
x
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Test 6: Unary plus and minus combination
#[test]
fn test_parser_076_plus_minus_combo() {
    let code = r#"
let a = +10
let b = -5
let result = a + b
result
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

// Test 7: Unary plus in function call
#[test]
fn test_parser_076_unary_plus_in_call() {
    let code = r#"
fun abs(x: i32) -> i32 {
    if x < 0 { -x } else { +x }
}

let result = abs(42)
result
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Test 8: Unary plus with variable
#[test]
fn test_parser_076_unary_plus_variable() {
    let code = r#"
let x = 100
let y = +x
y
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

// Test 9: Transpile mode - verify unary plus parses and transpiles correctly
// Note: Unary plus is identity operation, so +42 transpiles to 42 (optimization)
#[test]
fn test_parser_076_transpile_mode() {
    let code = r#"
let x = +42
x
"#;
    let temp_file = "/tmp/test_parser_076_transpile.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("transpile")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Test 10: Unary plus precedence (should bind tighter than binary)
#[test]
fn test_parser_076_precedence() {
    let code = r#"
let result = +10 * 2  // Should be (+10) * 2 = 20, not +(10 * 2) = 20
result
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

// Test 11: Regression - ensure unary minus still works
#[test]
fn test_parser_076_regression_unary_minus() {
    let code = r#"
let x = -42
x
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("-42"));
}

// Test 12: Check mode
#[test]
fn test_parser_076_check_mode() {
    let code = r#"
let x = +42
x
"#;
    let temp_file = "/tmp/test_parser_076_check.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();
}
