//! CLI Contract Tests: `ruchy runtime`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, performance analysis)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Runtime analysis completed successfully
//! - Exit code 1: Analysis failed OR file not found OR syntax error
//! - stdout: Performance report (profiling, `BigO`, benchmarks, memory)
//! - stderr: Error messages (analysis errors, missing files)
//! - Options: --profile, --bigo, --bench, --compare, --memory, --verbose, --output
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (runtime: 0.3 → target 0.5, HIGH RISK)
//!
//! **Note**: Runtime tool is HIGH RISK (complexity 20, minimal test coverage)

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
fn cli_runtime_valid_program_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\nprintln(x)\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_runtime_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("runtime")
        .arg("nonexistent_xyz.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_runtime_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_runtime_outputs_performance_report() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "perf_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Performance")
            .or(predicate::str::contains("Runtime"))
            .or(predicate::str::contains("Execution")));
}

// ============================================================================
// CLI CONTRACT TESTS: PROFILE OPTION
// ============================================================================

#[test]
fn cli_runtime_profile_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "profile_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("profil")
            .or(predicate::str::contains("Profil"))
            .or(predicate::str::contains("Performance")));
}

#[test]
fn cli_runtime_profile_with_loop() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "profile_loop.ruchy",
        r"
for i in range(10) {
    println(i)
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--profile")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: BIGO OPTION
// ============================================================================

#[test]
fn cli_runtime_bigo_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bigo_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("BigO")
            .or(predicate::str::contains("O("))
            .or(predicate::str::contains("complexity")));
}

#[test]
fn cli_runtime_bigo_with_loop() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "bigo_loop.ruchy",
        r"
for i in range(100) {
    let x = i * 2
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("O(")
            .or(predicate::str::contains("complexity")));
}

#[test]
fn cli_runtime_bigo_with_nested_loops() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "bigo_nested.ruchy",
        r"
for i in range(10) {
    for j in range(10) {
        let x = i * j
    }
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: BENCH OPTION
// ============================================================================

#[test]
fn cli_runtime_bench_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bench_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bench")
        .assert()
        .success()
        .stdout(predicate::str::contains("bench")
            .or(predicate::str::contains("Bench"))
            .or(predicate::str::contains("statistical")));
}

#[test]
fn cli_runtime_bench_with_computation() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "bench_compute.ruchy",
        r"
let sum = 0
for i in range(100) {
    sum = sum + i
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bench")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: COMPARE OPTION
// ============================================================================

#[test]
fn cli_runtime_compare_two_files() {
    let temp = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp, "file1.ruchy", "let x = 42\n");
    let file2 = create_temp_file(&temp, "file2.ruchy", "let y = 100\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file1)
        .arg("--compare")
        .arg(&file2)
        .assert()
        .success()
        .stdout(predicate::str::contains("compar")
            .or(predicate::str::contains("Compar"))
            .or(predicate::str::contains("vs")));
}

#[test]
fn cli_runtime_compare_with_nonexistent_baseline() {
    let temp = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp, "file1.ruchy", "let x = 42\n");

    // Note: Compare succeeds even with nonexistent baseline
    // This is by design - allows comparing against a baseline that will be created later
    ruchy_cmd()
        .arg("runtime")
        .arg(&file1)
        .arg("--compare")
        .arg("nonexistent.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Comparison")
            .or(predicate::str::contains("Baseline"))
            .or(predicate::str::contains("faster")));
}

// ============================================================================
// CLI CONTRACT TESTS: MEMORY OPTION
// ============================================================================

#[test]
fn cli_runtime_memory_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "memory_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--memory")
        .assert()
        .success()
        .stdout(predicate::str::contains("memory")
            .or(predicate::str::contains("Memory"))
            .or(predicate::str::contains("allocation")));
}

#[test]
fn cli_runtime_memory_with_allocations() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "memory_alloc.ruchy",
        r"
let arr = [1, 2, 3, 4, 5]
let sum = 0
for i in arr {
    sum = sum + i
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--memory")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: COMBINED OPTIONS
// ============================================================================

#[test]
fn cli_runtime_profile_and_bigo() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "combined1.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--profile")
        .arg("--bigo")
        .assert()
        .success();
}

#[test]
fn cli_runtime_all_flags() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "all_flags.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--profile")
        .arg("--bigo")
        .arg("--bench")
        .arg("--memory")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT FILE
// ============================================================================

#[test]
fn cli_runtime_output_to_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output_test.ruchy", "let x = 42\n");
    let output_file = temp.path().join("runtime_report.txt");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Verify output file was created
    assert!(output_file.exists(), "Output file should be created");
}

#[test]
fn cli_runtime_output_with_bigo() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "output_bigo.ruchy", "let x = 42\n");
    let output_file = temp.path().join("bigo_report.txt");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
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
fn cli_runtime_verbose_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn cli_runtime_verbose_with_profile() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "verbose_profile.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--verbose")
        .arg("--profile")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR MESSAGES
// ============================================================================

#[test]
fn cli_runtime_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("runtime")
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
fn cli_runtime_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_runtime_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    // Empty files fail with "Parse error: Unexpected end of input"
    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unexpected end of input")
            .or(predicate::str::contains("Parse error")));
}

#[test]
fn cli_runtime_complex_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "complex.ruchy",
        r"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let result = factorial(5)
println(result)
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success();
}

#[test]
fn cli_runtime_recursive_function_bigo() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "recursive.ruchy",
        r"
fun fib(n) {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fib(10)
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: HELP AND USAGE
// ============================================================================

#[test]
fn cli_runtime_help_flag() {
    ruchy_cmd()
        .arg("runtime")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("runtime")
            .or(predicate::str::contains("Runtime"))
            .or(predicate::str::contains("performance")));
}

// ============================================================================
// CLI CONTRACT TESTS: PERFORMANCE SCENARIOS
// ============================================================================

#[test]
fn cli_runtime_constant_time_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "constant.ruchy",
        r"
let x = 42
let y = x * 2
println(y)
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("O(1)")
            .or(predicate::str::contains("constant")));
}

#[test]
fn cli_runtime_linear_time_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "linear.ruchy",
        r"
for i in range(100) {
    println(i)
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("O(n)")
            .or(predicate::str::contains("linear")));
}

#[test]
fn cli_runtime_quadratic_time_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "quadratic.ruchy",
        r"
for i in range(10) {
    for j in range(10) {
        println(i * j)
    }
}
",
    );

    ruchy_cmd()
        .arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("O(n")
            .or(predicate::str::contains("quadratic")));
}
