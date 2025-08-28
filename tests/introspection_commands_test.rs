// Test for REPL-UX-003: Rich introspection commands (:env, :type, :ast, :inspect)
// Validates comprehensive introspection functionality

use ruchy::runtime::Repl;

#[test]
fn test_env_command_empty() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval(":env").unwrap();
    assert_eq!(result, "No bindings");
    
    // :bindings should be an alias for :env
    let result = repl.eval(":bindings").unwrap();
    assert_eq!(result, "No bindings");
}

#[test]
fn test_env_command_with_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Create some bindings
    let _ = repl.eval("let x = 42");
    let _ = repl.eval("let name = \"Alice\"");
    let _ = repl.eval("let items = [1, 2, 3]");
    
    let result = repl.eval(":env").unwrap();
    assert!(result.contains("x: 42"));
    assert!(result.contains("name: \"Alice\""));
    assert!(result.contains("items: [1, 2, 3]"));
}

#[test]
fn test_type_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test type of literal
    let result = repl.eval(":type 42").unwrap();
    assert!(result.contains("Type:"));
    assert!(result.contains("i32") || result.contains("Int"));
    
    // Test type of string
    let result = repl.eval(":type \"hello\"").unwrap();
    assert!(result.contains("String"));
    
    // Test type of list
    let result = repl.eval(":type [1, 2, 3]").unwrap();
    assert!(result.contains("[") || result.contains("List") || result.contains("Array"));
    
    // Test type of boolean
    let result = repl.eval(":type true").unwrap();
    assert!(result.contains("bool") || result.contains("Bool"));
}

#[test]
fn test_type_command_with_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Create a binding
    let _ = repl.eval("let my_number = 3.14");
    
    // Check type of the binding
    let result = repl.eval(":type my_number").unwrap();
    assert!(result.contains("Type:"));
    assert!(result.contains("Float"));
}

#[test]
fn test_ast_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test AST of simple expression
    let result = repl.eval(":ast 1 + 2").unwrap();
    assert!(result.contains("Binary") || result.contains("BinaryOp"));
    assert!(result.contains("Add"));
    
    // Test AST of function call
    let result = repl.eval(":ast print(\"hello\")").unwrap();
    assert!(result.contains("Call") || result.contains("FunctionCall"));
    assert!(result.contains("print"));
    
    // Test AST of literal
    let result = repl.eval(":ast 42").unwrap();
    assert!(result.contains("Literal") || result.contains("Int"));
    assert!(result.contains("42"));
}

#[test]
fn test_ast_command_complex() {
    let mut repl = Repl::new().unwrap();
    
    // Test AST of if expression
    let result = repl.eval(":ast if true { 1 } else { 2 }").unwrap();
    assert!(result.contains("If"));
    assert!(result.contains("true"));
    assert!(result.contains("1"));
    assert!(result.contains("2"));
}

#[test]
fn test_inspect_command_not_found() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval(":inspect nonexistent").unwrap();
    assert!(result.contains("Variable 'nonexistent' not found"));
    assert!(result.contains(":env"));
}

#[test]
fn test_inspect_command_simple_values() {
    let mut repl = Repl::new().unwrap();
    
    // Inspect an integer
    let _ = repl.eval("let num = 42");
    let result = repl.eval(":inspect num").unwrap();
    assert!(result.contains("┌─ Inspector"));
    assert!(result.contains("Variable: num"));
    assert!(result.contains("Type: Int"));
    assert!(result.contains("Value: 42"));
    assert!(result.contains("Memory:"));
    assert!(result.contains("└─"));
    
    // Inspect a string
    let _ = repl.eval("let greeting = \"Hello, World!\"");
    let result = repl.eval(":inspect greeting").unwrap();
    assert!(result.contains("Variable: greeting"));
    assert!(result.contains("Type: String"));
    assert!(result.contains("Length: 13 chars"));
    
    // Inspect a boolean
    let _ = repl.eval("let flag = true");
    let result = repl.eval(":inspect flag").unwrap();
    assert!(result.contains("Variable: flag"));
    assert!(result.contains("Type: Bool"));
    assert!(result.contains("Value: true"));
}

#[test]
fn test_inspect_command_collections() {
    let mut repl = Repl::new().unwrap();
    
    // Inspect a list
    let _ = repl.eval("let numbers = [1, 2, 3, 4, 5]");
    let result = repl.eval(":inspect numbers").unwrap();
    assert!(result.contains("Variable: numbers"));
    assert!(result.contains("Type: List"));
    assert!(result.contains("Length: 5"));
    assert!(result.contains("[Enter] Browse entries"));
    assert!(result.contains("[S] Statistics"));
    
    // Inspect an object
    let _ = repl.eval("let person = {name: \"Alice\", age: 30}");
    let result = repl.eval(":inspect person").unwrap();
    assert!(result.contains("Variable: person"));
    assert!(result.contains("Type: Object"));
    assert!(result.contains("Fields: 2"));
    assert!(result.contains("[Enter] Browse entries"));
}

#[test]
fn test_inspect_command_functions() {
    let mut repl = Repl::new().unwrap();
    
    // Create a function
    let _ = repl.eval("let add = fn(a, b) { a + b }");
    let result = repl.eval(":inspect add").unwrap();
    assert!(result.contains("Variable: add"));
    assert!(result.contains("Type:"));
    assert!(result.contains("Function") || result.contains("Lambda"));
    assert!(result.contains("[P] Show parameters"));
    assert!(result.contains("[B] Show body"));
}

#[test]
fn test_inspect_memory_estimation() {
    let mut repl = Repl::new().unwrap();
    
    // Small integer
    let _ = repl.eval("let small = 42");
    let result = repl.eval(":inspect small").unwrap();
    assert!(result.contains("Memory: ~8 bytes"));
    
    // String with content
    let _ = repl.eval("let text = \"Hello\"");
    let result = repl.eval(":inspect text").unwrap();
    assert!(result.contains("Memory: ~"));
    assert!(result.contains("bytes"));
    
    // Large list
    let _ = repl.eval("let big_list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
    let result = repl.eval(":inspect big_list").unwrap();
    assert!(result.contains("Memory: ~"));
}

#[test]
fn test_all_introspection_commands_workflow() {
    let mut repl = Repl::new().unwrap();
    
    // Setup: Create various types of data
    let _ = repl.eval("let x = 42");
    let _ = repl.eval("let y = x * 2");
    let _ = repl.eval("let data = [x, y, 100]");
    
    // 1. Check environment
    let env_result = repl.eval(":env").unwrap();
    assert!(env_result.contains("x: 42"));
    assert!(env_result.contains("y: 84"));
    assert!(env_result.contains("data:"));
    
    // 2. Check types
    let type_result = repl.eval(":type data").unwrap();
    assert!(type_result.contains("Type:"));
    assert!(type_result.contains("List"));
    
    // 3. Check AST
    let ast_result = repl.eval(":ast x + y").unwrap();
    assert!(ast_result.contains("Ident"));
    assert!(ast_result.contains("x"));
    assert!(ast_result.contains("y"));
    
    // 4. Inspect values
    let inspect_result = repl.eval(":inspect data").unwrap();
    assert!(inspect_result.contains("┌─ Inspector"));
    assert!(inspect_result.contains("Type: List"));
    assert!(inspect_result.contains("Length: 3"));
}

#[test]
fn test_command_error_handling() {
    let mut repl = Repl::new().unwrap();
    
    // :type without argument
    let result = repl.eval(":type").unwrap();
    assert!(result.contains("Usage: :type <expression>"));
    
    // :ast without argument
    let result = repl.eval(":ast").unwrap();
    assert!(result.contains("Usage: :ast <expression>"));
    
    // :inspect without argument
    let result = repl.eval(":inspect").unwrap();
    assert!(result.contains("Usage: :inspect <variable>"));
    
    // :type with invalid expression
    let result = repl.eval(":type +++").unwrap();
    assert!(result.contains("error") || result.contains("Error"));
    
    // :ast with invalid expression
    let result = repl.eval(":ast +++").unwrap();
    assert!(result.contains("error") || result.contains("Error"));
}