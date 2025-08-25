// Comprehensive CLI command tests for Deno parity
// Toyota Way: Every command must have automated tests

#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Test the `check` command - syntax validation
#[test]
fn test_check_command_valid_syntax() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = 42 in x + 1").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("check")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));
}

#[test]
fn test_check_command_invalid_syntax() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = ").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("check")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Syntax error"));
}

/// Test the `fmt` command - code formatting
#[test]
fn test_fmt_command_check_mode() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x=42 in x+1").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("fmt")
        .arg(&file)
        .arg("--check")
        .assert()
        .failure()  // Exits with 1 when formatting needed
        .stdout(predicate::str::contains("needs formatting"));
}

#[test]
fn test_fmt_command_stdout() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x=42 in x+1").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("fmt")
        .arg(&file)
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("let"));
}

/// Test the `lint` command - code quality checks
#[test]
fn test_lint_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = 42 in x + 1").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("lint")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn test_lint_command_json_output() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = 42 in 43").unwrap(); // unused variable
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("lint")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

/// Test the `test` command - test runner
#[test]
fn test_test_command_no_tests() {
    let dir = tempdir().unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("test")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
fn test_test_command_with_coverage() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun test_add() { assert(1 + 1 == 2) }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("test")
        .arg(&file)
        .arg("--coverage")
        .assert()
        .success();
}

/// Test the `doc` command - documentation generation
#[test]
fn test_doc_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "/// Adds two numbers\nfun add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("doc")
        .arg(&file)
        .assert()
        .success();
}

/// Test the `bench` command - benchmarking
#[test]
fn test_bench_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun fib(n: i32) -> i32 { if n <= 1 { n } else { fib(n-1) + fib(n-2) } }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("bench")
        .arg(&file)
        .assert()
        .success();
}

/// Test the `ast` command - AST analysis
#[test]
fn test_ast_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = 42 in x + 1").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("ast")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Let"));
}

#[test]
fn test_ast_command_json() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "42").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("ast")
        .arg(&file)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
fn test_ast_command_metrics() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = 42 in x + 1").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("ast")
        .arg(&file)
        .arg("--metrics")
        .assert()
        .success()
        .stdout(predicate::str::contains("Metrics"));
}

/// Test the `provability` command - formal verification
#[test]
fn test_provability_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun pure_add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("provability")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Provability"));
}

#[test]
fn test_provability_command_verify() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun factorial(n: i32) -> i32 { if n <= 1 { 1 } else { n * factorial(n-1) } }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("provability")
        .arg(&file)
        .arg("--verify")
        .assert()
        .success();
}

/// Test the `runtime` command - performance analysis
#[test]
fn test_runtime_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun bubble_sort(arr: [i32]) -> [i32] { arr }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Performance"));
}

#[test]
fn test_runtime_command_bigo() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun linear_search(arr: [i32], x: i32) -> bool { for item in arr { if item == x { return true } } false }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg(&file)
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("O("));
}

/// Test the `score` command - quality scoring
#[test]
fn test_score_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun well_written() { 42 }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("score")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Score"));
}

#[test]
fn test_score_command_json() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "42").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("score")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

/// Test the `quality-gate` command
#[test]
fn test_quality_gate_command() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "fun clean_code() { 42 }").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("quality-gate")
        .arg(&file)
        .assert()
        .success();
}

/// Test the `run` command - execute Ruchy files
#[test]
fn test_run_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "42").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("run")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

/// Test the `transpile` command
#[test]
fn test_transpile_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    fs::write(&file, "let x = 42 in x").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("let mut x"));
}

/// Test the `compile` command
#[test]
fn test_compile_command_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.ruchy");
    let output = dir.path().join("test_binary");
    fs::write(&file, "42").unwrap();
    
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();
    
    // Check that binary was created
    assert!(output.exists());
}

/// Test the `repl` command
#[test]
fn test_repl_command_help() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("repl")
        .write_stdin(":help\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(":help"));
}

/// Test one-liner evaluation with -e flag
#[test]
fn test_eval_flag() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("1 + 1")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_eval_flag_json_format() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("[1, 2, 3]")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

// Toyota Way: Test for quality gate compliance
#[test]
fn test_all_commands_have_help() {
    let commands = vec![
        "check", "fmt", "lint", "test", "doc", "bench",
        "ast", "provability", "runtime", "score", "quality-gate",
        "run", "transpile", "compile", "repl"
    ];
    
    for cmd in commands {
        let mut command = Command::cargo_bin("ruchy").unwrap();
        command.arg(cmd)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains(cmd));
    }
}

// Toyota Way: Ensure error messages are helpful
#[test]
fn test_helpful_error_messages() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("check")
        .arg("/nonexistent/file.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

// Test version consistency
#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ruchy"));
}