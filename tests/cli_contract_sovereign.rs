#![allow(missing_docs)]
//! CLI Contract Tests for Ruchy 5.0 Sovereign Platform commands.
//!
//! Tests: infra, sim, widget, apr, model, purify, migrate-4to5.
//! Per ruchy-5.0-sovereign-platform.md Section 6.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// ruchy infra
// ============================================================================

#[test]
fn test_infra_plan_valid_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("infra.ruchy");
    fs::write(&file, "# infra spec").unwrap();

    ruchy_cmd()
        .arg("infra")
        .arg("plan")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("infra plan"));
}

#[test]
fn test_infra_plan_missing_file() {
    ruchy_cmd()
        .arg("infra")
        .arg("plan")
        .arg("/nonexistent/infra.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_infra_status() {
    ruchy_cmd()
        .arg("infra")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("infra status"));
}

#[test]
fn test_infra_apply_no_yes() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("infra.ruchy");
    fs::write(&file, "# infra spec").unwrap();

    ruchy_cmd()
        .arg("infra")
        .arg("apply")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn test_infra_drift() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("infra.ruchy");
    fs::write(&file, "# infra spec").unwrap();

    ruchy_cmd()
        .arg("infra")
        .arg("drift")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn test_infra_destroy_missing_file() {
    ruchy_cmd()
        .arg("infra")
        .arg("destroy")
        .arg("/nonexistent")
        .assert()
        .failure();
}

// ============================================================================
// ruchy sim
// ============================================================================

#[test]
fn test_sim_run_valid_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("sim.ruchy");
    fs::write(&file, "# simulation").unwrap();

    ruchy_cmd()
        .arg("sim")
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("sim run"));
}

#[test]
fn test_sim_run_with_seed() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("sim.ruchy");
    fs::write(&file, "# simulation").unwrap();

    ruchy_cmd()
        .arg("sim")
        .arg("run")
        .arg(&file)
        .arg("--seed")
        .arg("42")
        .assert()
        .success()
        .stdout(predicate::str::contains("Seed: 42"));
}

#[test]
fn test_sim_inspect() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("sim.ruchy");
    fs::write(&file, "# simulation").unwrap();

    ruchy_cmd()
        .arg("sim")
        .arg("inspect")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn test_sim_verify() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("sim.ruchy");
    fs::write(&file, "# simulation").unwrap();

    ruchy_cmd()
        .arg("sim")
        .arg("verify")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("invariants verified"));
}

#[test]
fn test_sim_export_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("sim.ruchy");
    fs::write(&file, "# simulation").unwrap();

    ruchy_cmd()
        .arg("sim")
        .arg("export")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

// ============================================================================
// ruchy widget
// ============================================================================

#[test]
fn test_widget_serve() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("widget.ruchy");
    fs::write(&file, "# widget").unwrap();

    ruchy_cmd()
        .arg("widget")
        .arg("serve")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("widget serve"));
}

#[test]
fn test_widget_build() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("widget.ruchy");
    fs::write(&file, "# widget").unwrap();
    let output = dir.path().join("dist");

    ruchy_cmd()
        .arg("widget")
        .arg("build")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();
}

#[test]
fn test_widget_test() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("widget.ruchy");
    fs::write(&file, "# widget").unwrap();

    ruchy_cmd()
        .arg("widget")
        .arg("test")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn test_widget_inspect() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("widget.ruchy");
    fs::write(&file, "# widget").unwrap();

    ruchy_cmd()
        .arg("widget")
        .arg("inspect")
        .arg(&file)
        .assert()
        .success();
}

// ============================================================================
// ruchy apr
// ============================================================================

#[test]
fn test_apr_run() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("model.ruchy");
    fs::write(&file, "# ml pipeline").unwrap();

    ruchy_cmd()
        .arg("apr")
        .arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Training complete"));
}

#[test]
fn test_apr_quantize() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("model.ruchy");
    fs::write(&file, "# ml").unwrap();

    ruchy_cmd()
        .arg("apr")
        .arg("quantize")
        .arg(&file)
        .arg("--bits")
        .arg("8")
        .assert()
        .success()
        .stdout(predicate::str::contains("8-bit"));
}

#[test]
fn test_apr_bench() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("model.ruchy");
    fs::write(&file, "# ml").unwrap();

    ruchy_cmd()
        .arg("apr")
        .arg("bench")
        .arg(&file)
        .arg("--iterations")
        .arg("50")
        .assert()
        .success()
        .stdout(predicate::str::contains("50 iterations"));
}

// ============================================================================
// ruchy model
// ============================================================================

#[test]
fn test_model_save() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("checkpoint.bin");

    ruchy_cmd()
        .arg("model")
        .arg("save")
        .arg("my-model")
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("my-model"));
}

#[test]
fn test_model_load() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("model.bin");
    fs::write(&file, "fake model data").unwrap();

    ruchy_cmd()
        .arg("model")
        .arg("load")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Model loaded"));
}

#[test]
fn test_model_export_onnx() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("model.bin");
    fs::write(&file, "fake model data").unwrap();

    ruchy_cmd()
        .arg("model")
        .arg("export")
        .arg(&file)
        .arg("--format")
        .arg("onnx")
        .assert()
        .success()
        .stdout(predicate::str::contains("Export complete"));
}

#[test]
fn test_model_verify() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("model.bin");
    fs::write(&file, "fake model data").unwrap();

    ruchy_cmd()
        .arg("model")
        .arg("verify")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("integrity verified"));
}

// ============================================================================
// ruchy purify
// ============================================================================

#[test]
fn test_purify_valid_path() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("script.sh");
    fs::write(&file, "#!/bin/bash\necho hello").unwrap();

    ruchy_cmd()
        .arg("purify")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("purify"));
}

#[test]
fn test_purify_missing_path() {
    ruchy_cmd()
        .arg("purify")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

#[test]
fn test_purify_verbose() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("script.sh");
    fs::write(&file, "#!/bin/bash\necho hello").unwrap();

    ruchy_cmd()
        .arg("purify")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Analyzing"));
}

// ============================================================================
// ruchy migrate-4to5
// ============================================================================

#[test]
fn test_migrate_4to5_dry_run() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("old.ruchy");
    fs::write(&file, "let signal = 42\nlet x = signal + 1").unwrap();

    ruchy_cmd()
        .arg("migrate-4to5")
        .arg(dir.path())
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("signal"));
}

#[test]
fn test_migrate_4to5_apply() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("old.ruchy");
    fs::write(&file, "let yield = 0\n").unwrap();

    ruchy_cmd()
        .arg("migrate-4to5")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Migration complete"));

    // Verify file was modified
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("yield_val"));
}

#[test]
fn test_migrate_4to5_clean_files() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("clean.ruchy");
    fs::write(&file, "let x = 42\nlet y = x + 1\n").unwrap();

    ruchy_cmd()
        .arg("migrate-4to5")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 identifiers renamed"));
}

// ============================================================================
// ruchy contracts
// ============================================================================

#[test]
fn test_contracts_sync() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("code.ruchy");
    fs::write(&file, "fn main() {}").unwrap();

    ruchy_cmd()
        .arg("contracts")
        .arg("sync")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("contracts sync"));
}

#[test]
fn test_contracts_list() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("code.ruchy");
    fs::write(&file, "fn main() {}").unwrap();

    ruchy_cmd()
        .arg("contracts")
        .arg("list")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("contracts list"));
}

#[test]
fn test_contracts_check() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("code.ruchy");
    fs::write(&file, "fn main() {}").unwrap();

    ruchy_cmd()
        .arg("contracts")
        .arg("check")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Contract coverage"));
}

#[test]
fn test_contracts_check_with_threshold() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("code.ruchy");
    fs::write(&file, "fn main() {}").unwrap();

    ruchy_cmd()
        .arg("contracts")
        .arg("check")
        .arg(&file)
        .arg("--min-coverage")
        .arg("80")
        .assert()
        .success()
        .stdout(predicate::str::contains("80.0%"));
}

// ============================================================================
// ruchy suggest-contracts
// ============================================================================

#[test]
fn test_suggest_contracts() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("code.ruchy");
    fs::write(&file, "fn main() {}").unwrap();

    ruchy_cmd()
        .arg("suggest-contracts")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("suggest-contracts"));
}

#[test]
fn test_suggest_contracts_yaml() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("code.ruchy");
    fs::write(&file, "fn main() {}").unwrap();

    ruchy_cmd()
        .arg("suggest-contracts")
        .arg(&file)
        .arg("--format")
        .arg("yaml")
        .assert()
        .success();
}

// ============================================================================
// Version verification
// ============================================================================

#[test]
fn test_version_is_5_0() {
    ruchy_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("5.0.0-alpha"));
}
