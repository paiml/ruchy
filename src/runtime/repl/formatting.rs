//! REPL Output Formatting
//!
//! Handles formatting of values, errors, and AST for display in the REPL.

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
    use crate::runtime::Value;
    use std::rc::Rc;

    #[test]
    fn test_format_integer() {
        let value = Value::Integer(42);
        assert_eq!(value.to_string(), "42");
    }

    #[test]
    fn test_format_string() {
        let value = Value::from_string("hello".to_string());
        assert_eq!(value.to_string(), "\"hello\"");
    }

    #[test]
    fn test_format_bool() {
        let value = Value::Bool(true);
        assert_eq!(value.to_string(), "true");
    }

    #[test]
    fn test_format_nil() {
        let value = Value::Nil;
        assert_eq!(value.to_string(), "nil");
    }

    #[test]
    fn test_format_array() {
        let value = Value::Array(Rc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        assert_eq!(value.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_format_tuple() {
        let value = Value::Tuple(Rc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        assert_eq!(value.to_string(), "(1, \"test\")");
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
        assert_eq!(value.to_string(), "1..=10");
    }

    #[test]
    fn test_format_range_exclusive() {
        let value = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        assert_eq!(value.to_string(), "1..10");
    }
}
