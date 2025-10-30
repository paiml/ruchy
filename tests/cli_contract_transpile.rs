//! CLI Contract Tests: `ruchy transpile`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, output files)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Successful transpilation
//! - Exit code 1: Invalid syntax OR file not found
//! - stdout: Rust code (default) OR written to file (-o flag)
//! - stderr: Error messages
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
fn cli_transpile_valid_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "valid.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_transpile_invalid_syntax_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_transpile_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("transpile")
        .arg("nonexistent.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (default output)
// ============================================================================

#[test]
fn cli_transpile_outputs_rust_to_stdout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "hello.ruchy", "let greeting = \"hello\"\n");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main")) // Rust code
        .stdout(predicate::str::contains("let greeting"));
}

#[test]
fn cli_transpile_rust_contains_let_binding() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "binding.ruchy", "let x = 1 + 2\n");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("let x")); // Rust let binding
}

// ============================================================================
// CLI CONTRACT TESTS: FILE OUTPUT (-o flag)
// ============================================================================

#[test]
fn cli_transpile_to_file_creates_output() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.ruchy", "let x = 100\n");
    let output = temp.path().join("output.rs");

    ruchy_cmd()
        .arg("transpile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    // Verify output file exists and contains Rust code
    assert!(output.exists(), "Output file should exist");
    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("fn main"), "Should contain Rust fn main, got: {content}");
}

#[test]
fn cli_transpile_to_file_no_stdout() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.ruchy", "let y = 200\n");
    let output = temp.path().join("output.rs");

    ruchy_cmd()
        .arg("transpile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::is_empty()); // No stdout when writing to file
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR HANDLING
// ============================================================================

#[test]
fn cli_transpile_invalid_syntax_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn cli_transpile_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("transpile")
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
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_transpile_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure(); // Empty file is error
}

#[test]
fn cli_transpile_comment_only_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// Just a comment\n");

    ruchy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure(); // Comment-only is error
}
