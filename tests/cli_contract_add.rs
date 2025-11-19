#![allow(missing_docs)]
// CLI Contract Tests for `ruchy add` command
use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_add_help() {
    ruchy_cmd()
        .arg("add")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("add").or(predicate::str::contains("dependency")));
}

#[test]
fn test_add_missing_package() {
    ruchy_cmd().arg("add").assert().failure();
}
