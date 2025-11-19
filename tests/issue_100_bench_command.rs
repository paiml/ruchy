//! Issue #100: ruchy bench command implementation tests
//!
//! Tests the `ruchy bench` command for performance measurement.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/100>
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)

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
// BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_issue_100_bench_simple_script() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 1 + 1\n");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Benchmark Results"))
        .stdout(predicate::str::contains("iterations"))
        .stdout(predicate::str::contains("ms")); // Should show timing
}

#[test]
fn test_issue_100_bench_custom_iterations() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "iter.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--iterations")
        .arg("50")
        .assert()
        .success()
        .stdout(predicate::str::contains("50")); // Should mention iteration count
}

#[test]
fn test_issue_100_bench_with_warmup() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "warmup.ruchy", "let x = 100\n");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--warmup")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("warmup").or(predicate::str::contains("Benchmark")));
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_issue_100_bench_text_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "text.ruchy", "let x = 5\n");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Benchmark Results")); // Human-readable
}

#[test]
fn test_issue_100_bench_json_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "json.ruchy", "let y = 10\n");

    let output = ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // JSON output should be parseable
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains('{') && stdout.contains('}'),
        "JSON format should contain braces"
    );
}

#[test]
fn test_issue_100_bench_csv_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "csv.ruchy", "let z = 20\n");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--format")
        .arg("csv")
        .assert()
        .success()
        .stdout(predicate::str::contains(",").or(predicate::str::contains("time")));
    // CSV should have commas
}

// ============================================================================
// STATISTICS TESTS
// ============================================================================

#[test]
fn test_issue_100_bench_shows_statistics() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "stats.ruchy", "let x = 1 + 2 + 3\n");

    let output = ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--iterations")
        .arg("20")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let stdout_lower = stdout.to_lowercase();

    // Should show min/max/average timing statistics (case-insensitive)
    let has_statistics = stdout_lower.contains("min")
        || stdout_lower.contains("max")
        || stdout_lower.contains("average")
        || stdout_lower.contains("mean")
        || stdout_lower.contains("avg");

    assert!(has_statistics, "Benchmark should show timing statistics");
}

#[test]
fn test_issue_100_bench_verbose_mode() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose.ruchy", "let x = 99\n");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--verbose")
        .arg("--iterations")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("iteration").or(predicate::str::contains("run")));
}

// ============================================================================
// OUTPUT FILE TESTS
// ============================================================================

#[test]
fn test_issue_100_bench_save_to_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "save.ruchy", "let x = 123\n");
    let output_file = temp.path().join("results.txt");

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Output file should exist and contain results
    assert!(output_file.exists(), "Output file should be created");
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(
        !content.is_empty(),
        "Output file should contain benchmark results"
    );
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_100_bench_missing_file() {
    ruchy_cmd()
        .arg("bench")
        .arg("nonexistent_file_12345.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_issue_100_bench_invalid_syntax() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n"); // Incomplete

    ruchy_cmd()
        .arg("bench")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Syntax")));
}
