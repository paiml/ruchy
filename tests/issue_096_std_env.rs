//! EXTREME TDD - RED Phase: Tests for Issue #96 (`std::env` module)
//!
//! GitHub Issue: #96 - `std::env` module not available
//! Severity: CRITICAL
//! Impact: Blocks all CLI applications requiring command-line argument parsing
//!
//! These tests will FAIL initially (RED phase) until we implement `std::env` module.

use predicates::prelude::*;

/// Test basic `env::args()` access
/// Tests that the use `std::env` import works and `env::args()` returns program arguments
#[test]
#[ignore = "BUG: std::env not working"]
fn test_issue_096_env_args_basic() {
    let script = r#"
use std::env;

fun main() {
    let args = env::args();
    println!("Args count: {}", args.len());
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_env_args_basic.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Args count: 1")); // At minimum, the script path

    std::fs::remove_file(temp_file).unwrap();
}

/// Test `env::args()` with multiple arguments
/// In the future when we support passing args to scripts
#[test]
#[ignore = "Script argument passing not yet implemented in CLI"]
fn test_issue_096_env_args_multiple() {
    let script = r#"
use std::env;

fun main() {
    let args = env::args();
    println!("Total args: {}", args.len());

    let mut i = 0;
    while i < args.len() {
        println!("Arg {}: {}", i, args[i]);
        i = i + 1;
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_env_args_multiple.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .arg("arg1")
        .arg("arg2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total args: 3"))
        .stdout(predicate::str::contains("Arg 1: arg1"))
        .stdout(predicate::str::contains("Arg 2: arg2"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test `env::var()` for reading environment variables
#[test]
fn test_issue_096_env_var_get() {
    let script = r#"
use std::env;

fun main() {
    // Set a test environment variable for this test
    let result = env::var("RUCHY_TEST_VAR");
    match result {
        Ok(value) => println!("Value: {}", value),
        Err(_) => println!("Not found"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_env_var_get.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .env("RUCHY_TEST_VAR", "test_value_123")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Value: test_value_123"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test `env::var()` for missing environment variable
#[test]
fn test_issue_096_env_var_not_found() {
    let script = r#"
use std::env;

fun main() {
    let result = env::var("RUCHY_NONEXISTENT_VAR_XYZ");
    match result {
        Ok(value) => println!("Found: {}", value),
        Err(_) => println!("Not found"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_env_var_not_found.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .env_remove("RUCHY_NONEXISTENT_VAR_XYZ") // Ensure it doesn't exist
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Not found"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test `env::args()` returns Vec with at least the program name
#[test]
fn test_issue_096_env_args_nonempty() {
    let script = r#"
use std::env;

fun main() {
    let args = env::args();
    if args.len() > 0 {
        println!("Has args: yes");
        println!("First arg exists: yes");
    } else {
        println!("Has args: no");
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_env_args_nonempty.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Has args: yes"))
        .stdout(predicate::str::contains("First arg exists: yes"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test that use `std::env` import doesn't fail
#[test]
fn test_issue_096_std_env_import() {
    let script = r#"
use std::env;

fun main() {
    println!("Import successful");
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_std_env_import.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Import successful"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test `env::var()` with common environment variables (HOME, PATH, USER)
#[test]
fn test_issue_096_env_var_common() {
    let script = r#"
use std::env;

fun main() {
    // Try to get a common env var that should exist
    let home = env::var("HOME");
    match home {
        Ok(_) => println!("HOME exists: yes"),
        Err(_) => println!("HOME exists: no"),
    }
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_env_var_common.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("HOME exists: yes"));

    std::fs::remove_file(temp_file).unwrap();
}

/// Test real-world use case: CLI tool with argument parsing
#[test]
#[ignore = "BUG: std::env not working"]
fn test_issue_096_cli_tool_pattern() {
    let script = r#"
use std::env;

fun main() {
    let args = env::args();

    if args.len() < 2 {
        println!("Usage: program <command>");
        return;
    }

    println!("CLI tool started");
    println!("Args received: {}", args.len());
}
"#;

    let temp_file = std::env::temp_dir().join("issue_096_cli_tool_pattern.ruchy");
    std::fs::write(&temp_file, script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: program <command>"));

    std::fs::remove_file(temp_file).unwrap();
}
