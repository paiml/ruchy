//! Value Formatting Utilities - Extracted for 100% Coverage
//!
//! Handles string formatting with placeholders and format specifiers.

use crate::runtime::Value;

/// Format string with placeholder replacement
/// Handles `{}` and `{:?}` placeholders
pub fn format_string_with_values(format_str: &str, values: &[Value]) -> String {
    let mut result = String::new();
    let mut chars = format_str.chars().peekable();
    let mut value_index = 0;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&':') {
                chars.next();
                if chars.peek() == Some(&'?') {
                    chars.next();
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        if value_index < values.len() {
                            result.push_str(&format!("{:?}", values[value_index]));
                            value_index += 1;
                        } else {
                            result.push_str("{:?}");
                        }
                    } else {
                        result.push_str("{:?");
                    }
                } else {
                    result.push_str("{:");
                }
            } else if chars.peek() == Some(&'}') {
                chars.next();
                if value_index < values.len() {
                    // Extract raw string without Display quotes
                    let display_val = match &values[value_index] {
                        Value::String(ref s) => s.as_ref().to_string(),
                        _ => values[value_index].to_string(),
                    };
                    result.push_str(&display_val);
                    value_index += 1;
                } else {
                    result.push_str("{}");
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Format a value with a format specifier like `:.2` for floats
pub fn format_value_with_spec(value: &Value, spec: &str) -> String {
    // Parse format specifier (e.g., ":.2" -> precision 2)
    if let Some(stripped) = spec.strip_prefix(":.") {
        if let Ok(precision) = stripped.parse::<usize>() {
            match value {
                Value::Float(f) => return format!("{:.precision$}", f, precision = precision),
                Value::Integer(i) => {
                    return format!("{:.precision$}", *i as f64, precision = precision)
                }
                _ => {}
            }
        }
    }
    // Default formatting if spec doesn't match or isn't supported
    value.to_string()
}

/// Format value for debug output
pub fn format_value_debug(value: &Value) -> String {
    format!("{:?}", value)
}

/// Format value for display output (no extra quotes for strings)
pub fn format_value_display(value: &Value) -> String {
    match value {
        Value::String(s) => s.as_ref().to_string(),
        _ => value.to_string(),
    }
}

// ============================================================================
// Tests - 100% Coverage Target
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // format_string_with_values tests
    #[test]
    fn test_format_simple_placeholder() {
        let result = format_string_with_values("Hello {}", &[Value::from_string("World".to_string())]);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_format_multiple_placeholders() {
        let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = format_string_with_values("{} + {} = {}", &values);
        assert_eq!(result, "1 + 2 = 3");
    }

    #[test]
    fn test_format_debug_placeholder() {
        let result = format_string_with_values("{:?}", &[Value::from_string("test".to_string())]);
        assert!(result.contains("String"));
    }

    #[test]
    fn test_format_no_placeholders() {
        let result = format_string_with_values("Hello World", &[]);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_format_more_placeholders_than_values() {
        let result = format_string_with_values("{} {}", &[Value::Integer(1)]);
        assert_eq!(result, "1 {}");
    }

    #[test]
    fn test_format_more_values_than_placeholders() {
        let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = format_string_with_values("{}", &values);
        assert_eq!(result, "1");
    }

    #[test]
    fn test_format_incomplete_debug_placeholder() {
        let result = format_string_with_values("{:?x", &[Value::Integer(1)]);
        assert_eq!(result, "{:?x");
    }

    #[test]
    fn test_format_incomplete_colon_placeholder() {
        let result = format_string_with_values("{:x", &[Value::Integer(1)]);
        assert_eq!(result, "{:x");
    }

    #[test]
    fn test_format_lone_open_brace() {
        let result = format_string_with_values("test { more", &[]);
        assert_eq!(result, "test { more");
    }

    #[test]
    fn test_format_integer_value() {
        let result = format_string_with_values("{}", &[Value::Integer(42)]);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_float_value() {
        let result = format_string_with_values("{}", &[Value::Float(3.14)]);
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_format_bool_value() {
        let result = format_string_with_values("{}", &[Value::Bool(true)]);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_format_nil_value() {
        let result = format_string_with_values("{}", &[Value::Nil]);
        assert_eq!(result, "nil");
    }

    #[test]
    fn test_format_debug_missing_value() {
        let result = format_string_with_values("{:?}", &[]);
        assert_eq!(result, "{:?}");
    }

    // format_value_with_spec tests
    #[test]
    fn test_spec_float_precision_2() {
        let result = format_value_with_spec(&Value::Float(3.14159), ":.2");
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_spec_float_precision_4() {
        let result = format_value_with_spec(&Value::Float(2.71828), ":.4");
        assert_eq!(result, "2.7183");
    }

    #[test]
    fn test_spec_integer_as_float() {
        let result = format_value_with_spec(&Value::Integer(42), ":.2");
        assert_eq!(result, "42.00");
    }

    #[test]
    fn test_spec_invalid_precision() {
        let result = format_value_with_spec(&Value::Float(3.14), ":.abc");
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_spec_no_precision() {
        let result = format_value_with_spec(&Value::Float(3.14), ":.");
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_spec_empty() {
        let result = format_value_with_spec(&Value::Float(3.14), "");
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_spec_non_numeric_value() {
        let result = format_value_with_spec(&Value::from_string("hello".to_string()), ":.2");
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_spec_bool_value() {
        let result = format_value_with_spec(&Value::Bool(true), ":.2");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_spec_precision_0() {
        let result = format_value_with_spec(&Value::Float(3.14159), ":.0");
        assert_eq!(result, "3");
    }

    // format_value_debug tests
    #[test]
    fn test_debug_integer() {
        let result = format_value_debug(&Value::Integer(42));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_debug_string() {
        let result = format_value_debug(&Value::from_string("test".to_string()));
        assert!(result.contains("String"));
    }

    #[test]
    fn test_debug_array() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)].into_boxed_slice()));
        let result = format_value_debug(&arr);
        assert!(result.contains("Array"));
    }

    // format_value_display tests
    #[test]
    fn test_display_string_no_quotes() {
        let result = format_value_display(&Value::from_string("hello".to_string()));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_display_integer() {
        let result = format_value_display(&Value::Integer(42));
        assert_eq!(result, "42");
    }

    #[test]
    fn test_display_float() {
        let result = format_value_display(&Value::Float(3.14));
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_display_bool() {
        let result = format_value_display(&Value::Bool(false));
        assert_eq!(result, "false");
    }

    #[test]
    fn test_display_nil() {
        let result = format_value_display(&Value::Nil);
        assert_eq!(result, "nil");
    }

    // Edge cases
    #[test]
    fn test_format_empty_string() {
        let result = format_string_with_values("", &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_only_placeholder() {
        let result = format_string_with_values("{}", &[Value::Integer(1)]);
        assert_eq!(result, "1");
    }

    #[test]
    fn test_format_adjacent_placeholders() {
        let result = format_string_with_values("{}{}", &[Value::Integer(1), Value::Integer(2)]);
        assert_eq!(result, "12");
    }

    #[test]
    fn test_spec_large_precision() {
        let result = format_value_with_spec(&Value::Float(1.0), ":.10");
        assert_eq!(result, "1.0000000000");
    }
}
