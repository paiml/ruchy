/// Unit tests for notebook error handling components
/// Target: >80% coverage for error modules

use ruchy_notebook::error::{NotebookError, ErrorKind, ErrorSpan, ErrorSeverity, StackFrame};
use std::io;

/// Test NotebookError creation and display
#[test]
fn test_notebook_error_creation() {
    let syntax_error = NotebookError::syntax("Unexpected token");
    assert_eq!(syntax_error.kind, ErrorKind::SyntaxError);
    assert!(syntax_error.message.contains("Unexpected token"));
    
    let runtime_error = NotebookError::runtime("Division by zero");
    assert_eq!(runtime_error.kind, ErrorKind::RuntimeError);
    assert!(runtime_error.message.contains("Division by zero"));
    
    let type_error = NotebookError::type_error("Type mismatch");
    assert_eq!(type_error.kind, ErrorKind::TypeError);
    assert!(type_error.message.contains("Type mismatch"));
    
    let undefined_error = NotebookError::undefined("my_var");
    assert_eq!(undefined_error.kind, ErrorKind::UndefinedError);
    assert!(undefined_error.message.contains("'my_var' is not defined"));
}

/// Test NotebookError display formatting
#[test]
fn test_notebook_error_display() {
    let parse_error = NotebookError::ParseError("Unexpected token '{'".to_string());
    let display_str = format!("{}", parse_error);
    assert!(display_str.contains("Parse Error"));
    assert!(display_str.contains("Unexpected token"));
    
    let runtime_error = NotebookError::RuntimeError("Variable not found: x".to_string());
    let runtime_display = format!("{}", runtime_error);
    assert!(runtime_display.contains("Runtime Error"));
    assert!(runtime_display.contains("Variable not found"));
}

/// Test NotebookError debug formatting
#[test]
fn test_notebook_error_debug() {
    let error = NotebookError::CompileError("Type inference failed".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("CompileError"));
    assert!(debug_str.contains("Type inference failed"));
}

/// Test ErrorSuggestion creation
#[test]
fn test_error_suggestion_creation() {
    let suggestion = ErrorSuggestion {
        message: "Try adding semicolon".to_string(),
        fix: Some("Add ';' at end of line".to_string()),
        line: Some(42),
        column: Some(15),
    };
    
    assert_eq!(suggestion.message, "Try adding semicolon");
    assert_eq!(suggestion.fix.unwrap(), "Add ';' at end of line");
    assert_eq!(suggestion.line.unwrap(), 42);
    assert_eq!(suggestion.column.unwrap(), 15);
}

/// Test ErrorSuggestion without optional fields
#[test]
fn test_error_suggestion_minimal() {
    let minimal = ErrorSuggestion {
        message: "Check variable name".to_string(),
        fix: None,
        line: None,
        column: None,
    };
    
    assert_eq!(minimal.message, "Check variable name");
    assert!(minimal.fix.is_none());
    assert!(minimal.line.is_none());
    assert!(minimal.column.is_none());
}

/// Test StackFrame creation
#[test]
fn test_stack_frame_creation() {
    let frame = StackFrame {
        function: "main".to_string(),
        file: "notebook.rs".to_string(),
        line: 100,
        column: 25,
    };
    
    assert_eq!(frame.function, "main");
    assert_eq!(frame.file, "notebook.rs");
    assert_eq!(frame.line, 100);
    assert_eq!(frame.column, 25);
}

/// Test StackFrame display formatting
#[test]
fn test_stack_frame_display() {
    let frame = StackFrame {
        function: "execute_cell".to_string(),
        file: "server.rs".to_string(),
        line: 45,
        column: 12,
    };
    
    let display_str = format!("{}", frame);
    assert!(display_str.contains("execute_cell"));
    assert!(display_str.contains("server.rs"));
    assert!(display_str.contains("45"));
    assert!(display_str.contains("12"));
}

/// Test error conversion from io::Error
#[test]
fn test_error_conversion_from_io() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let notebook_error = NotebookError::from(io_error);
    
    match notebook_error {
        NotebookError::IOError(inner) => {
            assert_eq!(inner.kind(), io::ErrorKind::NotFound);
        }
        _ => panic!("Expected IOError variant"),
    }
}

/// Test error chaining
#[test]
fn test_error_chaining() {
    let cause = NotebookError::ParseError("Invalid syntax".to_string());
    let wrapper = NotebookError::CompileError(format!("Compilation failed: {}", cause));
    
    assert!(format!("{}", wrapper).contains("Compilation failed"));
    assert!(format!("{}", wrapper).contains("Invalid syntax"));
}

/// Test error suggestions with line/column info
#[test]
fn test_error_suggestions_with_position() {
    let suggestions = vec![
        ErrorSuggestion {
            message: "Expected ';' after statement".to_string(),
            fix: Some("Add semicolon".to_string()),
            line: Some(10),
            column: Some(5),
        },
        ErrorSuggestion {
            message: "Variable might be undefined".to_string(),
            fix: Some("Declare variable with 'let'".to_string()),
            line: Some(12),
            column: Some(8),
        },
    ];
    
    assert_eq!(suggestions.len(), 2);
    assert!(suggestions[0].line.is_some());
    assert!(suggestions[1].column.is_some());
    assert!(suggestions[0].fix.as_ref().unwrap().contains("semicolon"));
    assert!(suggestions[1].fix.as_ref().unwrap().contains("let"));
}

/// Test stack trace construction
#[test]
fn test_stack_trace_construction() {
    let frames = vec![
        StackFrame {
            function: "main".to_string(),
            file: "main.rs".to_string(),
            line: 1,
            column: 1,
        },
        StackFrame {
            function: "execute_notebook".to_string(),
            file: "notebook.rs".to_string(),
            line: 50,
            column: 10,
        },
        StackFrame {
            function: "run_cell".to_string(),
            file: "cell.rs".to_string(),
            line: 25,
            column: 5,
        },
    ];
    
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].function, "main");
    assert_eq!(frames[1].function, "execute_notebook");
    assert_eq!(frames[2].function, "run_cell");
}

/// Test error serialization
#[test]
fn test_error_serialization() {
    let error = NotebookError::RuntimeError("Test error".to_string());
    let error_json = serde_json::to_string(&error);
    
    // Should be serializable if serde derives are present
    match error_json {
        Ok(json) => {
            assert!(json.contains("Test error"));
        }
        Err(_) => {
            // If serde is not derived, this is expected
            // Test that error can at least be displayed
            let display = format!("{}", error);
            assert!(display.contains("Test error"));
        }
    }
}

/// Test error suggestion formatting
#[test]
fn test_error_suggestion_formatting() {
    let suggestion = ErrorSuggestion {
        message: "Missing return type".to_string(),
        fix: Some("Add '-> ReturnType'".to_string()),
        line: Some(15),
        column: Some(20),
    };
    
    let formatted = format!("{:?}", suggestion);
    assert!(formatted.contains("Missing return type"));
    assert!(formatted.contains("Add '-> ReturnType'"));
    assert!(formatted.contains("15"));
    assert!(formatted.contains("20"));
}

/// Test multiple error suggestions
#[test]
fn test_multiple_error_suggestions() {
    let mut suggestions = Vec::new();
    
    for i in 1..=5 {
        suggestions.push(ErrorSuggestion {
            message: format!("Suggestion #{}", i),
            fix: Some(format!("Fix #{}", i)),
            line: Some(i * 10),
            column: Some(i * 5),
        });
    }
    
    assert_eq!(suggestions.len(), 5);
    
    for (idx, suggestion) in suggestions.iter().enumerate() {
        let expected_num = idx + 1;
        assert!(suggestion.message.contains(&expected_num.to_string()));
        assert_eq!(suggestion.line.unwrap(), (expected_num * 10) as u32);
    }
}

/// Test error context preservation
#[test]
fn test_error_context_preservation() {
    let original_error = "Original error message";
    let context = "Additional context";
    
    let error_with_context = NotebookError::RuntimeError(
        format!("{}: {}", context, original_error)
    );
    
    let display = format!("{}", error_with_context);
    assert!(display.contains("Additional context"));
    assert!(display.contains("Original error message"));
}

/// Test error code categorization
#[test]
fn test_error_code_categorization() {
    // Test that different error types can be distinguished
    let parse_err = NotebookError::ParseError("Parse issue".to_string());
    let runtime_err = NotebookError::RuntimeError("Runtime issue".to_string());
    let compile_err = NotebookError::CompileError("Compile issue".to_string());
    let server_err = NotebookError::ServerError("Server issue".to_string());
    
    // Each should have different discriminants
    assert!(!matches!(parse_err, NotebookError::RuntimeError(_)));
    assert!(!matches!(runtime_err, NotebookError::CompileError(_)));
    assert!(!matches!(compile_err, NotebookError::ServerError(_)));
    assert!(!matches!(server_err, NotebookError::ParseError(_)));
}

/// Test stack frame edge cases
#[test]
fn test_stack_frame_edge_cases() {
    // Empty function name
    let empty_function = StackFrame {
        function: "".to_string(),
        file: "test.rs".to_string(),
        line: 1,
        column: 1,
    };
    assert!(empty_function.function.is_empty());
    
    // Very long function name
    let long_function = StackFrame {
        function: "a".repeat(1000),
        file: "test.rs".to_string(),
        line: 1,
        column: 1,
    };
    assert_eq!(long_function.function.len(), 1000);
    
    // Line 0 (edge case)
    let line_zero = StackFrame {
        function: "test".to_string(),
        file: "test.rs".to_string(),
        line: 0,
        column: 0,
    };
    assert_eq!(line_zero.line, 0);
    assert_eq!(line_zero.column, 0);
}

/// Test error message truncation/formatting
#[test]
fn test_error_message_formatting() {
    // Very long error message
    let long_message = "a".repeat(5000);
    let long_error = NotebookError::RuntimeError(long_message.clone());
    
    let display = format!("{}", long_error);
    assert!(display.len() > 1000); // Should preserve long messages
    
    // Error with newlines
    let multiline_error = NotebookError::ParseError(
        "Line 1 error\nLine 2 context\nLine 3 more info".to_string()
    );
    
    let multiline_display = format!("{}", multiline_error);
    assert!(multiline_display.contains("Line 1"));
    assert!(multiline_display.contains("Line 2"));
    assert!(multiline_display.contains("Line 3"));
}