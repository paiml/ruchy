#![allow(missing_docs)]
//! CLI Contract Tests: `ruchy coverage`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, coverage reporting)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Coverage analysis successful
//! - Exit code 1: Analysis error OR file not found OR below threshold
//! - stdout: Coverage report (text/html/json format)
//! - stderr: Error messages (analysis errors, missing files)
//! - Threshold: Exit code 1 if coverage below threshold
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (coverage: 0.33 → target 0.5)

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
fn cli_coverage_valid_program_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "println(\"coverage test\")\n");

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_coverage_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_coverage_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("coverage")
        .arg("nonexistent_xyz.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_coverage_below_threshold_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "partial.ruchy",
        r#"
fun test_function(x) {
    if x > 0 {
        println("positive")
    } else {
        println("negative")
    }
}
test_function(5)
"#,
    );

    // Set unrealistically high threshold (100%) - should fail
    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--threshold")
        .arg("100.0")
        .assert()
        .failure(); // Exit code != 0 (below threshold)
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (coverage reports)
// ============================================================================

#[test]
fn cli_coverage_text_format_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\nprintln(x)\n");

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage")
            .or(predicate::str::contains("Lines"))
            .or(predicate::str::contains("%")));
}

#[test]
fn cli_coverage_json_format_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "json_test.ruchy", "println(\"json\")\n");

    let output = ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // JSON output should be parseable
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains('{') && stdout.contains('}'),
        "Should output JSON format"
    );
}

#[test]
fn cli_coverage_html_format_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "html_test.ruchy", "println(\"html\")\n");

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("<")
            .or(predicate::str::contains("html"))
            .or(predicate::str::contains("Coverage")));
}

// ============================================================================
// CLI CONTRACT TESTS: STDERR (error messages)
// ============================================================================

#[test]
fn cli_coverage_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

#[test]
fn cli_coverage_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("coverage")
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
// CLI CONTRACT TESTS: THRESHOLD BEHAVIOR
// ============================================================================

#[test]
fn cli_coverage_meets_threshold_succeeds() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "full.ruchy", "println(\"full coverage\")\n");

    // Low threshold (0%) - should always pass
    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--threshold")
        .arg("0.0")
        .assert()
        .success();
}

#[test]
fn cli_coverage_threshold_message() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "threshold_test.ruchy", "println(\"test\")\n");

    let output = ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--threshold")
        .arg("50.0")
        .assert();

    // Check if threshold is mentioned in output (either stdout or stderr)
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("threshold") || combined.contains("50"),
        "Threshold information should be displayed"
    );
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_coverage_empty_file_succeeds() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    // Empty file has 0/0 lines = 100% coverage (mathematically correct)
    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn cli_coverage_comment_only_succeeds() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// Just a comment\n");

    // Comment-only file has 0/0 lines = 100% coverage
    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn cli_coverage_complex_program() {
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

fun fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

println(factorial(5))
println(fibonacci(8))
",
    );

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage")
            .or(predicate::str::contains("%")));
}

#[test]
fn cli_coverage_with_verbose_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose.ruchy", "println(\"verbose test\")\n");

    ruchy_cmd()
        .arg("coverage")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success();
    // Note: --verbose currently not in CLI definition, but included for completeness
}
