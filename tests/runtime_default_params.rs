//! RUNTIME-DEFAULT-PARAMS: Default parameter value handling tests
//!
//! Target: Implement default parameter support in interpreter
//! Protocol: EXTREME TDD - RED phase (all tests MUST fail initially)
//!
//! Root Cause: `Value::Closure` only stores Vec<String>, not default values
//! Fix: Change to store Vec<(String, Option<Expr>)> + update call logic

use predicates::prelude::*;

// ============================================================================
// RED PHASE: FAILING TESTS (These will fail until GREEN phase is complete)
// ============================================================================

#[test]
fn test_default_param_single_missing_arg() {
    // Test calling function with 1 default param, providing 1 of 2 args
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r#"fn greet(name, title = "Mr.") { println(f"{title} {name}") }; greet("Smith")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Mr. Smith"));
}

#[test]
fn test_default_param_all_args_provided() {
    // Test calling function with all args explicitly provided
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r#"fn greet(name, title = "Mr.") { println(f"{title} {name}") }; greet("Johnson", "Dr.")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Dr. Johnson"));
}

#[test]
fn test_default_param_multiple_defaults() {
    // Test function with multiple default parameters
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r#"fn format_name(first, middle = "", last = "Doe") { if middle == "" { println(f"{first} {last}") } else { println(f"{first} {middle} {last}") } }; format_name("John")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("John Doe"));
}

#[test]
fn test_default_param_mixed_required_and_default() {
    // Test mix of required (no default) and optional (with default) params
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r#"fn connect(host, port = 8080) { println(f"{host}:{port}") }; connect("localhost")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("localhost:8080"));
}

#[test]
fn test_default_param_expression_not_just_literal() {
    // Test that default value can be an expression, not just a literal
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r"fn multiply(x, factor = 2 + 3) { println(x * factor) }; multiply(10)")
        .assert()
        .success()
        .stdout(predicate::str::contains("50"));
}

#[test]
fn test_default_param_zero_arguments() {
    // Test function with only default params can be called with zero args
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r#"fn get_config(host = "localhost", port = 8080) { println(f"{host}:{port}") }; get_config()"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("localhost:8080"));
}

#[test]
fn test_default_param_too_many_args_still_error() {
    // Test that providing too many args still produces an error
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(r#"fn greet(name, title = "Mr.") { println(f"{title} {name}") }; greet("Smith", "Dr.", "Extra")"#)
        .assert()
        .failure() // Should fail with error
        .stderr(predicate::str::contains("arguments"));
}

#[test]
#[ignore = "Default params in examples/02_functions needs update"]
fn test_default_param_examples_02_functions() {
    // Test the actual example from 02_functions.ruchy
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("run")
        .arg("examples/02_functions.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Mr. Smith!"))
        .stdout(predicate::str::contains("Hello, Dr. Johnson!"));
}
