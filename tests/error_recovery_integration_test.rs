// Integration test for REPL-UX-001: Error recovery system integration
// Validates that error recovery works in realistic usage scenarios

use ruchy::runtime::{Repl, RecoveryOption, RecoveryResult};

#[test]
fn test_error_recovery_integration_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Try to evaluate an expression that will cause an error
    let result = repl.eval("let x = ");
    assert!(result.is_err(), "Should fail with incomplete let statement");
    
    // Should have error recovery available
    assert!(repl.has_error_recovery(), "Should have error recovery context");
    
    // Get error recovery prompt
    let prompt = repl.get_error_recovery_prompt();
    assert!(prompt.is_some(), "Should have error recovery prompt");
    
    let prompt_text = prompt.unwrap();
    assert!(prompt_text.contains("Recovery Options"), "Should contain recovery options");
    assert!(prompt_text.contains("1."), "Should have numbered options");
    
    // Get the recovery context and apply a recovery option
    if let Some(recovery) = repl.get_error_recovery() {
        // Find a suitable recovery option
        let recovery_option = recovery.options.iter()
            .find(|opt| matches!(opt, RecoveryOption::ContinueWithDefault(_)))
            .expect("Should have a continue with default option");
        
        // Apply the recovery
        let result = repl.apply_recovery(recovery_option.clone()).unwrap();
        
        match result {
            RecoveryResult::Recovered(expr) => {
                // The recovered expression should be evaluable
                let eval_result = repl.eval(&expr);
                assert!(eval_result.is_ok(), "Recovered expression should evaluate successfully");
                
                // Error recovery context should be cleared
                assert!(!repl.has_error_recovery(), "Error recovery context should be cleared");
            }
            _ => panic!("Expected recovered result"),
        }
    }
}

#[test]
fn test_error_recovery_workflow_with_real_errors() {
    let mut repl = Repl::new().unwrap();
    
    // Set up some context
    let _ = repl.eval("let valid_var = 42");
    
    // Try various error scenarios
    let error_cases = vec![
        ("let incomplete = ", "incomplete let statement"),
        ("undefined_variable", "undefined variable"),
        ("if ", "incomplete if statement"),
        ("func(", "incomplete function call"),
    ];
    
    for (input, description) in error_cases {
        println!("Testing: {description}");
        
        // Evaluate the erroneous input
        let result = repl.eval(input);
        assert!(result.is_err(), "Should fail for: {description}");
        
        // Should create error recovery context
        assert!(repl.has_error_recovery(), 
            "Should have error recovery for: {description}");
        
        // Should be able to get formatted recovery prompt
        let prompt = repl.get_error_recovery_prompt();
        assert!(prompt.is_some(), "Should have recovery prompt for: {description}");
        
        // Clear error recovery for next test
        repl.clear_error_recovery();
        assert!(!repl.has_error_recovery(), "Should clear error recovery");
    }
}

#[test]
fn test_error_recovery_with_suggestions() {
    let mut repl = Repl::new().unwrap();
    
    // Set up variables that can be suggested
    let _ = repl.eval("let user_name = \"Alice\"");
    let _ = repl.eval("let user_age = 25");
    
    // Try a typo that should trigger suggestions
    let result = repl.eval("user_nam");
    assert!(result.is_err(), "Should fail with undefined variable");
    
    assert!(repl.has_error_recovery(), "Should create error recovery");
    
    if let Some(recovery) = repl.get_error_recovery() {
        // Should suggest similar variable names
        let has_suggestion = recovery.options.iter().any(|opt| match opt {
            RecoveryOption::RetryWith(expr) => expr.contains("user_name"),
            _ => false,
        });
        
        assert!(has_suggestion, "Should suggest 'user_name' for typo 'user_nam'");
    }
}

#[test]
fn test_error_recovery_with_history() {
    let mut repl = Repl::new().unwrap();
    
    // Build up some history
    let _ = repl.eval("42");
    let _ = repl.eval("\"test string\"");
    let _ = repl.eval("true");
    
    // Create an error that should suggest history usage
    let result = repl.eval("broken_expression");
    assert!(result.is_err(), "Should fail with parse error");
    
    assert!(repl.has_error_recovery(), "Should create error recovery");
    
    if let Some(recovery) = repl.get_error_recovery() {
        // Should suggest using history values
        let has_history_suggestion = recovery.options.iter().any(|opt| 
            matches!(opt, RecoveryOption::UseHistoryValue(_))
        );
        
        assert!(has_history_suggestion, "Should suggest using history values");
        
        // Try applying a history recovery
        let history_option = recovery.options.iter()
            .find(|opt| matches!(opt, RecoveryOption::UseHistoryValue(1)))
            .expect("Should have history option for _1");
        
        let result = repl.apply_recovery(history_option.clone()).unwrap();
        match result {
            RecoveryResult::Recovered(expr) => {
                assert_eq!(expr, "_1", "Should recover with _1 reference");
                
                // Should be able to evaluate the history reference
                let eval_result = repl.eval(&expr);
                assert!(eval_result.is_ok(), "History reference should evaluate");
                assert_eq!(eval_result.unwrap().trim(), "42", "Should get first history value");
            }
            _ => panic!("Expected recovered result with history reference"),
        }
    }
}

#[test]
fn test_error_recovery_checkpoint_restoration() {
    let mut repl = Repl::new().unwrap();
    
    // Set up initial state
    let _ = repl.eval("let original = 100");
    let _ = repl.eval("let shared = 200");
    
    // Create an error (which creates a checkpoint)
    let result = repl.eval("invalid_syntax");
    assert!(result.is_err(), "Should fail with parse error");
    
    // Add more state after the error
    let _ = repl.eval("let after_error = 300");
    
    // Verify the new state exists
    let check_result = repl.eval("after_error");
    assert!(check_result.is_ok(), "New variable should exist");
    
    // Apply checkpoint recovery
    assert!(repl.has_error_recovery(), "Should have error recovery");
    
    if let Some(recovery) = repl.get_error_recovery() {
        let checkpoint_option = recovery.options.iter()
            .find(|opt| matches!(opt, RecoveryOption::RestoreCheckpoint))
            .expect("Should have checkpoint recovery option");
        
        let result = repl.apply_recovery(checkpoint_option.clone()).unwrap();
        match result {
            RecoveryResult::Restored => {
                // Original variables should still exist
                let orig_result = repl.eval("original");
                assert!(orig_result.is_ok(), "Original variable should be restored");
                assert_eq!(orig_result.unwrap().trim(), "100");
                
                let shared_result = repl.eval("shared");
                assert!(shared_result.is_ok(), "Shared variable should be restored");  
                assert_eq!(shared_result.unwrap().trim(), "200");
                
                // Variable added after error should be gone
                let after_result = repl.eval("after_error");
                assert!(after_result.is_err(), "Variable added after error should not exist");
            }
            _ => panic!("Expected restored result"),
        }
    }
}

#[test]
fn test_multiple_error_recovery_cycles() {
    let mut repl = Repl::new().unwrap();
    
    // First error cycle
    let result = repl.eval("let x = ");
    assert!(result.is_err(), "First error should fail");
    assert!(repl.has_error_recovery(), "Should create first recovery context");
    
    // Abort first recovery
    let result = repl.apply_recovery(RecoveryOption::Abort).unwrap();
    match result {
        RecoveryResult::Aborted => {
            assert!(!repl.has_error_recovery(), "Should clear recovery after abort");
        }
        _ => panic!("Expected aborted result"),
    }
    
    // Second error cycle  
    let result = repl.eval("undefined_var");
    assert!(result.is_err(), "Second error should fail");
    assert!(repl.has_error_recovery(), "Should create second recovery context");
    
    // Successfully recover from second error
    if let Some(recovery) = repl.get_error_recovery() {
        let retry_option = recovery.options.iter()
            .find(|opt| matches!(opt, RecoveryOption::RetryWith(_)))
            .or_else(|| recovery.options.iter()
                .find(|opt| matches!(opt, RecoveryOption::ContinueWithDefault(_))))
            .expect("Should have a retry or continue option");
        
        let result = repl.apply_recovery(retry_option.clone()).unwrap();
        match result {
            RecoveryResult::Recovered(_) => {
                assert!(!repl.has_error_recovery(), "Should clear recovery after success");
            }
            _ => panic!("Expected recovered result"),
        }
    }
    
    // Third error cycle - different type of error
    let result = repl.eval("if ");
    assert!(result.is_err(), "Third error should fail");
    assert!(repl.has_error_recovery(), "Should create third recovery context");
    
    // Should have different options for different error type
    if let Some(recovery) = repl.get_error_recovery() {
        assert!(!recovery.options.is_empty(), "Should have recovery options");
    }
}