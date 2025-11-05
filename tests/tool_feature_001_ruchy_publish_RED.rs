// RED TEST: ruchy publish command (TOOL-FEATURE-001)
// Missing feature discovered during Reaper v1.0.0 release validation
//
// Expected behavior:
// - Parse Ruchy.toml manifest
// - Validate package metadata (name, version, authors, license)
// - --dry-run: Validate without publishing
// - Create package tarball
// - (Future) Publish to registry
//
// This test will FAIL until ruchy publish is implemented

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// RED TEST 1: ruchy publish --dry-run validates Ruchy.toml
#[test]
fn test_tool_feature_001_01_publish_dry_run_validates_manifest_red() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create minimal Ruchy.toml
    let manifest = r#"[package]
name = "test-package"
version = "0.1.0"
authors = ["Test Author <test@example.com>"]
description = "A test package"
license = "MIT"

[dependencies]
"#;

    fs::write(project_path.join("Ruchy.toml"), manifest).unwrap();

    // Create main.ruchy
    fs::write(
        project_path.join("main.ruchy"),
        "fun main() {\n    println(\"Hello, world!\");\n}\n",
    )
    .unwrap();

    // Run ruchy publish --dry-run
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.current_dir(project_path)
        .arg("publish")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assertions: Should validate manifest successfully
    assert!(
        output.status.success(),
        "ruchy publish --dry-run should succeed with valid Ruchy.toml.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should show validation messages
    assert!(
        stdout.contains("Validating") || stdout.contains("package") || stdout.contains("test-package"),
        "Should show package validation.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should NOT actually publish in dry-run mode
    assert!(
        stdout.contains("dry-run") || stdout.contains("Would publish") || !stdout.contains("Published"),
        "Dry-run should not actually publish.\nStdout: {stdout}\nStderr: {stderr}"
    );
}

/// RED TEST 2: ruchy publish fails with missing Ruchy.toml
#[test]
fn test_tool_feature_001_02_publish_requires_manifest_red() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // NO Ruchy.toml created

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.current_dir(project_path).arg("publish");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail
    assert!(
        !output.status.success(),
        "ruchy publish should fail without Ruchy.toml.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should show helpful error message
    assert!(
        stdout.contains("Ruchy.toml") || stderr.contains("Ruchy.toml") || stdout.contains("manifest"),
        "Should mention missing Ruchy.toml.\nStdout: {stdout}\nStderr: {stderr}"
    );
}

/// RED TEST 3: ruchy publish validates required fields
#[test]
fn test_tool_feature_001_03_publish_validates_required_fields_red() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Invalid Ruchy.toml - missing required fields
    let manifest = r#"[package]
name = "incomplete-package"
# Missing: version, authors, description, license
"#;

    fs::write(project_path.join("Ruchy.toml"), manifest).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.current_dir(project_path)
        .arg("publish")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail validation
    assert!(
        !output.status.success(),
        "ruchy publish should fail with incomplete manifest.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should report missing fields
    let combined = format!("{stdout}{stderr}");
    let has_field_error = combined.contains("version")
        || combined.contains("authors")
        || combined.contains("required")
        || combined.contains("missing");

    assert!(
        has_field_error,
        "Should report missing required fields.\nStdout: {stdout}\nStderr: {stderr}"
    );
}

/// RED TEST 4: ruchy publish parses package metadata correctly
#[test]
fn test_tool_feature_001_04_publish_parses_metadata_red() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let manifest = r#"[package]
name = "my-awesome-package"
version = "1.2.3"
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
description = "An awesome Ruchy package for doing awesome things"
license = "MIT"
repository = "https://github.com/example/awesome"

[dependencies]
# Future: Add dependency support
"#;

    fs::write(project_path.join("Ruchy.toml"), manifest).unwrap();
    fs::write(project_path.join("main.ruchy"), "println(\"Test\");").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.current_dir(project_path)
        .arg("publish")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should parse and display metadata
    assert!(
        output.status.success(),
        "Should validate complete manifest.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should show parsed package info
    assert!(
        stdout.contains("my-awesome-package") || stdout.contains("1.2.3"),
        "Should display package name or version.\nStdout: {stdout}\nStderr: {stderr}"
    );
}

/// RED TEST 5: ruchy publish validates version format
#[test]
fn test_tool_feature_001_05_publish_validates_version_format_red() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Invalid version format
    let manifest = r#"[package]
name = "test-package"
version = "not-a-valid-semver"
authors = ["Test <test@example.com>"]
description = "Test"
license = "MIT"
"#;

    fs::write(project_path.join("Ruchy.toml"), manifest).unwrap();
    fs::write(project_path.join("main.ruchy"), "println(\"Test\");").unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.current_dir(project_path)
        .arg("publish")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail with invalid version
    assert!(
        !output.status.success(),
        "Should reject invalid semver version.\nStdout: {stdout}\nStderr: {stderr}"
    );

    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("version") || combined.contains("semver") || combined.contains("invalid"),
        "Should report version validation error.\nStdout: {stdout}\nStderr: {stderr}"
    );
}
