//! Path builtin functions
//!
//! This module handles path manipulation operations including join, parent,
//! file_name, extension, canonicalize, normalize, and components.

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

/// Helper: `path_join` operation
/// Complexity: 3 (minimal nesting)
pub(crate) fn eval_path_join(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_join", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(base), Value::String(component)) => {
            let path = std::path::Path::new(base.as_ref()).join(component.as_ref());
            Ok(Value::from_string(path.to_string_lossy().to_string()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_join() expects two string arguments".to_string(),
        )),
    }
}

/// Helper: Build path from array of string components
/// Complexity: 3 (extracted to reduce nesting)
pub(crate) fn build_path_from_value_components(
    components: &[Value],
) -> Result<std::path::PathBuf, InterpreterError> {
    let mut path = std::path::PathBuf::new();
    for component in components {
        match component {
            Value::String(s) => path.push(s.as_ref()),
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "path_join_many() expects array of strings".to_string(),
                ))
            }
        }
    }
    Ok(path)
}

/// Helper: `path_join_many` operation
/// Complexity: 3 (reduced via helper extraction)
pub(crate) fn eval_path_join_many(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_join_many", args, 1)?;
    match &args[0] {
        Value::Array(components) => {
            let path = build_path_from_value_components(components)?;
            Ok(Value::from_string(path.to_string_lossy().to_string()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_join_many() expects an array argument".to_string(),
        )),
    }
}

/// Helper: `path_parent` operation
/// Complexity: 4
pub(crate) fn eval_path_parent(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_parent", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.parent() {
                Some(parent) => Ok(Value::from_string(parent.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_parent() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_file_name` operation
/// Complexity: 4
pub(crate) fn eval_path_file_name(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_file_name", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.file_name() {
                Some(name) => Ok(Value::from_string(name.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_file_name() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_file_stem` operation
/// Complexity: 4
pub(crate) fn eval_path_file_stem(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_file_stem", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.file_stem() {
                Some(stem) => Ok(Value::from_string(stem.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_file_stem() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_extension` operation
/// Complexity: 4
pub(crate) fn eval_path_extension(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_extension", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.extension() {
                Some(ext) => Ok(Value::from_string(ext.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_extension() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_is_absolute` operation
/// Complexity: 2
pub(crate) fn eval_path_is_absolute(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_is_absolute", args, 1)?;
    match &args[0] {
        Value::String(path) => Ok(Value::Bool(
            std::path::Path::new(path.as_ref()).is_absolute(),
        )),
        _ => Err(InterpreterError::RuntimeError(
            "path_is_absolute() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_is_relative` operation
/// Complexity: 2
pub(crate) fn eval_path_is_relative(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_is_relative", args, 1)?;
    match &args[0] {
        Value::String(path) => Ok(Value::Bool(
            std::path::Path::new(path.as_ref()).is_relative(),
        )),
        _ => Err(InterpreterError::RuntimeError(
            "path_is_relative() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_canonicalize` operation
/// Complexity: 4
pub(crate) fn eval_path_canonicalize(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_canonicalize", args, 1)?;
    match &args[0] {
        Value::String(path) => match std::fs::canonicalize(path.as_ref()) {
            Ok(canonical) => Ok(Value::from_string(canonical.to_string_lossy().to_string())),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "Failed to canonicalize path: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "path_canonicalize() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_with_extension` operation
/// Complexity: 3
pub(crate) fn eval_path_with_extension(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_with_extension", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(ext)) => {
            let p = std::path::Path::new(path.as_ref()).with_extension(ext.as_ref());
            Ok(Value::from_string(p.to_string_lossy().to_string()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_with_extension() expects two string arguments".to_string(),
        )),
    }
}

/// Helper: `path_with_file_name` operation
/// Complexity: 3
pub(crate) fn eval_path_with_file_name(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_with_file_name", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(name)) => {
            let p = std::path::Path::new(path.as_ref()).with_file_name(name.as_ref());
            Ok(Value::from_string(p.to_string_lossy().to_string()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_with_file_name() expects two string arguments".to_string(),
        )),
    }
}

/// Helper: `path_components` operation
/// Complexity: 3
pub(crate) fn eval_path_components(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_components", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            let components: Vec<Value> = p
                .components()
                .map(|c| Value::from_string(c.as_os_str().to_string_lossy().to_string()))
                .collect();
            Ok(Value::Array(components.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_components() expects a string argument".to_string(),
        )),
    }
}

/// Helper: `path_normalize` operation
/// Complexity: 4
pub(crate) fn eval_path_normalize(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_normalize", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            let mut normalized = std::path::PathBuf::new();
            for component in p.components() {
                match component {
                    std::path::Component::CurDir => {}
                    std::path::Component::ParentDir => {
                        normalized.pop();
                    }
                    _ => normalized.push(component),
                }
            }
            Ok(Value::from_string(normalized.to_string_lossy().to_string()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "path_normalize() expects a string argument".to_string(),
        )),
    }
}

/// Dispatch path functions - Part 1 (functions 1-4)
pub(crate) fn try_eval_path_part1(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_join__" => Ok(Some(eval_path_join(args)?)),
        "__builtin_path_join_many__" => Ok(Some(eval_path_join_many(args)?)),
        "__builtin_path_parent__" => Ok(Some(eval_path_parent(args)?)),
        "__builtin_path_file_name__" => Ok(Some(eval_path_file_name(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch path functions - Part 2 (functions 5-8)
pub(crate) fn try_eval_path_part2(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_file_stem__" => Ok(Some(eval_path_file_stem(args)?)),
        "__builtin_path_extension__" => Ok(Some(eval_path_extension(args)?)),
        "__builtin_path_is_absolute__" => Ok(Some(eval_path_is_absolute(args)?)),
        "__builtin_path_is_relative__" => Ok(Some(eval_path_is_relative(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch path functions - Part 3a (functions 9-11)
pub(crate) fn try_eval_path_part3a(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_canonicalize__" => Ok(Some(eval_path_canonicalize(args)?)),
        "__builtin_path_with_extension__" => Ok(Some(eval_path_with_extension(args)?)),
        "__builtin_path_with_file_name__" => Ok(Some(eval_path_with_file_name(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch path functions - Part 3b (functions 12-13)
pub(crate) fn try_eval_path_part3b(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_components__" => Ok(Some(eval_path_components(args)?)),
        "__builtin_path_normalize__" => Ok(Some(eval_path_normalize(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for path functions
/// Complexity: 4 (loop pattern reduces cognitive load)
pub(crate) fn try_eval_path_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    let dispatchers: &[fn(&str, &[Value]) -> Result<Option<Value>, InterpreterError>] = &[
        try_eval_path_part1,
        try_eval_path_part2,
        try_eval_path_part3a,
        try_eval_path_part3b,
    ];

    for dispatcher in dispatchers {
        if let Some(result) = dispatcher(name, args)? {
            return Ok(Some(result));
        }
    }
    Ok(None)
}
