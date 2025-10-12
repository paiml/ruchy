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

/// Handle comparison operations
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
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
        AstBinaryOp::LessEqual => {
            let less = less_than_values(left, right)?;
            let equal = equal_values(left, right);
            Ok(Value::Bool(less || equal))
        }
        AstBinaryOp::GreaterEqual => {
            let greater = greater_than_values(left, right)?;
            let equal = equal_values(left, right);
            Ok(Value::Bool(greater || equal))
        }
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
        UnaryOp::Reference => {
            // In interpreted mode, the reference operator (&) is a no-op
            // The interpreter already manages value ownership internally
            // This allows Rust-like syntax (&value) to work in eval mode
            Ok(operand.clone())
        }
        UnaryOp::Deref => Err(InterpreterError::RuntimeError(
            "Dereference operator not yet implemented in interpreter".to_string(),
        )),
    }
}

// Arithmetic helper functions

/// Add two values
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
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

/// Modulo operation on two values
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn modulo_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
            a.checked_rem(*b).map(Value::Integer).ok_or_else(|| {
                InterpreterError::RuntimeError("Integer overflow in modulo".to_string())
            })
        }
        (Value::Float(a), Value::Float(b)) => {
            if *b == 0.0 {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(a % b))
        }
        (Value::Integer(a), Value::Float(b)) => {
            if *b == 0.0 {
                return Err(InterpreterError::DivisionByZero);
            }
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Float((*a as f64) % b))
        }
        (Value::Float(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
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
        (Value::Array(a), Value::Array(b)) => equal_arrays(a, b),
        // Tuples - delegate to helper
        (Value::Tuple(a), Value::Tuple(b)) => equal_tuples(a, b),
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

/// QUALITY-017: Compare object values field-by-field
/// Complexity: 5 (within Toyota Way limits)
fn equal_objects(
    a: &std::collections::HashMap<String, Value>,
    b: &std::collections::HashMap<String, Value>,
) -> bool {
    // Quick length check
    if a.len() != b.len() {
        return false;
    }
    // Check all fields match
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if !equal_values(val_a, val_b) {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

/// QUALITY-017: Compare array values element-by-element
/// Complexity: 2 (within Toyota Way limits)
fn equal_arrays(a: &[Value], b: &[Value]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| equal_values(x, y))
}

/// QUALITY-017: Compare tuple values element-by-element
/// Complexity: 2 (within Toyota Way limits)
fn equal_tuples(a: &[Value], b: &[Value]) -> bool {
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
            add_values(&Value::Integer(2), &Value::Integer(3)).unwrap(),
            Value::Integer(5)
        );
        assert_eq!(
            add_values(&Value::Float(2.5), &Value::Float(3.5)).unwrap(),
            Value::Float(6.0)
        );

        let s1 = Value::from_string("hello".to_string());
        let s2 = Value::from_string(" world".to_string());
        let result = add_values(&s1, &s2).unwrap();
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

        assert!(less_than_values(&Value::Integer(3), &Value::Integer(5)).unwrap());
        assert!(!less_than_values(&Value::Integer(5), &Value::Integer(3)).unwrap());

        assert!(greater_than_values(&Value::Integer(5), &Value::Integer(3)).unwrap());
        assert!(!greater_than_values(&Value::Integer(3), &Value::Integer(5)).unwrap());
    }

    /// QUALITY-017: Comprehensive tests for equal_values() before refactoring
    /// Tests cover all Value types to ensure refactoring doesn't break equality
    #[test]
    fn test_equal_values_comprehensive() {
        use std::collections::HashMap;
        use std::sync::Arc;

        // Primitives
        assert!(equal_values(&Value::Integer(42), &Value::Integer(42)));
        assert!(!equal_values(&Value::Integer(42), &Value::Integer(43)));
        assert!(equal_values(&Value::Float(3.14), &Value::Float(3.14)));
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
        let result = eval_logical_op(AstBinaryOp::And, &true_val, &false_val).unwrap();
        assert_eq!(result, false_val);

        // OR
        let result = eval_logical_op(AstBinaryOp::Or, &false_val, &true_val).unwrap();
        assert_eq!(result, true_val);
    }

    #[test]
    fn test_unary_ops() {
        assert_eq!(
            eval_unary_op(UnaryOp::Negate, &Value::Integer(5)).unwrap(),
            Value::Integer(-5)
        );
        assert_eq!(
            eval_unary_op(UnaryOp::Negate, &Value::Float(3.14)).unwrap(),
            Value::Float(-3.14)
        );

        assert_eq!(
            eval_unary_op(UnaryOp::Not, &Value::Bool(true)).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            eval_unary_op(UnaryOp::Not, &Value::Bool(false)).unwrap(),
            Value::Bool(true)
        );

        assert_eq!(
            eval_unary_op(UnaryOp::BitwiseNot, &Value::Integer(5)).unwrap(),
            Value::Integer(!5)
        );
    }

    #[test]
    fn test_division_by_zero() {
        // Integer division by zero should return error
        assert!(div_values(&Value::Integer(10), &Value::Integer(0)).is_err());

        // Float division by zero follows IEEE 754 - returns infinity
        let result = div_values(&Value::Float(10.0), &Value::Float(0.0)).unwrap();
        match result {
            Value::Float(f) => assert!(f.is_infinite()),
            _ => panic!("Expected Float result"),
        }

        // Modulo by zero should return error
        assert!(modulo_values(&Value::Integer(10), &Value::Integer(0)).is_err());
    }
}
