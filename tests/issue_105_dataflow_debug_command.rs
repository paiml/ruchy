//! Issue #105: ruchy dataflow:debug not implemented
//!
//! Tests the `ruchy dataflow:debug` command for `DataFrame` pipeline debugging.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/105>
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
fn test_issue_105_dataflow_debug_default() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dataflow Debugger"));
}

#[test]
fn test_issue_105_dataflow_debug_with_max_rows() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--max-rows")
        .arg("500")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_with_timeout() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--timeout")
        .arg("5000")
        .assert()
        .success();
}

// ============================================================================
// PROFILING AND TRACKING TESTS
// ============================================================================

#[test]
fn test_issue_105_dataflow_debug_enable_profiling() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--enable-profiling")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_track_memory() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--track-memory")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_compute_diffs() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--compute-diffs")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_auto_materialize() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--auto-materialize")
        .assert()
        .success();
}

// ============================================================================
// FORMAT TESTS
// ============================================================================

#[test]
fn test_issue_105_dataflow_debug_format_text() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--format")
        .arg("text")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_format_json() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("debug.json");

    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--format")
        .arg("json")
        .arg("--export")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "JSON output file should be created");
}

#[test]
fn test_issue_105_dataflow_debug_format_interactive() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--format")
        .arg("interactive")
        .assert()
        .success();
}

// ============================================================================
// START MODE TESTS
// ============================================================================

#[test]
fn test_issue_105_dataflow_debug_start_mode_overview() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--start-mode")
        .arg("overview")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_start_mode_stages() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--start-mode")
        .arg("stages")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_start_mode_data() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--start-mode")
        .arg("data")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_start_mode_metrics() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--start-mode")
        .arg("metrics")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_start_mode_history() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--start-mode")
        .arg("history")
        .assert()
        .success();
}

// ============================================================================
// OPTIONS TESTS
// ============================================================================

#[test]
fn test_issue_105_dataflow_debug_verbose() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_no_color() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--no-color")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_sample_rate() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--sample-rate")
        .arg("0.5")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_refresh_interval() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--refresh-interval")
        .arg("2000")
        .assert()
        .success();
}

#[test]
fn test_issue_105_dataflow_debug_export() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("debug_data.txt");

    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--export")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Export file should be created");
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_105_dataflow_debug_invalid_format() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--format")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("format").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_105_dataflow_debug_invalid_start_mode() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--start-mode")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("start").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_105_dataflow_debug_invalid_sample_rate() {
    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--sample-rate")
        .arg("2.0")
        .assert()
        .failure();
}

// ============================================================================
// COMBINED FLAGS TESTS
// ============================================================================

#[test]
fn test_issue_105_dataflow_debug_all_flags() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("comprehensive_debug.json");

    ruchy_cmd()
        .arg("dataflow:debug")
        .arg("--max-rows")
        .arg("500")
        .arg("--timeout")
        .arg("5000")
        .arg("--auto-materialize")
        .arg("--enable-profiling")
        .arg("--track-memory")
        .arg("--compute-diffs")
        .arg("--sample-rate")
        .arg("0.8")
        .arg("--refresh-interval")
        .arg("2000")
        .arg("--format")
        .arg("json")
        .arg("--start-mode")
        .arg("overview")
        .arg("--verbose")
        .arg("--export")
        .arg(&output)
        .assert()
        .success();

    assert!(
        output.exists(),
        "Comprehensive debug output should be created"
    );
}
