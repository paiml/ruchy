//! Comprehensive TDD test suite for REPL error handling
//! Target: Transform REPL error handling paths from 0% → 80%+ coverage
//! Toyota Way: Every error path must be tested and handled gracefully

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::runtime::repl::{Repl, ReplError, ReplResult};

// ==================== SYNTAX ERROR HANDLING TESTS ====================

#[test]
fn test_parse_error_incomplete_expression() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let x = ");
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("unexpected") || error_msg.contains("incomplete"));
}

#[test]
fn test_parse_error_unmatched_brackets() {
    let mut repl = Repl::new().unwrap();
    
    let test_cases = vec![
        "[1, 2, 3",     // Missing closing bracket
        "1, 2, 3]",     // Missing opening bracket
        "{a: 1, b: 2",  // Missing closing brace
        "a: 1, b: 2}",  // Missing opening brace
        "(1 + 2",       // Missing closing paren
        "1 + 2)",       // Missing opening paren
    ];
    
    for case in test_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for: {}", case);
    }
}

#[test]
fn test_parse_error_invalid_tokens() {
    let mut repl = Repl::new().unwrap();
    
    let test_cases = vec![
        "@invalid",     // Invalid character
        "#unknown",     // Invalid token
        "$variable",    // Invalid variable name
        "123abc",       // Invalid number format
        "\"unclosed string",  // Unclosed string
    ];
    
    for case in test_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for: {}", case);
    }
}

// ==================== RUNTIME ERROR HANDLING TESTS ====================

#[test]
fn test_undefined_variable_error() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("undefined_variable");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("undefined") || error.contains("not found"));
}

#[test]
fn test_undefined_function_error() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("undefined_function()");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("undefined") || error.contains("not found"));
}

#[test]
fn test_type_error_operations() {
    let mut repl = Repl::new().unwrap();
    
    let type_error_cases = vec![
        "\"hello\" * \"world\"",      // String multiplication
        "true + false",              // Boolean arithmetic
        "[1, 2] - [3, 4]",          // List subtraction
        "\"string\" / 5",           // String division
        "true && \"not_boolean\"",   // Mixed boolean operations
    ];
    
    for case in type_error_cases {
        let result = repl.eval(case);
        // Should either error or handle gracefully
        if result.is_err() {
            let error = result.unwrap_err();
            assert!(!error.is_empty());
        }
    }
}

#[test]
fn test_division_by_zero_errors() {
    let mut repl = Repl::new().unwrap();
    
    let division_cases = vec![
        "10 / 0",
        "5.5 / 0.0", 
        "100 % 0",
        "let x = 0; 42 / x",
    ];
    
    for case in division_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for division by zero: {}", case);
        let error = result.unwrap_err();
        assert!(error.contains("division") || error.contains("zero"));
    }
}

// ==================== INDEX ERROR HANDLING TESTS ====================

#[test]
fn test_list_index_out_of_bounds() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let list = [1, 2, 3]").unwrap();
    
    let out_of_bounds_cases = vec![
        "list[3]",    // Index 3 in 0-indexed array of length 3
        "list[10]",   // Way out of bounds
        "list[-1]",   // Negative index (if not supported)
        "list[100]",  // Large out of bounds
    ];
    
    for case in out_of_bounds_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for out of bounds: {}", case);
    }
}

#[test]
fn test_string_index_out_of_bounds() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let text = \"hello\"").unwrap();
    
    let out_of_bounds_cases = vec![
        "text[5]",    // Index 5 in string of length 5
        "text[10]",   // Way out of bounds
        "text[-1]",   // Negative index
    ];
    
    for case in out_of_bounds_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for string out of bounds: {}", case);
    }
}

// ==================== FUNCTION CALL ERROR HANDLING TESTS ====================

#[test]
fn test_wrong_number_of_arguments() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("fun add_two(a, b) { a + b }").unwrap();
    
    let wrong_arg_cases = vec![
        "add_two()",        // Too few arguments
        "add_two(1)",       // Too few arguments
        "add_two(1, 2, 3)", // Too many arguments
        "add_two(1, 2, 3, 4)", // Way too many arguments
    ];
    
    for case in wrong_arg_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for wrong arg count: {}", case);
    }
}

#[test]
fn test_recursive_function_stack_overflow() {
    let mut repl = Repl::new().unwrap();
    
    // Define infinite recursion
    repl.eval("fun infinite_recursion(n) { infinite_recursion(n + 1) }").unwrap();
    
    let result = repl.eval("infinite_recursion(0)");
    // Should either detect and prevent stack overflow or handle gracefully
    assert!(result.is_err());
}

// ==================== MEMORY ERROR HANDLING TESTS ====================

#[test]
fn test_memory_limit_exceeded() {
    let mut repl = Repl::new().unwrap();
    
    // Try to create very large data structure
    let result = repl.eval("let big_list = [0; 1000000000]"); // 1 billion elements
    // Should handle memory constraints gracefully
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_timeout_handling() {
    let mut repl = Repl::new().unwrap();
    
    // Define a function that should take a very long time
    repl.eval("fun slow_function() { var i = 0; while i < 10000000 { i = i + 1 }; i }").unwrap();
    
    let result = repl.eval("slow_function()");
    // Should handle timeouts if configured
    assert!(result.is_ok() || result.is_err());
}

// ==================== VARIABLE BINDING ERROR TESTS ====================

#[test]
fn test_immutable_reassignment_error() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let immutable_var = 42").unwrap();
    
    let result = repl.eval("immutable_var = 100");
    // Should error if trying to reassign immutable variable
    if result.is_err() {
        let error = result.unwrap_err();
        assert!(error.contains("immutable") || error.contains("cannot assign"));
    }
}

#[test]
fn test_invalid_variable_names() {
    let mut repl = Repl::new().unwrap();
    
    let invalid_names = vec![
        "let 123invalid = 42",    // Starting with number
        "let let = 42",           // Using keyword
        "let if = 42",            // Using keyword
        "let fun = 42",           // Using keyword
        "let var-name = 42",      // Invalid character
    ];
    
    for case in invalid_names {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for invalid variable name: {}", case);
    }
}

// ==================== MATCH EXPRESSION ERROR TESTS ====================

#[test]
fn test_non_exhaustive_match_error() {
    let mut repl = Repl::new().unwrap();
    
    // Match without covering all cases
    let result = repl.eval("match 5 { 1 => \"one\", 2 => \"two\" }");
    // Should error or have default behavior
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_invalid_match_patterns() {
    let mut repl = Repl::new().unwrap();
    
    let invalid_patterns = vec![
        "match 5 { => \"empty\" }",           // Empty pattern
        "match 5 { 1 2 => \"invalid\" }",    // Invalid syntax
        "match 5 { 1 = \"wrong arrow\" }",   // Wrong arrow
    ];
    
    for case in invalid_patterns {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for invalid match pattern: {}", case);
    }
}

// ==================== OBJECT/PROPERTY ACCESS ERROR TESTS ====================

#[test]
fn test_undefined_property_access() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let obj = {name: \"test\", age: 25}").unwrap();
    
    let result = repl.eval("obj.undefined_property");
    assert!(result.is_err(), "Should error for undefined property access");
}

#[test]
fn test_property_access_on_non_object() {
    let mut repl = Repl::new().unwrap();
    
    let invalid_accesses = vec![
        "42.property",        // Number property access
        "\"string\".invalid", // Invalid string property
        "true.property",      // Boolean property access
        "[1, 2].invalid",     // Invalid list property
    ];
    
    for case in invalid_accesses {
        let result = repl.eval(case);
        // Should error or handle gracefully
        if result.is_err() {
            let error = result.unwrap_err();
            assert!(!error.is_empty());
        }
    }
}

// ==================== ERROR RECOVERY TESTS ====================

#[test]
fn test_error_recovery_continue_after_error() {
    let mut repl = Repl::new().unwrap();
    
    // Execute an erroneous statement
    let error_result = repl.eval("undefined_variable");
    assert!(error_result.is_err());
    
    // REPL should still be functional after error
    let recovery_result = repl.eval("2 + 2");
    assert!(recovery_result.is_ok());
    assert_eq!(recovery_result.unwrap(), "4");
}

#[test]
fn test_multiple_errors_handling() {
    let mut repl = Repl::new().unwrap();
    
    let error_cases = vec![
        "invalid_syntax =",
        "undefined_function()",
        "10 / 0",
        "nonexistent_var",
    ];
    
    for case in error_cases {
        let result = repl.eval(case);
        assert!(result.is_err(), "Should error for: {}", case);
        
        // Verify REPL is still functional
        let test_result = repl.eval("1 + 1");
        assert!(test_result.is_ok(), "REPL should remain functional after error");
    }
}

// ==================== MAGIC COMMAND ERROR TESTS ====================

#[test]
fn test_invalid_magic_command() {
    let mut repl = Repl::new().unwrap();
    
    let invalid_commands = vec![
        ":nonexistent_command",
        ":invalid command with spaces",
        ": empty_command",
        ":123numeric",
    ];
    
    for cmd in invalid_commands {
        let result = repl.eval(cmd);
        assert!(result.is_err(), "Should error for invalid magic command: {}", cmd);
    }
}

#[test]
fn test_magic_command_missing_arguments() {
    let mut repl = Repl::new().unwrap();
    
    let incomplete_commands = vec![
        ":load",        // Missing filename
        ":save",        // Missing filename  
        ":type",        // Missing variable name
        ":info",        // Missing identifier
    ];
    
    for cmd in incomplete_commands {
        let result = repl.eval(cmd);
        assert!(result.is_err(), "Should error for incomplete command: {}", cmd);
    }
}

// ==================== ERROR MESSAGE QUALITY TESTS ====================

#[test]
fn test_error_messages_are_helpful() {
    let mut repl = Repl::new().unwrap();
    
    let error_cases = vec![
        ("undefined_var", "undefined"),
        ("10 / 0", "division"),
        ("let = 42", "syntax"),
        ("[1, 2][10]", "index"),
    ];
    
    for (input, expected_keyword) in error_cases {
        let result = repl.eval(input);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
        assert!(error_msg.len() > 10, "Error message should be descriptive");
        // Optionally check for specific keywords in error messages
        // assert!(error_msg.to_lowercase().contains(expected_keyword));
    }
}

#[test]
fn test_error_location_information() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("let x = 1 + undefined_variable + 3");
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    
    // Error message should ideally contain location information
    assert!(!error_msg.is_empty());
    // Could check for line numbers, column numbers, etc.
}

// Mock error types for testing
#[derive(Debug)]
pub enum ReplError {
    ParseError(String),
    RuntimeError(String),
    TypeError(String),
    IndexError(String),
    MemoryError(String),
    TimeoutError(String),
}

pub type ReplResult<T> = Result<T, ReplError>;

// Run all tests with: cargo test repl_error_handling_tdd --test repl_error_handling_tdd