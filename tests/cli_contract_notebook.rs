//! CLI Contract Tests: `ruchy notebook`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, notebook functionality)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Notebook validation/launch successful
//! - Exit code 1: Validation failed OR file not found OR syntax error
//! - stdout: Notebook output (validation mode) or server info (interactive mode)
//! - stderr: Error messages (validation errors, server errors)
//! - Options: --port, --open, --host
//! - Non-interactive mode: FILE argument for validation (TOOL-VALIDATION-003)
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (notebook: 0.38 → target 0.5, HIGH RISK)
//!
//! **Note**: Notebook tool is HIGH RISK (complexity 8, minimal test coverage)

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
// CLI CONTRACT TESTS: NON-INTERACTIVE FILE VALIDATION (TOOL-VALIDATION-003)
// ============================================================================

#[test]
fn cli_notebook_validate_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\nprintln(x)\n");

    // Non-interactive validation mode
    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_notebook_validate_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("notebook")
        .arg("nonexistent_xyz.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_notebook_validate_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_notebook_validate_outputs_success_message() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "validate_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid")
            .or(predicate::str::contains("Valid"))
            .or(predicate::str::contains("success")));
}

// ============================================================================
// CLI CONTRACT TESTS: PORT OPTION
// ============================================================================

#[test]
#[ignore] // Starts server, requires manual shutdown
fn cli_notebook_custom_port() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--port")
        .arg("9090")
        .timeout(std::time::Duration::from_secs(2))
        .assert(); // Will timeout but tests port parsing
}

#[test]
fn cli_notebook_invalid_port_format() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--port")
        .arg("invalid")
        .assert()
        .failure(); // Invalid port number
}

#[test]
fn cli_notebook_port_out_of_range() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--port")
        .arg("99999") // Port > 65535
        .assert()
        .failure(); // Port out of range
}

// ============================================================================
// CLI CONTRACT TESTS: HOST OPTION
// ============================================================================

#[test]
#[ignore] // Starts server
fn cli_notebook_custom_host() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--host")
        .arg("0.0.0.0")
        .timeout(std::time::Duration::from_secs(2))
        .assert(); // Will timeout but tests host parsing
}

#[test]
#[ignore] // Starts server
fn cli_notebook_localhost_host() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--host")
        .arg("localhost")
        .timeout(std::time::Duration::from_secs(2))
        .assert();
}

// ============================================================================
// CLI CONTRACT TESTS: OPEN OPTION
// ============================================================================

#[test]
#[ignore] // Starts server and opens browser
fn cli_notebook_open_browser_flag() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--open")
        .timeout(std::time::Duration::from_secs(2))
        .assert(); // Will timeout but tests --open flag
}

// ============================================================================
// CLI CONTRACT TESTS: COMBINED OPTIONS WITH FILE VALIDATION
// ============================================================================

#[test]
fn cli_notebook_validate_with_port_option() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "port_test.ruchy", "let x = 42\n");

    // Port option should be ignored in file validation mode
    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .arg("--port")
        .arg("9090")
        .assert()
        .success();
}

#[test]
fn cli_notebook_validate_with_host_option() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "host_test.ruchy", "let x = 42\n");

    // Host option should be ignored in file validation mode
    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .arg("--host")
        .arg("localhost")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
fn cli_notebook_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("notebook")
        .arg("missing.ruchy")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

#[test]
fn cli_notebook_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_notebook_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    // Empty files should fail validation
    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty program")
            .or(predicate::str::contains("Parse error")));
}

#[test]
fn cli_notebook_complex_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "complex.ruchy",
        r#"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let result = factorial(5)
println(result)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn cli_notebook_notebook_with_cells() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "notebook_cells.ruchy",
        r#"
let x = 42
println(x)

let y = x * 2
println(y)

let z = y + x
println(z)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: HELP
// ============================================================================

#[test]
fn cli_notebook_help_flag() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("notebook")
            .or(predicate::str::contains("Notebook"))
            .or(predicate::str::contains("interactive")));
}

// ============================================================================
// CLI CONTRACT TESTS: VALIDATION SCENARIOS
// ============================================================================

#[test]
fn cli_notebook_validate_with_functions() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "functions.ruchy",
        r#"
fun add(a, b) {
    a + b
}

fun multiply(a, b) {
    a * b
}

let result1 = add(5, 3)
let result2 = multiply(result1, 2)
println(result2)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn cli_notebook_validate_with_loops() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "loops.ruchy",
        r#"
for i in range(10) {
    println(i)
}

let sum = 0
for i in range(5) {
    sum = sum + i
}
println(sum)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn cli_notebook_validate_with_conditionals() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "conditionals.ruchy",
        r#"
let x = 42

if x > 40 {
    println("x is greater than 40")
} else {
    println("x is not greater than 40")
}

let y = if x > 50 { 100 } else { 50 }
println(y)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn cli_notebook_validate_with_arrays() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "arrays.ruchy",
        r#"
let arr = [1, 2, 3, 4, 5]
for item in arr {
    println(item)
}

let sum = 0
for i in range(5) {
    sum = sum + arr[i]
}
println(sum)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn cli_notebook_validate_with_strings() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "strings.ruchy",
        r#"
let greeting = "Hello"
let name = "World"
let message = greeting + " " + name
println(message)

let upper = message.to_uppercase()
println(upper)
"#,
    );

    ruchy_cmd()
        .arg("notebook")
        .arg(&file)
        .assert()
        .success();
}
