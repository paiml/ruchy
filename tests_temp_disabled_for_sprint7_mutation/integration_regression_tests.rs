//! Integration regression tests
//!
//! These tests ensure that bugs found in integration testing (from ruchy-book and ruchy-repl-demos)
//! are fixed and never regress. Each test corresponds to a BUG ticket from the roadmap.

#![allow(clippy::needless_raw_string_hashes)]

use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

/// Helper to compile and run Ruchy code
fn run_ruchy_code(code: &str) -> Result<String, String> {
    // Write code to temporary file
    let temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;
    fs::write(temp_file.path(), code).map_err(|e| e.to_string())?;

    // Run with ruchy
    let output = Command::new("./target/release/ruchy")
        .arg("run")
        .arg(temp_file.path())
        .output()
        .map_err(|e| format!("Failed to run ruchy: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// BUG-002: Higher-order functions were incorrectly typed as String parameters
#[test]
fn test_bug_002_higher_order_functions() {
    let code = r#"
fun apply(f, x) {
    f(x)
}

fun double(n) {
    n * 2
}

let result = apply(double, 5)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Higher-order functions should work");
    assert_eq!(output.trim(), "10", "apply(double, 5) should return 10");
}

/// Test function composition
#[test]
fn test_function_composition() {
    let code = r#"
fun compose(f, g, x) {
    f(g(x))
}

fun add_one(n) {
    n + 1
}

fun double(n) {
    n * 2
}

let result = compose(double, add_one, 5)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Function composition should work");
    assert_eq!(
        output.trim(),
        "12",
        "compose(double, add_one, 5) should return 12"
    );
}

/// Test lambdas as arguments
#[test]
fn test_lambda_arguments() {
    let code = r#"
fun apply(f, x) {
    f(x)
}

let result = apply(|n| { n * 3 }, 7)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Lambda arguments should work");
    assert_eq!(output.trim(), "21", "apply(|n| n*3, 7) should return 21");
}

/// Test map-like function
#[test]
fn test_map_function() {
    let code = r#"
fun map_value(f, value) {
    f(value)
}

fun square(n) {
    n * n
}

let result = map_value(square, 8)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Map-like functions should work");
    assert_eq!(output.trim(), "64", "map_value(square, 8) should return 64");
}

/// Test filter-like function with predicate
#[test]
fn test_filter_predicate() {
    let code = r#"
fun filter_value(pred, value) {
    if pred(value) {
        value
    } else {
        0
    }
}

fun is_even(n) {
    n % 2 == 0
}

let result1 = filter_value(is_even, 4)
let result2 = filter_value(is_even, 5)
println(result1)
println(result2)
"#;

    let output = run_ruchy_code(code).expect("Filter predicates should work");
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "4", "filter_value(is_even, 4) should return 4");
    assert_eq!(lines[1], "0", "filter_value(is_even, 5) should return 0");
}

/// Test currying
#[test]
fn test_currying() {
    let code = r#"
fun make_multiplier(x) {
    |y| { x * y }
}

let times_three = make_multiplier(3)
let result = times_three(7)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Currying should work");
    assert_eq!(output.trim(), "21", "times_three(7) should return 21");
}

/// Test nested higher-order functions
#[test]
fn test_nested_higher_order() {
    let code = r#"
fun twice(f, x) {
    f(f(x))
}

fun inc(n) {
    n + 1
}

let result = twice(inc, 10)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Nested higher-order functions should work");
    assert_eq!(output.trim(), "12", "twice(inc, 10) should return 12");
}

/// Test chained function calls
#[test]
fn test_chained_calls() {
    let code = r#"
fun add(x, y) {
    x + y
}

fun curry_add(x) {
    |y| { add(x, y) }
}

let add_five = curry_add(5)
let result = add_five(15)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Chained function calls should work");
    assert_eq!(output.trim(), "20", "add_five(15) should return 20");
}

/// Test function as return value
#[test]
fn test_function_return() {
    let code = r#"
fun get_operator(op) {
    if op == "add" {
        |x, y| { x + y }
    } else {
        |x, y| { x * y }
    }
}

let adder = get_operator("add")
let result = adder(3, 4)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Functions as return values should work");
    assert_eq!(output.trim(), "7", "adder(3, 4) should return 7");
}

/// Test reduce-like function
#[test]
fn test_reduce_function() {
    let code = r#"
fun fold(f, acc, val) {
    f(acc, val)
}

fun add(a, b) {
    a + b
}

let result = fold(add, 10, 5)
println(result)
"#;

    let output = run_ruchy_code(code).expect("Reduce-like functions should work");
    assert_eq!(output.trim(), "15", "fold(add, 10, 5) should return 15");
}
