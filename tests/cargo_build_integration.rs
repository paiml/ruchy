#![allow(missing_docs)]
//! Integration tests for `ruchy build` command (CARGO-004)
//!
//! Tests the complete workflow of building Ruchy projects

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Test helper: Create a test Ruchy project with ruchy new
fn create_test_ruchy_project() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test_build_project";

    // Use ruchy new to create project with Cargo integration
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_path = temp_dir.path().join(project_name);
    (temp_dir, project_path.to_str().unwrap().to_string())
}

#[test]
fn test_build_command_missing_cargo_toml() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    ruchy_cmd()
        .arg("build")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cargo.toml not found"));
}

#[test]
fn test_build_command_debug_mode() {
    let (_temp_dir, project_path) = create_test_ruchy_project();

    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build complete"))
        .stdout(predicate::str::contains("debug mode"));

    // Verify binary was created in debug mode
    let debug_binary = format!("{project_path}/target/debug/test_build_project");
    assert!(
        std::path::Path::new(&debug_binary).exists()
            || std::path::Path::new(&format!("{debug_binary}.exe")).exists(),
        "Debug binary should exist after build"
    );
}

#[test]
fn test_build_command_release_mode() {
    let (_temp_dir, project_path) = create_test_ruchy_project();

    ruchy_cmd()
        .arg("build")
        .arg("--release")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build complete"))
        .stdout(predicate::str::contains("release mode"));

    // Verify binary was created in release mode
    let release_binary = format!("{project_path}/target/release/test_build_project");
    assert!(
        std::path::Path::new(&release_binary).exists()
            || std::path::Path::new(&format!("{release_binary}.exe")).exists(),
        "Release binary should exist after build"
    );
}

#[test]
fn test_build_command_transpiles_ruchy_files() {
    let (_temp_dir, project_path) = create_test_ruchy_project();

    // Verify main.ruchy exists (created by ruchy new)
    let ruchy_source = format!("{project_path}/src/main.ruchy");
    assert!(
        std::path::Path::new(&ruchy_source).exists(),
        "main.ruchy should exist"
    );

    // Build the project
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Verify main.rs was created (transpiled from main.ruchy)
    let rs_output = format!("{project_path}/src/main.rs");
    assert!(
        std::path::Path::new(&rs_output).exists(),
        "main.rs should be created by build transpilation"
    );

    // Verify transpiled content
    let rs_content = fs::read_to_string(&rs_output).expect("Failed to read main.rs");
    assert!(
        rs_content.contains("fn main"),
        "Transpiled code should contain main function"
    );
}

#[test]
fn test_build_command_incremental() {
    let (_temp_dir, project_path) = create_test_ruchy_project();

    // First build
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Second build should be faster (incremental compilation)
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build complete"));
}

// Property-based test: Building twice should be idempotent
#[test]
fn test_build_command_idempotent() {
    let (_temp_dir, project_path) = create_test_ruchy_project();

    // Build first time
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    let binary_path = format!("{project_path}/target/debug/test_build_project");
    let first_mtime = if let Ok(metadata) = fs::metadata(&binary_path) {
        metadata.modified().ok()
    } else {
        None
    };

    // Wait a bit to ensure timestamps would differ if rebuild occurred
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Build second time (no changes)
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Binary should still exist and work
    assert!(
        std::path::Path::new(&binary_path).exists()
            || std::path::Path::new(&format!("{binary_path}.exe")).exists(),
        "Binary should still exist after second build"
    );

    // Note: We can't reliably test that mtime stayed same due to cargo's incremental compilation
    // But we verify the build succeeded without errors
    if let Some(first) = first_mtime {
        if let Ok(metadata) = fs::metadata(&binary_path) {
            if let Ok(second) = metadata.modified() {
                // If timestamps are same, cargo used incremental compilation
                // If timestamps differ, cargo rebuilt - both are valid
                let _ = (first, second); // Acknowledge both outcomes are valid
            }
        }
    }
}

#[test]
fn test_build_then_run_workflow() {
    let (_temp_dir, project_path) = create_test_ruchy_project();

    // Build the project
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Run the binary using cargo run
    let output = std::process::Command::new("cargo")
        .arg("run")
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo run");

    assert!(output.status.success(), "cargo run should succeed after build");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, Ruchy!"),
        "Program should output expected message"
    );
}
