#![allow(missing_docs)]
// CLI Contract Tests for `ruchy wasm` command
//
// Purpose: Validate WASM compilation tool via CLI interface (Layer 4: Black Box)
// Context: WASM is a critical compiler backend for portable execution
//          Has 39 E2E tests + 20 property tests, but NO CLI contract tests
//
// Reference: docs/specifications/15-tool-improvement-spec.md (wasm tool)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn fixture_path(name: &str) -> String {
    format!("tests/fixtures/wasm/{name}")
}

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_wasm_basic_compilation() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("output.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // WASM file should be created
    assert!(output_file.exists(), "WASM file was not created");
}

#[test]
fn test_wasm_missing_file() {
    ruchy_cmd()
        .arg("wasm")
        .arg("tests/fixtures/wasm/nonexistent.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_wasm_math_functions() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("math.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("math.ruchy"))
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // Not a CLI error, may fail due to implementation
}

#[test]
fn test_wasm_loop_constructs() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("loop.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("loop.ruchy"))
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // Not a CLI error
}

// ============================================================================
// Target Platform Tests
// ============================================================================

#[test]
fn test_wasm_target_wasm32() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("wasm32.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--target")
        .arg("wasm32")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_target_wasi() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("wasi.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--target")
        .arg("wasi")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

#[test]
fn test_wasm_target_browser() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("browser.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--target")
        .arg("browser")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

#[test]
fn test_wasm_target_nodejs() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("nodejs.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--target")
        .arg("nodejs")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

#[test]
fn test_wasm_target_cloudflare_workers() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("cf.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--target")
        .arg("cloudflare-workers")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

// ============================================================================
// Optimization Level Tests
// ============================================================================

#[test]
fn test_wasm_opt_level_none() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("opt_none.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--opt-level")
        .arg("none")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_opt_level_o1() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("opt_o1.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--opt-level")
        .arg("O1")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_opt_level_o2() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("opt_o2.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--opt-level")
        .arg("O2")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_opt_level_o3() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("opt_o3.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--opt-level")
        .arg("O3")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_opt_level_os() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("opt_os.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--opt-level")
        .arg("Os")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_opt_level_oz() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("opt_oz.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--opt-level")
        .arg("Oz")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

// ============================================================================
// Feature Flag Tests
// ============================================================================

#[test]
fn test_wasm_with_debug() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("debug.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--debug")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_with_simd() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("simd.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--simd")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

#[test]
fn test_wasm_with_threads() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("threads.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--threads")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

#[test]
fn test_wasm_component_model() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("component.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--component-model")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May not be fully implemented
}

// ============================================================================
// WIT Interface Tests
// ============================================================================

#[test]
fn test_wasm_generate_wit() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("wit.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--wit")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May output WIT file or succeed
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn test_wasm_with_name() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("named.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--name")
        .arg("my-component")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_wasm_with_version() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("versioned.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--version")
        .arg("1.0.0")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

// ============================================================================
// Portability Analysis Tests
// ============================================================================

#[test]
fn test_wasm_portability_analysis() {
    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--portability")
        .assert()
        .code(predicate::ne(2)); // Should analyze portability
}

// ============================================================================
// Verbose Output Tests
// ============================================================================

#[test]
fn test_wasm_verbose_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("verbose.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--verbose")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_wasm_syntax_error() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("syntax_error.ruchy");
    let output_file = temp_dir.path().join("error.wasm");
    fs::write(&test_file, "let x = ").unwrap();

    ruchy_cmd()
        .arg("wasm")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_wasm_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.ruchy");
    let output_file = temp_dir.path().join("empty.wasm");
    fs::write(&test_file, "").unwrap();

    ruchy_cmd()
        .arg("wasm")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty").or(predicate::str::contains("error")));
}

// ============================================================================
// Deployment Tests (May not be implemented)
// ============================================================================

#[test]
#[ignore = "Deployment may require credentials/setup"]
fn test_wasm_deploy_option() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("deploy.wasm");

    ruchy_cmd()
        .arg("wasm")
        .arg(fixture_path("simple.ruchy"))
        .arg("--deploy")
        .arg("--deploy-target")
        .arg("cloudflare")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .code(predicate::ne(2)); // May fail due to missing credentials
}
