#![allow(missing_docs)]
//! CLI Contract Tests: `ruchy run`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, execution behavior)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Successful execution
//! - Exit code 1: Runtime error OR syntax error OR file not found
//! - stdout: Program output
//! - stderr: Error messages (runtime errors, panics)
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
fn cli_run_valid_program_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "hello.ruchy", "println(\"Hello, World!\")\n");

    ruchy_cmd().arg("run").arg(&file).assert().success(); // Exit code 0
}

#[test]
fn cli_run_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd().arg("run").arg(&file).assert().failure(); // Exit code != 0
}

#[test]
fn cli_run_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("run")
        .arg("nonexistent_file_xyz.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_run_runtime_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "runtime_error.ruchy", "let x = 1 / 0\n"); // Division by zero

    ruchy_cmd().arg("run").arg(&file).assert().failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (program output)
// ============================================================================

#[test]
fn cli_run_println_writes_stdout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output.ruchy", "println(\"test output\")\n");

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("test output"));
}

#[test]
fn cli_run_multiple_println_writes_lines() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "multi_line.ruchy",
        "println(\"line 1\")\nprintln(\"line 2\")\nprintln(\"line 3\")\n",
    );

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn cli_run_arithmetic_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "math.ruchy", "println(2 + 2)\n");

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn cli_run_string_interpolation_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "interpolation.ruchy",
        "let name = \"Ruchy\"\nprintln(f\"Hello, {name}!\")\n",
    );

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Ruchy!"));
}

// ============================================================================
// CLI CONTRACT TESTS: STDERR (error messages)
// ============================================================================

#[test]
fn cli_run_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

#[test]
fn cli_run_runtime_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "div_zero.ruchy", "let x = 10 / 0\n");

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Division by zero"));
}

#[test]
fn cli_run_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("run")
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
// CLI CONTRACT TESTS: FUNCTION EXECUTION
// ============================================================================

#[test]
fn cli_run_function_definition_and_call() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "function.ruchy",
        "fun greet(name) {\n  println(f\"Hello, {name}!\")\n}\ngreet(\"World\")\n",
    );

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

#[test]
fn cli_run_function_returns_value() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "return.ruchy",
        "fun add(a, b) {\n  a + b\n}\nlet result = add(3, 5)\nprintln(result)\n",
    );

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("8"));
}

// ============================================================================
// CLI CONTRACT TESTS: CONTROL FLOW
// ============================================================================

#[test]
fn cli_run_if_expression() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "if.ruchy",
        "let x = 10\nif x > 5 {\n  println(\"greater\")\n} else {\n  println(\"lesser\")\n}\n",
    );

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("greater"));
}

#[test]
fn cli_run_for_loop() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "loop.ruchy",
        "for i in range(0, 3) {\n  println(i)\n}\n",
    );

    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_run_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    ruchy_cmd().arg("run").arg(&file).assert().failure(); // Empty file is error
}

#[test]
fn cli_run_comment_only_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// Just a comment\n");

    ruchy_cmd().arg("run").arg(&file).assert().failure(); // Comment-only is error
}

#[test]
fn cli_run_no_output_program_succeeds() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "silent.ruchy", "let x = 42\n"); // No println

    // EXPECTED: No output (user didn't call println)
    // ACTUAL: Prints "42" and internal message structure
    ruchy_cmd()
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::is_empty()); // No output is valid
}
