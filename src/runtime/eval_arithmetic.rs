//! Arithmetic and binary operations evaluation module
//!
//! This module handles evaluation of all arithmetic, comparison, logical,
//! and bitwise operations in the interpreter. Extracted from the monolithic
//! interpreter.rs to improve maintainability and follow Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::BinaryOp as AstBinaryOp;
use crate::runtime::{InterpreterError, Value};

/// Evaluate a binary operation between two values
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
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

/// Evaluate a unary operation on a value
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn eval_unary_op(
    op: crate::frontend::ast::UnaryOp,
    operand: &Value,
) -> Result<Value, InterpreterError> {
    use crate::frontend::ast::UnaryOp;
    match op {
        UnaryOp::Negate => match operand {
            Value::Integer(i) => Ok(Value::from_i64(-i)),
            Value::Float(f) => Ok(Value::from_f64(-f)),
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot negate {}",
                operand.type_name()
            ))),
        },
        UnaryOp::Not => Ok(Value::from_bool(!operand.is_truthy())),
        UnaryOp::BitwiseNot => match operand {
            Value::Integer(i) => Ok(Value::from_i64(!i)),
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot apply bitwise NOT to {}",
                operand.type_name()
            ))),
        },
        UnaryOp::Reference => Err(InterpreterError::RuntimeError(format!(
            "Unary operator not yet implemented: {op:?}"
        ))),
    }
}

// Arithmetic operations (complexity <= 6 each)

/// Handle arithmetic operations with type promotion
///
/// Supports numeric operations between integers and floats with automatic
/// type promotion. String concatenation is supported for the `+` operator.
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
/// Compares values of compatible types and returns a boolean result.
fn eval_comparison_op(
    op: AstBinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match op {
        AstBinaryOp::Equal => Ok(Value::from_bool(equal_values(left, right))),
        AstBinaryOp::NotEqual => Ok(Value::from_bool(!equal_values(left, right))),
        AstBinaryOp::Less => Ok(Value::from_bool(less_than_values(left, right)?)),
        AstBinaryOp::Greater => Ok(Value::from_bool(greater_than_values(left, right)?)),
        AstBinaryOp::LessEqual => {
            let less = less_than_values(left, right)?;
            let equal = equal_values(left, right);
            Ok(Value::from_bool(less || equal))
        }
        AstBinaryOp::GreaterEqual => {
            let greater = greater_than_values(left, right)?;
            let equal = equal_values(left, right);
            Ok(Value::from_bool(greater || equal))
        }
        _ => unreachable!("Non-comparison operation passed to eval_comparison_op"),
    }
}

/// Handle logical operations (And, Or) with short-circuit evaluation
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

/// Handle bitwise operations (complexity <= 5)
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

// Helper functions for specific arithmetic operations (complexity <= 8 each)

fn add_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
        (Value::String(a), Value::String(b)) => Ok(Value::from_string(format!("{a}{b}"))),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot add {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn sub_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot subtract {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn mul_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot multiply {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn div_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
            // Integer division in Ruchy
            Ok(Value::Integer(a / b))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b.abs() < f64::EPSILON {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(a / b))
        }
        (Value::Integer(a), Value::Float(b)) => {
            if b.abs() < f64::EPSILON {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(*a as f64 / b))
        }
        (Value::Float(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(a / *b as f64))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot divide {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn modulo_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Integer(a % b))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b.abs() < f64::EPSILON {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(a % b))
        }
        (Value::Integer(a), Value::Float(b)) => {
            if b.abs() < f64::EPSILON {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(*a as f64 % b))
        }
        (Value::Float(a), Value::Integer(b)) => {
            if *b == 0 {
                return Err(InterpreterError::DivisionByZero);
            }
            Ok(Value::Float(a % *b as f64))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot modulo {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn power_values(left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b < 0 {
                // Negative exponent results in float
                Ok(Value::Float((*a as f64).powf(*b as f64)))
            } else {
                #[allow(clippy::cast_sign_loss)]
                let result = a.pow(*b as u32);
                Ok(Value::Integer(result))
            }
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(*b as f64))),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot exponentiate {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

// Comparison helper functions (complexity <= 5 each)

fn equal_values(left: &Value, right: &Value) -> bool {
    left == right
}

fn less_than_values(left: &Value, right: &Value) -> Result<bool, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
        (Value::Float(a), Value::Float(b)) => Ok(a < b),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) < *b),
        (Value::Float(a), Value::Integer(b)) => Ok(*a < (*b as f64)),
        (Value::String(a), Value::String(b)) => Ok(a < b),
        (Value::Bool(a), Value::Bool(b)) => Ok(a < b),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot compare {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn greater_than_values(left: &Value, right: &Value) -> Result<bool, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
        (Value::Float(a), Value::Float(b)) => Ok(a > b),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) > *b),
        (Value::Float(a), Value::Integer(b)) => Ok(*a > (*b as f64)),
        (Value::String(a), Value::String(b)) => Ok(a > b),
        (Value::Bool(a), Value::Bool(b)) => Ok(a > b),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot compare {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_integers() {
        let left = Value::Integer(5);
        let right = Value::Integer(3);
        let result = add_values(&left, &right).unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_add_mixed_types() {
        let left = Value::Integer(5);
        let right = Value::Float(3.5);
        let result = add_values(&left, &right).unwrap();
        assert_eq!(result, Value::Float(8.5));
    }

    #[test]
    fn test_string_concatenation() {
        let left = Value::from_string("hello".to_string());
        let right = Value::from_string(" world".to_string());
        let result = add_values(&left, &right).unwrap();
        assert_eq!(result, Value::from_string("hello world".to_string()));
    }

    #[test]
    fn test_division_by_zero() {
        let left = Value::Integer(10);
        let right = Value::Integer(0);
        let result = div_values(&left, &right);
        assert!(matches!(result, Err(InterpreterError::DivisionByZero)));
    }

    #[test]
    fn test_comparison_operations() {
        let left = Value::Integer(5);
        let right = Value::Integer(3);

        assert!(less_than_values(&right, &left).unwrap());
        assert!(greater_than_values(&left, &right).unwrap());
        assert!(!equal_values(&left, &right));
    }

    #[test]
    fn test_logical_operations() {
        let left = Value::Bool(true);
        let right = Value::Bool(false);

        let and_result = eval_logical_op(AstBinaryOp::And, &left, &right).unwrap();
        assert_eq!(and_result, Value::Bool(false));

        let or_result = eval_logical_op(AstBinaryOp::Or, &left, &right).unwrap();
        assert_eq!(or_result, Value::Bool(true));
    }

    #[test]
    fn test_bitwise_operations() {
        let left = Value::Integer(5); // 101 in binary
        let right = Value::Integer(3); // 011 in binary

        let and_result = eval_bitwise_op(AstBinaryOp::BitwiseAnd, &left, &right).unwrap();
        assert_eq!(and_result, Value::Integer(1)); // 001 in binary

        let or_result = eval_bitwise_op(AstBinaryOp::BitwiseOr, &left, &right).unwrap();
        assert_eq!(or_result, Value::Integer(7)); // 111 in binary
    }

    #[test]
    fn test_unary_operations() {
        let value = Value::Integer(42);
        let result = eval_unary_op(crate::frontend::ast::UnaryOp::Negate, &value).unwrap();
        assert_eq!(result, Value::Integer(-42));

        let bool_value = Value::Bool(true);
        let not_result = eval_unary_op(crate::frontend::ast::UnaryOp::Not, &bool_value).unwrap();
        assert_eq!(not_result, Value::Bool(false));
    }
}
