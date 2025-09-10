/// Comprehensive tests for notebook error handling system
/// Target: >80% coverage for error modules

use ruchy_notebook::error::{NotebookError, ErrorKind, ErrorSpan, ErrorSeverity, StackFrame};
use std::io;
use serde_json;

/// Test all NotebookError creation methods
#[test]
fn test_error_creation_methods() {
    // Test syntax error
    let syntax_err = NotebookError::syntax("Missing semicolon");
    assert_eq!(syntax_err.kind, ErrorKind::SyntaxError);
    assert_eq!(syntax_err.message, "Missing semicolon");
    assert_eq!(syntax_err.severity, ErrorSeverity::Error);
    
    // Test runtime error
    let runtime_err = NotebookError::runtime("Null pointer dereference");
    assert_eq!(runtime_err.kind, ErrorKind::RuntimeError);
    assert_eq!(runtime_err.message, "Null pointer dereference");
    
    // Test type error
    let type_err = NotebookError::type_error("Expected int, got string");
    assert_eq!(type_err.kind, ErrorKind::TypeError);
    assert_eq!(type_err.message, "Expected int, got string");
    
    // Test undefined error
    let undefined_err = NotebookError::undefined("my_variable");
    assert_eq!(undefined_err.kind, ErrorKind::UndefinedError);
    assert_eq!(undefined_err.message, "'my_variable' is not defined");
    
    // Test VM error
    let vm_err = NotebookError::vm_error("Stack overflow");
    assert_eq!(vm_err.kind, ErrorKind::VmError);
    assert_eq!(vm_err.message, "Stack overflow");
    
    // Test generic new method
    let generic_err = NotebookError::new(ErrorKind::ModuleError, "Import failed");
    assert_eq!(generic_err.kind, ErrorKind::ModuleError);
    assert_eq!(generic_err.message, "Import failed");
}

/// Test ErrorSpan creation and methods
#[test]
fn test_error_span_creation() {
    // Basic span
    let span = ErrorSpan::new(10, 25, 3, 5);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 25);
    assert_eq!(span.line, 3);
    assert_eq!(span.column, 5);
    assert!(span.file.is_none());
    
    // Span with file
    let span_with_file = ErrorSpan::new(0, 10, 1, 1).with_file("test.ruchy");
    assert_eq!(span_with_file.file.as_ref().unwrap(), "test.ruchy");
}

/// Test error builder pattern
#[test]
fn test_error_builder_pattern() {
    let error = NotebookError::syntax("Expected '}'")
        .with_span(ErrorSpan::new(42, 43, 5, 10).with_file("main.ruchy"))
        .with_suggestions(vec![
            "Add closing brace".to_string(),
            "Check bracket matching".to_string(),
        ])
        .with_help("Braces must be balanced in Ruchy")
        .with_severity(ErrorSeverity::Critical);
    
    assert_eq!(error.kind, ErrorKind::SyntaxError);
    assert_eq!(error.message, "Expected '}'");
    assert!(error.span.is_some());
    assert_eq!(error.suggestions.len(), 2);
    assert!(error.help.is_some());
    assert_eq!(error.severity, ErrorSeverity::Critical);
    
    let span = error.span.unwrap();
    assert_eq!(span.start, 42);
    assert_eq!(span.end, 43);
    assert_eq!(span.line, 5);
    assert_eq!(span.column, 10);
    assert_eq!(span.file.as_ref().unwrap(), "main.ruchy");
}

/// Test error kind names
#[test]
fn test_error_kind_names() {
    assert_eq!(NotebookError::syntax("test").kind_name(), "SyntaxError");
    assert_eq!(NotebookError::runtime("test").kind_name(), "RuntimeError");
    assert_eq!(NotebookError::type_error("test").kind_name(), "TypeError");
    assert_eq!(NotebookError::undefined("test").kind_name(), "UndefinedError");
    assert_eq!(NotebookError::vm_error("test").kind_name(), "VmError");
    
    let module_err = NotebookError::new(ErrorKind::ModuleError, "test");
    assert_eq!(module_err.kind_name(), "ModuleError");
    
    let memory_err = NotebookError::new(ErrorKind::MemoryError, "test");
    assert_eq!(memory_err.kind_name(), "MemoryError");
    
    let io_err = NotebookError::new(ErrorKind::IoError, "test");
    assert_eq!(io_err.kind_name(), "IoError");
    
    let conv_err = NotebookError::new(ErrorKind::ConversionError, "test");
    assert_eq!(conv_err.kind_name(), "ConversionError");
}

/// Test formatted message generation
#[test]
fn test_formatted_message() {
    // Simple error without extras
    let simple_err = NotebookError::runtime("Simple error");
    let formatted = simple_err.formatted_message();
    assert!(formatted.contains("RuntimeError: Simple error"));
    
    // Error with span
    let err_with_span = NotebookError::syntax("Missing token")
        .with_span(ErrorSpan::new(10, 15, 3, 8).with_file("test.ruchy"));
    let formatted_span = err_with_span.formatted_message();
    assert!(formatted_span.contains("SyntaxError: Missing token"));
    assert!(formatted_span.contains("at line 3, column 8"));
    assert!(formatted_span.contains("in test.ruchy"));
    
    // Error with suggestions
    let err_with_suggestions = NotebookError::undefined("my_var")
        .with_suggestions(vec![
            "Did you mean 'my_val'?".to_string(),
            "Check variable spelling".to_string(),
        ]);
    let formatted_suggestions = err_with_suggestions.formatted_message();
    assert!(formatted_suggestions.contains("Suggestions:"));
    assert!(formatted_suggestions.contains("- Did you mean 'my_val'?"));
    assert!(formatted_suggestions.contains("- Check variable spelling"));
    
    // Error with help
    let err_with_help = NotebookError::type_error("Type mismatch")
        .with_help("Use explicit type annotations");
    let formatted_help = err_with_help.formatted_message();
    assert!(formatted_help.contains("Help: Use explicit type annotations"));
    
    // Error with everything
    let complete_err = NotebookError::syntax("Expected ';'")
        .with_span(ErrorSpan::new(20, 21, 2, 15))
        .with_suggestions(vec!["Add semicolon".to_string()])
        .with_help("Statements must end with semicolons");
    let complete_formatted = complete_err.formatted_message();
    assert!(complete_formatted.contains("SyntaxError"));
    assert!(complete_formatted.contains("at line 2, column 15"));
    assert!(complete_formatted.contains("Suggestions:"));
    assert!(complete_formatted.contains("Help:"));
}

/// Test error severity levels and ordering
#[test]
fn test_error_severity() {
    assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
    assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
    assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
    
    // Test default severity
    let default_err = NotebookError::syntax("test");
    assert_eq!(default_err.severity, ErrorSeverity::Error);
    
    // Test custom severity
    let warning_err = NotebookError::syntax("Unused variable")
        .with_severity(ErrorSeverity::Warning);
    assert_eq!(warning_err.severity, ErrorSeverity::Warning);
    
    let critical_err = NotebookError::runtime("Stack overflow")
        .with_severity(ErrorSeverity::Critical);
    assert_eq!(critical_err.severity, ErrorSeverity::Critical);
}

/// Test error conversions from other error types
#[test]
fn test_error_conversions() {
    // From io::Error
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let notebook_error = NotebookError::from(io_error);
    assert_eq!(notebook_error.kind, ErrorKind::IoError);
    assert!(notebook_error.message.contains("File not found"));
    
    // From anyhow::Error
    let anyhow_error = anyhow::Error::msg("Something went wrong");
    let from_anyhow = NotebookError::from(anyhow_error);
    assert_eq!(from_anyhow.kind, ErrorKind::RuntimeError);
    assert!(from_anyhow.message.contains("Something went wrong"));
    
    // From serde_json::Error
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let from_json = NotebookError::from(json_error);
    assert_eq!(from_json.kind, ErrorKind::ConversionError);
    assert!(from_json.message.contains("JSON error"));
}

/// Test error serialization and deserialization
#[test]
fn test_error_serialization() {
    let error = NotebookError::syntax("Parse error")
        .with_span(ErrorSpan::new(0, 5, 1, 1).with_file("test.ruchy"))
        .with_suggestions(vec!["Fix syntax".to_string()])
        .with_help("Check documentation")
        .with_severity(ErrorSeverity::Error);
    
    // Test serialization
    let serialized = serde_json::to_string(&error).expect("Should serialize");
    assert!(serialized.contains("SyntaxError"));
    assert!(serialized.contains("Parse error"));
    assert!(serialized.contains("test.ruchy"));
    
    // Test deserialization
    let deserialized: NotebookError = serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(deserialized.kind, ErrorKind::SyntaxError);
    assert_eq!(deserialized.message, "Parse error");
    assert_eq!(deserialized.severity, ErrorSeverity::Error);
    assert!(deserialized.span.is_some());
    assert_eq!(deserialized.suggestions.len(), 1);
    assert!(deserialized.help.is_some());
}

/// Test ErrorSpan serialization
#[test]
fn test_error_span_serialization() {
    let span = ErrorSpan::new(10, 20, 2, 5).with_file("main.ruchy");
    
    let serialized = serde_json::to_string(&span).expect("Should serialize");
    let deserialized: ErrorSpan = serde_json::from_str(&serialized).expect("Should deserialize");
    
    assert_eq!(deserialized.start, 10);
    assert_eq!(deserialized.end, 20);
    assert_eq!(deserialized.line, 2);
    assert_eq!(deserialized.column, 5);
    assert_eq!(deserialized.file.as_ref().unwrap(), "main.ruchy");
}

/// Test error display trait
#[test]
fn test_error_display() {
    let error = NotebookError::runtime("Test error")
        .with_span(ErrorSpan::new(0, 5, 1, 1));
    
    let display_str = format!("{}", error);
    assert!(display_str.contains("RuntimeError: Test error"));
    assert!(display_str.contains("at line 1, column 1"));
}

/// Test error debug trait
#[test]
fn test_error_debug() {
    let error = NotebookError::syntax("Debug test");
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("NotebookError"));
    assert!(debug_str.contains("SyntaxError"));
    assert!(debug_str.contains("Debug test"));
}

/// Test error kind equality and patterns
#[test]
fn test_error_kind_patterns() {
    let syntax_err = NotebookError::syntax("test");
    let another_syntax = NotebookError::syntax("different message");
    let runtime_err = NotebookError::runtime("test");
    
    assert_eq!(syntax_err.kind, ErrorKind::SyntaxError);
    assert_eq!(another_syntax.kind, ErrorKind::SyntaxError);
    assert_eq!(runtime_err.kind, ErrorKind::RuntimeError);
    
    assert_eq!(syntax_err.kind, another_syntax.kind);
    assert_ne!(syntax_err.kind, runtime_err.kind);
}

/// Test error cloning
#[test]
fn test_error_cloning() {
    let original = NotebookError::type_error("Original error")
        .with_span(ErrorSpan::new(5, 10, 2, 3))
        .with_suggestions(vec!["Suggestion".to_string()]);
    
    let cloned = original.clone();
    
    assert_eq!(cloned.kind, original.kind);
    assert_eq!(cloned.message, original.message);
    assert_eq!(cloned.severity, original.severity);
    assert_eq!(cloned.suggestions, original.suggestions);
    
    // Verify span was cloned correctly
    assert!(cloned.span.is_some());
    let original_span = original.span.as_ref().unwrap();
    let cloned_span = cloned.span.as_ref().unwrap();
    assert_eq!(cloned_span.start, original_span.start);
    assert_eq!(cloned_span.line, original_span.line);
}

/// Test edge cases and boundary conditions
#[test]
fn test_error_edge_cases() {
    // Empty message
    let empty_msg_err = NotebookError::syntax("");
    assert_eq!(empty_msg_err.message, "");
    
    // Very long message
    let long_msg = "a".repeat(10000);
    let long_err = NotebookError::runtime(&long_msg);
    assert_eq!(long_err.message.len(), 10000);
    
    // Zero-position span
    let zero_span = ErrorSpan::new(0, 0, 0, 0);
    let zero_err = NotebookError::syntax("test").with_span(zero_span);
    let span = zero_err.span.unwrap();
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    assert_eq!(span.line, 0);
    assert_eq!(span.column, 0);
    
    // Large span values
    let large_span = ErrorSpan::new(usize::MAX - 1, usize::MAX, usize::MAX, usize::MAX);
    assert_eq!(large_span.start, usize::MAX - 1);
    assert_eq!(large_span.end, usize::MAX);
    
    // Empty suggestions vec
    let no_suggestions = NotebookError::undefined("var").with_suggestions(vec![]);
    assert_eq!(no_suggestions.suggestions.len(), 0);
    
    // Many suggestions
    let many_suggestions: Vec<String> = (0..100).map(|i| format!("Suggestion {}", i)).collect();
    let many_sugg_err = NotebookError::syntax("test").with_suggestions(many_suggestions);
    assert_eq!(many_sugg_err.suggestions.len(), 100);
}

/// Test all ErrorKind variants
#[test]
fn test_all_error_kinds() {
    let kinds = vec![
        ErrorKind::SyntaxError,
        ErrorKind::RuntimeError,
        ErrorKind::TypeError,
        ErrorKind::UndefinedError,
        ErrorKind::ModuleError,
        ErrorKind::MemoryError,
        ErrorKind::IoError,
        ErrorKind::ConversionError,
        ErrorKind::VmError,
    ];
    
    for kind in kinds {
        let error = NotebookError::new(kind.clone(), "Test message");
        assert_eq!(error.kind, kind);
        
        // Test that kind name is not empty
        assert!(!error.kind_name().is_empty());
    }
}

/// Test StackFrame if available
#[test]
fn test_stack_frame() {
    // This test assumes StackFrame exists based on the imports
    // If it doesn't exist, this test will fail at compile time
    let frame = StackFrame {
        function: "test_function".to_string(),
        file: "test.rs".to_string(),
        line: 42,
        column: 10,
    };
    
    assert_eq!(frame.function, "test_function");
    assert_eq!(frame.file, "test.rs");
    assert_eq!(frame.line, 42);
    assert_eq!(frame.column, 10);
}

/// Test error message with special characters
#[test]
fn test_error_with_special_characters() {
    let special_chars = "Error with émojis 🚀 and ñewlines\nand tabs\t";
    let error = NotebookError::runtime(special_chars);
    
    assert_eq!(error.message, special_chars);
    
    // Should be serializable
    let serialized = serde_json::to_string(&error).expect("Should serialize");
    let deserialized: NotebookError = serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(deserialized.message, special_chars);
}

/// Test error building with method chaining
#[test] 
fn test_error_method_chaining() {
    let error = NotebookError::new(ErrorKind::TypeError, "Base error")
        .with_severity(ErrorSeverity::Warning)
        .with_help("This is help text")
        .with_suggestions(vec!["First suggestion".to_string()])
        .with_span(ErrorSpan::new(1, 2, 3, 4));
    
    assert_eq!(error.kind, ErrorKind::TypeError);
    assert_eq!(error.severity, ErrorSeverity::Warning);
    assert!(error.help.is_some());
    assert_eq!(error.suggestions.len(), 1);
    assert!(error.span.is_some());
    
    // Verify all methods can be chained in any order
    let error2 = NotebookError::syntax("Another error")
        .with_span(ErrorSpan::new(10, 20, 5, 8))
        .with_suggestions(vec!["Suggestion 1".to_string(), "Suggestion 2".to_string()])
        .with_severity(ErrorSeverity::Critical)
        .with_help("More help");
    
    assert_eq!(error2.suggestions.len(), 2);
    assert_eq!(error2.severity, ErrorSeverity::Critical);
}