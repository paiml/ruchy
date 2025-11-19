#![allow(missing_docs)]
// CLI Contract Tests for `ruchy publish` command
use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_publish_help() {
    ruchy_cmd()
        .arg("publish")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("publish").or(predicate::str::contains("registry")));
}

#[test]
#[ignore = "Publishing requires proper package setup and credentials"]
fn test_publish_no_package() {
    ruchy_cmd().arg("publish").assert().failure();
}
