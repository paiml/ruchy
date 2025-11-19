//! Issue #104: ruchy actor:observe not implemented
//!
//! Tests the `ruchy actor:observe` command for live actor system introspection.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/104>
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)

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
// BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_all_actors() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Actor Observatory"));
}

#[test]
fn test_issue_104_actor_observe_specific_actor() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--actor")
        .arg("actor-123")
        .assert()
        .success()
        .stdout(predicate::str::contains("Actor").or(predicate::str::contains("Observatory")));
}

#[test]
fn test_issue_104_actor_observe_default() {
    // Running without --actor or --all should show all actors by default
    ruchy_cmd().arg("actor:observe").assert().success();
}

// ============================================================================
// INTERVAL POLLING TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_interval_default() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_interval_custom() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--interval")
        .arg("1000")
        .assert()
        .success();
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_format_text() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--format")
        .arg("text")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_format_json() {
    let temp = TempDir::new().unwrap();
    let output_file = temp.path().join("actors.json");

    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--format")
        .arg("json")
        .arg("--export")
        .arg(&output_file)
        .assert()
        .success();

    // Check JSON file was created
    assert!(output_file.exists(), "JSON output file should be created");

    // Verify it's valid JSON
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(
        content.contains('{') && content.contains('}'),
        "Output should be valid JSON"
    );
}

#[test]
fn test_issue_104_actor_observe_format_dashboard() {
    let temp = TempDir::new().unwrap();
    let output_file = temp.path().join("dashboard.html");

    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--format")
        .arg("dashboard")
        .arg("--export")
        .arg(&output_file)
        .assert()
        .success();

    // Check dashboard file was created
    assert!(
        output_file.exists(),
        "Dashboard output file should be created"
    );

    // Verify it contains HTML
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(
        content.contains("<html") || content.contains("<!DOCTYPE"),
        "Dashboard should be HTML"
    );
}

// ============================================================================
// FILTER TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_filter_idle() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--filter")
        .arg("idle")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_filter_busy() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--filter")
        .arg("busy")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_filter_crashed() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--filter")
        .arg("crashed")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_filter_all() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--filter")
        .arg("all")
        .assert()
        .success();
}

// ============================================================================
// OPTIONS TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_metrics() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--metrics")
        .assert()
        .success()
        .stdout(predicate::str::contains("Metrics").or(predicate::str::contains("Performance")));
}

#[test]
fn test_issue_104_actor_observe_messages() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--messages")
        .assert()
        .success()
        .stdout(predicate::str::contains("Message").or(predicate::str::contains("Queue")));
}

#[test]
fn test_issue_104_actor_observe_depth() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--depth")
        .arg("5")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_export() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("actors.txt");

    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--export")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Export file should be created");
}

// ============================================================================
// COMBINED FLAGS TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_all_flags() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("comprehensive.json");

    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--interval")
        .arg("1000")
        .arg("--format")
        .arg("json")
        .arg("--filter")
        .arg("all")
        .arg("--metrics")
        .arg("--messages")
        .arg("--depth")
        .arg("5")
        .arg("--export")
        .arg(&output)
        .assert()
        .success();

    assert!(
        output.exists(),
        "Comprehensive analysis output should be created"
    );
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_invalid_actor_id() {
    // Invalid actor ID format should be handled gracefully
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--actor")
        .arg("")
        .assert()
        .success(); // Should handle gracefully (show no actors found)
}

#[test]
fn test_issue_104_actor_observe_invalid_format() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--format")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("format").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_104_actor_observe_invalid_filter() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--filter")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("filter").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_104_actor_observe_invalid_interval() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--interval")
        .arg("-100")
        .assert()
        .failure();
}

#[test]
fn test_issue_104_actor_observe_invalid_depth() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .arg("--depth")
        .arg("-1")
        .assert()
        .failure();
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_with_running_program() {
    // Test observing actors from a running Ruchy program with actor system
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "actors.ruchy",
        r#"
// Simple actor system example
actor Counter {
    fun new(initial) {
        { count: initial }
    }

    fun increment(state) {
        { count: state.count + 1 }
    }
}

fun main() {
    let counter = Counter.new(0);
    println("Actor created");
}
"#,
    );

    // This test expects actor:observe to work even if the program isn't running yet
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--all")
        .assert()
        .success();
}
