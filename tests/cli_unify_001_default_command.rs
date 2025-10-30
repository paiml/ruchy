#![allow(missing_docs)]
// CLI-UNIFY-001: Test that 'ruchy' (no args) opens REPL
// EXTREME TDD: RED phase - these tests will FAIL initially

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_ruchy_no_args_opens_repl() {
    // CRITICAL: `ruchy` with no args should open REPL, not show help
    // Expected behavior: REPL prompt appears
    // Current behavior: Shows help message (WRONG!)

    let mut cmd = ruchy_cmd();
    let assert = cmd
        .write_stdin("1 + 1\n:quit\n")
        .assert();

    // REPL should:
    // 1. Show banner
    // 2. Accept input
    // 3. Show result
    // 4. NOT show help text
    assert
        .success()
        .stdout(predicate::str::contains("Welcome to Ruchy REPL"))  // REPL banner
        .stdout(predicate::str::contains("2"))  // Evaluation result of 1 + 1
        .stdout(predicate::str::contains("Usage:").not()); // NOT help text
}

#[test]
fn test_ruchy_help_flag_shows_help() {
    // `ruchy --help` should show help (this behavior is correct)
    ruchy_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Options:"));
}

#[test]
fn test_ruchy_invalid_args_shows_error() {
    // Invalid arguments should show error and suggest --help
    ruchy_cmd()
        .arg("--invalid-flag-that-does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("unknown")));
}

#[test]
fn test_ruchy_repl_explicit_opens_repl() {
    // `ruchy repl` should open REPL (this should already work)
    let mut cmd = ruchy_cmd();
    let assert = cmd
        .arg("repl")
        .write_stdin(":quit\n")
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("Ruchy").or(predicate::str::contains(">>")));
}
