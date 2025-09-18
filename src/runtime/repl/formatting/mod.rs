//! Output Formatting - EXTREME Quality
//!
//! Handles pretty-printing with <10 complexity

use crate::runtime::value::Value;
use crate::frontend::ast::Expr;

/// Format values for display (complexity: 8)
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Unit => String::new(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e10 {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::List(items) => format_list(items),
        Value::Tuple(items) => format_tuple(items),
        Value::Object(fields) => format_object(fields),
        _ => format!("{:?}", value),
    }
}

/// Format a list (complexity: 4)
fn format_list(items: &[Value]) -> String {
    let formatted: Vec<String> = items.iter()
        .map(|v| format_value(v))
        .collect();
    format!("[{}]", formatted.join(", "))
}

/// Format a tuple (complexity: 4)
fn format_tuple(items: &[Value]) -> String {
    let formatted: Vec<String> = items.iter()
        .map(|v| format_value(v))
        .collect();
    format!("({})", formatted.join(", "))
}

/// Format an object (complexity: 5)
fn format_object(fields: &[(String, Value)]) -> String {
    let formatted: Vec<String> = fields.iter()
        .map(|(k, v)| format!("{}: {}", k, format_value(v)))
        .collect();
    format!("{{ {} }}", formatted.join(", "))
}

/// Format an error message (complexity: 3)
pub fn format_error(error: &str) -> String {
    format!("Error: {}", error)
}

/// Format AST for display (complexity: 2)
pub fn format_ast(expr: &Expr) -> String {
    format!("{:#?}", expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_formatting() {
        assert_eq!(format_value(&Value::Int(42)), "42");
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_list_formatting() {
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
        assert_eq!(format_value(&list), "[1, 2, 3]");
    }

    #[test]
    fn test_error_formatting() {
        assert_eq!(format_error("test error"), "Error: test error");
    }
}