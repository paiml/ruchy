#![allow(missing_docs)]
// CLI Contract Tests for `ruchy dataflow:debug` command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_dataflow_debug_help() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("dataflow").or(predicate::str::contains("DataFrame")));
}

#[test]
#[ignore = "dataflow:debug is interactive, doesn't accept file arguments"]
fn test_dataflow_debug_missing_file() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("nonexistent.ruchy")
        .assert()
        .failure();
}

#[test]
#[ignore = "dataflow:debug is interactive, doesn't accept file arguments"]
fn test_dataflow_debug_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("df.ruchy");
    fs::write(&test_file, "let df = df![]").unwrap();

    ruchy_cmd()
        .arg("dataflow:debug")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2));
}
