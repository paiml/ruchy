#![allow(missing_docs)]
// CLI Contract Tests for `ruchy prove` command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_prove_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "let x = 42").unwrap();

    ruchy_cmd()
        .arg("prove")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_prove_missing_file() {
    ruchy_cmd()
        .arg("prove")
        .arg("nonexistent.ruchy")
        .assert()
        .failure();
}

#[test]
fn test_prove_help() {
    ruchy_cmd()
        .arg("prove")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("prove").or(predicate::str::contains("theorem")));
}
