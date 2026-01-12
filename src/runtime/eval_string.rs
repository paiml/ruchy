//! String method evaluation module
//!
//! This module handles evaluation of string methods in the interpreter.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

/// Evaluate a string method call
///
/// # Complexity
/// Cyclomatic complexity: 20 (will be decomposed further)
pub fn eval_string_method(
    s: &Arc<str>,
    method: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    match args.len() {
        0 => dispatch_zero_arg_string_method(s, method),
        1 => dispatch_single_arg_string_method(s, method, &args[0]),
        2 => dispatch_two_arg_string_method(s, method, &args[0], &args[1]),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Invalid argument count for string method: {method}"
        ))),
    }
}

fn dispatch_zero_arg_string_method(s: &Arc<str>, method: &str) -> Result<Value, InterpreterError> {
    match method {
        "len" | "length" => eval_string_len(s),
        "to_upper" => eval_string_to_upper(s),
        "to_lower" => eval_string_to_lower(s),
        "trim" => eval_string_trim(s),
        "to_string" => eval_string_to_string(s),
        "trim_start" => eval_string_trim_start(s),
        "trim_end" => eval_string_trim_end(s),
        "is_empty" => eval_string_is_empty(s),
        "chars" => eval_string_chars(s),
        "lines" => eval_string_lines(s),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown zero-argument string method: {method}"
        ))),
    }
}

fn dispatch_single_arg_string_method(
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

fn dispatch_two_arg_string_method(
    s: &Arc<str>,
    method: &str,
    arg1: &Value,
    arg2: &Value,
) -> Result<Value, InterpreterError> {
    match method {
        "replace" => eval_string_replace(s, arg1, arg2),
        "substring" => eval_string_substring(s, arg1, arg2),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown two-argument string method: {method}"
        ))),
    }
}

// No-argument string methods (complexity <= 3 each)

fn eval_string_len(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::Integer(s.len() as i64))
}

fn eval_string_to_upper(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.to_uppercase()))
}

fn eval_string_to_lower(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.to_lowercase()))
}

fn eval_string_trim(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.trim().to_string()))
}

fn eval_string_to_string(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.to_string()))
}

fn eval_string_trim_start(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.trim_start().to_string()))
}

fn eval_string_trim_end(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::from_string(s.trim_end().to_string()))
}

fn eval_string_is_empty(s: &Arc<str>) -> Result<Value, InterpreterError> {
    Ok(Value::Bool(s.is_empty()))
}

fn eval_string_chars(s: &Arc<str>) -> Result<Value, InterpreterError> {
    let chars: Vec<Value> = s
        .chars()
        .map(|c| Value::from_string(c.to_string()))
        .collect();
    Ok(Value::Array(Arc::from(chars)))
}

fn eval_string_lines(s: &Arc<str>) -> Result<Value, InterpreterError> {
    let lines: Vec<Value> = s
        .lines()
        .map(|line| Value::from_string(line.to_string()))
        .collect();
    Ok(Value::Array(Arc::from(lines)))
}

// Single-argument string methods (complexity <= 5 each)

fn eval_string_contains(s: &Arc<str>, needle: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(needle_str) = needle {
        Ok(Value::Bool(s.contains(&**needle_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "contains expects string argument".to_string(),
        ))
    }
}

fn eval_string_starts_with(s: &Arc<str>, prefix: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(prefix_str) = prefix {
        Ok(Value::Bool(s.starts_with(&**prefix_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "starts_with expects string argument".to_string(),
        ))
    }
}

fn eval_string_ends_with(s: &Arc<str>, suffix: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(suffix_str) = suffix {
        Ok(Value::Bool(s.ends_with(&**suffix_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "ends_with expects string argument".to_string(),
        ))
    }
}

fn eval_string_split(s: &Arc<str>, separator: &Value) -> Result<Value, InterpreterError> {
    if let Value::String(sep_str) = separator {
        let parts: Vec<Value> = s
            .split(&**sep_str)
            .map(|part| Value::from_string(part.to_string()))
            .collect();
        Ok(Value::Array(Arc::from(parts)))
    } else {
        Err(InterpreterError::RuntimeError(
            "split expects string argument".to_string(),
        ))
    }
}

fn eval_string_repeat(s: &Arc<str>, n: &Value) -> Result<Value, InterpreterError> {
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

fn eval_string_char_at(s: &Arc<str>, index: &Value) -> Result<Value, InterpreterError> {
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

fn eval_string_replace(s: &Arc<str>, from: &Value, to: &Value) -> Result<Value, InterpreterError> {
    if let (Value::String(from_str), Value::String(to_str)) = (from, to) {
        Ok(Value::from_string(s.replace(&**from_str, to_str)))
    } else {
        Err(InterpreterError::RuntimeError(
            "replace expects two string arguments".to_string(),
        ))
    }
}

fn eval_string_substring(
    s: &Arc<str>,
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
        let s = Arc::from("hello");
        let result = eval_string_len(&s).expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_string_to_upper() {
        let s = Arc::from("hello");
        let result = eval_string_to_upper(&s).expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_string_contains() {
        let s = Arc::from("hello world");
        let needle = Value::from_string("world".to_string());
        let result = eval_string_contains(&s, &needle).expect("operation should succeed in test");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_split() {
        let s = Arc::from("a,b,c");
        let separator = Value::from_string(",".to_string());
        let result = eval_string_split(&s, &separator).expect("operation should succeed in test");
        if let Value::Array(parts) = result {
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], Value::from_string("a".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_string_replace() {
        let s = Arc::from("hello world");
        let from = Value::from_string("world".to_string());
        let to = Value::from_string("Rust".to_string());
        let result = eval_string_replace(&s, &from, &to).expect("operation should succeed in test");
        assert_eq!(result, Value::from_string("hello Rust".to_string()));
    }

    #[test]
    fn test_eval_string_method_match_arm_zero_args() {
        // Mutation test: Verify match arm for 0 args exists
        // MISSED: delete match arm 0 in eval_string_method (line 20)

        let s = Arc::from("hello");

        // Test zero-arg method dispatch works
        let result = eval_string_method(&s, "len", &[]);
        assert!(
            result.is_ok(),
            "Zero-arg dispatch should work (match arm 0)"
        );
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::Integer(5)
        );

        // Test that wrong arg count fails
        let result = eval_string_method(&s, "len", &[Value::Integer(1)]);
        assert!(
            result.is_err(),
            "len with 1 arg should fail (proves match on arg count)"
        );
    }

    #[test]
    fn test_dispatch_zero_arg_string_method_trim_start() {
        // Mutation test: Verify "trim_start" match arm exists
        // MISSED: delete match arm "trim_start" in dispatch_zero_arg_string_method (line 36)

        let s = Arc::from("  hello  ");

        // Test trim_start method exists
        let result = dispatch_zero_arg_string_method(&s, "trim_start");
        assert!(
            result.is_ok(),
            "trim_start method should exist (match arm test)"
        );
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::from_string("hello  ".to_string())
        );

        // Also test it actually trims (not just returns the string)
        let s2 = Arc::from("  test");
        let result2 = dispatch_zero_arg_string_method(&s2, "trim_start")
            .expect("operation should succeed in test");
        assert_eq!(result2, Value::from_string("test".to_string()));
    }

    #[test]
    fn test_dispatch_single_arg_string_method_char_at() {
        // Mutation test: Verify "char_at" match arm exists
        // MISSED: delete match arm "char_at" in dispatch_single_arg_string_method (line 58)

        let s = Arc::from("hello");

        // Test char_at method exists
        let result = dispatch_single_arg_string_method(&s, "char_at", &Value::Integer(1));
        assert!(
            result.is_ok(),
            "char_at method should exist (match arm test)"
        );
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::from_string("e".to_string())
        );
    }

    #[test]
    fn test_dispatch_two_arg_string_method_substring() {
        // Mutation test: Verify "substring" match arm exists
        // MISSED: delete match arm "substring" in dispatch_two_arg_string_method (line 73)

        let s = Arc::from("hello");

        // Test substring method exists
        let result =
            dispatch_two_arg_string_method(&s, "substring", &Value::Integer(1), &Value::Integer(4));
        assert!(
            result.is_ok(),
            "substring method should exist (match arm test)"
        );
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::from_string("ell".to_string())
        );
    }

    #[test]
    fn test_eval_string_char_at_comparison_operator() {
        // Mutation test: Verify >= operator (not <) for index validation
        // MISSED: replace >= with < in eval_string_char_at (line 194:17)

        let s = Arc::from("hello");

        // Test with valid non-negative index (>= 0) - should work
        let result = eval_string_char_at(&s, &Value::Integer(0));
        assert!(
            result.is_ok(),
            "char_at with index 0 should work (tests >= 0 check)"
        );
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::from_string("h".to_string())
        );

        // Test with valid positive index
        let result = eval_string_char_at(&s, &Value::Integer(2));
        assert!(result.is_ok(), "char_at with positive index should work");
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::from_string("l".to_string())
        );

        // Test with negative index - should fail (proves >= not <)
        let result = eval_string_char_at(&s, &Value::Integer(-1));
        assert!(
            result.is_err(),
            "char_at with negative index should fail (proves >= operator)"
        );
    }

    #[test]
    fn test_eval_string_substring_boolean_operator() {
        // Mutation test: Verify && operator (not ||) in substring validation
        // MISSED: replace && with || in eval_string_substring (line 231:28)

        let s = Arc::from("hello");

        // Test with valid indices (start >= 0 AND end >= start) - should work
        let result = eval_string_substring(&s, &Value::Integer(1), &Value::Integer(3));
        assert!(
            result.is_ok(),
            "substring with valid indices should work (tests && logic)"
        );
        assert_eq!(
            result.expect("operation should succeed in test"),
            Value::from_string("el".to_string())
        );

        // Test with start < 0 (first condition false) - should fail
        let result = eval_string_substring(&s, &Value::Integer(-1), &Value::Integer(3));
        assert!(
            result.is_err(),
            "substring with start < 0 should fail (proves && not ||)"
        );

        // Test with end < start (second condition false) - should fail
        let result = eval_string_substring(&s, &Value::Integer(3), &Value::Integer(1));
        assert!(
            result.is_err(),
            "substring with end < start should fail (proves && logic)"
        );

        // Test with both conditions true - should work
        let result = eval_string_substring(&s, &Value::Integer(0), &Value::Integer(5));
        assert!(
            result.is_ok(),
            "substring with both conditions true should work"
        );
    }

    // === EXTREME TDD Round 20 tests ===

    #[test]
    fn test_string_to_lower() {
        let s = Arc::from("HELLO");
        let result = eval_string_to_lower(&s).expect("operation should succeed");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_trim() {
        let s = Arc::from("  hello  ");
        let result = eval_string_trim(&s).expect("operation should succeed");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_trim_end() {
        let s = Arc::from("  hello  ");
        let result = eval_string_trim_end(&s).expect("operation should succeed");
        assert_eq!(result, Value::from_string("  hello".to_string()));
    }

    #[test]
    fn test_string_is_empty() {
        let empty = Arc::from("");
        let non_empty = Arc::from("hello");

        assert_eq!(eval_string_is_empty(&empty).unwrap(), Value::Bool(true));
        assert_eq!(
            eval_string_is_empty(&non_empty).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_string_chars() {
        let s = Arc::from("abc");
        let result = eval_string_chars(&s).expect("operation should succeed");
        if let Value::Array(chars) = result {
            assert_eq!(chars.len(), 3);
            assert_eq!(chars[0], Value::from_string("a".to_string()));
            assert_eq!(chars[1], Value::from_string("b".to_string()));
            assert_eq!(chars[2], Value::from_string("c".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_string_lines() {
        let s = Arc::from("line1\nline2\nline3");
        let result = eval_string_lines(&s).expect("operation should succeed");
        if let Value::Array(lines) = result {
            assert_eq!(lines.len(), 3);
            assert_eq!(lines[0], Value::from_string("line1".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_string_starts_with() {
        let s = Arc::from("hello world");
        let prefix = Value::from_string("hello".to_string());
        let result = eval_string_starts_with(&s, &prefix).unwrap();
        assert_eq!(result, Value::Bool(true));

        let wrong_prefix = Value::from_string("world".to_string());
        let result2 = eval_string_starts_with(&s, &wrong_prefix).unwrap();
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_string_ends_with() {
        let s = Arc::from("hello world");
        let suffix = Value::from_string("world".to_string());
        let result = eval_string_ends_with(&s, &suffix).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_repeat() {
        let s = Arc::from("ab");
        let result = eval_string_repeat(&s, &Value::Integer(3)).unwrap();
        assert_eq!(result, Value::from_string("ababab".to_string()));
    }

    #[test]
    fn test_string_repeat_negative() {
        let s = Arc::from("ab");
        let result = eval_string_repeat(&s, &Value::Integer(-1));
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_zero_arg_method() {
        let s = Arc::from("hello");
        let result = dispatch_zero_arg_string_method(&s, "unknown_method");
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Unknown"));
    }

    #[test]
    fn test_unknown_single_arg_method() {
        let s = Arc::from("hello");
        let result = dispatch_single_arg_string_method(&s, "unknown", &Value::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_two_arg_method() {
        let s = Arc::from("hello");
        let result =
            dispatch_two_arg_string_method(&s, "unknown", &Value::Integer(1), &Value::Integer(2));
        assert!(result.is_err());
    }

    // === EXTREME TDD Round 24 - Coverage Push Tests ===

    #[test]
    fn test_string_to_string_method() {
        let s = Arc::from("hello");
        let result = eval_string_to_string(&s).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_length_alias() {
        let s = Arc::from("hello");
        let result = dispatch_zero_arg_string_method(&s, "length").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_string_contains_wrong_type() {
        let s = Arc::from("hello");
        let result = eval_string_contains(&s, &Value::Integer(42));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects string"));
    }

    #[test]
    fn test_string_starts_with_wrong_type() {
        let s = Arc::from("hello");
        let result = eval_string_starts_with(&s, &Value::Integer(42));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects string"));
    }

    #[test]
    fn test_string_ends_with_wrong_type() {
        let s = Arc::from("hello");
        let result = eval_string_ends_with(&s, &Value::Integer(42));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects string"));
    }

    #[test]
    fn test_string_split_wrong_type() {
        let s = Arc::from("hello");
        let result = eval_string_split(&s, &Value::Integer(42));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects string"));
    }

    #[test]
    fn test_string_repeat_wrong_type() {
        let s = Arc::from("hello");
        let result = eval_string_repeat(&s, &Value::from_string("test".to_string()));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects integer"));
    }

    #[test]
    fn test_string_char_at_wrong_type() {
        let s = Arc::from("hello");
        let result = eval_string_char_at(&s, &Value::from_string("test".to_string()));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("expects integer"));
    }

    #[test]
    fn test_string_char_at_out_of_bounds() {
        let s = Arc::from("hello");
        let result = eval_string_char_at(&s, &Value::Integer(100)).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_string_replace_wrong_types() {
        let s = Arc::from("hello");
        // First arg wrong type
        let result =
            eval_string_replace(&s, &Value::Integer(1), &Value::from_string("x".to_string()));
        assert!(result.is_err());

        // Second arg wrong type
        let result2 =
            eval_string_replace(&s, &Value::from_string("l".to_string()), &Value::Integer(1));
        assert!(result2.is_err());
    }

    #[test]
    fn test_string_substring_wrong_types() {
        let s = Arc::from("hello");
        // First arg wrong type
        let result =
            eval_string_substring(&s, &Value::from_string("a".to_string()), &Value::Integer(3));
        assert!(result.is_err());

        // Second arg wrong type
        let result2 =
            eval_string_substring(&s, &Value::Integer(0), &Value::from_string("a".to_string()));
        assert!(result2.is_err());
    }

    #[test]
    fn test_string_substring_clamping() {
        let s = Arc::from("hello");
        // End index beyond string length - should be clamped
        let result = eval_string_substring(&s, &Value::Integer(2), &Value::Integer(100)).unwrap();
        assert_eq!(result, Value::from_string("llo".to_string()));
    }

    #[test]
    fn test_string_method_too_many_args() {
        let s = Arc::from("hello");
        let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = eval_string_method(&s, "replace", &args);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("Invalid argument count"));
    }

    #[test]
    fn test_string_len_empty() {
        let s = Arc::from("");
        let result = eval_string_len(&s).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_string_to_upper_unicode() {
        let s = Arc::from("héllo");
        let result = eval_string_to_upper(&s).unwrap();
        assert_eq!(result, Value::from_string("HÉLLO".to_string()));
    }

    #[test]
    fn test_string_to_lower_unicode() {
        let s = Arc::from("HÉLLO");
        let result = eval_string_to_lower(&s).unwrap();
        assert_eq!(result, Value::from_string("héllo".to_string()));
    }

    #[test]
    fn test_string_chars_unicode() {
        let s = Arc::from("日本");
        let result = eval_string_chars(&s).unwrap();
        if let Value::Array(chars) = result {
            assert_eq!(chars.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_lines_empty() {
        let s = Arc::from("");
        let result = eval_string_lines(&s).unwrap();
        if let Value::Array(lines) = result {
            // Empty string produces zero lines
            assert_eq!(lines.len(), 0);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_lines_crlf() {
        let s = Arc::from("line1\r\nline2");
        let result = eval_string_lines(&s).unwrap();
        if let Value::Array(lines) = result {
            assert_eq!(lines.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_split_empty_separator() {
        let s = Arc::from("hello");
        let sep = Value::from_string("".to_string());
        let result = eval_string_split(&s, &sep).unwrap();
        if let Value::Array(parts) = result {
            // Empty separator splits on every character boundary
            assert!(parts.len() > 1);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_contains_false() {
        let s = Arc::from("hello");
        let needle = Value::from_string("xyz".to_string());
        let result = eval_string_contains(&s, &needle).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_repeat_zero() {
        let s = Arc::from("hello");
        let result = eval_string_repeat(&s, &Value::Integer(0)).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_trim_no_whitespace() {
        let s = Arc::from("hello");
        let result = eval_string_trim(&s).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_trim_start_no_whitespace() {
        let s = Arc::from("hello");
        let result = eval_string_trim_start(&s).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_trim_end_no_whitespace() {
        let s = Arc::from("hello");
        let result = eval_string_trim_end(&s).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_substring_zero_start() {
        let s = Arc::from("hello");
        let result = eval_string_substring(&s, &Value::Integer(0), &Value::Integer(3)).unwrap();
        assert_eq!(result, Value::from_string("hel".to_string()));
    }

    #[test]
    fn test_string_replace_multiple_occurrences() {
        let s = Arc::from("hello hello hello");
        let from = Value::from_string("hello".to_string());
        let to = Value::from_string("hi".to_string());
        let result = eval_string_replace(&s, &from, &to).unwrap();
        assert_eq!(result, Value::from_string("hi hi hi".to_string()));
    }

    #[test]
    fn test_string_replace_no_match() {
        let s = Arc::from("hello");
        let from = Value::from_string("xyz".to_string());
        let to = Value::from_string("abc".to_string());
        let result = eval_string_replace(&s, &from, &to).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_string_method_single_arg_dispatch() {
        let s = Arc::from("hello");
        let args = vec![Value::from_string("l".to_string())];
        let result = eval_string_method(&s, "contains", &args).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_string_method_two_arg_dispatch() {
        let s = Arc::from("hello");
        let args = vec![Value::Integer(1), Value::Integer(4)];
        let result = eval_string_method(&s, "substring", &args).unwrap();
        assert_eq!(result, Value::from_string("ell".to_string()));
    }

    #[test]
    fn test_string_char_at_unicode() {
        let s = Arc::from("日本語");
        let result = eval_string_char_at(&s, &Value::Integer(1)).unwrap();
        assert_eq!(result, Value::from_string("本".to_string()));
    }

    #[test]
    fn test_string_substring_unicode() {
        let s = Arc::from("日本語");
        let result = eval_string_substring(&s, &Value::Integer(0), &Value::Integer(2)).unwrap();
        assert_eq!(result, Value::from_string("日本".to_string()));
    }

    // === EXTREME TDD Round 136 - Push to 70+ Tests ===

    #[test]
    fn test_string_len_unicode() {
        let s = Arc::from("日本語");
        // len returns byte length, not char count
        let result = eval_string_len(&s).unwrap();
        // Each Japanese char is 3 bytes in UTF-8
        assert_eq!(result, Value::Integer(9));
    }

    #[test]
    fn test_string_chars_empty() {
        let s = Arc::from("");
        let result = eval_string_chars(&s).unwrap();
        if let Value::Array(chars) = result {
            assert_eq!(chars.len(), 0);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_split_no_match() {
        let s = Arc::from("hello");
        let sep = Value::from_string("x".to_string());
        let result = eval_string_split(&s, &sep).unwrap();
        if let Value::Array(parts) = result {
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0], Value::from_string("hello".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_split_multiple_separators() {
        let s = Arc::from("a::b::c::d");
        let sep = Value::from_string("::".to_string());
        let result = eval_string_split(&s, &sep).unwrap();
        if let Value::Array(parts) = result {
            assert_eq!(parts.len(), 4);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_starts_with_empty() {
        let s = Arc::from("hello");
        let prefix = Value::from_string("".to_string());
        let result = eval_string_starts_with(&s, &prefix).unwrap();
        // Empty prefix always matches
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_ends_with_empty() {
        let s = Arc::from("hello");
        let suffix = Value::from_string("".to_string());
        let result = eval_string_ends_with(&s, &suffix).unwrap();
        // Empty suffix always matches
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_ends_with_false() {
        let s = Arc::from("hello");
        let suffix = Value::from_string("xyz".to_string());
        let result = eval_string_ends_with(&s, &suffix).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_contains_empty() {
        let s = Arc::from("hello");
        let needle = Value::from_string("".to_string());
        let result = eval_string_contains(&s, &needle).unwrap();
        // Empty needle always found
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_replace_empty_from() {
        let s = Arc::from("hello");
        let from = Value::from_string("".to_string());
        let to = Value::from_string("X".to_string());
        let result = eval_string_replace(&s, &from, &to).unwrap();
        // Empty string replacement inserts between each char
        if let Value::String(result_str) = result {
            assert!(result_str.contains("X"));
        }
    }

    #[test]
    fn test_string_replace_to_empty() {
        let s = Arc::from("hello");
        let from = Value::from_string("l".to_string());
        let to = Value::from_string("".to_string());
        let result = eval_string_replace(&s, &from, &to).unwrap();
        assert_eq!(result, Value::from_string("heo".to_string()));
    }

    #[test]
    fn test_string_substring_start_equals_end() {
        let s = Arc::from("hello");
        let result = eval_string_substring(&s, &Value::Integer(2), &Value::Integer(2)).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_repeat_one() {
        let s = Arc::from("hello");
        let result = eval_string_repeat(&s, &Value::Integer(1)).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_char_at_first() {
        let s = Arc::from("hello");
        let result = eval_string_char_at(&s, &Value::Integer(0)).unwrap();
        assert_eq!(result, Value::from_string("h".to_string()));
    }

    #[test]
    fn test_string_char_at_last() {
        let s = Arc::from("hello");
        let result = eval_string_char_at(&s, &Value::Integer(4)).unwrap();
        assert_eq!(result, Value::from_string("o".to_string()));
    }

    #[test]
    fn test_string_trim_only_whitespace() {
        let s = Arc::from("   ");
        let result = eval_string_trim(&s).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_trim_start_only_whitespace() {
        let s = Arc::from("   ");
        let result = eval_string_trim_start(&s).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_trim_end_only_whitespace() {
        let s = Arc::from("   ");
        let result = eval_string_trim_end(&s).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_lines_trailing_newline() {
        let s = Arc::from("line1\nline2\n");
        let result = eval_string_lines(&s).unwrap();
        if let Value::Array(lines) = result {
            // Trailing newline creates empty string at end
            assert_eq!(lines.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_to_upper_mixed() {
        let s = Arc::from("HeLLo WoRLd");
        let result = eval_string_to_upper(&s).unwrap();
        assert_eq!(result, Value::from_string("HELLO WORLD".to_string()));
    }

    #[test]
    fn test_string_to_lower_mixed() {
        let s = Arc::from("HeLLo WoRLd");
        let result = eval_string_to_lower(&s).unwrap();
        assert_eq!(result, Value::from_string("hello world".to_string()));
    }
}
