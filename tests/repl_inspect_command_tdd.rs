// TDD Tests for REPL :inspect command (REPL-002)
//
// Requirements:
// 1. `:inspect <expr>` should evaluate expression and return detailed info
// 2. Show type, value, size, and structure
// 3. Format nested structures with indentation
// 4. Handle all value types: primitives, arrays, objects, functions
// 5. Handle errors gracefully

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_inspect_command_integer() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test integer inspection
    let result = repl.eval(":inspect 42").unwrap();
    assert!(
        result.contains("Type:") && result.contains("Integer"),
        "Expected Type: Integer but got: {}",
        result
    );
    assert!(
        result.contains("Value:") && result.contains("42"),
        "Expected Value: 42 but got: {}",
        result
    );
}

#[test]
fn test_inspect_command_string() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test string inspection
    let result = repl.eval(":inspect \"hello\"").unwrap();
    assert!(
        result.contains("Type:") && result.contains("String"),
        "Expected Type: String but got: {}",
        result
    );
    assert!(
        result.contains("Value:") && result.contains("hello"),
        "Expected Value: hello but got: {}",
        result
    );
    assert!(
        result.contains("Length:") && result.contains("5"),
        "Expected Length: 5 but got: {}",
        result
    );
}

#[test]
fn test_inspect_command_array() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test array inspection
    let result = repl.eval(":inspect [1, 2, 3]").unwrap();
    assert!(
        result.contains("Type:") && result.contains("Array"),
        "Expected Type: Array but got: {}",
        result
    );
    assert!(
        result.contains("Length:") && result.contains("3"),
        "Expected Length: 3 but got: {}",
        result
    );
    assert!(
        result.contains("Elements:"),
        "Expected Elements: section but got: {}",
        result
    );
}

#[test]
fn test_inspect_command_variable() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define a variable
    repl.eval("let data = [10, 20, 30]").unwrap();

    // Inspect the variable
    let result = repl.eval(":inspect data").unwrap();
    assert!(
        result.contains("Type:") && result.contains("Array"),
        "Expected Type: Array but got: {}",
        result
    );
    assert!(
        result.contains("Length:") && result.contains("3"),
        "Expected Length: 3 but got: {}",
        result
    );
}

#[test]
fn test_inspect_command_nested_structure() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test nested array
    let result = repl.eval(":inspect [[1, 2], [3, 4]]").unwrap();
    assert!(
        result.contains("Type:") && result.contains("Array"),
        "Expected Type: Array but got: {}",
        result
    );
    assert!(
        result.contains("Length:") && result.contains("2"),
        "Expected Length: 2 but got: {}",
        result
    );
    // Should show nested structure
    assert!(
        result.contains("[0]:") || result.contains("Element 0:"),
        "Expected element indexing but got: {}",
        result
    );
}

#[test]
fn test_inspect_command_no_args() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test without arguments
    let result = repl.eval(":inspect").unwrap();
    assert!(
        result.contains("Usage") || result.contains("inspect <expression>"),
        "Expected usage message but got: {}",
        result
    );
}

#[test]
fn test_inspect_command_format() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test format structure
    let result = repl.eval(":inspect 42").unwrap();

    // Should contain structured output
    assert!(
        result.contains("Type:"),
        "Expected 'Type:' label but got: {}",
        result
    );
    assert!(
        result.contains("Value:"),
        "Expected 'Value:' label but got: {}",
        result
    );
}
