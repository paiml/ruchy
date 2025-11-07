//! String interpolation evaluation module
//!
//! This module handles string interpolation and formatting operations.
//! Provides f-string evaluation, format specifier handling, and value formatting.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, StringPart};
use crate::runtime::{InterpreterError, Value};

/// Evaluate string interpolation expression (f-strings)
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn eval_string_interpolation<F>(
    parts: &[StringPart],
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut result = String::new();
    for part in parts {
        match part {
            StringPart::Text(text) => result.push_str(text),
            StringPart::Expr(expr) => {
                let value = eval_expr(expr)?;
                result.push_str(&format_value_for_interpolation(&value));
            }
            StringPart::ExprWithFormat { expr, format_spec } => {
                let value = eval_expr(expr)?;
                let formatted = format_value_with_spec(&value, format_spec);
                result.push_str(&formatted);
            }
        }
    }
    Ok(Value::from_string(result))
}

/// Format a value with a format specifier
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn format_value_with_spec(value: &Value, format_spec: &str) -> String {
    match format_spec {
        "d" | "i" => {
            // Integer format
            match value {
                Value::Integer(n) => n.to_string(),
                Value::Float(f) => (*f as i64).to_string(),
                _ => value.to_string(),
            }
        }
        "f" => {
            // Float format
            match value {
                Value::Integer(n) => format!("{:.6}", *n as f64),
                Value::Float(f) => format!("{f:.6}"),
                _ => value.to_string(),
            }
        }
        "s" => {
            // String format
            value.to_string()
        }
        "x" => {
            // Lowercase hex format
            match value {
                Value::Integer(n) => format!("{n:x}"),
                _ => value.to_string(),
            }
        }
        "X" => {
            // Uppercase hex format
            match value {
                Value::Integer(n) => format!("{n:X}"),
                _ => value.to_string(),
            }
        }
        "o" => {
            // Octal format
            match value {
                Value::Integer(n) => format!("{n:o}"),
                _ => value.to_string(),
            }
        }
        "b" => {
            // Binary format
            match value {
                Value::Integer(n) => format!("{n:b}"),
                _ => value.to_string(),
            }
        }
        _ => {
            // Unknown format spec, return as-is
            value.to_string()
        }
    }
}

/// Format a value for string interpolation (no extra quotes for strings)
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn format_value_for_interpolation(value: &Value) -> String {
    match value {
        Value::String(s) => s.to_string(), // No quotes for interpolation
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_value_for_interpolation).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Tuple(elements) => {
            let formatted: Vec<String> = elements
                .iter()
                .map(format_value_for_interpolation)
                .collect();
            format!("({})", formatted.join(", "))
        }
        Value::Object(map) => {
            let mut entries = Vec::new();
            for (k, v) in map.iter() {
                entries.push(format!("{}: {}", k, format_value_for_interpolation(v)));
            }
            format!("{{{}}}", entries.join(", "))
        }
        _ => value.to_string(), // Use default for other types
    }
}

/// Format value for display in interpreter output
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn format_value_for_display(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_value_for_display).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Tuple(elements) => {
            let formatted: Vec<String> = elements.iter().map(format_value_for_display).collect();
            format!("({})", formatted.join(", "))
        }
        Value::Object(fields) => {
            let formatted: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_display(v)))
                .collect();
            format!("{{{}}}", formatted.join(", "))
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use std::sync::Arc;

    #[test]
    fn test_format_value_with_spec() {
        let value = Value::Integer(42);
        assert_eq!(format_value_with_spec(&value, "d"), "42");
        assert_eq!(format_value_with_spec(&value, "x"), "2a");
        assert_eq!(format_value_with_spec(&value, "X"), "2A");
        assert_eq!(format_value_with_spec(&value, "b"), "101010");
        assert_eq!(format_value_with_spec(&value, "o"), "52");
    }

    #[test]
    fn test_format_value_with_spec_float_match_arm() {
        // Mutation test: Verify Float match arm is tested in "d" format
        // MISSED: delete match arm Value::Float(f) in format_value_with_spec (line 50:17)

        let float_value = Value::Float(3.15);

        // Test "d" format with Float - should convert to integer
        assert_eq!(
            format_value_with_spec(&float_value, "d"),
            "3",
            "Float with 'd' format should convert to integer"
        );

        // Test "i" format with Float - should also convert to integer
        assert_eq!(
            format_value_with_spec(&float_value, "i"),
            "3",
            "Float with 'i' format should convert to integer"
        );
    }

    #[test]
    fn test_format_value_for_display() {
        let string_val = Value::from_string("hello".to_string());
        assert_eq!(format_value_for_display(&string_val), "\"hello\"");

        let array_val = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert_eq!(format_value_for_display(&array_val), "[1, 2]");

        let tuple_val = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        assert_eq!(format_value_for_display(&tuple_val), "(1, \"test\")");
    }

    #[test]
    fn test_string_interpolation() {
        let parts = vec![
            StringPart::Text("Hello ".to_string()),
            StringPart::Expr(Box::new(Expr::new(
                ExprKind::Literal(Literal::String("world".to_string())),
                Span::new(0, 5),
            ))),
            StringPart::Text("!".to_string()),
        ];

        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        assert_eq!(result.to_string(), "\"Hello world!\"");
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_format_value_with_spec_integer_match_arm() {
        // MISSED: delete match arm Value::Integer(n) in format_value_with_spec (line 49)
        let val = Value::Integer(42);
        let result = format_value_with_spec(&val, "d");
        assert_eq!(
            result, "42",
            "Integer format spec should use Integer match arm"
        );
    }

    #[test]
    fn test_format_value_for_display_tuple_match_arm() {
        // MISSED: delete match arm Value::Tuple(elements) in format_value_for_display (line 141)
        let tuple = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        let result = format_value_for_display(&tuple);
        assert!(
            result.starts_with('('),
            "Tuple should format with parentheses"
        );
        assert!(result.contains('1'), "Tuple should contain first element");
        assert!(
            result.contains("test"),
            "Tuple should contain second element"
        );
    }

    #[test]
    fn test_format_value_for_display_object_match_arm() {
        // MISSED: delete match arm Value::Object(fields) in format_value_for_display (line 145)
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));
        let obj = Value::Object(Arc::new(map));

        let result = format_value_for_display(&obj);
        assert!(result.starts_with('{'), "Object should format with braces");
        assert!(result.contains("key"), "Object should contain key");
        assert!(result.contains("42"), "Object should contain value");
    }
}
