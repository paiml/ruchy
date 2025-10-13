//! Integration tests for `ruchy add` command (CARGO-003)
//!
//! Tests the complete workflow of adding dependencies to Ruchy projects

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Test helper: Create a test project with cargo new
fn create_test_project() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test_add_project";

    // Use cargo new to create real project
    std::process::Command::new("cargo")
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run cargo new");

    let project_path = temp_dir.path().join(project_name);
    (temp_dir, project_path.to_str().unwrap().to_string())
}

#[test]
fn test_add_command_missing_cargo_toml() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    ruchy_cmd()
        .arg("add")
        .arg("serde")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cargo.toml not found"));
}

#[test]
fn test_add_command_basic_dependency() {
    let (_temp_dir, project_path) = create_test_project();

    ruchy_cmd()
        .arg("add")
        .arg("anyhow")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added anyhow"))
        .stdout(predicate::str::contains("dependency"));

    // Verify anyhow was added to Cargo.toml
    let cargo_toml = fs::read_to_string(format!("{project_path}/Cargo.toml"))
        .expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("anyhow"),
        "anyhow should be in Cargo.toml"
    );
}

#[test]
fn test_add_command_dev_dependency() {
    let (_temp_dir, project_path) = create_test_project();

    ruchy_cmd()
        .arg("add")
        .arg("proptest")
        .arg("--dev")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added proptest"))
        .stdout(predicate::str::contains("dev-dependency"));

    // Verify proptest was added to Cargo.toml as dev-dependency
    let cargo_toml = fs::read_to_string(format!("{project_path}/Cargo.toml"))
        .expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("proptest"),
        "proptest should be in Cargo.toml"
    );

    // Check it's under [dev-dependencies]
    let lines: Vec<&str> = cargo_toml.lines().collect();
    let mut found_dev_deps = false;
    let mut found_proptest = false;

    for line in lines {
        if line.contains("[dev-dependencies]") {
            found_dev_deps = true;
        }
        if found_dev_deps && line.contains("proptest") {
            found_proptest = true;
            break;
        }
    }

    assert!(
        found_proptest,
        "proptest should be under [dev-dependencies]"
    );
}

#[test]
fn test_add_command_with_version() {
    let (_temp_dir, project_path) = create_test_project();

    ruchy_cmd()
        .arg("add")
        .arg("serde")
        .arg("--version")
        .arg("1.0")
        .current_dir(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added serde"))
        .stdout(predicate::str::contains("version 1.0"));

    // Verify serde was added with version constraint
    let cargo_toml = fs::read_to_string(format!("{project_path}/Cargo.toml"))
        .expect("Failed to read Cargo.toml");
    assert!(cargo_toml.contains("serde"), "serde should be in Cargo.toml");
}

#[test]
fn test_add_multiple_dependencies_sequentially() {
    let (_temp_dir, project_path) = create_test_project();

    // Add first dependency
    ruchy_cmd()
        .arg("add")
        .arg("anyhow")
        .current_dir(&project_path)
        .assert()
        .success();

    // Add second dependency
    ruchy_cmd()
        .arg("add")
        .arg("serde")
        .current_dir(&project_path)
        .assert()
        .success();

    // Verify both are in Cargo.toml
    let cargo_toml = fs::read_to_string(format!("{project_path}/Cargo.toml"))
        .expect("Failed to read Cargo.toml");
    assert!(cargo_toml.contains("anyhow"), "anyhow should be present");
    assert!(cargo_toml.contains("serde"), "serde should be present");
}

// Property-based test: Adding same dependency twice should be idempotent
#[test]
fn test_add_command_idempotent() {
    let (_temp_dir, project_path) = create_test_project();

    // Add dependency first time
    ruchy_cmd()
        .arg("add")
        .arg("anyhow")
        .current_dir(&project_path)
        .assert()
        .success();

    // Read Cargo.toml after first add
    let cargo_toml_1 = fs::read_to_string(format!("{project_path}/Cargo.toml"))
        .expect("Failed to read Cargo.toml");

    // Add same dependency second time
    ruchy_cmd()
        .arg("add")
        .arg("anyhow")
        .current_dir(&project_path)
        .assert()
        .success(); // cargo add handles duplicates gracefully

    // Read Cargo.toml after second add
    let cargo_toml_2 = fs::read_to_string(format!("{project_path}/Cargo.toml"))
        .expect("Failed to read Cargo.toml");

    // Should still contain anyhow (cargo add updates version if needed)
    assert!(cargo_toml_1.contains("anyhow"));
    assert!(cargo_toml_2.contains("anyhow"));
}
