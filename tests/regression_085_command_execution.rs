#![allow(missing_docs)]
// Regression tests for GitHub Issue #85: Missing std::process::Command execution
// https://github.com/paiml/ruchy/issues/85
//
// REGRESSION INFO:
// - Working Version: None (never implemented)
// - Broken Versions: All versions prior to v3.148.0
// - Error: "Runtime error: Undefined variable: Command"
// - Type: Missing stdlib feature
//
// ROOT CAUSE: std::process::Command was never implemented
//   - No Command::new() handler in eval_qualified_name
//   - No Command methods (.arg, .output, .status) implemented
//   - No String::from_utf8() for byte array conversion
//   - Pattern matching didn't support EnumVariant for Ok/Err
//
// SOLUTION: Comprehensive std::process::Command implementation
//   - Added Command::new() to eval_qualified_name (interpreter.rs:2059)
//   - Implemented Command methods in eval_method_dispatch.rs:
//     - .arg() builds argument list (line 240-259)
//     - .output() executes and returns Result<Output, Error> (line 306-361)
//     - .status() executes and returns Result<ExitStatus, Error> (line 260-305)
//   - Implemented String::from_utf8() in eval_builtin.rs:2936-2976
//   - Enhanced pattern matching for Ok/Err EnumVariant (eval_pattern_match.rs:195-261)
//   - Added ExitStatus.success() method (eval_method_dispatch.rs:384-412)
//   - All 4 regression tests pass ✅
//
// Test naming convention: test_regression_085_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Test #1: Basic `Command::new()` and `output()` (minimal reproduction from Issue #85)
/// This is the exact test case reported in the GitHub issue.
#[test]
fn test_regression_085_command_basic_output() {
    let code = r#"
use std::process::Command;

fun main() {
    let output = Command::new("echo")
        .arg("Hello from Ruchy!")
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8(result.stdout);
            match stdout {
                Ok(text) => println!("{}", text),
                Err(_) => println!("Failed to decode stdout")
            }
        }
        Err(_) => println!("Command failed")
    }
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from Ruchy!"));
}

/// Test #2: Command with status checking
/// Verifies that Command execution returns proper status codes
#[test]
fn test_regression_085_command_status() {
    let code = r#"
use std::process::Command;

fun main() {
    let status = Command::new("echo")
        .arg("test")
        .status();

    match status {
        Ok(s) => {
            if s.success() {
                println!("Success: true");
            } else {
                println!("Success: false");
            }
        }
        Err(_) => println!("Command failed")
    }
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success: true"));
}

/// Test #3: Command with multiple arguments
/// Verifies that Command handles multiple arguments correctly
#[test]
fn test_regression_085_command_multiple_args() {
    let code = r#"
use std::process::Command;

fun main() {
    let output = Command::new("printf")
        .arg("%s %s\n")
        .arg("Hello")
        .arg("World")
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8(result.stdout);
            match stdout {
                Ok(text) => println!("{}", text),
                Err(_) => println!("Failed")
            }
        }
        Err(_) => println!("Command failed")
    }
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

/// Test #4: Command error handling (non-existent command)
/// Verifies that Command properly handles errors
#[test]
fn test_regression_085_command_error_handling() {
    let code = r#"
use std::process::Command;

fun main() {
    let output = Command::new("nonexistent_command_12345")
        .output();

    match output {
        Ok(_) => println!("Unexpected success"),
        Err(_) => println!("Error handled correctly")
    }
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Error handled correctly"));
}
