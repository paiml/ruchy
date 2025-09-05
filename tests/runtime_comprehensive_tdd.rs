//! Comprehensive TDD tests for runtime modules
//! Target: Improve overall runtime coverage with complexity ≤10

#[cfg(test)]
mod tests {
    use ruchy::runtime::interpreter::{Interpreter, Value};
    use ruchy::frontend::parser::Parser;
    use std::rc::Rc;
    
    // Helper function to evaluate a string
    fn eval_string(interpreter: &mut Interpreter, input: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse_expr()?;
        Ok(interpreter.eval_expr(&expr)?)
    }
    
    // Test 1: Basic integer arithmetic (complexity: 4)
    #[test]
    fn test_integer_arithmetic() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "2 + 3").unwrap();
        assert_eq!(result, Value::Integer(5));
        
        let result = eval_string(&mut interpreter, "10 - 4").unwrap();
        assert_eq!(result, Value::Integer(6));
        
        let result = eval_string(&mut interpreter, "3 * 4").unwrap();
        assert_eq!(result, Value::Integer(12));
    }
    
    // Test 2: Float arithmetic (complexity: 4)
    #[test]
    fn test_float_arithmetic() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "2.5 + 3.5").unwrap();
        assert_eq!(result, Value::Float(6.0));
        
        let result = eval_string(&mut interpreter, "10.0 / 4.0").unwrap();
        assert_eq!(result, Value::Float(2.5));
    }
    
    // Test 3: Boolean logic (complexity: 4)
    #[test]
    fn test_boolean_logic() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "true && true").unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = eval_string(&mut interpreter, "true && false").unwrap();
        assert_eq!(result, Value::Bool(false));
        
        let result = eval_string(&mut interpreter, "false || true").unwrap();
        assert_eq!(result, Value::Bool(true));
    }
    
    // Test 4: String concatenation (complexity: 3)
    #[test]
    fn test_string_concat() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "\"hello\" + \" \" + \"world\"").unwrap();
        if let Value::String(s) = result {
            assert_eq!(&**s, "hello world");
        } else {
            panic!("Expected string");
        }
    }
    
    // Test 5: Variable assignment (complexity: 3)
    #[test]
    fn test_variable_assignment() {
        let mut interpreter = Interpreter::new();
        
        eval_string(&mut interpreter, "x = 10").unwrap();
        let result = eval_string(&mut interpreter, "x + 5").unwrap();
        assert_eq!(result, Value::Integer(15));
    }
    
    // Test 6: Array creation (complexity: 4)
    #[test]
    fn test_array_creation() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "[1, 2, 3]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[2], Value::Integer(3));
        } else {
            panic!("Expected array");
        }
    }
    
    // Test 7: If expression (complexity: 4)
    #[test]
    fn test_if_expression() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "if true { 10 } else { 20 }").unwrap();
        assert_eq!(result, Value::Integer(10));
        
        let result = eval_string(&mut interpreter, "if false { 10 } else { 20 }").unwrap();
        assert_eq!(result, Value::Integer(20));
    }
    
    // Test 8: Comparison operators (complexity: 5)
    #[test]
    fn test_comparison() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "5 > 3").unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = eval_string(&mut interpreter, "5 < 3").unwrap();
        assert_eq!(result, Value::Bool(false));
        
        let result = eval_string(&mut interpreter, "5 == 5").unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = eval_string(&mut interpreter, "5 != 3").unwrap();
        assert_eq!(result, Value::Bool(true));
    }
    
    // Test 9: Block expressions (complexity: 4)
    #[test]
    fn test_block_expression() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "{ x = 5; y = 10; x + y }").unwrap();
        assert_eq!(result, Value::Integer(15));
    }
    
    // Test 10: Unary operators (complexity: 3)
    #[test]
    fn test_unary_operators() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "-5").unwrap();
        assert_eq!(result, Value::Integer(-5));
        
        let result = eval_string(&mut interpreter, "!true").unwrap();
        assert_eq!(result, Value::Bool(false));
    }
    
    // Test 11: Parser with interpreter (complexity: 5)
    #[test]
    fn test_parser_integration() {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new("2 * (3 + 4)");
        
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(14));
    }
    
    // Test 12: Multiple statements (complexity: 5)
    #[test]
    fn test_multiple_statements() {
        let mut interpreter = Interpreter::new();
        
        eval_string(&mut interpreter, "a = 1").unwrap();
        eval_string(&mut interpreter, "b = 2").unwrap();
        eval_string(&mut interpreter, "c = 3").unwrap();
        
        let result = eval_string(&mut interpreter, "a + b + c").unwrap();
        assert_eq!(result, Value::Integer(6));
    }
    
    // Test 13: Nested if expressions (complexity: 5)
    #[test]
    fn test_nested_if() {
        let mut interpreter = Interpreter::new();
        
        let code = "if true { if false { 1 } else { 2 } } else { 3 }";
        let result = eval_string(&mut interpreter, code).unwrap();
        assert_eq!(result, Value::Integer(2));
    }
    
    // Test 14: Complex arithmetic (complexity: 4)
    #[test]
    fn test_complex_arithmetic() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "(10 + 20) * 3 - 5").unwrap();
        assert_eq!(result, Value::Integer(85));
        
        let result = eval_string(&mut interpreter, "100 / (2 + 3)").unwrap();
        assert_eq!(result, Value::Integer(20));
    }
    
    // Test 15: Mixed types (complexity: 5)
    #[test]
    fn test_mixed_types() {
        let mut interpreter = Interpreter::new();
        
        // Integer to float conversion
        let result = eval_string(&mut interpreter, "2 + 3.5").unwrap();
        assert_eq!(result, Value::Float(5.5));
        
        // Float division
        let result = eval_string(&mut interpreter, "10.0 / 3.0").unwrap();
        if let Value::Float(f) = result {
            assert!((f - 3.333333).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }
    
    // Test 16: Logical short-circuit (complexity: 5)
    #[test]
    fn test_logical_short_circuit() {
        let mut interpreter = Interpreter::new();
        
        // Should short-circuit and not evaluate second part
        eval_string(&mut interpreter, "x = 0").unwrap();
        let result = eval_string(&mut interpreter, "false && (x = 10)").unwrap();
        assert_eq!(result, Value::Bool(false));
        
        // x should still be 0
        let result = eval_string(&mut interpreter, "x").unwrap();
        assert_eq!(result, Value::Integer(0));
    }
    
    // Test 17: While loop (complexity: 6)
    #[test]
    fn test_while_loop() {
        let mut interpreter = Interpreter::new();
        
        let code = "{ i = 0; sum = 0; while i < 5 { sum = sum + i; i = i + 1 }; sum }";
        let result = eval_string(&mut interpreter, code).unwrap();
        assert_eq!(result, Value::Integer(10)); // 0+1+2+3+4
    }
    
    // Test 18: For loop with range (complexity: 5)
    #[test]
    fn test_for_loop() {
        let mut interpreter = Interpreter::new();
        
        let code = "{ sum = 0; for i in 0..5 { sum = sum + i }; sum }";
        let result = eval_string(&mut interpreter, code).unwrap();
        assert_eq!(result, Value::Integer(10)); // 0+1+2+3+4
    }
    
    // Test 19: Array indexing (complexity: 4)
    #[test]
    #[ignore = "IndexAccess not yet implemented"]
    fn test_array_indexing() {
        let mut interpreter = Interpreter::new();
        
        eval_string(&mut interpreter, "arr = [10, 20, 30]").unwrap();
        let result = eval_string(&mut interpreter, "arr[0]").unwrap();
        assert_eq!(result, Value::Integer(10));
        
        let result = eval_string(&mut interpreter, "arr[2]").unwrap();
        assert_eq!(result, Value::Integer(30));
    }
    
    // Test 20: Function definition and call (complexity: 6)
    #[test]
    fn test_function_definition() {
        let mut interpreter = Interpreter::new();
        
        // Define a simple function
        eval_string(&mut interpreter, "fun add(a, b) { a + b }").unwrap();
        
        // Call the function
        let result = eval_string(&mut interpreter, "add(3, 4)").unwrap();
        assert_eq!(result, Value::Integer(7));
        
        let result = eval_string(&mut interpreter, "add(10, 20)").unwrap();
        assert_eq!(result, Value::Integer(30));
    }
    
    // Test 21: Lambda functions (complexity: 5)
    #[test]
    fn test_lambda() {
        let mut interpreter = Interpreter::new();
        
        eval_string(&mut interpreter, "double = |x| x * 2").unwrap();
        let result = eval_string(&mut interpreter, "double(5)").unwrap();
        assert_eq!(result, Value::Integer(10));
    }
    
    // Test 22: Error handling (complexity: 4)
    #[test]
    fn test_error_handling() {
        let mut interpreter = Interpreter::new();
        
        // Undefined variable
        let result = eval_string(&mut interpreter, "undefined_var");
        assert!(result.is_err());
        
        // Division by zero
        let result = eval_string(&mut interpreter, "10 / 0");
        assert!(result.is_err());
    }
    
    // Test 23: Nil value (complexity: 3)
    #[test]
    #[ignore = "nil not yet implemented"]
    fn test_nil_value() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "nil").unwrap();
        assert_eq!(result, Value::Nil);
        
        let result = eval_string(&mut interpreter, "x = nil; x").unwrap();
        assert_eq!(result, Value::Nil);
    }
    
    // Test 24: Tuple creation (complexity: 4)
    #[test]
    fn test_tuple_creation() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "(1, 2, 3)").unwrap();
        if let Value::Tuple(t) = result {
            assert_eq!(t.len(), 3);
            assert_eq!(t[0], Value::Integer(1));
            assert_eq!(t[2], Value::Integer(3));
        } else {
            panic!("Expected tuple");
        }
    }
    
    // Test 25: Modulo operator (complexity: 3)
    #[test]
    fn test_modulo() {
        let mut interpreter = Interpreter::new();
        
        let result = eval_string(&mut interpreter, "10 % 3").unwrap();
        assert_eq!(result, Value::Integer(1));
        
        let result = eval_string(&mut interpreter, "20 % 5").unwrap();
        assert_eq!(result, Value::Integer(0));
    }
}