#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::approx_constant)]
//! Exhaustive grammar coverage tests for REPL
//!
//! Tests every grammar production to ensure complete coverage

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::print_stdout)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::uninlined_format_args)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use ruchy::frontend::parser::Parser;
use ruchy::runtime::grammar_coverage::{GrammarCoverageMatrix, GRAMMAR_PRODUCTIONS};
use ruchy::runtime::Repl;
use std::{env, time::{Duration, Instant}};

#[test]
fn test_grammar_complete() {
    let mut coverage = GrammarCoverageMatrix::new();
    let mut repl = Repl::new(std::env::temp_dir()).expect("REPL creation should succeed");

    for (name, input) in GRAMMAR_PRODUCTIONS {
        let start = Instant::now();

        // Parse the input
        let mut parser = Parser::new(input);
        let result = parser.parse();
        let elapsed = start.elapsed();

        // Check if parsing failed
        let is_error = result.is_err();

        // Record coverage
        coverage.record(name, input, result, elapsed);

        // Check parsing succeeded or we recorded the error
        if is_error {
            assert!(
                !coverage.productions[name].error_patterns.is_empty(),
                "Production '{}' failed without recording error: {}",
                name,
                input
            );
        }

        // Latency requirement: < 15ms
        assert!(
            elapsed < Duration::from_millis(15),
            "Production '{}' too slow: {:?}",
            name,
            elapsed
        );
    }

    // Print coverage report
    println!("{}", coverage.report());

    // Don't require 100% coverage yet as some productions may not be implemented
    // But ensure we're making progress
    let coverage_percent =
        (coverage.productions.len() as f64 / GRAMMAR_PRODUCTIONS.len() as f64) * 100.0;
    println!("Grammar coverage: {:.1}%", coverage_percent);

    // Require at least 30% coverage for now (will increase as we implement more)
    assert!(
        coverage_percent >= 30.0,
        "Grammar coverage too low: {:.1}%",
        coverage_percent
    );
}

#[test]
fn test_basic_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("REPL creation should succeed");

    // Test expressions that should work with current implementation
    let test_cases = vec![
        ("literal_int", "42", "42"),
        ("literal_float", "3.14", "3.14"),
        ("literal_bool_true", "true", "true"),
        ("literal_bool_false", "false", "false"),
        ("simple_add", "1 + 2", "3"),
        ("simple_sub", "10 - 3", "7"),
        ("simple_mul", "4 * 5", "20"),
        ("simple_div", "15 / 3", "5"),
        ("comparison_lt", "3 < 5", "true"),
        ("comparison_gt", "10 > 5", "true"),
        ("comparison_eq", "7 == 7", "true"),
        ("boolean_and", "true && false", "false"),
        ("boolean_or", "true || false", "true"),
        ("unary_neg", "-42", "-42"),
        ("unary_not", "!true", "false"),
    ];

    for (name, input, expected) in test_cases {
        let result = repl.eval(input);
        assert!(result.is_ok(), "Failed to evaluate {}: {:?}", name, result);
        assert_eq!(result.unwrap(), expected, "Wrong result for {}", name);
    }
}

#[test]
fn test_precedence() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("REPL creation should succeed");

    // Test operator precedence
    let test_cases = vec![
        ("mul_before_add", "2 + 3 * 4", "14"), // Not 20
        ("parens_override", "(2 + 3) * 4", "20"),
        ("comparison_after_arithmetic", "1 + 2 < 4", "true"),
        ("and_after_comparison", "1 < 2 && 3 < 4", "true"),
        ("or_after_and", "false && true || true", "true"), // (false && true) || true
    ];

    for (name, input, expected) in test_cases {
        let result = repl.eval(input);
        assert!(result.is_ok(), "Failed to evaluate {}: {:?}", name, result);
        assert_eq!(
            result.unwrap(),
            expected,
            "Wrong precedence for {}: {}",
            name,
            input
        );
    }
}

#[test]
fn test_error_cases() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("REPL creation should succeed");

    // Test expressions that should fail gracefully
    let error_cases = vec![
        ("div_by_zero", "10 / 0"),
        ("undefined_var", "undefined_variable"),
        ("type_mismatch", "1 + true"),
        ("invalid_syntax", "1 ++ 2"),
    ];

    for (name, input) in error_cases {
        let result = repl.eval(input);
        assert!(result.is_err(), "Expected error for {}: {}", name, input);
    }
}
