// RED Phase Test for Issue #88: Module system (use imports) not working
//
// GitHub Issue: https://github.com/YourRepo/ruchy/issues/88
//
// Problem: Parser creates Import AST nodes, but interpreter doesn't load modules
// Error: "Undefined variable: mylib"
//
// Expected: ModuleLoader should load .ruchy files and add symbols to scope
//
// EXTREME TDD Methodology:
// 1. RED: Create failing test using exact reproduction from Issue #88
// 2. GREEN: Wire ModuleLoader into interpreter's Import handling
// 3. REFACTOR: Add property tests for circular dependencies

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// RED: Test basic module import (exact reproduction from Issue #88)
///
/// This is the minimal failing case from the GitHub issue.
#[test]
#[ignore = "RED phase: module imports not yet implemented - ISSUE-088"]
fn test_issue_088_basic_module_import() {
    let temp_dir = TempDir::new().unwrap();

    // Create mylib.ruchy
    let mylib_file = temp_dir.path().join("mylib.ruchy");
    let mylib_code = r"
// Simple library module for testing
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun multiply(a: i32, b: i32) -> i32 {
    a * b
}
";
    fs::write(&mylib_file, mylib_code).unwrap();

    // Create main.ruchy
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r#"
// Test: Import using 'use' syntax
use mylib;

fun main() {
    println("Testing Ruchy module import...");
    let result = mylib::add(2, 3);
    println(result);
}
"#;
    fs::write(&main_file, main_code).unwrap();

    // RED: Currently fails with "Undefined variable: mylib"
    // GREEN: Should output "5"
    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Testing Ruchy module import..."),
        "Expected greeting, got: {stdout}"
    );
    assert!(
        stdout.contains('5'),
        "Expected result 5 from mylib::add(2, 3), got: {stdout}"
    );
}

/// RED: Test module import with multiple functions
#[test]
#[ignore = "RED phase: module imports not yet implemented - ISSUE-088"]
fn test_issue_088_multiple_function_calls() {
    let temp_dir = TempDir::new().unwrap();

    // Create math.ruchy
    let math_file = temp_dir.path().join("math.ruchy");
    let math_code = r"
fun double(x: i32) -> i32 {
    x * 2
}

fun triple(x: i32) -> i32 {
    x * 3
}
";
    fs::write(&math_file, math_code).unwrap();

    // Create main.ruchy
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r"
use math;

fun main() {
    let a = math::double(5);
    let b = math::triple(5);
    println(a);
    println(b);
}
";
    fs::write(&main_file, main_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("10"),
        "Expected 10 from double(5), got: {stdout}"
    );
    assert!(
        stdout.contains("15"),
        "Expected 15 from triple(5), got: {stdout}"
    );
}

/// RED: Test nested module calls
#[test]
#[ignore = "RED phase: module imports not yet implemented - ISSUE-088"]
fn test_issue_088_nested_module_calls() {
    let temp_dir = TempDir::new().unwrap();

    // Create utils.ruchy
    let utils_file = temp_dir.path().join("utils.ruchy");
    let utils_code = r"
fun square(x: i32) -> i32 {
    x * x
}
";
    fs::write(&utils_file, utils_code).unwrap();

    // Create main.ruchy
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r"
use utils;

fun main() {
    let result = utils::square(utils::square(2));
    println(result);
}
";
    fs::write(&main_file, main_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // square(2) = 4, square(4) = 16
    assert!(
        stdout.contains("16"),
        "Expected 16 from square(square(2)), got: {stdout}"
    );
}

/// RED: Test module with variables
#[test]
#[ignore = "RED phase: module imports not yet implemented - ISSUE-088"]
fn test_issue_088_module_with_constants() {
    let temp_dir = TempDir::new().unwrap();

    // Create constants.ruchy
    let constants_file = temp_dir.path().join("constants.ruchy");
    let constants_code = r"
let PI = 3;
let MAX_SIZE = 100;

fun get_pi() -> i32 {
    PI
}
";
    fs::write(&constants_file, constants_code).unwrap();

    // Create main.ruchy
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r"
use constants;

fun main() {
    let pi_value = constants::get_pi();
    println(pi_value);
}
";
    fs::write(&main_file, main_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains('3'),
        "Expected 3 from constants::get_pi(), got: {stdout}"
    );
}

/// RED: Test module not found error
///
/// This should give a clear error, not "Undefined variable"
#[test]
fn test_issue_088_module_not_found() {
    let temp_dir = TempDir::new().unwrap();

    // Create main.ruchy (but NOT nonexistent.ruchy)
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r#"
use nonexistent;

fun main() {
    println("This should fail");
}
"#;
    fs::write(&main_file, main_code).unwrap();

    // Should fail with module not found error
    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    // Should mention "module" not "Undefined variable"
    assert!(
        stderr.contains("module") || stderr.contains("file") || stderr.contains("not found"),
        "Expected module/file not found error, got: {stderr}"
    );
}

/// GREEN: Verify existing single-file programs still work
///
/// Module system shouldn't break programs without imports
#[test]
#[ignore = "RED phase: module imports not yet implemented - ISSUE-088"]
fn test_issue_088_no_imports_still_works() {
    let temp_dir = TempDir::new().unwrap();

    let test_file = temp_dir.path().join("simple.ruchy");
    let code = r"
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun main() {
    let result = add(2, 3);
    println(result);
}
";
    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("simple.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains('5'), "Expected 5, got: {stdout}");
}
