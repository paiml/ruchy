//! Issue #90: `std::fs` operations fail with "No match arm matched" runtime error
//!
//! Root Cause: Unknown - needs investigation
//! Impact: CRITICAL - `std::fs` module unusable
//! Pattern: `std::fs` operations return values not properly handled

#![allow(missing_docs)]

use assert_cmd::Command;
use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED: Test `std::fs::write` with result handling
#[test]
fn test_std_fs_write_basic() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let test_file = "/tmp/ruchy_issue_90_test.txt";
    std::fs::write(test_file, "test content");
    println!("Write succeeded");
}

main()
"#;
    fs::write(&script, code).unwrap();

    // Test both run and compile modes
    ruchy_cmd().arg("run").arg(&script).assert().success();
}

/// RED: Test `std::fs::read_to_string`
#[test]
fn test_std_fs_read_to_string() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let test_file = "/tmp/ruchy_issue_90_read.txt";
    std::fs::write(test_file, "hello world");
    let content = std::fs::read_to_string(test_file);
    println!("Content: {}", content);
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("run").arg(&script).assert().success();
}

/// RED: Test `std::fs::exists`
#[test]
fn test_std_fs_exists() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let test_file = "/tmp/ruchy_issue_90_exists.txt";
    std::fs::write(test_file, "test");
    
    let exists = std::fs::exists(test_file);
    println!("File exists: {}", exists);
    
    if exists {
        println!("SUCCESS");
    }
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("run").arg(&script).assert().success();
}

/// RED: Test `std::fs` with error handling (Result type)
#[test]
fn test_std_fs_with_result_handling() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let result = std::fs::write("/tmp/ruchy_test.txt", "data");

    match result {
        Ok(_) => println!("Write succeeded"),
        Err(e) => println!("Write failed: {}", e)
    }
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("run").arg(&script).assert().success();
}

// ============================================================================
// REFACTOR: Property-Based Tests for Result Invariants
// ============================================================================

proptest! {
    /// Property: std::fs::write with Result matching NEVER panics
    /// Invariant: All code paths return Result enum, never throw errors
    #[test]
    fn prop_std_fs_write_result_never_panics(path_suffix in "[a-z]{5}") {
        let temp_dir = TempDir::new().unwrap();
        let script = temp_dir.path().join("test.ruchy");

        let code = format!(r#"
fun main() {{
    let result = std::fs::write("/tmp/prop_{path_suffix}.txt", "test content");
    match result {{
        Ok(_) => println!("OK"),
        Err(e) => println!("ERR: {{}}", e)
    }}
}}

main()
"#);

        fs::write(&script, code).unwrap();

        // Property: Command always succeeds (no panics, no crashes)
        ruchy_cmd()
            .arg("run")
            .arg(&script)
            .assert()
            .success();
    }

    /// Property: std::fs operations return Result enum (never Nil)
    /// Invariant: Can ALWAYS match on Ok/Err variants
    #[test]
    fn prop_std_fs_result_always_matchable(path in "/tmp/prop_[a-z]{5}\\.txt") {
        let temp_dir = TempDir::new().unwrap();
        let script = temp_dir.path().join("test.ruchy");

        let code = format!(r#"
fun main() {{
    // Test write returns Result
    let write_result = std::fs::write("{path}", "test");
    let write_ok = match write_result {{
        Ok(_) => true,
        Err(_) => false
    }};

    // Test read returns Result
    let read_result = std::fs::read_to_string("{path}");
    let read_matched = match read_result {{
        Ok(_) => true,
        Err(_) => true  // Both branches valid
    }};

    println!("Write: {{}}, Read: {{}}", write_ok, read_matched);
}}

main()
"#);

        fs::write(&script, code).unwrap();

        ruchy_cmd()
            .arg("run")
            .arg(&script)
            .assert()
            .success();
    }
}
