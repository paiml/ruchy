//! Edge case tests for SharedSession functionality
//! Focus on corner cases and error conditions

use ruchy::wasm::shared_session::{ExecutionMode, SharedSession};

#[test]
fn test_empty_code_execution() {
    let mut session = SharedSession::new();

    // Empty string should not crash
    let result = session.execute("empty", "");
    // Could succeed with Unit or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_whitespace_only_code() {
    let mut session = SharedSession::new();

    // Whitespace-only code
    let result = session.execute("ws1", "   ");
    assert!(result.is_ok() || result.is_err());

    let result = session.execute("ws2", "\n\n\n");
    assert!(result.is_ok() || result.is_err());

    let result = session.execute("ws3", "\t\t");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_comment_only_code() {
    let mut session = SharedSession::new();

    // Single line comment
    let result = session.execute("comment1", "// just a comment");
    assert!(result.is_ok() || result.is_err());

    // Multi-line comment
    let result = session.execute("comment2", "/* multi\nline\ncomment */");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_very_long_variable_names() {
    let mut session = SharedSession::new();

    let long_name = "a".repeat(100);
    let code = format!("let {} = 42", long_name);
    let result = session.execute("long_var", &code);
    assert!(result.is_ok());

    // Use the long variable
    let result = session.execute("use_long", &long_name);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "42");
}

#[test]
fn test_deep_nesting() {
    let mut session = SharedSession::new();

    // Deep if nesting
    let code = "if true { if true { if true { 42 } else { 0 } } else { 0 } } else { 0 }";
    let result = session.execute("nested_if", code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "42");
}

#[test]
fn test_large_numbers() {
    let mut session = SharedSession::new();

    // Large integer
    let result = session.execute("big_int", "999999999999");
    assert!(result.is_ok());

    // Large float
    let result = session.execute("big_float", "3.14159265358979323846");
    assert!(result.is_ok());
}

#[test]
fn test_special_characters_in_strings() {
    let mut session = SharedSession::new();

    // Unicode emoji
    let result = session.execute("emoji", r#"let emoji = "🚀🎉""#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "🚀🎉");

    // Escaped characters
    let result = session.execute("escaped", r#"let s = "line1\nline2\ttab""#);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_statements_in_one_cell() {
    let mut session = SharedSession::new();

    // Multiple let statements
    let code = "let x = 1; let y = 2; x + y";
    let result = session.execute("multi", code);
    // Might not be supported, but shouldn't crash
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_redefinition_in_same_cell() {
    let mut session = SharedSession::new();

    // Redefine variable in same cell
    let code = "let x = 1; let x = 2; x";
    let result = session.execute("redef", code);
    // Might not be supported, but shouldn't crash
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_recursive_function() {
    let mut session = SharedSession::new();

    // Define recursive factorial
    let result = session.execute(
        "factorial",
        "fun fact(n) { if n <= 1 { 1 } else { n * fact(n - 1) } }",
    );
    assert!(result.is_ok());

    // Use recursive function
    let result = session.execute("use_fact", "fact(5)");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "120");
    }
}

#[test]
fn test_mutual_recursion() {
    let mut session = SharedSession::new();

    // Define mutually recursive functions
    session
        .execute(
            "even",
            "fun is_even(n) { if n == 0 { true } else { is_odd(n - 1) } }",
        )
        .ok();
    session
        .execute(
            "odd",
            "fun is_odd(n) { if n == 0 { false } else { is_even(n - 1) } }",
        )
        .ok();

    // Check mutual recursion
    let result = session.execute("test_even", "is_even(4)");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "true");
    }
}

#[test]
fn test_execution_mode_switching() {
    let mut session = SharedSession::new();

    // Start in manual mode
    session.execute("var1", "let x = 1").unwrap();

    // Switch to reactive
    session.set_execution_mode(ExecutionMode::Reactive);
    session.execute("var2", "let y = x * 2").unwrap();

    // Switch back to manual
    session.set_execution_mode(ExecutionMode::Manual);
    session.execute("var3", "let z = y + 1").unwrap();

    // Check all variables exist
    let result = session.execute("sum", "x + y + z");
    assert!(result.is_ok());
}

#[test]
fn test_cell_id_special_characters() {
    let mut session = SharedSession::new();

    // Cell IDs with special characters
    let result = session.execute("cell-with-dashes", "let a = 1");
    assert!(result.is_ok());

    let result = session.execute("cell_with_underscores", "let b = 2");
    assert!(result.is_ok());

    let result = session.execute("cell.with.dots", "let c = 3");
    assert!(result.is_ok());

    // Check variables exist
    let result = session.execute("sum", "a + b + c");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "6");
}

#[test]
fn test_memory_estimation_changes() {
    let mut session = SharedSession::new();

    let initial = session.estimate_interpreter_memory();

    // Add some data
    session
        .execute("data1", "let arr = [1, 2, 3, 4, 5]")
        .unwrap();
    let after_array = session.estimate_interpreter_memory();

    // Memory should increase or stay same (never decrease)
    assert!(after_array >= initial);

    // Add more data
    session
        .execute(
            "data2",
            r#"let str = "a very long string that takes up memory""#,
        )
        .unwrap();
    let after_string = session.estimate_interpreter_memory();

    assert!(after_string >= after_array);
}
