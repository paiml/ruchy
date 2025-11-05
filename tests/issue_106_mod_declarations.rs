//! Issue #106: Support mod scanner; syntax (module declarations without braces)
//!
//! Tests Rust-style module declarations that reference external files.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/106>
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// BASIC MODULE DECLARATION TESTS
// ============================================================================

#[test]
fn test_issue_106_simple_mod_declaration() {
    let temp = TempDir::new().unwrap();

    // Create module file
    create_temp_file(
        &temp,
        "scanner.ruchy",
        r#"
pub fun scan() {
    println!("Scanning...")
}
"#,
    );

    // Create main file with mod declaration
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r"
mod scanner;

fun main() {
    scanner::scan()
}

main()
",
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning..."));
}

#[test]
fn test_issue_106_multiple_mod_declarations() {
    let temp = TempDir::new().unwrap();

    // Create first module
    create_temp_file(
        &temp,
        "utils.ruchy",
        r#"
pub fun helper() {
    "Helper function"
}
"#,
    );

    // Create second module
    create_temp_file(
        &temp,
        "logger.ruchy",
        r#"
pub fun log(msg) {
    println!("LOG: {}", msg)
}
"#,
    );

    // Create main file with multiple mod declarations
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r"
mod utils;
mod logger;

fun main() {
    let msg = utils::helper()
    logger::log(msg)
}

main()
",
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("LOG: Helper function"));
}

#[test]
fn test_issue_106_nested_module_calls() {
    let temp = TempDir::new().unwrap();

    // Create module with multiple functions
    create_temp_file(
        &temp,
        "math.ruchy",
        r"
pub fun add(x, y) {
    x + y
}

pub fun multiply(x, y) {
    x * y
}
",
    );

    // Create main file using both functions
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
mod math;

fun main() {
    let a = math::add(5, 3)
    let b = math::multiply(a, 2)
    println!("Result: {}", b)
}

main()
"#,
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 16"));
}

// ============================================================================
// COMPILATION TESTS (Issue #103 + Issue #106)
// ============================================================================

#[test]
fn test_issue_106_compile_with_mod_declaration() {
    let temp = TempDir::new().unwrap();

    // Create module file
    create_temp_file(
        &temp,
        "calculator.ruchy",
        r"
pub fun compute(x, y) {
    x + y
}
",
    );

    // Create main file with mod declaration
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
mod calculator;

fun main() {
    let result = calculator::compute(10, 20)
    println!("Result: {}", result)
}

main()
"#,
    );

    let output = temp.path().join("compiled_test");

    ruchy_cmd()
        .arg("compile")
        .arg(&main_file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));

    assert!(output.exists(), "Binary should be created");
}

#[test]
fn test_issue_106_compiled_binary_executes() {
    let temp = TempDir::new().unwrap();

    // Create module file
    create_temp_file(
        &temp,
        "processor.ruchy",
        r"
pub fun process(value) {
    value * 2
}
",
    );

    // Create main file
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
mod processor;

fun main() {
    let result = processor::process(42)
    println!("Processed: {}", result)
}

main()
"#,
    );

    let output = temp.path().join("binary_test");

    // Compile
    ruchy_cmd()
        .arg("compile")
        .arg(&main_file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    // Execute binary
    Command::new(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed: 84"));
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_106_missing_module_file() {
    let temp = TempDir::new().unwrap();

    // Create main file referencing non-existent module
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r"
mod nonexistent;

fun main() {
    nonexistent::function()
}
",
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to find module")
            .or(predicate::str::contains("Module 'nonexistent' not found")));
}

#[test]
fn test_issue_106_invalid_module_syntax() {
    let temp = TempDir::new().unwrap();

    // Create module file with syntax error
    create_temp_file(
        &temp,
        "broken.ruchy",
        r"
pub fun missing_body()
",
    );

    // Create main file
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r"
mod broken;

fun main() {
    broken::missing_body()
}
",
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Syntax error")
            .or(predicate::str::contains("Expected")));
}

// ============================================================================
// COMPATIBILITY TESTS (Ensure existing inline modules still work)
// ============================================================================

#[test]
fn test_issue_106_inline_modules_still_work() {
    let temp = TempDir::new().unwrap();

    // Use inline module syntax (should still work)
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
mod utils {
    pub fun greet() {
        println!("Hello from inline module")
    }
}

fun main() {
    utils::greet()
}

main()
"#,
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from inline module"));
}

#[test]
fn test_issue_106_mixed_inline_and_declaration() {
    let temp = TempDir::new().unwrap();

    // Create external module
    create_temp_file(
        &temp,
        "external.ruchy",
        r#"
pub fun external_func() {
    "From external"
}
"#,
    );

    // Mix inline and declaration syntax
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
mod external;

mod inline {
    pub fun inline_func() {
        "From inline"
    }
}

fun main() {
    println!("{}", external::external_func())
    println!("{}", inline::inline_func())
}

main()
"#,
    );

    ruchy_cmd()
        .arg(main_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("From external"))
        .stdout(predicate::str::contains("From inline"));
}
