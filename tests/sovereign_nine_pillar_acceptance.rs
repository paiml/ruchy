#![allow(missing_docs)]
//! Nine-Pillar Acceptance Tests for Ruchy 5.0 Sovereign Platform.
//!
//! Per `docs/specifications/ruchy-5.0-sovereign-platform.md` Section 2 and Section 10,
//! each pillar must pass acceptance before the 5.0.0 release. This file verifies the
//! surface area (parser reservations, transpile output, and CLI entry points) for
//! each pillar compiles and dispatches correctly.
//!
//! Ticket: [EMBED-001] Nine-pillar integration gate (rc.1 precondition).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn write_script(contents: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("script.ruchy");
    fs::write(&path, contents).unwrap();
    (dir, path)
}

// ============================================================================
// Pillar 1 - Correctness (provable contracts / Silver verification)
// ============================================================================

#[test]
fn test_pillar_1_correctness_contract_keywords_reserved() {
    // `requires` / `ensures` are reserved; using them as identifiers MUST fail.
    let (_dir, path) = write_script("let requires = 1\n");
    ruchy_cmd().arg("check").arg(&path).assert().failure();
}

#[test]
fn test_pillar_1_correctness_prove_cli_available() {
    ruchy_cmd()
        .arg("prove")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("prove"));
}

#[test]
fn test_pillar_1_correctness_contracts_subcommands() {
    ruchy_cmd()
        .arg("contracts")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_pillar_1_correctness_suggest_contracts() {
    ruchy_cmd()
        .arg("suggest-contracts")
        .arg("--help")
        .assert()
        .success();
}

// ============================================================================
// Pillar 2 - Compute (trueno SIMD)
// ============================================================================

#[test]
fn test_pillar_2_compute_transpile_surface() {
    // Basic numeric script must transpile; SIMD lowering is opt-in and
    // exercised by dedicated trueno bridge tests.
    let (_dir, path) = write_script("let x = 1 + 2\n");
    ruchy_cmd()
        .arg("transpile")
        .arg(&path)
        .assert()
        .success();
}

// ============================================================================
// Pillar 3 - Infrastructure (forjar IaC)
// ============================================================================

#[test]
fn test_pillar_3_infrastructure_infra_keyword_reserved() {
    let (_dir, path) = write_script("let infra = 1\n");
    ruchy_cmd().arg("check").arg(&path).assert().failure();
}

#[test]
fn test_pillar_3_infrastructure_infra_subcommands() {
    for sub in ["plan", "apply", "drift", "status", "destroy"] {
        ruchy_cmd()
            .arg("infra")
            .arg(sub)
            .arg("--help")
            .assert()
            .success();
    }
}

// ============================================================================
// Pillar 4 - Scripting (bashrs shell transpilation)
// ============================================================================

#[test]
fn test_pillar_4_scripting_purify_cli_available() {
    ruchy_cmd()
        .arg("purify")
        .arg("--help")
        .assert()
        .success();
}

// ============================================================================
// Pillar 5 - Learning (aprender ML)
// ============================================================================

#[test]
fn test_pillar_5_learning_apr_subcommands() {
    for sub in ["run", "serve", "quantize", "inspect", "bench", "eval"] {
        ruchy_cmd()
            .arg("apr")
            .arg(sub)
            .arg("--help")
            .assert()
            .success();
    }
}

#[test]
fn test_pillar_5_learning_model_subcommands() {
    for sub in ["save", "load", "export", "import", "inspect", "verify"] {
        ruchy_cmd()
            .arg("model")
            .arg(sub)
            .arg("--help")
            .assert()
            .success();
    }
}

// ============================================================================
// Pillar 6 - Visualization (presentar widgets + signals)
// ============================================================================

#[test]
fn test_pillar_6_visualization_signal_keyword_reserved() {
    let (_dir, path) = write_script("let signal = 1\n");
    ruchy_cmd().arg("check").arg(&path).assert().failure();
}

#[test]
fn test_pillar_6_visualization_widget_subcommands() {
    for sub in ["serve", "build", "test", "inspect"] {
        ruchy_cmd()
            .arg("widget")
            .arg(sub)
            .arg("--help")
            .assert()
            .success();
    }
}

// ============================================================================
// Pillar 7 - Simulation (simular DES)
// ============================================================================

#[test]
fn test_pillar_7_simulation_sim_subcommands() {
    for sub in ["run", "inspect", "verify", "export"] {
        ruchy_cmd()
            .arg("sim")
            .arg(sub)
            .arg("--help")
            .assert()
            .success();
    }
}

// ============================================================================
// Pillar 8 - Testing (probar / mutations / playbooks)
// ============================================================================

#[test]
fn test_pillar_8_testing_probar_flags_registered() {
    // Help text must advertise the new test flags so downstream tooling can
    // discover them without grepping source.
    let output = ruchy_cmd().arg("test").arg("--help").output().unwrap();
    let help = String::from_utf8_lossy(&output.stdout);
    for flag in ["--probar", "--playbook", "--visual-regression", "--mutations"] {
        assert!(
            help.contains(flag),
            "test --help must advertise {flag}, got: {help}"
        );
    }
}

// ============================================================================
// Pillar 9 - Embedding (ruchy-embed)
// ============================================================================

#[test]
fn test_pillar_9_embedding_yield_keyword_reserved() {
    let (_dir, path) = write_script("let yield = 1\n");
    ruchy_cmd().arg("check").arg(&path).assert().failure();
}

// ============================================================================
// Migration tooling (cross-cutting)
// ============================================================================

#[test]
fn test_migrate_4to5_cli_available() {
    ruchy_cmd()
        .arg("migrate-4to5")
        .arg("--help")
        .assert()
        .success();
}
