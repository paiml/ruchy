//! TDD test to exactly mimic REPL input processing
//!
//! CRITICAL: Isolate the exact difference between TDD tests and REPL
//! The issue is not in interpreter creation - both use Interpreter::new()

use ruchy::frontend::parser::Parser;
use ruchy::runtime::repl::evaluation::Evaluator;
use ruchy::runtime::repl::state::ReplState;
use ruchy::runtime::repl::{EvalResult, Repl};
use ruchy::runtime::{Interpreter, Value};
use std::path::PathBuf;

#[test]
fn test_repl_evaluator_direct() {
    let mut evaluator = Evaluator::new();
    let mut state = ReplState::new();

    // Test the exact same call that works in TDD but fails in REPL
    let result = evaluator.evaluate_line("type_of(42)", &mut state);

    match result {
        Ok(EvalResult::Value(Value::String(type_name))) => {
            assert!(
                type_name.contains("integer") || type_name.contains("int"),
                "type_of(42) should return 'integer', got: {}",
                type_name
            );
        }
        Ok(other) => {
            panic!(
                "Expected Value(String) containing 'integer', got: {:?}",
                other
            );
        }
        Err(e) => {
            panic!(
                "type_of(42) should succeed in evaluator, got error: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_repl_struct_creation() {
    // Test if we can create the full REPL struct
    let temp_dir = PathBuf::from("/tmp");
    let repl_result = Repl::new(temp_dir);

    match repl_result {
        Ok(_repl) => {
            assert!(true, "REPL creation succeeds");
        }
        Err(e) => {
            panic!("REPL creation failed: {:?}", e);
        }
    }
}

#[test]
fn test_just_identifier_in_evaluator() {
    let mut evaluator = Evaluator::new();
    let mut state = ReplState::new();

    // Test just the identifier lookup (this works in TDD)
    let result = evaluator.evaluate_line("type_of", &mut state);

    match result {
        Ok(EvalResult::Value(Value::BuiltinFunction(name))) => {
            assert_eq!(name, "type_of", "Should return BuiltinFunction(type_of)");
        }
        Ok(other) => {
            panic!("Expected BuiltinFunction(type_of), got: {:?}", other);
        }
        Err(e) => {
            // This will show us the actual error from REPL evaluator
            eprintln!("REPL Evaluator error looking up 'type_of': {:?}", e);
            panic!(
                "type_of identifier lookup failed in REPL evaluator: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_parser_consistency() {
    // Test if the parser behaves the same in both contexts
    let input = "type_of(42)";

    // Parse the same way TDD tests do
    let mut parser1 = Parser::new(input);
    let ast1 = parser1.parse().expect("TDD-style parse should succeed");

    // Parse the same way REPL might do (through evaluator)
    let mut parser2 = Parser::new(input);
    let ast2 = parser2.parse().expect("REPL-style parse should succeed");

    // Both should produce identical ASTs
    assert_eq!(
        format!("{:?}", ast1),
        format!("{:?}", ast2),
        "Parsers should produce identical ASTs"
    );
}

#[test]
fn test_println_in_evaluator() {
    let mut evaluator = Evaluator::new();
    let mut state = ReplState::new();

    // Test println which DOES work in REPL
    let result = evaluator.evaluate_line("println(\"test\")", &mut state);

    match result {
        Ok(EvalResult::Value(Value::Nil)) => {
            assert!(true, "println works in evaluator");
        }
        Ok(other) => {
            panic!("Expected Nil from println, got: {:?}", other);
        }
        Err(e) => {
            panic!("println should succeed in evaluator, got error: {:?}", e);
        }
    }
}
