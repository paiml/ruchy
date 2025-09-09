//! TDD tests for runtime/binary_ops.rs - achieving 90%+ coverage
//! QDD Metrics Target: 
//! - Line Coverage: ≥90%
//! - Branch Coverage: ≥85%
//! - All public APIs: 100%

use ruchy::runtime::binary_ops::evaluate_binary_op;
use ruchy::runtime::Value;
use ruchy::frontend::ast::BinaryOp;

// ============================================================================
// Arithmetic Operations Tests
// ============================================================================

#[test]
fn test_add_integers() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(8));
}

#[test]
fn test_add_floats() {
    let lhs = Value::Float(2.5);
    let rhs = Value::Float(1.5);
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_add_strings() {
    let lhs = Value::String("Hello ".to_string());
    let rhs = Value::String("World".to_string());
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String("Hello World".to_string()));
}

#[test]
fn test_add_lists() {
    let lhs = Value::List(vec![Value::Int(1), Value::Int(2)]);
    let rhs = Value::List(vec![Value::Int(3), Value::Int(4)]);
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::List(vec![
        Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)
    ]));
}

#[test]
fn test_add_incompatible_types() {
    let lhs = Value::Int(5);
    let rhs = Value::String("test".to_string());
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot add"));
}

#[test]
fn test_subtract_integers() {
    let lhs = Value::Int(10);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Subtract, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(7));
}

#[test]
fn test_subtract_floats() {
    let lhs = Value::Float(5.5);
    let rhs = Value::Float(2.5);
    let result = evaluate_binary_op(&BinaryOp::Subtract, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_subtract_negative_result() {
    let lhs = Value::Int(3);
    let rhs = Value::Int(10);
    let result = evaluate_binary_op(&BinaryOp::Subtract, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(-7));
}

#[test]
fn test_subtract_incompatible() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Int(1);
    let result = evaluate_binary_op(&BinaryOp::Subtract, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_multiply_integers() {
    let lhs = Value::Int(6);
    let rhs = Value::Int(7);
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_multiply_floats() {
    let lhs = Value::Float(2.5);
    let rhs = Value::Float(4.0);
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(10.0));
}

#[test]
fn test_multiply_string_by_int() {
    let lhs = Value::String("ab".to_string());
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String("ababab".to_string()));
}

#[test]
fn test_multiply_int_by_string() {
    let lhs = Value::Int(2);
    let rhs = Value::String("xy".to_string());
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String("xyxy".to_string()));
}

#[test]
fn test_multiply_string_negative_times() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Int(-1);
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("negative times"));
}

#[test]
fn test_multiply_incompatible() {
    let lhs = Value::Bool(true);
    let rhs = Value::Int(2);
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_divide_integers() {
    let lhs = Value::Int(20);
    let rhs = Value::Int(4);
    let result = evaluate_binary_op(&BinaryOp::Divide, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_divide_integers_truncation() {
    let lhs = Value::Int(7);
    let rhs = Value::Int(2);
    let result = evaluate_binary_op(&BinaryOp::Divide, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(3)); // Integer division truncates
}

#[test]
fn test_divide_floats() {
    let lhs = Value::Float(7.5);
    let rhs = Value::Float(2.5);
    let result = evaluate_binary_op(&BinaryOp::Divide, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_divide_by_zero_int() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(0);
    let result = evaluate_binary_op(&BinaryOp::Divide, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
}

#[test]
fn test_divide_by_zero_float() {
    let lhs = Value::Float(5.0);
    let rhs = Value::Float(0.0);
    let result = evaluate_binary_op(&BinaryOp::Divide, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
}

#[test]
fn test_divide_incompatible() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Int(2);
    let result = evaluate_binary_op(&BinaryOp::Divide, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_modulo_positive() {
    let lhs = Value::Int(10);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Modulo, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_modulo_negative() {
    let lhs = Value::Int(-10);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Modulo, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(-1));
}

#[test]
fn test_modulo_by_zero() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(0);
    let result = evaluate_binary_op(&BinaryOp::Modulo, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Modulo by zero"));
}

#[test]
fn test_modulo_incompatible() {
    let lhs = Value::Float(5.5);
    let rhs = Value::Float(2.0);
    let result = evaluate_binary_op(&BinaryOp::Modulo, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_power_integers() {
    let lhs = Value::Int(2);
    let rhs = Value::Int(10);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(1024));
}

#[test]
fn test_power_integer_zero_exponent() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(0);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_power_integer_negative_exponent() {
    let lhs = Value::Int(2);
    let rhs = Value::Int(-1);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Negative exponent"));
}

#[test]
fn test_power_floats() {
    let lhs = Value::Float(2.0);
    let rhs = Value::Float(3.0);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(8.0));
}

#[test]
fn test_power_float_negative_exponent() {
    let lhs = Value::Float(2.0);
    let rhs = Value::Float(-1.0);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(0.5));
}

#[test]
fn test_power_incompatible() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Int(2);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Comparison Operations Tests
// ============================================================================

#[test]
fn test_equal_integers() {
    let lhs = Value::Int(42);
    let rhs = Value::Int(42);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let rhs = Value::Int(43);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_equal_floats() {
    let lhs = Value::Float(3.14);
    let rhs = Value::Float(3.14);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_strings() {
    let lhs = Value::String("hello".to_string());
    let rhs = Value::String("hello".to_string());
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_bools() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(true);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_chars() {
    let lhs = Value::Char('a');
    let rhs = Value::Char('a');
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_lists() {
    let lhs = Value::List(vec![Value::Int(1), Value::Int(2)]);
    let rhs = Value::List(vec![Value::Int(1), Value::Int(2)]);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_lists_different() {
    let lhs = Value::List(vec![Value::Int(1), Value::Int(2)]);
    let rhs = Value::List(vec![Value::Int(1), Value::Int(3)]);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_equal_tuples() {
    let lhs = Value::Tuple(vec![Value::Int(1), Value::String("a".to_string())]);
    let rhs = Value::Tuple(vec![Value::Int(1), Value::String("a".to_string())]);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_unit() {
    let lhs = Value::Unit;
    let rhs = Value::Unit;
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_different_types() {
    let lhs = Value::Int(42);
    let rhs = Value::String("42".to_string());
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_not_equal() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::NotEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let rhs = Value::Int(5);
    let result = evaluate_binary_op(&BinaryOp::NotEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_less_integers() {
    let lhs = Value::Int(3);
    let rhs = Value::Int(5);
    let result = evaluate_binary_op(&BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let lhs = Value::Int(5);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_less_floats() {
    let lhs = Value::Float(2.5);
    let rhs = Value::Float(3.5);
    let result = evaluate_binary_op(&BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_less_strings() {
    let lhs = Value::String("apple".to_string());
    let rhs = Value::String("banana".to_string());
    let result = evaluate_binary_op(&BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_less_incompatible() {
    let lhs = Value::Int(5);
    let rhs = Value::Bool(true);
    let result = evaluate_binary_op(&BinaryOp::Less, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_less_equal() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(5);
    let result = evaluate_binary_op(&BinaryOp::LessEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let rhs = Value::Int(6);
    let result = evaluate_binary_op(&BinaryOp::LessEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let rhs = Value::Int(4);
    let result = evaluate_binary_op(&BinaryOp::LessEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_greater() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::Greater, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let rhs = Value::Int(7);
    let result = evaluate_binary_op(&BinaryOp::Greater, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_greater_equal() {
    let lhs = Value::Float(3.14);
    let rhs = Value::Float(3.14);
    let result = evaluate_binary_op(&BinaryOp::GreaterEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let rhs = Value::Float(2.0);
    let result = evaluate_binary_op(&BinaryOp::GreaterEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Logical Operations Tests
// ============================================================================

#[test]
fn test_and_true_true() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(true);
    let result = evaluate_binary_op(&BinaryOp::And, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_and_true_false() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(false);
    let result = evaluate_binary_op(&BinaryOp::And, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_and_false_false() {
    let lhs = Value::Bool(false);
    let rhs = Value::Bool(false);
    let result = evaluate_binary_op(&BinaryOp::And, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_and_incompatible() {
    let lhs = Value::Int(1);
    let rhs = Value::Bool(true);
    let result = evaluate_binary_op(&BinaryOp::And, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_or_false_false() {
    let lhs = Value::Bool(false);
    let rhs = Value::Bool(false);
    let result = evaluate_binary_op(&BinaryOp::Or, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_or_true_false() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(false);
    let result = evaluate_binary_op(&BinaryOp::Or, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_or_true_true() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(true);
    let result = evaluate_binary_op(&BinaryOp::Or, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_or_incompatible() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Bool(false);
    let result = evaluate_binary_op(&BinaryOp::Or, &lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Bitwise Operations Tests
// ============================================================================

#[test]
fn test_bitwise_and() {
    let lhs = Value::Int(0b1010);
    let rhs = Value::Int(0b1100);
    let result = evaluate_binary_op(&BinaryOp::BitwiseAnd, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(0b1000));
}

#[test]
fn test_bitwise_and_zero() {
    let lhs = Value::Int(0xFF);
    let rhs = Value::Int(0);
    let result = evaluate_binary_op(&BinaryOp::BitwiseAnd, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_bitwise_and_incompatible() {
    let lhs = Value::Float(5.0);
    let rhs = Value::Int(3);
    let result = evaluate_binary_op(&BinaryOp::BitwiseAnd, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_bitwise_or() {
    let lhs = Value::Int(0b1010);
    let rhs = Value::Int(0b1100);
    let result = evaluate_binary_op(&BinaryOp::BitwiseOr, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(0b1110));
}

#[test]
fn test_bitwise_or_all_ones() {
    let lhs = Value::Int(0xFF);
    let rhs = Value::Int(0xFF00);
    let result = evaluate_binary_op(&BinaryOp::BitwiseOr, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(0xFFFF));
}

#[test]
fn test_bitwise_xor() {
    let lhs = Value::Int(0b1010);
    let rhs = Value::Int(0b1100);
    let result = evaluate_binary_op(&BinaryOp::BitwiseXor, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(0b0110));
}

#[test]
fn test_bitwise_xor_self() {
    let lhs = Value::Int(42);
    let rhs = Value::Int(42);
    let result = evaluate_binary_op(&BinaryOp::BitwiseXor, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_left_shift() {
    let lhs = Value::Int(1);
    let rhs = Value::Int(4);
    let result = evaluate_binary_op(&BinaryOp::LeftShift, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(16));
}

#[test]
fn test_left_shift_zero() {
    let lhs = Value::Int(42);
    let rhs = Value::Int(0);
    let result = evaluate_binary_op(&BinaryOp::LeftShift, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_left_shift_negative() {
    let lhs = Value::Int(5);
    let rhs = Value::Int(-1);
    let result = evaluate_binary_op(&BinaryOp::LeftShift, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid shift amount"));
}

#[test]
fn test_left_shift_too_large() {
    let lhs = Value::Int(1);
    let rhs = Value::Int(64);
    let result = evaluate_binary_op(&BinaryOp::LeftShift, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid shift amount"));
}

#[test]
fn test_left_shift_incompatible() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Int(2);
    let result = evaluate_binary_op(&BinaryOp::LeftShift, &lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Null Coalesce Tests
// ============================================================================

#[test]
fn test_null_coalesce_with_unit() {
    let lhs = Value::Unit;
    let rhs = Value::Int(42);
    let result = evaluate_binary_op(&BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_null_coalesce_with_value() {
    let lhs = Value::Int(10);
    let rhs = Value::Int(42);
    let result = evaluate_binary_op(&BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_null_coalesce_string() {
    let lhs = Value::String("present".to_string());
    let rhs = Value::String("default".to_string());
    let result = evaluate_binary_op(&BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String("present".to_string()));
}

#[test]
fn test_null_coalesce_chain() {
    let lhs = Value::Unit;
    let rhs = Value::Unit;
    let result = evaluate_binary_op(&BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Unit);
}

// ============================================================================
// Edge Cases and Special Values
// ============================================================================

#[test]
fn test_add_empty_strings() {
    let lhs = Value::String("".to_string());
    let rhs = Value::String("".to_string());
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String("".to_string()));
}

#[test]
fn test_add_empty_lists() {
    let lhs = Value::List(vec![]);
    let rhs = Value::List(vec![]);
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::List(vec![]));
}

#[test]
fn test_multiply_string_by_zero() {
    let lhs = Value::String("test".to_string());
    let rhs = Value::Int(0);
    let result = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String("".to_string()));
}

#[test]
fn test_power_one_base() {
    let lhs = Value::Int(1);
    let rhs = Value::Int(100);
    let result = evaluate_binary_op(&BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_float_equality_epsilon() {
    // Test that float equality uses epsilon comparison
    let lhs = Value::Float(1.0);
    let rhs = Value::Float(1.0 + f64::EPSILON / 2.0);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_nested_list_equality() {
    let lhs = Value::List(vec![
        Value::List(vec![Value::Int(1), Value::Int(2)]),
        Value::List(vec![Value::Int(3)]),
    ]);
    let rhs = Value::List(vec![
        Value::List(vec![Value::Int(1), Value::Int(2)]),
        Value::List(vec![Value::Int(3)]),
    ]);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_list_length_mismatch() {
    let lhs = Value::List(vec![Value::Int(1), Value::Int(2)]);
    let rhs = Value::List(vec![Value::Int(1)]);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_tuple_length_mismatch() {
    let lhs = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
    let rhs = Value::Tuple(vec![Value::Int(1)]);
    let result = evaluate_binary_op(&BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_large_integer_add() {
    // Test with large but safe integers
    let lhs = Value::Int(1_000_000_000);
    let rhs = Value::Int(2_000_000_000);
    let result = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Int(3_000_000_000));
}

#[test]
fn test_all_comparison_ops_with_strings() {
    let apple = Value::String("apple".to_string());
    let banana = Value::String("banana".to_string());
    
    let less = evaluate_binary_op(&BinaryOp::Less, &apple, &banana).unwrap();
    assert_eq!(less, Value::Bool(true));
    
    let less_eq = evaluate_binary_op(&BinaryOp::LessEqual, &apple, &banana).unwrap();
    assert_eq!(less_eq, Value::Bool(true));
    
    let greater = evaluate_binary_op(&BinaryOp::Greater, &apple, &banana).unwrap();
    assert_eq!(greater, Value::Bool(false));
    
    let greater_eq = evaluate_binary_op(&BinaryOp::GreaterEqual, &apple, &banana).unwrap();
    assert_eq!(greater_eq, Value::Bool(false));
}