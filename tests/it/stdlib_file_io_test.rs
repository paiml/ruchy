#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]
//! STDLIB-008: File I/O functions (`file_exists`, `append_file`, `delete_file`)
//!
//! Tests for basic file system operations
//! IMPLEMENTATION: File I/O via std::fs

use assert_cmd::Command;
use std::fs;
use std::path::Path;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// GREEN PHASE: file_exists() tests (NOW WORKING)
// ============================================================================

#[test]
fn test_file_exists_true() {
    // Create a temporary test file
    let test_file = "/tmp/ruchy_test_exists.txt";
    fs::write(test_file, "test content").unwrap();

    let code = format!(r#"println(file_exists("{test_file}"))"#);

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout("true\n");

    // Cleanup
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_file_exists_false() {
    let code = r#"println(file_exists("/tmp/nonexistent_file_ruchy_test_12345.txt"))"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("false\n");
}

// ============================================================================
// GREEN PHASE: append_file() tests (NOW WORKING)
// ============================================================================

#[test]
fn test_append_file_to_existing() {
    let test_file = "/tmp/ruchy_test_append.txt";
    fs::write(test_file, "line1\n").unwrap();

    let code = format!(r#"append_file("{test_file}", "line2\n")"#);

    ruchy_cmd().arg("-e").arg(&code).assert().success();

    // Verify content
    let content = fs::read_to_string(test_file).unwrap();
    assert_eq!(content, "line1\nline2\n");

    // Cleanup
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_append_file_creates_new() {
    let test_file = "/tmp/ruchy_test_append_new.txt";
    let _ = fs::remove_file(test_file); // Ensure it doesn't exist

    let code = format!(r#"append_file("{test_file}", "new content\n")"#);

    ruchy_cmd().arg("-e").arg(&code).assert().success();

    // Verify content
    let content = fs::read_to_string(test_file).unwrap();
    assert_eq!(content, "new content\n");

    // Cleanup
    let _ = fs::remove_file(test_file);
}

// ============================================================================
// GREEN PHASE: delete_file() tests (NOW WORKING)
// ============================================================================

#[test]
fn test_delete_file_exists() {
    let test_file = "/tmp/ruchy_test_delete.txt";
    fs::write(test_file, "to be deleted").unwrap();

    let code = format!(r#"delete_file("{test_file}")"#);

    ruchy_cmd().arg("-e").arg(&code).assert().success();

    // Verify file was deleted
    assert!(!Path::new(test_file).exists());
}

#[test]
fn test_delete_file_nonexistent() {
    let test_file = "/tmp/ruchy_test_delete_nonexistent.txt";
    let _ = fs::remove_file(test_file); // Ensure it doesn't exist

    let code = format!(r#"delete_file("{test_file}")"#);

    // Should succeed (idempotent delete)
    ruchy_cmd().arg("-e").arg(&code).assert().success();
}
