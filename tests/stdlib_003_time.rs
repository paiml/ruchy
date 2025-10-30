#![allow(missing_docs)]
// STDLIB-003: std::time Module Implementation
//
// GitHub Issue: https://github.com/paiml/ruchy/issues/55
//
// FEATURE: Implement std::time::now_millis() for timing measurements
// Use case: Compiler benchmarking and performance optimization infrastructure
//
// API:
// - std::time::now_millis() -> i64  // Milliseconds since Unix epoch
//
// ROOT CAUSE: Ruchy lacks time measurement capabilities, blocking INFRA-001/002/003
//
// FIX: Implement std::time module with now_millis() function
// - Interpreter mode: Use std::time::SystemTime in Rust
// - Transpiler mode: Generate Rust code calling std::time::SystemTime
//
// IMPACT: Unblocks compiler optimization infrastructure with real timing measurements

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp file with code and return path
fn write_temp_file(code: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).expect("Failed to write temp file");
    (temp_dir, file_path)
}

/// Test basic `std::time::now_millis()` functionality
#[test]
fn test_stdlib_003_basic_now_millis() {
    let code = r#"
let timestamp = std::time::now_millis()
println("Timestamp: {}", timestamp)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Timestamp:"));
}

/// Test that timestamp is reasonable (> Jan 1, 2023)
#[test]
fn test_stdlib_003_timestamp_reasonable() {
    let code = r#"
let timestamp = std::time::now_millis()
let jan_2023 = 1672531200000  # Jan 1, 2023 in ms
if timestamp > jan_2023 {
    println("✓ Timestamp is reasonable")
} else {
    println("✗ Timestamp too old: {}", timestamp)
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Timestamp is reasonable"));
}

/// Test elapsed time measurement
#[test]
fn test_stdlib_003_elapsed_time() {
    let code = r#"
let start = std::time::now_millis()
let end = std::time::now_millis()
let elapsed = end - start
println("Elapsed: {} ms", elapsed)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Elapsed:"));
}

/// Test benchmarking pattern (from Issue #55)
#[test]
fn test_stdlib_003_benchmark_pattern() {
    let code = r#"
fun benchmark_operation() -> i64 {
    let start = std::time::now_millis()
    # Simulated work
    let result = 42 + 42
    let end = std::time::now_millis()
    end - start
}

let duration = benchmark_operation()
println("Duration: {} ms", duration)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Duration:"));
}

/// Test transpiler mode generates correct Rust code
#[test]
fn test_stdlib_003_transpile() {
    let code = r#"
let timestamp = std::time::now_millis()
println("Time: {}", timestamp)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("transpile")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("SystemTime"))
        .stdout(predicate::str::contains("duration_since"));
}

/// Test check command passes
#[test]
fn test_stdlib_003_check() {
    let code = r"
let timestamp = std::time::now_millis()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("check")
        .arg(file_path)
        .assert()
        .success();
}

/// Test lint command passes
#[test]
fn test_stdlib_003_lint() {
    let code = r"
let timestamp = std::time::now_millis()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("lint")
        .arg(file_path)
        .assert()
        .success();
}

/// Test AST structure for `std::time::now_millis()` call
#[test]
fn test_stdlib_003_ast() {
    let code = r"
std::time::now_millis()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("ast")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Call"))
        .stdout(predicate::str::contains("time"));
}

/// Test multiple sequential calls return different values
#[test]
fn test_stdlib_003_time_advances() {
    let code = r#"
let t1 = std::time::now_millis()
let t2 = std::time::now_millis()
if t2 >= t1 {
    println("✓ Time advances")
} else {
    println("✗ Time went backwards!")
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Time advances"));
}

/// Test compile command works
#[test]
fn test_stdlib_003_compile() {
    let code = r#"
let timestamp = std::time::now_millis()
println("Compiled at: {}", timestamp)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("compile")
        .arg(file_path)
        .assert()
        .success();
}
