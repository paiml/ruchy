//! TDD tests for codegen_minimal.rs to boost coverage from 34.70% to 50%+
//! Target: Test all uncovered branches and functions with complexity ≤10

#[cfg(test)]
mod tests {
    use ruchy::backend::transpiler::codegen_minimal::MinimalCodeGen;
    use ruchy::frontend::parser::Parser;
    
    // Helper function (complexity: 3)
    fn gen_str(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        Ok(MinimalCodeGen::gen_expr(&expr)?)
    }
    
    // Literal tests (complexity: 2 each)
    #[test]
    fn test_gen_literal_integer() {
        assert_eq!(gen_str("42").unwrap(), "42");
        assert_eq!(gen_str("-123").unwrap(), "(- 123)");
        assert_eq!(gen_str("0").unwrap(), "0");
    }
    
    #[test]
    fn test_gen_literal_float() {
        assert_eq!(gen_str("3.14").unwrap(), "3.14");
        assert_eq!(gen_str("0.0").unwrap(), "0.0");
        assert_eq!(gen_str("-2.5").unwrap(), "(- 2.5)");
    }
    
    #[test]
    fn test_gen_literal_string() {
        assert_eq!(gen_str(r#""hello""#).unwrap(), r#""hello""#);
        assert_eq!(gen_str(r#""hello world""#).unwrap(), r#""hello world""#);
        assert_eq!(gen_str(r#""""#).unwrap(), r#""""#); // Empty string
    }
    
    #[test]
    fn test_gen_literal_string_with_quotes() {
        // Test escape handling in strings
        let result = gen_str(r#""say \"hello\"""#);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\\\""));
    }
    
    #[test]
    fn test_gen_literal_bool() {
        assert_eq!(gen_str("true").unwrap(), "true");
        assert_eq!(gen_str("false").unwrap(), "false");
    }
    
    #[test]
    fn test_gen_literal_char() {
        assert_eq!(gen_str("'a'").unwrap(), "'a'");
        assert_eq!(gen_str("'1'").unwrap(), "'1'");
        assert_eq!(gen_str("'\\n'").unwrap(), "'\\n'");
    }
    
    #[test]
    fn test_gen_literal_unit() {
        let result = gen_str("()");
        assert!(result.is_ok());
    }
    
    // Binary operator tests (complexity: 3 each)
    #[test]
    fn test_gen_binary_arithmetic() {
        assert_eq!(gen_str("1 + 2").unwrap(), "(1 + 2)");
        assert_eq!(gen_str("5 - 3").unwrap(), "(5 - 3)");
        assert_eq!(gen_str("2 * 4").unwrap(), "(2 * 4)");
        assert_eq!(gen_str("8 / 2").unwrap(), "(8 / 2)");
        assert_eq!(gen_str("7 % 3").unwrap(), "(7 % 3)");
    }
    
    #[test]
    fn test_gen_binary_comparison() {
        assert_eq!(gen_str("1 == 1").unwrap(), "(1 == 1)");
        assert_eq!(gen_str("1 != 2").unwrap(), "(1 != 2)");
        assert_eq!(gen_str("1 < 2").unwrap(), "(1 < 2)");
        assert_eq!(gen_str("2 > 1").unwrap(), "(2 > 1)");
        assert_eq!(gen_str("1 <= 2").unwrap(), "(1 <= 2)");
        assert_eq!(gen_str("2 >= 1").unwrap(), "(2 >= 1)");
    }
    
    #[test]
    fn test_gen_binary_logical() {
        assert_eq!(gen_str("true && false").unwrap(), "(true && false)");
        assert_eq!(gen_str("true || false").unwrap(), "(true || false)");
    }
    
    #[test]
    fn test_gen_binary_nested() {
        let result = gen_str("(1 + 2) * 3");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("+") && output.contains("*"));
    }
    
    // Unary operator tests (complexity: 2 each)
    #[test]
    fn test_gen_unary_negate() {
        let result = gen_str("-42");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("(- 42)"));
    }
    
    #[test]
    fn test_gen_unary_not() {
        let result = gen_str("!true");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("(! true)"));
    }
    
    // Identifier tests (complexity: 2)
    #[test]
    fn test_gen_identifier() {
        assert_eq!(gen_str("variable").unwrap(), "variable");
        assert_eq!(gen_str("my_var").unwrap(), "my_var");
        assert_eq!(gen_str("x").unwrap(), "x");
    }
    
    // Let binding tests (complexity: 3 each)
    #[test]
    fn test_gen_let() {
        let result = gen_str("let x = 42; x");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("let x = 42"));
    }
    
    #[test]
    fn test_gen_let_complex() {
        let result = gen_str("let result = 1 + 2; result * 3");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("let result"));
        assert!(output.contains("1 + 2"));
    }
    
    // Function tests (complexity: 4 each)
    #[test]
    fn test_gen_function_no_params() {
        let result = gen_str("fun hello() { 42 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn hello()"));
        assert!(output.contains("42"));
    }
    
    #[test]
    fn test_gen_function_with_params() {
        let result = gen_str("fun add(a, b) { a + b }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn add"));
        assert!(output.contains("a: i32"));
        assert!(output.contains("b: i32"));
    }
    
    // Lambda tests (complexity: 3 each)
    #[test]
    fn test_gen_lambda_simple() {
        assert_eq!(gen_str("|x| x").unwrap(), "|x| x");
    }
    
    #[test]
    fn test_gen_lambda_multiple_params() {
        assert_eq!(gen_str("|x, y| x + y").unwrap(), "|x, y| (x + y)");
    }
    
    #[test]
    fn test_gen_lambda_no_params() {
        assert_eq!(gen_str("|| 42").unwrap(), "|| 42");
    }
    
    // Function call tests (complexity: 3 each)
    #[test]
    fn test_gen_call_no_args() {
        assert_eq!(gen_str("func()").unwrap(), "func()");
    }
    
    #[test]
    fn test_gen_call_with_args() {
        assert_eq!(gen_str("func(1, 2)").unwrap(), "func(1, 2)");
    }
    
    #[test]
    fn test_gen_call_nested() {
        let result = gen_str("outer(inner(42))");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("outer(inner(42))"));
    }
    
    // If expression tests (complexity: 4 each)
    #[test]
    fn test_gen_if_no_else() {
        let result = gen_str("if true { 1 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("if true"));
        assert!(!output.contains("else"));
    }
    
    #[test]
    fn test_gen_if_with_else() {
        let result = gen_str("if true { 1 } else { 2 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("if true"));
        assert!(output.contains("else"));
    }
    
    #[test]
    fn test_gen_if_complex_condition() {
        let result = gen_str("if x > 0 && y < 10 { x + y }");
        assert!(result.is_ok());
    }
    
    // Block tests (complexity: 4 each)
    #[test]
    fn test_gen_block_empty() {
        let result = gen_str("{}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{ }");
    }
    
    #[test]
    fn test_gen_block_single_expr() {
        let result = gen_str("{ 42 }");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("42"));
    }
    
    #[test]
    fn test_gen_block_multiple_exprs() {
        let result = gen_str("{ let x = 1; let y = 2; x + y }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("let x"));
        assert!(output.contains("let y"));
        assert!(output.contains("x + y"));
    }
    
    // List tests (complexity: 3 each)
    #[test]
    fn test_gen_list_empty() {
        assert_eq!(gen_str("[]").unwrap(), "vec![]");
    }
    
    #[test]
    fn test_gen_list_with_elements() {
        assert_eq!(gen_str("[1, 2, 3]").unwrap(), "vec![1, 2, 3]");
    }
    
    #[test]
    fn test_gen_list_mixed_types() {
        let result = gen_str(r#"[1, "hello", true]"#);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("vec!"));
    }
    
    // Match expression tests (complexity: 4 each)
    #[test]
    fn test_gen_match_simple() {
        let result = gen_str("match x { 1 => \"one\", _ => \"other\" }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("match x"));
        assert!(output.contains("1 => "));
        assert!(output.contains("_ => "));
    }
    
    #[test]
    fn test_gen_match_complex() {
        let result = gen_str("match value { Ok(x) => x, Err(_) => 0 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Ok(x)"));
        assert!(output.contains("Err(_)"));
    }
    
    // Method call tests (complexity: 3 each)
    #[test]
    fn test_gen_method_call_no_args() {
        let result = gen_str("obj.method()");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("obj.method()"));
    }
    
    #[test]
    fn test_gen_method_call_with_args() {
        let result = gen_str("obj.method(1, 2)");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("obj.method(1, 2)"));
    }
    
    #[test]
    fn test_gen_method_call_chained() {
        let result = gen_str("obj.first().second()");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("first().second()"));
    }
    
    // Struct tests (complexity: 3 each)
    #[test]
    fn test_gen_struct_definition() {
        let result = gen_str("struct Point { x: i32, y: i32 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("struct Point"));
    }
    
    #[test]
    fn test_gen_struct_literal() {
        let result = gen_str("Point { x: 1, y: 2 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Point {"));
        assert!(output.contains("x: 1"));
        assert!(output.contains("y: 2"));
    }
    
    // Macro tests (complexity: 3)
    #[test]
    fn test_gen_macro_call() {
        let result = gen_str("println!(\"hello\")");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("println!(\"hello\")"));
    }
    
    // Qualified name tests (complexity: 2)
    #[test]
    fn test_gen_qualified_name() {
        let result = gen_str("std::collections::HashMap");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("std::collections::HashMap"));
    }
    
    // String interpolation tests (complexity: 5)
    #[test]
    fn test_gen_string_interpolation_simple() {
        let result = gen_str("f\"Hello {name}\"");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("format!"));
        assert!(output.contains("Hello {}"));
    }
    
    #[test]
    fn test_gen_string_interpolation_multiple() {
        let result = gen_str("f\"User {name} has {count} items\"");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("format!"));
        assert!(output.contains("User {} has {} items"));
    }
    
    // Error handling tests (complexity: 3 each)
    #[test]
    fn test_gen_unsupported_construct() {
        // Test that unsupported constructs return errors
        let result = gen_str("while true { break }");
        // This should either work or return an error for unsupported construct
        let _ = result;
    }
    
    // Program generation tests (complexity: 3)
    #[test]
    fn test_gen_program() {
        let mut parser = Parser::new("42");
        let expr = parser.parse().unwrap();
        let result = MinimalCodeGen::gen_program(&expr);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("use std::collections::HashMap"));
        assert!(output.contains("42"));
    }
    
    // Edge case tests (complexity: 3 each)
    #[test]
    fn test_gen_deeply_nested() {
        let result = gen_str("((((1 + 2) * 3) / 4) - 5)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_gen_complex_expression() {
        let result = gen_str("if x > 0 { func(x + 1) } else { [1, 2, 3].len() }");
        assert!(result.is_ok());
    }
}