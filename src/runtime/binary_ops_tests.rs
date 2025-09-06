//! Unit tests for binary operations module
//! Ensures comprehensive coverage of all binary operation evaluation paths

#[cfg(test)]
mod tests {
    use crate::runtime::{Value, binary_ops::evaluate_binary_op};
    use crate::frontend::ast::BinaryOp;

    #[test]
    fn test_int_addition() {
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Int(5), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_int_subtraction() {
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::Int(10), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_int_multiplication() {
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Int(4), &Value::Int(7)).unwrap();
        assert_eq!(result, Value::Int(28));
    }

    #[test]
    fn test_int_division() {
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Int(20), &Value::Int(4)).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_int_modulo() {
        let result = evaluate_binary_op(&BinaryOp::Modulo, &Value::Int(17), &Value::Int(5)).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_float_addition() {
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Float(3.5), &Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(6.0));
    }

    #[test]
    fn test_float_subtraction() {
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::Float(10.5), &Value::Float(3.5)).unwrap();
        assert_eq!(result, Value::Float(7.0));
    }

    #[test]
    fn test_float_multiplication() {
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Float(2.5), &Value::Float(4.0)).unwrap();
        assert_eq!(result, Value::Float(10.0));
    }

    #[test]
    fn test_float_division() {
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Float(15.0), &Value::Float(3.0)).unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_mixed_numeric_addition() {
        // Mixed numeric operations are not supported - they should return an error
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Int(5), &Value::Float(2.5));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot add"));
        
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Float(3.5), &Value::Int(2));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot add"));
    }

    #[test]
    fn test_string_concatenation() {
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::String("Hello, ".to_string()), &Value::String("World!".to_string())).unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_list_concatenation() {
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(3), Value::Int(4)]);
        let result = evaluate_binary_op(&BinaryOp::Add, &list1, &list2).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    }

    #[test]
    fn test_boolean_and() {
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(true), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(true), &Value::Bool(false)).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(false), &Value::Bool(true)).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(false), &Value::Bool(false)).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_boolean_or() {
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(true), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(true), &Value::Bool(false)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(false), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(false), &Value::Bool(false)).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_int_comparisons() {
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Int(5), &Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(false));
        
        assert_eq!(evaluate_binary_op(&BinaryOp::NotEqual, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::NotEqual, &Value::Int(5), &Value::Int(5)).unwrap(), Value::Bool(false));
        
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::Int(3), &Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(false));
        
        assert_eq!(evaluate_binary_op(&BinaryOp::LessEqual, &Value::Int(3), &Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::LessEqual, &Value::Int(5), &Value::Int(5)).unwrap(), Value::Bool(true));
        
        assert_eq!(evaluate_binary_op(&BinaryOp::Greater, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Greater, &Value::Int(3), &Value::Int(5)).unwrap(), Value::Bool(false));
        
        assert_eq!(evaluate_binary_op(&BinaryOp::GreaterEqual, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::GreaterEqual, &Value::Int(5), &Value::Int(5)).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_float_comparisons() {
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Float(5.0), &Value::Float(5.0)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::Float(3.5), &Value::Float(5.5)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Greater, &Value::Float(7.5), &Value::Float(3.5)).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_comparisons() {
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::String("hello".to_string()), &Value::String("hello".to_string())).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::String("hello".to_string()), &Value::String("world".to_string())).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::NotEqual, &Value::String("hello".to_string()), &Value::String("world".to_string())).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_boolean_comparisons() {
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Bool(true), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Bool(true), &Value::Bool(false)).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::NotEqual, &Value::Bool(true), &Value::Bool(false)).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_power_operation() {
        assert_eq!(evaluate_binary_op(&BinaryOp::Power, &Value::Int(2), &Value::Int(3)).unwrap(), Value::Int(8));
        assert_eq!(evaluate_binary_op(&BinaryOp::Power, &Value::Float(2.0), &Value::Float(3.0)).unwrap(), Value::Float(8.0));
    }

    #[test]
    fn test_division_by_zero_error() {
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Int(10), &Value::Int(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[test]
    fn test_modulo_by_zero_error() {
        let result = evaluate_binary_op(&BinaryOp::Modulo, &Value::Int(10), &Value::Int(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Modulo by zero"));
    }

    #[test]
    fn test_type_mismatch_errors() {
        // String + Int should fail
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::String("hello".to_string()), &Value::Int(5));
        assert!(result.is_err());
        
        // Bool + Int should fail
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Bool(true), &Value::Int(5));
        assert!(result.is_err());
        
        // List - List should fail
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::List(vec![]), &Value::List(vec![]));
        assert!(result.is_err());
    }

    #[test]
    fn test_unit_operations_error() {
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Unit, &Value::Unit);
        assert!(result.is_err());
    }

    // In operator not available in current AST
    // Removed test_in_operator

    #[test]
    fn test_bitwise_operations() {
        assert_eq!(evaluate_binary_op(&BinaryOp::BitwiseAnd, &Value::Int(12), &Value::Int(10)).unwrap(), Value::Int(8));
        assert_eq!(evaluate_binary_op(&BinaryOp::BitwiseOr, &Value::Int(12), &Value::Int(10)).unwrap(), Value::Int(14));
        assert_eq!(evaluate_binary_op(&BinaryOp::BitwiseXor, &Value::Int(12), &Value::Int(10)).unwrap(), Value::Int(6));
        assert_eq!(evaluate_binary_op(&BinaryOp::LeftShift, &Value::Int(3), &Value::Int(2)).unwrap(), Value::Int(12));
        // RightShift not in current BinaryOp enum
    }

    // FloorDivide operator not available in current AST
    // Removed test_floor_division

    // Is and IsNot operators not available in current AST
    // Removed test_is_operator and test_is_not_operator

    // Range and RangeInclusive operators not available in current AST
    // Removed test_range_operator and test_inclusive_range_operator
}