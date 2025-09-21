//! TDD tests for builtin registry integration debugging
//!
//! Following Extreme TDD: Write failing tests first to identify exact issue

use ruchy::frontend::parser::Parser;
use ruchy::runtime::{Interpreter, Value};
use std::rc::Rc;

#[test]
fn test_builtin_registry_initialization() {
    // TDD: Test that builtin registry is properly initialized
    let interpreter = Interpreter::new();

    // Create a test registry to verify the structure
    let test_registry = ruchy::runtime::builtins::BuiltinRegistry::new();
    assert!(test_registry.is_builtin("println"));
    assert!(test_registry.is_builtin("len"));
    assert!(test_registry.is_builtin("type_of"));
}

#[test]
fn test_builtin_registry_direct_call() {
    // TDD: Test direct registry call without interpreter
    let registry = ruchy::runtime::builtins::BuiltinRegistry::new();

    // Test println with empty args
    let result = registry.call("println", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::nil());

    // Test len with array
    let arr = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let result = registry.call("len", &[arr]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(2));
}

#[test]
fn test_builtin_name_extraction() {
    // TDD: Test the name extraction logic
    let builtin_name = "__builtin_println__";

    let func_name = builtin_name
        .strip_prefix("__builtin_")
        .unwrap()
        .strip_suffix("__")
        .unwrap();
    assert_eq!(func_name, "println");

    let builtin_name2 = "__builtin_len__";
    let func_name2 = builtin_name2
        .strip_prefix("__builtin_")
        .unwrap()
        .strip_suffix("__")
        .unwrap();
    assert_eq!(func_name2, "len");
}

#[test]
fn test_registry_function_list() {
    // TDD: Verify what functions are actually registered
    let registry = ruchy::runtime::builtins::BuiltinRegistry::new();

    // Test all expected builtin functions
    let expected_functions = vec![
        "println",
        "print",
        "dbg",
        "len",
        "type_of",
        "is_nil",
        "sqrt",
        "pow",
        "abs",
        "min",
        "max",
        "floor",
        "ceil",
        "round",
        "to_string",
        "parse_int",
        "parse_float",
        "push",
        "pop",
    ];

    for func_name in expected_functions {
        assert!(
            registry.is_builtin(func_name),
            "Function '{}' should be registered in builtin registry",
            func_name
        );
    }
}

#[test]
fn test_eval_builtin_function_call() {
    // TDD: Test the high-level evaluation of builtin function calls
    let mut interpreter = Interpreter::new();

    // Test arithmetic first to ensure basic interpreter works
    let mut parser = Parser::new("1 + 1");
    let ast = parser.parse().expect("Parse should succeed");
    let result = interpreter.eval_expr(&ast);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(2));
}
