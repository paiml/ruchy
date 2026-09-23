#![allow(missing_docs)]

//! TESTRUN-1: `ruchy test` runs the `@test` functions of a multi-item file.
//!
//! A file with two or more top-level items parses to a `Block`. The test
//! runner evaluated that block in a scope it then popped, so every function
//! in the file vanished before the runner called the `@test` functions. The
//! calls used to fall back to actor message values and counted as passes, so
//! `ruchy test` never ran a test in a multi-item file.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn write_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("write test file");
    path
}

const PASSING: &str = r#"
fun double(x) { x * 2 }

@test("double works")
fun test_double() {
    assert_eq(double(21), 42, "double(21) = 42")
}
"#;

const FAILING: &str = r#"
fun helper() { 2 }

@test("helper is three")
fun test_helper_is_three() {
    assert_eq(helper(), 3, "helper() should be 3")
}
"#;

const COMPREHENSIVE: &str = r#"
// Comprehensive test program
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let result = factorial(5)
println(result)

@test("factorial works")
fun test_factorial() {
    assert_eq(factorial(5), 120, "5! = 120")
}
"#;

#[test]
fn test_testrun_1_01_passing_test_in_multi_item_file_passes() {
    let dir = TempDir::new().expect("temp dir");
    let file = write_file(&dir, "pass.ruchy", PASSING);
    ruchy_cmd()
        .arg("test")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Passed: 1"))
        .stdout(predicate::str::contains("All tests passed"));
}

#[test]
fn test_testrun_1_02_failing_assert_eq_fails_the_run_and_names_the_test() {
    let dir = TempDir::new().expect("temp dir");
    let file = write_file(&dir, "fail.ruchy", FAILING);
    ruchy_cmd()
        .arg("test")
        .arg(&file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("test_helper_is_three"))
        .stdout(predicate::str::contains("helper() should be 3"))
        .stdout(predicate::str::contains("Undefined function").not());
}

#[test]
fn test_testrun_1_03_test_calls_helper_defined_in_same_file() {
    let dir = TempDir::new().expect("temp dir");
    let src = r#"
fun square(x) { x * x }
fun sum_of_squares(a, b) { square(a) + square(b) }

@test("sum of squares")
fun test_sum_of_squares() {
    assert_eq(sum_of_squares(3, 4), 25, "3^2 + 4^2 = 25")
}
"#;
    let file = write_file(&dir, "helpers.ruchy", src);
    ruchy_cmd()
        .arg("test")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test passed: test_sum_of_squares"));
}

#[test]
fn test_testrun_1_04_summary_counts_are_exact() {
    let dir = TempDir::new().expect("temp dir");
    write_file(&dir, "a_pass.ruchy", PASSING);
    write_file(&dir, "b_pass.ruchy", COMPREHENSIVE);
    write_file(&dir, "c_fail.ruchy", FAILING);
    ruchy_cmd()
        .arg("test")
        .arg(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Total: 3"))
        .stdout(predicate::str::contains("Passed: 2"))
        .stdout(predicate::str::contains("Failed: 1"))
        .stderr(predicate::str::contains("1 tests failed"));
}

#[test]
fn test_testrun_1_05_comprehensive_program_passes() {
    let dir = TempDir::new().expect("temp dir");
    let file = write_file(&dir, "comprehensive.ruchy", COMPREHENSIVE);
    ruchy_cmd()
        .arg("test")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Passed: 1"));
}

#[test]
fn test_testrun_1_06_top_level_statements_run_once() {
    let dir = TempDir::new().expect("temp dir");
    let file = write_file(&dir, "comprehensive.ruchy", COMPREHENSIVE);
    let out = ruchy_cmd().arg("test").arg(&file).output().expect("run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.matches("120").count(), 1, "stdout: {stdout}");
}

#[test]
fn test_testrun_1_07_second_test_failing_fails_the_file() {
    let dir = TempDir::new().expect("temp dir");
    let src = r#"
fun one() { 1 }

@test("one is one")
fun test_one_is_one() {
    assert_eq(one(), 1, "one() = 1")
}

@test("one is two")
fun test_one_is_two() {
    assert_eq(one(), 2, "one() should be 2")
}
"#;
    let file = write_file(&dir, "second.ruchy", src);
    ruchy_cmd()
        .arg("test")
        .arg(&file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("test_one_is_two"))
        .stdout(predicate::str::contains("Failed: 1"));
}
