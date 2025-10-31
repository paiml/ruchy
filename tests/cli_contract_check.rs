//! CLI Contract Tests: `ruchy check`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, error messages)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!

#![allow(clippy::ignore_without_reason)] // CLI contract tests with known limitations
#![allow(missing_docs)]
//! **Contract Specification**:
//! - Exit code 0: Valid syntax
//! - Exit code 1: Invalid syntax OR file not found
//! - stdout: Success messages ("✓ Syntax is valid")
//! - stderr: Error messages with <file:line:col>
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_check_valid_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "valid.ruchy", "let x = 1 + 1\n");

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_check_invalid_syntax_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n"); // Missing value

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_check_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("check")
        .arg("nonexistent_file_12345.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT/STDERR
// ============================================================================

#[test]
fn cli_check_valid_file_writes_stdout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "valid.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("✓").or(predicate::str::contains("valid")));
}

#[test]
fn cli_check_invalid_syntax_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "fun f( { }\n"); // Malformed

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

#[test]
fn cli_check_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("check")
        .arg("missing.ruchy")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
#[ignore = "DEFECT: Error messages don't include filename (CLI-CONTRACT-CHECK-001)"]
fn cli_check_error_includes_filename() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("bad_syntax.ruchy"));
}

#[test]
#[ignore = "DEFECT: Error messages don't include line number (CLI-CONTRACT-CHECK-002)"]
fn cli_check_error_includes_line_number() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "error_line.ruchy", "let x = 1\nlet y = \n"); // Line 2 error

    let output = ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure();

    // Should mention line number (either "line 2" or ":2:" format)
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("line 2") || stderr.contains(":2:") || stderr.contains(" 2"),
        "Error should include line number, got: {stderr}"
    );
}

#[test]
#[ignore = "LIMITATION: `check` tool doesn't support multiple files (CLI-CONTRACT-CHECK-003)"]
fn cli_check_multiple_files_checks_all() {
    let temp = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp, "valid1.ruchy", "let x = 1\n");
    let file2 = create_temp_file(&temp, "valid2.ruchy", "let y = 2\n");

    ruchy_cmd()
        .arg("check")
        .arg(&file1)
        .arg(&file2)
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_check_empty_file_is_error() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure() // Empty file is syntax error (Ruchy requires non-empty programs)
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn cli_check_whitespace_only_is_error() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "whitespace.ruchy", "   \n  \n   \n");

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure() // Whitespace-only is syntax error
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn cli_check_comment_only_is_error() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// This is a comment\n");

    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .failure() // Comment-only is syntax error (no actual code)
        .stderr(predicate::str::contains("Unexpected end of input"));
}
