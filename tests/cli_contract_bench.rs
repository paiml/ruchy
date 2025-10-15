// CLI Contract Tests for `ruchy bench` command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_bench_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "let x = 42 * 2").unwrap();

    ruchy_cmd()
        .arg("bench")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_bench_missing_file() {
    // bench command returns "not yet implemented" even for missing files
    ruchy_cmd()
        .arg("bench")
        .arg("nonexistent.ruchy")
        .assert()
        .code(predicate::ne(2)); // Not a CLI error
}

#[test]
fn test_bench_iterations_option() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("bench")
        .arg(&test_file)
        .arg("--iterations")
        .arg("10")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_bench_warmup_option() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("bench")
        .arg(&test_file)
        .arg("--warmup")
        .arg("5")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_bench_format_json() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("bench")
        .arg(&test_file)
        .arg("--format")
        .arg("json")
        .assert()
        .code(predicate::ne(2));
}
