//! CLI Contract Tests: `ruchy provability`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, formal verification)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Verification completed successfully (all checks passed)
//! - Exit code 1: Verification failed OR file not found OR syntax error
//! - stdout: Verification report (contracts, invariants, termination, bounds)
//! - stderr: Error messages (verification errors, missing files)
//! - Options: --verify, --contracts, --invariants, --termination, --bounds, --verbose, --output
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (provability: 0.23 → target 0.5, HIGH RISK)
//!
//! **Note**: Provability tool is HIGH RISK (complexity 13, minimal test coverage)

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
// CLI CONTRACT TESTS: BASIC BEHAVIOR
// ============================================================================

#[test]
fn cli_provability_valid_program_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\nprintln(x)\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_provability_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("provability")
        .arg("nonexistent_xyz.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_provability_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_provability_outputs_verification_report() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verify_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Verification")
            .or(predicate::str::contains("Provability"))
            .or(predicate::str::contains("Formal")));
}

// ============================================================================
// CLI CONTRACT TESTS: VERIFY OPTION
// ============================================================================

#[test]
fn cli_provability_verify_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verify.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .assert()
        .success()
        .stdout(predicate::str::contains("verif")
            .or(predicate::str::contains("Verif"))
            .or(predicate::str::contains("formal")));
}

#[test]
fn cli_provability_verify_with_function() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "verify_func.ruchy",
        r#"
fun add(a, b) {
    a + b
}

add(1, 2)
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: CONTRACTS OPTION
// ============================================================================

#[test]
fn cli_provability_contracts_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "contracts.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--contracts")
        .assert()
        .success()
        .stdout(predicate::str::contains("contract")
            .or(predicate::str::contains("Contract"))
            .or(predicate::str::contains("condition")));
}

#[test]
fn cli_provability_contracts_with_precondition() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "contracts_pre.ruchy",
        r#"
fun divide(a, b) {
    if b == 0 {
        return 0
    }
    a / b
}

divide(10, 2)
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--contracts")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: INVARIANTS OPTION
// ============================================================================

#[test]
fn cli_provability_invariants_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invariants.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--invariants")
        .assert()
        .success()
        .stdout(predicate::str::contains("invariant")
            .or(predicate::str::contains("Invariant"))
            .or(predicate::str::contains("loop")));
}

#[test]
fn cli_provability_invariants_with_loop() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "invariants_loop.ruchy",
        r#"
let sum = 0
for i in range(10) {
    sum = sum + i
}
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--invariants")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: TERMINATION OPTION
// ============================================================================

#[test]
fn cli_provability_termination_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "termination.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--termination")
        .assert()
        .success()
        .stdout(predicate::str::contains("terminat")
            .or(predicate::str::contains("Terminat"))
            .or(predicate::str::contains("halts")));
}

#[test]
fn cli_provability_termination_with_loop() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "termination_loop.ruchy",
        r#"
let i = 0
while i < 10 {
    i = i + 1
}
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--termination")
        .assert()
        .success();
}

#[test]
fn cli_provability_termination_with_recursion() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "termination_recursion.ruchy",
        r#"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

factorial(5)
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--termination")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: BOUNDS OPTION
// ============================================================================

#[test]
fn cli_provability_bounds_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bounds.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--bounds")
        .assert()
        .success()
        .stdout(predicate::str::contains("bounds")
            .or(predicate::str::contains("Bounds"))
            .or(predicate::str::contains("array")));
}

#[test]
fn cli_provability_bounds_with_array() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "bounds_array.ruchy",
        r#"
let arr = [1, 2, 3, 4, 5]
for i in range(5) {
    println(arr[i])
}
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--bounds")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: COMBINED OPTIONS
// ============================================================================

#[test]
fn cli_provability_verify_and_contracts() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "combined1.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .arg("--contracts")
        .assert()
        .success();
}

#[test]
fn cli_provability_all_flags() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "all_flags.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .arg("--contracts")
        .arg("--invariants")
        .arg("--termination")
        .arg("--bounds")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT FILE
// ============================================================================

#[test]
fn cli_provability_output_to_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output_test.ruchy", "let x = 42\n");
    let output_file = temp.path().join("provability_report.txt");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Verify output file was created
    assert!(output_file.exists(), "Output file should be created");
}

#[test]
fn cli_provability_output_with_verify() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output_verify.ruchy", "let x = 42\n");
    let output_file = temp.path().join("verify_report.txt");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    assert!(output_file.exists(), "Output file should be created");
}

// ============================================================================
// CLI CONTRACT TESTS: VERBOSE MODE
// ============================================================================

#[test]
fn cli_provability_verbose_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn cli_provability_verbose_with_verify() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose_verify.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verbose")
        .arg("--verify")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
fn cli_provability_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("provability")
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
fn cli_provability_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_provability_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    // Empty files should fail with parse error
    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unexpected end of input")
            .or(predicate::str::contains("Parse error")));
}

#[test]
fn cli_provability_complex_program() {
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
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: HELP AND USAGE
// ============================================================================

#[test]
fn cli_provability_help_flag() {
    ruchy_cmd()
        .arg("provability")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("provability")
            .or(predicate::str::contains("Provability"))
            .or(predicate::str::contains("verification")));
}

// ============================================================================
// CLI CONTRACT TESTS: VERIFICATION SCENARIOS
// ============================================================================

#[test]
fn cli_provability_safe_array_access() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "safe_array.ruchy",
        r#"
let arr = [1, 2, 3]
for i in range(3) {
    println(arr[i])
}
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--bounds")
        .assert()
        .success();
}

#[test]
fn cli_provability_loop_invariant_maintained() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "loop_invariant.ruchy",
        r#"
let sum = 0
let i = 0
while i < 10 {
    sum = sum + i
    i = i + 1
}
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--invariants")
        .assert()
        .success();
}

#[test]
fn cli_provability_terminating_recursion() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "terminating_recursion.ruchy",
        r#"
fun countdown(n) {
    if n <= 0 {
        println("Done!")
    } else {
        println(n)
        countdown(n - 1)
    }
}

countdown(5)
"#,
    );

    ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--termination")
        .assert()
        .success();
}
