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
use std::sync::Arc;

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
    // EVALUATOR-001: Strip turbofish syntax from method names (centralized)
    // Example: "parse::<i32>" becomes "parse"
    // Turbofish is for type hints only, not used in runtime method lookup
    let base_method = if let Some(pos) = method.find("::") {
        &method[..pos]
    } else {
        method
    };

    match receiver {
        Value::String(s) => eval_string::eval_string_method(s, base_method, arg_values),
        Value::Array(arr) => eval_array::eval_array_method(
            arr,
            base_method,
            arg_values,
            &mut eval_function_call_value,
        ),
        Value::Float(f) => eval_float_method(*f, base_method, args_empty),
        Value::Integer(n) => eval_integer_method(*n, base_method, arg_values),
        Value::DataFrame { columns } => eval_dataframe_method(columns, base_method, arg_values),
        #[cfg(not(target_arch = "wasm32"))]
        Value::HtmlDocument(doc) => crate::runtime::eval_html_methods::eval_html_document_method(
            doc,
            base_method,
            arg_values,
        ),
        #[cfg(not(target_arch = "wasm32"))]
        Value::HtmlElement(element) => crate::runtime::eval_html_methods::eval_html_element_method(
            element,
            base_method,
            arg_values,
        ),
        Value::Object(obj) => eval_object_method(obj, base_method, arg_values),
        _ => eval_generic_method(receiver, base_method, args_empty),
    }
}

// Type-specific method evaluators (complexity <= 8 each)

/// Evaluate float methods with mathematical operations
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_float_method(f: f64, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
    // Issue #91: Special case for powf - suggest ** operator instead
    if method == "powf" {
        return Err(InterpreterError::RuntimeError(
            "Float method 'powf' not available. Use ** operator for exponentiation (e.g., 2.0 ** 3.0)".to_string()
        ));
    }

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

/// Helper: Validate no-argument method call
fn require_no_args(method: &str, arg_values: &[Value]) -> Result<(), InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(format!(
            "Integer method '{method}' takes no arguments"
        )));
    }
    Ok(())
}

/// Helper: Evaluate integer pow method
fn eval_integer_pow(n: i64, arg_values: &[Value]) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(format!(
            "Integer method 'pow' requires exactly 1 argument, got {}",
            arg_values.len()
        )));
    }
    match &arg_values[0] {
        Value::Integer(exp) => {
            if *exp < 0 {
                return Err(InterpreterError::RuntimeError(
                    "Integer pow() exponent must be non-negative".to_string(),
                ));
            }
            Ok(Value::Integer(n.pow(*exp as u32)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Integer pow() requires integer exponent, got {}",
            arg_values[0].type_name()
        ))),
    }
}

/// Evaluate integer methods with mathematical operations
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
/// Cognitive complexity: reduced via helpers
fn eval_integer_method(
    n: i64,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "abs" => {
            require_no_args(method, arg_values)?;
            Ok(Value::Integer(n.abs()))
        }
        "sqrt" => {
            require_no_args(method, arg_values)?;
            Ok(Value::Float((n as f64).sqrt()))
        }
        "to_float" => {
            require_no_args(method, arg_values)?;
            Ok(Value::Float(n as f64))
        }
        "to_string" => {
            require_no_args(method, arg_values)?;
            Ok(Value::from_string(n.to_string()))
        }
        "signum" => {
            require_no_args(method, arg_values)?;
            Ok(Value::Integer(n.signum()))
        }
        "pow" => eval_integer_pow(n, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown integer method: {method}"
        ))),
    }
}

/// Helper: Try to dispatch builtin function from object method
fn try_dispatch_builtin(
    obj: &std::collections::HashMap<String, Value>,
    method: &str,
    arg_values: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    let Some(Value::String(builtin_marker)) = obj.get(method) else {
        return Ok(None);
    };

    if !builtin_marker.starts_with("__builtin_") {
        return Ok(None);
    }

    match crate::runtime::eval_builtin::eval_builtin_function(builtin_marker, arg_values)? {
        Some(value) => Ok(Some(value)),
        None => Err(InterpreterError::RuntimeError(format!(
            "Unknown builtin function: {builtin_marker}"
        ))),
    }
}

/// Evaluate methods on `Object` (`HashMap`) types
///
/// Dispatches based on `__type` marker to appropriate handler
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
/// Cognitive complexity: reduced via helpers
fn eval_object_method(
    obj: &std::collections::HashMap<String, Value>,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    // Check __type marker to route to appropriate handler
    if let Some(Value::String(type_name)) = obj.get("__type") {
        return match &**type_name {
            "Command" => eval_command_method(obj, method, arg_values),
            "ExitStatus" => eval_exit_status_method(obj, method, arg_values),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown object type: {type_name}"
            ))),
        };
    }

    // Issue #96: Fallback for module functions (std::env, std::fs, etc.)
    if let Some(value) = try_dispatch_builtin(obj, method, arg_values)? {
        return Ok(value);
    }

    Err(InterpreterError::RuntimeError(
        "Object is missing __type marker".to_string(),
    ))
}

/// Evaluate methods on Command objects (RUNTIME-090, Issue #75)
///
/// # Complexity
/// Helper: Build `std::process::Command` from Command object
/// Cyclomatic complexity: 3 (A+ standard)
#[cfg(not(target_arch = "wasm32"))]
fn build_command_from_obj(
    obj: &std::collections::HashMap<String, Value>,
) -> Result<std::process::Command, InterpreterError> {
    let program = match obj.get("program") {
        Some(Value::String(p)) => &**p,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "Command object missing 'program' field".to_string(),
            ))
        }
    };

    let args = match obj.get("args") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => Arc::new([]),
    };

    let mut command = std::process::Command::new(program);
    for arg in args.iter() {
        if let Value::String(arg_str) = arg {
            command.arg(&**arg_str);
        }
    }

    Ok(command)
}

/// Evaluate methods on Command objects (post-refactoring)
/// Cyclomatic complexity: 9 (A+ standard: ≤10)
/// Reduced from 15 via Extract Function refactoring (Issue #93)
#[cfg(not(target_arch = "wasm32"))]
fn eval_command_method(
    obj: &std::collections::HashMap<String, Value>,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "arg" => {
            // Mutate the command object by adding an argument
            if arg_values.len() != 1 {
                return Err(InterpreterError::RuntimeError(
                    "Command.arg() requires exactly 1 argument".to_string(),
                ));
            }
            if let Value::String(arg_str) = &arg_values[0] {
                let mut new_obj = obj.clone();
                if let Some(Value::Array(args)) = new_obj.get("args").cloned() {
                    let mut new_args = args.to_vec();
                    new_args.push(Value::from_string(arg_str.to_string()));
                    new_obj.insert("args".to_string(), Value::Array(Arc::from(new_args)));
                }
                Ok(Value::Object(Arc::new(new_obj)))
            } else {
                Err(InterpreterError::RuntimeError(
                    "Command.arg() expects a string argument".to_string(),
                ))
            }
        }
        "status" => {
            // Execute the command and return Result<ExitStatus, Error> (Issue #85)
            let mut command = build_command_from_obj(obj)?;

            match command.status() {
                Ok(status) => {
                    // Create ExitStatus object with success() method
                    let mut status_obj = std::collections::HashMap::new();
                    status_obj.insert(
                        "__type".to_string(),
                        Value::from_string("ExitStatus".to_string()),
                    );
                    status_obj.insert("success".to_string(), Value::from_bool(status.success()));
                    status_obj.insert(
                        "code".to_string(),
                        Value::Integer(i64::from(status.code().unwrap_or(-1))),
                    );

                    // Return Result::Ok(status_obj)
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Ok".to_string(),
                        data: Some(vec![Value::Object(Arc::new(status_obj))]),
                    })
                }
                Err(e) => {
                    // Return Result::Err(error_string)
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Err".to_string(),
                        data: Some(vec![Value::from_string(e.to_string())]),
                    })
                }
            }
        }
        "output" => {
            // Execute the command and return Result<Output, Error> (Issue #85)
            let mut command = build_command_from_obj(obj)?;

            match command.output() {
                Ok(output) => {
                    // Create Output object with stdout, stderr, status fields
                    let mut output_obj = std::collections::HashMap::new();
                    output_obj.insert(
                        "__type".to_string(),
                        Value::from_string("Output".to_string()),
                    );

                    // Store stdout as byte array
                    let stdout_bytes: Vec<Value> =
                        output.stdout.iter().map(|b| Value::Byte(*b)).collect();
                    output_obj.insert("stdout".to_string(), Value::Array(Arc::from(stdout_bytes)));

                    // Store stderr as byte array
                    let stderr_bytes: Vec<Value> =
                        output.stderr.iter().map(|b| Value::Byte(*b)).collect();
                    output_obj.insert("stderr".to_string(), Value::Array(Arc::from(stderr_bytes)));

                    // Store exit status (success/code)
                    let mut status_obj = std::collections::HashMap::new();
                    status_obj.insert(
                        "__type".to_string(),
                        Value::from_string("ExitStatus".to_string()),
                    );
                    status_obj.insert(
                        "success".to_string(),
                        Value::from_bool(output.status.success()),
                    );
                    status_obj.insert(
                        "code".to_string(),
                        Value::Integer(i64::from(output.status.code().unwrap_or(-1))),
                    );
                    output_obj.insert("status".to_string(), Value::Object(Arc::new(status_obj)));

                    // Return Result::Ok(output_obj)
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Ok".to_string(),
                        data: Some(vec![Value::Object(Arc::new(output_obj))]),
                    })
                }
                Err(e) => {
                    // Return Result::Err(error_string)
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Err".to_string(),
                        data: Some(vec![Value::from_string(e.to_string())]),
                    })
                }
            }
        }
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown Command method: {method}"
        ))),
    }
}

/// Stub for WASM - Command methods not available
#[cfg(target_arch = "wasm32")]
fn eval_command_method(
    _obj: &std::collections::HashMap<String, Value>,
    method: &str,
    _arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    Err(InterpreterError::RuntimeError(format!(
        "Command method '{method}' not available in WASM"
    )))
}

/// Evaluate methods on `ExitStatus` objects (Issue #85)
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_exit_status_method(
    obj: &std::collections::HashMap<String, Value>,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "success" => {
            if !arg_values.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "ExitStatus.success() takes no arguments".to_string(),
                ));
            }
            // Return the success boolean field
            match obj.get("success") {
                Some(value) => Ok(value.clone()),
                None => Err(InterpreterError::RuntimeError(
                    "ExitStatus object missing 'success' field".to_string(),
                )),
            }
        }
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown ExitStatus method: {method}"
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
    Ok(Value::from_array(column_names))
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

    Ok(Value::Array(Arc::from(
        vec![Value::Integer(rows as i64), Value::Integer(cols as i64)].as_slice(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // FLOAT METHOD TESTS
    // =========================================================================

    #[test]
    fn test_float_sqrt() {
        assert_eq!(
            eval_float_method(9.0, "sqrt", true).unwrap(),
            Value::Float(3.0)
        );
        assert_eq!(
            eval_float_method(0.0, "sqrt", true).unwrap(),
            Value::Float(0.0)
        );
    }

    #[test]
    fn test_float_abs() {
        assert_eq!(
            eval_float_method(-5.5, "abs", true).unwrap(),
            Value::Float(5.5)
        );
        assert_eq!(
            eval_float_method(5.5, "abs", true).unwrap(),
            Value::Float(5.5)
        );
    }

    #[test]
    fn test_float_round() {
        assert_eq!(
            eval_float_method(3.7, "round", true).unwrap(),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_float_method(3.2, "round", true).unwrap(),
            Value::Float(3.0)
        );
    }

    #[test]
    fn test_float_floor() {
        assert_eq!(
            eval_float_method(3.7, "floor", true).unwrap(),
            Value::Float(3.0)
        );
        assert_eq!(
            eval_float_method(-3.7, "floor", true).unwrap(),
            Value::Float(-4.0)
        );
    }

    #[test]
    fn test_float_ceil() {
        assert_eq!(
            eval_float_method(3.2, "ceil", true).unwrap(),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_float_method(-3.2, "ceil", true).unwrap(),
            Value::Float(-3.0)
        );
    }

    #[test]
    fn test_float_trig() {
        // sin, cos, tan
        assert!((eval_float_method(0.0, "sin", true).unwrap() == Value::Float(0.0)));
        assert!((eval_float_method(0.0, "cos", true).unwrap() == Value::Float(1.0)));
        assert!((eval_float_method(0.0, "tan", true).unwrap() == Value::Float(0.0)));
    }

    #[test]
    fn test_float_log() {
        assert_eq!(
            eval_float_method(std::f64::consts::E, "ln", true).unwrap(),
            Value::Float(1.0)
        );
        assert_eq!(
            eval_float_method(10.0, "log10", true).unwrap(),
            Value::Float(1.0)
        );
        assert_eq!(
            eval_float_method(0.0, "exp", true).unwrap(),
            Value::Float(1.0)
        );
    }

    #[test]
    fn test_float_to_string() {
        let result = eval_float_method(std::f64::consts::PI, "to_string", true).unwrap();
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), &std::f64::consts::PI.to_string()),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_float_powf_error() {
        let result = eval_float_method(2.0, "powf", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Use ** operator"));
    }

    #[test]
    fn test_float_with_args_error() {
        let result = eval_float_method(5.0, "sqrt", false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("takes no arguments"));
    }

    #[test]
    fn test_float_unknown_method() {
        let result = eval_float_method(5.0, "unknown", true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown float method"));
    }

    // =========================================================================
    // INTEGER METHOD TESTS
    // =========================================================================

    #[test]
    fn test_integer_abs() {
        assert_eq!(
            eval_integer_method(-42, "abs", &[]).unwrap(),
            Value::Integer(42)
        );
        assert_eq!(
            eval_integer_method(42, "abs", &[]).unwrap(),
            Value::Integer(42)
        );
        assert_eq!(
            eval_integer_method(0, "abs", &[]).unwrap(),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(
            eval_integer_method(16, "sqrt", &[]).unwrap(),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_integer_method(0, "sqrt", &[]).unwrap(),
            Value::Float(0.0)
        );
    }

    #[test]
    fn test_integer_to_float() {
        assert_eq!(
            eval_integer_method(42, "to_float", &[]).unwrap(),
            Value::Float(42.0)
        );
        assert_eq!(
            eval_integer_method(-5, "to_float", &[]).unwrap(),
            Value::Float(-5.0)
        );
    }

    #[test]
    fn test_integer_to_string() {
        let result = eval_integer_method(123, "to_string", &[]).unwrap();
        assert_eq!(result, Value::from_string("123".to_string()));
    }

    #[test]
    fn test_integer_signum() {
        assert_eq!(
            eval_integer_method(42, "signum", &[]).unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            eval_integer_method(-42, "signum", &[]).unwrap(),
            Value::Integer(-1)
        );
        assert_eq!(
            eval_integer_method(0, "signum", &[]).unwrap(),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_integer_pow() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(3)]).unwrap();
        assert_eq!(result, Value::Integer(8));

        let result = eval_integer_method(5, "pow", &[Value::Integer(0)]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_integer_pow_negative_exponent_error() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(-1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-negative"));
    }

    #[test]
    fn test_integer_pow_wrong_type_error() {
        let result = eval_integer_method(2, "pow", &[Value::Float(3.0)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("integer exponent"));
    }

    #[test]
    fn test_integer_pow_wrong_arg_count() {
        let result = eval_integer_method(2, "pow", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1 argument"));
    }

    #[test]
    fn test_integer_abs_with_args_error() {
        let result = eval_integer_method(42, "abs", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("takes no arguments"));
    }

    #[test]
    fn test_integer_unknown_method() {
        let result = eval_integer_method(42, "unknown", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown integer method"));
    }

    // =========================================================================
    // DATAFRAME METHOD TESTS
    // =========================================================================

    #[test]
    fn test_dataframe_count() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        assert_eq!(
            eval_dataframe_count(&columns, &[]).unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_dataframe_sum() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        assert_eq!(
            eval_dataframe_sum(&columns, &[]).unwrap(),
            Value::Float(6.0)
        );
    }

    #[test]
    fn test_dataframe_sum_mixed_types() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Float(2.5), Value::Integer(3)],
        }];
        assert_eq!(
            eval_dataframe_sum(&columns, &[]).unwrap(),
            Value::Float(6.5)
        );
    }

    #[test]
    fn test_dataframe_mean() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        assert_eq!(
            eval_dataframe_mean(&columns, &[]).unwrap(),
            Value::Float(2.0)
        );
    }

    #[test]
    fn test_dataframe_max() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(5), Value::Integer(3)],
        }];
        assert_eq!(
            eval_dataframe_max(&columns, &[]).unwrap(),
            Value::Float(5.0)
        );
    }

    #[test]
    fn test_dataframe_min() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(5), Value::Integer(1), Value::Integer(3)],
        }];
        assert_eq!(
            eval_dataframe_min(&columns, &[]).unwrap(),
            Value::Float(1.0)
        );
    }

    #[test]
    fn test_dataframe_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(2)],
            },
        ];
        let result = eval_dataframe_columns(&columns, &[]).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], Value::from_string("a".to_string()));
                assert_eq!(arr[1], Value::from_string("b".to_string()));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_dataframe_shape() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(4), Value::Integer(5), Value::Integer(6)],
            },
        ];
        let result = eval_dataframe_shape(&columns, &[]).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr[0], Value::Integer(3)); // rows
                assert_eq!(arr[1], Value::Integer(2)); // columns
            }
            _ => panic!("Expected Array"),
        }
    }

    // =========================================================================
    // GENERIC METHOD TESTS
    // =========================================================================

    #[test]
    fn test_generic_to_string_bool() {
        let value = Value::Bool(true);
        assert_eq!(
            eval_generic_method(&value, "to_string", true).unwrap(),
            Value::from_string("true".to_string())
        );
    }

    #[test]
    fn test_generic_to_string_nil() {
        let value = Value::Nil;
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("nil".to_string()));
    }

    #[test]
    fn test_generic_unknown_method() {
        let value = Value::Bool(true);
        let result = eval_generic_method(&value, "unknown", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // =========================================================================
    // DISPATCH TESTS (dispatch_method_call)
    // =========================================================================

    #[test]
    fn test_dispatch_turbofish_stripping() {
        // Test that turbofish syntax is stripped from method names
        // Example: "parse::<i32>" becomes "parse"
        let s = Arc::from("42");
        let value = Value::String(s);

        // Mock closures
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        // This should work because turbofish is stripped
        let result = dispatch_method_call(
            &value,
            "to_string::<String>", // With turbofish
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
    }
}
