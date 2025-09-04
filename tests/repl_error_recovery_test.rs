// Test for REPL-UX-001: Error recovery UI with interactive recovery options
// Validates comprehensive error recovery system with user-friendly options

use ruchy::runtime::Repl;
use ruchy::runtime::repl::{RecoveryOption, RecoveryResult};

#[test]
fn test_error_recovery_creation() {
    let mut repl = Repl::new().unwrap();
    
    // Create error recovery for incomplete let statement
    let recovery = repl.create_error_recovery("let x = ", "Unexpected EOF at line 1:8");
    
    assert_eq!(recovery.failed_expression, "let x = ");
    assert!(recovery.error_message.contains("Unexpected EOF"));
    assert!(!recovery.options.is_empty());
    
    // Should suggest appropriate recovery options
    let has_continue_default = recovery.options.iter().any(|opt| 
        matches!(opt, RecoveryOption::ContinueWithDefault(_))
    );
    let has_retry_with = recovery.options.iter().any(|opt| 
        matches!(opt, RecoveryOption::RetryWith(_))
    );
    let has_show_completions = recovery.options.iter().any(|opt| 
        matches!(opt, RecoveryOption::ShowCompletions)
    );
    
    assert!(has_continue_default, "Should suggest continuing with default");
    assert!(has_retry_with, "Should suggest retry with value");
    assert!(has_show_completions, "Should suggest showing completions");
}

#[test]
fn test_error_recovery_position_parsing() {
    let mut repl = Repl::new().unwrap();
    
    // Test position parsing from error messages
    let recovery = repl.create_error_recovery(
        "let x = ",
        "Unexpected EOF at line 1:8, expected expression"
    );
    
    assert_eq!(recovery.position, Some((1, 8)));
}

#[test]
fn test_undefined_variable_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Set up some variables
    let _ = repl.eval("let variable_name = 42");
    let _ = repl.eval("let similar_var = 100");
    
    // Create recovery for undefined variable with typo
    let recovery = repl.create_error_recovery(
        "variabel_name",
        "undefined variable 'variabel_name' not found"
    );
    
    // Should suggest similar variables
    let retry_options: Vec<_> = recovery.options.iter()
        .filter_map(|opt| match opt {
            RecoveryOption::RetryWith(expr) => Some(expr),
            _ => None,
        })
        .collect();
    
    assert!(!retry_options.is_empty(), "Should suggest similar variables");
    assert!(retry_options.iter().any(|expr| expr.contains("variable_name")),
        "Should suggest 'variable_name' as correction");
}

#[test] 
fn test_type_mismatch_recovery() {
    let mut repl = Repl::new().unwrap();
    
    let recovery = repl.create_error_recovery(
        "42",
        "type mismatch: expected String, found Integer"
    );
    
    // Should suggest type conversion
    let has_to_string = recovery.options.iter().any(|opt| match opt {
        RecoveryOption::RetryWith(expr) => expr.contains(".to_string()"),
        _ => false,
    });
    
    assert!(has_to_string, "Should suggest .to_string() conversion");
}

#[test]
fn test_recovery_option_application() {
    let mut repl = Repl::new().unwrap();
    
    // Test continue with default
    let result = repl.apply_recovery(
        RecoveryOption::ContinueWithDefault("let x = 0".to_string())
    ).unwrap();
    
    match result {
        RecoveryResult::Recovered(expr) => {
            assert_eq!(expr, "let x = 0");
        }
        _ => panic!("Expected Recovered result"),
    }
    
    // Test abort
    let result = repl.apply_recovery(RecoveryOption::Abort).unwrap();
    match result {
        RecoveryResult::Aborted => {}, // Expected
        _ => panic!("Expected Aborted result"),
    }
}

#[test]
fn test_history_value_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Add some history
    let _ = repl.eval("42");
    let _ = repl.eval("100");
    
    let recovery = repl.create_error_recovery("broken_expr", "parse error");
    
    // Should suggest recent history values
    let has_history_option = recovery.options.iter().any(|opt| 
        matches!(opt, RecoveryOption::UseHistoryValue(_))
    );
    assert!(has_history_option, "Should suggest using history values");
    
    // Apply history value recovery
    let result = repl.apply_recovery(RecoveryOption::UseHistoryValue(1)).unwrap();
    match result {
        RecoveryResult::Recovered(expr) => {
            assert_eq!(expr, "_1");
        }
        _ => panic!("Expected Recovered result with history reference"),
    }
}

#[test]
fn test_checkpoint_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Set up initial state
    let _ = repl.eval("let original = 123");
    
    // Create error recovery (automatically creates checkpoint)
    let _recovery = repl.create_error_recovery("invalid syntax", "parse error");
    
    // Modify state after error
    let _ = repl.eval("let new_var = 456");
    
    // Apply checkpoint recovery
    let result = repl.apply_recovery(RecoveryOption::RestoreCheckpoint).unwrap();
    match result {
        RecoveryResult::Restored => {
            // Verify original state is restored
            let result = repl.eval("original");
            assert!(result.is_ok());
            assert_eq!(result.unwrap().trim(), "123");
            
            // New variable should not exist (restored from checkpoint before it was added)
            let result = repl.eval("new_var");
            assert!(result.is_err());
        }
        _ => panic!("Expected Restored result"),
    }
}

#[test]
fn test_completions_generation() {
    let mut repl = Repl::new().unwrap();
    
    // Set up some variables
    let _ = repl.eval("let test_var = 42");
    let _ = repl.eval("let testing = 100");
    
    // Test empty expression completions
    let recovery = repl.create_error_recovery("", "expected expression");
    assert!(!recovery.completions.is_empty(), "Should provide completions for empty input");
    
    // Should include common keywords
    assert!(recovery.completions.iter().any(|c| c == "let "));
    assert!(recovery.completions.iter().any(|c| c == "if "));
    
    // Test partial variable name completions
    let completions = repl.generate_completions_for_error("test");
    assert!(completions.iter().any(|c| c == "test_var"), 
        "Should suggest 'test_var' for partial 'test'");
    assert!(completions.iter().any(|c| c == "testing"), 
        "Should suggest 'testing' for partial 'test'");
}

#[test]
fn test_show_completions_recovery() {
    let mut repl = Repl::new().unwrap();
    
    let _ = repl.eval("let completion_test = 42");
    
    let _recovery = repl.create_error_recovery("comp", "expected expression");
    
    let result = repl.apply_recovery(RecoveryOption::ShowCompletions).unwrap();
    match result {
        RecoveryResult::ShowCompletions(completions) => {
            assert!(!completions.is_empty(), "Should return completions");
        }
        _ => panic!("Expected ShowCompletions result"),
    }
}

#[test] 
fn test_error_recovery_context_management() {
    let mut repl = Repl::new().unwrap();
    
    // Initially no recovery context
    assert!(repl.get_error_recovery().is_none());
    
    // Create recovery context
    let _recovery = repl.create_error_recovery("test", "error");
    assert!(repl.get_error_recovery().is_some());
    
    // Clear recovery context
    repl.clear_error_recovery();
    assert!(repl.get_error_recovery().is_none());
}

#[test]
fn test_error_recovery_formatting() {
    let mut repl = Repl::new().unwrap();
    
    let recovery = repl.create_error_recovery(
        "let x = ",
        "Unexpected EOF at line 1:8"
    );
    
    let formatted = repl.format_error_recovery(&recovery);
    
    // Should include error message
    assert!(formatted.contains("Unexpected EOF"));
    
    // Should include failed expression
    assert!(formatted.contains("let x = "));
    
    // Should include recovery options
    assert!(formatted.contains("Recovery Options:"));
    assert!(formatted.contains("1."));
    
    // Should include instructions
    assert!(formatted.contains("Enter option number"));
}

#[test]
fn test_edit_distance_calculation() {
    let repl = Repl::new().unwrap();
    
    // Test exact match
    assert_eq!(repl.edit_distance("hello", "hello"), 0);
    
    // Test single character differences
    assert_eq!(repl.edit_distance("hello", "hallo"), 1); // substitution
    assert_eq!(repl.edit_distance("hello", "hllo"), 1);  // deletion
    assert_eq!(repl.edit_distance("hello", "helllo"), 1); // insertion
    
    // Test multiple character differences
    assert_eq!(repl.edit_distance("hello", "world"), 4);
    assert_eq!(repl.edit_distance("kitten", "sitting"), 3);
}

#[test]
fn test_recovery_for_incomplete_expressions() {
    let mut repl = Repl::new().unwrap();
    
    // Test incomplete if statement
    let recovery = repl.create_error_recovery("if ", "expected expression after 'if'");
    assert!(!recovery.options.is_empty());
    
    // Test incomplete function call
    let recovery = repl.create_error_recovery("func(", "expected closing parenthesis");
    assert!(!recovery.options.is_empty());
    
    // Test incomplete array literal
    let recovery = repl.create_error_recovery("[1, 2,", "expected expression");
    assert!(!recovery.options.is_empty());
}

#[test]
fn test_recovery_option_priority() {
    let mut repl = Repl::new().unwrap();
    
    // Set up state with variables and history
    let _ = repl.eval("let similar_name = 42");
    let _ = repl.eval("100");
    
    let recovery = repl.create_error_recovery(
        "similar_nam",
        "undefined variable 'similar_nam'"
    );
    
    // Recovery options should be prioritized appropriately
    assert!(!recovery.options.is_empty());
    
    // Should include typo correction suggestions
    let has_typo_fix = recovery.options.iter().any(|opt| match opt {
        RecoveryOption::RetryWith(expr) => expr.contains("similar_name"),
        _ => false,
    });
    assert!(has_typo_fix, "Should suggest typo correction");
    
    // Should always include standard recovery options
    assert!(recovery.options.iter().any(|opt| matches!(opt, RecoveryOption::Abort)));
    assert!(recovery.options.iter().any(|opt| matches!(opt, RecoveryOption::RestoreCheckpoint)));
}

#[test]
fn test_recovery_with_let_statement_patterns() {
    let mut repl = Repl::new().unwrap();
    
    // Test incomplete let with equals
    let recovery = repl.create_error_recovery(
        "let my_var = ", 
        "expected expression after '='"
    );
    
    // Should suggest appropriate default values
    let continue_options: Vec<_> = recovery.options.iter()
        .filter_map(|opt| match opt {
            RecoveryOption::ContinueWithDefault(expr) => Some(expr),
            _ => None,
        })
        .collect();
    
    assert!(!continue_options.is_empty());
    assert!(continue_options.iter().any(|expr| expr.contains("my_var")));
    
    // Should provide retry options with common values
    let retry_options: Vec<_> = recovery.options.iter()
        .filter_map(|opt| match opt {
            RecoveryOption::RetryWith(expr) => Some(expr),
            _ => None,
        })
        .collect();
    
    assert!(!retry_options.is_empty());
}

#[test]
fn test_recovery_system_integration() {
    let mut repl = Repl::new().unwrap();
    
    // Test full recovery workflow
    let _ = repl.eval("let good_var = 123");
    
    // Create error recovery
    let recovery = repl.create_error_recovery(
        "let bad_var = ",
        "Unexpected EOF at line 1:14"
    );
    
    // Format for display
    let formatted = repl.format_error_recovery(&recovery);
    assert!(!formatted.is_empty());
    
    // Apply recovery option
    let recovery_option = recovery.options.into_iter()
        .find(|opt| matches!(opt, RecoveryOption::ContinueWithDefault(_)))
        .expect("Should have continue with default option");
    
    let result = repl.apply_recovery(recovery_option).unwrap();
    
    match result {
        RecoveryResult::Recovered(expr) => {
            // Should be able to evaluate recovered expression
            let eval_result = repl.eval(&expr);
            assert!(eval_result.is_ok(), "Recovered expression should evaluate successfully");
        }
        _ => panic!("Expected successful recovery"),
    }
    
    // Recovery context should be cleared
    assert!(repl.get_error_recovery().is_none());
}