#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Core Interpreter Reliability Tests
//!
//! Following Toyota Way: These tests MUST pass 100% before any feature work
//! This is our "Andon Cord" - if these fail, we stop everything

#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unwrap_used)]

use ruchy::runtime::Repl;
use std::{env, time::{Duration, Instant};

/// Helper macro for testing REPL evaluation
macro_rules! assert_eval {
    ($repl:expr, $input:expr, $expected:expr) => {
        let result = $repl.eval($input);
        assert!(
            result.is_ok(),
            "Failed to evaluate '{}': {:?}",
            $input,
            result
        );
        let output = result.unwrap();
        assert_eq!(output, $expected, "Input: {}", $input);
    };
}

/// Helper macro for testing that evaluation fails
macro_rules! assert_eval_err {
    ($repl:expr, $input:expr) => {
        let result = $repl.eval($input);
        assert!(
            result.is_err(),
            "Expected error for '{}' but got: {:?}",
            $input,
            result
        );
    };
}

// ============================================================================
// SECTION 1: ARITHMETIC OPERATIONS
// ============================================================================

#[test]
fn test_integer_arithmetic() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Basic operations
    assert_eval!(repl, "1 + 2", "3");
    assert_eval!(repl, "10 - 3", "7");
    assert_eval!(repl, "4 * 5", "20");
    assert_eval!(repl, "15 / 3", "5");
    assert_eval!(repl, "17 % 5", "2");

    // Negative numbers
    assert_eval!(repl, "-5 + 3", "-2");
    assert_eval!(repl, "5 + -3", "2");
    assert_eval!(repl, "-5 * -3", "15");
}

#[test]
fn test_operator_precedence() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "2 + 3 * 4", "14");
    assert_eval!(repl, "(2 + 3) * 4", "20");
    assert_eval!(repl, "10 - 2 * 3", "4");
    assert_eval!(repl, "(10 - 2) * 3", "24");
    assert_eval!(repl, "20 / 4 + 3", "8");
    assert_eval!(repl, "20 / (4 + 1)", "4");
}

#[test]
fn test_float_arithmetic() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "1.5 + 2.5", "4");
    assert_eval!(repl, "10.0 - 3.5", "6.5");
    assert_eval!(repl, "2.5 * 4.0", "10");
    assert_eval!(repl, "7.5 / 2.5", "3");
}

#[test]
fn test_division_by_zero() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval_err!(repl, "5 / 0");
    assert_eval_err!(repl, "10 % 0");
}

// ============================================================================
// SECTION 2: VARIABLE BINDINGS
// ============================================================================

#[test]
fn test_immutable_bindings() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "let x = 10", "10");
    assert_eval!(repl, "x", "10");
    assert_eval!(repl, "let y = x + 5", "15");
    assert_eval!(repl, "y", "15");
    assert_eval!(repl, "x + y", "25");
}

#[test]
fn test_mutable_bindings() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "let mut x = 10", "10");
    assert_eval!(repl, "x", "10");
    assert_eval!(repl, "x = 20", "20");
    assert_eval!(repl, "x", "20");
}

#[test]
fn test_undefined_variable() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval_err!(repl, "undefined_var");
    assert_eval_err!(repl, "x + y");
}

#[test]
fn test_variable_shadowing() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "let x = 10", "10");
    assert_eval!(repl, "let x = 20", "20");
    assert_eval!(repl, "x", "20");
}

// ============================================================================
// SECTION 3: FUNCTIONS
// ============================================================================

#[test]
fn test_simple_function() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "fun double(x) { x * 2 }", "fn double(x)");
    assert_eval!(repl, "double(5)", "10");
    assert_eval!(repl, "double(21)", "42");
}

#[test]
fn test_recursive_factorial() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let factorial = r"
        fun fact(n) {
            if n <= 1 { 1 } else { n * fact(n - 1) }
        }
    ";

    repl.eval(factorial).expect("Failed to define factorial");
    assert_eval!(repl, "fact(0)", "1");
    assert_eval!(repl, "fact(1)", "1");
    assert_eval!(repl, "fact(5)", "120");
    assert_eval!(repl, "fact(10)", "3628800");
}

#[test]
fn test_fibonacci() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let fibonacci = r"
        fun fib(n) {
            if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
        }
    ";

    repl.eval(fibonacci).expect("Failed to define fibonacci");
    assert_eval!(repl, "fib(0)", "0");
    assert_eval!(repl, "fib(1)", "1");
    assert_eval!(repl, "fib(10)", "55");
}

#[test]
fn test_higher_order_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    repl.eval("fun apply(f, x) { f(x) }")
        .expect("Failed to define apply");
    repl.eval("fun double(x) { x * 2 }")
        .expect("Failed to define double");

    assert_eval!(repl, "apply(double, 5)", "10");
}

// ============================================================================
// SECTION 4: CONTROL FLOW
// ============================================================================

#[test]
fn test_if_else() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "if true { 10 } else { 20 }", "10");
    assert_eval!(repl, "if false { 10 } else { 20 }", "20");
    assert_eval!(repl, "if 5 > 3 { \"yes\" } else { \"no\" }", "\"yes\"");
    assert_eval!(repl, "if 2 > 3 { \"yes\" } else { \"no\" }", "\"no\"");
}

#[test]
fn test_nested_if() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let nested = r#"
        let x = 10
        if x > 5 {
            if x > 8 { "very big" } else { "big" }
        } else {
            "small"
        }
    "#;

    assert_eval!(repl, nested, "\"very big\"");
}

#[test]
fn test_match_expression() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let match_expr = r#"
        let x = 2
        match x {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#;

    assert_eval!(repl, match_expr, "\"two\"");
}

#[test]
fn test_for_loop() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Test with list
    repl.eval("let mut sum = 0").expect("Failed to create sum");
    repl.eval("for x in [1, 2, 3, 4, 5] { sum = sum + x }")
        .expect("Failed to run loop");
    assert_eval!(repl, "sum", "15");
}

#[test]
fn test_while_loop() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let while_loop = r"
        let mut i = 0
        let mut sum = 0
        while i < 5 {
            sum = sum + i
            i = i + 1
        }
        sum
    ";

    assert_eval!(repl, while_loop, "10"); // 0 + 1 + 2 + 3 + 4
}

// ============================================================================
// SECTION 5: DATA TYPES
// ============================================================================

#[test]
fn test_strings() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, r#""hello""#, r#""hello""#);
    assert_eval!(repl, r#""hello" + " " + "world""#, r#""hello world""#);
    assert_eval!(repl, r#""test".len()"#, "4");
}

#[test]
fn test_string_interpolation() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "let name = \"Alice\"", "\"Alice\"");
    assert_eval!(repl, r#"f"Hello, {name}!""#, r#""Hello, Alice!""#);

    assert_eval!(repl, "let x = 42", "42");
    assert_eval!(repl, r#"f"The answer is {x}""#, r#""The answer is 42""#);
}

#[test]
fn test_lists() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "[1, 2, 3]", "[1, 2, 3]");
    assert_eval!(repl, "[1, 2, 3].len()", "3");
    assert_eval!(repl, "[1, 2, 3, 4, 5].map(|x| x * 2)", "[2, 4, 6, 8, 10]");
    assert_eval!(repl, "[1, 2, 3, 4, 5].filter(|x| x > 2)", "[3, 4, 5]");
}

#[test]
fn test_tuples() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "(1, 2, 3)", "(1, 2, 3)");
    assert_eval!(repl, "(\"hello\", 42, true)", "(\"hello\", 42, true)");
}

#[test]
fn test_option_type() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "Some(42)", "Option::Some(42)");
    assert_eval!(repl, "None", "Option::None");
}

#[test]
fn test_result_type() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "Ok(42)", "Result::Ok(42)");
    assert_eval!(
        repl,
        r#"Err("error message")"#,
        r#"Result::Err("error message")"#
    );
}

// ============================================================================
// SECTION 6: LAMBDA EXPRESSIONS
// ============================================================================

#[test]
fn test_lambda_basic() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval!(repl, "let add = |x, y| x + y", "|x, y| <closure>");
    assert_eval!(repl, "add(3, 4)", "7");
}

#[test]
fn test_lambda_closure() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    repl.eval("let x = 10").expect("Failed to set x");
    assert_eval!(repl, "let add_x = |y| x + y", "|y| <closure>");
    assert_eval!(repl, "add_x(5)", "15");
}

// ============================================================================
// SECTION 7: ERROR HANDLING
// ============================================================================

#[test]
fn test_stack_overflow_protection() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Infinite recursion should error, not crash
    repl.eval("fun infinite() { infinite() }")
        .expect("Failed to define infinite");
    assert_eval_err!(repl, "infinite()");
}

#[test]
fn test_type_errors() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    assert_eval_err!(repl, r#"5 + "string""#);
    assert_eval_err!(repl, "true * 5");
}

// ============================================================================
// SECTION 8: PERFORMANCE BOUNDS
// ============================================================================

#[test]
fn test_response_time() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let start = Instant::now();
    let _ = repl.eval("1 + 2");
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(100),
        "Simple expression took {duration:?}, expected < 100ms"
    );
}

#[test]
fn test_deep_recursion() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Should handle reasonable recursion depth
    repl.eval("fun sum(n) { if n <= 0 { 0 } else { n + sum(n - 1) } }")
        .expect("Failed to define sum");

    // This should work with default depth limit of 100
    assert_eval!(repl, "sum(50)", "1275"); // sum of 1..50

    // This should fail due to depth limit
    assert_eval_err!(repl, "sum(200)");
}

// ============================================================================
// SECTION 9: SESSION PERSISTENCE
// ============================================================================

#[test]
fn test_session_state_persistence() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Define things in sequence
    assert_eval!(repl, "let x = 10", "10");
    assert_eval!(repl, "fun double(n) { n * 2 }", "fn double(n)");
    assert_eval!(repl, "let y = double(x)", "20");

    // Everything should still be available
    assert_eval!(repl, "x", "10");
    assert_eval!(repl, "y", "20");
    assert_eval!(repl, "double(y)", "40");
}

#[test]
fn test_error_recovery() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Set up state
    assert_eval!(repl, "let x = 10", "10");

    // Cause an error
    assert_eval_err!(repl, "undefined_function()");

    // State should still be intact
    assert_eval!(repl, "x", "10");

    // Should be able to continue
    assert_eval!(repl, "let y = 20", "20");
    assert_eval!(repl, "x + y", "30");
}

// ============================================================================
// SECTION 10: REGRESSION TESTS
// ============================================================================

#[test]
fn test_regression_tuple_parsing() {
    // Bug: Tuples used to fail parsing in REPL
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    assert_eval!(repl, "(1, 2, 3)", "(1, 2, 3)");
}

#[test]
fn test_regression_struct_creation() {
    // Bug: Struct literals had type mismatches
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    repl.eval("struct Point { x: i32, y: i32 }")
        .expect("Failed to define struct");
    // Note: Object fields may appear in any order due to HashMap
    let result = repl
        .eval(r"Point { x: 10, y: 20 }")
        .expect("Failed to create struct");
    assert!(result.contains(r#""x": 10"#));
    assert!(result.contains(r#""y": 20"#));
}

#[test]
fn test_regression_enum_variants() {
    // Bug: Enum variants weren't properly constructed
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    repl.eval("enum Color { Red, Green, Blue }")
        .expect("Failed to define enum");
    assert_eval!(repl, "Color::Red", "Color::Red");
}
