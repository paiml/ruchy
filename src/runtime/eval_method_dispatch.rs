//! Method dispatch evaluation module
//!
//! This module handles evaluation of all method calls in the interpreter.
//! Provides a centralized dispatch system for different value types.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::Expr;
use crate::runtime::eval_array;
use crate::runtime::eval_string;
use crate::runtime::interpreter::DataFrameColumn;
use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate a method call on a receiver value
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_method_call<F1, F2, F3>(
    receiver_value: &Value,
    method: &str,
    arg_values: &[Value],
    args_empty: bool,
    eval_function_call_value: F1,
    eval_dataframe_filter_method: F2,
    eval_expr_with_column_context: F3,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
    F2: Fn(&Value, &[Expr]) -> Result<Value, InterpreterError>,
    F3: Fn(&Expr, &[DataFrameColumn], usize) -> Result<Value, InterpreterError>,
{
    dispatch_method_call(
        receiver_value,
        method,
        arg_values,
        args_empty,
        eval_function_call_value,
        eval_dataframe_filter_method,
        eval_expr_with_column_context,
    )
}

/// Dispatch method call to appropriate handler based on receiver type
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn dispatch_method_call<F1, F2, F3>(
    receiver: &Value,
    method: &str,
    arg_values: &[Value],
    args_empty: bool,
    mut eval_function_call_value: F1,
    _eval_dataframe_filter_method: F2,
    _eval_expr_with_column_context: F3,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
    F2: Fn(&Value, &[Expr]) -> Result<Value, InterpreterError>,
    F3: Fn(&Expr, &[DataFrameColumn], usize) -> Result<Value, InterpreterError>,
{
    match receiver {
        Value::String(s) => eval_string::eval_string_method(s, method, arg_values),
        Value::Array(arr) => {
            eval_array::eval_array_method(arr, method, arg_values, &mut eval_function_call_value)
        }
        Value::Float(f) => eval_float_method(*f, method, args_empty),
        Value::Integer(n) => eval_integer_method(*n, method, args_empty),
        Value::DataFrame { columns } => eval_dataframe_method(columns, method, arg_values),
        _ => eval_generic_method(receiver, method, args_empty),
    }
}

// Type-specific method evaluators (complexity <= 8 each)

/// Evaluate float methods with mathematical operations
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_float_method(f: f64, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
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
        "sin" => Ok(Value::Float(f.sin())),
        "cos" => Ok(Value::Float(f.cos())),
        "tan" => Ok(Value::Float(f.tan())),
        "ln" => Ok(Value::Float(f.ln())),
        "log10" => Ok(Value::Float(f.log10())),
        "exp" => Ok(Value::Float(f.exp())),
        "to_string" => Ok(Value::from_string(f.to_string())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown float method: {method}"
        ))),
    }
}

/// Evaluate integer methods with mathematical operations
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_integer_method(n: i64, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
    if !args_empty {
        return Err(InterpreterError::RuntimeError(format!(
            "Integer method '{method}' takes no arguments"
        )));
    }

    match method {
        "abs" => Ok(Value::Integer(n.abs())),
        "sqrt" => Ok(Value::Float((n as f64).sqrt())),
        "to_float" => Ok(Value::Float(n as f64)),
        "to_string" => Ok(Value::from_string(n.to_string())),
        "signum" => Ok(Value::Integer(n.signum())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown integer method: {method}"
        ))),
    }
}

/// Evaluate generic methods available on all types
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_generic_method(
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

/// Evaluate `DataFrame` methods for data analysis operations
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn eval_dataframe_method(
    columns: &[DataFrameColumn],
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "select" => eval_dataframe_select(columns, arg_values),
        "sum" => eval_dataframe_sum(columns, arg_values),
        "count" => eval_dataframe_count(columns, arg_values),
        "mean" => eval_dataframe_mean(columns, arg_values),
        "max" => eval_dataframe_max(columns, arg_values),
        "min" => eval_dataframe_min(columns, arg_values),
        "columns" => eval_dataframe_columns(columns, arg_values),
        "shape" => eval_dataframe_shape(columns, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown DataFrame method: {method}"
        ))),
    }
}

// DataFrame method implementations (complexity <= 5 each)

fn eval_dataframe_select(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.select() requires exactly 1 argument (column_name)".to_string(),
        ));
    }

    if let Value::String(column_name) = &arg_values[0] {
        for col in columns {
            if col.name == **column_name {
                return Ok(Value::DataFrame {
                    columns: vec![col.clone()],
                });
            }
        }
        Err(InterpreterError::RuntimeError(format!(
            "Column '{column_name}' not found in DataFrame"
        )))
    } else {
        Err(InterpreterError::RuntimeError(
            "DataFrame.select() expects column name as string".to_string(),
        ))
    }
}

fn eval_dataframe_sum(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.sum() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    for col in columns {
        for value in &col.values {
            match value {
                Value::Integer(i) => total += *i as f64,
                Value::Float(f) => total += f,
                _ => {} // Skip non-numeric values
            }
        }
    }
    Ok(Value::Float(total))
}

fn eval_dataframe_count(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.count() takes no arguments".to_string(),
        ));
    }

    let count = if columns.is_empty() {
        0
    } else {
        columns[0].values.len()
    };
    Ok(Value::Integer(count as i64))
}

fn eval_dataframe_mean(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.mean() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    let mut count = 0;
    for col in columns {
        for value in &col.values {
            match value {
                Value::Integer(i) => {
                    total += *i as f64;
                    count += 1;
                }
                Value::Float(f) => {
                    total += f;
                    count += 1;
                }
                _ => {} // Skip non-numeric values
            }
        }
    }

    if count == 0 {
        Ok(Value::Nil)
    } else {
        Ok(Value::Float(total / f64::from(count)))
    }
}

fn eval_dataframe_max(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.max() takes no arguments".to_string(),
        ));
    }

    let mut max_val: Option<f64> = None;
    for col in columns {
        for value in &col.values {
            let val = match value {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => continue,
            };
            max_val = Some(max_val.map_or(val, |current| val.max(current)));
        }
    }

    match max_val {
        Some(val) => Ok(Value::Float(val)),
        None => Ok(Value::Nil),
    }
}

fn eval_dataframe_min(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.min() takes no arguments".to_string(),
        ));
    }

    let mut min_val: Option<f64> = None;
    for col in columns {
        for value in &col.values {
            let val = match value {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => continue,
            };
            min_val = Some(min_val.map_or(val, |current| val.min(current)));
        }
    }

    match min_val {
        Some(val) => Ok(Value::Float(val)),
        None => Ok(Value::Nil),
    }
}

fn eval_dataframe_columns(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.columns() takes no arguments".to_string(),
        ));
    }

    let column_names: Vec<Value> = columns
        .iter()
        .map(|col| Value::from_string(col.name.clone()))
        .collect();
    Ok(Value::Array(Rc::new(column_names)))
}

fn eval_dataframe_shape(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.shape() takes no arguments".to_string(),
        ));
    }

    let rows = if columns.is_empty() {
        0
    } else {
        columns[0].values.len()
    };
    let cols = columns.len();

    Ok(Value::Array(Rc::new(vec![
        Value::Integer(rows as i64),
        Value::Integer(cols as i64),
    ])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_methods() {
        let result = eval_float_method(9.0, "sqrt", true).unwrap();
        assert_eq!(result, Value::Float(3.0));

        let result = eval_float_method(-5.5, "abs", true).unwrap();
        assert_eq!(result, Value::Float(5.5));

        let result = eval_float_method(3.7, "floor", true).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_integer_methods() {
        let result = eval_integer_method(-42, "abs", true).unwrap();
        assert_eq!(result, Value::Integer(42));

        let result = eval_integer_method(123, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("123".to_string()));
    }

    #[test]
    fn test_generic_methods() {
        let value = Value::Bool(true);
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("true".to_string()));
    }

    #[test]
    fn test_dataframe_count() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        let result = eval_dataframe_count(&columns, &[]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_dataframe_sum() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        let result = eval_dataframe_sum(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(6.0));
    }
}
