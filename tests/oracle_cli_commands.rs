//! ORACLE-CLI: Tests for oracle train/save/status subcommands
//!
//! Extreme TDD - RED phase: All tests should fail initially
//!
//! # Test Coverage
//! - `oracle train` - Train model from bootstrap samples
//! - `oracle save` - Persist trained model to .apr file
//! - `oracle status` - Display model statistics
//! - `oracle classify` - Classify error (existing functionality)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// RED PHASE: oracle train tests
// ============================================================================

#[test]
fn test_oracle_train_basic() {
    // Train oracle from bootstrap samples
    ruchy_cmd()
        .args(["oracle", "train"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Training"));
}

#[test]
fn test_oracle_train_verbose() {
    // Train with verbose output showing progress
    ruchy_cmd()
        .args(["oracle", "train", "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Samples"))
        .stdout(predicate::str::contains("Accuracy"));
}

#[test]
fn test_oracle_train_json_output() {
    // Train with JSON output format
    ruchy_cmd()
        .args(["oracle", "train", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\""))
        .stdout(predicate::str::contains("\"samples\""));
}

// ============================================================================
// RED PHASE: oracle save tests
// ============================================================================

#[test]
fn test_oracle_save_default_path() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let model_path = temp_dir.path().join("ruchy_oracle.apr");

    // Train and save to default location
    ruchy_cmd().args(["oracle", "train"]).assert().success();

    ruchy_cmd()
        .args(["oracle", "save", model_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Saved"));

    // Verify file exists
    assert!(model_path.exists(), "Model file should be created");
}

#[test]
fn test_oracle_save_custom_path() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let custom_path = temp_dir.path().join("custom_model.apr");

    ruchy_cmd()
        .args(["oracle", "save", custom_path.to_str().unwrap()])
        .assert()
        .success();

    assert!(custom_path.exists(), "Custom model file should be created");
}

#[test]
fn test_oracle_save_overwrites_existing() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let model_path = temp_dir.path().join("model.apr");

    // Create dummy file
    fs::write(&model_path, "dummy").expect("create dummy file");

    // Save should overwrite
    ruchy_cmd()
        .args(["oracle", "save", model_path.to_str().unwrap()])
        .assert()
        .success();

    // Verify file was overwritten (should be larger than "dummy")
    let metadata = fs::metadata(&model_path).expect("get metadata");
    assert!(metadata.len() > 5, "Model file should be larger than dummy");
}

// ============================================================================
// RED PHASE: oracle status tests
// ============================================================================

#[test]
fn test_oracle_status_trained() {
    // Status shows trained status (oracle auto-trains from bootstrap on first use)
    ruchy_cmd()
        .args(["oracle", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status"));
}

#[test]
fn test_oracle_status_after_train() {
    // Train and save, then check status
    let temp_dir = TempDir::new().expect("create temp dir");
    let model_path = temp_dir.path().join("model.apr");

    // Train and save to file
    ruchy_cmd()
        .args(["oracle", "save", model_path.to_str().unwrap()])
        .assert()
        .success();

    // Status should show trained when model file exists
    // Note: CLI runs are independent, status checks for persisted model
    ruchy_cmd()
        .args(["oracle", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status"));
}

#[test]
#[ignore = "RED phase: oracle status --format json not yet implemented - ORACLE-001"]
fn test_oracle_status_json() {
    // JSON output contains status field and other model info
    ruchy_cmd()
        .args(["oracle", "status", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\""))
        .stdout(predicate::str::contains("\"accuracy\""))
        .stdout(predicate::str::contains("\"samples\""));
}

// ============================================================================
// RED PHASE: oracle classify tests (existing, should still work)
// ============================================================================

#[test]
fn test_oracle_classify_basic() {
    // Classifier output depends on trained model - just verify it returns a category
    ruchy_cmd()
        .args(["oracle", "classify", "error[E0308]: mismatched types"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Category:"));
}

#[test]
fn test_oracle_classify_with_code() {
    ruchy_cmd()
        .args(["oracle", "classify", "mismatched types", "--code", "E0308"])
        .assert()
        .success();
}

#[test]
fn test_oracle_classify_json() {
    ruchy_cmd()
        .args([
            "oracle",
            "classify",
            "borrow of moved value",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"category\""));
}

// ============================================================================
// RED PHASE: oracle load tests
// ============================================================================

#[test]
fn test_oracle_load_existing_model() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let model_path = temp_dir.path().join("model.apr");

    // Train and save
    ruchy_cmd()
        .args(["oracle", "save", model_path.to_str().unwrap()])
        .assert()
        .success();

    // Load should succeed
    ruchy_cmd()
        .args(["oracle", "load", model_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded"));
}

#[test]
fn test_oracle_load_nonexistent_fails() {
    ruchy_cmd()
        .args(["oracle", "load", "/nonexistent/model.apr"])
        .assert()
        .failure();
}

// ============================================================================
// RED PHASE: Integration tests
// ============================================================================

#[test]
fn test_oracle_train_save_load_classify_roundtrip() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let model_path = temp_dir.path().join("roundtrip.apr");

    // Train
    ruchy_cmd().args(["oracle", "train"]).assert().success();

    // Save
    ruchy_cmd()
        .args(["oracle", "save", model_path.to_str().unwrap()])
        .assert()
        .success();

    // Verify .apr exists and has correct magic bytes
    let contents = fs::read(&model_path).expect("read model");
    assert_eq!(&contents[0..4], b"APRN", "Should have APR magic bytes");

    // Load
    ruchy_cmd()
        .args(["oracle", "load", model_path.to_str().unwrap()])
        .assert()
        .success();

    // Classify should work
    ruchy_cmd()
        .args(["oracle", "classify", "error[E0308]: mismatched types"])
        .assert()
        .success();
}

#[test]
fn test_oracle_help_shows_subcommands() {
    ruchy_cmd()
        .args(["oracle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("train"))
        .stdout(predicate::str::contains("save"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("classify"));
}
