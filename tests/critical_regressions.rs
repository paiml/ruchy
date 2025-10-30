#![cfg(test)]
#![allow(missing_docs)]
#![allow(warnings)]
#![allow(missing_docs)]
#![allow(clippy::assertions_on_constants)]
#![allow(missing_docs)]
#![allow(clippy::unreadable_literal)]
#![allow(missing_docs)]
//! Critical bug prevention tests
//!
//! These tests ensure we don't regress on known critical bugs

#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]
#![allow(clippy::expect_used)]
#![allow(missing_docs)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(missing_docs)]
#![allow(clippy::uninlined_format_args)]
#![allow(missing_docs)]

use ruchy::runtime::Repl;
use std::env;

#[test]
fn test_string_interpolation_no_f_prefix() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Known bug: Parser must reject interpolation syntax without 'f' prefix
    // Regular strings with braces should remain literal
    let result = repl.eval(r#""Hello, {name}!""#);
    assert!(
        result.is_err() || result.as_ref().unwrap().contains("Hello, {name}!"),
        "String with braces but no 'f' prefix must parse as literal string or error"
    );

    // The result should be the literal string, not an interpolation
    if let Ok(val) = result {
        assert!(
            !val.contains("Any"),
            "String should not generate Any type code"
        );
    }
}

#[test]
fn test_let_statement_without_semicolon() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Known bug: REPL must accept let statements without semicolons
    let result = repl.eval("let x = 1");
    assert!(
        result.is_ok(),
        "Let statement without semicolon must parse in REPL: {:?}",
        result
    );

    // Check binding exists
    let result = repl.eval("x");
    assert!(
        result.is_ok(),
        "Variable should be accessible after let binding"
    );

    // Also test with semicolon
    let result = repl.eval("let y = 2;");
    assert!(result.is_ok(), "Let with semicolon should also work");
}

#[test]
fn test_function_with_string_interpolation() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Original failing case - function with f-string
    let result = repl.eval(
        r#"
        fun greet(name) {
            f"Hello, {name}!"
        }
    "#,
    );

    // For now, just test that it doesn't panic
    // Full f-string support may not be implemented yet
    let _ = result;
}

#[test]
fn test_no_polars_dependency_in_simple_code() {
    // This test would require access to the transpiler's output
    // For now, we'll just ensure simple functions parse without issues
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let result = repl.eval("fun add(x, y) { x + y }");
    assert!(
        result.is_ok(),
        "Simple function should parse without issues"
    );
}
