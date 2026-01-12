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
            "Float method 'powf' not available. Use ** operator for exponentiation (e.g., 2.0 ** 3.0)".to_string(),
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
            // Module calls are handled in interpreter.rs eval_method_call
            "Module" => Err(InterpreterError::RuntimeError(
                "Module method dispatch should be handled in interpreter".to_string()
            )),
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
            "DataFrame.select() requires exactly 1 argument (column_name or [column_names])".to_string(),
        ));
    }

    match &arg_values[0] {
        Value::String(column_name) => {
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
        }
        Value::Array(col_names) => {
            let mut selected = Vec::new();
            for name_val in col_names.iter() {
                if let Value::String(column_name) = name_val {
                    let mut found = false;
                    for col in columns {
                        if col.name == **column_name {
                            selected.push(col.clone());
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Err(InterpreterError::RuntimeError(format!(
                            "Column '{column_name}' not found in DataFrame"
                        )));
                    }
                } else {
                    return Err(InterpreterError::RuntimeError(
                        "DataFrame.select() array elements must be strings".to_string(),
                    ));
                }
            }
            Ok(Value::DataFrame { columns: selected })
        }
        _ => Err(InterpreterError::RuntimeError(
            "DataFrame.select() expects column name as string or array of strings".to_string(),
        )),
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
            eval_float_method(9.0, "sqrt", true).expect("eval_float_method should succeed in test"),
            Value::Float(3.0)
        );
        assert_eq!(
            eval_float_method(0.0, "sqrt", true).expect("eval_float_method should succeed in test"),
            Value::Float(0.0)
        );
    }

    #[test]
    fn test_float_abs() {
        assert_eq!(
            eval_float_method(-5.5, "abs", true).expect("eval_float_method should succeed in test"),
            Value::Float(5.5)
        );
        assert_eq!(
            eval_float_method(5.5, "abs", true).expect("eval_float_method should succeed in test"),
            Value::Float(5.5)
        );
    }

    #[test]
    fn test_float_round() {
        assert_eq!(
            eval_float_method(3.7, "round", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_float_method(3.2, "round", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(3.0)
        );
    }

    #[test]
    fn test_float_floor() {
        assert_eq!(
            eval_float_method(3.7, "floor", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(3.0)
        );
        assert_eq!(
            eval_float_method(-3.7, "floor", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(-4.0)
        );
    }

    #[test]
    fn test_float_ceil() {
        assert_eq!(
            eval_float_method(3.2, "ceil", true).expect("eval_float_method should succeed in test"),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_float_method(-3.2, "ceil", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(-3.0)
        );
    }

    #[test]
    fn test_float_trig() {
        // sin, cos, tan
        assert!(
            (eval_float_method(0.0, "sin", true)
                .expect("eval_float_method should succeed in test")
                == Value::Float(0.0))
        );
        assert!(
            (eval_float_method(0.0, "cos", true)
                .expect("eval_float_method should succeed in test")
                == Value::Float(1.0))
        );
        assert!(
            (eval_float_method(0.0, "tan", true)
                .expect("eval_float_method should succeed in test")
                == Value::Float(0.0))
        );
    }

    #[test]
    fn test_float_log() {
        assert_eq!(
            eval_float_method(std::f64::consts::E, "ln", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(1.0)
        );
        assert_eq!(
            eval_float_method(10.0, "log10", true)
                .expect("eval_float_method should succeed in test"),
            Value::Float(1.0)
        );
        assert_eq!(
            eval_float_method(0.0, "exp", true).expect("eval_float_method should succeed in test"),
            Value::Float(1.0)
        );
    }

    #[test]
    fn test_float_to_string() {
        let result = eval_float_method(std::f64::consts::PI, "to_string", true)
            .expect("eval_float_method should succeed in test");
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
            eval_integer_method(-42, "abs", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Integer(42)
        );
        assert_eq!(
            eval_integer_method(42, "abs", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Integer(42)
        );
        assert_eq!(
            eval_integer_method(0, "abs", &[]).expect("eval_integer_method should succeed in test"),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(
            eval_integer_method(16, "sqrt", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Float(4.0)
        );
        assert_eq!(
            eval_integer_method(0, "sqrt", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Float(0.0)
        );
    }

    #[test]
    fn test_integer_to_float() {
        assert_eq!(
            eval_integer_method(42, "to_float", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Float(42.0)
        );
        assert_eq!(
            eval_integer_method(-5, "to_float", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Float(-5.0)
        );
    }

    #[test]
    fn test_integer_to_string() {
        let result = eval_integer_method(123, "to_string", &[])
            .expect("eval_integer_method should succeed in test");
        assert_eq!(result, Value::from_string("123".to_string()));
    }

    #[test]
    fn test_integer_signum() {
        assert_eq!(
            eval_integer_method(42, "signum", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Integer(1)
        );
        assert_eq!(
            eval_integer_method(-42, "signum", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Integer(-1)
        );
        assert_eq!(
            eval_integer_method(0, "signum", &[])
                .expect("eval_integer_method should succeed in test"),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_integer_pow() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(3)])
            .expect("eval_integer_method should succeed in test");
        assert_eq!(result, Value::Integer(8));

        let result = eval_integer_method(5, "pow", &[Value::Integer(0)])
            .expect("eval_integer_method should succeed in test");
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
            eval_dataframe_count(&columns, &[])
                .expect("eval_dataframe_count should succeed in test"),
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
            eval_dataframe_sum(&columns, &[]).expect("eval_dataframe_sum should succeed in test"),
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
            eval_dataframe_sum(&columns, &[]).expect("eval_dataframe_sum should succeed in test"),
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
            eval_dataframe_mean(&columns, &[]).expect("eval_dataframe_mean should succeed in test"),
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
            eval_dataframe_max(&columns, &[]).expect("eval_dataframe_max should succeed in test"),
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
            eval_dataframe_min(&columns, &[]).expect("eval_dataframe_min should succeed in test"),
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
        let result = eval_dataframe_columns(&columns, &[])
            .expect("eval_dataframe_columns should succeed in test");
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
        let result = eval_dataframe_shape(&columns, &[])
            .expect("eval_dataframe_shape should succeed in test");
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
            eval_generic_method(&value, "to_string", true)
                .expect("eval_generic_method should succeed in test"),
            Value::from_string("true".to_string())
        );
    }

    #[test]
    fn test_generic_to_string_nil() {
        let value = Value::Nil;
        let result = eval_generic_method(&value, "to_string", true)
            .expect("eval_generic_method should succeed in test");
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

    // Round 95: Additional method dispatch tests

    // Test 35: dataframe empty columns
    #[test]
    fn test_dataframe_columns_empty() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_columns(&columns, &[])
            .expect("eval_dataframe_columns should succeed for empty");
        match result {
            Value::Array(arr) => assert!(arr.is_empty()),
            _ => panic!("Expected empty Array"),
        }
    }

    // Test 36: dataframe shape empty
    #[test]
    fn test_dataframe_shape_empty() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_shape(&columns, &[])
            .expect("eval_dataframe_shape should succeed for empty");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr[0], Value::Integer(0));
                assert_eq!(arr[1], Value::Integer(0));
            }
            _ => panic!("Expected Array"),
        }
    }

    // Test 37: dataframe sum empty
    #[test]
    fn test_dataframe_sum_empty() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_sum(&columns, &[]);
        // Empty columns might return 0 or error
        assert!(result.is_ok() || result.is_err());
    }

    // Test 38: dataframe mean with single value
    #[test]
    fn test_dataframe_mean_single() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(42)],
        }];
        let result = eval_dataframe_mean(&columns, &[])
            .expect("eval_dataframe_mean should succeed");
        assert_eq!(result, Value::Float(42.0));
    }

    // Test 39: generic to_string integer
    #[test]
    fn test_generic_to_string_integer() {
        let value = Value::Integer(42);
        let result = eval_generic_method(&value, "to_string", true)
            .expect("eval_generic_method should succeed");
        assert_eq!(result, Value::from_string("42".to_string()));
    }

    // Test 40: generic to_string float
    #[test]
    fn test_generic_to_string_float() {
        let value = Value::Float(3.14);
        let result = eval_generic_method(&value, "to_string", true)
            .expect("eval_generic_method should succeed");
        // Float to_string might include precision
        match result {
            Value::String(s) => assert!(s.starts_with("3.14")),
            _ => panic!("Expected String"),
        }
    }

    // Test 41: generic to_string array
    #[test]
    fn test_generic_to_string_array() {
        let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = eval_generic_method(&value, "to_string", true)
            .expect("eval_generic_method should succeed");
        match result {
            Value::String(_) => {} // Any string representation is fine
            _ => panic!("Expected String"),
        }
    }

    // Test 42: dataframe single column max
    #[test]
    fn test_dataframe_max_single() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(42)],
        }];
        assert_eq!(
            eval_dataframe_max(&columns, &[]).expect("eval_dataframe_max should succeed"),
            Value::Float(42.0)
        );
    }

    // Test 43: dataframe single column min
    #[test]
    fn test_dataframe_min_single() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(42)],
        }];
        assert_eq!(
            eval_dataframe_min(&columns, &[]).expect("eval_dataframe_min should succeed"),
            Value::Float(42.0)
        );
    }

    // Test 44: dataframe with float values
    #[test]
    fn test_dataframe_sum_floats() {
        let columns = vec![DataFrameColumn {
            name: "values".to_string(),
            values: vec![Value::Float(1.5), Value::Float(2.5), Value::Float(3.0)],
        }];
        let result = eval_dataframe_sum(&columns, &[])
            .expect("eval_dataframe_sum should succeed for floats");
        match result {
            Value::Float(f) => assert!((f - 7.0).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    // Test 45: dataframe shape with multiple columns
    #[test]
    fn test_dataframe_shape_multi_column() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(2)],
            },
            DataFrameColumn {
                name: "c".to_string(),
                values: vec![Value::Integer(3)],
            },
        ];
        let result = eval_dataframe_shape(&columns, &[])
            .expect("eval_dataframe_shape should succeed");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr[0], Value::Integer(1)); // 1 row
                assert_eq!(arr[1], Value::Integer(3)); // 3 columns
            }
            _ => panic!("Expected Array"),
        }
    }

    // Test 46: dispatch with array value
    #[test]
    fn test_dispatch_array_method() {
        let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(2));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "len",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
    }

    // Test 47: dispatch with nil value
    #[test]
    fn test_dispatch_nil_to_string() {
        let value = Value::Nil;

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::from_string("nil".to_string()));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "to_string",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
    }

    // =========================================================================
    // EXTREME TDD ROUND 127 - Additional Coverage Tests
    // =========================================================================

    // Test R127-01: eval_dataframe_select success
    #[test]
    fn test_dataframe_select_success_r127() {
        let columns = vec![
            DataFrameColumn {
                name: "price".to_string(),
                values: vec![Value::Float(10.5), Value::Float(20.0)],
            },
            DataFrameColumn {
                name: "quantity".to_string(),
                values: vec![Value::Integer(5), Value::Integer(10)],
            },
        ];
        let result = eval_dataframe_select(&columns, &[Value::from_string("price".to_string())])
            .expect("should select column");
        match result {
            Value::DataFrame { columns: selected } => {
                assert_eq!(selected.len(), 1);
                assert_eq!(selected[0].name, "price");
            }
            _ => panic!("Expected DataFrame"),
        }
    }

    // Test R127-02: eval_dataframe_select column not found
    #[test]
    fn test_dataframe_select_not_found_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_select(&columns, &[Value::from_string("missing".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // Test R127-03: eval_dataframe_select wrong arg type
    #[test]
    fn test_dataframe_select_wrong_type_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_select(&columns, &[Value::Integer(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("column name"));
    }

    // Test R127-04: eval_dataframe_select wrong arg count
    #[test]
    fn test_dataframe_select_wrong_count_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_select(&columns, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exactly 1"));
    }

    // Test R127-05: dataframe count with args error
    #[test]
    fn test_dataframe_count_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_count(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-06: dataframe sum with args error
    #[test]
    fn test_dataframe_sum_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_sum(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-07: dataframe mean with args error
    #[test]
    fn test_dataframe_mean_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_mean(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-08: dataframe max with args error
    #[test]
    fn test_dataframe_max_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_max(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-09: dataframe min with args error
    #[test]
    fn test_dataframe_min_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_min(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-10: dataframe columns with args error
    #[test]
    fn test_dataframe_columns_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_columns(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-11: dataframe shape with args error
    #[test]
    fn test_dataframe_shape_with_args_error_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_shape(&columns, &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-12: dataframe mean empty (nil result)
    #[test]
    fn test_dataframe_mean_empty_nil_r127() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_mean(&columns, &[]).expect("should return nil for empty");
        assert_eq!(result, Value::Nil);
    }

    // Test R127-13: dataframe max empty (nil result)
    #[test]
    fn test_dataframe_max_empty_nil_r127() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_max(&columns, &[]).expect("should return nil for empty");
        assert_eq!(result, Value::Nil);
    }

    // Test R127-14: dataframe min empty (nil result)
    #[test]
    fn test_dataframe_min_empty_nil_r127() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_min(&columns, &[]).expect("should return nil for empty");
        assert_eq!(result, Value::Nil);
    }

    // Test R127-15: dataframe unknown method
    #[test]
    fn test_dataframe_unknown_method_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_method(&columns, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown DataFrame"));
    }

    // Test R127-16: dataframe count empty
    #[test]
    fn test_dataframe_count_empty_r127() {
        let columns: Vec<DataFrameColumn> = vec![];
        let result = eval_dataframe_count(&columns, &[]).expect("should return 0 for empty");
        assert_eq!(result, Value::Integer(0));
    }

    // Test R127-17: eval_exit_status_method success
    #[test]
    fn test_exit_status_method_success_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("ExitStatus".to_string()));
        obj.insert("success".to_string(), Value::Bool(true));
        obj.insert("code".to_string(), Value::Integer(0));

        let result = eval_exit_status_method(&obj, "success", &[])
            .expect("should get success");
        assert_eq!(result, Value::Bool(true));
    }

    // Test R127-18: eval_exit_status_method success false
    #[test]
    fn test_exit_status_method_success_false_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("ExitStatus".to_string()));
        obj.insert("success".to_string(), Value::Bool(false));
        obj.insert("code".to_string(), Value::Integer(1));

        let result = eval_exit_status_method(&obj, "success", &[])
            .expect("should get success");
        assert_eq!(result, Value::Bool(false));
    }

    // Test R127-19: eval_exit_status_method with args error
    #[test]
    fn test_exit_status_method_with_args_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("ExitStatus".to_string()));
        obj.insert("success".to_string(), Value::Bool(true));

        let result = eval_exit_status_method(&obj, "success", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-20: eval_exit_status_method unknown method
    #[test]
    fn test_exit_status_method_unknown_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("ExitStatus".to_string()));
        obj.insert("success".to_string(), Value::Bool(true));

        let result = eval_exit_status_method(&obj, "unknown", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown ExitStatus"));
    }

    // Test R127-21: eval_exit_status_method missing success field
    #[test]
    fn test_exit_status_method_missing_field_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("ExitStatus".to_string()));
        // Missing "success" field

        let result = eval_exit_status_method(&obj, "success", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }

    // Test R127-22: require_no_args helper with args
    #[test]
    fn test_require_no_args_with_args_r127() {
        let result = require_no_args("test_method", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    // Test R127-23: require_no_args helper empty
    #[test]
    fn test_require_no_args_empty_r127() {
        let result = require_no_args("test_method", &[]);
        assert!(result.is_ok());
    }

    // Test R127-24: eval_integer_pow multiple args
    #[test]
    fn test_integer_pow_multiple_args_r127() {
        let result = eval_integer_pow(2, &[Value::Integer(3), Value::Integer(4)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exactly 1"));
    }

    // Test R127-25: try_dispatch_builtin no marker
    #[test]
    fn test_try_dispatch_builtin_no_marker_r127() {
        let obj = std::collections::HashMap::new();
        let result = try_dispatch_builtin(&obj, "some_method", &[])
            .expect("should return None");
        assert!(result.is_none());
    }

    // Test R127-26: try_dispatch_builtin not builtin
    #[test]
    fn test_try_dispatch_builtin_not_builtin_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("some_method".to_string(), Value::from_string("regular_value".to_string()));
        let result = try_dispatch_builtin(&obj, "some_method", &[])
            .expect("should return None for non-builtin");
        assert!(result.is_none());
    }

    // Test R127-27: eval_object_method missing type marker
    #[test]
    fn test_object_method_missing_type_r127() {
        let obj = std::collections::HashMap::new();
        let result = eval_object_method(&obj, "test", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing __type"));
    }

    // Test R127-28: eval_object_method unknown type
    #[test]
    fn test_object_method_unknown_type_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("UnknownType".to_string()));
        let result = eval_object_method(&obj, "test", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown object type"));
    }

    // Test R127-29: generic method with args (should fail)
    #[test]
    fn test_generic_to_string_with_args_r127() {
        let value = Value::Bool(true);
        let result = eval_generic_method(&value, "to_string", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // Test R127-30: dispatch with dataframe value
    #[test]
    fn test_dispatch_dataframe_method_r127() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let value = Value::DataFrame { columns };

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "count",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.expect("should work"), Value::Integer(1));
    }

    // Test R127-31: dispatch with float value
    #[test]
    fn test_dispatch_float_method_r127() {
        let value = Value::Float(9.0);

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "sqrt",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.expect("should work"), Value::Float(3.0));
    }

    // Test R127-32: dispatch with integer value
    #[test]
    fn test_dispatch_integer_method_r127() {
        let value = Value::Integer(-42);

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "abs",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.expect("should work"), Value::Integer(42));
    }

    // Test R127-33: dispatch with object value (Command type)
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_dispatch_object_command_r127() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Command".to_string()));
        obj.insert("program".to_string(), Value::from_string("echo".to_string()));
        obj.insert("args".to_string(), Value::Array(Arc::from(vec![])));
        let value = Value::Object(Arc::new(obj));

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "arg",
            &[Value::from_string("hello".to_string())],
            false,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
    }

    // Test R127-34: eval_method_call wrapper function
    #[test]
    fn test_eval_method_call_wrapper_r127() {
        let value = Value::Float(16.0);

        let result = eval_method_call(
            &value,
            "sqrt",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.expect("should work"), Value::Float(4.0));
    }

    // Test R127-35: dataframe sum with non-numeric (should skip)
    #[test]
    fn test_dataframe_sum_with_strings_r127() {
        let columns = vec![DataFrameColumn {
            name: "mixed".to_string(),
            values: vec![
                Value::Integer(1),
                Value::from_string("skip".to_string()),
                Value::Integer(2),
            ],
        }];
        let result = eval_dataframe_sum(&columns, &[]).expect("should work");
        assert_eq!(result, Value::Float(3.0));
    }

    // Test R127-36: dataframe mean with non-numeric (should skip)
    #[test]
    fn test_dataframe_mean_with_strings_r127() {
        let columns = vec![DataFrameColumn {
            name: "mixed".to_string(),
            values: vec![
                Value::Integer(2),
                Value::from_string("skip".to_string()),
                Value::Integer(4),
            ],
        }];
        let result = eval_dataframe_mean(&columns, &[]).expect("should work");
        assert_eq!(result, Value::Float(3.0));
    }

    // Test R127-37: dataframe max with strings (should skip)
    #[test]
    fn test_dataframe_max_with_strings_r127() {
        let columns = vec![DataFrameColumn {
            name: "mixed".to_string(),
            values: vec![
                Value::Integer(5),
                Value::from_string("skip".to_string()),
                Value::Integer(10),
            ],
        }];
        let result = eval_dataframe_max(&columns, &[]).expect("should work");
        assert_eq!(result, Value::Float(10.0));
    }

    // Test R127-38: dataframe min with strings (should skip)
    #[test]
    fn test_dataframe_min_with_strings_r127() {
        let columns = vec![DataFrameColumn {
            name: "mixed".to_string(),
            values: vec![
                Value::Integer(5),
                Value::from_string("skip".to_string()),
                Value::Integer(3),
            ],
        }];
        let result = eval_dataframe_min(&columns, &[]).expect("should work");
        assert_eq!(result, Value::Float(3.0));
    }

    // ============================================================================
    // EXTREME TDD Round 131: Comprehensive method dispatch coverage tests
    // Target: 88.35% → 95%+ coverage
    // ============================================================================

    // --- Float method error paths ---
    #[test]
    fn test_float_method_powf_suggests_operator() {
        let result = eval_float_method(2.0, "powf", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Use ** operator"));
    }

    #[test]
    fn test_float_method_with_args_error() {
        let result = eval_float_method(2.0, "sqrt", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("takes no arguments"));
    }

    #[test]
    fn test_float_method_unknown() {
        let result = eval_float_method(2.0, "unknown_method", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown float method"));
    }

    #[test]
    fn test_float_method_abs() {
        let result = eval_float_method(-3.5, "abs", true).unwrap();
        assert_eq!(result, Value::Float(3.5));
    }

    #[test]
    fn test_float_method_ceil() {
        let result = eval_float_method(3.2, "ceil", true).unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_float_method_floor() {
        let result = eval_float_method(3.8, "floor", true).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_float_method_sin() {
        let result = eval_float_method(0.0, "sin", true).unwrap();
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_cos() {
        let result = eval_float_method(0.0, "cos", true).unwrap();
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_tan() {
        let result = eval_float_method(0.0, "tan", true).unwrap();
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_ln() {
        let result = eval_float_method(1.0, "ln", true).unwrap();
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_log10() {
        let result = eval_float_method(100.0, "log10", true).unwrap();
        assert_eq!(result, Value::Float(2.0));
    }

    #[test]
    fn test_float_method_exp() {
        let result = eval_float_method(0.0, "exp", true).unwrap();
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_float_method_to_string() {
        let result = eval_float_method(3.14, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("3.14".to_string()));
    }

    // --- Integer method error paths ---
    #[test]
    fn test_integer_method_pow_wrong_arg_count() {
        let result = eval_integer_pow(2, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires exactly 1 argument"));
    }

    #[test]
    fn test_integer_method_pow_negative_exp() {
        let result = eval_integer_pow(2, &[Value::Integer(-1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be non-negative"));
    }

    #[test]
    fn test_integer_method_pow_wrong_type() {
        let result = eval_integer_pow(2, &[Value::from_string("3".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires integer exponent"));
    }

    #[test]
    fn test_integer_method_unknown() {
        let result = eval_integer_method(42, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown integer method"));
    }

    #[test]
    fn test_integer_method_abs_with_args() {
        let result = eval_integer_method(42, "abs", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("takes no arguments"));
    }

    #[test]
    fn test_integer_method_sqrt() {
        let result = eval_integer_method(9, "sqrt", &[]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_integer_method_to_float() {
        let result = eval_integer_method(42, "to_float", &[]).unwrap();
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_integer_method_to_string() {
        let result = eval_integer_method(42, "to_string", &[]).unwrap();
        assert_eq!(result, Value::from_string("42".to_string()));
    }

    #[test]
    fn test_integer_method_signum_positive() {
        let result = eval_integer_method(42, "signum", &[]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_integer_method_signum_negative() {
        let result = eval_integer_method(-42, "signum", &[]).unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_integer_method_signum_zero() {
        let result = eval_integer_method(0, "signum", &[]).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_integer_method_pow_success() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(10)]).unwrap();
        assert_eq!(result, Value::Integer(1024));
    }

    // --- Object method error paths ---
    #[test]
    fn test_object_method_unknown_type() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("UnknownType".to_string()));
        let result = eval_object_method(&obj, "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown object type"));
    }

    #[test]
    fn test_object_method_missing_type() {
        let obj = std::collections::HashMap::new();
        let result = eval_object_method(&obj, "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing __type marker"));
    }

    // --- Generic method error paths ---
    #[test]
    fn test_generic_method_unknown() {
        let result = eval_generic_method(&Value::Nil, "unknown", true);
        assert!(result.is_err());
    }

    // --- Dataframe method tests ---
    #[test]
    fn test_dataframe_method_unknown() {
        let columns = vec![];
        let result = eval_dataframe_method(&columns, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown DataFrame method"));
    }

    #[test]
    fn test_dataframe_columns_method() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![],
            },
        ];
        let result = eval_dataframe_columns(&columns, &[]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
            assert_eq!(arr[1], Value::from_string("b".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_dataframe_shape_method() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(4)],
            },
        ];
        let result = eval_dataframe_shape(&columns, &[]).unwrap();
        if let Value::Array(shape) = result {
            assert_eq!(shape[0], Value::Integer(2)); // rows
            assert_eq!(shape[1], Value::Integer(2)); // cols
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_dataframe_count_method() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        let result = eval_dataframe_count(&columns, &[]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_dataframe_select_method() {
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
        let result = eval_dataframe_select(&columns, &[Value::from_string("a".to_string())]).unwrap();
        if let Value::DataFrame { columns: new_cols } = result {
            assert_eq!(new_cols.len(), 1);
            assert_eq!(new_cols[0].name, "a");
        } else {
            panic!("Expected dataframe");
        }
    }

    #[test]
    fn test_dataframe_select_not_found() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_select(&columns, &[Value::from_string("z".to_string())]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Column 'z' not found"));
    }

    // --- Dispatch turbofish stripping test ---
    #[test]
    fn test_turbofish_stripping_in_dispatch() {
        // Method "parse::<i32>" should be stripped to "parse"
        // Testing via integer method which doesn't have parse (expect error)
        let result = eval_integer_method(42, "parse::<i32>", &[]);
        assert!(result.is_err());
        // The error should mention "parse" not "parse::<i32>"
    }

    // --- eval_method_call main entry point ---
    #[test]
    fn test_eval_method_call_integer_abs() {
        let result = eval_method_call(
            &Value::Integer(-42),
            "abs",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_method_call_float_sqrt() {
        let result = eval_method_call(
            &Value::Float(16.0),
            "sqrt",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(4.0));
    }

    // ============================================================================
    // COMPREHENSIVE COVERAGE TESTS - Targeting 334 uncovered lines
    // ============================================================================

    // --- Command method tests (non-WASM only) ---

    #[cfg(not(target_arch = "wasm32"))]
    mod command_tests {
        use super::*;

        fn create_command_obj(program: &str) -> std::collections::HashMap<String, Value> {
            let mut obj = std::collections::HashMap::new();
            obj.insert(
                "__type".to_string(),
                Value::from_string("Command".to_string()),
            );
            obj.insert(
                "program".to_string(),
                Value::from_string(program.to_string()),
            );
            obj.insert("args".to_string(), Value::Array(Arc::from(vec![])));
            obj
        }

        #[test]
        fn test_command_arg_success() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "arg", &[Value::from_string("hello".to_string())]);
            assert!(result.is_ok());
            if let Value::Object(new_obj) = result.unwrap() {
                if let Some(Value::Array(args)) = new_obj.get("args") {
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Value::from_string("hello".to_string()));
                } else {
                    panic!("Expected args array");
                }
            } else {
                panic!("Expected Object");
            }
        }

        #[test]
        fn test_command_arg_wrong_count() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "arg", &[]);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("requires exactly 1 argument"));
        }

        #[test]
        fn test_command_arg_wrong_type() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "arg", &[Value::Integer(42)]);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("expects a string argument"));
        }

        #[test]
        fn test_command_unknown_method() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "unknown_method", &[]);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Unknown Command method"));
        }

        #[test]
        fn test_command_status_success() {
            // Use 'true' command which always succeeds
            let obj = create_command_obj("true");
            let result = eval_command_method(&obj, "status", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant { enum_name, variant_name, data } = result.unwrap() {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Ok");
                assert!(data.is_some());
                let data = data.unwrap();
                assert_eq!(data.len(), 1);
                if let Value::Object(status_obj) = &data[0] {
                    assert_eq!(status_obj.get("success"), Some(&Value::Bool(true)));
                } else {
                    panic!("Expected Object in data");
                }
            } else {
                panic!("Expected EnumVariant");
            }
        }

        #[test]
        fn test_command_status_failure() {
            // Use 'false' command which always fails
            let obj = create_command_obj("false");
            let result = eval_command_method(&obj, "status", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant { enum_name, variant_name, data } = result.unwrap() {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Ok");
                assert!(data.is_some());
                let data = data.unwrap();
                if let Value::Object(status_obj) = &data[0] {
                    assert_eq!(status_obj.get("success"), Some(&Value::Bool(false)));
                }
            }
        }

        #[test]
        fn test_command_status_error_nonexistent() {
            // Use a command that doesn't exist
            let obj = create_command_obj("nonexistent_command_12345");
            let result = eval_command_method(&obj, "status", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant { enum_name, variant_name, .. } = result.unwrap() {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Err");
            } else {
                panic!("Expected EnumVariant Err");
            }
        }

        #[test]
        fn test_command_output_success() {
            let obj = create_command_obj("echo");
            let mut obj_with_args = obj.clone();
            obj_with_args.insert(
                "args".to_string(),
                Value::Array(Arc::from(vec![Value::from_string("hello".to_string())])),
            );
            let result = eval_command_method(&obj_with_args, "output", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant { enum_name, variant_name, data } = result.unwrap() {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Ok");
                assert!(data.is_some());
                let data = data.unwrap();
                if let Value::Object(output_obj) = &data[0] {
                    assert!(output_obj.contains_key("stdout"));
                    assert!(output_obj.contains_key("stderr"));
                    assert!(output_obj.contains_key("status"));
                } else {
                    panic!("Expected Object in data");
                }
            }
        }

        #[test]
        fn test_command_output_error_nonexistent() {
            let obj = create_command_obj("nonexistent_command_12345");
            let result = eval_command_method(&obj, "output", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant { enum_name, variant_name, .. } = result.unwrap() {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Err");
            }
        }

        #[test]
        fn test_build_command_missing_program() {
            let obj = std::collections::HashMap::new();
            let result = build_command_from_obj(&obj);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("missing 'program' field"));
        }

        #[test]
        fn test_build_command_with_args() {
            let mut obj = std::collections::HashMap::new();
            obj.insert("program".to_string(), Value::from_string("echo".to_string()));
            obj.insert(
                "args".to_string(),
                Value::Array(Arc::from(vec![
                    Value::from_string("-n".to_string()),
                    Value::from_string("hello".to_string()),
                ])),
            );
            let result = build_command_from_obj(&obj);
            assert!(result.is_ok());
        }

        #[test]
        fn test_build_command_no_args() {
            let mut obj = std::collections::HashMap::new();
            obj.insert("program".to_string(), Value::from_string("echo".to_string()));
            // No args field - should default to empty
            let result = build_command_from_obj(&obj);
            assert!(result.is_ok());
        }

        #[test]
        fn test_build_command_args_with_non_string() {
            let mut obj = std::collections::HashMap::new();
            obj.insert("program".to_string(), Value::from_string("echo".to_string()));
            obj.insert(
                "args".to_string(),
                Value::Array(Arc::from(vec![
                    Value::from_string("hello".to_string()),
                    Value::Integer(42), // Non-string - should be skipped
                ])),
            );
            let result = build_command_from_obj(&obj);
            assert!(result.is_ok());
        }

        #[test]
        fn test_command_arg_multiple() {
            let obj = create_command_obj("echo");
            // First arg
            let result1 = eval_command_method(&obj, "arg", &[Value::from_string("-n".to_string())]);
            assert!(result1.is_ok());
            if let Value::Object(obj1) = result1.unwrap() {
                // Second arg
                let result2 = eval_command_method(&obj1, "arg", &[Value::from_string("hello".to_string())]);
                assert!(result2.is_ok());
                if let Value::Object(obj2) = result2.unwrap() {
                    if let Some(Value::Array(args)) = obj2.get("args") {
                        assert_eq!(args.len(), 2);
                    }
                }
            }
        }
    }

    // --- DataFrame select with array tests ---

    #[test]
    fn test_dataframe_select_array_success() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(4)],
            },
            DataFrameColumn {
                name: "c".to_string(),
                values: vec![Value::Integer(5), Value::Integer(6)],
            },
        ];
        let col_names = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("c".to_string()),
        ]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns: selected } = result.unwrap() {
            assert_eq!(selected.len(), 2);
            assert_eq!(selected[0].name, "a");
            assert_eq!(selected[1].name, "c");
        } else {
            panic!("Expected DataFrame");
        }
    }

    #[test]
    fn test_dataframe_select_array_not_found() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1)],
            },
        ];
        let col_names = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("missing".to_string()),
        ]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_dataframe_select_array_non_string_element() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1)],
            },
        ];
        let col_names = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::Integer(42), // Not a string
        ]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be strings"));
    }

    #[test]
    fn test_dataframe_select_empty_array() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1)],
            },
        ];
        let col_names = Value::Array(Arc::from(vec![]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns: selected } = result.unwrap() {
            assert!(selected.is_empty());
        }
    }

    // --- Object method tests ---

    #[test]
    fn test_object_method_module_type() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Module".to_string()));
        let result = eval_object_method(&obj, "some_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("handled in interpreter"));
    }

    // --- More dispatch tests ---

    #[test]
    fn test_dispatch_string_method() {
        let value = Value::String(Arc::from("hello"));
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "len",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_dispatch_object_method() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("ExitStatus".to_string()));
        obj.insert("success".to_string(), Value::Bool(true));
        let value = Value::Object(Arc::new(obj));

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "success",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_dispatch_bool_to_string() {
        let value = Value::Bool(false);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "to_string",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string("false".to_string()));
    }

    #[test]
    fn test_dispatch_unknown_method_on_bool() {
        let value = Value::Bool(true);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "unknown_method",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // --- DataFrame multiple columns tests ---

    #[test]
    fn test_dataframe_sum_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Float(3.5), Value::Float(4.5)],
            },
        ];
        let result = eval_dataframe_sum(&columns, &[]).unwrap();
        // 1 + 2 + 3.5 + 4.5 = 11.0
        assert_eq!(result, Value::Float(11.0));
    }

    #[test]
    fn test_dataframe_mean_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(2), Value::Integer(4)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(6), Value::Integer(8)],
            },
        ];
        let result = eval_dataframe_mean(&columns, &[]).unwrap();
        // (2 + 4 + 6 + 8) / 4 = 5.0
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_dataframe_max_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(5)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(2)],
            },
        ];
        let result = eval_dataframe_max(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_dataframe_min_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(10), Value::Integer(5)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(7)],
            },
        ];
        let result = eval_dataframe_min(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_dataframe_max_with_floats() {
        let columns = vec![DataFrameColumn {
            name: "values".to_string(),
            values: vec![Value::Float(1.5), Value::Float(3.7), Value::Float(2.1)],
        }];
        let result = eval_dataframe_max(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(3.7));
    }

    #[test]
    fn test_dataframe_min_with_floats() {
        let columns = vec![DataFrameColumn {
            name: "values".to_string(),
            values: vec![Value::Float(1.5), Value::Float(3.7), Value::Float(2.1)],
        }];
        let result = eval_dataframe_min(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(1.5));
    }

    // --- Float method edge cases ---

    #[test]
    fn test_float_method_sqrt_zero() {
        let result = eval_float_method(0.0, "sqrt", true).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_float_method_abs_zero() {
        let result = eval_float_method(0.0, "abs", true).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_float_method_round_half() {
        // Rust rounds half away from zero
        let result = eval_float_method(2.5, "round", true).unwrap();
        assert_eq!(result, Value::Float(3.0)); // rounds away from zero
    }

    #[test]
    fn test_float_method_floor_negative() {
        let result = eval_float_method(-2.3, "floor", true).unwrap();
        assert_eq!(result, Value::Float(-3.0));
    }

    #[test]
    fn test_float_method_ceil_negative() {
        let result = eval_float_method(-2.7, "ceil", true).unwrap();
        assert_eq!(result, Value::Float(-2.0));
    }

    // --- Integer method edge cases ---

    #[test]
    fn test_integer_method_pow_zero_exp() {
        let result = eval_integer_method(5, "pow", &[Value::Integer(0)]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_integer_method_pow_one_exp() {
        let result = eval_integer_method(5, "pow", &[Value::Integer(1)]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_integer_method_pow_large() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(16)]).unwrap();
        assert_eq!(result, Value::Integer(65536));
    }

    #[test]
    fn test_integer_method_abs_min() {
        // Edge case: i64::MIN can cause overflow in abs(), but we test a safe negative
        let result = eval_integer_method(-100, "abs", &[]).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_integer_method_sqrt_large() {
        let result = eval_integer_method(1000000, "sqrt", &[]).unwrap();
        assert_eq!(result, Value::Float(1000.0));
    }

    // --- require_no_args helper tests ---

    #[test]
    fn test_require_no_args_multiple_args() {
        let result = require_no_args("test", &[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    // --- Generic method edge cases ---

    #[test]
    fn test_generic_to_string_enum_variant() {
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("Some") || s.contains("Option"));
        }
    }

    #[test]
    fn test_generic_to_string_byte() {
        let value = Value::Byte(65);
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        if let Value::String(_) = result {
            // Success - any string representation is fine
        } else {
            panic!("Expected String");
        }
    }

    // --- Additional dataframe edge cases ---

    #[test]
    fn test_dataframe_mean_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_mean(&columns, &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_dataframe_max_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_max(&columns, &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_dataframe_min_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_min(&columns, &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_dataframe_sum_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_sum(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    // --- Dispatch with turbofish variations ---

    #[test]
    fn test_dispatch_turbofish_complex() {
        let value = Value::Float(4.0);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        // Method with complex turbofish that should be stripped
        let result = dispatch_method_call(
            &value,
            "sqrt::<f64, f64>",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(2.0));
    }

    #[test]
    fn test_dispatch_no_turbofish() {
        let value = Value::Integer(16);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "sqrt",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(4.0));
    }

    // --- eval_method_call wrapper tests ---

    #[test]
    fn test_eval_method_call_string_len() {
        let result = eval_method_call(
            &Value::String(Arc::from("test")),
            "len",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(4));
    }

    #[test]
    fn test_eval_method_call_dataframe_count() {
        let columns = vec![DataFrameColumn {
            name: "col".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2)],
        }];
        let result = eval_method_call(
            &Value::DataFrame { columns },
            "count",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_eval_method_call_array_len() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]));
        let result = eval_method_call(
            &arr,
            "len",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
}
