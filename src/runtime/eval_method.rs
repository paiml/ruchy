//! Method call evaluation module
//!
//! This module handles method dispatch and evaluation for all Value types,
//! including strings, arrays, floats, integers, `DataFrames`, and generic methods.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::Expr;
use crate::runtime::{DataFrameColumn, InterpreterError, Value};
use std::rc::Rc;

/// Main method call evaluation entry point
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_method_call<F>(
    receiver: &Expr,
    method: &str,
    args: &[Expr],
    mut eval_expr: F,
    mut eval_dataframe_filter: impl FnMut(&Value, &[Expr]) -> Result<Value, InterpreterError>,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let receiver_value = eval_expr(receiver)?;

    // Special handling for DataFrame filter method - don't pre-evaluate the condition
    if matches!(receiver_value, Value::DataFrame { .. }) && method == "filter" {
        return eval_dataframe_filter(&receiver_value, args);
    }

    let arg_values: Result<Vec<_>, _> = args.iter().map(eval_expr).collect();
    let arg_values = arg_values?;

    dispatch_method_call(&receiver_value, method, &arg_values, args.is_empty())
}

/// Dispatch method call based on receiver type
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
pub fn dispatch_method_call(
    receiver: &Value,
    method: &str,
    arg_values: &[Value],
    args_empty: bool,
) -> Result<Value, InterpreterError> {
    match receiver {
        Value::String(s) => eval_string_method(s, method, arg_values),
        Value::Array(arr) => eval_array_method_simple(arr, method, arg_values),
        Value::Float(f) => eval_float_method(*f, method, args_empty),
        Value::Integer(n) => eval_integer_method(*n, method, args_empty),
        Value::DataFrame { columns } => eval_dataframe_method_simple(columns, method, arg_values),
        _ => eval_generic_method(receiver, method, args_empty),
    }
}

/// Evaluate float methods
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_float_method(
    f: f64,
    method: &str,
    args_empty: bool,
) -> Result<Value, InterpreterError> {
    if !args_empty {
        return Err(InterpreterError::RuntimeError(format!(
            "Float method '{method}' takes no arguments"
        )));
    }

    match method {
        "sqrt" => Ok(Value::Float(f.sqrt())),
        "abs" => Ok(Value::Float(f.abs())),
        "round" => Ok(Value::Float(f.round())),
        "floor" => Ok(Value::Float(f.floor())),
        "ceil" => Ok(Value::Float(f.ceil())),
        "to_string" => Ok(Value::from_string(f.to_string())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown float method: {method}"
        ))),
    }
}

/// Evaluate integer methods
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn eval_integer_method(
    n: i64,
    method: &str,
    args_empty: bool,
) -> Result<Value, InterpreterError> {
    if !args_empty {
        return Err(InterpreterError::RuntimeError(format!(
            "Integer method '{method}' takes no arguments"
        )));
    }

    match method {
        "abs" => Ok(Value::Integer(n.abs())),
        "to_string" => Ok(Value::from_string(n.to_string())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown integer method: {method}"
        ))),
    }
}

/// Evaluate generic methods available on all types
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_generic_method(
    receiver: &Value,
    method: &str,
    args_empty: bool,
) -> Result<Value, InterpreterError> {
    if method == "to_string" && args_empty {
        Ok(Value::from_string(receiver.to_string()))
    } else {
        Err(InterpreterError::RuntimeError(format!(
            "Method '{}' not found for type {}",
            method,
            receiver.type_name()
        )))
    }
}

/// Evaluate string methods
///
/// # Complexity
/// Cyclomatic complexity: High - needs delegation to `eval_string_methods` module
pub fn eval_string_method(
    s: &Rc<str>,
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    // Delegate to the already extracted eval_string_methods module
    super::eval_string_methods::eval_string_method(s, method, args)
}

/// Simple array method evaluation (for non-higher-order methods)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_array_method_simple(
    arr: &Rc<[Value]>,
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "len" | "length" => {
            if !args.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "Array method 'len' takes no arguments".to_string(),
                ));
            }
            Ok(Value::Integer(arr.len() as i64))
        }
        "is_empty" => {
            if !args.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "Array method 'is_empty' takes no arguments".to_string(),
                ));
            }
            Ok(Value::Bool(arr.is_empty()))
        }
        _ => {
            // For complex array methods, delegate to eval_array module
            // This requires passing a function evaluator which we don't have here
            Err(InterpreterError::RuntimeError(format!(
                "Array method '{method}' requires interpreter context"
            )))
        }
    }
}

/// Simple `DataFrame` method evaluation (for non-complex operations)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_dataframe_method_simple(
    columns: &[DataFrameColumn],
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "shape" => {
            if !args.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame method 'shape' takes no arguments".to_string(),
                ));
            }
            let rows = columns.first().map_or(0, |c| c.values.len());
            let cols = columns.len();
            Ok(Value::Tuple(Rc::from(
                vec![Value::Integer(rows as i64), Value::Integer(cols as i64)].as_slice(),
            )))
        }
        "columns" => {
            if !args.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame method 'columns' takes no arguments".to_string(),
                ));
            }
            let col_names = columns
                .iter()
                .map(|c| Value::from_string(c.name.clone()))
                .collect();
            Ok(Value::from_array(col_names))
        }
        _ => {
            // For complex DataFrame operations, delegate to eval_dataframe_ops
            super::eval_dataframe_ops::eval_dataframe_method(columns, method, args)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_float_method() {
        assert_eq!(
            eval_float_method(4.0, "sqrt", true).unwrap(),
            Value::Float(2.0)
        );
        assert_eq!(
            eval_float_method(-3.5, "abs", true).unwrap(),
            Value::Float(3.5)
        );
        assert_eq!(
            eval_float_method(3.7, "round", true).unwrap(),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_float_method(3.7, "floor", true).unwrap(),
            Value::Float(3.0)
        );
        assert_eq!(
            eval_float_method(3.2, "ceil", true).unwrap(),
            Value::Float(4.0)
        );
    }

    #[test]
    fn test_eval_integer_method() {
        assert_eq!(
            eval_integer_method(-5, "abs", true).unwrap(),
            Value::Integer(5)
        );

        let result = eval_integer_method(42, "to_string", true).unwrap();
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "42"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_eval_generic_method() {
        let val = Value::Integer(42);
        let result = eval_generic_method(&val, "to_string", true).unwrap();
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "42"),
            _ => panic!("Expected string value"),
        }

        // Test error case
        assert!(eval_generic_method(&val, "unknown", true).is_err());
    }

    #[test]
    fn test_method_argument_validation() {
        assert!(eval_float_method(4.0, "sqrt", false).is_err());
        assert!(eval_integer_method(5, "abs", false).is_err());
        assert!(eval_generic_method(&Value::Nil, "type", false).is_err());
    }
}
