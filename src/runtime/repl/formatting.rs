//! REPL Output Formatting
//!
//! Handles formatting of values, errors, and AST for display in the REPL.

use crate::runtime::interpreter::Value;

/// Format a value for display in the REPL (complexity: 9)
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{s}\""),
        Value::Bool(b) => b.to_string(),
        Value::Nil => "nil".to_string(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Tuple(tuple) => {
            let elements: Vec<String> = tuple.iter().map(format_value).collect();
            format!("({})", elements.join(", "))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj.iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Range { start, end, inclusive } => {
            if *inclusive {
                format!("{}..={}", format_value(start), format_value(end))
            } else {
                format!("{}..{}", format_value(start), format_value(end))
            }
        }
        Value::EnumVariant { variant_name, data } => {
            match data {
                Some(values) => {
                    let formatted: Vec<String> = values.iter().map(format_value).collect();
                    format!("{}({})", variant_name, formatted.join(", "))
                }
                None => variant_name.clone(),
            }
        }
        Value::Closure { .. } => "<closure>".to_string(),
        Value::DataFrame { .. } => "<dataframe>".to_string(),
    }
}

/// Format an error for display in the REPL (complexity: 2)
pub fn format_error(error: &str) -> String {
    format!("Error: {error}")
}

/// Format AST for display in the REPL (complexity: 1)
pub fn format_ast(ast: &str) -> String {
    // Simplified AST formatting for now
    format!("AST: {ast}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn test_format_integer() {
        let value = Value::Integer(42);
        assert_eq!(format_value(&value), "42");
    }

    #[test]
    fn test_format_string() {
        let value = Value::String(Rc::new("hello".to_string()));
        assert_eq!(format_value(&value), "\"hello\"");
    }

    #[test]
    fn test_format_bool() {
        let value = Value::Bool(true);
        assert_eq!(format_value(&value), "true");
    }

    #[test]
    fn test_format_nil() {
        let value = Value::Nil;
        assert_eq!(format_value(&value), "nil");
    }

    #[test]
    fn test_format_array() {
        let value = Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        assert_eq!(format_value(&value), "[1, 2, 3]");
    }

    #[test]
    fn test_format_tuple() {
        let value = Value::Tuple(Rc::new(vec![
            Value::Integer(1),
            Value::String(Rc::new("test".to_string())),
        ]));
        assert_eq!(format_value(&value), "(1, \"test\")");
    }

    #[test]
    fn test_format_error() {
        assert_eq!(format_error("test error"), "Error: test error");
    }

    #[test]
    fn test_format_range_inclusive() {
        let value = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };
        assert_eq!(format_value(&value), "1..=10");
    }

    #[test]
    fn test_format_range_exclusive() {
        let value = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        assert_eq!(format_value(&value), "1..10");
    }
}