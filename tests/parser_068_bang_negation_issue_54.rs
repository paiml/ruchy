#![allow(missing_docs)]
// PARSER-068: Fix Bang (!) token ambiguity - Boolean negation vs Actor Send operator
// GitHub Issue: https://github.com/paiml/ruchy/issues/54
//
// BUG: The ! operator was causing runtime hangs when used as prefix unary NOT
// after a newline, because parser treated it as infix binary Send operator.
//
// ROOT CAUSE: Token::Bang serves dual purpose:
//   - Prefix unary: Logical NOT (!expr)
//   - Infix binary: Actor Send (actor ! message)
//
// FIX: Check for whitespace gap before Bang token. If gap detected, treat as prefix NOT.
// Modified: try_new_actor_operators() and try_binary_operators() in src/frontend/parser/mod.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp file with code and return path
fn write_temp_file(code: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).expect("Failed to write temp file");
    (temp_dir, file_path)
}

/// Test basic boolean negation in let statement body (original bug report)
#[test]
fn test_parser_068_basic_negation_let_body() {
    let code = r#"
fun main() {
    let x = false
    let result = !x
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: true"));
}

/// Test boolean negation in function return position
#[test]
fn test_parser_068_negation_function_return() {
    let code = r#"
fun test_negation() -> bool {
    let is_false = false
    !is_false
}

fun main() {
    let result = test_negation()
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: true"));
}

/// Test negation with true literal
#[test]
fn test_parser_068_negation_true() {
    let code = r#"
fun main() {
    let x = true
    let result = !x
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: false"));
}

/// Test double negation (!!expr)
#[test]
fn test_parser_068_double_negation() {
    let code = r#"
fun main() {
    let x = false
    let result = !!x
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: false"));
}

/// Test negation in if condition
#[test]
fn test_parser_068_negation_in_condition() {
    let code = r#"
fun main() {
    let is_valid = false
    if !is_valid {
        println("Not valid")
    } else {
        println("Valid")
    }
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Not valid"));
}

/// Test negation with complex expression
#[test]
fn test_parser_068_negation_complex() {
    let code = r#"
fun main() {
    let a = true
    let b = false
    let result = !(a && b)
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: true"));
}

/// Test inline negation (no whitespace gap) - should still work
#[test]
fn test_parser_068_inline_negation() {
    let code = r#"
fun main() {
    let x = false
    let result = !x
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: true"));
}

/// Test negation with comment between variable and operator
#[test]
fn test_parser_068_negation_with_comment() {
    let code = r#"
fun main() {
    let x = false
    # This is a comment
    let result = !x
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: true"));
}

/// Test negation in nested expressions
#[test]
fn test_parser_068_negation_nested() {
    let code = r#"
fun main() {
    let x = true
    let y = false
    let result = !x || !y
    println("Result: {}", result)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: true"));
}

/// Test that AST is correct - negation should be Unary NOT, not Binary Send
#[test]
fn test_parser_068_ast_structure() {
    let code = r"
fun test() {
    let x = false
    !x
}
";

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("ast")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unary"))
        .stdout(predicate::str::contains("Not"))
        .stdout(predicate::str::contains("Send").not()); // Should NOT be Binary Send
}

// REGRESSION TESTS: Verify we didn't break Actor Send operator

/// Test that Actor Send (!) still works when adjacent (no whitespace)
#[test]
#[ignore = "Actor model not yet implemented"]
fn test_parser_068_actor_send_adjacent() {
    let code = r"
        actor Counter {
            state: { count: 0 }

            receive Increment {
                self.count = self.count + 1
            }
        }

        fun main() {
            let counter = Counter()
            counter ! Increment
        }
    ";

    ruchy_cmd().arg("-e").arg(code).assert().success();
}

/// Test basic negation with multiple variables
#[test]
fn test_parser_068_multiple_negations() {
    let code = r#"
fun main() {
    let a = false
    let b = true
    let c = false

    let r1 = !a
    let r2 = !b
    let r3 = !c

    println("r1: {}, r2: {}, r3: {}", r1, r2, r3)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("r1: true"))
        .stdout(predicate::str::contains("r2: false"))
        .stdout(predicate::str::contains("r3: true"));
}
