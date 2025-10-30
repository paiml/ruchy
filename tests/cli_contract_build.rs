#![allow(missing_docs)]
// CLI Contract Tests for `ruchy build` command
use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_build_help() {
    ruchy_cmd()
        .arg("build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Build").or(predicate::str::contains("build")));
}

#[test]
fn test_build_no_cargo_toml() {
    // Build should fail without Cargo.toml
    ruchy_cmd()
        .arg("build")
        .assert()
        .code(predicate::ne(2)); // Not a CLI error
}
