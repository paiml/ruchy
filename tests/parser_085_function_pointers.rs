//! PARSER-085: Function Pointer Support (GitHub Issue #70)
//!
//! Tests for function pointer type syntax (`fn()` types) and higher-order functions.
//!
//! Discovered during: RUCHY-005 Deno Updater conversion (2025-10-28)
//! Blocks: ubuntu-config-scripts TypeScript→Ruchy conversions
//!
//! Reference: <https://github.com/paiml/ruchy/issues/70>
//!
//! EXTREME TDD PROTOCOL:
//! ✅ RED Phase: This test file (all tests should FAIL)
//! ⏸️ GREEN Phase: Implement parser support for `fn()` type syntax
//! ⏸️ REFACTOR Phase: Apply quality gates (complexity ≤10)

use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

/// Test parsing simple function pointer type (no parameters, no return)
#[test]
fn test_parser_085_01_simple_function_pointer_type() {
    let code = r"
fun run(f: fn()) {
    f();
}

fun main() {}
";

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    // RED: Expected to FAIL until parser implements fn() type syntax
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test parsing function pointer with parameters
#[test]
fn test_parser_085_02_function_pointer_with_params() {
    let code = r"
fun apply(f: fn(i32), x: i32) {
    f(x);
}

fun main() {}
";

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test parsing function pointer with return type
#[test]
fn test_parser_085_03_function_pointer_with_return() {
    let code = r"
fun transform(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fun main() {}
";

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test parsing function pointer with multiple parameters
#[test]
fn test_parser_085_04_function_pointer_multiple_params() {
    let code = r"
fun combine(f: fn(i32, i32) -> i32, a: i32, b: i32) -> i32 {
    f(a, b)
}

fun main() {}
";

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test calling function pointer (original failing case from Issue #70)
/// FIXED: GitHub Issue #71 resolved - &mut now parses correctly
#[test]
fn test_parser_085_05_call_function_pointer() {
    let code = r#"
fun hello() {
    println!("Hello");
}

fun run_test(test_fn: fn(), passed: &mut i32) {
    test_fn();
    *passed += 1;
}

fun main() {
    let mut count = 0;
    run_test(hello, &mut count);
    println!("Count: {}", count);
}
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Integration test: Reproduces exact RUCHY-005 blocker scenario
/// FIXED: GitHub Issue #71 resolved - &mut now parses correctly
#[test]
fn test_parser_085_06_ruchy_005_blocker_reproduction() {
    // This is the EXACT code that blocked RUCHY-005 Deno Updater conversion
    let code = r#"
fun test_compare_versions_equal() {
    println!("TEST: compare_versions - equal versions");
}

fun run_test(test_fn: fn(), passed: &mut i32, failed: &mut i32) {
    test_fn();
    *passed += 1;
}

fun main() {
    let mut passed = 0;
    let mut failed = 0;
    run_test(test_compare_versions_equal, &mut passed, &mut failed);
    println!("Passed: {}", passed);
}
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test transpilation generates valid Rust code
#[test]
fn test_parser_085_07_transpile_function_pointer() {
    let code = r"
fun apply(f: fn(i32) -> i32, value: i32) -> i32 {
    f(value)
}

fun main() {}
";

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    // Test transpilation works
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test end-to-end evaluation works
/// EXTREME TDD: RED phase - this test should FAIL until GREEN phase implementation
#[test]
fn test_parser_085_08_eval_function_pointer() {
    let code = r#"
fun double(x: i32) -> i32 {
    x * 2
}

fun apply_op(f: fn(i32) -> i32, value: i32) -> i32 {
    f(value)
}

fun main() {
    let result = apply_op(double, 5);
    println!("{}", result);
}
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, code).unwrap();

    // Test evaluation works and outputs "10"
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicates::str::contains("10"));
}
