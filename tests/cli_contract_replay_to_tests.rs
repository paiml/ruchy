#![allow(missing_docs)]
// CLI Contract Tests for `ruchy replay-to-tests` command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_replay_to_tests_missing_file() {
    ruchy_cmd()
        .arg("replay-to-tests")
        .arg("nonexistent.replay")
        .assert()
        .failure();
}

#[test]
fn test_replay_to_tests_help() {
    ruchy_cmd()
        .arg("replay-to-tests")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("replay").or(predicate::str::contains("REPL")));
}

#[test]
fn test_replay_to_tests_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.replay");
    fs::write(&test_file, "let x = 42\nprintln(x)").unwrap();

    ruchy_cmd()
        .arg("replay-to-tests")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2));
}
