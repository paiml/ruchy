#![allow(missing_docs)]
//! STDLIB-008: File I/O functions (`file_exists`, `append_file`, `delete_file`)
//!
//! ROOT CAUSE: Missing basic file system operations
//! SOLUTION: Implement `file_exists()`, `append_file()`, `delete_file()`
//!
//! EXTREME TDD: RED → GREEN → REFACTOR

use assert_cmd::Command;
use std::fs;
use std::path::Path;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// RED PHASE: file_exists() tests (WILL FAIL)
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
        .stdout("true\nnil\n");

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
        .stdout("false\nnil\n");
}

// ============================================================================
// RED PHASE: append_file() tests (WILL FAIL)
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
// RED PHASE: delete_file() tests (WILL FAIL)
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
