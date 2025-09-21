//! String method evaluation module
//!
//! This module handles evaluation of string methods in the interpreter.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate a string method call
///
/// # Complexity
/// Cyclomatic complexity: 20 (will be decomposed further)
pub fn eval_string_method(
    s: &Rc<str>,
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        // Simple no-argument methods
        "len" | "length" if args.is_empty() => eval_string_len(s),
        "to_upper" if args.is_empty() => eval_string_to_upper(s),
        "to_lower" if args.is_empty() => eval_string_to_lower(s),
        "trim" if args.is_empty() => eval_string_trim(s),
        "to_string" if args.is_empty() => eval_string_to_string(s),
        "trim_start" if args.is_empty() => eval_string_trim_start(s),
        "trim_end" if args.is_empty() => eval_string_trim_end(s),
        "is_empty" if args.is_empty() => eval_string_is_empty(s),
        "chars" if args.is_empty() => eval_string_chars(s),
        "lines" if args.is_empty() => eval_string_lines(s),

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

        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown string method: {method}"
        ))),
    }
}

// No-argument string methods (complexity <= 3 each)

fn eval_string_len(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::Integer(s.len() as i64))
}

fn eval_string_to_upper(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.to_uppercase()))
}

fn eval_string_to_lower(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.to_lowercase()))
}

fn eval_string_trim(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.trim().to_string()))
}

fn eval_string_to_string(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.to_string()))
}

fn eval_string_trim_start(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.trim_start().to_string()))
}

fn eval_string_trim_end(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.trim_end().to_string()))
}

fn eval_string_is_empty(s: &Rc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::Bool(s.is_empty()))
}

fn eval_string_chars(s: &Rc<str>) -> Result<Value, InterpreterError> {
    let chars: Vec<Value> = s
        .chars()
        .map(|c| Value::from_string(c.to_string()))
        .collect();
    Ok(Value::Array(Rc::from(chars)))
}

fn eval_string_lines(s: &Rc<str>) -> Result<Value, InterpreterError> {
    let lines: Vec<Value> = s
        .lines()
        .map(|line| Value::from_string(line.to_string()))
        .collect();
    Ok(Value::Array(Rc::from(lines)))
}

// Single-argument string methods (complexity <= 5 each)

fn eval_string_contains(s: &Rc<str>, needle: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(needle_str) = needle {
        Ok(Value::Bool(s.contains(&**needle_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "contains expects string argument".to_string(),
        ))
    }
}

fn eval_string_starts_with(s: &Rc<str>, prefix: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(prefix_str) = prefix {
        Ok(Value::Bool(s.starts_with(&**prefix_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "starts_with expects string argument".to_string(),
        ))
    }
}

fn eval_string_ends_with(s: &Rc<str>, suffix: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(suffix_str) = suffix {
        Ok(Value::Bool(s.ends_with(&**suffix_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "ends_with expects string argument".to_string(),
        ))
    }
}

fn eval_string_split(s: &Rc<str>, separator: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(sep_str) = separator {
        let parts: Vec<Value> = s
            .split(&**sep_str)
            .map(|part| Value::from_string(part.to_string()))
            .collect();
        Ok(Value::Array(Rc::from(parts)))
    } else {
        Err(InterpreterError::RuntimeError(
            "split expects string argument".to_string(),
        ))
    }
}

fn eval_string_repeat(s: &Rc<str>, n: &Value) -> Result<Value, InterpreterError> {
    if let Value::Integer(count) = n {
        if *count >= 0 {
            Ok(Value::from_string(s.repeat(*count as usize)))
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

fn eval_string_char_at(s: &Rc<str>, index: &Value) -> Result<Value, InterpreterError> {
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

// Two-argument string methods (complexity <= 8 each)

fn eval_string_replace(s: &Rc<str>, from: &Value, to: &Value) -> Result<Value, InterpreterError> {
    if let (Value::String(from_str), Value::String(to_str)) = (from, to) {
        Ok(Value::from_string(s.replace(&**from_str, to_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "replace expects two string arguments".to_string(),
        ))
    }
}

fn eval_string_substring(
    s: &Rc<str>,
    start: &Value,
    end: &Value,
) -> Result<Value, InterpreterError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_len() {
        let s = Rc::from("hello");
        let result = eval_string_len(&s).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_string_to_upper() {
        let s = Rc::from("hello");
        let result = eval_string_to_upper(&s).unwrap();
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_string_contains() {
        let s = Rc::from("hello world");
        let needle = Value::from_string("world".to_string());
        let result = eval_string_contains(&s, &needle).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_split() {
        let s = Rc::from("a,b,c");
        let separator = Value::from_string(",".to_string());
        let result = eval_string_split(&s, &separator).unwrap();
        if let Value::Array(parts) = result {
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], Value::from_string("a".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_string_replace() {
        let s = Rc::from("hello world");
        let from = Value::from_string("world".to_string());
        let to = Value::from_string("Rust".to_string());
        let result = eval_string_replace(&s, &from, &to).unwrap();
        assert_eq!(result, Value::from_string("hello Rust".to_string()));
    }
}
