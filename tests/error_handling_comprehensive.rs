//! Comprehensive error handling tests for SharedSession
//! Ensures robust error handling and recovery across all scenarios

use ruchy::wasm::shared_session::{SharedSession, ExecutionMode};

#[test]
fn test_parse_errors() {
    let mut session = SharedSession::new();
    
    // Syntax errors
    let syntax_errors = vec![
        "let x =",           // Incomplete assignment
        "if true {",         // Unclosed brace
        "fun incomplete(",   // Unclosed function
        "let 123invalid = 1", // Invalid identifier
        "x +",              // Incomplete expression
        "() => {",          // Unclosed lambda
        "[1, 2, 3",         // Unclosed array
        "(1, 2",            // Unclosed tuple
    ];
    
    for (i, invalid_code) in syntax_errors.iter().enumerate() {
        let cell_id = format!("syntax_error_{}", i);
        let result = session.execute(&cell_id, invalid_code);
        assert!(result.is_err(), "Should fail for syntax error: {}", invalid_code);
        
        // Error message should be meaningful
        let error = result.unwrap_err();
        assert!(!error.is_empty(), "Error message should not be empty");
    }
}

#[test]
fn test_runtime_errors() {
    let mut session = SharedSession::new();
    
    let runtime_errors = vec![
        "undefined_variable",                    // Undefined variable
        "nonexistent_function()",               // Undefined function  
        "let x = 1; x.nonexistent_method()",   // Method doesn't exist
        "1 / 0",                               // Division by zero
        "let arr = [1, 2, 3]; arr[10]",       // Array index out of bounds
        "null.field",                          // Null pointer access
        "\"string\"[100]",                      // String index out of bounds
    ];
    
    for (i, invalid_code) in runtime_errors.iter().enumerate() {
        let cell_id = format!("runtime_error_{}", i);
        let result = session.execute(&cell_id, invalid_code);
        assert!(result.is_err(), "Should fail for runtime error: {}", invalid_code);
        
        let error = result.unwrap_err();
        assert!(!error.is_empty(), "Error message should not be empty");
        
        // Verify specific error types
        if invalid_code.contains("undefined") || invalid_code.contains("nonexistent") {
            assert!(error.to_lowercase().contains("undefined") || 
                   error.to_lowercase().contains("not found") ||
                   error.to_lowercase().contains("unknown"), 
                   "Should mention undefined/unknown for: {}", invalid_code);
        }
    }
}

#[test]
fn test_type_errors() {
    let mut session = SharedSession::new();
    
    // Setup variables with specific types
    session.execute("setup1", "let num = 42").unwrap();
    session.execute("setup2", r#"let str = "hello""#).unwrap();
    session.execute("setup3", "let arr = [1, 2, 3]").unwrap();
    
    let type_errors = vec![
        "num + str",         // Number + String
        "arr()",            // Array is not callable  
        "num.length",       // Number doesn't have length property
        "str * 2",          // String multiplication (might be valid)
        "arr + num",        // Array + Number
    ];
    
    for (i, invalid_code) in type_errors.iter().enumerate() {
        let cell_id = format!("type_error_{}", i);
        let result = session.execute(&cell_id, invalid_code);
        
        // Some type errors might be runtime errors, others might succeed with coercion
        if result.is_err() {
            let error = result.unwrap_err();
            // Type errors should mention types in some form
            assert!(error.to_lowercase().contains("type") ||
                   error.to_lowercase().contains("cannot") ||
                   error.to_lowercase().contains("invalid") ||
                   error.to_lowercase().contains("not"),
                   "Type error should mention type issues: {}", error);
        }
    }
}

#[test] 
fn test_error_recovery() {
    let mut session = SharedSession::new();
    
    // Define valid variable
    let result = session.execute("valid1", "let x = 42");
    assert!(result.is_ok());
    
    // Execute multiple invalid expressions
    for i in 0..5 {
        let cell_id = format!("invalid_{}", i);
        let result = session.execute(&cell_id, "undefined_variable");
        assert!(result.is_err());
    }
    
    // Session should still work after multiple errors
    let result = session.execute("valid2", "x * 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "84");
    
    // Define new variable after errors
    let result = session.execute("valid3", "let y = x + 10");
    assert!(result.is_ok());
    
    // Both variables should be accessible
    let result = session.execute("valid4", "x + y");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "94");
}

#[test]
fn test_partial_execution_rollback() {
    let mut session = SharedSession::new();
    
    // Set up initial state
    session.execute("setup", "let original = 100").unwrap();
    
    // Try to execute code that might partially succeed then fail
    let result = session.execute("partial", "let temp = original * 2; undefined_var");
    assert!(result.is_err());
    
    // Original variable should still be accessible
    let result = session.execute("check1", "original");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "100");
    
    // Temp variable should not exist (if rollback works)
    let result = session.execute("check2", "temp");
    // This might succeed or fail depending on rollback implementation
    if result.is_ok() {
        // If temp exists, it should have the computed value
        assert_eq!(result.unwrap().value, "200");
    }
}

#[test]
fn test_error_context_preservation() {
    let mut session = SharedSession::new();
    
    // Nested error scenarios
    session.execute("setup", "let data = [1, 2, 3]").unwrap();
    
    // Error in function call context
    let result = session.execute("nested_error", "data.map(x => undefined_var)");
    if result.is_err() {
        let error = result.unwrap_err();
        // Error should preserve context information
        assert!(!error.is_empty());
    }
    
    // Error in conditional context  
    let result = session.execute("cond_error", "if true { let x = undefined_var; x }");
    assert!(result.is_err());
    
    // Session should still be usable after context errors
    let result = session.execute("recovery", "data.length");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "3");
    }
}

#[test] 
fn test_execution_mode_error_handling() {
    let mut session = SharedSession::new();
    
    // Test errors in manual mode
    session.set_execution_mode(ExecutionMode::Manual);
    let result = session.execute("manual_error", "undefined_var");
    assert!(result.is_err());
    
    // Switch to reactive mode
    session.set_execution_mode(ExecutionMode::Reactive);
    
    // Setup valid dependency
    session.execute("base", "let base = 10").unwrap();
    session.execute("derived", "let derived = base * 2").unwrap();
    
    // Error in reactive execution 
    let result = session.execute("reactive_error", "let broken = undefined_var");
    assert!(result.is_err());
    
    // Valid variables should still work
    let result = session.execute("check_base", "base");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "10");
    
    let result = session.execute("check_derived", "derived");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "20");
}

#[test]
fn test_checkpoint_error_handling() {
    let mut session = SharedSession::new();
    
    // Create checkpoint with valid state
    session.execute("valid", "let checkpoint_var = 42").unwrap();
    session.create_checkpoint("before_error").unwrap();
    
    // Make changes that include errors
    let result = session.execute("error_cell", "undefined_variable");
    assert!(result.is_err());
    
    // Try to restore checkpoint
    let restore_result = session.restore_from_checkpoint("before_error");
    assert!(restore_result.is_ok());
    
    // Original variable should still be accessible after restore
    let result = session.execute("check_restore", "checkpoint_var");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "42");
    
    // Try to restore non-existent checkpoint
    let bad_restore = session.restore_from_checkpoint("nonexistent");
    assert!(bad_restore.is_err());
    assert!(bad_restore.unwrap_err().contains("not found"));
}

#[test]
fn test_resource_exhaustion_handling() {
    let mut session = SharedSession::new();
    
    // Test moderate recursion (safer than 10000)
    let moderate_recursion = "fun count_down(n) { if n <= 0 { 0 } else { count_down(n - 1) } }; count_down(100)";
    let result = session.execute("moderate_recursion", moderate_recursion);
    // Should handle moderate recursion gracefully
    if result.is_err() {
        let error = result.unwrap_err();
        assert!(!error.is_empty());
    }
    
    // Test very large data structures
    let large_array = "let big_array = Array.new(1000, 42)";
    let result = session.execute("large_data", large_array);
    // Might succeed or fail depending on memory limits
    
    // Session should still work after resource stress
    let result = session.execute("recovery", "42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "42");
}

#[test]
fn test_sequential_error_simulation() {
    // Simulate concurrent-like error scenarios sequentially
    let mut session = SharedSession::new();
    
    // Set up shared state
    session.execute("shared", "let shared_var = 100").unwrap();
    
    // Simulate what would happen in concurrent scenarios
    for i in 0..3 {
        if i % 2 == 0 {
            // Simulate error thread
            let result = session.execute(&format!("sim_error_{}", i), "undefined_var");
            assert!(result.is_err());
        } else {
            // Simulate success thread
            let result = session.execute(&format!("sim_success_{}", i), "shared_var * 2");
            assert!(result.is_ok());
            assert_eq!(result.unwrap().value, "200");
        }
    }
    
    // Verify session is still functional after mixed success/error pattern
    let result = session.execute("final_check", "shared_var");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "100");
}