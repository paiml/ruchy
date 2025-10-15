// CLI Contract Tests for `ruchy score` command
//
// Purpose: Validate quality scoring tool via CLI interface (Layer 4: Black Box)
// Context: Unified quality scoring system (RUCHY-0810)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_score_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "let x = 42\nprintln(x)").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2)); // Not a CLI error
}

#[test]
fn test_score_missing_file() {
    ruchy_cmd()
        .arg("score")
        .arg("tests/fixtures/nonexistent.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_score_depth_shallow() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .arg("--depth")
        .arg("shallow")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_score_depth_standard() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .arg("--depth")
        .arg("standard")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_score_depth_deep() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .arg("--depth")
        .arg("deep")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_score_fast_mode() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .arg("--fast")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_score_deep_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .arg("--deep")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_score_explain_option() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("score")
        .arg(&test_file)
        .arg("--explain")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_score_help() {
    ruchy_cmd()
        .arg("score")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("quality scoring"));
}
