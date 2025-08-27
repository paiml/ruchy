// TDD Test Suite for Broken REPL Commands
// Testing all documented REPL commands that are currently broken or incomplete

use ruchy::runtime::repl::Repl;
use std::fs;

#[test]
fn test_type_command_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Test :type command with basic expressions
    let result = repl.eval(":type 42").unwrap();
    assert!(result.contains("Integer") || result.contains("i32") || result.contains("Int"),
        "Expected type information for integer, got: {}", result);
    
    let result = repl.eval(":type \"hello\"").unwrap();
    assert!(result.contains("String") || result.contains("str"),
        "Expected type information for string, got: {}", result);
    
    let result = repl.eval(":type true").unwrap();
    assert!(result.contains("Bool") || result.contains("bool") || result.contains("boolean"),
        "Expected type information for boolean, got: {}", result);
}

#[test]
fn test_type_command_with_variables() {
    let mut repl = Repl::new().unwrap();
    
    // Define a variable first
    repl.eval("let x = 42").unwrap();
    
    // Test :type on variables
    let result = repl.eval(":type x").unwrap();
    assert!(result.contains("Integer") || result.contains("i32") || result.contains("Int"),
        "Expected type information for variable x, got: {}", result);
}

#[test]
fn test_type_command_with_functions() {
    let mut repl = Repl::new().unwrap();
    
    // Define a function
    repl.eval("fn add(a, b) { a + b }").unwrap();
    
    // Test :type on functions  
    let result = repl.eval(":type add").unwrap();
    assert!(result.contains("Function") || result.contains("fn") || result.contains("->"),
        "Expected type information for function, got: {}", result);
}

#[test]
fn test_ast_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test :ast command
    let result = repl.eval(":ast 2 + 2").unwrap();
    assert!(result.contains("Binary") || result.contains("Add") || result.contains("+"),
        "Expected AST representation, got: {}", result);
    
    // Test with more complex expression
    let result = repl.eval(":ast if true { 1 } else { 0 }").unwrap();
    assert!(result.contains("If") || result.contains("Conditional"),
        "Expected AST with If node, got: {}", result);
}

#[test]
fn test_compile_command() {
    let mut repl = Repl::new().unwrap();
    
    // Add some code to compile
    repl.eval("let x = 10").unwrap();
    repl.eval("fn double(n) { n * 2 }").unwrap();
    
    // Test :compile command
    let result = repl.eval(":compile").unwrap();
    assert!(!result.contains("Error") && !result.contains("failed"),
        "Compile command should work, got: {}", result);
}

#[test]
fn test_save_load_commands() {
    let mut repl = Repl::new().unwrap();
    let test_file = "/tmp/test_repl_session.ruchy";
    
    // Add some history
    repl.eval("let x = 42").unwrap();
    repl.eval("let y = 100").unwrap();
    repl.eval("x + y").unwrap();
    
    // Save session
    let save_result = repl.eval(&format!(":save {}", test_file)).unwrap();
    assert!(!save_result.contains("Error") && !save_result.contains("failed"),
        "Save command should work, got: {}", save_result);
    
    // Check file exists
    assert!(fs::metadata(test_file).is_ok(), "Save should create file");
    
    // Create new REPL and load
    let mut new_repl = Repl::new().unwrap();
    let load_result = new_repl.eval(&format!(":load {}", test_file)).unwrap();
    assert!(!load_result.contains("Error") && !load_result.contains("failed"),
        "Load command should work, got: {}", load_result);
    
    // Variables should be available
    let x_val = new_repl.eval("x").unwrap();
    assert_eq!(x_val, "42", "Loaded session should have x = 42");
    
    let y_val = new_repl.eval("y").unwrap();
    assert_eq!(y_val, "100", "Loaded session should have y = 100");
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_bindings_env_commands() {
    let mut repl = Repl::new().unwrap();
    
    // Define some variables
    repl.eval("let x = 10").unwrap();
    repl.eval("let name = \"Ruchy\"").unwrap();
    repl.eval("fn greet(n) { f\"Hello {n}\" }").unwrap();
    
    // Test :bindings command
    let result = repl.eval(":bindings").unwrap();
    assert!(result.contains("x") && result.contains("10"),
        ":bindings should show x = 10, got: {}", result);
    assert!(result.contains("name") && result.contains("Ruchy"),
        ":bindings should show name = Ruchy, got: {}", result);
    assert!(result.contains("greet"),
        ":bindings should show greet function, got: {}", result);
    
    // Test :env command (alias for :bindings)
    let env_result = repl.eval(":env").unwrap();
    assert!(env_result.contains("x") && env_result.contains("10"),
        ":env should show x = 10, got: {}", env_result);
}

#[test]
fn test_history_command() {
    let mut repl = Repl::new().unwrap();
    
    // Execute some commands
    repl.eval("1 + 1").unwrap();
    repl.eval("\"hello\"").unwrap();
    repl.eval("[1, 2, 3]").unwrap();
    
    // Test :history command
    let result = repl.eval(":history").unwrap();
    assert!(result.contains("1 + 1") || result.contains("2"),
        ":history should show first command, got: {}", result);
    assert!(result.contains("hello"),
        ":history should show second command, got: {}", result);
    assert!(result.contains("[1, 2, 3]") || result.contains("1, 2, 3"),
        ":history should show third command, got: {}", result);
}

#[test]
fn test_search_command() {
    let mut repl = Repl::new().unwrap();
    
    // Add diverse history
    repl.eval("let x = 42").unwrap();
    repl.eval("let name = \"Alice\"").unwrap();
    repl.eval("fn calculate(n) { n * 2 }").unwrap();
    repl.eval("calculate(21)").unwrap();
    
    // Test :search command
    let result = repl.eval(":search calculate").unwrap();
    assert!(result.contains("calculate"),
        ":search should find 'calculate' entries, got: {}", result);
    
    let result = repl.eval(":search 42").unwrap();
    assert!(result.contains("42"),
        ":search should find entries with '42', got: {}", result);
}

#[test]
fn test_reset_command() {
    let mut repl = Repl::new().unwrap();
    
    // Add state
    repl.eval("let x = 100").unwrap();
    repl.eval("fn test() { 42 }").unwrap();
    
    // Verify state exists
    assert_eq!(repl.eval("x").unwrap(), "100");
    assert_eq!(repl.eval("test()").unwrap(), "42");
    
    // Test :reset command
    let result = repl.eval(":reset").unwrap();
    assert!(!result.contains("Error"),
        ":reset should work, got: {}", result);
    
    // State should be cleared
    let x_result = repl.eval("x");
    assert!(x_result.as_ref().is_err() || 
            x_result.as_ref().unwrap().contains("not defined") || 
            x_result.as_ref().unwrap().contains("undefined") || 
            x_result.as_ref().unwrap().contains("Error"),
        "x should be undefined after reset");
}

#[test]
fn test_help_command_completeness() {
    let mut repl = Repl::new().unwrap();
    
    // Test that :help lists all commands
    let result = repl.eval(":help").unwrap();
    
    // Check all documented commands are listed
    assert!(result.contains(":type"), ":help should list :type command");
    assert!(result.contains(":ast"), ":help should list :ast command");
    assert!(result.contains(":compile"), ":help should list :compile command");
    assert!(result.contains(":load"), ":help should list :load command");
    assert!(result.contains(":save"), ":help should list :save command");
    assert!(result.contains(":bindings") || result.contains(":env"), 
        ":help should list :bindings/:env command");
    assert!(result.contains(":history"), ":help should list :history command");
    assert!(result.contains(":search"), ":help should list :search command");
    assert!(result.contains(":clear"), ":help should list :clear command");
    assert!(result.contains(":reset"), ":help should list :reset command");
    assert!(result.contains(":quit"), ":help should list :quit command");
}

#[test]
fn test_clear_command() {
    let mut repl = Repl::new().unwrap();
    
    // Add history and definitions
    repl.eval("let x = 10").unwrap();
    repl.eval("let y = 20").unwrap();
    repl.eval("x + y").unwrap();
    
    // Test :clear command
    let result = repl.eval(":clear").unwrap();
    assert!(!result.contains("Error"),
        ":clear should work, got: {}", result);
    
    // Definitions should be cleared
    let x_result = repl.eval("x");
    assert!(x_result.as_ref().is_err() || 
            x_result.as_ref().unwrap().contains("not defined") || 
            x_result.as_ref().unwrap().contains("undefined") || 
            x_result.as_ref().unwrap().contains("Error"),
        "x should be undefined after clear");
    
    // History should be cleared (test via :history)
    let history_result = repl.eval(":history").unwrap();
    assert!(history_result.contains("empty") || history_result.contains("No history") || 
            history_result.len() < 50,  // Short response indicates empty
        "History should be empty after clear, got: {}", history_result);
}

#[test]
fn test_command_error_handling() {
    let mut repl = Repl::new().unwrap();
    
    // Test invalid commands
    let result = repl.eval(":unknown").unwrap();
    assert!(result.contains("Unknown") || result.contains("Invalid") || 
            result.contains("not recognized"),
        "Should handle unknown command gracefully, got: {}", result);
    
    // Test commands with missing arguments
    let result = repl.eval(":load").unwrap();
    assert!(result.contains("Usage") || result.contains("filename") || 
            result.contains("argument"),
        "Should show usage for :load without args, got: {}", result);
    
    let result = repl.eval(":save").unwrap();
    assert!(result.contains("Usage") || result.contains("filename") || 
            result.contains("argument"),
        "Should show usage for :save without args, got: {}", result);
    
    // Test empty :type
    let result = repl.eval(":type").unwrap();
    assert!(result.contains("Usage") || result.contains("expression"),
        "Should show usage for :type without expression, got: {}", result);
}

#[test]
fn test_multiline_command_support() {
    let mut repl = Repl::new().unwrap();
    
    // Define a function using single-line syntax
    repl.eval("fn test() { 42 }").unwrap();
    
    // Now test that commands work after definitions
    let result = repl.eval(":type test").unwrap();
    assert!(result.contains("Function") || result.contains("fn"),
        "Commands should work after function definition, got: {}", result);
    
    // Test other commands still work
    let bindings_result = repl.eval(":bindings").unwrap();
    assert!(bindings_result.contains("test"),
        "Should show test function in bindings, got: {}", bindings_result);
}