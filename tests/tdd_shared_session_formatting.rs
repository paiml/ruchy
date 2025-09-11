//! TDD tests for SharedSession value formatting
//! 
//! These tests ensure proper formatting of values returned from cells,
//! following TDD RED->GREEN->REFACTOR methodology

use ruchy::wasm::notebook::NotebookRuntime;

#[test]
fn test_integer_value_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute cell that returns an integer
    let result = runtime.execute_cell_with_session("cell1", "42").unwrap();
    
    // Should return "42" not "Integer(42)"
    assert_eq!(result.value, "42");
    assert_eq!(result.result, "42");
}

#[test]
fn test_arithmetic_expression_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute arithmetic expression
    let result = runtime.execute_cell_with_session("cell1", "21 + 21").unwrap();
    
    // Should return "42" not "Integer(42)"
    assert_eq!(result.value, "42");
}

#[test]
fn test_variable_assignment_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute variable assignment
    let result = runtime.execute_cell_with_session("cell1", "let x = 42").unwrap();
    
    // Should return "42" as the value of the assignment
    assert_eq!(result.value, "42");
}

#[test]
fn test_variable_reference_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Set up variable
    runtime.execute_cell_with_session("cell1", "let x = 42").unwrap();
    
    // Reference variable
    let result = runtime.execute_cell_with_session("cell2", "x").unwrap();
    
    // Should return "42" not "Integer(42)"
    assert_eq!(result.value, "42");
}

#[test]
fn test_function_call_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Define function
    runtime.execute_cell_with_session("cell1", "fun add(a, b) { a + b }").unwrap();
    
    // Call function
    let result = runtime.execute_cell_with_session("cell2", "add(3, 4)").unwrap();
    
    // Should return "7" not "Integer(7)"
    assert_eq!(result.value, "7");
}

#[test]
fn test_float_value_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute float expression
    let result = runtime.execute_cell_with_session("cell1", "3.14").unwrap();
    
    // Should return "3.14" not "Float(3.14)"
    assert_eq!(result.value, "3.14");
}

#[test]
fn test_string_value_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute string literal
    let result = runtime.execute_cell_with_session("cell1", "\"hello\"").unwrap();
    
    // Should return "hello" (with or without quotes is debatable)
    assert!(result.value == "hello" || result.value == "\"hello\"");
}

#[test]
fn test_boolean_value_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute boolean expression
    let result = runtime.execute_cell_with_session("cell1", "true").unwrap();
    
    // Should return "true" not "Bool(true)"
    assert_eq!(result.value, "true");
}

#[test]
fn test_array_value_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute array literal
    let result = runtime.execute_cell_with_session("cell1", "[1, 2, 3]").unwrap();
    
    // Should return "[1, 2, 3]" not "Array([1, 2, 3])"
    assert_eq!(result.value, "[1, 2, 3]");
}

#[test]
fn test_nil_value_formatting() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Execute nil/null value
    let result = runtime.execute_cell_with_session("cell1", "nil").unwrap();
    
    // Should return "nil" not "Nil"
    assert_eq!(result.value, "nil");
}