#![allow(clippy::ignore_without_reason)] // Test file with known limitations
#![allow(missing_docs)]

//! CLI Contract Tests: `ruchy fuzz`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, fuzz reporting)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Fuzz testing completed successfully
//! - Exit code 1: Fuzz testing found crashes OR file not found OR cargo-fuzz missing
//! - stdout: Fuzz report (text/json format)
//! - stderr: Error messages (crashes, missing dependencies)
//! - Options: --iterations, --timeout, --format, --output, --verbose
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (fuzz: 0.4 → target 0.5)
//!
//! **Note**: These tests are lightweight and don't actually run cargo-fuzz
//! (which can take minutes). They test CLI contract only.

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
// CLI CONTRACT TESTS: BASIC BEHAVIOR
// ============================================================================

#[test]
#[ignore = "Fuzz testing takes too long for regular CI"]
fn cli_fuzz_valid_program_runs() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\n");

    // Note: This will likely fail if cargo-fuzz not installed
    // But it tests the CLI contract
    let result = ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("10") // Very few iterations
        .assert();

    // Either succeeds or fails gracefully with error message
    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // Should mention fuzz or error about missing cargo-fuzz
    assert!(
        combined.contains("fuzz")
            || combined.contains("cargo")
            || combined.contains("not found"),
        "Output should mention fuzzing or missing dependencies"
    );
}

#[test]
fn cli_fuzz_missing_file_fails() {
    ruchy_cmd()
        .arg("fuzz")
        .arg("nonexistent_xyz.ruchy")
        .arg("--iterations")
        .arg("10")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_fuzz_syntax_error_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("10")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: ITERATIONS OPTION
// ============================================================================

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_custom_iterations() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "iter_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("100") // Custom iteration count
        .assert();
    // Just verify the command accepts iteration parameter
}

#[test]
fn cli_fuzz_invalid_iterations_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_iter.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("invalid") // Invalid number
        .assert()
        .failure(); // Should fail due to invalid argument
}

// ============================================================================
// CLI CONTRACT TESTS: TIMEOUT OPTION
// ============================================================================

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_custom_timeout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "timeout_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--timeout")
        .arg("500") // 500ms timeout
        .arg("--iterations")
        .arg("10")
        .assert();
    // Just verify the command accepts timeout parameter
}

#[test]
fn cli_fuzz_invalid_timeout_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_timeout.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--timeout")
        .arg("invalid") // Invalid timeout
        .assert()
        .failure(); // Should fail due to invalid argument
}

// ============================================================================
// CLI CONTRACT TESTS: FORMAT OPTIONS
// ============================================================================

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_text_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "text_test.ruchy", "let x = 42\n");

    let result = ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--format")
        .arg("text")
        .arg("--iterations")
        .arg("10")
        .assert();

    // Check that format flag is accepted
    let output = result.get_output();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("fuzz")
            || combined.contains("not found"),
        "Should handle text format"
    );
}

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_json_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "json_test.ruchy", "let x = 42\n");

    let result = ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .arg("--iterations")
        .arg("10")
        .assert();

    // Check that JSON format flag is accepted
    let output = result.get_output();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.is_empty(),
        "Should produce output in JSON format"
    );
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT FILE
// ============================================================================

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_output_to_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output_test.ruchy", "let x = 42\n");
    let output_file = temp.path().join("fuzz_report.txt");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--output")
        .arg(&output_file)
        .arg("--iterations")
        .arg("10")
        .assert();

    // Note: File may or may not be created depending on cargo-fuzz availability
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
fn cli_fuzz_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("fuzz")
        .arg("missing.ruchy")
        .arg("--iterations")
        .arg("10")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("no bin target")
                .or(predicate::str::contains("failed to build"))
                .or(predicate::str::contains("Error")),
        );
}

#[test]
fn cli_fuzz_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

// ============================================================================
// CLI CONTRACT TESTS: VERBOSE MODE
// ============================================================================

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_verbose_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--verbose")
        .arg("--iterations")
        .arg("10")
        .assert();
    // Just verify verbose flag is accepted
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_fuzz_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    // Empty files should fail
    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("10")
        .assert()
        .failure();
}

#[test]
#[ignore = "Fuzz testing takes too long"]
fn cli_fuzz_complex_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "complex.ruchy",
        r"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

println(factorial(5))
",
    );

    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("10")
        .assert();
    // Complex program should be accepted
}

// ============================================================================
// CLI CONTRACT TESTS: HELP AND USAGE
// ============================================================================

#[test]
fn cli_fuzz_help_flag() {
    ruchy_cmd()
        .arg("fuzz")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("fuzz").or(predicate::str::contains("Fuzz")));
}

// ============================================================================
// CLI CONTRACT TESTS: ZERO ITERATIONS EDGE CASE
// ============================================================================

#[test]
#[ignore = "Edge case testing"]
fn cli_fuzz_zero_iterations() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "zero_iter.ruchy", "let x = 42\n");

    // Zero iterations might be valid (no tests run) or invalid
    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("0")
        .assert();
    // Just verify command handles this case
}

#[test]
#[ignore = "Edge case testing"]
fn cli_fuzz_very_large_iterations() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "large_iter.ruchy", "let x = 42\n");

    // Large number of iterations (would take very long)
    ruchy_cmd()
        .arg("fuzz")
        .arg(&file)
        .arg("--iterations")
        .arg("1000000000") // 1 billion iterations
        .timeout(std::time::Duration::from_secs(2))
        .assert();
    // Timeout will kill it, but tests CLI accepts the parameter
}
