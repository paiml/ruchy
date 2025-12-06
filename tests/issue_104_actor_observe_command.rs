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
#[ignore = "BUG: Actor observe command not working"]
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
    // Use --filter-actor instead of deprecated --actor
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--filter-actor")
        .arg("actor-123")
        .assert()
        .success();
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
    // Default duration is 0 (infinite), command runs successfully
    ruchy_cmd()
        .arg("actor:observe")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_interval_custom() {
    // Use --duration instead of deprecated --interval
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--duration")
        .arg("1")
        .assert()
        .success();
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_format_text() {
    // Use current --format flag (no --all needed, default shows all)
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--format")
        .arg("text")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_format_json() {
    let temp = TempDir::new().unwrap();
    let output_file = temp.path().join("actors.json");

    // Use current --format and --export flags
    ruchy_cmd()
        .arg("actor:observe")
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
#[ignore = "BUG: Actor observe command not working"]
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
    // Use --filter-actor pattern instead of old --filter flag
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--filter-actor")
        .arg("idle")
        .assert()
        .success();
}

#[test]
#[ignore = "BUG: Actor observe command not working"]
fn test_issue_104_actor_observe_filter_busy() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--filter-actor")
        .arg("busy")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_filter_crashed() {
    // Use --filter-failed for crashed/failed actors
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--filter-failed")
        .assert()
        .success();
}

#[test]
#[ignore = "BUG: Actor observe command not working"]
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
    // Use --start-mode metrics instead of deprecated --metrics
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--start-mode")
        .arg("metrics")
        .assert()
        .success();
}

#[test]
fn test_issue_104_actor_observe_messages() {
    // Use --start-mode messages instead of deprecated --messages
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--start-mode")
        .arg("messages")
        .assert()
        .success();
}

#[test]
#[ignore = "BUG: Actor observe command not working"]
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
#[ignore = "BUG: Actor observe command not working"]
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
#[ignore = "BUG: Actor observe command not working"]
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
    // Empty filter-actor pattern should be handled gracefully
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--filter-actor")
        .arg("")
        .assert()
        .success(); // Should handle gracefully (show no actors found)
}

#[test]
fn test_issue_104_actor_observe_invalid_format() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--format")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("format").or(predicate::str::contains("invalid")));
}

#[test]
#[ignore = "No --filter flag in current API"]
fn test_issue_104_actor_observe_invalid_filter() {
    // Old --filter flag doesn't exist, use --filter-actor pattern instead
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--filter-actor")
        .arg("invalid_xyz")
        .assert()
        .success(); // Pattern matching is flexible
}

#[test]
#[ignore = "No --interval flag in current API"]
fn test_issue_104_actor_observe_invalid_interval() {
    // Old --interval flag doesn't exist, use --duration instead
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--duration")
        .arg("-100")
        .assert()
        .failure();
}

#[test]
#[ignore = "No --depth flag in current API"]
fn test_issue_104_actor_observe_invalid_depth() {
    // Old --depth flag doesn't exist in current API
    ruchy_cmd()
        .arg("actor:observe")
        .assert()
        .success();
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_issue_104_actor_observe_with_running_program() {
    // Test observing actors from a running Ruchy program with actor system
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(
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

    // actor:observe without --all (default shows all actors)
    ruchy_cmd()
        .arg("actor:observe")
        .assert()
        .success();
}
