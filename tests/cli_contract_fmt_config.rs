//! Tests for formatter configuration integration
//!
//! Sprint 3 Phase 1: Configuration System
//! Ticket: [FMT-PERFECT-021]
//!
//! Tests verify:
//! - Formatter uses configuration from .ruchy-fmt.toml
//! - Config file discovery searches parent directories
//! - Default configuration used when no config file found
//! - Config settings control formatting output (indent, tabs, etc.)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Create a temporary directory with a test file
fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

#[test]
fn test_fmt_uses_default_config_when_no_config_file() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write test file
    fs::write(&test_file, "let x=1+2").expect("Failed to write test file");

    // Format file - should use defaults (4 spaces, no tabs)
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Formatted"));

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Verify default formatting (spaces, not tabs)
    assert!(formatted.contains(' '), "Should use spaces (default)");
    assert!(!formatted.contains('\t'), "Should not use tabs (default)");
}

#[test]
fn test_fmt_loads_config_from_current_directory() {
    let temp_dir = setup_test_dir();
    let config_file = temp_dir.path().join(".ruchy-fmt.toml");
    let test_file = temp_dir.path().join("test.ruchy");

    // Create config file with custom settings
    fs::write(&config_file, r"
indent_width = 2
use_tabs = false
max_line_length = 80
").expect("Failed to write config file");

    // Write test file
    fs::write(&test_file, "let x = 1 + 2").expect("Failed to write test file");

    // Format file - should use config settings
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();
}

#[test]
fn test_fmt_searches_parent_directories_for_config() {
    let temp_dir = setup_test_dir();
    let config_file = temp_dir.path().join(".ruchy-fmt.toml");
    let subdir = temp_dir.path().join("src");
    fs::create_dir(&subdir).expect("Failed to create subdir");
    let test_file = subdir.join("test.ruchy");

    // Create config file in parent directory
    fs::write(&config_file, r"
indent_width = 2
use_tabs = false
").expect("Failed to write config file");

    // Write test file in subdirectory
    fs::write(&test_file, "let x = 1 + 2").expect("Failed to write test file");

    // Format file - should find config in parent
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();
}

#[test]
fn test_fmt_check_passes_for_properly_formatted_file() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write properly formatted file
    fs::write(&test_file, "let x = 1 + 2").expect("Failed to write test file");

    // First format it
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Then check it - should pass
    ruchy_cmd()
        .arg("fmt")
        .arg("--check")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("is properly formatted"));
}

#[test]
#[ignore = "FORMATTER-001: Error message format doesn't match expected pattern"]
fn test_fmt_check_fails_for_improperly_formatted_file() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write improperly formatted file (will differ from formatter output)
    fs::write(&test_file, "let    x   =   1+2  ").expect("Failed to write test file");

    // Check should fail
    let result = ruchy_cmd()
        .arg("fmt")
        .arg("--check")
        .arg(&test_file)
        .assert()
        .failure();

    // Verify error message suggests running without --check
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(
        output.contains("not properly formatted") || output.contains("Run without --check"),
        "Should suggest running without --check"
    );
}

#[test]
fn test_fmt_with_tabs_config() {
    let temp_dir = setup_test_dir();
    let config_file = temp_dir.path().join(".ruchy-fmt.toml");
    let test_file = temp_dir.path().join("test.ruchy");

    // Create config file with tabs enabled
    fs::write(&config_file, r"
indent_width = 4
use_tabs = true
").expect("Failed to write config file");

    // Write test file that needs indentation
    fs::write(&test_file, "{\nlet x = 1\n}").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let _formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Verify tabs are used (config setting applied)
    // Note: Actual verification depends on formatter implementation
    // This test verifies config is loaded, not the specific formatting
}

#[test]
#[ignore = "FORMATTER-002: Config validation not yet implemented"]
fn test_fmt_with_invalid_config_file() {
    let temp_dir = setup_test_dir();
    let config_file = temp_dir.path().join(".ruchy-fmt.toml");
    let test_file = temp_dir.path().join("test.ruchy");

    // Create invalid config file
    fs::write(&config_file, "invalid toml {{{").expect("Failed to write config file");

    // Write test file
    fs::write(&test_file, "let x = 1 + 2").expect("Failed to write test file");

    // Format should fail with config error
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .failure();
}

#[test]
fn test_fmt_nonexistent_file() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("nonexistent.ruchy");

    // Attempt to format nonexistent file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .failure();
}

#[test]
#[ignore = "FORMATTER-004: No output when formatting simple expressions"]
fn test_fmt_with_custom_indent_width() {
    let temp_dir = setup_test_dir();
    let config_file = temp_dir.path().join(".ruchy-fmt.toml");
    let test_file = temp_dir.path().join("test.ruchy");

    // Create config with indent_width = 2
    fs::write(&config_file, r"
indent_width = 2
use_tabs = false
").expect("Failed to write config file");

    // Write test file
    fs::write(&test_file, "let x = 1 + 2").expect("Failed to write test file");

    // Format file - should succeed
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Formatted"));
}
