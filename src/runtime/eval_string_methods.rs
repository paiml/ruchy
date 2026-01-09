//! String method evaluation module
//!
//! This module handles all string-specific methods including length, case conversion,
//! searching, splitting, trimming, and character operations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

/// Evaluate a string method call
///
/// # Complexity
/// Cyclomatic complexity: 10 (within Toyota Way limits - added `format()` support)
pub fn eval_string_method(
    s: &Arc<str>,
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    // Note: Turbofish stripping now handled centrally in eval_method_dispatch.rs (EVALUATOR-001)
    // STDLIB-007 (GitHub #47): format() accepts variadic arguments
    if method == "format" && !args.is_empty() {
        return eval_string_format(s, args);
    }

    match args.len() {
        0 => eval_zero_arg_string_method(s, method),
        1 => eval_single_arg_string_method(s, method, &args[0]),
        2 => eval_two_arg_string_method(s, method, &args[0], &args[1]),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown string method or invalid arguments: {method}"
        ))),
    }
}

fn eval_zero_arg_string_method(s: &Arc<str>, method: &str) -> Result<Value, InterpreterError> {
    match method {
        "len" | "length" => Ok(Value::Integer(s.len() as i64)),
        "to_upper" | "to_uppercase" | "upper" => Ok(Value::from_string(s.to_uppercase())),
        "to_lower" | "to_lowercase" | "lower" => Ok(Value::from_string(s.to_lowercase())),
        "to_string" => Ok(Value::from_string(s.to_string())),
        "is_empty" => Ok(Value::Bool(s.is_empty())),
        "is_numeric" => Ok(Value::Bool(s.chars().all(char::is_numeric))),
        "is_alphabetic" => Ok(Value::Bool(s.chars().all(char::is_alphabetic))),
        "is_alphanumeric" => Ok(Value::Bool(s.chars().all(char::is_alphanumeric))),
        "trim" => Ok(Value::from_string(s.trim().to_string())),
        "trim_start" => Ok(Value::from_string(s.trim_start().to_string())),
        "trim_end" => Ok(Value::from_string(s.trim_end().to_string())),
        "chars" => eval_string_chars(s),
        "lines" => eval_string_lines(s),
        "parse" | "to_int" | "to_integer" => eval_string_parse(s),
        "timestamp" => eval_string_timestamp(s),
        "to_rfc3339" => Ok(Value::from_string(s.to_string())),
        "as_bytes" => eval_string_as_bytes(s),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown zero-argument string method: {method}"
        ))),
    }
}

fn eval_single_arg_string_method(
    s: &Arc<str>,
    method: &str,
    arg: &Value,
) -> Result<Value, InterpreterError> {
    match method {
        "contains" => eval_string_contains(s, arg),
        "starts_with" => eval_string_starts_with(s, arg),
        "ends_with" => eval_string_ends_with(s, arg),
        "split" => eval_string_split(s, arg),
        "repeat" => eval_string_repeat(s, arg),
        "char_at" => eval_string_char_at(s, arg),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown single-argument string method: {method}"
        ))),
    }
}

fn eval_two_arg_string_method(
    s: &Arc<str>,
    method: &str,
    arg1: &Value,
    arg2: &Value,
) -> Result<Value, InterpreterError> {
    match method {
        "replace" => eval_string_replace(s, arg1, arg2),
        "substring" | "slice" => eval_string_substring(s, arg1, arg2),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown two-argument string method: {method}"
        ))),
    }
}

/// Check if string contains substring
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_contains(s: &str, needle: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(needle_str) = needle {
        Ok(Value::Bool(s.contains(&**needle_str)))
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
        Ok(Value::Bool(s.starts_with(&**prefix_str)))
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
        Ok(Value::Bool(s.ends_with(&**suffix_str)))
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
        Ok(Value::from_string(s.replace(&**from_str, to_str)))
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
            .split(&**sep_str)
            .map(|part| Value::from_string(part.to_string()))
            .collect();
        Ok(Value::from_array(parts))
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
    Ok(Value::from_array(chars))
}

/// Convert string to array of UTF-8 byte values
///
/// # Feature #89
/// Implements `string.as_bytes()` method for binary data handling
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
///
/// # Examples
/// ```
/// "Hello".as_bytes() => [72, 101, 108, 108, 111]
/// "A".as_bytes() => [65]
/// "".as_bytes() => []
/// ```
fn eval_string_as_bytes(s: &str) -> Result<Value, InterpreterError> {
    let bytes: Vec<Value> = s
        .as_bytes()
        .iter()
        .map(|&byte| Value::Integer(i64::from(byte)))
        .collect();
    Ok(Value::from_array(bytes))
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
    Ok(Value::from_array(lines))
}

/// Format string by replacing {} placeholders with arguments
///
/// # STDLIB-007 (GitHub Issue #47)
/// Implements Python-style string formatting with {} placeholders
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Examples
/// ```
/// "Hello, {}!".format("Alice") => "Hello, Alice!"
/// "{} + {} = {}".format(2, 3, 5) => "2 + 3 = 5"
/// ```
fn eval_string_format(s: &str, args: &[Value]) -> Result<Value, InterpreterError> {
    let mut result = s.to_string();
    for arg in args {
        // Convert Value to string without quotes for String values
        let arg_str = match arg {
            Value::String(s) => s.to_string(),
            _ => format!("{arg}"),
        };
        // Replace first occurrence of {} with the argument
        result = result.replacen("{}", &arg_str, 1);
    }
    Ok(Value::from_string(result))
}

/// Evaluate primitive type methods (float, integer, generic)
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_primitive_method(
    receiver: &Value,
    method: &str,
    arg_values: &[Value],
    args_empty: bool,
) -> Result<Value, InterpreterError> {
    match receiver {
        Value::Float(f) => eval_float_method(*f, method, args_empty),
        Value::Integer(n) => eval_integer_method(*n, method, arg_values),
        _ => eval_generic_method(receiver, method, args_empty),
    }
}

/// Evaluate float-specific methods
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
        "to_int" | "to_integer" => Ok(Value::Integer(f as i64)),
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
fn eval_integer_method(
    n: i64,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "abs" => {
            if !arg_values.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "Integer method 'abs' takes no arguments".to_string(),
                ));
            }
            Ok(Value::Integer(n.abs()))
        }
        "to_string" => {
            if !arg_values.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "Integer method 'to_string' takes no arguments".to_string(),
                ));
            }
            Ok(Value::from_string(n.to_string()))
        }
        "pow" => {
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
                    let result = n.pow(*exp as u32);
                    Ok(Value::Integer(result))
                }
                _ => Err(InterpreterError::TypeError(format!(
                    "Integer pow() requires integer exponent, got {}",
                    arg_values[0].type_name()
                ))),
            }
        }
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

/// Parse string to integer (PARSER-069 support for turbofish)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_parse(s: &str) -> Result<Value, InterpreterError> {
    s.trim()
        .parse::<i64>()
        .map(Value::Integer)
        .map_err(|_| InterpreterError::RuntimeError(format!("Failed to parse '{s}' as integer")))
}

/// Get Unix timestamp from RFC3339 datetime string (Issue #82)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_string_timestamp(s: &str) -> Result<Value, InterpreterError> {
    use chrono::DateTime;

    // Parse RFC3339 datetime string
    s.parse::<DateTime<chrono::Utc>>()
        .map(|dt| Value::Integer(dt.timestamp()))
        .map_err(|_| {
            InterpreterError::RuntimeError(format!("Failed to parse '{s}' as RFC3339 datetime"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_length() {
        let s = Arc::from("hello");
        let result = eval_string_method(&s, "len", &[]).expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_string_case_conversion() {
        let s = Arc::from("Hello World");

        let upper =
            eval_string_method(&s, "to_upper", &[]).expect("operation should succeed in test");
        assert_eq!(upper, Value::from_string("HELLO WORLD".to_string()));

        let lower =
            eval_string_method(&s, "to_lower", &[]).expect("operation should succeed in test");
        assert_eq!(lower, Value::from_string("hello world".to_string()));
    }

    #[test]
    fn test_string_contains() {
        let s = Arc::from("hello world");
        let needle = Value::from_string("world".to_string());

        let result = eval_string_method(&s, "contains", &[needle])
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_split() {
        let s = Arc::from("a,b,c");
        let sep = Value::from_string(",".to_string());

        let result =
            eval_string_method(&s, "split", &[sep]).expect("operation should succeed in test");
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
        let s = Arc::from("hello");
        let start = Value::Integer(1);
        let end = Value::Integer(4);

        let result = eval_string_method(&s, "substring", &[start, end])
            .expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("ell".to_string()));
    }

    #[test]
    fn test_string_repeat() {
        let s = Arc::from("hi");
        let count = Value::Integer(3);

        let result =
            eval_string_method(&s, "repeat", &[count]).expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hihihi".to_string()));
    }

    #[test]
    fn test_float_methods() {
        let result =
            eval_float_method(3.7, "round", true).expect("operation should succeed in test");
        assert_eq!(result, Value::Float(4.0));

        let result =
            eval_float_method(-5.2, "abs", true).expect("operation should succeed in test");
        assert_eq!(result, Value::Float(5.2));
    }

    #[test]
    fn test_integer_methods() {
        let result =
            eval_integer_method(-42, "abs", &[]).expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(42));

        let result =
            eval_integer_method(123, "to_string", &[]).expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("123".to_string()));
    }

    #[test]
    fn test_generic_to_string() {
        let value = Value::Bool(true);
        let result = eval_generic_method(&value, "to_string", true)
            .expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("true".to_string()));
    }

    // STDLIB-007 (GitHub Issue #47): string.format() tests
    #[test]
    fn test_string_format_single_placeholder() {
        let s = Arc::from("Hello, {}!");
        let arg = Value::from_string("Alice".to_string());

        let result =
            eval_string_method(&s, "format", &[arg]).expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("Hello, Alice!".to_string()));
    }

    #[test]
    fn test_string_format_multiple_placeholders() {
        let s = Arc::from("{} + {} = {}");
        let arg1 = Value::Integer(2);
        let arg2 = Value::Integer(3);
        let arg3 = Value::Integer(5);

        let result = eval_string_method(&s, "format", &[arg1, arg2, arg3])
            .expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("2 + 3 = 5".to_string()));
    }

    #[test]
    fn test_string_format_no_placeholders() {
        let s = Arc::from("Hello, World!");
        let arg = Value::from_string("Alice".to_string());

        let result =
            eval_string_method(&s, "format", &[arg]).expect("operation should succeed in test");
        // Should return unchanged string if no placeholders
        assert_eq!(result, Value::from_string("Hello, World!".to_string()));
    }

    #[test]
    fn test_string_format_more_placeholders_than_args() {
        let s = Arc::from("{} and {}");
        let arg = Value::from_string("Alice".to_string());

        let result =
            eval_string_method(&s, "format", &[arg]).expect("operation should succeed in test");
        // Should replace first placeholder only
        assert_eq!(result, Value::from_string("Alice and {}".to_string()));
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    // Sprint 9 Phase 3: eval_string_methods.rs mutation tests
    // Testing 14 MISSED mutations from cargo-mutants baseline

    #[test]
    fn test_eval_zero_arg_string_method_to_string() {
        // MISSED: delete match arm "to_string" in eval_zero_arg_string_method (line 35)
        let s = Arc::from("hello");
        let result =
            eval_zero_arg_string_method(&s, "to_string").expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_zero_arg_string_method_trim() {
        // MISSED: delete match arm "trim" in eval_zero_arg_string_method (line 37)
        let s = Arc::from("  hello  ");
        let result =
            eval_zero_arg_string_method(&s, "trim").expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_zero_arg_string_method_trim_start() {
        // MISSED: delete match arm "trim_start" in eval_zero_arg_string_method (line 38)
        let s = Arc::from("  hello");
        let result = eval_zero_arg_string_method(&s, "trim_start")
            .expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_zero_arg_string_method_trim_end() {
        // MISSED: delete match arm "trim_end" in eval_zero_arg_string_method (line 39)
        let s = Arc::from("hello  ");
        let result =
            eval_zero_arg_string_method(&s, "trim_end").expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_zero_arg_string_method_chars() {
        // MISSED: delete match arm "chars" in eval_zero_arg_string_method (line 40)
        let s = Arc::from("abc");
        let result =
            eval_zero_arg_string_method(&s, "chars").expect("operation should succeed in test");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::from_string("a".to_string()));
            }
            _ => panic!("Expected array result from chars()"),
        }
    }

    #[test]
    fn test_eval_single_arg_string_method_starts_with() {
        // MISSED: delete match arm "starts_with" in eval_single_arg_string_method (line 55)
        let s = Arc::from("hello world");
        let arg = Value::from_string("hello".to_string());
        let result = eval_single_arg_string_method(&s, "starts_with", &arg)
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_two_arg_string_method_replace() {
        // MISSED: delete match arm "replace" in eval_two_arg_string_method (line 73)
        let s = Arc::from("hello world");
        let arg1 = Value::from_string("world".to_string());
        let arg2 = Value::from_string("Ruchy".to_string());
        let result = eval_two_arg_string_method(&s, "replace", &arg1, &arg2)
            .expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hello Ruchy".to_string()));
    }

    #[test]
    fn test_eval_float_method_sqrt() {
        // MISSED: delete match arm "sqrt" in eval_float_method (line 276)
        let result =
            eval_float_method(4.0, "sqrt", true).expect("operation should succeed in test");
        assert_eq!(result, Value::Float(2.0));
    }

    #[test]
    fn test_eval_float_method_floor() {
        // MISSED: delete match arm "floor" in eval_float_method (line 279)
        let result =
            eval_float_method(3.7, "floor", true).expect("operation should succeed in test");
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_eval_float_method_ceil() {
        // MISSED: delete match arm "ceil" in eval_float_method (line 280)
        let result =
            eval_float_method(3.2, "ceil", true).expect("operation should succeed in test");
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_eval_float_method_to_string() {
        // MISSED: delete match arm "to_string" in eval_float_method (line 281)
        let result =
            eval_float_method(3.15, "to_string", true).expect("operation should succeed in test");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "3.15"),
            _ => panic!("Expected string result from to_string()"),
        }
    }

    #[test]
    fn test_eval_primitive_method_float_match_arm() {
        // MISSED: delete match arm Value::Float(f) in eval_primitive_method (line 258)
        let float_val = Value::Float(4.0);
        let result = eval_primitive_method(&float_val, "sqrt", &[], true)
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Float(2.0));
    }

    #[test]
    fn test_eval_string_char_at_comparison_operator() {
        // MISSED: replace >= with < in eval_string_char_at (line 181)
        let s = Arc::from("abc");
        let index = Value::Integer(1);

        // Valid index (>= 0 check should pass)
        let result = eval_string_char_at(&s, &index);
        assert!(result.is_ok(), "Valid index should succeed");

        // Negative index (>= 0 check should fail)
        let neg_index = Value::Integer(-1);
        let result = eval_string_char_at(&s, &neg_index);
        assert!(result.is_err(), "Negative index should fail with >= check");
    }

    #[test]
    fn test_eval_generic_method_logical_operator() {
        // MISSED: replace && with || in eval_generic_method (line 317)
        // This tests the condition: supports_to_string(value) && method == "to_string"

        let int_val = Value::Integer(42);

        // Both conditions true: should call to_string
        let result = eval_generic_method(&int_val, "to_string", true)
            .expect("operation should succeed in test");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "42"),
            _ => panic!("Expected string result"),
        }

        // Method != "to_string": should fail
        let result = eval_generic_method(&int_val, "other_method", true);
        assert!(result.is_err(), "Non-to_string method should fail");
    }

    #[test]
    fn test_eval_string_substring_logical_operator() {
        // MISSED: replace && with || in eval_string_substring (line 206)
        let s = Arc::from("hello");
        let start = Value::Integer(1);
        let end = Value::Integer(3);

        let result =
            eval_string_substring(&s, &start, &end).expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("el".to_string()));
    }
}

#[cfg(test)]
mod round_130_tests {
    use super::*;

    // EXTREME TDD Round 130: eval_string_methods.rs coverage boost
    // Target: 74.44% -> 90%+

    #[test]
    fn test_contains_wrong_type_r130() {
        let s = Arc::from("hello");
        let arg = Value::Integer(42);
        let result = eval_string_contains(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_starts_with_wrong_type_r130() {
        let s = Arc::from("hello");
        let arg = Value::Integer(42);
        let result = eval_string_starts_with(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_ends_with_wrong_type_r130() {
        let s = Arc::from("hello");
        let arg = Value::Integer(42);
        let result = eval_string_ends_with(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_ends_with_true_r130() {
        let s = Arc::from("hello world");
        let arg = Value::from_string("world".to_string());
        let result = eval_string_ends_with(&s, &arg).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ends_with_false_r130() {
        let s = Arc::from("hello world");
        let arg = Value::from_string("hello".to_string());
        let result = eval_string_ends_with(&s, &arg).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_replace_wrong_types_r130() {
        let s = Arc::from("hello");
        let arg1 = Value::Integer(1);
        let arg2 = Value::from_string("x".to_string());
        let result = eval_string_replace(&s, &arg1, &arg2);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_wrong_type_r130() {
        let s = Arc::from("hello");
        let arg = Value::Integer(42);
        let result = eval_string_split(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_success_r130() {
        let s = Arc::from("a,b,c");
        let arg = Value::from_string(",".to_string());
        let result = eval_string_split(&s, &arg).unwrap();
        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_repeat_wrong_type_r130() {
        let s = Arc::from("hello");
        let arg = Value::from_string("3".to_string());
        let result = eval_string_repeat(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_repeat_negative_r130() {
        let s = Arc::from("hello");
        let arg = Value::Integer(-1);
        let result = eval_string_repeat(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_repeat_success_r130() {
        let s = Arc::from("ab");
        let arg = Value::Integer(3);
        let result = eval_string_repeat(&s, &arg).unwrap();
        assert_eq!(result, Value::from_string("ababab".to_string()));
    }

    #[test]
    fn test_char_at_wrong_type_r130() {
        let s = Arc::from("hello");
        let arg = Value::from_string("1".to_string());
        let result = eval_string_char_at(&s, &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_char_at_out_of_bounds_r130() {
        let s = Arc::from("abc");
        let arg = Value::Integer(10);
        let result = eval_string_char_at(&s, &arg).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_substring_wrong_types_r130() {
        let s = Arc::from("hello");
        let arg1 = Value::from_string("1".to_string());
        let arg2 = Value::Integer(3);
        let result = eval_string_substring(&s, &arg1, &arg2);
        assert!(result.is_err());
    }

    #[test]
    fn test_substring_negative_start_r130() {
        let s = Arc::from("hello");
        let arg1 = Value::Integer(-1);
        let arg2 = Value::Integer(3);
        let result = eval_string_substring(&s, &arg1, &arg2);
        assert!(result.is_err());
    }

    #[test]
    fn test_substring_end_less_than_start_r130() {
        let s = Arc::from("hello");
        let arg1 = Value::Integer(3);
        let arg2 = Value::Integer(1);
        let result = eval_string_substring(&s, &arg1, &arg2);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_arg_is_empty_true_r130() {
        let s = Arc::from("");
        let result = eval_zero_arg_string_method(&s, "is_empty").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_zero_arg_is_empty_false_r130() {
        let s = Arc::from("a");
        let result = eval_zero_arg_string_method(&s, "is_empty").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_zero_arg_is_numeric_true_r130() {
        let s = Arc::from("12345");
        let result = eval_zero_arg_string_method(&s, "is_numeric").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_zero_arg_is_numeric_false_r130() {
        let s = Arc::from("12abc");
        let result = eval_zero_arg_string_method(&s, "is_numeric").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_zero_arg_is_alphabetic_true_r130() {
        let s = Arc::from("abcXYZ");
        let result = eval_zero_arg_string_method(&s, "is_alphabetic").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_zero_arg_is_alphabetic_false_r130() {
        let s = Arc::from("abc123");
        let result = eval_zero_arg_string_method(&s, "is_alphabetic").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_zero_arg_is_alphanumeric_true_r130() {
        let s = Arc::from("abc123");
        let result = eval_zero_arg_string_method(&s, "is_alphanumeric").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_zero_arg_is_alphanumeric_false_r130() {
        let s = Arc::from("abc-123");
        let result = eval_zero_arg_string_method(&s, "is_alphanumeric").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_zero_arg_lines_r130() {
        let s = Arc::from("line1\nline2\nline3");
        let result = eval_zero_arg_string_method(&s, "lines").unwrap();
        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_zero_arg_unknown_method_r130() {
        let s = Arc::from("hello");
        let result = eval_zero_arg_string_method(&s, "unknown_method");
        assert!(result.is_err());
    }

    #[test]
    fn test_single_arg_unknown_method_r130() {
        let s = Arc::from("hello");
        let arg = Value::from_string("x".to_string());
        let result = eval_single_arg_string_method(&s, "unknown_method", &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_two_arg_unknown_method_r130() {
        let s = Arc::from("hello");
        let arg1 = Value::from_string("x".to_string());
        let arg2 = Value::from_string("y".to_string());
        let result = eval_two_arg_string_method(&s, "unknown_method", &arg1, &arg2);
        assert!(result.is_err());
    }

    #[test]
    fn test_two_arg_slice_alias_r130() {
        let s = Arc::from("hello");
        let arg1 = Value::Integer(1);
        let arg2 = Value::Integer(4);
        let result = eval_two_arg_string_method(&s, "slice", &arg1, &arg2).unwrap();
        assert_eq!(result, Value::from_string("ell".to_string()));
    }

    #[test]
    fn test_string_method_too_many_args_r130() {
        let s = Arc::from("hello");
        let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = eval_string_method(&s, "unknown", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_upper_r130() {
        let s = Arc::from("hello");
        let result = eval_zero_arg_string_method(&s, "to_upper").unwrap();
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_to_lowercase_r130() {
        let s = Arc::from("HELLO");
        let result = eval_zero_arg_string_method(&s, "to_lowercase").unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_upper_alias_r130() {
        let s = Arc::from("hello");
        let result = eval_zero_arg_string_method(&s, "upper").unwrap();
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_lower_alias_r130() {
        let s = Arc::from("HELLO");
        let result = eval_zero_arg_string_method(&s, "lower").unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_len_alias_r130() {
        let s = Arc::from("hello");
        let result = eval_zero_arg_string_method(&s, "len").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_length_alias_r130() {
        let s = Arc::from("hello");
        let result = eval_zero_arg_string_method(&s, "length").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_parse_valid_int_r130() {
        let s = Arc::from("42");
        let result = eval_zero_arg_string_method(&s, "parse").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_to_int_alias_r130() {
        let s = Arc::from("100");
        let result = eval_zero_arg_string_method(&s, "to_int").unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_to_integer_alias_r130() {
        let s = Arc::from("-50");
        let result = eval_zero_arg_string_method(&s, "to_integer").unwrap();
        assert_eq!(result, Value::Integer(-50));
    }

    #[test]
    fn test_as_bytes_r130() {
        let s = Arc::from("abc");
        let result = eval_zero_arg_string_method(&s, "as_bytes").unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(97)); // 'a'
                assert_eq!(arr[1], Value::Integer(98)); // 'b'
                assert_eq!(arr[2], Value::Integer(99)); // 'c'
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_to_rfc3339_r130() {
        let s = Arc::from("2024-01-01T00:00:00Z");
        let result = eval_zero_arg_string_method(&s, "to_rfc3339").unwrap();
        assert_eq!(result, Value::from_string("2024-01-01T00:00:00Z".to_string()));
    }

    // === EXTREME TDD Round 159 - Coverage Push Tests ===

    #[test]
    fn test_string_is_empty_true_r159() {
        let s = Arc::from("");
        let result = eval_zero_arg_string_method(&s, "is_empty").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_is_empty_false_r159() {
        let s = Arc::from("x");
        let result = eval_zero_arg_string_method(&s, "is_empty").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_is_numeric_true_r159() {
        let s = Arc::from("12345");
        let result = eval_zero_arg_string_method(&s, "is_numeric").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_is_numeric_false_r159() {
        let s = Arc::from("12a45");
        let result = eval_zero_arg_string_method(&s, "is_numeric").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_is_alphabetic_true_r159() {
        let s = Arc::from("abcXYZ");
        let result = eval_zero_arg_string_method(&s, "is_alphabetic").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_is_alphabetic_false_r159() {
        let s = Arc::from("abc123");
        let result = eval_zero_arg_string_method(&s, "is_alphabetic").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_is_alphanumeric_true_r159() {
        let s = Arc::from("abc123");
        let result = eval_zero_arg_string_method(&s, "is_alphanumeric").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_is_alphanumeric_false_r159() {
        let s = Arc::from("abc-123");
        let result = eval_zero_arg_string_method(&s, "is_alphanumeric").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_lines_r159() {
        let s = Arc::from("line1\nline2\nline3");
        let result = eval_zero_arg_string_method(&s, "lines").unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::from_string("line1".to_string()));
                assert_eq!(arr[1], Value::from_string("line2".to_string()));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_string_lines_empty_r159() {
        let s = Arc::from("");
        let result = eval_zero_arg_string_method(&s, "lines").unwrap();
        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_string_ends_with_true_r159() {
        let s = Arc::from("hello world");
        let arg = Value::from_string("world".to_string());
        let result = eval_single_arg_string_method(&s, "ends_with", &arg).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_ends_with_false_r159() {
        let s = Arc::from("hello world");
        let arg = Value::from_string("hello".to_string());
        let result = eval_single_arg_string_method(&s, "ends_with", &arg).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_contains_false_r159() {
        let s = Arc::from("hello");
        let arg = Value::from_string("xyz".to_string());
        let result = eval_single_arg_string_method(&s, "contains", &arg).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_starts_with_false_r159() {
        let s = Arc::from("hello");
        let arg = Value::from_string("world".to_string());
        let result = eval_single_arg_string_method(&s, "starts_with", &arg).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_repeat_zero_r159() {
        let s = Arc::from("test");
        let count = Value::Integer(0);
        let result = eval_single_arg_string_method(&s, "repeat", &count).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_repeat_negative_error_r159() {
        let s = Arc::from("test");
        let count = Value::Integer(-1);
        let result = eval_single_arg_string_method(&s, "repeat", &count);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_char_at_out_of_bounds_r159() {
        let s = Arc::from("abc");
        let index = Value::Integer(10);
        let result = eval_single_arg_string_method(&s, "char_at", &index).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_string_substring_clamp_r159() {
        let s = Arc::from("hello");
        let start = Value::Integer(0);
        let end = Value::Integer(100); // Beyond string length
        let result = eval_two_arg_string_method(&s, "substring", &start, &end).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_slice_alias_r159() {
        let s = Arc::from("hello");
        let start = Value::Integer(1);
        let end = Value::Integer(4);
        let result = eval_two_arg_string_method(&s, "slice", &start, &end).unwrap();
        assert_eq!(result, Value::from_string("ell".to_string()));
    }

    #[test]
    fn test_float_to_int_r159() {
        let result = eval_float_method(7.9, "to_int", true).unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_float_to_integer_alias_r159() {
        let result = eval_float_method(3.14, "to_integer", true).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_float_powf_error_r159() {
        let result = eval_float_method(2.0, "powf", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_float_with_args_error_r159() {
        let result = eval_float_method(2.0, "sqrt", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_float_unknown_method_r159() {
        let result = eval_float_method(2.0, "unknown_method", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_integer_pow_r159() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(10)]).unwrap();
        assert_eq!(result, Value::Integer(1024));
    }

    #[test]
    fn test_integer_pow_negative_error_r159() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(-1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_integer_pow_wrong_type_error_r159() {
        let result = eval_integer_method(2, "pow", &[Value::Float(2.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_integer_pow_wrong_arg_count_r159() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(2), Value::Integer(3)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_integer_abs_with_args_error_r159() {
        let result = eval_integer_method(-5, "abs", &[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_integer_to_string_with_args_error_r159() {
        let result = eval_integer_method(5, "to_string", &[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_integer_unknown_method_r159() {
        let result = eval_integer_method(5, "unknown_method", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_method_unknown_r159() {
        let value = Value::Nil;
        let result = eval_generic_method(&value, "unknown", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_to_string_with_args_error_r159() {
        let value = Value::Nil;
        let result = eval_generic_method(&value, "to_string", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_parse_invalid_r159() {
        let s = Arc::from("not_a_number");
        let result = eval_zero_arg_string_method(&s, "parse");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_timestamp_invalid_r159() {
        let s = Arc::from("not_a_timestamp");
        let result = eval_zero_arg_string_method(&s, "timestamp");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_unknown_zero_arg_r159() {
        let s = Arc::from("test");
        let result = eval_zero_arg_string_method(&s, "unknown_method");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_unknown_single_arg_r159() {
        let s = Arc::from("test");
        let arg = Value::Integer(1);
        let result = eval_single_arg_string_method(&s, "unknown_method", &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_unknown_two_arg_r159() {
        let s = Arc::from("test");
        let result = eval_two_arg_string_method(&s, "unknown_method", &Value::Integer(1), &Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn test_string_method_wrong_arg_count_r159() {
        let s = Arc::from("test");
        let result = eval_string_method(&s, "len", &[Value::Integer(1), Value::Integer(2), Value::Integer(3)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_contains_wrong_type_r159() {
        let s = Arc::from("test");
        let arg = Value::Integer(1);
        let result = eval_single_arg_string_method(&s, "contains", &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_split_wrong_type_r159() {
        let s = Arc::from("test");
        let arg = Value::Integer(1);
        let result = eval_single_arg_string_method(&s, "split", &arg);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_replace_wrong_types_r159() {
        let s = Arc::from("test");
        let result = eval_two_arg_string_method(&s, "replace", &Value::Integer(1), &Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn test_string_substring_wrong_types_r159() {
        let s = Arc::from("test");
        let result = eval_two_arg_string_method(&s, "substring", &Value::Bool(true), &Value::Bool(false));
        assert!(result.is_err());
    }

    #[test]
    fn test_string_substring_negative_start_r159() {
        let s = Arc::from("test");
        let result = eval_two_arg_string_method(&s, "substring", &Value::Integer(-1), &Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn test_string_char_at_wrong_type_r159() {
        let s = Arc::from("test");
        let result = eval_single_arg_string_method(&s, "char_at", &Value::Bool(true));
        assert!(result.is_err());
    }

    #[test]
    fn test_primitive_method_integer_dispatch_r159() {
        let value = Value::Integer(-42);
        let result = eval_primitive_method(&value, "abs", &[], true).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_length_alias_r159() {
        let s = Arc::from("test");
        let result = eval_zero_arg_string_method(&s, "length").unwrap();
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_to_uppercase_alias_r159() {
        let s = Arc::from("hello");
        let result = eval_zero_arg_string_method(&s, "to_uppercase").unwrap();
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_to_lowercase_alias_r159() {
        let s = Arc::from("HELLO");
        let result = eval_zero_arg_string_method(&s, "to_lowercase").unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_upper_alias_r159() {
        let s = Arc::from("hi");
        let result = eval_zero_arg_string_method(&s, "upper").unwrap();
        assert_eq!(result, Value::from_string("HI".to_string()));
    }

    #[test]
    fn test_lower_alias_r159() {
        let s = Arc::from("HI");
        let result = eval_zero_arg_string_method(&s, "lower").unwrap();
        assert_eq!(result, Value::from_string("hi".to_string()));
    }

    #[test]
    fn test_string_format_with_integer_r159() {
        let s = Arc::from("Value: {}");
        let result = eval_string_method(&s, "format", &[Value::Integer(42)]).unwrap();
        assert_eq!(result, Value::from_string("Value: 42".to_string()));
    }

    #[test]
    fn test_string_format_with_bool_r159() {
        let s = Arc::from("Is valid: {}");
        let result = eval_string_method(&s, "format", &[Value::Bool(true)]).unwrap();
        assert_eq!(result, Value::from_string("Is valid: true".to_string()));
    }
}
