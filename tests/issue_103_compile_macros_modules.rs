//! Issue #103: ruchy compile broken - macros and modules not supported
//!
//! Tests compilation of programs using macros (println!, format!) and module imports.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/103>
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)
//!
//! **BUG 1**: Macros fail with "Unsupported expression kind: `MacroInvocation`"
//! **BUG 2**: Module imports fail with "Failed to resolve import module"

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
// MACRO COMPILATION TESTS (BUG 1)
// ============================================================================

#[test]
fn test_issue_103_compile_println_macro() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "println_test.ruchy",
        r#"
fun main() {
    println!("Hello from compiled binary")
}

main()
"#,
    );
    let output = temp.path().join("println_test");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));

    // Binary should exist and be executable
    assert!(output.exists(), "Binary should be created");
}

#[test]
fn test_issue_103_compile_format_macro() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "format_test.ruchy",
        r#"
let name = "Ruchy"
let msg = format!("Hello, {}", name)
println!("{}", msg)
"#,
    );
    let output = temp.path().join("format_test");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Binary should be created");
}

#[test]
fn test_issue_103_compile_multiple_macros() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "multi_macro.ruchy",
        r#"
let x = 5
let y = 3
let result = x + y
println!("Calculating: {} + {} = {}", x, y, result)
let msg = format!("Result: {}", result)
println!("{}", msg)
"#,
    );
    let output = temp.path().join("multi_macro");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(
        output.exists(),
        "Binary with multiple macros should compile"
    );
}

// ============================================================================
// MODULE IMPORT TESTS (BUG 2)
// ============================================================================

#[test]
fn test_issue_103_compile_simple_import() {
    let temp = TempDir::new().unwrap();

    // Create module file
    let module_file = create_temp_file(
        &temp,
        "math_utils.ruchy",
        r"
fun add(x, y) {
    x + y
}

fun multiply(x, y) {
    x * y
}
",
    );

    // Create main file that imports the module
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
use math_utils::{add, multiply}

let result = add(5, 3)
println!("Result: {}", result)
"#,
    );

    let output = temp.path().join("import_test");

    ruchy_cmd()
        .arg("compile")
        .arg(&main_file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Binary with module import should compile");
}

#[test]
fn test_issue_103_compile_import_specific_functions() {
    let temp = TempDir::new().unwrap();

    // Create utility module
    create_temp_file(
        &temp,
        "utils.ruchy",
        r"
fun square(n) {
    n * n
}

fun cube(n) {
    n * n * n
}
",
    );

    // Create main file with specific imports
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r#"
use utils::{square, cube}

fun main() {
    println!("Square: {}", square(4))
    println!("Cube: {}", cube(3))
}

main()
"#,
    );

    let output = temp.path().join("specific_import");

    ruchy_cmd()
        .arg("compile")
        .arg(&main_file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(
        output.exists(),
        "Binary with specific function imports should compile"
    );
}

// ============================================================================
// COMBINED TESTS (Macros + Imports)
// ============================================================================

#[test]
fn test_issue_103_compile_macros_and_imports() {
    let temp = TempDir::new().unwrap();

    // Create logger module
    create_temp_file(
        &temp,
        "logger.ruchy",
        r#"
fun log_info(msg) {
    println!("[INFO] {}", msg)
}

fun log_error(msg) {
    println!("[ERROR] {}", msg)
}
"#,
    );

    // Create main file using both imports and macros
    let main_file = create_temp_file(
        &temp,
        "app.ruchy",
        r#"
use logger

fun main() {
    logger.log_info("Application started")
    let result = 42
    println!("Computed result: {}", result)
    logger.log_error("Test error message")
}

main()
"#,
    );

    let output = temp.path().join("combined_test");

    ruchy_cmd()
        .arg("compile")
        .arg(&main_file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(
        output.exists(),
        "Binary using both macros and imports should compile"
    );
}

// ============================================================================
// BINARY EXECUTION TESTS (Verify compiled binary works)
// ============================================================================

#[test]
fn test_issue_103_compiled_binary_executes() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "exec_test.ruchy",
        r#"
fun main() {
    println!("Execution test")
}

main()
"#,
    );
    let output = temp.path().join("exec_binary");

    // Compile the binary
    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    // Execute the compiled binary
    Command::new(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution test"));
}

#[test]
fn test_issue_103_compiled_binary_with_imports_executes() {
    let temp = TempDir::new().unwrap();

    // Create helper module
    create_temp_file(
        &temp,
        "helper.ruchy",
        r#"
fun get_message() {
    "Module execution works!"
}
"#,
    );

    // Create main file
    let main_file = create_temp_file(
        &temp,
        "main.ruchy",
        r"
use helper

fun main() {
    let msg = helper.get_message()
    println!(msg)
}

main()
",
    );

    let output = temp.path().join("import_exec");

    // Compile
    ruchy_cmd()
        .arg("compile")
        .arg(&main_file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    // Execute
    Command::new(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Module execution works!"));
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_103_compile_missing_module() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "missing_import.ruchy",
        r#"
use nonexistent_module

fun main() {
    println!("This should fail")
}
"#,
    );
    let output = temp.path().join("missing_mod");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Failed to resolve"))
                .or(predicate::str::contains("module")),
        );
}
