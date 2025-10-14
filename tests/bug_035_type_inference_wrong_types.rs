//! BUG-035: Type Inference Generates Incorrect Types
//!
//! **Problem**: Type inference generates i32 for all parameters, even string paths
//! **Discovered**: GitHub Issue #35
//! **Severity**: MEDIUM - Generates incorrect type signatures
//!
//! **Expected**: Smart inference based on built-in function signatures
//! **Actual**: Defaults to i32 for parameters used as function arguments
//!
//! **Root Cause**: is_param_used_as_function_argument() doesn't check what function
//! the parameter is passed to, just returns true → defaults to i32
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: fs_read parameter should be &str, not i32
#[test]
fn test_bug_035_red_fs_read_parameter_type() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun read_file(path) {
    fs_read(path)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Transpile and check generated Rust
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let transpiled = String::from_utf8_lossy(&output);

    // RED: Currently generates: fn read_file(path: i32)
    // Should generate: fn read_file(path: &str)
    assert!(
        transpiled.contains("path: &str") || transpiled.contains("path : & str"),
        "Parameter should be &str for file path, found: {}",
        transpiled
    );
}

/// Test 2: env_args doesn't need parameters - return type inference
#[test]
fn test_bug_035_red_env_args_return_type() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_args() {
    env_args()
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Transpile and check return type
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let transpiled = String::from_utf8_lossy(&output);

    // Should NOT default to i32 for return type
    // env_args() returns Vec<String>, so function should return Vec<String> or similar
    assert!(
        !transpiled.contains("-> i32"),
        "Return type should not be i32 for env_args, found: {}",
        transpiled
    );
}

/// Test 3: http_get parameter should be &str (URL), not i32
#[test]
fn test_bug_035_red_http_get_parameter_type() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun fetch_data(url) {
    http_get(url)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let transpiled = String::from_utf8_lossy(&output);

    assert!(
        transpiled.contains("url: &str") || transpiled.contains("url : & str"),
        "Parameter should be &str for URL, found: {}",
        transpiled
    );
}

/// Test 4: Numeric parameters should still be i32 (baseline)
#[test]
fn test_bug_035_baseline_numeric_parameter() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun add(a, b) {
    a + b
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let transpiled = String::from_utf8_lossy(&output);

    // Numeric operations should still infer i32 (baseline - already working)
    assert!(
        transpiled.contains("a: i32") || transpiled.contains("a : i32"),
        "Numeric parameters should be i32, found: {}",
        transpiled
    );
}

/// Test 5: Compilation should succeed with correct types
#[test]
fn test_bug_035_red_compilation_succeeds() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun read_config(path) {
    fs_read(path)
}

fun main() {
    let content = read_config("config.txt");
    println(content);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should compile successfully with correct type inference
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document the RED phase
#[test]
fn test_bug_035_red_phase_summary() {
    println!("BUG-035 RED Phase: Type Inference Wrong Types");
    println!("");
    println!("Problem: Parameters default to i32 even for string paths");
    println!("Impact: Incorrect type signatures in generated Rust");
    println!("");
    println!("Test Suite Created:");
    println!("1. fs_read parameter type (&str expected, i32 actual)");
    println!("2. env_args return type (not i32)");
    println!("3. http_get parameter type (&str expected)");
    println!("4. Numeric parameters - baseline (i32 correct)");
    println!("5. Compilation succeeds with correct types");
    println!("");
    println!("Expected Results:");
    println!("- RED Phase: Tests 1-3, 5 FAIL (wrong types)");
    println!("- RED Phase: Test 4 PASS (baseline)");
    println!("- GREEN Phase: ALL tests PASS after fix");
    println!("");
    println!("Fix Strategy:");
    println!("- Check built-in function signatures to infer parameter types");
    println!("- fs_read(path: &str) → infer path parameter as &str");
    println!("- http_get(url: &str) → infer url parameter as &str");
    println!("- Keep i32 inference for numeric operations");
}
