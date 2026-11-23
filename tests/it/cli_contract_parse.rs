#![allow(missing_docs)]
// CLI Contract Tests for `ruchy parse` command
//
// Purpose: Validate AST parser tool via CLI interface (Layer 4: Black Box)
// Context: Core tool for parsing Ruchy files and displaying AST

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn _fixture_path(name: &str) -> String {
    format!("tests/fixtures/{name}")
}

#[test]
fn test_parse_simple_expression() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("parse")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Expr").or(predicate::str::contains("Let")));
}

#[test]
fn test_parse_missing_file() {
    ruchy_cmd()
        .arg("parse")
        .arg("tests/fixtures/nonexistent.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_parse_syntax_error() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("syntax_error.ruchy");
    fs::write(&test_file, "let x = ").unwrap();

    ruchy_cmd()
        .arg("parse")
        .arg(&test_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_parse_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.ruchy");
    fs::write(&test_file, "").unwrap();

    ruchy_cmd()
        .arg("parse")
        .arg(&test_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty").or(predicate::str::contains("error")));
}

#[test]
fn test_parse_function_definition() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("function.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd()
        .arg("parse")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Expr"));
}

#[test]
fn test_parse_control_flow() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("control.ruchy");
    fs::write(&test_file, "if x > 0 { println(\"positive\") }").unwrap();

    ruchy_cmd()
        .arg("parse")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Expr"));
}

#[test]
fn test_parse_complex_expression() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complex.ruchy");
    fs::write(&test_file, "let x = (10 + 5) * (20 - 3) / 2").unwrap();

    ruchy_cmd()
        .arg("parse")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Expr"));
}
