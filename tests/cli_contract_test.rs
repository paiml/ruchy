#![allow(missing_docs)]
// CLI Contract Tests for `ruchy test` command
//
// Purpose: Validate test runner tool via CLI interface (Layer 4: Black Box)
// Context: Core development tool for running Ruchy test suites

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_test_missing_path() {
    ruchy_cmd()
        .arg("test")
        .arg("tests/fixtures/nonexistent.ruchy")
        .assert()
        .failure();
}

#[test]
fn test_test_watch_option() {
    // Just verify the option is recognized (won't actually watch)
    ruchy_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--watch"));
}

#[test]
fn test_test_verbose_option() {
    ruchy_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn test_test_filter_option() {
    ruchy_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--filter"));
}

#[test]
fn test_test_coverage_option() {
    ruchy_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--coverage"));
}

#[test]
fn test_test_parallel_option() {
    ruchy_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--parallel"));
}

#[test]
fn test_test_format_option() {
    ruchy_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}
