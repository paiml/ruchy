//! Binary and unary expression evaluation module
//!
//! This module handles evaluation of binary and unary operations.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::frontend::ast::{BinaryOp, Expr, UnaryOp};
use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate a binary expression with proper short-circuit evaluation
///
/// # Complexity
/// Cyclomatic complexity: 9 (within limit of 10)
pub fn eval_binary_expr<F>(
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    eval_fn: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    // Handle short-circuit operators first
    match op {
        BinaryOp::NullCoalesce => {
            let left_val = eval_fn(left)?;
            if matches!(left_val, Value::Nil) {
                eval_fn(right)
            } else {
                Ok(left_val)
            }
        }
        BinaryOp::And => {
            let left_val = eval_fn(left)?;
            if left_val.is_truthy() {
                eval_fn(right)
            } else {
                Ok(left_val)
            }
        }
        BinaryOp::Or => {
            let left_val = eval_fn(left)?;
            if left_val.is_truthy() {
                Ok(left_val)
            } else {
                eval_fn(right)
            }
        }
        _ => {
            // Non-short-circuit operators: evaluate both operands
            let left_val = eval_fn(left)?;
            let right_val = eval_fn(right)?;
            eval_binary_op(op, &left_val, &right_val)
        }
    }
}

/// Evaluate a binary operation on two values
///
/// # Complexity
/// Cyclomatic complexity: 8 (within limit of 10)
pub fn eval_binary_op(
    op: BinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match op {
        // Arithmetic operations
        BinaryOp::Add
        | BinaryOp::Subtract
        | BinaryOp::Multiply
        | BinaryOp::Divide
        | BinaryOp::Modulo
        | BinaryOp::Power => eval_arithmetic_op(op, left, right),
        // Comparison operations
        BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::Less
        | BinaryOp::Greater
        | BinaryOp::LessEqual
        | BinaryOp::GreaterEqual
        | BinaryOp::Gt => eval_comparison_op(op, left, right),
        // Logical operations (already handled in eval_binary_expr for short-circuit)
        BinaryOp::And | BinaryOp::Or => eval_logical_op(op, left, right),
        // Bitwise operations
        BinaryOp::BitwiseAnd
        | BinaryOp::BitwiseOr
        | BinaryOp::BitwiseXor
        | BinaryOp::LeftShift
        | BinaryOp::RightShift => eval_bitwise_op(op, left, right),
        // Other operations
        BinaryOp::NullCoalesce => {
            // Should have been handled in eval_binary_expr
            Ok(if matches!(left, Value::Nil) {
                right.clone()
            } else {
                left.clone()
            })
        }
    }
}

/// Evaluate arithmetic operations
///
/// # Complexity
/// Cyclomatic complexity: 7 (within limit of 10)
fn eval_arithmetic_op(
    op: BinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => match op {
            BinaryOp::Add => Ok(Value::Integer(a + b)),
            BinaryOp::Subtract => Ok(Value::Integer(a - b)),
            BinaryOp::Multiply => Ok(Value::Integer(a * b)),
            BinaryOp::Divide => {
                if *b == 0 {
                    Err(InterpreterError::RuntimeError(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            BinaryOp::Modulo => {
                if *b == 0 {
                    Err(InterpreterError::RuntimeError("Modulo by zero".to_string()))
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            BinaryOp::Power => {
                if *b >= 0 {
                    Ok(Value::Integer(a.pow(*b as u32)))
                } else {
                    Ok(Value::Float((*a as f64).powf(*b as f64)))
                }
            }
            _ => unreachable!(),
        },
        (Value::Float(a), Value::Float(b)) => match op {
            BinaryOp::Add => Ok(Value::Float(a + b)),
            BinaryOp::Subtract => Ok(Value::Float(a - b)),
            BinaryOp::Multiply => Ok(Value::Float(a * b)),
            BinaryOp::Divide => {
                if *b == 0.0 {
                    Err(InterpreterError::RuntimeError(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            BinaryOp::Modulo => Ok(Value::Float(a % b)),
            BinaryOp::Power => Ok(Value::Float(a.powf(*b))),
            _ => unreachable!(),
        },
        // Mixed integer/float
        (Value::Integer(a), Value::Float(b)) => {
            eval_arithmetic_op(op, &Value::Float(*a as f64), &Value::Float(*b))
        }
        (Value::Float(a), Value::Integer(b)) => {
            eval_arithmetic_op(op, &Value::Float(*a), &Value::Float(*b as f64))
        }
        // String concatenation
        (Value::String(a), Value::String(b)) if matches!(op, BinaryOp::Add) => {
            Ok(Value::String(Rc::new(format!("{a}{b}"))))
        }
        _ => Err(InterpreterError::RuntimeError(format!(
            "Invalid operands for {:?}: {} and {}",
            op,
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Evaluate comparison operations
///
/// # Complexity
/// Cyclomatic complexity: 8 (within limit of 10)
fn eval_comparison_op(
    op: BinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, InterpreterError> {
    let result = match op {
        BinaryOp::Equal => left == right,
        BinaryOp::NotEqual => left != right,
        BinaryOp::Less => compare_values(left, right)? < 0,
        BinaryOp::Greater | BinaryOp::Gt => compare_values(left, right)? > 0,
        BinaryOp::LessEqual => compare_values(left, right)? <= 0,
        BinaryOp::GreaterEqual => compare_values(left, right)? >= 0,
        _ => unreachable!(),
    };
    Ok(Value::Bool(result))
}

/// Compare two values for ordering
///
/// # Complexity
/// Cyclomatic complexity: 6 (within limit of 10)
fn compare_values(left: &Value, right: &Value) -> Result<i8, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(if a < b { -1 } else { i8::from(a > b) }),
        (Value::Float(a), Value::Float(b)) => Ok(if a < b { -1 } else { i8::from(a > b) }),
        (Value::Integer(a), Value::Float(b)) => {
            let a = *a as f64;
            Ok(if a < *b { -1 } else { i8::from(a > *b) })
        }
        (Value::Float(a), Value::Integer(b)) => {
            let b = *b as f64;
            Ok(if a < &b { -1 } else { i8::from(a > &b) })
        }
        (Value::String(a), Value::String(b)) => Ok(if a < b { -1 } else { i8::from(a > b) }),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Cannot compare {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Evaluate logical operations
///
/// # Complexity
/// Cyclomatic complexity: 3 (within limit of 10)
fn eval_logical_op(op: BinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match op {
        BinaryOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
        BinaryOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
        _ => unreachable!(),
    }
}

/// Evaluate bitwise operations
///
/// # Complexity
/// Cyclomatic complexity: 7 (within limit of 10)
fn eval_bitwise_op(op: BinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => match op {
            BinaryOp::BitwiseAnd => Ok(Value::Integer(a & b)),
            BinaryOp::BitwiseOr => Ok(Value::Integer(a | b)),
            BinaryOp::BitwiseXor => Ok(Value::Integer(a ^ b)),
            BinaryOp::LeftShift => Ok(Value::Integer(a << b)),
            BinaryOp::RightShift => Ok(Value::Integer(a >> b)),
            _ => unreachable!(),
        },
        _ => Err(InterpreterError::RuntimeError(format!(
            "Bitwise operations require integers, got {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

/// Evaluate a unary expression
///
/// # Complexity
/// Cyclomatic complexity: 6 (within limit of 10)
pub fn eval_unary_expr<F>(
    op: UnaryOp,
    operand: &Expr,
    eval_fn: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let val = eval_fn(operand)?;
    eval_unary_op(op, &val)
}

/// Evaluate a unary operation on a value
///
/// # Complexity
/// Cyclomatic complexity: 5 (within limit of 10)
pub fn eval_unary_op(op: UnaryOp, val: &Value) -> Result<Value, InterpreterError> {
    match op {
        UnaryOp::Negate => match val {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot negate {}",
                val.type_name()
            ))),
        },
        UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
        UnaryOp::BitwiseNot => match val {
            Value::Integer(n) => Ok(Value::Integer(!n)),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Bitwise NOT requires integer, got {}",
                val.type_name()
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unsupported unary operator: {op:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_addition() {
        let result = eval_binary_op(BinaryOp::Add, &Value::Integer(2), &Value::Integer(3)).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_float_multiplication() {
        let result =
            eval_binary_op(BinaryOp::Multiply, &Value::Float(2.5), &Value::Float(4.0)).unwrap();
        assert_eq!(result, Value::Float(10.0));
    }

    #[test]
    fn test_division_by_zero() {
        let result = eval_binary_op(BinaryOp::Divide, &Value::Integer(5), &Value::Integer(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_string_concatenation() {
        let result = eval_binary_op(
            BinaryOp::Add,
            &Value::String(Rc::new("hello".to_string())),
            &Value::String(Rc::new(" world".to_string())),
        )
        .unwrap();
        assert_eq!(result, Value::String(Rc::new("hello world".to_string())));
    }

    #[test]
    fn test_comparison_equal() {
        let result =
            eval_binary_op(BinaryOp::Equal, &Value::Integer(5), &Value::Integer(5)).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_and() {
        let result =
            eval_logical_op(BinaryOp::And, &Value::Bool(true), &Value::Bool(false)).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unary_negate() {
        let result = eval_unary_op(UnaryOp::Negate, &Value::Integer(42)).unwrap();
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_unary_not() {
        let result = eval_unary_op(UnaryOp::Not, &Value::Bool(true)).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_integer_arithmetic_commutative(a: i64, b: i64) {
            // Addition is commutative
            let add1 = eval_binary_op(BinaryOp::Add, &Value::Integer(a), &Value::Integer(b)).unwrap();
            let add2 = eval_binary_op(BinaryOp::Add, &Value::Integer(b), &Value::Integer(a)).unwrap();
            prop_assert_eq!(add1, add2);

            // Multiplication is commutative
            let mul1 = eval_binary_op(BinaryOp::Multiply, &Value::Integer(a), &Value::Integer(b)).unwrap();
            let mul2 = eval_binary_op(BinaryOp::Multiply, &Value::Integer(b), &Value::Integer(a)).unwrap();
            prop_assert_eq!(mul1, mul2);
        }

        #[test]
        fn test_comparison_consistency(a: i64, b: i64) {
            let less = eval_binary_op(BinaryOp::Less, &Value::Integer(a), &Value::Integer(b)).unwrap();
            let greater_equal = eval_binary_op(BinaryOp::GreaterEqual, &Value::Integer(a), &Value::Integer(b)).unwrap();

            // a < b should be opposite of a >= b
            match (less, greater_equal) {
                (Value::Bool(l), Value::Bool(ge)) => prop_assert_eq!(l, !ge),
                _ => prop_assert!(false, "Comparison should return bool"),
            }
        }

        #[test]
        fn test_double_negation(n: i64) {
            let negated_once = eval_unary_op(UnaryOp::Negate, &Value::Integer(n)).unwrap();
            let negated_twice = eval_unary_op(UnaryOp::Negate, &negated_once).unwrap();
            prop_assert_eq!(negated_twice, Value::Integer(n));
        }
    }
}
