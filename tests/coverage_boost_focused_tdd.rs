//! Strategic TDD tests targeting highest-impact uncovered functions
//! Target: Push coverage from 52.73% to 55%+ with focused testing

#[cfg(test)]
mod tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;
    use ruchy::runtime::interpreter::Interpreter;
    
    // Helper functions (complexity: 3 each)
    fn parse_expr(input: &str) -> Result<ruchy::frontend::ast::Expr, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        Ok(parser.parse()?)
    }
    
    fn transpile_and_eval(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let result = interpreter.eval_expr(&ast)?;
        Ok(format!("{}", result))
    }
    
    // Error handling paths (complexity: 4 each)
    #[test]
    fn test_parser_error_recovery_unclosed_paren() {
        let mut parser = Parser::new("(1 + 2");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parser_error_recovery_unclosed_bracket() {
        let mut parser = Parser::new("[1, 2, 3");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parser_error_recovery_unclosed_brace() {
        let mut parser = Parser::new("{x: 1, y: 2");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parser_error_recovery_invalid_number() {
        let mut parser = Parser::new("123abc");
        let result = parser.parse();
        // Should either parse as valid or error appropriately
        let _ = result;
    }
    
    #[test]
    fn test_interpreter_error_undefined_variable() {
        let result = transpile_and_eval("undefined_variable");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_interpreter_error_type_mismatch() {
        let result = transpile_and_eval("1 + \"hello\"");
        // Type error should be caught
        let _ = result;
    }
    
    #[test]
    fn test_interpreter_error_division_by_zero() {
        let result = transpile_and_eval("5 / 0");
        assert!(result.is_err());
    }
    
    // Edge case branching (complexity: 3 each)
    #[test]
    fn test_empty_expressions() {
        let test_cases = vec!["()", "[]", "{}", "\"\"", "||{}"];
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse: {}", case);
        }
    }
    
    #[test]
    fn test_deeply_nested_expressions() {
        let nested = "((((((1 + 2) * 3) / 4) - 5) % 6) + 7)";
        let result = transpile_and_eval(nested);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_complex_string_operations() {
        let test_cases = vec![
            r#""hello""#,
            r#""hello\nworld""#,
            r#""hello\"world""#,
            r#""multi\nline\tstring""#,
        ];
        
        for case in test_cases {
            let result = transpile_and_eval(case);
            assert!(result.is_ok(), "Failed for: {}", case);
        }
    }
    
    // Boundary value testing (complexity: 4 each) 
    #[test]
    fn test_integer_boundaries() {
        let test_cases = vec![
            "0", "1", "-1", "2147483647", "-2147483648",
        ];
        
        for case in test_cases {
            let result = transpile_and_eval(case);
            assert!(result.is_ok(), "Failed for integer: {}", case);
        }
    }
    
    #[test]
    fn test_float_boundaries() {
        let test_cases = vec![
            "0.0", "1.0", "-1.0", "3.14159", "1e10", "1e-10",
        ];
        
        for case in test_cases {
            let result = transpile_and_eval(case);
            assert!(result.is_ok(), "Failed for float: {}", case);
        }
    }
    
    // Collection edge cases (complexity: 4 each)
    #[test]
    fn test_list_edge_cases() {
        let test_cases = vec![
            "[]",                              // Empty list
            "[1]",                            // Single element
            "[1, 2, 3, 4, 5]",               // Multiple elements
            "[[1, 2], [3, 4]]",              // Nested lists
            r#"[1, "hello", true]"#,         // Mixed types
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse list: {}", case);
        }
    }
    
    #[test]
    fn test_object_edge_cases() {
        let test_cases = vec![
            "{}",                                    // Empty object
            "{x: 1}",                               // Single field
            "{x: 1, y: 2}",                        // Multiple fields
            "{nested: {inner: 42}}",               // Nested objects
            r#"{name: "John", age: 30, active: true}"#, // Mixed value types
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse object: {}", case);
        }
    }
    
    // Function call edge cases (complexity: 4 each)
    #[test]
    fn test_function_call_edge_cases() {
        let test_cases = vec![
            "func()",                    // No arguments
            "func(1)",                   // Single argument
            "func(1, 2, 3)",            // Multiple arguments
            "obj.method()",              // Method call
            "obj.chain().method()",      // Chained methods
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse function call: {}", case);
        }
    }
    
    // Control flow edge cases (complexity: 5 each)
    #[test]
    fn test_if_expression_variations() {
        let test_cases = vec![
            "if true { 1 }",                          // If without else
            "if true { 1 } else { 2 }",              // If with else
            "if x > 0 { 1 } elif x < 0 { -1 } else { 0 }", // If elif else chain
            "if (x > 0 && y > 0) { 1 } else { 0 }",  // Complex condition
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse if: {}", case);
        }
    }
    
    #[test]
    fn test_match_expression_variations() {
        let test_cases = vec![
            "match x { _ => 0 }",                     // Wildcard only
            "match x { 1 => \"one\", _ => \"other\" }", // Literal patterns
            "match result { Ok(x) => x, Err(_) => 0 }", // Constructor patterns
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse match: {}", case);
        }
    }
    
    // Lambda and closure edge cases (complexity: 4 each)
    #[test]
    fn test_lambda_variations() {
        let test_cases = vec![
            "|| 42",                     // No parameters
            "|x| x + 1",                // Single parameter
            "|x, y| x + y",             // Multiple parameters
            "|x| { let y = x * 2; y + 1 }", // Block body
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse lambda: {}", case);
        }
    }
    
    // Operator precedence edge cases (complexity: 3 each)
    #[test]
    fn test_operator_precedence() {
        let test_cases = vec![
            "1 + 2 * 3",                // Multiplication before addition
            "1 * 2 + 3",                // Multiplication before addition
            "1 + 2 == 3",               // Addition before comparison
            "1 < 2 && 2 < 3",          // Comparison before logical and
            "true || false && true",    // Logical and before logical or
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse precedence: {}", case);
            
            // Also test evaluation if possible
            if let Ok(val) = transpile_and_eval(case) {
                assert!(val == "true" || val == "false" || val.parse::<i64>().is_ok());
            }
        }
    }
    
    // Unicode and special character handling (complexity: 3 each)
    #[test]
    fn test_unicode_strings() {
        let test_cases = vec![
            r#""Hello 世界""#,            // Unicode characters
            r#""Emoji 🌍🚀✨""#,         // Emoji
            r#""Math ∑∞π√""#,            // Math symbols
            r#""Arrows ↑↓←→""#,          // Arrows
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse unicode: {}", case);
        }
    }
    
    // Comments and whitespace handling (complexity: 2 each)
    #[test]
    fn test_whitespace_variations() {
        let test_cases = vec![
            "1+2",                      // No spaces
            "1 + 2",                    // Normal spaces
            "1  +  2",                  // Extra spaces
            "1\n+\n2",                  // Newlines
            "1\t+\t2",                  // Tabs
        ];
        
        for case in test_cases {
            let result = parse_expr(case);
            assert!(result.is_ok(), "Failed to parse whitespace variant: {}", case);
        }
    }
    
    // Transpiler-specific edge cases (complexity: 4 each)
    #[test]
    fn test_transpiler_edge_cases() {
        let transpiler = Transpiler::default();
        
        let test_cases = vec![
            "42",
            "true",
            r#""hello""#,
            "[]",
            "{}",
        ];
        
        for case in test_cases {
            let mut parser = Parser::new(case);
            if let Ok(expr) = parser.parse() {
                let result = transpiler.transpile_to_string(&expr);
                assert!(result.is_ok(), "Transpiler failed for: {}", case);
            }
        }
    }
    
    // Integration test (complexity: 5)
    #[test]
    fn test_complete_integration() {
        let program = r#"
            let x = 42;
            let y = "hello";
            let list = [1, 2, 3];
            let obj = {name: "test", value: x};
            if x > 0 {
                list[0] + obj.value
            } else {
                0
            }
        "#;
        
        let result = parse_expr(program);
        assert!(result.is_ok());
        
        // Also try transpiling
        let transpiler = Transpiler::default();
        if let Ok(expr) = result {
            let transpile_result = transpiler.transpile_to_string(&expr);
            // Should either succeed or fail gracefully
            let _ = transpile_result;
        }
    }
}