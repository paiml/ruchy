//! Minimal TDD test to debug builtin registry issue
//!
//! EXTREME TDD: Create the simplest possible test to isolate the problem

use ruchy::runtime::builtins::BuiltinRegistry;
use ruchy::runtime::{InterpreterError, Value};
use std::rc::Rc;

#[test]
fn test_registry_has_println() {
    let registry = BuiltinRegistry::new();

    // Debug: Print what functions are actually registered
    assert!(
        registry.is_builtin("println"),
        "println should be registered"
    );
    assert!(registry.is_builtin("len"), "len should be registered");
    assert!(
        registry.is_builtin("type_of"),
        "type_of should be registered"
    );
}

#[test]
fn test_registry_call_println() {
    let registry = BuiltinRegistry::new();

    // Check the exact call that's failing in the interpreter
    let result = registry.call("println", &[]);

    match result {
        Ok(value) => {
            assert_eq!(value, Value::nil(), "println should return nil");
        }
        Err(e) => {
            panic!("Registry call failed: {:?}", e);
        }
    }
}

#[test]
fn test_string_processing() {
    let builtin_string = "__builtin_println__";

    // Check the exact string processing logic from interpreter
    let func_name = builtin_string
        .strip_prefix("__builtin_")
        .unwrap()
        .strip_suffix("__")
        .unwrap();
    assert_eq!(func_name, "println");

    // Now test if registry has this function
    let registry = BuiltinRegistry::new();
    assert!(
        registry.is_builtin(func_name),
        "Registry should have println after string processing"
    );
}

#[test]
fn test_len_function() {
    let registry = BuiltinRegistry::new();

    // Check len function specifically
    let arr = Value::Array(Rc::new(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]));
    let result = registry.call("len", &[arr]);

    match result {
        Ok(Value::Integer(3)) => {
            // Success - len works correctly
        }
        Ok(other) => {
            panic!("len returned wrong value: {:?}", other);
        }
        Err(e) => {
            panic!("len call failed: {:?}", e);
        }
    }
}

#[test]
fn test_all_registered_functions() {
    let registry = BuiltinRegistry::new();

    // Check all the functions that should be registered
    let functions = vec![
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

    for func_name in functions {
        assert!(
            registry.is_builtin(func_name),
            "Function '{}' should be registered",
            func_name
        );
    }
}
