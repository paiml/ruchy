//! Platform-specific and miscellaneous builtin functions
//!
//! This module handles HTTP, HTML, Process, File, and String builtin functions,
//! as well as type conversion functions (str, int, float, bool, etc.).

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

use std::collections::HashMap;
use std::sync::Arc;

// ==============================================================================
// File Builtin Functions (ISSUE-116)
// ==============================================================================

/// Dispatcher for File namespace methods
/// Complexity: 3
pub(crate) fn try_eval_file_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "File_open" => Ok(Some(eval_file_open(args)?)),
        "__builtin_open__" => Ok(Some(eval_open(args)?)),
        _ => Ok(None),
    }
}

/// ISSUE-116: File.open(path) - Opens file and returns File object
/// Complexity: 4
pub(crate) fn eval_file_open(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("File.open", args, 1)?;

    match &args[0] {
        Value::String(path) => {
            let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
                InterpreterError::RuntimeError(format!("Failed to open file '{path}': {e}"))
            })?;

            let lines: Vec<String> = content
                .lines()
                .map(std::string::ToString::to_string)
                .collect();

            let mut file_obj = std::collections::HashMap::new();
            file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
            file_obj.insert("path".to_string(), Value::from_string(path.to_string()));
            file_obj.insert(
                "lines".to_string(),
                Value::Array(Arc::from(
                    lines
                        .into_iter()
                        .map(Value::from_string)
                        .collect::<Vec<_>>(),
                )),
            );
            file_obj.insert("position".to_string(), Value::Integer(0));
            file_obj.insert("closed".to_string(), Value::Bool(false));

            Ok(Value::ObjectMut(Arc::new(std::sync::Mutex::new(file_obj))))
        }
        _ => Err(InterpreterError::RuntimeError(
            "File.open() expects a string argument".to_string(),
        )),
    }
}

/// ISSUE-116: open(path, mode) - Standalone function for opening files
/// Complexity: 5
pub(crate) fn eval_open(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("open", args, 2)?;

    let path = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "open() expects first argument to be a string (path)".to_string(),
            ))
        }
    };

    let mode = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "open() expects second argument to be a string (mode)".to_string(),
            ))
        }
    };

    if mode != "r" {
        return Err(InterpreterError::RuntimeError(format!(
            "open() mode '{mode}' not supported. Only 'r' (read) is currently supported."
        )));
    }

    eval_file_open(&[Value::from_string(path.to_string())])
}

// ==============================================================================
// HTTP Builtin Functions (STDLIB-PHASE-5)
// ==============================================================================

/// Dispatcher for HTTP builtin functions
#[cfg(all(not(target_arch = "wasm32"), feature = "http-client"))]
pub(crate) fn try_eval_http_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "http_get" => Ok(Some(eval_http_get(args)?)),
        "http_post" => Ok(Some(eval_http_post(args)?)),
        "http_put" => Ok(Some(eval_http_put(args)?)),
        "http_delete" => Ok(Some(eval_http_delete(args)?)),
        _ => Ok(None),
    }
}

/// Stub for builds without http-client feature
#[cfg(not(all(not(target_arch = "wasm32"), feature = "http-client")))]
pub(crate) fn try_eval_http_function(
    _name: &str,
    _args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    Ok(None)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "http-client"))]
pub(crate) fn eval_http_get(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_get", args, 1)?;
    match &args[0] {
        Value::String(url) => match crate::stdlib::http::get(url) {
            Ok(response) => Ok(Value::from_string(response)),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "HTTP GET failed: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "http_get() expects a string URL".to_string(),
        )),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "http-client"))]
pub(crate) fn eval_http_post(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_post", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(url), Value::String(body)) => match crate::stdlib::http::post(url, body) {
            Ok(response) => Ok(Value::from_string(response)),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "HTTP POST failed: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "http_post() expects two string arguments".to_string(),
        )),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "http-client"))]
pub(crate) fn eval_http_put(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_put", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(url), Value::String(body)) => match crate::stdlib::http::put(url, body) {
            Ok(response) => Ok(Value::from_string(response)),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "HTTP PUT failed: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "http_put() expects two string arguments".to_string(),
        )),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "http-client"))]
pub(crate) fn eval_http_delete(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_delete", args, 1)?;
    match &args[0] {
        Value::String(url) => match crate::stdlib::http::delete(url) {
            Ok(response) => Ok(Value::from_string(response)),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "HTTP DELETE failed: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "http_delete() expects a string URL".to_string(),
        )),
    }
}

// ============================================================================
// HTML Parsing Functions (HTTP-002-C, STD-011)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn try_eval_html_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "Html_parse" => Ok(Some(eval_html_parse(args)?)),
        _ => Ok(None),
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn try_eval_html_function(
    _name: &str,
    _args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    Ok(None)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn eval_html_parse(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("Html.parse", args, 1)?;
    match &args[0] {
        Value::String(html) => {
            let doc = crate::stdlib::html::HtmlDocument::parse(html);
            Ok(Value::HtmlDocument(doc))
        }
        _ => Err(InterpreterError::RuntimeError(
            "Html.parse() expects a string".to_string(),
        )),
    }
}

// ============================================================================
// Process Functions (RUNTIME-090, Issue #75)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn try_eval_process_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_command_new__" => Ok(Some(eval_command_new(args)?)),
        _ => Ok(None),
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn try_eval_process_function(
    _name: &str,
    _args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    Ok(None)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn eval_command_new(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("Command::new", args, 1)?;
    match &args[0] {
        Value::String(program) => {
            let mut cmd_obj = HashMap::new();
            cmd_obj.insert(
                "__type".to_string(),
                Value::from_string("Command".to_string()),
            );
            cmd_obj.insert(
                "program".to_string(),
                Value::from_string(program.to_string()),
            );
            cmd_obj.insert("args".to_string(), Value::Array(Arc::new([])));
            Ok(Value::Object(Arc::new(cmd_obj)))
        }
        _ => Err(InterpreterError::RuntimeError(
            "Command::new() expects a string program name".to_string(),
        )),
    }
}

// ============================================================================
// String Functions (REGRESSION-077, Issue #77)
// ============================================================================

/// String function dispatcher
pub(crate) fn try_eval_string_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_String_new__" => Ok(Some(eval_string_new(args)?)),
        "__builtin_String_from__" => Ok(Some(eval_string_from(args)?)),
        "__builtin_String_from_utf8__" => Ok(Some(eval_string_from_utf8(args)?)),
        _ => Ok(None),
    }
}

/// Eval: `String::new()`
pub(crate) fn eval_string_new(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("String::new", args, 0)?;
    Ok(Value::from_string(String::new()))
}

/// Eval: `String::from(value)`
pub(crate) fn eval_string_from(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("String::from", args, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::from_string(s.to_string())),
        other => Ok(Value::from_string(format!("{other}"))),
    }
}

/// Eval: `String::from_utf8(bytes)`
/// Returns Result<String, Error>
/// Complexity: 4
pub(crate) fn eval_string_from_utf8(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("String::from_utf8", args, 1)?;
    match &args[0] {
        Value::Array(arr) => {
            let mut bytes = Vec::with_capacity(arr.len());
            for val in arr.iter() {
                if let Value::Byte(b) = val {
                    bytes.push(*b);
                } else {
                    return Err(InterpreterError::TypeError(
                        "String::from_utf8() requires an array of bytes".to_string(),
                    ));
                }
            }

            match String::from_utf8(bytes) {
                Ok(s) => Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Ok".to_string(),
                    data: Some(vec![Value::from_string(s)]),
                }),
                Err(e) => Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Err".to_string(),
                    data: Some(vec![Value::from_string(e.to_string())]),
                }),
            }
        }
        _ => Err(InterpreterError::TypeError(
            "String::from_utf8() requires an array argument".to_string(),
        )),
    }
}

// ============================================================================
// Type Conversion Functions (STDLIB-001)
// ============================================================================

/// Convert any value to string
pub(crate) fn eval_str(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("str", args, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.clone())),
        other => Ok(Value::from_string(format!("{other}"))),
    }
}

/// Convert value to string using Display trait
pub(crate) fn eval_to_string(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("to_string", args, 1)?;
    Ok(Value::from_string(format!("{}", args[0])))
}

/// Convert value to integer
pub(crate) fn eval_int(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("int", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(*f as i64)),
        Value::String(s) => s.parse::<i64>().map(Value::Integer).map_err(|_| {
            InterpreterError::RuntimeError(format!("int() cannot parse string: '{s}'"))
        }),
        Value::Bool(b) => Ok(Value::Integer(i64::from(*b))),
        _ => Err(InterpreterError::RuntimeError(format!(
            "int() does not support type: {}",
            args[0]
        ))),
    }
}

/// Convert value to float
pub(crate) fn eval_float(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("float", args, 1)?;
    match &args[0] {
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Integer(n) => Ok(Value::Float(*n as f64)),
        Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
            InterpreterError::RuntimeError(format!("float() cannot parse string: '{s}'"))
        }),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        _ => Err(InterpreterError::RuntimeError(format!(
            "float() does not support type: {}",
            args[0]
        ))),
    }
}

/// Parse string to integer with validation
pub(crate) fn eval_parse_int(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("parse_int", args, 1)?;
    match &args[0] {
        Value::String(s) => s.parse::<i64>().map(Value::Integer).map_err(|_| {
            InterpreterError::RuntimeError(format!("parse_int() cannot parse string: '{s}'"))
        }),
        _ => Err(InterpreterError::RuntimeError(format!(
            "parse_int() expects a string, got {}",
            args[0].type_name()
        ))),
    }
}

/// Parse string to float with validation
pub(crate) fn eval_parse_float(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("parse_float", args, 1)?;
    match &args[0] {
        Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
            InterpreterError::RuntimeError(format!("parse_float() cannot parse string: '{s}'"))
        }),
        _ => Err(InterpreterError::RuntimeError(format!(
            "parse_float() expects a string, got {}",
            args[0].type_name()
        ))),
    }
}

/// Convert value to boolean
pub(crate) fn eval_bool(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("bool", args, 1)?;
    let result = match &args[0] {
        Value::Bool(b) => *b,
        Value::Integer(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Nil => false,
        Value::Array(arr) => !arr.is_empty(),
        _ => true,
    };
    Ok(Value::Bool(result))
}

/// Builtin `assert_eq` function for testing
pub(crate) fn eval_assert_eq(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() < 2 {
        return Err(InterpreterError::RuntimeError(
            "assert_eq() expects at least 2 arguments (expected, actual)".to_string(),
        ));
    }

    let expected = &args[0];
    let actual = &args[1];
    let message = if args.len() > 2 {
        format!("{}", args[2])
    } else {
        format!("Assertion failed: expected {expected:?}, got {actual:?}")
    };

    if expected == actual {
        Ok(Value::Nil)
    } else {
        Err(InterpreterError::AssertionFailed(message))
    }
}

/// Builtin assert function for testing
pub(crate) fn eval_assert(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "assert() expects at least 1 argument (condition)".to_string(),
        ));
    }

    let condition = &args[0];
    let message = if args.len() > 1 {
        format!("{}", args[1])
    } else {
        "Assertion failed".to_string()
    };

    match condition {
        Value::Bool(true) => Ok(Value::Nil),
        Value::Bool(false) => Err(InterpreterError::AssertionFailed(message)),
        _ => Err(InterpreterError::RuntimeError(
            "assert() expects a boolean condition".to_string(),
        )),
    }
}
