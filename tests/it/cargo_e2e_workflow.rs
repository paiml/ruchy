#![allow(missing_docs)]
//! End-to-end integration tests for Cargo workflows (CARGO-005)
//!
//! Tests complete workflows: new → add → build → run

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
#[ignore = "E2E test: Runs full cargo build (>60s timeout) - use 'make test-e2e' for full E2E suite"]
fn test_e2e_create_project_add_dependency_build() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "e2e_test_project";
    let project_path = temp_dir.path().join(project_name);

    // Step 1: Create new Ruchy project
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Ruchy project"));

    // Verify project structure
    assert!(
        project_path.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        project_path.join("build.rs").exists(),
        "build.rs should exist"
    );
    assert!(
        project_path.join("src/main.ruchy").exists(),
        "main.ruchy should exist"
    );

    // Step 2: Add a dependency (anyhow for error handling)
    ruchy_cmd()
        .arg("add")
        .arg("anyhow")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added anyhow"));

    // Verify anyhow was added
    let cargo_toml =
        fs::read_to_string(project_path.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("anyhow"),
        "Cargo.toml should contain anyhow"
    );

    // Step 3: Build the project
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build complete"));

    // Verify binary was created
    let binary_path = project_path.join("target/debug").join(project_name);
    assert!(
        binary_path.exists() || binary_path.with_extension("exe").exists(),
        "Binary should exist after build"
    );

    // Step 4: Run the binary
    let output = std::process::Command::new("cargo")
        .arg("run")
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo run");

    assert!(output.status.success(), "cargo run should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, Ruchy!"),
        "Should output expected message"
    );
}

#[test]
#[ignore = "E2E test: Runs full cargo build (>60s timeout) - use 'make test-e2e' for full E2E suite"]
fn test_e2e_library_project_with_dependencies() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "e2e_lib_project";
    let project_path = temp_dir.path().join(project_name);

    // Step 1: Create library project
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .arg("--lib")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify library structure
    assert!(
        project_path.join("src/lib.ruchy").exists(),
        "lib.ruchy should exist"
    );

    // Step 2: Add development dependency
    ruchy_cmd()
        .arg("add")
        .arg("proptest")
        .arg("--dev")
        .current_dir(&project_path)
        .assert()
        .success();

    // Step 3: Build library
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Step 4: Run tests
    let output = std::process::Command::new("cargo")
        .arg("test")
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo test");

    assert!(output.status.success(), "cargo test should succeed");
}

#[test]
#[ignore = "E2E test: Runs full cargo build (>60s timeout) - use 'make test-e2e' for full E2E suite"]
fn test_e2e_multiple_dependencies_and_build() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "e2e_multi_deps";
    let project_path = temp_dir.path().join(project_name);

    // Create project
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Add multiple dependencies
    let dependencies = vec!["anyhow", "serde"];

    for dep in &dependencies {
        ruchy_cmd()
            .arg("add")
            .arg(dep)
            .current_dir(&project_path)
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("Added {dep}")));
    }

    // Verify all dependencies in Cargo.toml
    let cargo_toml =
        fs::read_to_string(project_path.join("Cargo.toml")).expect("Failed to read Cargo.toml");

    for dep in &dependencies {
        assert!(cargo_toml.contains(dep), "{dep} should be in Cargo.toml");
    }

    // Build should succeed with all dependencies
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();
}

#[test]
#[ignore = "E2E test: Runs full cargo build (>60s timeout) - use 'make test-e2e' for full E2E suite"]
fn test_e2e_modify_source_rebuild() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "e2e_rebuild_test";
    let project_path = temp_dir.path().join(project_name);

    // Create and build project
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Modify the Ruchy source file
    let main_ruchy = project_path.join("src/main.ruchy");
    let modified_content = r#"
fun main() {
    println("Modified Ruchy program!");
}
"#;
    fs::write(&main_ruchy, modified_content).expect("Failed to modify main.ruchy");

    // Rebuild (should detect change and recompile)
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    // Run and verify modified output
    let output = std::process::Command::new("cargo")
        .arg("run")
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Modified Ruchy program!"),
        "Should output modified message"
    );
}

#[test]
#[ignore = "E2E test: Runs full cargo build (>60s timeout) - use 'make test-e2e' for full E2E suite"]
fn test_e2e_release_build_optimization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "e2e_release_test";
    let project_path = temp_dir.path().join(project_name);

    // Create project
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Debug build
    ruchy_cmd()
        .arg("build")
        .current_dir(&project_path)
        .assert()
        .success();

    let debug_binary = project_path.join("target/debug").join(project_name);
    let debug_exists = debug_binary.exists() || debug_binary.with_extension("exe").exists();
    assert!(debug_exists, "Debug binary should exist");

    // Release build
    ruchy_cmd()
        .arg("build")
        .arg("--release")
        .current_dir(&project_path)
        .assert()
        .success();

    let release_binary = project_path.join("target/release").join(project_name);
    let release_exists = release_binary.exists() || release_binary.with_extension("exe").exists();
    assert!(release_exists, "Release binary should exist");

    // Note: We can't reliably test that release is smaller/faster in CI
    // but we verify both builds succeed
}

// Property-based test: Workflow should be reproducible
#[test]
#[ignore = "E2E test: Runs full cargo build (>60s timeout) - use 'make test-e2e' for full E2E suite"]
fn test_e2e_workflow_reproducibility() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Run workflow twice with same steps
    for iteration in 1..=2 {
        let project_name = format!("e2e_repro_{iteration}");
        let project_path = temp_dir.path().join(&project_name);

        ruchy_cmd()
            .arg("new")
            .arg(&project_name)
            .current_dir(temp_dir.path())
            .assert()
            .success();

        ruchy_cmd()
            .arg("add")
            .arg("anyhow")
            .current_dir(&project_path)
            .assert()
            .success();

        ruchy_cmd()
            .arg("build")
            .current_dir(&project_path)
            .assert()
            .success();

        // Both iterations should produce working binaries
        let binary = project_path.join("target/debug").join(&project_name);
        assert!(
            binary.exists() || binary.with_extension("exe").exists(),
            "Binary should exist in iteration {iteration}"
        );
    }
}
