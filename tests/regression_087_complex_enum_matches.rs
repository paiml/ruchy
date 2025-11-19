// RED Phase Test for Issue #87: Syntax error in complex files with multiple enum matches
//
// This test MUST FAIL initially, demonstrating the parser state issue with
// large files containing multiple enum match patterns.
//
// Expected failure: "Syntax error: Expected RightBrace, found Identifier"
// Root cause: Parser state not properly reset between match expressions in complex files
//
// Test follows EXTREME TDD methodology:
// 1. RED: Test fails with syntax error (current)
// 2. GREEN: Fix parser state handling
// 3. REFACTOR: Add line numbers to error messages

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Test RED phase: Large file with multiple enum matches fails to parse
///
/// This reproduces Issue #87 where files with 100+ lines and multiple
/// match expressions cause parser state issues.
#[test]
fn test_regression_087_complex_enum_matches_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_issue_87.ruchy");

    // Create a complex file with multiple enum match patterns
    // Based on the issue report: error appears around line 80-105
    let code = r#"
// Complex file with multiple enum match patterns
// Reproduces Issue #87: Syntax error in files with repeated match patterns

enum AudioError {
    DeviceNotFound,
    PermissionDenied,
    InitializationFailed,
}

struct AudioDevice {
    name: String,
    id: i32,
}

// Function 1: First match pattern (should work)
fun test_detect_devices_1() {
    let devices = match detect_audio() {
        Ok(d) => d,
        Err(e) => {
            println!("Error occurred");
            println!("Reason: {:?}", e);
            return;
        }
    };

    println!("Found {} devices", devices.len());
}

// Function 2: Second match pattern (starting to accumulate state)
fun test_detect_devices_2() {
    let devices = match detect_audio() {
        Ok(d) => d,
        Err(e) => {
            println!("Error occurred");
            println!("Reason: {:?}", e);
            return;
        }
    };

    let validated = match validate_devices(devices) {
        Ok(v) => v,
        Err(e) => {
            println!("Validation error");
            println!("Details: {:?}", e);
            return;
        }
    };

    println!("Validated {} devices", validated.len());
}

// Function 3: Third match pattern (state accumulation continues)
fun test_detect_devices_3() {
    let devices = match detect_audio() {
        Ok(d) => d,
        Err(e) => {
            println!("Error occurred");
            println!("Reason: {:?}", e);
            return;
        }
    };

    let validated = match validate_devices(devices) {
        Ok(v) => v,
        Err(e) => {
            println!("Validation error");
            println!("Details: {:?}", e);
            return;
        }
    };

    let initialized = match initialize_devices(validated) {
        Ok(i) => i,
        Err(e) => {
            println!("Initialization error");
            println!("Details: {:?}", e);
            return;
        }
    };

    println!("Initialized {} devices", initialized.len());
}

// Line 90+: Around here the error appears according to issue report
// Function 4: Fourth match pattern (critical accumulation point)
fun test_detect_devices_4() {
    let devices = match detect_audio() {
        Ok(d) => d,
        Err(e) => {
            println!("Error occurred");
            println!("Reason: {:?}", e);
            return;
        }
    };

    let validated = match validate_devices(devices) {
        Ok(v) => v,
        Err(e) => {
            println!("Validation error");
            println!("Details: {:?}", e);
            return;
        }
    };

    let initialized = match initialize_devices(validated) {
        Ok(i) => i,
        Err(e) => {
            println!("Initialization error");
            println!("Details: {:?}", e);
            return;
        }
    };

    let configured = match configure_devices(initialized) {
        Ok(c) => c,
        Err(e) => {
            println!("Configuration error");  // Expected: Syntax error HERE
            println!("Details: {:?}", e);
            return;
        }
    };

    println!("Configured {} devices", configured.len());
}

// Helper functions (stubbed)
fun detect_audio() -> Result<Vec<AudioDevice>, AudioError> {
    Ok(vec![])
}

fun validate_devices(devices: Vec<AudioDevice>) -> Result<Vec<AudioDevice>, AudioError> {
    Ok(devices)
}

fun initialize_devices(devices: Vec<AudioDevice>) -> Result<Vec<AudioDevice>, AudioError> {
    Ok(devices)
}

fun configure_devices(devices: Vec<AudioDevice>) -> Result<Vec<AudioDevice>, AudioError> {
    Ok(devices)
}

fun main() {
    test_detect_devices_1();
    test_detect_devices_2();
    test_detect_devices_3();
    test_detect_devices_4();
    println!("All tests passed!");
}
"#;

    fs::write(&test_file, code).unwrap();

    // RED: This should succeed but currently fails with syntax error
    let output = ruchy_cmd().arg("run").arg(&test_file).assert();

    // Expected to pass after fix
    output.success();
}

/// Test RED phase: Verify individual patterns work
///
/// This confirms that the patterns work in isolation,
/// proving it's a state accumulation issue, not a syntax issue.
#[test]
fn test_regression_087_individual_pattern_works() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_simple.ruchy");

    let code = r#"
enum AudioError {
    DeviceNotFound,
}

struct AudioDevice {
    name: String,
}

fun detect_audio() -> Result<Vec<AudioDevice>, AudioError> {
    Ok(vec![])
}

fun main() {
    let devices = match detect_audio() {
        Ok(d) => d,
        Err(e) => {
            println!("Error occurred");
            println!("Reason: {:?}", e);
            return;
        }
    };

    println!("Success!");
}
"#;

    fs::write(&test_file, code).unwrap();

    // This should work (and does work according to issue report)
    ruchy_cmd().arg("run").arg(&test_file).assert().success();
}

/// Test GREEN phase: Verify error messages include line numbers
///
/// After fixing the parser state issue, we should also
/// add line numbers to error messages for better debugging.
#[test]
#[ignore = "GREEN phase: Implement after fixing parser state"]
fn test_regression_087_error_messages_include_line_numbers() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_syntax_error.ruchy");

    // Intentionally broken code
    let code = r#"
fun main() {
    let x = {
        println!("test");
    // Missing closing brace
    println!("This should show line number");
}
"#;

    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd().arg("run").arg(&test_file).assert().failure();

    // After GREEN phase, error should include line number
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("line") || stderr.contains(':'),
        "Error message should include line number information: {stderr}"
    );
}
