//! TDD tests for runtime utilities and helper functions
//! Target: Test utility functions with complexity ≤10

#[cfg(test)]
mod tests {
    use ruchy::runtime::interpreter::{Interpreter, Value};
    use ruchy::frontend::parser::Parser;
    
    // Interpreter creation tests (complexity: 2 each)
    #[test]
    fn test_interpreter_new() {
        let interpreter = Interpreter::new();
        // Should create successfully
        let _ = interpreter;
    }
    
    #[test]
    fn test_interpreter_multiple_instances() {
        let _interpreter1 = Interpreter::new();
        let _interpreter2 = Interpreter::new();
        // Should be able to create multiple instances
    }
    
    // Value creation helpers (complexity: 2 each)
    #[test]
    fn test_value_from_i64() {
        let val = Value::from_i64(42);
        match val {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer"),
        }
    }
    
    #[test]
    fn test_value_from_f64() {
        let val = Value::from_f64(3.14);
        match val {
            Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }
    
    #[test]
    fn test_value_from_bool() {
        let val_true = Value::from_bool(true);
        let val_false = Value::from_bool(false);
        
        match val_true {
            Value::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected bool"),
        }
        
        match val_false {
            Value::Bool(b) => assert_eq!(b, false),
            _ => panic!("Expected bool"),
        }
    }
    
    #[test]
    fn test_value_from_string() {
        let val = Value::from_string("hello".to_string());
        match val {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string"),
        }
    }
    
    #[test]
    fn test_value_nil() {
        let val = Value::nil();
        match val {
            Value::Nil => {},
            _ => panic!("Expected nil"),
        }
    }
    
    // Type name tests (complexity: 2 each)
    #[test]
    fn test_value_type_names() {
        assert_eq!(Value::Integer(42).type_name(), "integer");
        assert_eq!(Value::Float(3.14).type_name(), "float");
        assert_eq!(Value::Bool(true).type_name(), "boolean");
        assert_eq!(Value::String("hello".to_string().into()).type_name(), "string");
        assert_eq!(Value::Nil.type_name(), "nil");
    }
    
    // Truthiness tests (complexity: 3 each)
    #[test]
    fn test_integer_truthiness() {
        assert!(Value::Integer(1).is_truthy());
        assert!(Value::Integer(0).is_truthy());
        assert!(Value::Integer(-1).is_truthy());
    }
    
    #[test]
    fn test_float_truthiness() {
        assert!(Value::Float(1.0).is_truthy());
        assert!(Value::Float(0.0).is_truthy());
        assert!(Value::Float(-1.0).is_truthy());
    }
    
    #[test]
    fn test_bool_truthiness() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
    }
    
    #[test]
    fn test_string_truthiness() {
        assert!(Value::String("hello".to_string().into()).is_truthy());
        assert!(Value::String("".to_string().into()).is_truthy());
    }
    
    #[test]
    fn test_nil_truthiness() {
        assert!(!Value::Nil.is_truthy());
    }
    
    // Simple evaluation tests (complexity: 4 each)
    #[test]
    fn test_eval_simple_values() {
        let mut interpreter = Interpreter::new();
        
        let test_cases = vec![
            ("42", "42"),
            ("3.14", "3.14"),
            ("true", "true"),
            ("false", "false"),
            (r#""hello""#, "hello"),
        ];
        
        for (input, expected) in test_cases {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let result = interpreter.eval_expr(&ast).unwrap();
            let result_str = format!("{}", result);
            assert_eq!(result_str, expected, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_eval_arithmetic_basic() {
        let mut interpreter = Interpreter::new();
        
        let test_cases = vec![
            ("1 + 1", "2"),
            ("5 - 3", "2"),
            ("2 * 3", "6"),
            ("10 / 2", "5"),
        ];
        
        for (input, expected) in test_cases {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let result = interpreter.eval_expr(&ast).unwrap();
            let result_str = format!("{}", result);
            assert_eq!(result_str, expected, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_eval_comparisons() {
        let mut interpreter = Interpreter::new();
        
        let test_cases = vec![
            ("1 == 1", "true"),
            ("1 != 2", "true"),
            ("1 < 2", "true"),
            ("2 > 1", "true"),
            ("1 <= 1", "true"),
            ("2 >= 2", "true"),
        ];
        
        for (input, expected) in test_cases {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let result = interpreter.eval_expr(&ast).unwrap();
            let result_str = format!("{}", result);
            assert_eq!(result_str, expected, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_eval_logical_ops() {
        let mut interpreter = Interpreter::new();
        
        let test_cases = vec![
            ("true && true", "true"),
            ("true && false", "false"),
            ("false || true", "true"),
            ("false || false", "false"),
            ("!true", "false"),
            ("!false", "true"),
        ];
        
        for (input, expected) in test_cases {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let result = interpreter.eval_expr(&ast).unwrap();
            let result_str = format!("{}", result);
            assert_eq!(result_str, expected, "Failed for input: {}", input);
        }
    }
    
    // Value conversion tests (complexity: 3 each)
    #[test]
    fn test_value_as_i64_success() {
        let val = Value::Integer(42);
        let result = val.as_i64();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_value_as_i64_error() {
        let val = Value::Float(3.14);
        let result = val.as_i64();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_value_as_f64_success() {
        let val = Value::Float(3.14);
        let result = val.as_f64();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3.14);
    }
    
    #[test]
    fn test_value_as_f64_error() {
        let val = Value::Integer(42);
        let result = val.as_f64();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_value_as_bool_success() {
        let val = Value::Bool(true);
        let result = val.as_bool();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
    
    #[test]
    fn test_value_as_bool_error() {
        let val = Value::Integer(1);
        let result = val.as_bool();
        assert!(result.is_err());
    }
    
    // Display formatting tests (complexity: 2 each)
    #[test]
    fn test_value_display_formats() {
        let values = vec![
            (Value::Integer(42), "42"),
            (Value::Float(3.14), "3.14"),
            (Value::Bool(true), "true"),
            (Value::Bool(false), "false"),
            (Value::String("hello".to_string().into()), "hello"),
            (Value::Nil, "nil"),
        ];
        
        for (value, expected) in values {
            let formatted = format!("{}", value);
            assert_eq!(formatted, expected);
        }
    }
    
    // Clone and equality tests (complexity: 2 each)
    #[test]
    fn test_value_clone() {
        let original = Value::Integer(42);
        let cloned = original.clone();
        
        assert_eq!(original, cloned);
    }
    
    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Integer(43));
        
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_ne!(Value::Float(3.14), Value::Float(3.15));
        
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
        
        assert_eq!(Value::Nil, Value::Nil);
        
        // Different types should not be equal
        assert_ne!(Value::Integer(42), Value::Float(42.0));
        assert_ne!(Value::Bool(true), Value::Integer(1));
    }
}