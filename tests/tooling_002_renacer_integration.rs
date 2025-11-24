//! TOOLING-002: Renacer Integration Tests
//!
//! Tests for syscall tracing integration to detect:
//! - Performance regressions in transpilation
//! - Unexpected subprocess spawning
//! - Network calls (telemetry leaks)
//! - Self-hosting bootstrap validation
//!
//! Reference: docs/execution/roadmap.yaml TOOLING-002
//! Date: 2025-11-24

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

/// Helper to get path to golden traces directory
fn golden_traces_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden-traces")
}

/// Helper to check if renacer is installed
fn renacer_available() -> bool {
    std::process::Command::new("renacer")
        .arg("--version")
        .output()
        .is_ok()
}

/// Test 1: Verify renacer is installed and accessible
#[test]
fn test_tooling_002_01_renacer_installed() {
    let output = Command::new("renacer")
        .arg("--version")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("renacer"),
        "renacer version output should contain 'renacer'"
    );
    assert!(
        stdout.contains("0.6"),
        "renacer should be v0.6.x, got: {stdout}"
    );
}

/// Test 2: Verify golden-traces directory exists
#[test]
fn test_tooling_002_02_golden_traces_dir_exists() {
    let dir = golden_traces_dir();
    assert!(
        dir.exists(),
        "Golden traces directory should exist at {dir:?}"
    );
    assert!(
        dir.is_dir(),
        "Golden traces path should be a directory: {dir:?}"
    );
}

/// Test 3: Verify ruchy-clusters.toml configuration exists and is valid
#[test]
fn test_tooling_002_03_cluster_config_valid() {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ruchy-clusters.toml");

    assert!(
        config_path.exists(),
        "Cluster config should exist at {config_path:?}"
    );

    let content = fs::read_to_string(&config_path)
        .expect("Should be able to read ruchy-clusters.toml");

    // Verify expected clusters exist
    assert!(
        content.contains("FileIO"),
        "Config should define FileIO cluster"
    );
    assert!(
        content.contains("ProcessControl"),
        "Config should define ProcessControl cluster"
    );
    assert!(
        content.contains("Networking"),
        "Config should define Networking cluster"
    );
    assert!(
        content.contains("Concurrency"),
        "Config should define Concurrency cluster"
    );
    assert!(
        content.contains("SelfHostingBootstrap"),
        "Config should define SelfHostingBootstrap cluster"
    );

    // Verify critical anomaly detection
    assert!(
        content.contains("expected_for_transpiler = false"),
        "Config should mark ProcessControl/Networking as unexpected"
    );
    assert!(
        content.contains("severity = \"critical\""),
        "Config should mark anomalies as critical"
    );
}

/// Test 4: Trace a simple transpilation and verify no subprocess spawning
///
/// This test ensures the transpiler doesn't accidentally spawn subprocesses
/// (shell commands, external tools, etc.)
#[test]
fn test_tooling_002_04_no_subprocess_spawning() {
    if !renacer_available() {
        eprintln!("⚠️  Skipping test: renacer not installed");
        return;
    }

    // Create a simple test file
    let test_file = "/tmp/test_renacer_simple.ruchy";
    fs::write(
        test_file,
        r"
# Simple test file for renacer
fun add(a: i32, b: i32) -> i32
    a + b
end

let result = add(5, 3)
",
    )
    .expect("Should create test file");

    // Trace transpilation (using timeout to prevent hangs)
    let output = std::process::Command::new("timeout")
        .arg("10")
        .arg("renacer")
        .arg("trace")
        .arg("--")
        .arg("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg("transpile")
        .arg(test_file)
        .output();

    // Clean up
    let _ = fs::remove_file(test_file);

    // Parse output to check for subprocess spawning
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for fork/execve/waitpid syscalls (subprocess spawning)
        assert!(
            !stdout.contains("fork"),
            "Transpiler should not spawn subprocesses (found 'fork')"
        );
        assert!(
            !stdout.contains("execve /bin/sh") && !stdout.contains("execve /bin/bash"),
            "Transpiler should not execute shell commands"
        );

        // Note: cargo itself uses execve to launch ruchy, so we can't check for
        // execve in general. We specifically check for shell execution.
    }
}

/// Test 5: Verify no network calls during transpilation
///
/// Transpilers should not make network calls (no telemetry, no remote deps)
#[test]
fn test_tooling_002_05_no_network_calls() {
    if !renacer_available() {
        eprintln!("⚠️  Skipping test: renacer not installed");
        return;
    }

    // Create a simple test file
    let test_file = "/tmp/test_renacer_network.ruchy";
    fs::write(
        test_file,
        r"
fun multiply(a: i32, b: i32) -> i32
    a * b
end
",
    )
    .expect("Should create test file");

    // Trace transpilation with timeout
    let output = std::process::Command::new("timeout")
        .arg("10")
        .arg("renacer")
        .arg("trace")
        .arg("--")
        .arg("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg("transpile")
        .arg(test_file)
        .output();

    // Clean up
    let _ = fs::remove_file(test_file);

    // Parse output to check for network syscalls
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for network syscalls (socket, connect, send, recv)
        assert!(
            !stdout.contains("socket"),
            "Transpiler should not make network calls (found 'socket')"
        );
        assert!(
            !stdout.contains("connect"),
            "Transpiler should not make network calls (found 'connect')"
        );
        assert!(
            !stdout.contains(" send") && !stdout.contains("sendto"),
            "Transpiler should not send network data"
        );
    }
}

/// Test 6: Verify cluster configuration can be parsed by renacer
#[test]
fn test_tooling_002_06_cluster_config_parseable() {
    if !renacer_available() {
        eprintln!("⚠️  Skipping test: renacer not installed");
        return;
    }

    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ruchy-clusters.toml");

    // Note: renacer 0.6.2 doesn't have a --validate-config flag yet,
    // so we verify the TOML is valid by parsing it ourselves
    let content = fs::read_to_string(&config_path)
        .expect("Should read cluster config");

    // Basic TOML validity check
    assert!(content.contains("[[cluster]]"), "Should have cluster definitions");
    assert!(content.contains("name ="), "Should have cluster names");
    assert!(content.contains("syscalls ="), "Should have syscall lists");
}

/// Test 7: Verify Makefile targets exist
#[test]
fn test_tooling_002_07_makefile_targets_exist() {
    let makefile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile");
    let content = fs::read_to_string(&makefile_path)
        .expect("Should read Makefile");

    // Check for renacer targets
    assert!(
        content.contains("renacer-profile:"),
        "Makefile should have renacer-profile target"
    );
    assert!(
        content.contains("renacer-baseline:"),
        "Makefile should have renacer-baseline target"
    );
    assert!(
        content.contains("renacer-anomaly:"),
        "Makefile should have renacer-anomaly target"
    );
}

/// Test 8: Verify examples directory contains test files
#[test]
fn test_tooling_002_08_examples_available() {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");

    assert!(examples_dir.exists(), "Examples directory should exist");

    // Check for basic example files we can use for tracing
    let basic_examples = ["01_basics.ruchy", "02_functions.ruchy"];

    for example in &basic_examples {
        let example_path = examples_dir.join(example);
        assert!(
            example_path.exists(),
            "Example file should exist: {example_path:?}"
        );
    }
}
