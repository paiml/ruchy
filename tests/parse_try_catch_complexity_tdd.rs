//! TDD Tests for parse_try_catch Complexity Reduction
//! 
//! These tests drive the implementation of complexity reduction from 15 → ≤10
//! Following strict TDD: Red -> Green -> Refactor

use ruchy::frontend::parser::Parser;

#[test]
fn test_parse_basic_try_catch() {
    let code = r#"
        try {
            risky_operation()
        } catch (e) {
            handle_error(e)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse basic try-catch: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_finally() {
    let code = r#"
        try {
            risky_operation()
        } catch (e) {
            handle_error(e)
        } finally {
            cleanup()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse try-catch-finally: {:?}", result.err());
}

#[test]
fn test_parse_nested_try_catch() {
    let code = r#"
        try {
            try {
                deeply_risky()
            } catch (inner) {
                partial_recovery()
            }
        } catch (outer) {
            full_recovery()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse nested try-catch: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_typed_exception() {
    let code = r#"
        try {
            might_fail()
        } catch (e: NetworkError) {
            retry_network()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse typed catch: {:?}", result.err());
}

#[test]
fn test_parse_multiple_catch_blocks() {
    let code = r#"
        try {
            complex_operation()
        } catch (e: NetworkError) {
            handle_network()
        } catch (e: DatabaseError) {
            handle_database()
        } catch (e) {
            handle_generic()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse multiple catch blocks: {:?}", result.err());
}

#[test]
fn test_parse_try_without_catch() {
    let code = r#"
        try {
            operation()
        } finally {
            cleanup()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse try-finally without catch: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_complex_body() {
    let code = r#"
        try {
            let result = fetch_data();
            if result.is_empty() {
                throw EmptyDataError("No data available");
            }
            process_data(result);
            return result.size();
        } catch (e: EmptyDataError) {
            log_warning(e.message());
            return 0;
        } catch (e) {
            log_error("Unexpected error: " + e.toString());
            throw e;
        } finally {
            close_connections();
            cleanup_temp_files();
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse complex try-catch-finally: {:?}", result.err());
}

#[test]
fn test_parse_try_with_return_in_catch() {
    let code = r#"
        try {
            return dangerous_operation();
        } catch (e) {
            return safe_fallback();
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse try-catch with returns: {:?}", result.err());
}

#[test]
fn test_parse_empty_catch_block() {
    let code = r#"
        try {
            operation()
        } catch (e) {
            // Empty catch block
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse empty catch block: {:?}", result.err());
}

#[test]
fn test_parse_catch_without_parameter() {
    let code = r#"
        try {
            operation()
        } catch {
            default_error_handling()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse catch without parameter: {:?}", result.err());
}

#[test]
fn test_complexity_reduction_comprehensive() {
    // Test multiple variations to ensure complexity reduction works
    let test_cases = vec![
        "try { a() } catch (e) { b() }",
        "try { a() } finally { b() }",
        "try { a() } catch (e) { b() } finally { c() }",
        "try { try { a() } catch (e) { b() } } catch (e) { c() }",
    ];
    
    for (i, code) in test_cases.iter().enumerate() {
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Test case {} should parse after complexity reduction: {:?}", i, result.err());
    }
}

#[test]
fn test_error_handling_edge_cases() {
    // Test edge cases that contribute to complexity
    let edge_cases = vec![
        ("try { } catch (e) { }", "empty try block"),
        ("try { a(); b(); c(); } catch (e) { d(); e(); }", "multiple statements"),
        ("try { if (x) { y() } } catch (e) { if (z) { w() } }", "nested control flow"),
    ];
    
    for (code, description) in edge_cases {
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Edge case '{}' should parse: {:?}", description, result.err());
    }
}

#[test]
fn test_helper_function_effectiveness() {
    // This test ensures helper functions created for complexity reduction work correctly
    let complex_nested = r#"
        fun process_file(filename: String) -> Result<String, Error> {
            try {
                let file = open_file(filename);
                try {
                    let content = read_file(file);
                    try {
                        return validate_content(content);
                    } catch (e: ValidationError) {
                        return Err(format("Validation failed: {}", e));
                    }
                } catch (e: ReadError) {
                    return Err(format("Read failed: {}", e));
                }
            } catch (e: FileNotFoundError) {
                return Err(format("File not found: {}", e));
            } finally {
                cleanup_resources();
            }
        }
    "#;
    
    let mut parser = Parser::new(complex_nested);
    let result = parser.parse();
    assert!(result.is_ok(), "Complex nested try-catch should work after refactoring: {:?}", result.err());
}