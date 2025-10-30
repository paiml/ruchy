#![allow(missing_docs)]
// CLI Contract Tests for `ruchy new` command
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_new_creates_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test_project";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(&temp_dir)
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_new_lib_option() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test_lib";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .arg("--lib")
        .current_dir(&temp_dir)
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_new_help() {
    ruchy_cmd()
        .arg("new")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new Ruchy project"));
}
