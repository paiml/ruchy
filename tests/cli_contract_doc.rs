#![allow(missing_docs)]
// CLI Contract Tests for `ruchy doc` command
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_doc_simple_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "fn add(a, b) { a + b }").unwrap();

    ruchy_cmd()
        .arg("doc")
        .arg(&test_file)
        .assert()
        .code(predicate::ne(2));
}

#[test]
fn test_doc_missing_file() {
    ruchy_cmd()
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

    ruchy_cmd()
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

    ruchy_cmd()
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

    ruchy_cmd()
        .arg("doc")
        .arg(&test_file)
        .arg("--private")
        .assert()
        .code(predicate::ne(2));
}
