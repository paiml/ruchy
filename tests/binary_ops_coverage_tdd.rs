//! TDD tests for runtime/binary_ops.rs - achieving 90%+ coverage
//! QDD Metrics Target:
//! - Line Coverage: ≥90%
//! - Branch Coverage: ≥85%
//! - All public APIs: 100%

use ruchy::frontend::ast::BinaryOp;
use ruchy::runtime::eval_operations::eval_binary_op;
use ruchy::runtime::Value;
use std::rc::Rc;

// ============================================================================
// Arithmetic Operations Tests
// ============================================================================

#[test]
fn test_add_integers() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(8));
}

#[test]
fn test_add_floats() {
    let lhs = Value::Float(2.5);
    let rhs = Value::Float(1.5);
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_add_strings() {
    let lhs = Value::String(Rc::new("Hello ".to_string()));
    let rhs = Value::String(Rc::new("World".to_string()));
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String(Rc::new("Hello World".to_string())));
}

#[test]
fn test_add_lists() {
    let lhs = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let rhs = Value::Array(Rc::new(vec![Value::Integer(3), Value::Integer(4)]));
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(
        result,
        Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4)
        ]))
    );
}

#[test]
fn test_add_incompatible_types() {
    let lhs = Value::Integer(5);
    let rhs = Value::String(Rc::new("test".to_string()));
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot add"));
}

#[test]
fn test_subtract_integers() {
    let lhs = Value::Integer(10);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Subtract, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn test_subtract_floats() {
    let lhs = Value::Float(5.5);
    let rhs = Value::Float(2.5);
    let result = eval_binary_op(BinaryOp::Subtract, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_subtract_negative_result() {
    let lhs = Value::Integer(3);
    let rhs = Value::Integer(10);
    let result = eval_binary_op(BinaryOp::Subtract, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(-7));
}

#[test]
fn test_subtract_incompatible() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Integer(1);
    let result = eval_binary_op(BinaryOp::Subtract, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_multiply_integers() {
    let lhs = Value::Integer(6);
    let rhs = Value::Integer(7);
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_multiply_floats() {
    let lhs = Value::Float(2.5);
    let rhs = Value::Float(4.0);
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(10.0));
}

#[test]
fn test_multiply_string_by_int() {
    let lhs = Value::String(Rc::new("ab".to_string()));
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String(Rc::new("ababab".to_string())));
}

#[test]
fn test_multiply_int_by_string() {
    let lhs = Value::Integer(2);
    let rhs = Value::String(Rc::new("xy".to_string()));
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String(Rc::new("xyxy".to_string())));
}

#[test]
fn test_multiply_string_negative_times() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Integer(-1);
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("negative times"));
}

#[test]
fn test_multiply_incompatible() {
    let lhs = Value::Bool(true);
    let rhs = Value::Integer(2);
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_divide_integers() {
    let lhs = Value::Integer(20);
    let rhs = Value::Integer(4);
    let result = eval_binary_op(BinaryOp::Divide, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_divide_integers_truncation() {
    let lhs = Value::Integer(7);
    let rhs = Value::Integer(2);
    let result = eval_binary_op(BinaryOp::Divide, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(3)); // Integer division truncates
}

#[test]
fn test_divide_floats() {
    let lhs = Value::Float(7.5);
    let rhs = Value::Float(2.5);
    let result = eval_binary_op(BinaryOp::Divide, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_divide_by_zero_int() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(0);
    let result = eval_binary_op(BinaryOp::Divide, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
}

#[test]
fn test_divide_by_zero_float() {
    let lhs = Value::Float(5.0);
    let rhs = Value::Float(0.0);
    let result = eval_binary_op(BinaryOp::Divide, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
}

#[test]
fn test_divide_incompatible() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Integer(2);
    let result = eval_binary_op(BinaryOp::Divide, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_modulo_positive() {
    let lhs = Value::Integer(10);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Modulo, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_modulo_negative() {
    let lhs = Value::Integer(-10);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Modulo, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(-1));
}

#[test]
fn test_modulo_by_zero() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(0);
    let result = eval_binary_op(BinaryOp::Modulo, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Modulo by zero"));
}

#[test]
fn test_modulo_incompatible() {
    let lhs = Value::Float(5.5);
    let rhs = Value::Float(2.0);
    let result = eval_binary_op(BinaryOp::Modulo, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_power_integers() {
    let lhs = Value::Integer(2);
    let rhs = Value::Integer(10);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(1024));
}

#[test]
fn test_power_integer_zero_exponent() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(0);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_power_integer_negative_exponent() {
    let lhs = Value::Integer(2);
    let rhs = Value::Integer(-1);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Negative exponent"));
}

#[test]
fn test_power_floats() {
    let lhs = Value::Float(2.0);
    let rhs = Value::Float(3.0);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(8.0));
}

#[test]
fn test_power_float_negative_exponent() {
    let lhs = Value::Float(2.0);
    let rhs = Value::Float(-1.0);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Float(0.5));
}

#[test]
fn test_power_incompatible() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Integer(2);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Comparison Operations Tests
// ============================================================================

#[test]
fn test_equal_integers() {
    let lhs = Value::Integer(42);
    let rhs = Value::Integer(42);
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let rhs = Value::Integer(43);
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_equal_floats() {
    let lhs = Value::Float(3.14);
    let rhs = Value::Float(3.14);
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_strings() {
    let lhs = Value::String(Rc::new("hello".to_string()));
    let rhs = Value::String(Rc::new("hello".to_string()));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_bools() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(true);
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_chars() {
    let lhs = Value::String(Rc::new("a".to_string()));
    let rhs = Value::String(Rc::new("a".to_string()));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_lists() {
    let lhs = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let rhs = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_lists_different() {
    let lhs = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let rhs = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(3)]));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_equal_tuples() {
    let lhs = Value::Tuple(Rc::new(vec![
        Value::Integer(1),
        Value::String(Rc::new("a".to_string())),
    ]));
    let rhs = Value::Tuple(Rc::new(vec![
        Value::Integer(1),
        Value::String(Rc::new("a".to_string())),
    ]));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_unit() {
    let lhs = Value::Nil;
    let rhs = Value::Nil;
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_different_types() {
    let lhs = Value::Integer(42);
    let rhs = Value::String(Rc::new("42".to_string()));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_not_equal() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::NotEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let rhs = Value::Integer(5);
    let result = eval_binary_op(BinaryOp::NotEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_less_integers() {
    let lhs = Value::Integer(3);
    let rhs = Value::Integer(5);
    let result = eval_binary_op(BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let lhs = Value::Integer(5);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_less_floats() {
    let lhs = Value::Float(2.5);
    let rhs = Value::Float(3.5);
    let result = eval_binary_op(BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_less_strings() {
    let lhs = Value::String(Rc::new("apple".to_string()));
    let rhs = Value::String(Rc::new("banana".to_string()));
    let result = eval_binary_op(BinaryOp::Less, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_less_incompatible() {
    let lhs = Value::Integer(5);
    let rhs = Value::Bool(true);
    let result = eval_binary_op(BinaryOp::Less, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_less_equal() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(5);
    let result = eval_binary_op(BinaryOp::LessEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let rhs = Value::Integer(6);
    let result = eval_binary_op(BinaryOp::LessEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let rhs = Value::Integer(4);
    let result = eval_binary_op(BinaryOp::LessEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_greater() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::Greater, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let rhs = Value::Integer(7);
    let result = eval_binary_op(BinaryOp::Greater, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_greater_equal() {
    let lhs = Value::Float(3.14);
    let rhs = Value::Float(3.14);
    let result = eval_binary_op(BinaryOp::GreaterEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));

    let rhs = Value::Float(2.0);
    let result = eval_binary_op(BinaryOp::GreaterEqual, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Logical Operations Tests
// ============================================================================

#[test]
fn test_and_true_true() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(true);
    let result = eval_binary_op(BinaryOp::And, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_and_true_false() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(false);
    let result = eval_binary_op(BinaryOp::And, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_and_false_false() {
    let lhs = Value::Bool(false);
    let rhs = Value::Bool(false);
    let result = eval_binary_op(BinaryOp::And, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_and_incompatible() {
    let lhs = Value::Integer(1);
    let rhs = Value::Bool(true);
    let result = eval_binary_op(BinaryOp::And, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_or_false_false() {
    let lhs = Value::Bool(false);
    let rhs = Value::Bool(false);
    let result = eval_binary_op(BinaryOp::Or, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_or_true_false() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(false);
    let result = eval_binary_op(BinaryOp::Or, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_or_true_true() {
    let lhs = Value::Bool(true);
    let rhs = Value::Bool(true);
    let result = eval_binary_op(BinaryOp::Or, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_or_incompatible() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Bool(false);
    let result = eval_binary_op(BinaryOp::Or, &lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Bitwise Operations Tests
// ============================================================================

#[test]
fn test_bitwise_and() {
    let lhs = Value::Integer(0b1010);
    let rhs = Value::Integer(0b1100);
    let result = eval_binary_op(BinaryOp::BitwiseAnd, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(0b1000));
}

#[test]
fn test_bitwise_and_zero() {
    let lhs = Value::Integer(0xFF);
    let rhs = Value::Integer(0);
    let result = eval_binary_op(BinaryOp::BitwiseAnd, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_bitwise_and_incompatible() {
    let lhs = Value::Float(5.0);
    let rhs = Value::Integer(3);
    let result = eval_binary_op(BinaryOp::BitwiseAnd, &lhs, &rhs);
    assert!(result.is_err());
}

#[test]
fn test_bitwise_or() {
    let lhs = Value::Integer(0b1010);
    let rhs = Value::Integer(0b1100);
    let result = eval_binary_op(BinaryOp::BitwiseOr, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(0b1110));
}

#[test]
fn test_bitwise_or_all_ones() {
    let lhs = Value::Integer(0xFF);
    let rhs = Value::Integer(0xFF00);
    let result = eval_binary_op(BinaryOp::BitwiseOr, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(0xFFFF));
}

#[test]
fn test_bitwise_xor() {
    let lhs = Value::Integer(0b1010);
    let rhs = Value::Integer(0b1100);
    let result = eval_binary_op(BinaryOp::BitwiseXor, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(0b0110));
}

#[test]
fn test_bitwise_xor_self() {
    let lhs = Value::Integer(42);
    let rhs = Value::Integer(42);
    let result = eval_binary_op(BinaryOp::BitwiseXor, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_left_shift() {
    let lhs = Value::Integer(1);
    let rhs = Value::Integer(4);
    let result = eval_binary_op(BinaryOp::LeftShift, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(16));
}

#[test]
fn test_left_shift_zero() {
    let lhs = Value::Integer(42);
    let rhs = Value::Integer(0);
    let result = eval_binary_op(BinaryOp::LeftShift, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_left_shift_negative() {
    let lhs = Value::Integer(5);
    let rhs = Value::Integer(-1);
    let result = eval_binary_op(BinaryOp::LeftShift, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid shift amount"));
}

#[test]
fn test_left_shift_too_large() {
    let lhs = Value::Integer(1);
    let rhs = Value::Integer(64);
    let result = eval_binary_op(BinaryOp::LeftShift, &lhs, &rhs);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid shift amount"));
}

#[test]
fn test_left_shift_incompatible() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Integer(2);
    let result = eval_binary_op(BinaryOp::LeftShift, &lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Null Coalesce Tests
// ============================================================================

#[test]
fn test_null_coalesce_with_unit() {
    let lhs = Value::Nil;
    let rhs = Value::Integer(42);
    let result = eval_binary_op(BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_null_coalesce_with_value() {
    let lhs = Value::Integer(10);
    let rhs = Value::Integer(42);
    let result = eval_binary_op(BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn test_null_coalesce_string() {
    let lhs = Value::String(Rc::new("present".to_string()));
    let rhs = Value::String(Rc::new("default".to_string()));
    let result = eval_binary_op(BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String(Rc::new("present".to_string())));
}

#[test]
fn test_null_coalesce_chain() {
    let lhs = Value::Nil;
    let rhs = Value::Nil;
    let result = eval_binary_op(BinaryOp::NullCoalesce, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Nil);
}

// ============================================================================
// Edge Cases and Special Values
// ============================================================================

#[test]
fn test_add_empty_strings() {
    let lhs = Value::String(Rc::new("".to_string()));
    let rhs = Value::String(Rc::new("".to_string()));
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String(Rc::new("".to_string())));
}

#[test]
fn test_add_empty_lists() {
    let lhs = Value::Array(Rc::new(vec![]));
    let rhs = Value::Array(Rc::new(vec![]));
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Array(Rc::new(vec![])));
}

#[test]
fn test_multiply_string_by_zero() {
    let lhs = Value::String(Rc::new("test".to_string()));
    let rhs = Value::Integer(0);
    let result = eval_binary_op(BinaryOp::Multiply, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::String(Rc::new("".to_string())));
}

#[test]
fn test_power_one_base() {
    let lhs = Value::Integer(1);
    let rhs = Value::Integer(100);
    let result = eval_binary_op(BinaryOp::Power, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_float_equality_epsilon() {
    // Check that float equality uses epsilon comparison
    let lhs = Value::Float(1.0);
    let rhs = Value::Float(1.0 + f64::EPSILON / 2.0);
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_nested_list_equality() {
    let lhs = Value::Array(Rc::new(vec![
        Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)])),
        Value::Array(Rc::new(vec![Value::Integer(3)])),
    ]));
    let rhs = Value::Array(Rc::new(vec![
        Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)])),
        Value::Array(Rc::new(vec![Value::Integer(3)])),
    ]));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_list_length_mismatch() {
    let lhs = Value::Array(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let rhs = Value::Array(Rc::new(vec![Value::Integer(1)]));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_tuple_length_mismatch() {
    let lhs = Value::Tuple(Rc::new(vec![Value::Integer(1), Value::Integer(2)]));
    let rhs = Value::Tuple(Rc::new(vec![Value::Integer(1)]));
    let result = eval_binary_op(BinaryOp::Equal, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_large_integer_add() {
    // Check with large but safe integers
    let lhs = Value::Integer(1_000_000_000);
    let rhs = Value::Integer(2_000_000_000);
    let result = eval_binary_op(BinaryOp::Add, &lhs, &rhs).unwrap();
    assert_eq!(result, Value::Integer(3_000_000_000));
}

#[test]
fn test_all_comparison_ops_with_strings() {
    let apple = Value::String(Rc::new("apple".to_string()));
    let banana = Value::String(Rc::new("banana".to_string()));

    let less = eval_binary_op(BinaryOp::Less, &apple, &banana).unwrap();
    assert_eq!(less, Value::Bool(true));

    let less_eq = eval_binary_op(BinaryOp::LessEqual, &apple, &banana).unwrap();
    assert_eq!(less_eq, Value::Bool(true));

    let greater = eval_binary_op(BinaryOp::Greater, &apple, &banana).unwrap();
    assert_eq!(greater, Value::Bool(false));

    let greater_eq = eval_binary_op(BinaryOp::GreaterEqual, &apple, &banana).unwrap();
    assert_eq!(greater_eq, Value::Bool(false));
}
