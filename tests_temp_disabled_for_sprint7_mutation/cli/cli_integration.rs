//! CLI Integration Tests for Ruchy
//!
//! Tests all CLI commands: eval (-e), parse (-p), transpile (-t), check, run

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

/// Test basic expression evaluation with -e flag
#[test]
fn eval_simple_expression() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("2 + 2")
        .assert()
        .success()
        .stdout("4\n");
}

#[test]
fn eval_string_interpolation() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#"f"Result: {5 * 3}""#)
        .assert()
        .success()
        .stdout("\"Result: 15\"\n");
}

#[test]
fn eval_list_operations() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("[1, 2, 3].map(|x| x * 2)")
        .assert()
        .success()
        .stdout("[2, 4, 6]\n");
}

#[test]
fn eval_filter_operation() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("[1, 2, 3, 4, 5].filter(|x| x > 3)")
        .assert()
        .success()
        .stdout("[4, 5]\n");
}

#[test]
fn eval_reduce_operation() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("[1, 2, 3, 4].reduce(0, |acc, x| acc + x)")
        .assert()
        .success()
        .stdout("10\n");
}

#[test]
fn eval_string_methods() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#""hello".to_upper()"#)
        .assert()
        .success()
        .stdout("\"HELLO\"\n");
}

#[test]
fn eval_with_json_output() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("42")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""success":true"#))
        .stdout(predicate::str::contains(r#""result":"42""#));
}

#[test]
fn eval_invalid_syntax() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("2 + + 3")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// Test file operations (check, transpile, run)
#[test]
fn check_valid_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"println("Hello, World!")"#).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));
}

#[test]
fn check_invalid_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x = ").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Syntax error"));
}

#[test]
fn transpile_function() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fun square(x: i32) -> i32 {{ x * x }}").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("fn square"));
}

#[test]
fn run_simple_program() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"println("Running!")"#).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running!"));
}

#[test]
fn run_with_calculations() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x = 10\nlet y = 20\nprintln(x + y)").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

/// Test piped input
#[test]
fn pipe_input_evaluation() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .write_stdin("2 * 21")
        .assert()
        .success()
        .stdout("42\n");
}

#[test]
fn pipe_multiline_input() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .write_stdin("let x = 5\nx * x")
        .assert()
        .success()
        .stdout("25\n");
}

/// Test AST output
#[test]
fn ast_command() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "1 + 2").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("ast")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Binary"))
        .stdout(predicate::str::contains("Add"));
}

/// Test verbose mode
#[test]
fn eval_with_verbose() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-v")
        .arg("-e")
        .arg("42")
        .assert()
        .success()
        .stderr(predicate::str::contains("Parsing expression"))
        .stdout("42\n");
}

/// Test match expressions
#[test]
fn eval_match_expression() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#"match 5 { 0 => "zero", 5 => "five", _ => "other" }"#)
        .assert()
        .success()
        .stdout("\"five\"\n");
}

/// Test async blocks
#[test]
fn eval_async_block() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("async { 42 }")
        .assert()
        .success();
}

/// Test lambdas with fat arrow
#[test]
fn eval_lambda_fat_arrow() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("let f = |x| => x * 2; f(21)")
        .assert()
        .success()
        .stdout("42\n");
}
