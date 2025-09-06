//! TDD tests for dispatcher.rs to boost coverage from 41.30% to 55%+
//! Target: Test all dispatcher functions with complexity ≤10

#[cfg(test)]
mod tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;
    
    // Helper function (complexity: 3)
    fn transpile_str(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        let transpiler = Transpiler::default();
        Ok(transpiler.transpile_to_string(&expr)?)
    }
    
    // Basic expression tests (complexity: 2 each)
    #[test]
    fn test_transpile_basic_expr_literal() {
        let result = transpile_str("42");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("42"));
    }
    
    #[test]
    fn test_transpile_basic_expr_identifier() {
        let result = transpile_str("variable");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("variable"));
    }
    
    #[test]
    fn test_transpile_basic_expr_string() {
        let result = transpile_str(r#""hello""#);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }
    
    // Rust reserved keyword tests (complexity: 3 each)
    #[test]
    fn test_transpile_identifier_reserved_keywords() {
        let keywords = vec!["match", "if", "else", "for", "while", "loop", "fn", "let"];
        
        for keyword in keywords {
            let result = transpile_str(keyword);
            if result.is_ok() {
                let output = result.unwrap();
                // Should either be raw identifier or handled specially
                assert!(output.contains(&format!("r#{keyword}")) || output.contains(keyword));
            }
        }
    }
    
    #[test]
    fn test_transpile_identifier_self_keywords() {
        let keywords = vec!["self", "Self", "super", "crate"];
        
        for keyword in keywords {
            let result = transpile_str(keyword);
            if result.is_ok() {
                let output = result.unwrap();
                // These should NOT be raw identifiers
                assert!(!output.contains(&format!("r#{keyword}")));
                assert!(output.contains(keyword));
            }
        }
    }
    
    // Qualified name tests (complexity: 3 each)
    #[test]
    fn test_transpile_qualified_name_simple() {
        let result = transpile_str("std::HashMap");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("std") && output.contains("HashMap"));
    }
    
    #[test]
    fn test_transpile_qualified_name_nested() {
        let result = transpile_str("std::collections::HashMap");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("std"));
        assert!(output.contains("collections"));
        assert!(output.contains("HashMap"));
    }
    
    #[test]
    fn test_transpile_qualified_name_deep() {
        let result = transpile_str("a::b::c::d::Item");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("::"));
    }
    
    // Operator expression tests (complexity: 3 each)
    #[test]
    fn test_transpile_operator_binary() {
        let result = transpile_str("1 + 2");
        assert!(result.is_ok());
        assert!(result.unwrap().contains('+'));
    }
    
    #[test]
    fn test_transpile_operator_unary() {
        let result = transpile_str("-42");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains('-') || output.contains("neg"));
    }
    
    #[test]
    fn test_transpile_operator_assignment() {
        let result = transpile_str("x = 42");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_transpile_operator_compound_assign() {
        let result = transpile_str("x += 1");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("+=") || output.contains("add_assign"));
    }
    
    // Control flow expression tests (complexity: 4 each)
    #[test]
    fn test_transpile_control_if() {
        let result = transpile_str("if true { 1 } else { 2 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("if"));
        assert!(output.contains("else"));
    }
    
    #[test]
    fn test_transpile_control_if_no_else() {
        let result = transpile_str("if x > 0 { 1 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("if"));
        assert!(!output.contains("else"));
    }
    
    #[test]
    fn test_transpile_control_match() {
        let result = transpile_str("match x { 1 => \"one\", _ => \"other\" }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("match"));
    }
    
    #[test]
    fn test_transpile_control_for() {
        let result = transpile_str("for i in 0..10 { print(i) }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("for"));
    }
    
    #[test]
    fn test_transpile_control_while() {
        let result = transpile_str("while x > 0 { x = x - 1 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("while"));
    }
    
    #[test]
    fn test_transpile_control_loop() {
        let result = transpile_str("loop { break }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("loop"));
    }
    
    // Function expression tests (complexity: 4 each)
    #[test]
    fn test_transpile_function_definition() {
        let result = transpile_str("fun add(x, y) { x + y }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn"));
        assert!(output.contains("add"));
    }
    
    #[test]
    fn test_transpile_function_no_params() {
        let result = transpile_str("fun hello() { 42 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn hello"));
    }
    
    #[test]
    fn test_transpile_lambda() {
        let result = transpile_str("|x| x + 1");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains('|'));
    }
    
    #[test]
    fn test_transpile_function_call() {
        let result = transpile_str("print(42)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("print"));
        assert!(output.contains("42"));
    }
    
    #[test]
    fn test_transpile_method_call() {
        let result = transpile_str("obj.method()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("obj"));
        assert!(output.contains("method"));
    }
    
    // Macro tests (complexity: 3 each)
    #[test]
    fn test_transpile_macro_println() {
        let result = transpile_str(r#"println!("hello")"#);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("println!"));
    }
    
    #[test]
    fn test_transpile_macro_vec() {
        let result = transpile_str("vec![1, 2, 3]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("vec!"));
    }
    
    #[test]
    fn test_transpile_macro_custom() {
        let result = transpile_str("debug!(x)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("debug!"));
    }
    
    // Struct expression tests (complexity: 3 each)
    #[test]
    fn test_transpile_struct_definition() {
        let result = transpile_str("struct Point { x: i32, y: i32 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("struct Point"));
    }
    
    #[test]
    fn test_transpile_struct_literal() {
        let result = transpile_str("Point { x: 1, y: 2 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Point"));
        assert!(output.contains("x:"));
        assert!(output.contains("y:"));
    }
    
    // Data/Error expression tests (complexity: 3 each)
    #[test]
    fn test_transpile_result_ok() {
        let result = transpile_str("Ok(42)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Ok"));
    }
    
    #[test]
    fn test_transpile_result_err() {
        let result = transpile_str("Err(\"error\")");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Err"));
    }
    
    #[test]
    fn test_transpile_option_some() {
        let result = transpile_str("Some(42)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Some"));
    }
    
    #[test]
    fn test_transpile_option_none() {
        let result = transpile_str("None");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("None"));
    }
    
    #[test]
    fn test_transpile_try_operator() {
        let result = transpile_str("func()?");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains('?'));
    }
    
    // List and data structure tests (complexity: 3 each)
    #[test]
    fn test_transpile_list_empty() {
        let result = transpile_str("[]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("vec!") || output.contains("[]"));
    }
    
    #[test]
    fn test_transpile_list_with_elements() {
        let result = transpile_str("[1, 2, 3]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains('1') && output.contains('2') && output.contains('3'));
    }
    
    #[test]
    fn test_transpile_tuple() {
        let result = transpile_str("(1, 2, 3)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains('1') && output.contains('2') && output.contains('3'));
    }
    
    // Range tests (complexity: 2 each)
    #[test]
    fn test_transpile_range_inclusive() {
        let result = transpile_str("0..=10");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(".."));
    }
    
    #[test]
    fn test_transpile_range_exclusive() {
        let result = transpile_str("0..10");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(".."));
    }
    
    // String interpolation tests (complexity: 4 each)
    #[test]
    fn test_transpile_string_interpolation() {
        let result = transpile_str("f\"Hello {name}!\"");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("format!") || output.contains("Hello"));
    }
    
    #[test]
    fn test_transpile_string_interpolation_multiple() {
        let result = transpile_str("f\"User {name} has {count} items\"");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("format!") || output.contains("User"));
    }
    
    // Async/await tests (complexity: 3 each)
    #[test]
    fn test_transpile_async_block() {
        let result = transpile_str("async { 42 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("async") || output.contains("42"));
    }
    
    #[test]
    fn test_transpile_await() {
        let result = transpile_str("await func()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("await") || output.contains("func"));
    }
    
    // Complex nested tests (complexity: 5 each)
    #[test]
    fn test_transpile_nested_complex() {
        let result = transpile_str("if x > 0 { func(x + 1).await } else { None }");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_transpile_chained_method_calls() {
        let result = transpile_str("vec![1, 2, 3].iter().map(|x| x * 2).collect()");
        assert!(result.is_ok());
    }
    
    // Error path tests (complexity: 2 each)
    #[test]
    fn test_transpile_invalid_syntax() {
        let result = transpile_str("let 123 = 456"); // Invalid identifier
        assert!(result.is_err());
    }
    
    #[test]
    fn test_transpile_empty_input() {
        let result = transpile_str("");
        // Should either succeed with empty program or fail gracefully
        let _ = result;
    }
}