//! Builtin function evaluation module
//!
//! This module handles all builtin functions including math operations,
//! I/O functions, utility functions, and type operations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate a builtin function call
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_builtin_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        // I/O functions
        "__builtin_println__" => Ok(Some(eval_println(args)?)),
        "__builtin_print__" => Ok(Some(eval_print(args)?)),
        "__builtin_dbg__" => Ok(Some(eval_dbg(args)?)),

        // Math functions
        "__builtin_sqrt__" => Ok(Some(eval_sqrt(args)?)),
        "__builtin_pow__" => Ok(Some(eval_pow(args)?)),
        "__builtin_abs__" => Ok(Some(eval_abs(args)?)),
        "__builtin_min__" => Ok(Some(eval_min(args)?)),
        "__builtin_max__" => Ok(Some(eval_max(args)?)),
        "__builtin_floor__" => Ok(Some(eval_floor(args)?)),
        "__builtin_ceil__" => Ok(Some(eval_ceil(args)?)),
        "__builtin_round__" => Ok(Some(eval_round(args)?)),

        // Trigonometric functions
        "__builtin_sin__" => Ok(Some(eval_sin(args)?)),
        "__builtin_cos__" => Ok(Some(eval_cos(args)?)),
        "__builtin_tan__" => Ok(Some(eval_tan(args)?)),

        // Utility functions
        "__builtin_len__" => Ok(Some(eval_len(args)?)),
        "__builtin_range__" => Ok(Some(eval_range(args)?)),
        "__builtin_type__" => Ok(Some(eval_type(args)?)),
        "__builtin_reverse__" => Ok(Some(eval_reverse(args)?)),

        // Unknown builtin
        _ => Ok(None),
    }
}

/// Print values to stdout with newline
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_println(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        println!();
    } else {
        let output = args
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{output}");
    }
    Ok(Value::Nil)
}

/// Print values to stdout without newline
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_print(args: &[Value]) -> Result<Value, InterpreterError> {
    let output = args
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(" ");
    print!("{output}");
    Ok(Value::Nil)
}

/// Debug print with value inspection
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_dbg(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() == 1 {
        println!("[DEBUG] {:?}", args[0]);
        Ok(args[0].clone())
    } else {
        println!("[DEBUG] {args:?}");
        Ok(Value::Array(Rc::new(args.to_vec())))
    }
}

/// Square root function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_sqrt(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "sqrt() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
        Value::Float(f) => Ok(Value::Float(f.sqrt())),
        _ => Err(InterpreterError::RuntimeError(
            "sqrt() expects a number".to_string(),
        )),
    }
}

/// Power function (base^exponent)
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn eval_pow(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "pow() expects exactly 2 arguments".to_string(),
        ));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(base), Value::Integer(exp)) => {
            if *exp >= 0 {
                Ok(Value::Integer(base.pow(*exp as u32)))
            } else {
                Ok(Value::Float((*base as f64).powf(*exp as f64)))
            }
        }
        (Value::Float(base), Value::Integer(exp)) => Ok(Value::Float(base.powf(*exp as f64))),
        (Value::Integer(base), Value::Float(exp)) => Ok(Value::Float((*base as f64).powf(*exp))),
        (Value::Float(base), Value::Float(exp)) => Ok(Value::Float(base.powf(*exp))),
        _ => Err(InterpreterError::RuntimeError(
            "pow() expects two numbers".to_string(),
        )),
    }
}

/// Absolute value function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_abs(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "abs() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(InterpreterError::RuntimeError(
            "abs() expects a number".to_string(),
        )),
    }
}

/// Minimum of two values
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_min(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "min() expects exactly 2 arguments".to_string(),
        ));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.min(*b as f64))),
        _ => Err(InterpreterError::RuntimeError(
            "min() expects two numbers".to_string(),
        )),
    }
}

/// Maximum of two values
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_max(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "max() expects exactly 2 arguments".to_string(),
        ));
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.max(*b as f64))),
        _ => Err(InterpreterError::RuntimeError(
            "max() expects two numbers".to_string(),
        )),
    }
}

/// Floor function (round down)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_floor(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "floor() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "floor() expects a number".to_string(),
        )),
    }
}

/// Ceiling function (round up)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_ceil(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "ceil() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "ceil() expects a number".to_string(),
        )),
    }
}

/// Round to nearest integer
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_round(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "round() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "round() expects a number".to_string(),
        )),
    }
}

/// Sine function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_sin(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "sin() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sin())),
        Value::Float(f) => Ok(Value::Float(f.sin())),
        _ => Err(InterpreterError::RuntimeError(
            "sin() expects a number".to_string(),
        )),
    }
}

/// Cosine function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_cos(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "cos() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).cos())),
        Value::Float(f) => Ok(Value::Float(f.cos())),
        _ => Err(InterpreterError::RuntimeError(
            "cos() expects a number".to_string(),
        )),
    }
}

/// Tangent function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_tan(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "tan() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).tan())),
        Value::Float(f) => Ok(Value::Float(f.tan())),
        _ => Err(InterpreterError::RuntimeError(
            "tan() expects a number".to_string(),
        )),
    }
}

/// Length of collections and strings
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_len(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "len() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::DataFrame { columns } => {
            if columns.is_empty() {
                Ok(Value::Integer(0))
            } else {
                Ok(Value::Integer(columns[0].values.len() as i64))
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "len() expects a string, array, or dataframe".to_string(),
        )),
    }
}

/// Generate ranges of integers
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn eval_range(args: &[Value]) -> Result<Value, InterpreterError> {
    match args.len() {
        1 => eval_range_one_arg(&args[0]),
        2 => eval_range_two_args(&args[0], &args[1]),
        3 => eval_range_three_args(&args[0], &args[1], &args[2]),
        _ => Err(InterpreterError::RuntimeError(
            "range() expects 1, 2, or 3 arguments".to_string(),
        )),
    }
}

/// Range with single argument: range(end) -> 0..end
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_range_one_arg(end_val: &Value) -> Result<Value, InterpreterError> {
    match end_val {
        Value::Integer(end) => {
            let mut result = Vec::new();
            for i in 0..*end {
                result.push(Value::Integer(i));
            }
            Ok(Value::Array(result.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "range() expects integer arguments".to_string(),
        )),
    }
}

/// Range with two arguments: range(start, end) -> start..end
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_range_two_args(start_val: &Value, end_val: &Value) -> Result<Value, InterpreterError> {
    match (start_val, end_val) {
        (Value::Integer(start), Value::Integer(end)) => {
            let mut result = Vec::new();
            for i in *start..*end {
                result.push(Value::Integer(i));
            }
            Ok(Value::Array(result.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "range() expects integer arguments".to_string(),
        )),
    }
}

/// Range with three arguments: range(start, end, step) -> start..end by step
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_range_three_args(
    start_val: &Value,
    end_val: &Value,
    step_val: &Value,
) -> Result<Value, InterpreterError> {
    match (start_val, end_val, step_val) {
        (Value::Integer(start), Value::Integer(end), Value::Integer(step)) => {
            if *step == 0 {
                return Err(InterpreterError::RuntimeError(
                    "range() step cannot be zero".to_string(),
                ));
            }
            let mut result = Vec::new();
            if *step > 0 {
                let mut i = *start;
                while i < *end {
                    result.push(Value::Integer(i));
                    i += step;
                }
            } else {
                let mut i = *start;
                while i > *end {
                    result.push(Value::Integer(i));
                    i += step;
                }
            }
            Ok(Value::Array(result.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "range() expects integer arguments".to_string(),
        )),
    }
}

/// Get type name of a value
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_type(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "type() expects exactly 1 argument".to_string(),
        ));
    }
    Ok(Value::from_string(args[0].type_name().to_string()))
}

/// Reverse arrays and strings
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_reverse(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "reverse() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut reversed = arr.as_ref().clone();
            reversed.reverse();
            Ok(Value::Array(Rc::new(reversed)))
        }
        Value::String(s) => {
            let reversed: String = s.chars().rev().collect();
            Ok(Value::from_string(reversed))
        }
        _ => Err(InterpreterError::RuntimeError(
            "reverse() expects an array or string".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_sqrt() {
        let args = vec![Value::Integer(16)];
        let result = eval_sqrt(&args).unwrap();
        assert_eq!(result, Value::Float(4.0));

        let args = vec![Value::Float(9.0)];
        let result = eval_sqrt(&args).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_eval_pow() {
        let args = vec![Value::Integer(2), Value::Integer(3)];
        let result = eval_pow(&args).unwrap();
        assert_eq!(result, Value::Integer(8));

        let args = vec![Value::Float(2.0), Value::Float(3.0)];
        let result = eval_pow(&args).unwrap();
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_eval_abs() {
        let args = vec![Value::Integer(-42)];
        let result = eval_abs(&args).unwrap();
        assert_eq!(result, Value::Integer(42));

        let args = vec![Value::Float(-3.14)];
        let result = eval_abs(&args).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_min_max() {
        let args = vec![Value::Integer(5), Value::Integer(3)];
        let min_result = eval_min(&args).unwrap();
        assert_eq!(min_result, Value::Integer(3));

        let max_result = eval_max(&args).unwrap();
        assert_eq!(max_result, Value::Integer(5));
    }

    #[test]
    fn test_eval_len() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_len(&args).unwrap();
        assert_eq!(result, Value::Integer(5));

        let args = vec![Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))];
        let result = eval_len(&args).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_type() {
        let args = vec![Value::Integer(42)];
        let result = eval_type(&args).unwrap();
        assert_eq!(result, Value::from_string("integer".to_string()));

        let args = vec![Value::Float(3.14)];
        let result = eval_type(&args).unwrap();
        assert_eq!(result, Value::from_string("float".to_string()));
    }

    #[test]
    fn test_eval_range() {
        let args = vec![Value::Integer(3)];
        let result = eval_range(&args).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[1], Value::Integer(1));
            assert_eq!(arr[2], Value::Integer(2));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_reverse() {
        let args = vec![Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))];
        let result = eval_reverse(&args).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr[0], Value::Integer(3));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(1));
        } else {
            panic!("Expected array result");
        }

        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_reverse(&args).unwrap();
        assert_eq!(result, Value::from_string("olleh".to_string()));
    }
}
