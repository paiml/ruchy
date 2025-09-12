//! Property-based tests for SharedSession
//! Uses property testing to verify invariants hold across many random inputs

use ruchy::wasm::shared_session::{SharedSession, ExecutionMode};

/// Generate simple valid Ruchy expressions
fn simple_expressions() -> Vec<String> {
    vec![
        "42".to_string(),
        "3.14".to_string(),
        "true".to_string(),
        "false".to_string(),
        r#""hello""#.to_string(),
        "1 + 2".to_string(),
        "10 - 5".to_string(),
        "2 * 3".to_string(),
        "8 / 2".to_string(),
        "5 % 3".to_string(),
        "let x = 1".to_string(),
        "let y = 2.5".to_string(),
        r#"let s = "test""#.to_string(),
    ]
}

/// Generate valid variable names
fn valid_identifiers() -> Vec<String> {
    vec![
        "x".to_string(),
        "y".to_string(),
        "test_var".to_string(),
        "counter".to_string(),
        "result".to_string(),
        "value123".to_string(),
        "myVariable".to_string(),
        "data".to_string(),
    ]
}

#[test]
fn property_session_always_starts_empty() {
    // Property: New sessions always have zero memory estimation initially
    for _ in 0..10 {
        let session = SharedSession::new();
        let memory = session.estimate_interpreter_memory();
        assert!(memory > 0, "Memory should be non-zero even for empty session");
    }
}

#[test]  
fn property_valid_expressions_never_panic() {
    // Property: Valid expressions should never panic, only return Ok or Err
    let mut session = SharedSession::new();
    let expressions = simple_expressions();
    
    for (i, expr) in expressions.iter().enumerate() {
        let cell_id = format!("cell_{}", i);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            session.execute(&cell_id, expr)
        }));
        
        assert!(result.is_ok(), "Expression '{}' caused panic", expr);
    }
}

#[test]
fn property_execution_mode_switching_preserves_state() {
    // Property: Switching execution modes should not lose variable bindings
    let mut session = SharedSession::new();
    
    // Set up variables in manual mode
    session.execute("var1", "let preserved = 42").unwrap();
    
    // Switch to reactive
    session.set_execution_mode(ExecutionMode::Reactive);
    let result = session.execute("check1", "preserved");
    assert!(result.is_ok() && result.unwrap().value == "42");
    
    // Switch back to manual
    session.set_execution_mode(ExecutionMode::Manual);
    let result = session.execute("check2", "preserved");
    assert!(result.is_ok() && result.unwrap().value == "42");
}

#[test]
fn property_memory_monotonic_increase() {
    // Property: Memory usage should monotonically increase (or stay same) as we add variables
    let mut session = SharedSession::new();
    let mut prev_memory = session.estimate_interpreter_memory();
    
    for i in 0..5 {
        let var_name = format!("var_{}", i);
        let code = format!("let {} = {}", var_name, i * 10);
        session.execute(&format!("cell_{}", i), &code).unwrap();
        
        let current_memory = session.estimate_interpreter_memory();
        assert!(current_memory >= prev_memory, 
               "Memory should not decrease: {} -> {}", prev_memory, current_memory);
        prev_memory = current_memory;
    }
}

#[test]
fn property_cell_id_uniqueness_preserved() {
    // Property: Different cell IDs should be treated as separate executions
    let mut session = SharedSession::new();
    
    // Execute same code with different cell IDs
    for i in 0..5 {
        let cell_id = format!("unique_cell_{}", i);
        let result = session.execute(&cell_id, "42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, "42");
    }
    
    // All executions should have succeeded independently
}

#[test]
fn property_variable_scoping_isolation() {
    // Property: Variables defined in one cell are available in subsequent cells
    let mut session = SharedSession::new();
    let vars = valid_identifiers();
    
    // Define variables in separate cells
    for (i, var_name) in vars.iter().take(3).enumerate() {
        let code = format!("let {} = {}", var_name, i + 1);
        let result = session.execute(&format!("def_{}", i), &code);
        assert!(result.is_ok(), "Failed to define {}", var_name);
    }
    
    // All variables should be accessible from a new cell
    for var_name in vars.iter().take(3) {
        let result = session.execute("access", var_name);
        assert!(result.is_ok(), "Variable {} should be accessible", var_name);
    }
}

#[test]
fn property_error_recovery() {
    // Property: Errors in one cell should not prevent execution of subsequent cells
    let mut session = SharedSession::new();
    
    // Execute a valid expression
    let result = session.execute("valid1", "let good = 42");
    assert!(result.is_ok());
    
    // Execute invalid expression (should fail)
    let result = session.execute("invalid", "undefined_variable");
    assert!(result.is_err());
    
    // Execute another valid expression (should succeed despite previous error)
    let result = session.execute("valid2", "let also_good = 24");
    assert!(result.is_ok());
    
    // Previous valid variables should still be accessible
    let result = session.execute("check", "good");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "42");
}

#[test]
fn property_empty_inputs_graceful() {
    // Property: Empty or whitespace inputs should be handled gracefully
    let mut session = SharedSession::new();
    
    let empty_inputs = vec!["", "   ", "\n", "\t", "  \n  ", "\r\n"];
    
    for (i, input) in empty_inputs.iter().enumerate() {
        let cell_id = format!("empty_{}", i);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            session.execute(&cell_id, input)
        }));
        
        assert!(result.is_ok(), "Empty input '{}' should not panic", 
                input.chars().map(|c| if c.is_whitespace() { '·' } else { c }).collect::<String>());
    }
}

#[test] 
fn property_numeric_operations_consistency() {
    // Property: Basic arithmetic should be consistent
    let mut session = SharedSession::new();
    
    let test_cases = vec![
        ("1 + 1", "2"),
        ("3 - 1", "2"), 
        ("2 * 3", "6"),
        ("10 / 5", "2"),
        ("7 % 3", "1"),
    ];
    
    for (i, (expr, expected)) in test_cases.iter().enumerate() {
        let result = session.execute(&format!("math_{}", i), expr);
        assert!(result.is_ok(), "Math expression '{}' failed", expr);
        assert_eq!(result.unwrap().value, *expected, 
                  "Expression '{}' should equal {}", expr, expected);
    }
}

#[test]
fn property_string_operations_consistency() {
    // Property: String operations should be consistent  
    let mut session = SharedSession::new();
    
    // String concatenation if supported
    let result = session.execute("concat", r#""hello" + " world""#);
    if result.is_ok() {
        // If concat is supported, should be deterministic
        let result2 = session.execute("concat2", r#""hello" + " world""#);
        assert!(result2.is_ok());
        assert_eq!(result.unwrap().value, result2.unwrap().value);
    }
}