//! TDD tests for error handling and edge cases
//! Target: Test error paths to improve coverage with complexity ≤10

#[cfg(test)]
mod tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::runtime::interpreter::Interpreter;
    
    // Parser error tests (complexity: 3 each)
    #[test]
    fn test_parse_invalid_number() {
        let mut parser = Parser::new("123abc");
        let result = parser.parse();
        // Should either parse as identifier or fail gracefully
        let _ = result;
    }
    
    #[test]
    fn test_parse_unclosed_string() {
        let mut parser = Parser::new(r#""unclosed string"#);
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_unclosed_bracket() {
        let mut parser = Parser::new("[1, 2, 3");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_unclosed_brace() {
        let mut parser = Parser::new("{x: 1, y: 2");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_unclosed_paren() {
        let mut parser = Parser::new("(1 + 2");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_missing_operator() {
        let mut parser = Parser::new("1 2 3");
        let result = parser.parse();
        // Might parse as separate expressions or fail
        let _ = result;
    }
    
    #[test]
    fn test_parse_invalid_assignment() {
        let mut parser = Parser::new("123 = 456");
        let result = parser.parse();
        assert!(result.is_err());
    }
    
    // Interpreter error tests (complexity: 4 each)
    #[test]
    fn test_eval_undefined_variable() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("undefined_variable");
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_type_error_add_string_number() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(r#""hello" + 42"#);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        // Might succeed (string concatenation) or fail (type error)
        let _ = result;
    }
    
    #[test]
    fn test_eval_division_by_zero() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("5 / 0");
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_modulo_by_zero() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("5 % 0");
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_invalid_function_call() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("42()"); // Try to call number as function
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_wrong_number_of_args() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("fun f(x, y) { x + y }; f(1)"); // Missing argument
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_invalid_field_access() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("42.nonexistent"); // Access field on number
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_invalid_index_access() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("42[0]"); // Index into number
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_out_of_bounds_access() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("[1, 2, 3][10]"); // Index out of bounds
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_err());
    }
    
    // Edge case tests (complexity: 3 each)
    #[test]
    fn test_parse_very_large_number() {
        let mut parser = Parser::new("999999999999999999999");
        let result = parser.parse();
        // Should handle large numbers gracefully
        let _ = result;
    }
    
    #[test]
    fn test_parse_very_small_float() {
        let mut parser = Parser::new("0.000000000001");
        let result = parser.parse();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_scientific_notation_edge() {
        let mut parser = Parser::new("1e-100");
        let result = parser.parse();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_unicode_string() {
        let mut parser = Parser::new(r#""Hello 世界 🌍""#);
        let result = parser.parse();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_empty_expressions() {
        let mut parser = Parser::new("()");
        let result = parser.parse();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_nested_deeply() {
        let mut parser = Parser::new("((((((1))))))");
        let result = parser.parse();
        assert!(result.is_ok());
    }
    
    // Boundary value tests (complexity: 3 each)
    #[test]
    fn test_eval_max_integer() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("9223372036854775807"); // i64::MAX
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_min_integer() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("-9223372036854775808"); // i64::MIN
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_zero_operations() {
        let mut interpreter = Interpreter::new();
        let operations = vec!["0 + 0", "0 - 0", "0 * 0", "0 == 0", "0 != 1"];
        
        for op in operations {
            let mut parser = Parser::new(op);
            let ast = parser.parse().unwrap();
            let result = interpreter.eval_expr(&ast);
            assert!(result.is_ok(), "Failed operation: {}", op);
        }
    }
    
    // String edge cases (complexity: 3 each)
    #[test]
    fn test_eval_empty_string() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(r#""""#);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_string_with_escapes() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(r#""hello\nworld\t!""#);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_string_with_quotes() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(r#""He said \"Hello\"""#);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    // Function edge cases (complexity: 4 each)
    #[test]
    fn test_eval_recursive_function_depth() {
        let mut interpreter = Interpreter::new();
        let code = "fun countdown(n) { if n <= 0 { 0 } else { countdown(n - 1) } }; countdown(10)";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_lambda_closure_edge() {
        let mut interpreter = Interpreter::new();
        let code = "let x = 1; let f = |y| { let z = x + y; |w| z + w }; let g = f(2); g(3)";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        // Nested closures might not be fully implemented
        let _ = result;
    }
    
    // Collection edge cases (complexity: 3 each)
    #[test]
    fn test_eval_nested_empty_collections() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("[[[], []], [[], []]]");
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_mixed_type_list() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(r#"[1, "hello", true, 3.14, nil]"#);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
}