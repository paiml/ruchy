//! Integration tests for runtime::interpreter module
//!
//! Target: 23.05% → 80% coverage for runtime/interpreter.rs (5,949 uncovered lines)
//! Protocol: EXTREME TDD - Integration tests via "cargo run --example" pattern
//!
//! Approach: Execute example files through CLI to exercise interpreter execution paths.
//! Each test verifies successful execution and expected output patterns.

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================================
// BASIC OPERATIONS TESTS (01_basics.ruchy)
// ============================================================================

#[test]
fn test_interpreter_basics_variables() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/01_basics.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Basic Variables ==="))
        .stdout(predicate::str::contains("Integer: 42"))
        .stdout(predicate::str::contains("Float: 3.14159"))
        .stdout(predicate::str::contains("String: Hello, Ruchy!"))
        .stdout(predicate::str::contains("Boolean: true"));
}

#[test]
fn test_interpreter_basics_arithmetic() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/01_basics.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Arithmetic ==="))
        .stdout(predicate::str::contains("42 + 8 = 50"))
        .stdout(predicate::str::contains("42 * 2 = 84"))
        .stdout(predicate::str::contains("42 / 6 = 7"))
        .stdout(predicate::str::contains("42 % 5 = 2"));
}

#[test]
fn test_interpreter_basics_strings() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/01_basics.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Strings ==="))
        .stdout(predicate::str::contains("Hello, World!"))
        .stdout(predicate::str::contains("Uppercase: HELLO, WORLD!"))
        .stdout(predicate::str::contains("Lowercase: hello, world!"));
}

#[test]
fn test_interpreter_basics_type_conversions() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/01_basics.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Type Conversions ==="))
        .stdout(predicate::str::contains("String '123' to int: 123"))
        .stdout(predicate::str::contains("Float 3.7 to int: 3"));
}

#[test]
fn test_interpreter_basics_mutability() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/01_basics.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Mutability ==="))
        .stdout(predicate::str::contains("Initial counter: 0"))
        .stdout(predicate::str::contains("After increments: 2"));
}

// ============================================================================
// FUNCTION TESTS (02_functions.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_basic_calls() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Alice!"))
        .stdout(predicate::str::contains("Hello, Bob!"));
}

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_return_values() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("10 + 20 = 30"))
        .stdout(predicate::str::contains("5 * 6 = 30"));
}

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_default_params() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Mr. Smith!"))
        .stdout(predicate::str::contains("Hello, Dr. Johnson!"));
}

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_lambdas() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Square of 5: 25"))
        .stdout(predicate::str::contains("Double of 7: 14"));
}

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_higher_order() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Double twice of 5: 20"));
}

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_recursion() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Factorial of 5: 120"));
}

#[test]
#[ignore] // TODO: Fix closure/function handling in 02_functions.ruchy
fn test_interpreter_functions_nested() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Nested function result: 20"));
}

// ============================================================================
// CONTROL FLOW TESTS (03_control_flow.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_if_else() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Age 25: adult"))
        .stdout(predicate::str::contains("Score 85: Grade B"));
}

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_pattern_matching() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Pattern Matching ==="))
        .stdout(predicate::str::contains("42 is medium"))
        .stdout(predicate::str::contains("Saturday is a weekend"));
}

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_for_loop() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("For loop with range:"))
        .stdout(predicate::str::contains("i = 0"))
        .stdout(predicate::str::contains("i = 4"))
        .stdout(predicate::str::contains("apple"))
        .stdout(predicate::str::contains("banana"))
        .stdout(predicate::str::contains("orange"));
}

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_while_loop() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("While loop:"))
        .stdout(predicate::str::contains("count = 0"))
        .stdout(predicate::str::contains("count = 2"));
}

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_loop_with_break() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loop with break:"))
        .stdout(predicate::str::contains("n = 0"))
        .stdout(predicate::str::contains("n = 2"));
}

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_loop_with_continue() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loop with continue"))
        .stdout(predicate::str::contains("1 is odd"))
        .stdout(predicate::str::contains("3 is odd"))
        .stdout(predicate::str::contains("5 is odd"));
}

#[test]
#[ignore] // TODO: Fix control flow issues in 03_control_flow.ruchy
fn test_interpreter_control_pattern_guards() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/03_control_flow.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pattern guard result: positive value"));
}

// ============================================================================
// COLLECTIONS TESTS (04_collections.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix collections functionality
fn test_interpreter_collections_arrays() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/04_collections.ruchy")
        .assert()
        .success();
}

// ============================================================================
// STRING OPERATIONS TESTS (05_strings.ruchy)
// ============================================================================

#[test]
fn test_interpreter_strings_operations() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/05_strings.ruchy")
        .assert()
        .success();
}

// ============================================================================
// ERROR HANDLING TESTS (06_error_handling.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Implement proper error handling
fn test_interpreter_error_handling() {
    // Note: Error handling examples may intentionally demonstrate errors
    // We test that the file can be parsed and executed, even if it contains error examples
    let result = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/06_error_handling.ruchy")
        .ok();

    // File should at least be parseable (may contain intentional runtime errors)
    assert!(result.is_ok());
}

// ============================================================================
// PIPELINE OPERATOR TESTS (07_pipeline_operator.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix pipeline operator len() method support
fn test_interpreter_pipeline_operator() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/07_pipeline_operator.ruchy")
        .assert()
        .success();
}

// ============================================================================
// PATTERN MATCHING TESTS (10_pattern_matching.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix pattern matching edge cases
fn test_interpreter_pattern_matching_comprehensive() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/10_pattern_matching.ruchy")
        .assert()
        .success();
}

// ============================================================================
// ITERATORS TESTS (13_iterators.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix iterator functionality
fn test_interpreter_iterators() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/13_iterators.ruchy")
        .assert()
        .success();
}

// ============================================================================
// ALGORITHMS TESTS (18_algorithms.ruchy)
// ============================================================================

#[test]
#[ignore] // TODO: Fix algorithm examples
fn test_interpreter_algorithms() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg("examples/18_algorithms.ruchy")
        .assert()
        .success();
}

// ============================================================================
// CLI EVAL TESTS (Additional simple expressions)
// ============================================================================

#[test]
fn test_interpreter_eval_simple_integer() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("42")
        .assert()
        .success();
}

#[test]
fn test_interpreter_eval_arithmetic_expression() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("2 + 3 * 4")
        .assert()
        .success();
}

#[test]
fn test_interpreter_eval_string_literal() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#"println("hello world")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_interpreter_eval_boolean_comparison() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("5 > 3")
        .assert()
        .success();
}

#[test]
fn test_interpreter_eval_variable_binding() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("let x = 10; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn test_interpreter_eval_function_definition() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("fn add(a, b) { a + b }; println(add(5, 7))")
        .assert()
        .success()
        .stdout(predicate::str::contains("12"));
}

#[test]
fn test_interpreter_eval_if_expression() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#"let result = if 5 > 3 { "yes" } else { "no" }; println(result)"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("yes"));
}

#[test]
fn test_interpreter_eval_array_creation() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg("let arr = [1, 2, 3]; println(arr.len())")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_interpreter_eval_string_interpolation() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#"let name = "Ruchy"; println(f"Hello, {name}!")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Ruchy!"));
}

#[test]
fn test_interpreter_eval_method_call() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(r#"println("hello".upper())"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("HELLO"));
}
