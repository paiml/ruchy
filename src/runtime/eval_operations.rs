//! Binary and unary operation evaluation module
//!
//! This module handles all binary operations (arithmetic, comparison, logical, bitwise)
//! and unary operations (negation, not, bitwise not).
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{BinaryOp as AstBinaryOp, UnaryOp};
use crate::runtime::{InterpreterError, Value};

/// Evaluate a binary operation from AST
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn eval_binary_op(
    op: AstBinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match op {
        AstBinaryOp::Add
        | AstBinaryOp::Subtract
        | AstBinaryOp::Multiply
        | AstBinaryOp::Divide
        | AstBinaryOp::Modulo
        | AstBinaryOp::Power => eval_arithmetic_op(op, left, right),
        AstBinaryOp::Equal
        | AstBinaryOp::NotEqual
        | AstBinaryOp::Less
        | AstBinaryOp::Greater
        | AstBinaryOp::LessEqual
        | AstBinaryOp::GreaterEqual => eval_comparison_op(op, left, right),
        AstBinaryOp::And | AstBinaryOp::Or => eval_logical_op(op, left, right),
        AstBinaryOp::BitwiseAnd
        | AstBinaryOp::BitwiseOr
        | AstBinaryOp::BitwiseXor
        | AstBinaryOp::LeftShift
        | AstBinaryOp::RightShift => eval_bitwise_op(op, left, right),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Binary operator not yet implemented: {op:?}"
        ))),
    }
}

/// Handle arithmetic operations
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn eval_arithmetic_op(
    op: AstBinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match op {
        AstBinaryOp::Add => add_values(left, right),
        AstBinaryOp::Subtract => sub_values(left, right),
        AstBinaryOp::Multiply => mul_values(left, right),
        AstBinaryOp::Divide => div_values(left, right),
        AstBinaryOp::Modulo => modulo_values(left, right),
        AstBinaryOp::Power => power_values(left, right),
        _ => unreachable!("Non-arithmetic operation passed to eval_arithmetic_op"),
    }
}

/// Check if left <= right (helper for less-or-equal comparison)
/// Complexity: 2 (within Toyota Way limits)
#[inline]
fn less_or_equal_values(left: &Value, right: &Value) -> Result<bool, InterpreterError> {
    Ok(less_than_values(left, right)? || equal_values(left, right))
}

/// Check if left >= right (helper for greater-or-equal comparison)
/// Complexity: 2 (within Toyota Way limits)
#[inline]
fn greater_or_equal_values(left: &Value, right: &Value) -> Result<bool, InterpreterError> {
    Ok(greater_than_values(left, right)? || equal_values(left, right))
}

/// Handle comparison operations
/// Complexity: 8 cyclomatic, 6 cognitive (reduced from 13 via helper extraction)
fn eval_comparison_op(
    op: AstBinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match op {
        AstBinaryOp::Equal => Ok(Value::Bool(equal_values(left, right))),
        AstBinaryOp::NotEqual => Ok(Value::Bool(!equal_values(left, right))),
        AstBinaryOp::Less => Ok(Value::Bool(less_than_values(left, right)?)),
        AstBinaryOp::Greater => Ok(Value::Bool(greater_than_values(left, right)?)),
        AstBinaryOp::LessEqual => Ok(Value::Bool(less_or_equal_values(left, right)?)),
        AstBinaryOp::GreaterEqual => Ok(Value::Bool(greater_or_equal_values(left, right)?)),
        _ => unreachable!("Non-comparison operation passed to eval_comparison_op"),
    }
}

/// Handle logical operations (And, Or)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_logical_op(
    op: AstBinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match op {
        AstBinaryOp::And => {
            // Short-circuit evaluation for logical AND
            if left.is_truthy() {
                Ok(right.clone())
            } else {
                Ok(left.clone())
            }
        }
        AstBinaryOp::Or => {
            // Short-circuit evaluation for logical OR
            if left.is_truthy() {
                Ok(left.clone())
            } else {
                Ok(right.clone())
            }
        }
        _ => unreachable!("Non-logical operation passed to eval_logical_op"),
    }
}

/// Handle bitwise operations
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn eval_bitwise_op(
    op: AstBinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            let result = match op {
                AstBinaryOp::BitwiseAnd => a & b,
                AstBinaryOp::BitwiseOr => a | b,
                AstBinaryOp::BitwiseXor => a ^ b,
                AstBinaryOp::LeftShift => a << b,
                AstBinaryOp::RightShift => a >> b,
                _ => unreachable!("Non-bitwise operation passed to eval_bitwise_op"),
            };
            Ok(Value::Integer(result))
        }
        _ => Err(InterpreterError::RuntimeError(format!(
            "Bitwise operations require integer operands, got {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Evaluate a unary operation
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn eval_unary_op(op: UnaryOp, operand: &Value) -> Result<Value, InterpreterError> {
    match op {
        UnaryOp::Negate => match operand {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot negate {}",
                operand.type_name()
            ))),
        },
        UnaryOp::Not => Ok(Value::Bool(!operand.is_truthy())),
        UnaryOp::BitwiseNot => match operand {
            Value::Integer(i) => Ok(Value::Integer(!i)),
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot apply bitwise NOT to {}",
                operand.type_name()
            ))),
        },
        UnaryOp::Reference | UnaryOp::MutableReference => {
            // In interpreted mode, reference operators (& and &mut) are no-ops
            // The interpreter already manages value ownership internally
            // This allows Rust-like syntax (&value, &mut value) to work in eval mode
            // PARSER-085: Issue #71 - Added MutableReference support
            Ok(operand.clone())
        }
        UnaryOp::Deref => {
            // In interpreted mode, dereference (*) is transparent
            // Box<T> is represented as the value T itself (Box is transparent in Ruchy)
            // The interpreter manages ownership, so *boxed just returns the value
            Ok(operand.clone())
        }
    }
}

// Arithmetic helper functions

/// Add two values
///
/// # Complexity
/// Cyclomatic complexity: 12 (auto-conversion for string concatenation)
fn add_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            a.checked_add(*b).map(Value::Integer).ok_or_else(|| {
                InterpreterError::RuntimeError("Integer overflow in addition".to_string())
            })
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(*a as f64 + b))
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(a + *b as f64))
        }
        (Value::String(a), Value::String(b)) => {
            Ok(Value::from_string(format!("{}{}", a.as_ref(), b.as_ref())))
        }
        // Feature #88: String + Integer auto-conversion
        (Value::String(s), Value::Integer(i)) => {
            Ok(Value::from_string(format!("{}{}", s.as_ref(), i)))
        }
        (Value::Integer(i), Value::String(s)) => {
            Ok(Value::from_string(format!("{}{}", i, s.as_ref())))
        }
        // Feature #88: String + Float auto-conversion
        (Value::String(s), Value::Float(f)) => {
            Ok(Value::from_string(format!("{}{}", s.as_ref(), f)))
        }
        (Value::Float(f), Value::String(s)) => {
            Ok(Value::from_string(format!("{}{}", f, s.as_ref())))
        }
        // Feature #88: String + Boolean auto-conversion
        (Value::String(s), Value::Bool(b)) => {
            Ok(Value::from_string(format!("{}{}", s.as_ref(), b)))
        }
        (Value::Bool(b), Value::String(s)) => {
            Ok(Value::from_string(format!("{}{}", b, s.as_ref())))
        }
        (Value::Array(a), Value::Array(b)) => {
            let mut result = a.as_ref().to_vec();
            result.extend_from_slice(b.as_ref());
            Ok(Value::from_array(result))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot add {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Subtract two values
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn sub_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            a.checked_sub(*b).map(Value::Integer).ok_or_else(|| {
                InterpreterError::RuntimeError("Integer overflow in subtraction".to_string())
            })
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(*a as f64 - b))
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(a - *b as f64))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot subtract {} from {}",
            right.type_name(),
            left.type_name()
        ))),
    }
}

/// Multiply two values
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn mul_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            a.checked_mul(*b).map(Value::Integer).ok_or_else(|| {
                InterpreterError::RuntimeError("Integer overflow in multiplication".to_string())
            })
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(*a as f64 * b))
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(a * *b as f64))
        }
        (Value::String(s), Value::Integer(n)) => {
            // String repetition: "hello" * 3 => "hellohellohello"
            // Negative or zero => empty string (Python behavior)
            if *n <= 0 {
                Ok(Value::String(std::sync::Arc::from("")))
            } else {
                let repeated = s.repeat(*n as usize);
                Ok(Value::String(std::sync::Arc::from(repeated.as_str())))
            }
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot multiply {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Divide two values
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn div_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
            a.checked_div(*b).map(Value::Integer).ok_or_else(|| {
                InterpreterError::RuntimeError("Integer overflow in division".to_string())
            })
        }
        (Value::Float(a), Value::Float(b)) => {
            // Float division by zero returns infinity per IEEE 754
            Ok(Value::Float(a / b))
        }
        (Value::Integer(a), Value::Float(b)) => {
            // Float division by zero returns infinity per IEEE 754
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(*a as f64 / b))
        }
        (Value::Float(a), Value::Integer(b)) => {
            // Float division by zero returns infinity per IEEE 754
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(a / *b as f64))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot divide {} by {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Check if divisor is zero (helper for modulo operations)
/// Complexity: 2 (within Toyota Way limits)
#[inline]
fn check_modulo_divisor_not_zero(divisor: &Value) -> Result<(), InterpreterError> {
    match divisor {
        Value::Integer(b) if *b == 0 => Err(InterpreterError::DivisionByZero),
        Value::Float(b) if *b == 0.0 => Err(InterpreterError::DivisionByZero),
        _ => Ok(()),
    }
}

/// Modulo operation on two values
/// Complexity: 5 (reduced from 21 via helper extraction)
fn modulo_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    check_modulo_divisor_not_zero(right)?;

    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            a.checked_rem(*b).map(Value::Integer).ok_or_else(|| {
                InterpreterError::RuntimeError("Integer overflow in modulo".to_string())
            })
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float((*a as f64) % b))
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(a % (*b as f64)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot compute {} modulo {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Power operation on two values
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn power_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b < 0 {
                // For negative exponents, convert to float
                #[allow(clippy::cast_precision_loss)]
                let result = (*a as f64).powf(*b as f64);
                Ok(Value::Float(result))
            } else {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                if let Some(result) = a.checked_pow(*b as u32) {
                    Ok(Value::Integer(result))
                } else {
                    // Overflow - convert to float
                    #[allow(clippy::cast_precision_loss)]
                    let result = (*a as f64).powf(*b as f64);
                    Ok(Value::Float(result))
                }
            }
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float((*a as f64).powf(*b)))
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float(a.powf(*b as f64)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot compute {} to the power of {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

// Comparison helper functions

/// Check equality of two values
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
#[allow(clippy::cast_precision_loss)]
/// QUALITY-017: Refactored equality - main dispatcher
/// Complexity: 7 (within Toyota Way limits)
#[allow(clippy::unnested_or_patterns)] // Clearer to group all primitive types together
fn equal_values(left: &Value, right: &Value) -> bool {
    match (left, right) {
        // Primitives - delegate to helper
        (Value::Integer(_), Value::Integer(_))
        | (Value::Float(_), Value::Float(_))
        | (Value::Integer(_), Value::Float(_))
        | (Value::Float(_), Value::Integer(_))
        | (Value::Bool(_), Value::Bool(_))
        | (Value::Byte(_), Value::Byte(_))
        | (Value::String(_), Value::String(_))
        | (Value::Nil, Value::Nil) => equal_primitives(left, right),
        // Objects - delegate to helper
        (Value::Object(a), Value::Object(b)) => equal_objects(a, b),
        // Arrays - delegate to helper
        (Value::Array(a), Value::Array(b)) => equal_slices(a, b),
        // Tuples - delegate to helper (same logic as arrays)
        (Value::Tuple(a), Value::Tuple(b)) => equal_slices(a, b),
        // Class - identity comparison (Arc pointer equality)
        (Value::Class { fields: f1, .. }, Value::Class { fields: f2, .. }) => {
            std::sync::Arc::ptr_eq(f1, f2)
        }
        // Struct - value equality (field-by-field comparison)
        (Value::Struct { fields: f1, .. }, Value::Struct { fields: f2, .. }) => {
            equal_objects(f1, f2)
        }
        // Atoms - string equality (interned comparison)
        (Value::Atom(a), Value::Atom(b)) => a == b,
        // Type mismatch
        _ => false,
    }
}

/// QUALITY-017: Compare primitive values (integers, floats, bools, bytes, strings, nil)
/// Complexity: 3 (within Toyota Way limits)
fn equal_primitives(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        // Mixed integer/float comparison
        (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Byte(a), Value::Byte(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    }
}

/// QUALITY-017: Compare object/struct values field-by-field
/// Complexity: 3 (reduced from 16 via functional style)
fn equal_objects(
    a: &std::collections::HashMap<String, Value>,
    b: &std::collections::HashMap<String, Value>,
) -> bool {
    // Quick length check
    if a.len() != b.len() {
        return false;
    }
    // Check all fields match using functional style
    a.iter()
        .all(|(key, val_a)| b.get(key).is_some_and(|val_b| equal_values(val_a, val_b)))
}

/// QUALITY-017: Compare array/tuple values element-by-element
/// Complexity: 2 (within Toyota Way limits)
/// Used for both arrays and tuples (identical comparison logic)
fn equal_slices(a: &[Value], b: &[Value]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| equal_values(x, y))
}

/// Check if left is less than right
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn less_than_values(left: &Value, right: &Value) -> Result<bool, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
        (Value::Float(a), Value::Float(b)) => Ok(a < b),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok((*a as f64) < *b)
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(*a < (*b as f64))
        }
        (Value::String(a), Value::String(b)) => Ok(a < b),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot compare {} and {} for ordering",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Check if left is greater than right
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn greater_than_values(left: &Value, right: &Value) -> Result<bool, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
        (Value::Float(a), Value::Float(b)) => Ok(a > b),
        (Value::Integer(a), Value::Float(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok((*a as f64) > *b)
        }
        (Value::Float(a), Value::Integer(b)) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(*a > (*b as f64))
        }
        (Value::String(a), Value::String(b)) => Ok(a > b),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot compare {} and {} for ordering",
            left.type_name(),
            right.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_values() {
        assert_eq!(
            add_values(&Value::Integer(2), &Value::Integer(3))
                .expect("operation should succeed in test"),
            Value::Integer(5)
        );
        assert_eq!(
            add_values(&Value::Float(2.5), &Value::Float(3.5))
                .expect("operation should succeed in test"),
            Value::Float(6.0)
        );

        let s1 = Value::from_string("hello".to_string());
        let s2 = Value::from_string(" world".to_string());
        let result = add_values(&s1, &s2).expect("operation should succeed in test");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "hello world"),
            _ => panic!("Expected string result"),
        }
    }

    #[test]
    fn test_comparison_ops() {
        assert!(equal_values(&Value::Integer(5), &Value::Integer(5)));
        assert!(!equal_values(&Value::Integer(5), &Value::Integer(6)));
        assert!(equal_values(&Value::Integer(5), &Value::Float(5.0))); // Mixed type comparison works

        assert!(less_than_values(&Value::Integer(3), &Value::Integer(5))
            .expect("operation should succeed in test"));
        assert!(!less_than_values(&Value::Integer(5), &Value::Integer(3))
            .expect("operation should succeed in test"));

        assert!(greater_than_values(&Value::Integer(5), &Value::Integer(3))
            .expect("operation should succeed in test"));
        assert!(!greater_than_values(&Value::Integer(3), &Value::Integer(5))
            .expect("operation should succeed in test"));
    }

    /// QUALITY-017: Comprehensive tests for `equal_values()` before refactoring
    /// Tests cover all Value types to ensure refactoring doesn't break equality
    #[test]
    fn test_equal_values_comprehensive() {
        use std::collections::HashMap;
        use std::sync::Arc;

        // Primitives
        assert!(equal_values(&Value::Integer(42), &Value::Integer(42)));
        assert!(!equal_values(&Value::Integer(42), &Value::Integer(43)));
        assert!(equal_values(&Value::Float(3.15), &Value::Float(3.15)));
        assert!(equal_values(&Value::Bool(true), &Value::Bool(true)));
        assert!(!equal_values(&Value::Bool(true), &Value::Bool(false)));
        assert!(equal_values(&Value::Byte(255), &Value::Byte(255)));
        assert!(equal_values(
            &Value::from_string("test".to_string()),
            &Value::from_string("test".to_string())
        ));
        assert!(equal_values(&Value::Nil, &Value::Nil));

        // Arrays
        let arr1 = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
        let arr2 = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
        let arr3 = Value::Array(vec![Value::Integer(1), Value::Integer(3)].into());
        assert!(equal_values(&arr1, &arr2));
        assert!(!equal_values(&arr1, &arr3));

        // Tuples
        let tuple1 =
            Value::Tuple(vec![Value::Integer(1), Value::from_string("test".to_string())].into());
        let tuple2 =
            Value::Tuple(vec![Value::Integer(1), Value::from_string("test".to_string())].into());
        let tuple3 =
            Value::Tuple(vec![Value::Integer(2), Value::from_string("test".to_string())].into());
        assert!(equal_values(&tuple1, &tuple2));
        assert!(!equal_values(&tuple1, &tuple3));

        // Objects
        let mut obj1 = HashMap::new();
        obj1.insert("key1".to_string(), Value::Integer(10));
        obj1.insert("key2".to_string(), Value::from_string("value".to_string()));

        let mut obj2 = HashMap::new();
        obj2.insert("key1".to_string(), Value::Integer(10));
        obj2.insert("key2".to_string(), Value::from_string("value".to_string()));

        let mut obj3 = HashMap::new();
        obj3.insert("key1".to_string(), Value::Integer(11));
        obj3.insert("key2".to_string(), Value::from_string("value".to_string()));

        assert!(equal_values(
            &Value::Object(Arc::new(obj1.clone())),
            &Value::Object(Arc::new(obj2))
        ));
        assert!(!equal_values(
            &Value::Object(Arc::new(obj1)),
            &Value::Object(Arc::new(obj3))
        ));

        // Type mismatches
        assert!(!equal_values(
            &Value::Integer(5),
            &Value::from_string("5".to_string())
        ));
        assert!(!equal_values(
            &Value::Array(vec![].into()),
            &Value::Tuple(vec![].into())
        ));
    }

    #[test]
    fn test_logical_ops() {
        let true_val = Value::Bool(true);
        let false_val = Value::Bool(false);

        // AND
        let result = eval_logical_op(AstBinaryOp::And, &true_val, &false_val)
            .expect("operation should succeed in test");
        assert_eq!(result, false_val);

        // OR
        let result = eval_logical_op(AstBinaryOp::Or, &false_val, &true_val)
            .expect("operation should succeed in test");
        assert_eq!(result, true_val);
    }

    #[test]
    fn test_unary_ops() {
        assert_eq!(
            eval_unary_op(UnaryOp::Negate, &Value::Integer(5))
                .expect("operation should succeed in test"),
            Value::Integer(-5)
        );
        assert_eq!(
            eval_unary_op(UnaryOp::Negate, &Value::Float(3.15))
                .expect("operation should succeed in test"),
            Value::Float(-3.15)
        );

        assert_eq!(
            eval_unary_op(UnaryOp::Not, &Value::Bool(true))
                .expect("operation should succeed in test"),
            Value::Bool(false)
        );
        assert_eq!(
            eval_unary_op(UnaryOp::Not, &Value::Bool(false))
                .expect("operation should succeed in test"),
            Value::Bool(true)
        );

        assert_eq!(
            eval_unary_op(UnaryOp::BitwiseNot, &Value::Integer(5))
                .expect("operation should succeed in test"),
            Value::Integer(!5)
        );
    }

    #[test]
    fn test_division_by_zero() {
        // Integer division by zero should return error
        assert!(div_values(&Value::Integer(10), &Value::Integer(0)).is_err());

        // Float division by zero follows IEEE 754 - returns infinity
        let result = div_values(&Value::Float(10.0), &Value::Float(0.0))
            .expect("operation should succeed in test");
        match result {
            Value::Float(f) => assert!(f.is_infinite()),
            _ => panic!("Expected Float result"),
        }

        // Modulo by zero should return error
        assert!(modulo_values(&Value::Integer(10), &Value::Integer(0)).is_err());
    }

    #[test]
    fn test_sub_values() {
        assert_eq!(
            sub_values(&Value::Integer(10), &Value::Integer(3))
                .expect("operation should succeed in test"),
            Value::Integer(7)
        );
        assert_eq!(
            sub_values(&Value::Float(10.5), &Value::Float(3.5))
                .expect("operation should succeed in test"),
            Value::Float(7.0)
        );
        // Mixed types
        assert_eq!(
            sub_values(&Value::Integer(10), &Value::Float(3.5))
                .expect("operation should succeed in test"),
            Value::Float(6.5)
        );
    }

    #[test]
    fn test_mul_values() {
        assert_eq!(
            mul_values(&Value::Integer(4), &Value::Integer(5))
                .expect("operation should succeed in test"),
            Value::Integer(20)
        );
        assert_eq!(
            mul_values(&Value::Float(2.5), &Value::Float(4.0))
                .expect("operation should succeed in test"),
            Value::Float(10.0)
        );
        // String repeat
        let result = mul_values(&Value::from_string("ab".to_string()), &Value::Integer(3))
            .expect("operation should succeed in test");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "ababab"),
            _ => panic!("Expected string result"),
        }
    }

    #[test]
    fn test_power_values() {
        assert_eq!(
            power_values(&Value::Integer(2), &Value::Integer(3))
                .expect("operation should succeed in test"),
            Value::Integer(8)
        );
        assert_eq!(
            power_values(&Value::Float(2.0), &Value::Float(3.0))
                .expect("operation should succeed in test"),
            Value::Float(8.0)
        );
    }

    #[test]
    fn test_modulo_values() {
        assert_eq!(
            modulo_values(&Value::Integer(10), &Value::Integer(3))
                .expect("operation should succeed in test"),
            Value::Integer(1)
        );
        assert_eq!(
            modulo_values(&Value::Float(10.5), &Value::Float(3.0))
                .expect("operation should succeed in test"),
            Value::Float(1.5)
        );
    }

    #[test]
    fn test_bitwise_ops() {
        // AND
        let result = eval_bitwise_op(AstBinaryOp::BitwiseAnd, &Value::Integer(0b1100), &Value::Integer(0b1010))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(0b1000));

        // OR
        let result = eval_bitwise_op(AstBinaryOp::BitwiseOr, &Value::Integer(0b1100), &Value::Integer(0b1010))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(0b1110));

        // XOR
        let result = eval_bitwise_op(AstBinaryOp::BitwiseXor, &Value::Integer(0b1100), &Value::Integer(0b1010))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(0b0110));

        // Left shift
        let result = eval_bitwise_op(AstBinaryOp::LeftShift, &Value::Integer(1), &Value::Integer(4))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(16));

        // Right shift
        let result = eval_bitwise_op(AstBinaryOp::RightShift, &Value::Integer(16), &Value::Integer(2))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_less_or_equal_values() {
        assert!(less_or_equal_values(&Value::Integer(3), &Value::Integer(5))
            .expect("operation should succeed in test"));
        assert!(less_or_equal_values(&Value::Integer(5), &Value::Integer(5))
            .expect("operation should succeed in test"));
        assert!(!less_or_equal_values(&Value::Integer(6), &Value::Integer(5))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_greater_or_equal_values() {
        assert!(greater_or_equal_values(&Value::Integer(6), &Value::Integer(5))
            .expect("operation should succeed in test"));
        assert!(greater_or_equal_values(&Value::Integer(5), &Value::Integer(5))
            .expect("operation should succeed in test"));
        assert!(!greater_or_equal_values(&Value::Integer(4), &Value::Integer(5))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_eval_binary_op_dispatcher() {
        // Test the main dispatcher with arithmetic
        let result = eval_binary_op(AstBinaryOp::Add, &Value::Integer(2), &Value::Integer(3))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(5));

        // Comparison
        let result = eval_binary_op(AstBinaryOp::Equal, &Value::Integer(5), &Value::Integer(5))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Bool(true));

        // Logical
        let result = eval_binary_op(AstBinaryOp::And, &Value::Bool(true), &Value::Bool(false))
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_mixed_type_comparisons() {
        // Integer vs Float
        assert!(less_than_values(&Value::Integer(3), &Value::Float(3.5))
            .expect("operation should succeed in test"));
        assert!(greater_than_values(&Value::Float(3.5), &Value::Integer(3))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_string_comparison() {
        assert!(less_than_values(
            &Value::from_string("abc".to_string()),
            &Value::from_string("abd".to_string())
        )
        .expect("operation should succeed in test"));
        assert!(!greater_than_values(
            &Value::from_string("abc".to_string()),
            &Value::from_string("abd".to_string())
        )
        .expect("operation should succeed in test"));
    }

    #[test]
    fn test_array_addition() {
        let arr1 = Value::from_array(vec![Value::Integer(1), Value::Integer(2)]);
        let arr2 = Value::from_array(vec![Value::Integer(3), Value::Integer(4)]);
        let result = add_values(&arr1, &arr2).expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 4);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_concat() {
        let s1 = Value::from_string("hello".to_string());
        let s2 = Value::from_string(" world".to_string());
        let result = add_values(&s1, &s2).expect("should succeed");
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_string_plus_integer() {
        let s = Value::from_string("count: ".to_string());
        let i = Value::Integer(42);
        let result = add_values(&s, &i).expect("should succeed");
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "count: 42");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_integer_plus_string() {
        let i = Value::Integer(42);
        let s = Value::from_string(" items".to_string());
        let result = add_values(&i, &s).expect("should succeed");
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "42 items");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_string_plus_float() {
        let s = Value::from_string("pi: ".to_string());
        let f = Value::Float(3.14);
        let result = add_values(&s, &f).expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.starts_with("pi: 3.14"));
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_string_plus_bool() {
        let s = Value::from_string("active: ".to_string());
        let b = Value::Bool(true);
        let result = add_values(&s, &b).expect("should succeed");
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "active: true");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_string_repeat_zero() {
        let s = Value::from_string("abc".to_string());
        let n = Value::Integer(0);
        let result = mul_values(&s, &n).expect("should succeed");
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_string_repeat_negative() {
        let s = Value::from_string("abc".to_string());
        let n = Value::Integer(-1);
        let result = mul_values(&s, &n).expect("should succeed");
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_unary_reference() {
        let v = Value::Integer(42);
        let result = eval_unary_op(UnaryOp::Reference, &v).expect("should succeed");
        assert_eq!(result, Value::Integer(42)); // Reference is no-op in interpreter
    }

    #[test]
    fn test_unary_mutable_reference() {
        let v = Value::Integer(42);
        let result = eval_unary_op(UnaryOp::MutableReference, &v).expect("should succeed");
        assert_eq!(result, Value::Integer(42)); // MutRef is no-op in interpreter
    }

    #[test]
    fn test_unary_deref() {
        let v = Value::Integer(42);
        let result = eval_unary_op(UnaryOp::Deref, &v).expect("should succeed");
        assert_eq!(result, Value::Integer(42)); // Deref is no-op in interpreter
    }

    #[test]
    fn test_bitwise_on_non_integer() {
        let result = eval_bitwise_op(AstBinaryOp::BitwiseAnd, &Value::Float(1.0), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_negate_on_invalid_type() {
        let result = eval_unary_op(UnaryOp::Negate, &Value::Bool(true));
        assert!(result.is_err());
    }

    #[test]
    fn test_bitwise_not_on_invalid_type() {
        let result = eval_unary_op(UnaryOp::BitwiseNot, &Value::Float(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_type_error() {
        let result = add_values(&Value::Bool(true), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_sub_type_error() {
        let result = sub_values(&Value::from_string("a".to_string()), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_not_equal() {
        let result = eval_comparison_op(
            AstBinaryOp::NotEqual,
            &Value::Integer(1),
            &Value::Integer(2),
        )
        .expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_mul_type_error() {
        let result = mul_values(&Value::Bool(true), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_div_type_error() {
        let result = div_values(&Value::from_string("a".to_string()), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_power_mixed_types() {
        let result = power_values(&Value::Integer(2), &Value::Float(3.0)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 8.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_modulo_float() {
        let result = modulo_values(&Value::Float(10.0), &Value::Float(3.0)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 1.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    // === EXTREME TDD Round 22 - Coverage Push Tests ===

    #[test]
    fn test_power_negative_exponent() {
        let result = power_values(&Value::Integer(2), &Value::Integer(-1)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 0.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float for negative exponent");
        }
    }

    #[test]
    fn test_power_large_overflow() {
        // Large exponent that causes overflow
        let result = power_values(&Value::Integer(100), &Value::Integer(100)).expect("should succeed");
        // Should convert to float on overflow
        assert!(matches!(result, Value::Float(_)));
    }

    #[test]
    fn test_modulo_int_float_mixed() {
        let result = modulo_values(&Value::Integer(10), &Value::Float(3.0)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 1.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_modulo_float_int_mixed() {
        let result = modulo_values(&Value::Float(10.0), &Value::Integer(3)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 1.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_modulo_zero_float() {
        let result = modulo_values(&Value::Integer(10), &Value::Float(0.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_type_error() {
        let result = modulo_values(&Value::from_string("a".to_string()), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_power_type_error() {
        let result = power_values(&Value::from_string("a".to_string()), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_power_float_int() {
        let result = power_values(&Value::Float(2.0), &Value::Integer(3)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 8.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_div_int_by_float() {
        let result = div_values(&Value::Integer(10), &Value::Float(4.0)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 2.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_div_float_by_int() {
        let result = div_values(&Value::Float(10.0), &Value::Integer(4)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 2.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_mul_int_float() {
        let result = mul_values(&Value::Integer(3), &Value::Float(2.5)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 7.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_mul_float_int() {
        let result = mul_values(&Value::Float(2.5), &Value::Integer(3)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 7.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_sub_int_float() {
        let result = sub_values(&Value::Integer(10), &Value::Float(3.5)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 6.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_sub_float_int() {
        let result = sub_values(&Value::Float(10.5), &Value::Integer(3)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 7.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_add_int_float() {
        let result = add_values(&Value::Integer(10), &Value::Float(3.5)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 13.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_add_float_int() {
        let result = add_values(&Value::Float(10.5), &Value::Integer(3)).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 13.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_logical_and_truthy_values() {
        // Non-bool truthy values - Integer(0) is still truthy in Ruchy
        let result = eval_logical_op(AstBinaryOp::And, &Value::Integer(1), &Value::Integer(2))
            .expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_logical_and_falsy_left() {
        // Only Bool(false) and Nil are falsy in Ruchy
        let result = eval_logical_op(AstBinaryOp::And, &Value::Bool(false), &Value::Integer(2))
            .expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or_truthy_left() {
        let result = eval_logical_op(AstBinaryOp::Or, &Value::Integer(1), &Value::Integer(2))
            .expect("should succeed");
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_logical_or_falsy_left() {
        // Only Bool(false) and Nil are falsy in Ruchy
        let result = eval_logical_op(AstBinaryOp::Or, &Value::Bool(false), &Value::Integer(2))
            .expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_equal_struct_values() {
        use std::collections::HashMap;
        use std::sync::Arc;

        let mut fields1 = HashMap::new();
        fields1.insert("x".to_string(), Value::Integer(1));
        let struct1 = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(fields1.clone()),
        };

        let mut fields2 = HashMap::new();
        fields2.insert("x".to_string(), Value::Integer(1));
        let struct2 = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(fields2),
        };

        assert!(equal_values(&struct1, &struct2));
    }

    #[test]
    fn test_equal_atom_values() {
        assert!(equal_values(
            &Value::Atom("foo".to_string()),
            &Value::Atom("foo".to_string())
        ));
        assert!(!equal_values(
            &Value::Atom("foo".to_string()),
            &Value::Atom("bar".to_string())
        ));
    }

    #[test]
    fn test_comparison_type_error() {
        let result = less_than_values(&Value::Bool(true), &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_equal_byte_values() {
        assert!(equal_values(&Value::Byte(255), &Value::Byte(255)));
        assert!(!equal_values(&Value::Byte(255), &Value::Byte(0)));
    }

    #[test]
    fn test_equal_class_pointer_identity() {
        use std::collections::HashMap;
        use std::sync::{Arc, RwLock};

        let fields = Arc::new(RwLock::new(HashMap::new()));
        let class1 = Value::Class {
            class_name: "Test".to_string(),
            fields: Arc::clone(&fields),
            methods: Arc::new(HashMap::new()),
        };
        let class2 = Value::Class {
            class_name: "Test".to_string(),
            fields: Arc::clone(&fields),
            methods: Arc::new(HashMap::new()),
        };

        // Same Arc pointer for fields - should be equal
        assert!(equal_values(&class1, &class2));
    }

    // === EXTREME TDD Round 137 - Push to 75+ Tests ===

    #[test]
    fn test_add_integers_negative() {
        let result = add_values(&Value::Integer(-5), &Value::Integer(-3)).unwrap();
        assert_eq!(result, Value::Integer(-8));
    }

    #[test]
    fn test_sub_integers_negative() {
        let result = sub_values(&Value::Integer(-5), &Value::Integer(-3)).unwrap();
        assert_eq!(result, Value::Integer(-2));
    }

    #[test]
    fn test_mul_integers_negative() {
        let result = mul_values(&Value::Integer(-5), &Value::Integer(3)).unwrap();
        assert_eq!(result, Value::Integer(-15));
    }

    #[test]
    fn test_div_integers_exact() {
        let result = div_values(&Value::Integer(10), &Value::Integer(2)).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_div_floats() {
        let result = div_values(&Value::Float(10.0), &Value::Float(4.0)).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 2.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_modulo_integers() {
        let result = modulo_values(&Value::Integer(10), &Value::Integer(3)).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_modulo_integers_zero_remainder() {
        let result = modulo_values(&Value::Integer(10), &Value::Integer(5)).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_equal_nil_values() {
        assert!(equal_values(&Value::Nil, &Value::Nil));
    }

    #[test]
    fn test_not_equal_nil_and_int() {
        assert!(!equal_values(&Value::Nil, &Value::Integer(0)));
    }

    #[test]
    fn test_less_than_floats() {
        let result = less_than_values(&Value::Float(1.5), &Value::Float(2.5)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_less_than_floats_equal() {
        let result = less_than_values(&Value::Float(2.5), &Value::Float(2.5)).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_greater_than_integers() {
        let result = greater_than_values(&Value::Integer(10), &Value::Integer(5)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_greater_than_integers_equal() {
        let result = greater_than_values(&Value::Integer(5), &Value::Integer(5)).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_equal_arrays() {
        use std::sync::Arc;
        let arr1 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let arr2 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(equal_values(&arr1, &arr2));
    }

    #[test]
    fn test_not_equal_arrays_different_length() {
        use std::sync::Arc;
        let arr1 = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let arr2 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(!equal_values(&arr1, &arr2));
    }

    // === EXTREME TDD Round 160 - Coverage Push Tests ===

    #[test]
    fn test_add_integer_overflow_r160() {
        // Test large integers near boundary
        let result = add_values(&Value::Integer(i64::MAX - 1), &Value::Integer(1)).unwrap();
        assert_eq!(result, Value::Integer(i64::MAX));
    }

    #[test]
    fn test_add_floats_r160() {
        let result = add_values(&Value::Float(1.5), &Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_add_string_concat_r160() {
        let result = add_values(
            &Value::from_string("hello".to_string()),
            &Value::from_string(" world".to_string()),
        ).unwrap();
        assert_eq!(result, Value::from_string("hello world".to_string()));
    }

    #[test]
    fn test_sub_integers_r160() {
        let result = sub_values(&Value::Integer(10), &Value::Integer(3)).unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_sub_floats_r160() {
        let result = sub_values(&Value::Float(5.5), &Value::Float(2.5)).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_sub_negative_result_r160() {
        let result = sub_values(&Value::Integer(3), &Value::Integer(10)).unwrap();
        assert_eq!(result, Value::Integer(-7));
    }

    #[test]
    fn test_mul_integers_r160() {
        let result = mul_values(&Value::Integer(7), &Value::Integer(6)).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_mul_floats_r160() {
        let result = mul_values(&Value::Float(2.5), &Value::Float(4.0)).unwrap();
        assert_eq!(result, Value::Float(10.0));
    }

    #[test]
    fn test_mul_zero_r160() {
        let result = mul_values(&Value::Integer(12345), &Value::Integer(0)).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_div_integers_r160() {
        let result = div_values(&Value::Integer(20), &Value::Integer(5)).unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_div_floats_r160() {
        let result = div_values(&Value::Float(10.0), &Value::Float(4.0)).unwrap();
        assert_eq!(result, Value::Float(2.5));
    }

    #[test]
    fn test_div_by_zero_integer_r160() {
        let result = div_values(&Value::Integer(10), &Value::Integer(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_div_by_zero_float_r160() {
        let result = div_values(&Value::Float(10.0), &Value::Float(0.0));
        // Float division by zero returns infinity, not error
        assert!(result.is_ok());
    }

    #[test]
    fn test_modulo_integers_r160() {
        let result = modulo_values(&Value::Integer(17), &Value::Integer(5)).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_modulo_negative_r160() {
        let result = modulo_values(&Value::Integer(-17), &Value::Integer(5)).unwrap();
        assert_eq!(result, Value::Integer(-2));
    }

    #[test]
    fn test_modulo_by_zero_r160() {
        let result = modulo_values(&Value::Integer(10), &Value::Integer(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_negate_integer_r160() {
        let result = eval_unary_op(UnaryOp::Negate, &Value::Integer(42)).unwrap();
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_negate_float_r160() {
        let result = eval_unary_op(UnaryOp::Negate, &Value::Float(3.14)).unwrap();
        assert_eq!(result, Value::Float(-3.14));
    }

    #[test]
    fn test_negate_already_negative_r160() {
        let result = eval_unary_op(UnaryOp::Negate, &Value::Integer(-100)).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_not_bool_true_r160() {
        let result = eval_unary_op(UnaryOp::Not, &Value::Bool(true)).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_not_bool_false_r160() {
        let result = eval_unary_op(UnaryOp::Not, &Value::Bool(false)).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_less_than_equal_integers_r160() {
        assert!(less_or_equal_values(&Value::Integer(5), &Value::Integer(5)).unwrap());
        assert!(less_or_equal_values(&Value::Integer(4), &Value::Integer(5)).unwrap());
        assert!(!less_or_equal_values(&Value::Integer(6), &Value::Integer(5)).unwrap());
    }

    #[test]
    fn test_greater_than_equal_integers_r160() {
        assert!(greater_or_equal_values(&Value::Integer(5), &Value::Integer(5)).unwrap());
        assert!(greater_or_equal_values(&Value::Integer(6), &Value::Integer(5)).unwrap());
        assert!(!greater_or_equal_values(&Value::Integer(4), &Value::Integer(5)).unwrap());
    }

    #[test]
    fn test_less_than_strings_r160() {
        let result = less_than_values(
            &Value::from_string("apple".to_string()),
            &Value::from_string("banana".to_string()),
        ).unwrap();
        assert!(result);
    }

    #[test]
    fn test_greater_than_strings_r160() {
        let result = greater_than_values(
            &Value::from_string("zebra".to_string()),
            &Value::from_string("apple".to_string()),
        ).unwrap();
        assert!(result);
    }

    #[test]
    fn test_equal_bools_r160() {
        assert!(equal_values(&Value::Bool(true), &Value::Bool(true)));
        assert!(equal_values(&Value::Bool(false), &Value::Bool(false)));
        assert!(!equal_values(&Value::Bool(true), &Value::Bool(false)));
    }

    #[test]
    fn test_equal_nil_r160() {
        assert!(equal_values(&Value::Nil, &Value::Nil));
    }

    #[test]
    fn test_equal_mixed_int_float_r160() {
        // Integer and Float with same value are equal in Ruchy
        assert!(equal_values(&Value::Integer(42), &Value::Float(42.0)));
        // But different types with incompatible semantics are not
        assert!(!equal_values(&Value::Integer(1), &Value::Bool(true)));
    }

    #[test]
    fn test_equal_strings_r160() {
        assert!(equal_values(
            &Value::from_string("hello".to_string()),
            &Value::from_string("hello".to_string()),
        ));
        assert!(!equal_values(
            &Value::from_string("hello".to_string()),
            &Value::from_string("world".to_string()),
        ));
    }

    #[test]
    fn test_equal_tuples_r160() {
        use std::sync::Arc;
        let t1 = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let t2 = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(equal_values(&t1, &t2));
    }

    #[test]
    fn test_not_equal_tuples_different_length_r160() {
        use std::sync::Arc;
        let t1 = Value::Tuple(Arc::from(vec![Value::Integer(1)]));
        let t2 = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(!equal_values(&t1, &t2));
    }

    #[test]
    fn test_logical_and_values_r160() {
        let result = eval_logical_op(AstBinaryOp::And, &Value::Bool(true), &Value::Bool(true)).unwrap();
        assert!(result.is_truthy());
        let result = eval_logical_op(AstBinaryOp::And, &Value::Bool(true), &Value::Bool(false)).unwrap();
        assert!(!result.is_truthy());
        let result = eval_logical_op(AstBinaryOp::And, &Value::Bool(false), &Value::Bool(true)).unwrap();
        assert!(!result.is_truthy());
    }

    #[test]
    fn test_logical_or_values_r160() {
        let result = eval_logical_op(AstBinaryOp::Or, &Value::Bool(true), &Value::Bool(false)).unwrap();
        assert!(result.is_truthy());
        let result = eval_logical_op(AstBinaryOp::Or, &Value::Bool(false), &Value::Bool(true)).unwrap();
        assert!(result.is_truthy());
        let result = eval_logical_op(AstBinaryOp::Or, &Value::Bool(false), &Value::Bool(false)).unwrap();
        assert!(!result.is_truthy());
    }
}
