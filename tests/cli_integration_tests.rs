//! Integration tests for CLI commands
//!
//! [TEST-COV-012] Increase CLI test coverage

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_ruchy_version() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ruchy"));
}

#[test]
fn test_ruchy_help() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_ruchy_eval() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(r#"println("Hello from eval")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from eval"));
}

#[test]
fn test_ruchy_check_valid() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x = 42").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("check").arg(file.path()).assert().success();
}

#[test]
fn test_ruchy_check_invalid() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x =").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("check").arg(file.path()).assert().failure();
}

#[test]
fn test_ruchy_ast() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x = 42").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("ast")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Let"));
}

#[test]
fn test_ruchy_fmt() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let   x   =   42").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("fmt").arg(file.path()).assert().success();
}

#[test]
fn test_ruchy_transpile() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x = 42").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("let x"));
}

#[test]
fn test_ruchy_run() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"println("Running test")"#).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("run")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running test"));
}

#[test]
fn test_ruchy_eval_math() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("println(2 + 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_ruchy_eval_string_concat() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(r#"println("Hello" + " " + "World")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_ruchy_eval_array() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("println([1, 2, 3])")
        .assert()
        .success()
        .stdout(predicate::str::contains("[1, 2, 3]"));
}

#[test]
fn test_ruchy_eval_function() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("fun add(a, b) { a + b }; println(add(3, 4))")
        .assert()
        .success()
        .stdout(predicate::str::contains("7"));
}

#[test]
fn test_ruchy_eval_if_expression() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(r#"println(if true { "yes" } else { "no" })"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("yes"));
}

#[test]
fn test_ruchy_eval_loop() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("for i in 0..3 { println(i) }")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}
