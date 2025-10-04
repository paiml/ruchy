//! TDD test to debug builtin function lookup
//!
//! Testing the exact flow: identifier lookup -> builtin registry check -> function call

use ruchy::frontend::parser::Parser;
use ruchy::runtime::{Interpreter, Value};

#[test]
fn test_builtin_registry_lookup() {
    let registry = ruchy::runtime::builtins::BuiltinRegistry::new();

    // Direct test: is_builtin should return true for type_of
    assert!(
        registry.is_builtin("type_of"),
        "type_of should be recognized as builtin"
    );
    assert!(
        registry.is_builtin("len"),
        "len should be recognized as builtin"
    );
    assert!(
        registry.is_builtin("println"),
        "println should be recognized as builtin"
    );
}

#[test]
fn test_identifier_lookup_returns_builtin_function() {
    let mut interpreter = Interpreter::new();

    // This should access the private lookup_variable function indirectly
    let mut parser = Parser::new("type_of");
    let ast = parser.parse().expect("Should parse identifier");

    // This should call lookup_variable and return BuiltinFunction variant
    let result = interpreter.eval_expr(&ast);
    match result {
        Ok(Value::BuiltinFunction(name)) => {
            assert_eq!(name, "type_of", "Should return BuiltinFunction(type_of)");
        }
        Ok(other) => {
            panic!("Expected BuiltinFunction(type_of), got: {:?}", other);
        }
        Err(e) => {
            panic!("Expected BuiltinFunction(type_of), got error: {:?}", e);
        }
    }
}

#[test]
fn test_function_call_with_builtin_function_value() {
    let mut interpreter = Interpreter::new();

    // Check the full pipeline: parse type_of call
    let mut parser = Parser::new("type_of(42)");
    let ast = parser.parse().expect("Should parse function call");

    // This should work end-to-end
    let result = interpreter.eval_expr(&ast);
    match result {
        Ok(Value::String(type_name)) => {
            assert!(
                type_name.contains("integer") || type_name.contains("int"),
                "type_of(42) should return 'integer', got: {}",
                type_name
            );
        }
        Ok(other) => {
            panic!("Expected String containing 'integer', got: {:?}", other);
        }
        Err(e) => {
            panic!("type_of(42) should succeed, got error: {:?}", e);
        }
    }
}
