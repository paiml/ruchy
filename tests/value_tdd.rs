//! TDD tests for Value type operations
//! Target: Test all Value methods and operations

#[cfg(test)]
mod tests {
    use ruchy::runtime::interpreter::Value;
    
    // Value creation tests (complexity: 2 each)
    #[test]
    fn test_value_integer() {
        let val = Value::Integer(42);
        assert_eq!(val.type_name(), "integer");
    }
    
    #[test]
    fn test_value_float() {
        let val = Value::Float(3.14);
        assert_eq!(val.type_name(), "float");
    }
    
    #[test]
    fn test_value_string() {
        let val = Value::String("hello".to_string().into());
        assert_eq!(val.type_name(), "string");
    }
    
    #[test]
    fn test_value_bool_true() {
        let val = Value::Bool(true);
        assert_eq!(val.type_name(), "boolean");
    }
    
    #[test]
    fn test_value_bool_false() {
        let val = Value::Bool(false);
        assert_eq!(val.type_name(), "boolean");
    }
    
    #[test]
    fn test_value_nil() {
        let val = Value::Nil;
        assert_eq!(val.type_name(), "nil");
    }
    
    // Type checking tests (complexity: 2 each)
    #[test]
    fn test_is_nil() {
        assert!(Value::Nil.is_nil());
        assert!(!Value::Integer(0).is_nil());
    }
    
    #[test]
    fn test_is_truthy_integer() {
        assert!(Value::Integer(1).is_truthy());
        assert!(Value::Integer(0).is_truthy()); // 0 is truthy in Ruchy
        assert!(Value::Integer(-1).is_truthy());
    }
    
    #[test]
    fn test_is_truthy_bool() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
    }
    
    #[test]
    fn test_is_truthy_nil() {
        assert!(!Value::Nil.is_truthy());
    }
    
    #[test]
    fn test_is_truthy_string() {
        assert!(Value::String("hello".to_string().into()).is_truthy());
        assert!(Value::String("".to_string().into()).is_truthy()); // Empty strings are truthy
    }
    
    // Basic value tests (complexity: 2 each) 
    #[test]
    fn test_value_pattern_match() {
        let val = Value::Integer(42);
        match val {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer"),
        }
    }
    
    #[test]
    fn test_value_string_pattern() {
        let val = Value::String("hello".to_string().into());
        match val {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string"),
        }
    }
    
    #[test]
    fn test_value_float_pattern() {
        let val = Value::Float(3.14);
        match val {
            Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }
    
    // Display tests (complexity: 2 each)
    #[test]
    fn test_display_integer() {
        let val = Value::Integer(42);
        assert_eq!(format!("{}", val), "42");
    }
    
    #[test]
    fn test_display_float() {
        let val = Value::Float(3.14);
        assert_eq!(format!("{}", val), "3.14");
    }
    
    #[test]
    fn test_display_string() {
        let val = Value::String("hello".to_string().into());
        assert_eq!(format!("{}", val), "hello");
    }
    
    #[test]
    fn test_display_bool_true() {
        let val = Value::Bool(true);
        assert_eq!(format!("{}", val), "true");
    }
    
    #[test]
    fn test_display_bool_false() {
        let val = Value::Bool(false);
        assert_eq!(format!("{}", val), "false");
    }
    
    #[test]
    fn test_display_nil() {
        let val = Value::Nil;
        assert_eq!(format!("{}", val), "nil");
    }
    
    // Equality tests (complexity: 3 each)
    #[test]
    fn test_equal_integers() {
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Integer(43));
    }
    
    #[test]
    fn test_equal_floats() {
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_ne!(Value::Float(3.14), Value::Float(3.15));
    }
    
    #[test]
    fn test_equal_strings() {
        let s1 = Value::String("hello".to_string().into());
        let s2 = Value::String("hello".to_string().into());
        let s3 = Value::String("world".to_string().into());
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }
    
    #[test]
    fn test_equal_bools() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Bool(false), Value::Bool(false));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }
    
    #[test]
    fn test_equal_nil() {
        assert_eq!(Value::Nil, Value::Nil);
    }
    
    #[test]
    fn test_not_equal_different_types() {
        assert_ne!(Value::Integer(42), Value::Float(42.0));
        assert_ne!(Value::Integer(1), Value::Bool(true));
        assert_ne!(Value::String("nil".to_string().into()), Value::Nil);
    }
    
    // Clone tests (complexity: 2 each)
    #[test]
    fn test_clone_integer() {
        let val = Value::Integer(42);
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }
    
    #[test]
    fn test_clone_string() {
        let val = Value::String("hello".to_string().into());
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }
}