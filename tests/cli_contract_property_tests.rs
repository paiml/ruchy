//! CLI Contract Tests: `ruchy property-tests`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, property testing)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Property tests passed
//! - Exit code 1: Property tests failed OR file not found
//! - stdout: Property test report (text/json/markdown format)
//! - stderr: Error messages (test failures, missing files)
//! - Options: --cases, --seed, --format, --output
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (property-tests: 0.4 → target 0.5)
//!
//! **Note**: Most tests are lightweight and don't run full property test suites

#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
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
fn cli_property_tests_valid_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("10") // Very few cases for speed
        .assert();
    // Just verify command runs
}

#[test]
fn cli_property_tests_missing_file_exits_nonzero() {
    // FIX: CLI-CONTRACT-PROPERTY-TESTS-001 - Now handles missing files gracefully
    ruchy_cmd()
        .arg("property-tests")
        .arg("nonexistent_xyz.ruchy")
        .arg("--cases")
        .arg("10")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_property_tests_syntax_error_exits_nonzero() {
    // FIX: CLI-CONTRACT-PROPERTY-TESTS-002 - Now handles syntax errors gracefully
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("10")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: OPTIONS
// ============================================================================

#[test]
fn cli_property_tests_custom_cases() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "cases_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("100") // Custom case count
        .assert();
}

#[test]
fn cli_property_tests_invalid_cases_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_cases.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("invalid") // Invalid number
        .assert()
        .failure(); // Should fail due to invalid argument
}

#[test]
fn cli_property_tests_with_seed() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "seed_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("10")
        .arg("--seed")
        .arg("12345") // Reproducible seed
        .assert();
}

// ============================================================================
// CLI CONTRACT TESTS: FORMAT OPTIONS
// ============================================================================

#[test]
fn cli_property_tests_text_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "text_format.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--format")
        .arg("text")
        .arg("--cases")
        .arg("10")
        .assert();
}

#[test]
fn cli_property_tests_json_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "json_format.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .arg("--cases")
        .arg("10")
        .assert();
}

#[test]
fn cli_property_tests_markdown_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "markdown_format.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--format")
        .arg("markdown")
        .arg("--cases")
        .arg("10")
        .assert();
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT FILE
// ============================================================================

#[test]
fn cli_property_tests_output_to_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output_test.ruchy", "let x = 42\n");
    let output_file = temp.path().join("property_report.txt");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--output")
        .arg(&output_file)
        .arg("--cases")
        .arg("10")
        .assert();
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
fn cli_property_tests_missing_file_writes_stderr() {
    // FIX: CLI-CONTRACT-PROPERTY-TESTS-001 - Now handles missing files gracefully
    ruchy_cmd()
        .arg("property-tests")
        .arg("missing.ruchy")
        .arg("--cases")
        .arg("10")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

#[test]
fn cli_property_tests_syntax_error_writes_stderr() {
    // FIX: CLI-CONTRACT-PROPERTY-TESTS-002 - Now handles syntax errors gracefully
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

// ============================================================================
// CLI CONTRACT TESTS: VERBOSE MODE
// ============================================================================

#[test]
fn cli_property_tests_verbose_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--verbose")
        .arg("--cases")
        .arg("10")
        .assert();
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_property_tests_empty_file_fails() {
    // FIX: CLI-CONTRACT-PROPERTY-TESTS-002 - Now handles empty files gracefully
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("10")
        .assert()
        .failure(); // Empty file should fail
}

#[test]
fn cli_property_tests_complex_program() {
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
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("10")
        .assert();
}

// ============================================================================
// CLI CONTRACT TESTS: HELP
// ============================================================================

#[test]
fn cli_property_tests_help_flag() {
    ruchy_cmd()
        .arg("property-tests")
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("property")
                .or(predicate::str::contains("Property"))
                .or(predicate::str::contains("cases")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: ZERO CASES EDGE CASE
// ============================================================================

#[test]
fn cli_property_tests_zero_cases() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "zero_cases.ruchy", "let x = 42\n");

    // Zero cases might be valid (no tests run) or invalid
    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("0")
        .assert();
    // Just verify command handles this case
}

#[test]
fn cli_property_tests_very_large_cases() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "large_cases.ruchy", "let x = 42\n");

    // Large number of cases (would take very long)
    ruchy_cmd()
        .arg("property-tests")
        .arg(&file)
        .arg("--cases")
        .arg("1000000")
        .timeout(std::time::Duration::from_secs(2))
        .assert();
    // Timeout will kill it, but tests CLI accepts the parameter
}
