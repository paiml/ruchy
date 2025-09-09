use ruchy_notebook::{
    NotebookError, SuggestionEngine, StackTrace, StateManager,
    error::{ErrorSpan, StackFrame, stack_trace::SourceLocation},
    state::global::GlobalValue,
};

#[test]
fn test_helpful_error_messages() {
    let mut engine = SuggestionEngine::new();
    engine.add_variable("username".to_string());
    engine.add_variable("user_email".to_string());
    engine.add_function("get_user".to_string());
    
    // Test undefined variable with suggestions
    let suggestions = engine.suggest_for_undefined("usename"); // typo
    assert!(!suggestions.is_empty());
    assert!(suggestions[0].contains("username"));
    
    // Test syntax error suggestions
    let syntax_suggestions = engine.suggest_for_syntax_error("expected ';'");
    assert!(syntax_suggestions.iter().any(|s| s.contains("semicolon")));
    
    // Test type error suggestions
    let type_suggestions = engine.suggest_for_type_error("int", "string");
    assert!(type_suggestions.iter().any(|s| s.contains("to_int")));
}

#[test]
fn test_error_formatting_with_context() {
    let error = NotebookError::undefined("my_variable")
        .with_span(
            ErrorSpan::new(15, 26, 3, 10)
                .with_file("notebook_cell_1.ruchy")
        )
        .with_suggestions(vec![
            "Did you mean 'my_value'?".to_string(),
            "Check variable spelling".to_string(),
            "Ensure variable is declared before use".to_string(),
        ])
        .with_help("Variables must be declared with 'let' or 'mut' before use");
    
    let formatted = error.formatted_message();
    
    // Check all components are present
    assert!(formatted.contains("UndefinedError"));
    assert!(formatted.contains("'my_variable' is not defined"));
    assert!(formatted.contains("line 3, column 10"));
    assert!(formatted.contains("notebook_cell_1.ruchy"));
    assert!(formatted.contains("Did you mean 'my_value'?"));
    assert!(formatted.contains("Help: Variables must be declared"));
    
    println!("Formatted error:\n{}", formatted);
}

#[test]
fn test_stack_trace_with_source() {
    let mut trace = StackTrace::new();
    
    // Simulate a call stack: main -> calculate -> divide
    let main_frame = StackFrame::new(0)
        .with_function("main")
        .with_location(
            SourceLocation::new(10, 5)
                .with_file("main.ruchy")
                .with_source_line("let result = calculate(a, b)")
        );
    
    let calculate_frame = StackFrame::new(25)
        .with_function("calculate")
        .with_location(
            SourceLocation::new(5, 20)
                .with_file("math.ruchy")
                .with_source_line("    return divide(x, y)")
        );
    
    let mut divide_frame = StackFrame::new(40)
        .with_function("divide")
        .with_location(
            SourceLocation::new(2, 15)
                .with_file("math.ruchy")
                .with_source_line("    return a / b  // Error: division by zero")
        );
    
    divide_frame.add_local("a", "10");
    divide_frame.add_local("b", "0");
    
    trace.push_frame(main_frame);
    trace.push_frame(calculate_frame);
    trace.push_frame(divide_frame);
    
    let formatted = trace.format();
    
    assert!(formatted.contains("Stack trace:"));
    assert!(formatted.contains("function 'divide'"));
    assert!(formatted.contains("function 'calculate'"));
    assert!(formatted.contains("function 'main'"));
    assert!(formatted.contains("return a / b"));
    assert!(formatted.contains("line 2"));
    assert!(formatted.contains("line 5"));
    assert!(formatted.contains("line 10"));
    
    println!("Stack trace:\n{}", formatted);
}

#[test]
fn test_global_state_user_experience() {
    let state_manager = StateManager::new();
    
    // Test normal global declaration
    let result = state_manager.declare_global(
        "APP_NAME".to_string(),
        GlobalValue::String("My Notebook App".to_string()),
        false,
        Some("cell_1".to_string()),
    );
    assert!(result.is_ok());
    
    // Test duplicate declaration error
    let duplicate_result = state_manager.declare_global(
        "APP_NAME".to_string(),
        GlobalValue::String("Different App".to_string()),
        false,
        Some("cell_2".to_string()),
    );
    assert!(duplicate_result.is_err());
    assert!(duplicate_result.unwrap_err().contains("already exists"));
    
    // Test mutable global
    state_manager.declare_global(
        "counter".to_string(),
        GlobalValue::Int(0),
        true,
        Some("cell_1".to_string()),
    ).unwrap();
    
    let update_result = state_manager.update_global("counter", GlobalValue::Int(5));
    assert!(update_result.is_ok());
    
    // Test immutable update error
    let immutable_update = state_manager.update_global(
        "APP_NAME", 
        GlobalValue::String("New Name".to_string())
    );
    assert!(immutable_update.is_err());
    assert!(immutable_update.unwrap_err().contains("immutable"));
}

#[test]
fn test_comprehensive_error_scenario() {
    // Simulate a realistic error scenario
    let mut engine = SuggestionEngine::new();
    
    // Add some context variables/functions
    engine.add_variable("data".to_string());
    engine.add_variable("results".to_string());
    engine.add_function("process_data".to_string());
    engine.add_function("calculate_mean".to_string());
    
    // User tries to access undefined variable
    let error = NotebookError::undefined("reults") // typo in "results"
        .with_span(
            ErrorSpan::new(42, 49, 5, 15)
                .with_file("analysis.ruchy")
        )
        .with_suggestions(engine.suggest_for_undefined("reults"));
    
    let formatted = error.formatted_message();
    
    // Should suggest the correct variable name
    assert!(formatted.contains("results"));
    assert!(formatted.contains("line 5"));
    
    // Test with stack trace for runtime error
    let mut trace = StackTrace::new();
    
    let frame = StackFrame::new(100)
        .with_function("process_data")
        .with_location(
            SourceLocation::new(12, 8)
                .with_file("analysis.ruchy")
                .with_source_line("    let avg = calculate_mean(empty_list)")
        );
    
    trace.push_frame(frame);
    
    let runtime_error = NotebookError::runtime("Cannot calculate mean of empty list")
        .with_span(ErrorSpan::new(0, 20, 12, 8))
        .with_suggestions(vec![
            "Check if the list is not empty before calculating mean".to_string(),
            "Use a default value for empty lists".to_string(),
        ])
        .with_help("Mathematical operations on empty collections are undefined");
    
    // Combine error with stack trace would happen in VM
    println!("Runtime error with context:\n{}", runtime_error.formatted_message());
    println!("\n{}", trace.format());
}

#[test]
fn test_educational_error_messages() {
    // Test beginner-friendly error messages
    let type_error = NotebookError::type_error("Cannot add string and number")
        .with_suggestions(vec![
            "Convert the number to string: value.to_string()".to_string(),
            "Convert the string to number: value.to_int()".to_string(),
            "Use string interpolation: f\"{text} {number}\"".to_string(),
        ])
        .with_help("Ruchy requires explicit type conversion. Unlike some languages, it won't automatically convert types.");
    
    let formatted = type_error.formatted_message();
    assert!(formatted.contains("explicit type conversion"));
    assert!(formatted.contains("to_string()"));
    assert!(formatted.contains("to_int()"));
    
    // Test syntax error with helpful context
    let syntax_error = NotebookError::syntax("Unexpected token ']'")
        .with_span(ErrorSpan::new(20, 21, 3, 15))
        .with_suggestions(vec![
            "Check for missing opening bracket '['".to_string(),
            "Verify array syntax: [element1, element2]".to_string(),
        ])
        .with_help("Arrays in Ruchy use square brackets: [1, 2, 3]");
    
    let syntax_formatted = syntax_error.formatted_message();
    assert!(syntax_formatted.contains("square brackets"));
    assert!(syntax_formatted.contains("missing opening bracket"));
}

#[test]
fn test_error_severity_levels() {
    let warning = NotebookError::syntax("Unused variable 'temp'")
        .with_severity(ruchy_notebook::error::ErrorSeverity::Warning);
    
    let error = NotebookError::runtime("Division by zero");
    
    let critical = NotebookError::vm_error("Stack overflow")
        .with_severity(ruchy_notebook::error::ErrorSeverity::Critical);
    
    // Test that severity affects presentation (in a real implementation)
    assert_eq!(warning.severity, ruchy_notebook::error::ErrorSeverity::Warning);
    assert_eq!(error.severity, ruchy_notebook::error::ErrorSeverity::Error);
    assert_eq!(critical.severity, ruchy_notebook::error::ErrorSeverity::Critical);
    
    assert!(critical.severity > error.severity);
    assert!(error.severity > warning.severity);
}

#[test]
fn test_suggestion_engine_learning() {
    let mut engine = SuggestionEngine::new();
    
    // Add user-defined symbols
    engine.add_variable("user_data".to_string());
    engine.add_variable("processed_data".to_string());
    engine.add_function("preprocess".to_string());
    engine.add_function("analyze".to_string());
    
    // Test suggestions for partial matches
    let suggestions = engine.suggest_for_undefined("proces"); // partial match
    assert!(!suggestions.is_empty());
    // Should suggest functions starting with "proces"
    
    let suggestions = engine.suggest_for_undefined("data_user"); // word order wrong
    assert!(!suggestions.is_empty());
    
    // Test that built-in functions are also suggested
    let println_suggestions = engine.suggest_for_undefined("printl");
    assert!(println_suggestions.iter().any(|s| s.contains("println")));
}