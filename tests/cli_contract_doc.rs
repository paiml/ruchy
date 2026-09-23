#![allow(missing_docs)]
// CLI Contract Tests for `ruchy doc` command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// `ruchy doc` writes `docs/<stem>.<ext>` under the current directory, so
/// every command here runs in a directory the test owns. Taking the directory
/// as an argument makes a test that forgets it fail to compile (RHLGA-1: the
/// cwd used to be the repository, and each run rewrote tracked files there).
fn ruchy_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.current_dir(dir);
    cmd
}

#[test]
fn test_doc_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd(temp_dir.path())
        .arg("doc")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_doc_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    ruchy_cmd(temp_dir.path())
        .arg("doc")
        .arg("nonexistent.ruchy")
        .assert()
        .failure();
}

#[test]
fn test_doc_format_html() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd(temp_dir.path())
        .arg("doc")
        .arg(&test_file)
        .arg("--format")
        .arg("html")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_doc_format_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd(temp_dir.path())
        .arg("doc")
        .arg(&test_file)
        .arg("--format")
        .arg("markdown")
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_doc_private_option() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd(temp_dir.path())
        .arg("doc")
        .arg(&test_file)
        .arg("--private")
        .assert()
        .code(predicate::ne(2));
}

/// The output lands in the command's directory, not beside the input file.
#[test]
fn test_rhlga_1_doc_output_stays_in_current_dir() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd(temp_dir.path())
        .arg("doc")
        .arg(&test_file)
        .assert()
        .success();
    assert!(temp_dir
        .path()
        .join("docs")
        .read_dir()
        .unwrap()
        .next()
        .is_some());
}
