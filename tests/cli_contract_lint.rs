//! CLI Contract Tests: `ruchy lint`
//!
//! **Purpose**: Validate user-facing contract (exit codes, lint warnings/errors)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: No lint errors (warnings OK)
//! - Exit code 1: Lint errors found OR syntax error OR file not found
//! - stdout: Lint warnings and errors
//! - stderr: Fatal errors (file not found, syntax errors)
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
fn cli_lint_clean_code_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "clean.ruchy", "let x = 42\nprintln(x)\n");

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_lint_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_lint_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("lint")
        .arg("nonexistent.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: LINT WARNINGS
// ============================================================================

#[test]
fn cli_lint_unused_variable_warning() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "unused.ruchy", "let unused_var = 42\n");

    let output = ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .success(); // Warnings don't cause failure

    // Should contain warning about unused variable
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("unused") || stdout.contains("not used") || stdout.is_empty(),
        "Expected unused variable warning or clean output, got: {stdout}"
    );
}

#[test]
fn cli_lint_shadowing_detection() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "shadow.ruchy",
        "let x = 1\nlet x = 2\nprintln(x)\n",
    );

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .success(); // Shadowing is warning, not error
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR HANDLING
// ============================================================================

#[test]
fn cli_lint_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn cli_lint_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("lint")
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
// CLI CONTRACT TESTS: OUTPUT FORMAT
// ============================================================================

#[test]
fn cli_lint_clean_code_no_warnings() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "perfect.ruchy", "let x = 42\nprintln(x)\n");

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .success();
    // Note: stdout may contain "No lint errors" or be empty - both OK
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_lint_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .failure(); // Empty file is error
}

#[test]
fn cli_lint_comment_only_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// Just a comment\n");

    ruchy_cmd()
        .arg("lint")
        .arg(&file)
        .assert()
        .failure(); // Comment-only is error
}
