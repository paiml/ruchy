//! TDD test to debug REPL vs normal interpreter differences
//!
//! CRITICAL ISSUE: TDD tests pass but REPL fails for builtin functions
//! This suggests different interpreter initialization paths

use ruchy::frontend::parser::Parser;
use ruchy::runtime::{Interpreter, Value};

#[test]
fn test_normal_interpreter_builtin_functions() {
    let mut interpreter = Interpreter::new();

    // This works in TDD tests
    let mut parser = Parser::new("type_of(42)");
    let ast = parser.parse().expect("Should parse function call");

    let result = interpreter.eval_expr(&ast);
    match result {
        Ok(Value::String(type_name)) => {
            assert!(
                type_name.contains("integer") || type_name.contains("int"),
                "type_of(42) should return 'integer', got: {type_name}"
            );
        }
        Ok(other) => {
            panic!("Expected String containing 'integer', got: {other:?}");
        }
        Err(e) => {
            panic!("type_of(42) should succeed, got error: {e:?}");
        }
    }
}

#[test]
fn test_check_repl_module_import() {
    // Test if we can access REPL modules to understand initialization
    // This will help us compare how REPL creates interpreters vs tests

    // First, let's verify builtin registry is correctly initialized in normal interpreter
    let _interpreter = Interpreter::new();

    // Access the builtin registry (if possible via internal methods)
    // We need to check if REPL might be creating interpreter differently

    // This test ensures we can at least create an interpreter
    assert!(true, "Interpreter creation succeeds");
}

#[test]
fn test_builtin_println_works() {
    let mut interpreter = Interpreter::new();

    // Test println which worked in REPL
    let mut parser = Parser::new("println(\"test\")");
    let ast = parser.parse().expect("Should parse println call");

    let result = interpreter.eval_expr(&ast);
    match result {
        Ok(Value::Nil) => {
            // println should return nil and print to stdout
            assert!(true, "println works correctly");
        }
        Ok(other) => {
            panic!("Expected Nil from println, got: {other:?}");
        }
        Err(e) => {
            panic!("println should succeed, got error: {e:?}");
        }
    }
}

#[test]
fn test_builtin_len_works() {
    let mut interpreter = Interpreter::new();

    // Test len which also worked in REPL
    let mut parser = Parser::new("len([1, 2, 3])");
    let ast = parser.parse().expect("Should parse len call");

    let result = interpreter.eval_expr(&ast);
    match result {
        Ok(Value::Integer(3)) => {
            assert!(true, "len works correctly");
        }
        Ok(other) => {
            panic!("Expected Integer(3) from len([1,2,3]), got: {other:?}");
        }
        Err(e) => {
            panic!("len should succeed, got error: {e:?}");
        }
    }
}

#[test]
fn test_direct_identifier_lookup() {
    let mut interpreter = Interpreter::new();

    // Test just the identifier "type_of" without calling it
    let mut parser = Parser::new("type_of");
    let ast = parser.parse().expect("Should parse identifier");

    let result = interpreter.eval_expr(&ast);
    match result {
        Ok(Value::BuiltinFunction(name)) => {
            assert_eq!(name, "type_of", "Should return BuiltinFunction(type_of)");
        }
        Ok(other) => {
            panic!("Expected BuiltinFunction(type_of), got: {other:?}");
        }
        Err(e) => {
            // This is the error we see in REPL - let's debug it
            eprintln!("ERROR looking up 'type_of' identifier: {e:?}");
            panic!("type_of identifier lookup failed: {e:?}");
        }
    }
}
