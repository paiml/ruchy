// TDD Tests for REPL :type command (REPL-001)
//
// Requirements:
// 1. `:type <expr>` should evaluate expression and return its type
// 2. Support all basic types: Integer, Float, Bool, String, Array, Object
// 3. Format: "Type: <typename>"
// 4. Handle errors gracefully

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_type_command_integer() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test integer literal
    let result = repl.eval(":type 42").unwrap();
    assert!(
        result.contains("Integer"),
        "Expected 'Integer' but got: {}",
        result
    );
}

#[test]
fn test_type_command_float() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test float literal
    let result = repl.eval(":type 3.14").unwrap();
    assert!(
        result.contains("Float"),
        "Expected 'Float' but got: {}",
        result
    );
}

#[test]
fn test_type_command_string() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test string literal
    let result = repl.eval(":type \"hello\"").unwrap();
    assert!(
        result.contains("String"),
        "Expected 'String' but got: {}",
        result
    );
}

#[test]
fn test_type_command_bool() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test boolean
    let result = repl.eval(":type true").unwrap();
    assert!(
        result.contains("Bool"),
        "Expected 'Bool' but got: {}",
        result
    );
}

#[test]
fn test_type_command_array() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test array
    let result = repl.eval(":type [1, 2, 3]").unwrap();
    assert!(
        result.contains("Array"),
        "Expected 'Array' but got: {}",
        result
    );
}

#[test]
fn test_type_command_variable() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define a variable
    repl.eval("let x = 42").unwrap();

    // Check its type
    let result = repl.eval(":type x").unwrap();
    assert!(
        result.contains("Integer"),
        "Expected 'Integer' but got: {}",
        result
    );
}

#[test]
fn test_type_command_expression() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test expression evaluation
    let result = repl.eval(":type 2 + 3").unwrap();
    assert!(
        result.contains("Integer"),
        "Expected 'Integer' but got: {}",
        result
    );
}

#[test]
fn test_type_command_format() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test exact format: "Type: <typename>"
    let result = repl.eval(":type 42").unwrap();
    assert!(
        result.starts_with("Type:"),
        "Expected format 'Type: <typename>' but got: {}",
        result
    );
}
