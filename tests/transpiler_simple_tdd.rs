//! Simple TDD tests for transpiler to boost coverage
//! Target: Test transpilation with minimal complexity ≤5

#[cfg(test)]
mod tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;
    
    // Helper function (complexity: 3)
    fn transpile_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(code);
        let ast = parser.parse()?;
        let transpiler = Transpiler::default();
        Ok(transpiler.transpile_to_string(&ast)?)
    }
    
    // Basic literal tests (complexity: 2 each)
    #[test]
    fn test_integer_literal() {
        let result = transpile_code("42");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_float_literal() {
        let result = transpile_code("3.14");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_string_literal() {
        let result = transpile_code(r#""hello""#);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_true_literal() {
        let result = transpile_code("true");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_false_literal() {
        let result = transpile_code("false");
        assert!(result.is_ok());
    }
    
    // Arithmetic operations (complexity: 2 each)
    #[test]
    fn test_addition() {
        let result = transpile_code("1 + 2");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_subtraction() {
        let result = transpile_code("5 - 3");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_multiplication() {
        let result = transpile_code("2 * 3");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_division() {
        let result = transpile_code("10 / 2");
        assert!(result.is_ok());
    }
    
    // Comparison operations (complexity: 2 each)
    #[test]
    fn test_equality() {
        let result = transpile_code("1 == 1");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_inequality() {
        let result = transpile_code("1 != 2");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_less_than() {
        let result = transpile_code("1 < 2");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_greater_than() {
        let result = transpile_code("2 > 1");
        assert!(result.is_ok());
    }
    
    // Variable operations (complexity: 2 each)
    #[test]
    fn test_identifier() {
        let result = transpile_code("x");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_let_binding() {
        let result = transpile_code("let x = 42");
        assert!(result.is_ok());
    }
    
    // Collection literals (complexity: 2 each)
    #[test]
    fn test_empty_list() {
        let result = transpile_code("[]");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_list_with_elements() {
        let result = transpile_code("[1, 2, 3]");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_empty_object() {
        let result = transpile_code("{}");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_object_with_fields() {
        let result = transpile_code("{x: 1, y: 2}");
        assert!(result.is_ok());
    }
    
    // Control flow (complexity: 2 each)
    #[test]
    fn test_if_expression() {
        let result = transpile_code("if true { 1 }");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_if_else() {
        let result = transpile_code("if true { 1 } else { 2 }");
        assert!(result.is_ok());
    }
    
    // Function-related (complexity: 2 each)
    #[test]
    fn test_function_definition() {
        let result = transpile_code("fun add(a, b) { a + b }");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_function_call() {
        let result = transpile_code("print(42)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_lambda() {
        let result = transpile_code("|x| x + 1");
        assert!(result.is_ok());
    }
    
    // Complex expressions (complexity: 3 each)
    #[test]
    fn test_nested_expression() {
        let result = transpile_code("(1 + 2) * 3");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_chained_calls() {
        let result = transpile_code("obj.method().field");
        assert!(result.is_ok());
    }
    
    // Loop constructs (complexity: 2 each)
    #[test]
    fn test_while_loop() {
        let result = transpile_code("while true { 1 }");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_for_loop() {
        let result = transpile_code("for x in [1, 2, 3] { x }");
        assert!(result.is_ok());
    }
    
    // Error handling (complexity: 2 each)
    #[test]
    fn test_invalid_syntax_fails() {
        let result = transpile_code("let 123 = 456"); // Invalid identifier
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_input() {
        let result = transpile_code("");
        // Empty input should either succeed (empty program) or fail gracefully
        let _ = result;
    }
}