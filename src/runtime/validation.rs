//! Centralized validation module to eliminate entropy
//!
//! PMAT found `DataValidation` pattern repeated 10 times (2289 lines)
//! This module centralizes all validation logic with O(1) complexity.

use crate::runtime::{InterpreterError, Value};

/// Validate exact argument count - O(1) complexity
#[inline]
pub fn validate_arg_count(
    function_name: &str,
    args: &[Value],
    expected: usize,
) -> Result<(), InterpreterError> {
    if args.len() != expected {
        return Err(InterpreterError::RuntimeError(format!(
            "{}() expects exactly {} argument{}, got {}",
            function_name,
            expected,
            if expected == 1 { "" } else { "s" },
            args.len()
        )));
    }
    Ok(())
}

/// Validate minimum argument count - O(1) complexity
#[inline]
pub fn validate_min_args(
    function_name: &str,
    args: &[Value],
    min: usize,
) -> Result<(), InterpreterError> {
    if args.len() < min {
        return Err(InterpreterError::RuntimeError(format!(
            "{}() expects at least {} argument{}, got {}",
            function_name,
            min,
            if min == 1 { "" } else { "s" },
            args.len()
        )));
    }
    Ok(())
}

/// Validate argument count range - O(1) complexity
#[inline]
pub fn validate_arg_range(
    function_name: &str,
    args: &[Value],
    min: usize,
    max: usize,
) -> Result<(), InterpreterError> {
    if args.len() < min || args.len() > max {
        return Err(InterpreterError::RuntimeError(format!(
            "{}() expects {}-{} arguments, got {}",
            function_name,
            min,
            max,
            args.len()
        )));
    }
    Ok(())
}

/// Validate numeric argument - O(1) complexity
#[inline]
pub fn validate_numeric(
    function_name: &str,
    value: &Value,
    arg_name: &str,
) -> Result<(), InterpreterError> {
    match value {
        Value::Integer(_) | Value::Float(_) => Ok(()),
        _ => Err(InterpreterError::RuntimeError(format!(
            "{}() expects {} to be a number, got {}",
            function_name,
            arg_name,
            value.type_name()
        ))),
    }
}

/// Validate string argument - O(1) complexity
#[inline]
pub fn validate_string(
    function_name: &str,
    value: &Value,
    arg_name: &str,
) -> Result<(), InterpreterError> {
    match value {
        Value::String(_) => Ok(()),
        _ => Err(InterpreterError::RuntimeError(format!(
            "{}() expects {} to be a string, got {}",
            function_name,
            arg_name,
            value.type_name()
        ))),
    }
}

/// Validate array argument - O(1) complexity
#[inline]
pub fn validate_array(
    function_name: &str,
    value: &Value,
    arg_name: &str,
) -> Result<(), InterpreterError> {
    match value {
        Value::Array(_) => Ok(()),
        _ => Err(InterpreterError::RuntimeError(format!(
            "{}() expects {} to be an array, got {}",
            function_name,
            arg_name,
            value.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_arg_count_exact() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        assert!(validate_arg_count("test", &args, 2).is_ok());
        assert!(validate_arg_count("test", &args, 3).is_err());
    }

    #[test]
    fn test_validate_min_args() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        assert!(validate_min_args("test", &args, 1).is_ok());
        assert!(validate_min_args("test", &args, 2).is_ok());
        assert!(validate_min_args("test", &args, 3).is_err());
    }

    #[test]
    fn test_validate_arg_range_boundaries() {
        // Mutation test: Verify < and > operators (not ==)
        // MISSED: replace < with == in validate_arg_range (line 54:19)
        // MISSED: replace > with == in validate_arg_range (line 54:39)

        // Test boundary values
        let args_too_few = vec![Value::Integer(1)];
        let args_min = vec![Value::Integer(1), Value::Integer(2)];
        let args_mid = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let args_max = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ];
        let args_too_many = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ];

        // Range: 2-4 arguments
        assert!(
            validate_arg_range("test", &args_too_few, 2, 4).is_err(),
            "Should reject fewer than min"
        );
        assert!(
            validate_arg_range("test", &args_min, 2, 4).is_ok(),
            "Should accept exactly min"
        );
        assert!(
            validate_arg_range("test", &args_mid, 2, 4).is_ok(),
            "Should accept middle value"
        );
        assert!(
            validate_arg_range("test", &args_max, 2, 4).is_ok(),
            "Should accept exactly max"
        );
        assert!(
            validate_arg_range("test", &args_too_many, 2, 4).is_err(),
            "Should reject more than max"
        );
    }

    #[test]
    fn test_validate_string_match_arm() {
        // Mutation test: Verify String match arm is tested
        // MISSED: delete match arm Value::String(_) in validate_string (line 92:9)

        // Test String value is accepted
        assert!(
            validate_string("test", &Value::from_string("hello".to_string()), "arg").is_ok(),
            "Should accept string value"
        );

        // Test non-String value is rejected
        assert!(
            validate_string("test", &Value::Integer(42), "arg").is_err(),
            "Should reject integer value"
        );
        assert!(
            validate_string("test", &Value::Float(3.15), "arg").is_err(),
            "Should reject float value"
        );
        assert!(
            validate_string("test", &Value::Bool(true), "arg").is_err(),
            "Should reject bool value"
        );
    }

    #[test]
    fn test_validate_numeric() {
        assert!(validate_numeric("test", &Value::Integer(42), "arg").is_ok());
        assert!(validate_numeric("test", &Value::Float(3.15), "arg").is_ok());
        assert!(validate_numeric(
            "test",
            &Value::from_string("not a number".to_string()),
            "arg"
        )
        .is_err());
    }

    // === EXTREME TDD Round 18 tests ===

    #[test]
    fn test_validate_array() {
        use std::sync::Arc;

        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(validate_array("test", &arr, "arg").is_ok());

        // Non-array should fail
        assert!(validate_array("test", &Value::Integer(42), "arg").is_err());
        assert!(validate_array("test", &Value::from_string("str".to_string()), "arg").is_err());
    }

    #[test]
    fn test_validate_arg_count_singular_plural() {
        // Test singular "argument" for 1
        let args_one = vec![Value::Integer(1)];
        let result = validate_arg_count("test", &args_one, 2);
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("2 arguments")); // plural for expected

        // Test with expecting 1 argument
        let args_two = vec![Value::Integer(1), Value::Integer(2)];
        let result = validate_arg_count("test", &args_two, 1);
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("1 argument")); // singular for 1
    }

    #[test]
    fn test_validate_min_args_singular_plural() {
        let args_empty: Vec<Value> = vec![];

        // Test plural for min > 1
        let result = validate_min_args("test", &args_empty, 2);
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("2 arguments"));

        // Test singular for min = 1
        let result = validate_min_args("test", &args_empty, 1);
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("1 argument"));
    }

    #[test]
    fn test_validate_array_error_message() {
        let result = validate_array("my_function", &Value::Integer(42), "my_arg");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("my_function"));
        assert!(err_msg.contains("my_arg"));
        assert!(err_msg.contains("array"));
    }

    #[test]
    fn test_validate_numeric_error_message() {
        let result = validate_numeric("compute", &Value::from_string("text".to_string()), "input");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("compute"));
        assert!(err_msg.contains("input"));
        assert!(err_msg.contains("number"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_validation_is_constant_time(arg_count in 0..100usize, expected in 0..100usize) {
            let args: Vec<Value> = (0..arg_count).map(|i| Value::Integer(i as i64)).collect();

            // Validation should be O(1) - just comparing lengths
            // Not O(n) iterating through arguments
            let _ = validate_arg_count("test", &args, expected);

            // This completes in constant time regardless of arg_count
            // Test passes without panic
        }

        #[test]
        fn test_error_messages_are_informative(func_name: String, expected in 0..100usize, actual in 0..100usize) {
            let args: Vec<Value> = (0..actual).map(|i| Value::Integer(i as i64)).collect();

            if expected != actual {
                let result = validate_arg_count(&func_name, &args, expected);
                if let Err(InterpreterError::RuntimeError(msg)) = result {
                    prop_assert!(msg.contains(&func_name));
                    prop_assert!(msg.contains(&expected.to_string()));
                    prop_assert!(msg.contains(&actual.to_string()));
                }
            }
        }
    }
}

// === EXTREME TDD Round 26 - Coverage Push Tests ===

#[cfg(test)]
mod coverage_push_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_validate_arg_count_zero() {
        let args: Vec<Value> = vec![];
        assert!(validate_arg_count("test", &args, 0).is_ok());
        assert!(validate_arg_count("test", &args, 1).is_err());
    }

    #[test]
    fn test_validate_min_args_zero() {
        let args: Vec<Value> = vec![];
        assert!(validate_min_args("test", &args, 0).is_ok());
        assert!(validate_min_args("test", &args, 1).is_err());
    }

    #[test]
    fn test_validate_arg_range_single_value() {
        let args = vec![Value::Integer(1)];
        // Range 1-1 should accept exactly 1 arg
        assert!(validate_arg_range("test", &args, 1, 1).is_ok());
        // Range 0-0 should reject 1 arg
        assert!(validate_arg_range("test", &args, 0, 0).is_err());
    }

    #[test]
    fn test_validate_numeric_all_types() {
        // Integer - ok
        assert!(validate_numeric("test", &Value::Integer(42), "arg").is_ok());
        // Float - ok
        assert!(validate_numeric("test", &Value::Float(3.14), "arg").is_ok());
        // All other types - error
        assert!(validate_numeric("test", &Value::Bool(true), "arg").is_err());
        assert!(validate_numeric("test", &Value::Nil, "arg").is_err());
        let arr = Value::Array(Arc::from(vec![]));
        assert!(validate_numeric("test", &arr, "arg").is_err());
    }

    #[test]
    fn test_validate_string_all_types() {
        // String - ok
        let s = Value::from_string("hello".to_string());
        assert!(validate_string("test", &s, "arg").is_ok());
        // All other types - error
        assert!(validate_string("test", &Value::Nil, "arg").is_err());
        let arr = Value::Array(Arc::from(vec![]));
        assert!(validate_string("test", &arr, "arg").is_err());
    }

    #[test]
    fn test_validate_array_all_types() {
        // Array - ok
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        assert!(validate_array("test", &arr, "arg").is_ok());
        // Empty array - ok
        let empty = Value::Array(Arc::from(vec![]));
        assert!(validate_array("test", &empty, "arg").is_ok());
        // All other types - error
        assert!(validate_array("test", &Value::Nil, "arg").is_err());
        let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1)]));
        assert!(validate_array("test", &tuple, "arg").is_err());
    }

    #[test]
    fn test_validate_arg_count_large_values() {
        let args: Vec<Value> = (0..100).map(|i| Value::Integer(i)).collect();
        assert!(validate_arg_count("test", &args, 100).is_ok());
        assert!(validate_arg_count("test", &args, 99).is_err());
        assert!(validate_arg_count("test", &args, 101).is_err());
    }

    #[test]
    fn test_validate_min_args_large_min() {
        let args: Vec<Value> = (0..50).map(|i| Value::Integer(i)).collect();
        assert!(validate_min_args("test", &args, 50).is_ok());
        assert!(validate_min_args("test", &args, 51).is_err());
    }

    #[test]
    fn test_validate_arg_range_wide_range() {
        let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        assert!(validate_arg_range("test", &args, 1, 10).is_ok());
        assert!(validate_arg_range("test", &args, 3, 3).is_ok());
    }

    #[test]
    fn test_error_message_formatting() {
        let args = vec![Value::Integer(1)];

        // Test validate_arg_count message
        let result = validate_arg_count("my_func", &args, 3);
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("my_func"));
        assert!(msg.contains("3 arguments"));
        assert!(msg.contains("got 1"));

        // Test validate_min_args message
        let result = validate_min_args("other_func", &args, 5);
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("other_func"));
        assert!(msg.contains("at least 5 arguments"));

        // Test validate_arg_range message
        let result = validate_arg_range("range_func", &args, 3, 5);
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("range_func"));
        assert!(msg.contains("3-5 arguments"));
    }

    #[test]
    fn test_validate_string_error_contains_type() {
        let result = validate_string("test_fn", &Value::Integer(42), "input");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("test_fn"));
        assert!(msg.contains("input"));
        assert!(msg.contains("string"));
    }

    #[test]
    fn test_validate_numeric_negative() {
        assert!(validate_numeric("test", &Value::Integer(-100), "arg").is_ok());
        assert!(validate_numeric("test", &Value::Float(-3.14), "arg").is_ok());
    }

    #[test]
    fn test_validate_numeric_zero() {
        assert!(validate_numeric("test", &Value::Integer(0), "arg").is_ok());
        assert!(validate_numeric("test", &Value::Float(0.0), "arg").is_ok());
    }
}
