#![allow(missing_docs)]
// EXTREME TDD - RED PHASE: Tests written FIRST
// Testing new CLI subcommands: property-tests, mutations, fuzz
// These tests WILL FAIL until implementation is complete

use std::process::Command;

/// RED PHASE TEST 1: property-tests command exists and runs
#[test]
fn test_property_tests_command_exists() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "property-tests", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "property-tests command should exist and show help"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("property-tests") || stdout.contains("Property"),
        "Help output should mention property-tests: {stdout}"
    );
}

/// RED PHASE TEST 2: property-tests runs on `lang_comp` directory
#[test]
fn test_property_tests_runs_on_lang_comp() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ruchy",
            "--",
            "property-tests",
            "tests/lang_comp/basic_syntax/",
            "--cases",
            "100", // Small number for fast test
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "property-tests should run successfully on valid path"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Property Test Report") || stdout.contains("property"),
        "Output should contain property test results: {stdout}"
    );
}

/// RED PHASE TEST 3: property-tests generates JSON report
#[test]
fn test_property_tests_json_format() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ruchy",
            "--",
            "property-tests",
            "tests/lang_comp/basic_syntax/",
            "--format",
            "json",
            "--cases",
            "100",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "property-tests should support JSON format"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output valid JSON
    assert!(
        stdout.contains('{') && stdout.contains('}'),
        "JSON output should contain braces: {stdout}"
    );
}

/// RED PHASE TEST 4: mutations command exists and runs
#[test]
fn test_mutations_command_exists() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "mutations", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "mutations command should exist and show help"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("mutation") || stdout.contains("Mutation"),
        "Help output should mention mutations: {stdout}"
    );
}

/// RED PHASE TEST 5: mutations runs on test file
#[test]
fn test_mutations_runs_on_test_file() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ruchy",
            "--",
            "mutations",
            "tests/lang_comp/basic_syntax/variables_test.rs",
            "--timeout",
            "60",
        ])
        .output()
        .expect("Failed to execute command");

    // May pass or fail depending on mutation coverage, but should run
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Mutation") || stderr.contains("mutation"),
        "Output should contain mutation test results: stdout={stdout}, stderr={stderr}"
    );
}

/// RED PHASE TEST 6: mutations generates JSON report
#[test]
fn test_mutations_json_format() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ruchy",
            "--",
            "mutations",
            "tests/lang_comp/basic_syntax/variables_test.rs",
            "--format",
            "json",
            "--timeout",
            "60",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output JSON (may be in stdout or file)
    let has_json = stdout.contains('{') && stdout.contains('}');
    assert!(
        has_json || output.status.success(),
        "mutations should support JSON format: {stdout}"
    );
}

/// RED PHASE TEST 7: fuzz command exists and runs
#[test]
fn test_fuzz_command_exists() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "fuzz", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "fuzz command should exist and show help"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fuzz") || stdout.contains("Fuzz"),
        "Help output should mention fuzz: {stdout}"
    );
}

/// RED PHASE TEST 8: fuzz runs with small iteration count
#[test]
fn test_fuzz_runs_with_iterations() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ruchy",
            "--",
            "fuzz",
            "parser",
            "--iterations",
            "100", // Small number for fast test
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Fuzz may fail if targets don't exist yet, but should attempt to run
    assert!(
        stdout.contains("Fuzz") || stderr.contains("fuzz") || stdout.contains("iterations"),
        "Output should contain fuzz test results: stdout={stdout}, stderr={stderr}"
    );
}

/// RED PHASE TEST 9: fuzz generates JSON report
#[test]
fn test_fuzz_json_format() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ruchy",
            "--",
            "fuzz",
            "parser",
            "--format",
            "json",
            "--iterations",
            "100",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output JSON
    let has_json = stdout.contains('{') && stdout.contains('}');
    assert!(
        has_json || output.status.success(),
        "fuzz should support JSON format: {stdout}"
    );
}

/// RED PHASE TEST 10: Verify all three commands exist in CLI
#[test]
fn test_all_testing_commands_in_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "ruchy --help should work");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // All three new commands should be listed in help
    let has_property_tests = stdout.contains("property-tests");
    let has_mutations = stdout.contains("mutations");
    let has_fuzz = stdout.contains("fuzz");

    assert!(
        has_property_tests,
        "Help should list property-tests command: {stdout}"
    );
    assert!(
        has_mutations,
        "Help should list mutations command: {stdout}"
    );
    assert!(has_fuzz, "Help should list fuzz command: {stdout}");
}
