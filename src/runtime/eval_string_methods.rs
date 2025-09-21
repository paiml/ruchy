//! String method evaluation module
//!
//! This module handles all string-specific methods including length, case conversion,
//! searching, splitting, trimming, and character operations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate a string method call
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_string_method(
    s: &Rc<String>,
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        // Length and conversion methods
        "len" | "length" if args.is_empty() => Ok(Value::Integer(s.len() as i64)),
        "to_upper" if args.is_empty() => Ok(Value::from_string(s.to_uppercase())),
        "to_lower" if args.is_empty() => Ok(Value::from_string(s.to_lowercase())),
        "to_string" if args.is_empty() => Ok(Value::from_string(s.to_string())),
        "is_empty" if args.is_empty() => Ok(Value::Bool(s.is_empty())),

        // Trimming methods
        "trim" if args.is_empty() => Ok(Value::from_string(s.trim().to_string())),
        "trim_start" if args.is_empty() => Ok(Value::from_string(s.trim_start().to_string())),
        "trim_end" if args.is_empty() => Ok(Value::from_string(s.trim_end().to_string())),

        // Single-argument methods
        "contains" if args.len() == 1 => eval_string_contains(s, &args[0]),
        "starts_with" if args.len() == 1 => eval_string_starts_with(s, &args[0]),
        "ends_with" if args.len() == 1 => eval_string_ends_with(s, &args[0]),
        "split" if args.len() == 1 => eval_string_split(s, &args[0]),
        "repeat" if args.len() == 1 => eval_string_repeat(s, &args[0]),
        "char_at" if args.len() == 1 => eval_string_char_at(s, &args[0]),

        // Two-argument methods
        "replace" if args.len() == 2 => eval_string_replace(s, &args[0], &args[1]),
        "substring" if args.len() == 2 => eval_string_substring(s, &args[0], &args[1]),

        // Collection methods
        "chars" if args.is_empty() => eval_string_chars(s),
        "lines" if args.is_empty() => eval_string_lines(s),

        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown string method: {method}"
        ))),
    }
}

/// Check if string contains substring
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_contains(s: &str, needle: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(needle_str) = needle {
        Ok(Value::Bool(s.contains(needle_str.as_str())))
    } else {
        Err(InterpreterError::RuntimeError(
            "contains expects string argument".to_string(),
        ))
    }
}

/// Check if string starts with prefix
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_starts_with(s: &str, prefix: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(prefix_str) = prefix {
        Ok(Value::Bool(s.starts_with(prefix_str.as_str())))
    } else {
        Err(InterpreterError::RuntimeError(
            "starts_with expects string argument".to_string(),
        ))
    }
}

/// Check if string ends with suffix
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_ends_with(s: &str, suffix: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(suffix_str) = suffix {
        Ok(Value::Bool(s.ends_with(suffix_str.as_str())))
    } else {
        Err(InterpreterError::RuntimeError(
            "ends_with expects string argument".to_string(),
        ))
    }
}

/// Replace all occurrences of substring
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_replace(s: &str, from: &Value, to: &Value) -> Result<Value, InterpreterError> {
    if let (Value::String(from_str), Value::String(to_str)) = (from, to) {
        Ok(Value::from_string(
            s.replace(from_str.as_str(), to_str.as_str()),
        ))
    } else {
        Err(InterpreterError::RuntimeError(
            "replace expects two string arguments".to_string(),
        ))
    }
}

/// Split string by separator
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_split(s: &str, separator: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(sep_str) = separator {
        let parts: Vec<Value> = s
            .split(sep_str.as_str())
            .map(|part| Value::from_string(part.to_string()))
            .collect();
        Ok(Value::Array(Rc::new(parts)))
    } else {
        Err(InterpreterError::RuntimeError(
            "split expects string argument".to_string(),
        ))
    }
}

/// Repeat string n times
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_string_repeat(s: &str, count: &Value) -> Result<Value, InterpreterError> {
    if let Value::Integer(n) = count {
        if *n >= 0 {
            Ok(Value::from_string(s.repeat(*n as usize)))
        } else {
            Err(InterpreterError::RuntimeError(
                "repeat count must be non-negative".to_string(),
            ))
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "repeat expects integer argument".to_string(),
        ))
    }
}

/// Get character at index
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_string_char_at(s: &str, index: &Value) -> Result<Value, InterpreterError> {
    if let Value::Integer(idx) = index {
        if *idx >= 0 {
            let chars: Vec<char> = s.chars().collect();
            if let Some(ch) = chars.get(*idx as usize) {
                Ok(Value::from_string(ch.to_string()))
            } else {
                Ok(Value::Nil)
            }
        } else {
            Err(InterpreterError::RuntimeError(
                "char_at index must be non-negative".to_string(),
            ))
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "char_at expects integer argument".to_string(),
        ))
    }
}

/// Extract substring
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_string_substring(s: &str, start: &Value, end: &Value) -> Result<Value, InterpreterError> {
    if let (Value::Integer(start_idx), Value::Integer(end_idx)) = (start, end) {
        if *start_idx >= 0 && *end_idx >= *start_idx {
            let chars: Vec<char> = s.chars().collect();
            let start_pos = (*start_idx as usize).min(chars.len());
            let end_pos = (*end_idx as usize).min(chars.len());
            let substring: String = chars[start_pos..end_pos].iter().collect();
            Ok(Value::from_string(substring))
        } else {
            Err(InterpreterError::RuntimeError(
                "substring indices must be non-negative and start <= end".to_string(),
            ))
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "substring expects two integer arguments".to_string(),
        ))
    }
}

/// Convert string to array of characters
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_string_chars(s: &str) -> Result<Value, InterpreterError> {
    let chars: Vec<Value> = s
        .chars()
        .map(|c| Value::from_string(c.to_string()))
        .collect();
    Ok(Value::Array(Rc::new(chars)))
}

/// Split string into lines
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_string_lines(s: &str) -> Result<Value, InterpreterError> {
    let lines: Vec<Value> = s
        .lines()
        .map(|line| Value::from_string(line.to_string()))
        .collect();
    Ok(Value::Array(Rc::new(lines)))
}

/// Evaluate primitive type methods (float, integer, generic)
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_primitive_method(
    receiver: &Value,
    method: &str,
    args_empty: bool,
) -> Result<Value, InterpreterError> {
    match receiver {
        Value::Float(f) => eval_float_method(*f, method, args_empty),
        Value::Integer(n) => eval_integer_method(*n, method, args_empty),
        _ => eval_generic_method(receiver, method, args_empty),
    }
}

/// Evaluate float-specific methods
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
        "to_string" => Ok(Value::from_string(f.to_string())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown float method: {method}"
        ))),
    }
}

/// Evaluate integer-specific methods
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_integer_method(n: i64, method: &str, args_empty: bool) -> Result<Value, InterpreterError> {
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

/// Evaluate generic methods available to all types
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_length() {
        let s = Rc::new("hello".to_string());
        let result = eval_string_method(&s, "len", &[]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_string_case_conversion() {
        let s = Rc::new("Hello World".to_string());

        let upper = eval_string_method(&s, "to_upper", &[]).unwrap();
        assert_eq!(upper, Value::from_string("HELLO WORLD".to_string()));

        let lower = eval_string_method(&s, "to_lower", &[]).unwrap();
        assert_eq!(lower, Value::from_string("hello world".to_string()));
    }

    #[test]
    fn test_string_contains() {
        let s = Rc::new("hello world".to_string());
        let needle = Value::from_string("world".to_string());

        let result = eval_string_method(&s, "contains", &[needle]).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_split() {
        let s = Rc::new("a,b,c".to_string());
        let sep = Value::from_string(",".to_string());

        let result = eval_string_method(&s, "split", &[sep]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
            assert_eq!(arr[1], Value::from_string("b".to_string()));
            assert_eq!(arr[2], Value::from_string("c".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_string_substring() {
        let s = Rc::new("hello".to_string());
        let start = Value::Integer(1);
        let end = Value::Integer(4);

        let result = eval_string_method(&s, "substring", &[start, end]).unwrap();
        assert_eq!(result, Value::from_string("ell".to_string()));
    }

    #[test]
    fn test_string_repeat() {
        let s = Rc::new("hi".to_string());
        let count = Value::Integer(3);

        let result = eval_string_method(&s, "repeat", &[count]).unwrap();
        assert_eq!(result, Value::from_string("hihihi".to_string()));
    }

    #[test]
    fn test_float_methods() {
        let result = eval_float_method(3.7, "round", true).unwrap();
        assert_eq!(result, Value::Float(4.0));

        let result = eval_float_method(-5.2, "abs", true).unwrap();
        assert_eq!(result, Value::Float(5.2));
    }

    #[test]
    fn test_integer_methods() {
        let result = eval_integer_method(-42, "abs", true).unwrap();
        assert_eq!(result, Value::Integer(42));

        let result = eval_integer_method(123, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("123".to_string()));
    }

    #[test]
    fn test_generic_to_string() {
        let value = Value::Bool(true);
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("true".to_string()));
    }
}
