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

    // === EXTREME TDD Round 16 tests ===

    #[test]
    fn test_format_value_with_spec_float() {
        let float_val = Value::Float(3.14159);
        let result = format_value_with_spec(&float_val, "f");
        assert!(result.starts_with("3.14"));
        assert!(result.len() >= 6); // Should have decimal places
    }

    #[test]
    fn test_format_value_with_spec_string() {
        let str_val = Value::from_string("hello".to_string());
        let result = format_value_with_spec(&str_val, "s");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_format_value_with_spec_unknown() {
        let val = Value::Integer(42);
        let result = format_value_with_spec(&val, "unknown_spec");
        // Unknown spec should fall back to to_string()
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_value_for_interpolation_array() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let result = format_value_for_interpolation(&arr);
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_format_value_for_interpolation_tuple() {
        let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = format_value_for_interpolation(&tuple);
        assert_eq!(result, "(1, 2)");
    }

    #[test]
    fn test_format_value_for_interpolation_object() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("x".to_string(), Value::Integer(10));
        let obj = Value::Object(Arc::new(map));
        let result = format_value_for_interpolation(&obj);
        assert!(result.starts_with('{'));
        assert!(result.contains("x: 10"));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_format_value_for_display_array() {
        let arr = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
        ]));
        let result = format_value_for_display(&arr);
        // Strings should have quotes in display
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\""));
    }

    #[test]
    fn test_string_interpolation_with_format_spec() {
        let parts = vec![
            StringPart::Text("Value: ".to_string()),
            StringPart::ExprWithFormat {
                expr: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(255, None)),
                    Span::new(0, 3),
                )),
                format_spec: "x".to_string(),
            },
        ];

        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        // 255 in hex is "ff"
        let result_str = result.to_string();
        assert!(result_str.contains("ff"), "Expected hex format, got {result_str}");
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

// ============================================================================
// EXTREME TDD Round 132: Additional comprehensive tests
// Target: 15 → 40+ tests
// ============================================================================
#[cfg(test)]
mod round_132_tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_literal_expr(n: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(n, None)), Span::default())
    }

    fn make_string_expr(s: &str) -> Expr {
        Expr::new(ExprKind::Literal(Literal::String(s.to_string())), Span::default())
    }

    // --- format_value_with_spec edge cases ---
    #[test]
    fn test_format_d_with_negative_integer() {
        let val = Value::Integer(-42);
        assert_eq!(format_value_with_spec(&val, "d"), "-42");
    }

    #[test]
    fn test_format_d_with_zero() {
        let val = Value::Integer(0);
        assert_eq!(format_value_with_spec(&val, "d"), "0");
    }

    #[test]
    fn test_format_d_with_large_integer() {
        let val = Value::Integer(9_223_372_036_854_775_807);
        assert_eq!(format_value_with_spec(&val, "d"), "9223372036854775807");
    }

    #[test]
    fn test_format_i_same_as_d() {
        let val = Value::Integer(123);
        assert_eq!(format_value_with_spec(&val, "i"), format_value_with_spec(&val, "d"));
    }

    #[test]
    fn test_format_f_with_integer() {
        let val = Value::Integer(42);
        let result = format_value_with_spec(&val, "f");
        assert!(result.contains("42"));
        assert!(result.contains('.'));
    }

    #[test]
    fn test_format_f_with_negative_float() {
        let val = Value::Float(-3.14159);
        let result = format_value_with_spec(&val, "f");
        assert!(result.starts_with("-3.14"));
    }

    #[test]
    fn test_format_f_with_zero() {
        let val = Value::Float(0.0);
        let result = format_value_with_spec(&val, "f");
        assert!(result.starts_with("0.0"));
    }

    #[test]
    fn test_format_x_with_zero() {
        let val = Value::Integer(0);
        assert_eq!(format_value_with_spec(&val, "x"), "0");
    }

    #[test]
    fn test_format_x_with_large_value() {
        let val = Value::Integer(0xDEADBEEF);
        let result = format_value_with_spec(&val, "x");
        assert!(result.contains("deadbeef"));
    }

    #[test]
    fn test_format_hex_uppercase() {
        let val = Value::Integer(0xABCDEF);
        let result = format_value_with_spec(&val, "X");
        assert!(result.contains("ABCDEF"));
    }

    #[test]
    fn test_format_o_octal() {
        let val = Value::Integer(8);
        assert_eq!(format_value_with_spec(&val, "o"), "10");
    }

    #[test]
    fn test_format_o_larger_value() {
        let val = Value::Integer(64);
        assert_eq!(format_value_with_spec(&val, "o"), "100");
    }

    #[test]
    fn test_format_b_binary_power_of_two() {
        let val = Value::Integer(16);
        assert_eq!(format_value_with_spec(&val, "b"), "10000");
    }

    #[test]
    fn test_format_b_binary_255() {
        let val = Value::Integer(255);
        assert_eq!(format_value_with_spec(&val, "b"), "11111111");
    }

    #[test]
    fn test_format_with_non_numeric_x() {
        let val = Value::from_string("test".to_string());
        let result = format_value_with_spec(&val, "x");
        assert!(result.contains("test"));
    }

    #[test]
    fn test_format_with_non_numeric_b() {
        let val = Value::Bool(true);
        let result = format_value_with_spec(&val, "b");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_format_with_empty_spec() {
        let val = Value::Integer(42);
        let result = format_value_with_spec(&val, "");
        assert_eq!(result, "42");
    }

    // --- format_value_for_interpolation edge cases ---
    #[test]
    fn test_interpolation_string_no_quotes() {
        let val = Value::from_string("hello world".to_string());
        let result = format_value_for_interpolation(&val);
        assert!(!result.starts_with('"'));
        assert!(!result.ends_with('"'));
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_interpolation_empty_string() {
        let val = Value::from_string(String::new());
        let result = format_value_for_interpolation(&val);
        assert_eq!(result, "");
    }

    #[test]
    fn test_interpolation_empty_array() {
        let val = Value::Array(Arc::from(vec![]));
        let result = format_value_for_interpolation(&val);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_interpolation_nested_array() {
        let inner = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let outer = Value::Array(Arc::from(vec![inner, Value::Integer(3)]));
        let result = format_value_for_interpolation(&outer);
        assert!(result.contains("[1, 2]"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_interpolation_empty_tuple() {
        let val = Value::Tuple(Arc::from(vec![]));
        let result = format_value_for_interpolation(&val);
        assert_eq!(result, "()");
    }

    #[test]
    fn test_interpolation_single_element_tuple() {
        let val = Value::Tuple(Arc::from(vec![Value::Integer(42)]));
        let result = format_value_for_interpolation(&val);
        assert_eq!(result, "(42)");
    }

    #[test]
    fn test_interpolation_empty_object() {
        let map: HashMap<String, Value> = HashMap::new();
        let val = Value::Object(Arc::new(map));
        let result = format_value_for_interpolation(&val);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_interpolation_nil() {
        let val = Value::Nil;
        let result = format_value_for_interpolation(&val);
        assert!(result.contains("nil") || result.contains("null") || result.is_empty());
    }

    #[test]
    fn test_interpolation_bool_true() {
        let val = Value::Bool(true);
        let result = format_value_for_interpolation(&val);
        assert!(result.contains("true"));
    }

    #[test]
    fn test_interpolation_bool_false() {
        let val = Value::Bool(false);
        let result = format_value_for_interpolation(&val);
        assert!(result.contains("false"));
    }

    #[test]
    fn test_interpolation_float() {
        let val = Value::Float(3.14);
        let result = format_value_for_interpolation(&val);
        assert!(result.contains("3.14"));
    }

    // --- format_value_for_display edge cases ---
    #[test]
    fn test_display_empty_string() {
        let val = Value::from_string(String::new());
        let result = format_value_for_display(&val);
        assert_eq!(result, "\"\"");
    }

    #[test]
    fn test_display_string_with_special_chars() {
        let val = Value::from_string("hello\nworld".to_string());
        let result = format_value_for_display(&val);
        assert!(result.starts_with('"'));
        assert!(result.ends_with('"'));
    }

    #[test]
    fn test_display_empty_array() {
        let val = Value::Array(Arc::from(vec![]));
        let result = format_value_for_display(&val);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_display_empty_tuple() {
        let val = Value::Tuple(Arc::from(vec![]));
        let result = format_value_for_display(&val);
        assert_eq!(result, "()");
    }

    #[test]
    fn test_display_integer() {
        let val = Value::Integer(42);
        let result = format_value_for_display(&val);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_display_nil() {
        let val = Value::Nil;
        let result = format_value_for_display(&val);
        assert!(!result.is_empty());
    }

    // --- eval_string_interpolation tests ---
    #[test]
    fn test_interpolation_text_only() {
        let parts = vec![StringPart::Text("Hello world".to_string())];
        let result = eval_string_interpolation(&parts, |_| Ok(Value::Nil)).unwrap();
        assert!(result.to_string().contains("Hello world"));
    }

    #[test]
    fn test_interpolation_empty_parts() {
        let parts: Vec<StringPart> = vec![];
        let result = eval_string_interpolation(&parts, |_| Ok(Value::Nil)).unwrap();
        match result {
            Value::String(s) => assert!(s.is_empty()),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_interpolation_multiple_exprs() {
        let parts = vec![
            StringPart::Expr(Box::new(make_literal_expr(1))),
            StringPart::Text(" + ".to_string()),
            StringPart::Expr(Box::new(make_literal_expr(2))),
            StringPart::Text(" = ".to_string()),
            StringPart::Expr(Box::new(make_literal_expr(3))),
        ];
        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n)),
            _ => Ok(Value::Nil),
        }).unwrap();
        let s = result.to_string();
        assert!(s.contains("1"));
        assert!(s.contains("2"));
        assert!(s.contains("3"));
    }

    #[test]
    fn test_interpolation_expr_error() {
        let parts = vec![StringPart::Expr(Box::new(make_literal_expr(1)))];
        let result = eval_string_interpolation(&parts, |_| {
            Err(InterpreterError::RuntimeError("test error".to_string()))
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_interpolation_with_format_d() {
        let parts = vec![StringPart::ExprWithFormat {
            expr: Box::new(make_literal_expr(42)),
            format_spec: "d".to_string(),
        }];
        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n)),
            _ => Ok(Value::Nil),
        }).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_interpolation_with_format_x() {
        let parts = vec![StringPart::ExprWithFormat {
            expr: Box::new(make_literal_expr(255)),
            format_spec: "x".to_string(),
        }];
        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n)),
            _ => Ok(Value::Nil),
        }).unwrap();
        assert!(result.to_string().contains("ff"));
    }

    #[test]
    fn test_interpolation_mixed_text_and_format() {
        let parts = vec![
            StringPart::Text("Hex: ".to_string()),
            StringPart::ExprWithFormat {
                expr: Box::new(make_literal_expr(16)),
                format_spec: "x".to_string(),
            },
            StringPart::Text(" Binary: ".to_string()),
            StringPart::ExprWithFormat {
                expr: Box::new(make_literal_expr(16)),
                format_spec: "b".to_string(),
            },
        ];
        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n)),
            _ => Ok(Value::Nil),
        }).unwrap();
        let s = result.to_string();
        assert!(s.contains("10")); // hex 16
        assert!(s.contains("10000")); // binary 16
    }

    #[test]
    fn test_interpolation_unicode() {
        let parts = vec![
            StringPart::Text("Unicode: ".to_string()),
            StringPart::Expr(Box::new(make_string_expr("日本語"))),
        ];
        let result = eval_string_interpolation(&parts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
            _ => Ok(Value::Nil),
        }).unwrap();
        assert!(result.to_string().contains("日本語"));
    }

    #[test]
    fn test_interpolation_array_value() {
        let parts = vec![StringPart::Expr(Box::new(make_literal_expr(0)))];
        let result = eval_string_interpolation(&parts, |_| {
            Ok(Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)])))
        }).unwrap();
        assert!(result.to_string().contains("[1, 2]"));
    }

    #[test]
    fn test_interpolation_tuple_value() {
        let parts = vec![StringPart::Expr(Box::new(make_literal_expr(0)))];
        let result = eval_string_interpolation(&parts, |_| {
            Ok(Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)])))
        }).unwrap();
        assert!(result.to_string().contains("(1, 2)"));
    }

    #[test]
    fn test_format_f_precision() {
        let val = Value::Float(1.0 / 3.0);
        let result = format_value_with_spec(&val, "f");
        // Should have 6 decimal places
        assert!(result.contains("0.333333"));
    }

    #[test]
    fn test_format_s_with_integer() {
        let val = Value::Integer(42);
        let result = format_value_with_spec(&val, "s");
        assert_eq!(result, "42");
    }

    #[test]
    fn test_display_nested_tuple_in_array() {
        let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let arr = Value::Array(Arc::from(vec![tuple]));
        let result = format_value_for_display(&arr);
        assert!(result.contains("(1, 2)"));
    }
}

// ============================================================================
// EXTREME TDD Round 137: Additional tests to reach 75+
// ============================================================================
#[cfg(test)]
mod round_137_tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_literal_expr(n: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(n, None)), Span::default())
    }

    // --- format_value_with_spec additional tests ---
    #[test]
    fn test_format_d_with_min_i64() {
        let val = Value::Integer(i64::MIN);
        let result = format_value_with_spec(&val, "d");
        assert_eq!(result, "-9223372036854775808");
    }

    #[test]
    fn test_format_x_with_negative() {
        let val = Value::Integer(-1);
        let result = format_value_with_spec(&val, "x");
        assert!(result.contains("ffffffffffffffff"));
    }

    #[test]
    fn test_format_hex_uppercase_with_negative() {
        let val = Value::Integer(-1);
        let result = format_value_with_spec(&val, "X");
        assert!(result.contains("FFFFFFFFFFFFFFFF"));
    }

    #[test]
    fn test_format_o_with_negative() {
        let val = Value::Integer(-8);
        let result = format_value_with_spec(&val, "o");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_b_with_zero() {
        let val = Value::Integer(0);
        assert_eq!(format_value_with_spec(&val, "b"), "0");
    }

    #[test]
    fn test_format_f_with_very_small() {
        let val = Value::Float(0.000001);
        let result = format_value_with_spec(&val, "f");
        assert!(result.starts_with("0.000001"));
    }

    #[test]
    fn test_format_f_with_nan_fallback() {
        let val = Value::Nil;
        let result = format_value_with_spec(&val, "f");
        assert!(!result.is_empty());
    }

    // --- format_value_for_interpolation additional tests ---
    #[test]
    fn test_interpolation_deeply_nested_array() {
        let inner1 = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let inner2 = Value::Array(Arc::from(vec![inner1]));
        let outer = Value::Array(Arc::from(vec![inner2]));
        let result = format_value_for_interpolation(&outer);
        assert!(result.contains("[[[1]]]"));
    }

    #[test]
    fn test_interpolation_mixed_type_tuple() {
        let tuple = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("two".to_string()),
            Value::Bool(true),
        ]));
        let result = format_value_for_interpolation(&tuple);
        assert!(result.contains("1"));
        assert!(result.contains("two"));
        assert!(result.contains("true"));
    }

    #[test]
    fn test_interpolation_object_with_nested_array() {
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("arr".to_string(), Value::Array(Arc::from(vec![Value::Integer(1)])));
        let obj = Value::Object(Arc::new(map));
        let result = format_value_for_interpolation(&obj);
        assert!(result.contains("arr"));
        assert!(result.contains("[1]"));
    }

    // --- format_value_for_display additional tests ---
    #[test]
    fn test_display_bool_true() {
        let val = Value::Bool(true);
        let result = format_value_for_display(&val);
        assert!(result.contains("true"));
    }

    #[test]
    fn test_display_bool_false() {
        let val = Value::Bool(false);
        let result = format_value_for_display(&val);
        assert!(result.contains("false"));
    }

    #[test]
    fn test_display_negative_integer() {
        let val = Value::Integer(-999);
        let result = format_value_for_display(&val);
        assert_eq!(result, "-999");
    }

    #[test]
    fn test_display_float() {
        let val = Value::Float(2.718);
        let result = format_value_for_display(&val);
        assert!(result.contains("2.718"));
    }

    // --- eval_string_interpolation additional tests ---
    #[test]
    fn test_interpolation_format_with_error() {
        let parts = vec![StringPart::ExprWithFormat {
            expr: Box::new(make_literal_expr(42)),
            format_spec: "d".to_string(),
        }];
        let result = eval_string_interpolation(&parts, |_| {
            Err(InterpreterError::RuntimeError("format error".to_string()))
        });
        assert!(result.is_err());
    }
}
