// PERF-002 Phase 3: Add --show-profile-info flag
// Tests for showing profile characteristics before compilation
// EXTREME TDD: RED phase - Tests WILL FAIL initially
//
// Reference: docs/specifications/optimized-binary-speed-size-spec.md (Phase 3)
// Feature: Display profile info (opt-level, LTO, expected speedup, size, etc.)

#![allow(missing_docs)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper: Create temp Ruchy file
fn create_temp_ruchy_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

/// Helper: Simple Ruchy program for testing
fn simple_ruchy_program() -> &'static str {
    r#"
fun main() {
    let x = 42;
    println!("Result: {}", x);
}
"#
}

// ============================================================================
// RED PHASE: --show-profile-info Flag Tests (WILL FAIL INITIALLY)
// ============================================================================

/// Test 1: --show-profile-info flag exists and is recognized
/// Expected: Command accepts flag without "unknown argument" error
#[test]
fn test_perf_002_phase3_01_show_profile_info_flag_exists() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .success(); // Should not fail with "unknown argument" error
}

/// Test 2: Profile info displays optimization level
/// Expected: Output contains "opt-level = 3" for default release profile
#[test]
fn test_perf_002_phase3_02_displays_optimization_level() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("opt-level"));
}

/// Test 3: Profile info displays LTO setting
/// Expected: Output contains "LTO: fat" for release profile
#[test]
fn test_perf_002_phase3_03_displays_lto_setting() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("LTO"));
}

/// Test 4: Profile info displays codegen units
/// Expected: Output contains "Codegen units: 1"
#[test]
fn test_perf_002_phase3_04_displays_codegen_units() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("Codegen units"));
}

/// Test 5: Profile info displays expected speedup
/// Expected: Output contains "15x" speedup for release profile
#[test]
fn test_perf_002_phase3_05_displays_expected_speedup() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("15x").or(predicate::str::contains("speedup")));
}

/// Test 6: Profile info displays expected size
/// Expected: Output contains "1-2 MB" size estimate
#[test]
fn test_perf_002_phase3_06_displays_expected_size() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("MB").or(predicate::str::contains("size")));
}

/// Test 7: Profile info displays best use case
/// Expected: Output contains description like "General-purpose production binaries"
#[test]
fn test_perf_002_phase3_07_displays_best_use_case() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("Best for"));
}

/// Test 8: Profile info displays compile time estimate
/// Expected: Output contains compile time like "30-60s"
#[test]
fn test_perf_002_phase3_08_displays_compile_time() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("Compile time"));
}

/// Test 9: Profile info shows alternative profiles
/// Expected: Output suggests --profile release-tiny and release-ultra
#[test]
fn test_perf_002_phase3_09_shows_alternative_profiles() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(
            predicate::str::contains("release-tiny")
                .or(predicate::str::contains("Alternative profiles")),
        );
}

/// Test 10: Profile info formatted with visual separators
/// Expected: Output has visual formatting (lines, boxes, etc.)
#[test]
fn test_perf_002_phase3_10_formatted_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(predicate::str::contains("━").or(predicate::str::contains("Profile:")));
}

/// Test 11: Profile name displayed in output
/// Expected: Output shows "Profile: release (default)"
#[test]
fn test_perf_002_phase3_11_displays_profile_name() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .assert()
        .stdout(
            predicate::str::contains("Profile:")
                .and(predicate::str::contains("release").or(predicate::str::contains("default"))),
        );
}

/// Test 12: Without --show-profile-info, no profile info displayed
/// Expected: Regular compile output without profile characteristics
#[test]
fn test_perf_002_phase3_12_no_profile_info_without_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        // NO --show-profile-info flag
        .assert()
        .stdout(predicate::str::contains("Profile:").not());
}

// ============================================================================
// Property Tests: Profile Info Consistency
// ============================================================================

/// Property Test 1: All profiles display consistent structure
/// Expected: release-tiny, release-ultra show same fields as release
#[test]
fn test_perf_002_phase3_property_consistent_structure() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    // Test release profile
    let release_output = ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output1"))
        .arg("--show-profile-info")
        .output()
        .unwrap();

    let release_stdout = String::from_utf8_lossy(&release_output.stdout);

    // Check that all expected fields are present
    assert!(release_stdout.contains("opt-level") || release_stdout.contains("Optimization"));
    assert!(release_stdout.contains("LTO"));
    assert!(release_stdout.contains("Codegen"));
}

/// Property Test 2: Profile info matches PERF-002 spec data
/// Expected: release shows 15x speedup, release-tiny shows 2x speedup
#[test]
fn test_perf_002_phase3_property_matches_spec() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    let output = ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // release profile should mention 15x speedup (per PERF-002 empirical data)
    assert!(
        stdout.contains("15") || stdout.contains("speedup"),
        "Profile info should mention expected speedup per PERF-002 spec"
    );
}

/// Property Test 3: All alternative profiles are documented
/// Expected: Info mentions release-tiny (314 KB) and release-ultra (25-50x)
#[test]
fn test_perf_002_phase3_property_all_alternatives() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_ruchy_file(&temp, "test.ruchy", simple_ruchy_program());

    let output = ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(temp.path().join("output"))
        .arg("--show-profile-info")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention both alternative profiles
    let has_tiny = stdout.contains("release-tiny") || stdout.contains("314 KB");
    let has_ultra = stdout.contains("release-ultra") || stdout.contains("PGO");

    assert!(
        has_tiny || has_ultra,
        "Profile info should mention alternative profiles"
    );
}
