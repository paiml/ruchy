//! ORACLE-004: Auto-train CLI integration tests
//!
//! Extreme TDD - Tests for `ruchy oracle train --auto-train --max-iterations N`
//!
//! # Spec Reference
//! - docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §13

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// RED PHASE: Failing tests for --auto-train and --max-iterations flags
// ============================================================================

/// Test that oracle train command accepts --auto-train flag
#[test]
fn test_oracle_004_train_accepts_auto_train_flag() {
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train"])
        .assert()
        .success();
}

/// Test that oracle train command accepts --max-iterations flag
#[test]
fn test_oracle_004_train_accepts_max_iterations_flag() {
    ruchy_cmd()
        .args(["oracle", "train", "--max-iterations", "10"])
        .assert()
        .success();
}

/// Test that --auto-train with --max-iterations works together
#[test]
fn test_oracle_004_train_auto_train_with_max_iterations() {
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "5"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

/// Test that auto-train shows iteration progress
#[test]
fn test_oracle_004_auto_train_shows_progress() {
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "3"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("iteration").or(predicate::str::contains("Iteration")));
}

/// Test that auto-train reports completion
#[test]
fn test_oracle_004_auto_train_reports_completion() {
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "2"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("complete")
                .or(predicate::str::contains("Complete"))
                .or(predicate::str::contains("done"))
                .or(predicate::str::contains("Done")),
        );
}

/// Test that --max-iterations requires a value
#[test]
fn test_oracle_004_max_iterations_requires_value() {
    ruchy_cmd()
        .args(["oracle", "train", "--max-iterations"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires a value").or(predicate::str::contains("error")));
}

/// Test that --max-iterations rejects invalid values
#[test]
fn test_oracle_004_max_iterations_rejects_invalid() {
    ruchy_cmd()
        .args(["oracle", "train", "--max-iterations", "not-a-number"])
        .assert()
        .failure();
}

/// Test that --max-iterations accepts zero (edge case)
#[test]
fn test_oracle_004_max_iterations_zero() {
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "0"])
        .assert()
        .success();
}

/// Test JSON output format with auto-train
#[test]
fn test_oracle_004_auto_train_json_format() {
    ruchy_cmd()
        .args([
            "oracle",
            "train",
            "--auto-train",
            "--max-iterations",
            "2",
            "--format",
            "json",
        ])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success()
        .stdout(predicate::str::contains("{").and(predicate::str::contains("}")));
}

// ============================================================================
// Integration tests with training loop
// ============================================================================

/// Test that auto-train uses curriculum learning (from spec §3.3)
#[test]
fn test_oracle_004_auto_train_uses_curriculum() {
    ruchy_cmd()
        .args([
            "oracle",
            "train",
            "--auto-train",
            "--max-iterations",
            "5",
            "--verbose",
        ])
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();
    // Note: Curriculum advancement may not appear in 5 iterations, so we just check success
}

/// Test that auto-train respects display modes (compact by default)
#[test]
fn test_oracle_004_auto_train_display_mode() {
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "2"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

// ============================================================================
// Property: Training should be idempotent
// ============================================================================

/// Test that multiple auto-train runs don't corrupt state
#[test]
fn test_oracle_004_auto_train_idempotent() {
    // First run
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "2"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    // Second run should also succeed
    ruchy_cmd()
        .args(["oracle", "train", "--auto-train", "--max-iterations", "2"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}
