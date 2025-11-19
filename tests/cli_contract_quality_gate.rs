#![allow(missing_docs)]
// CLI Contract Tests for `ruchy quality-gate` command
//
// Purpose: Validate quality gate enforcement tool via CLI interface (Layer 4: Black Box)
// Context: Quality gate enforcement system (RUCHY-0815)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_quality_gate_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "let x = 42\nprintln(x)").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2)); // Not a CLI error
}

#[test]
fn test_quality_gate_missing_file() {
    ruchy_cmd()
        .arg("quality-gate")
        .arg("tests/fixtures/nonexistent.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_quality_gate_depth_shallow() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--depth")
        .arg("shallow")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_depth_standard() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--depth")
        .arg("standard")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_depth_deep() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--depth")
        .arg("deep")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_fail_fast() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--fail-fast")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_format_console() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--format")
        .arg("console")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_format_json() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--format")
        .arg("json")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_format_junit() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("quality-gate")
        .arg(&test_file)
        .arg("--format")
        .arg("junit")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_quality_gate_help() {
    ruchy_cmd()
        .arg("quality-gate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("quality gate"));
}
