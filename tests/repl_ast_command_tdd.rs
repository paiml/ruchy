// TDD Tests for REPL :ast command (REPL-003)
//
// Requirements:
// 1. `:ast <expr>` should parse expression and show AST structure
// 2. Display AST in readable tree format
// 3. Show node types, operators, and structure
// 4. Handle all expression types
// 5. Handle errors gracefully

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_ast_command_simple_literal() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test simple integer literal
    let result = repl.eval(":ast 42").unwrap();
    assert!(
        result.contains("Literal") || result.contains("Integer"),
        "Expected AST with Literal/Integer but got: {}",
        result
    );
}

#[test]
fn test_ast_command_binary_expression() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test binary expression
    let result = repl.eval(":ast 2 + 3").unwrap();
    assert!(
        result.contains("Binary") || result.contains("Add") || result.contains("+"),
        "Expected AST with Binary/Add but got: {}",
        result
    );
}

#[test]
fn test_ast_command_function_call() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test function call
    let result = repl.eval(":ast println(42)").unwrap();
    assert!(
        result.contains("Call") || result.contains("println"),
        "Expected AST with Call/println but got: {}",
        result
    );
}

#[test]
fn test_ast_command_variable() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test variable reference
    let result = repl.eval(":ast x").unwrap();
    assert!(
        result.contains("Identifier") || result.contains("x"),
        "Expected AST with Identifier but got: {}",
        result
    );
}

#[test]
fn test_ast_command_complex_expression() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test complex nested expression
    let result = repl.eval(":ast (2 + 3) * 4").unwrap();
    assert!(
        result.contains("Binary") || result.contains("Multiply") || result.contains("*"),
        "Expected AST with Binary operations but got: {}",
        result
    );
}

#[test]
fn test_ast_command_no_args() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test without arguments
    let result = repl.eval(":ast").unwrap();
    assert!(
        result.contains("Usage") || result.contains("ast <expression>"),
        "Expected usage message but got: {}",
        result
    );
}

#[test]
fn test_ast_command_string() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test string literal
    let result = repl.eval(":ast \"hello\"").unwrap();
    assert!(
        result.contains("Literal") || result.contains("String") || result.contains("hello"),
        "Expected AST with String literal but got: {}",
        result
    );
}

#[test]
fn test_ast_command_array() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test array literal
    let result = repl.eval(":ast [1, 2, 3]").unwrap();
    assert!(
        result.contains("List") || result.contains("Array") || result.contains("["),
        "Expected AST with List/Array but got: {}",
        result
    );
}
