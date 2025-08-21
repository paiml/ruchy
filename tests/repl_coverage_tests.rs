//! Comprehensive REPL tests to achieve 80% coverage

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::runtime::{Repl, ReplConfig, Value};
use std::time::{Duration, Instant};

#[test]
fn test_repl_arithmetic_operations() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Basic arithmetic
    assert_eq!(repl.eval("1 + 1").unwrap(), "2");
    assert_eq!(repl.eval("10 - 5").unwrap(), "5");
    assert_eq!(repl.eval("3 * 4").unwrap(), "12");
    assert_eq!(repl.eval("20 / 4").unwrap(), "5");
    assert_eq!(repl.eval("10 % 3").unwrap(), "1");
    assert_eq!(repl.eval("2 ** 3").unwrap(), "8");

    // Float arithmetic
    assert_eq!(repl.eval("1.5 + 2.5").unwrap(), "4");
    assert_eq!(repl.eval("10.0 / 3.0").unwrap(), "3.3333333333333335");

    // Precedence
    assert_eq!(repl.eval("2 + 3 * 4").unwrap(), "14");
    assert_eq!(repl.eval("(2 + 3) * 4").unwrap(), "20");
}

#[test]
fn test_repl_string_operations() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval(r#""hello""#).unwrap(), r#""hello""#);
    assert_eq!(
        repl.eval(r#""hello" + " world""#).unwrap(),
        r#""hello world""#
    );
    assert_eq!(repl.eval(r#""a" + "b" + "c""#).unwrap(), r#""abc""#);
}

#[test]
fn test_repl_boolean_operations() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("true").unwrap(), "true");
    assert_eq!(repl.eval("false").unwrap(), "false");
    assert_eq!(repl.eval("true && true").unwrap(), "true");
    assert_eq!(repl.eval("true && false").unwrap(), "false");
    assert_eq!(repl.eval("false || true").unwrap(), "true");
    assert_eq!(repl.eval("false || false").unwrap(), "false");
    assert_eq!(repl.eval("!true").unwrap(), "false");
    assert_eq!(repl.eval("!false").unwrap(), "true");
}

#[test]
fn test_repl_comparison_operations() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("5 > 3").unwrap(), "true");
    assert_eq!(repl.eval("3 > 5").unwrap(), "false");
    assert_eq!(repl.eval("5 >= 5").unwrap(), "true");
    assert_eq!(repl.eval("3 < 5").unwrap(), "true");
    assert_eq!(repl.eval("5 <= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 == 5").unwrap(), "true");
    assert_eq!(repl.eval("5 != 3").unwrap(), "true");
}

#[test]
fn test_repl_variables() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("let x = 42").unwrap(), "42");
    assert_eq!(repl.eval("x").unwrap(), "42");
    assert_eq!(repl.eval("let y = x + 8").unwrap(), "50");
    assert_eq!(repl.eval("y").unwrap(), "50");

    // Variable reassignment
    assert_eq!(repl.eval("x = 100").unwrap(), "100");
    assert_eq!(repl.eval("x").unwrap(), "100");
}

#[test]
fn test_repl_if_expressions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("if true { 1 } else { 2 }").unwrap(), "1");
    assert_eq!(repl.eval("if false { 1 } else { 2 }").unwrap(), "2");
    assert_eq!(repl.eval("if 5 > 3 { 100 } else { 200 }").unwrap(), "100");

    // Nested if
    assert_eq!(
        repl.eval("if true { if false { 1 } else { 2 } } else { 3 }")
            .unwrap(),
        "2"
    );
}

#[test]
fn test_repl_blocks() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("{ 42 }").unwrap(), "42");
    assert_eq!(repl.eval("{ 1; 2; 3 }").unwrap(), "3");
    assert_eq!(repl.eval("{ let a = 5; a }").unwrap(), "5");
    assert_eq!(repl.eval("{ }").unwrap(), "()");
}

#[test]
fn test_repl_lists() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Lists evaluate to themselves
    assert_eq!(repl.eval("[1, 2, 3]").unwrap(), "[1, 2, 3]");
    assert_eq!(repl.eval("[true, false]").unwrap(), "[true, false]");
    assert_eq!(repl.eval("[]").unwrap(), "[]");
}

#[test]
fn test_repl_function_calls() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval(r#"println("hello")"#).unwrap(), "()");
    assert_eq!(repl.eval(r#"print("test")"#).unwrap(), "()");
}

#[test]
fn test_repl_error_cases() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Undefined variable
    assert!(repl.eval("undefined_var").is_err());

    // Division by zero
    assert!(repl.eval("1 / 0").is_err());
    assert!(repl.eval("5 % 0").is_err());

    // Type mismatches
    assert!(repl.eval("1 + true").is_err());
    assert!(repl.eval(r#"5 + "hello""#).is_err());

    // Unknown function
    assert!(repl.eval("unknown_func()").is_err());
}

#[test]
fn test_repl_with_custom_config() {
    let config = ReplConfig {
        max_memory: 1024,
        timeout: Duration::from_millis(10),
        max_depth: 5,
        debug: false,
    };

    let mut repl = Repl::with_config(config).expect("Failed to create REPL");

    // Should work with custom config
    assert_eq!(repl.eval("1 + 1").unwrap(), "2");
}

#[test]
fn test_repl_evaluate_expr_str() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    let deadline = Some(Instant::now() + Duration::from_millis(100));

    let value = repl.evaluate_expr_str("42", deadline).unwrap();
    assert_eq!(value, Value::Int(42));

    let value = repl.evaluate_expr_str("true", deadline).unwrap();
    assert_eq!(value, Value::Bool(true));

    let value = repl.evaluate_expr_str(r#""hello""#, deadline).unwrap();
    assert_eq!(value, Value::String("hello".to_string()));

    let value = repl.evaluate_expr_str("3.25", deadline).unwrap();
    assert_eq!(value, Value::Float(3.25));
}

#[test]
fn test_repl_memory_tracking() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // These should track memory but not exceed limits
    assert!(repl.eval("let x = 1").is_ok());
    assert!(repl.eval("let y = 2").is_ok());
    assert!(repl.eval("let z = 3").is_ok());
}

#[test]
fn test_repl_depth_limits() {
    let config = ReplConfig {
        max_memory: 10_000,
        timeout: Duration::from_millis(100),
        max_depth: 3, // Very shallow
        debug: false,
    };

    let mut repl = Repl::with_config(config).expect("Failed to create REPL");

    // Parentheses don't increase depth, but nested expressions do
    assert_eq!(repl.eval("((((1))))").unwrap(), "1");
    // Nested let bindings should fail due to depth
    assert!(repl.eval("let a = let b = let c = let d = 1").is_err());
}

#[test]
fn test_repl_timeout() {
    let config = ReplConfig {
        max_memory: 10_000,
        timeout: Duration::from_micros(1), // Extremely short timeout
        max_depth: 100,
        debug: false,
    };

    let mut repl = Repl::with_config(config).expect("Failed to create REPL");

    // Should timeout
    assert!(repl.eval("1 + 2 + 3 + 4 + 5").is_err());
}

#[test]
fn test_repl_char_literals() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Char literals work but may have different output format
    assert!(repl.eval("'a'").is_ok());
    assert!(repl.eval("'Z'").is_ok());
    assert!(repl.eval("'0'").is_ok());
}

#[test]
fn test_repl_unary_operations() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("-5").unwrap(), "-5");
    assert_eq!(repl.eval("-(-5)").unwrap(), "5");
}

#[test]
fn test_repl_range_expressions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Ranges return their representation
    assert_eq!(repl.eval("1..10").unwrap(), "1..10");
    assert_eq!(repl.eval("1..=10").unwrap(), "1..=10");
}

#[test]
fn test_repl_tuple_expressions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Tuples return their representation
    assert_eq!(repl.eval("(1, 2)").unwrap(), "(1, 2)");
    assert_eq!(repl.eval("(true, false, true)").unwrap(), "(true, false, true)");
}

#[test]
fn test_repl_multiline_detection() {
    // Test the static needs_continuation function
    assert!(Repl::needs_continuation("{"));
    assert!(Repl::needs_continuation("if true {"));
    assert!(Repl::needs_continuation("let x ="));
    assert!(Repl::needs_continuation("[1, 2,"));
    assert!(Repl::needs_continuation("(1 +"));
    assert!(Repl::needs_continuation(r#""hello"#));

    assert!(!Repl::needs_continuation("42"));
    assert!(!Repl::needs_continuation("let x = 5"));
    assert!(!Repl::needs_continuation("{ 1 }"));
}

#[test]
fn test_repl_pipeline_operator() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Pipeline operator works with functions
    assert_eq!(repl.eval("1 |> println").unwrap(), "()");
}

#[test]
fn test_repl_match_expressions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Match expressions return the matched arm value
    assert_eq!(repl.eval("match 1 { _ => 42 }").unwrap(), "42");
}

#[test]
fn test_repl_for_loops() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // For loops return the last expression value
    assert_eq!(repl.eval("for x in [1, 2, 3] { x }").unwrap(), "3");
}

#[test]
fn test_repl_while_loops() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // While loops return unit
    assert_eq!(repl.eval("while false { 1 }").unwrap(), "()");
}

#[test]
fn test_repl_loop_expressions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Loop expressions return unit
    assert_eq!(repl.eval("loop { break }").unwrap(), "()");
}

#[test]
fn test_repl_try_expressions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Try expressions return the try block value
    assert_eq!(repl.eval("try { 1 } catch e { 0 }").unwrap(), "1");
}
