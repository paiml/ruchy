//! Display trait implementations for runtime values and errors
//!
//! This module contains all Display formatting logic for Value and `InterpreterError` types,
//! extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::runtime::{DataFrameColumn, InterpreterError, Value};
use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(fl) => {
                if fl.fract() == 0.0 {
                    write!(f, "{fl:.1}")
                } else {
                    write!(f, "{fl}")
                }
            }
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Array(arr) => format_array(f, arr),
            Value::Tuple(elements) => format_tuple(f, elements),
            Value::Closure { .. } => write!(f, "<function>"),
            Value::DataFrame { columns } => format_dataframe(f, columns),
            Value::Object(obj) => format_object(f, obj),
            Value::Range {
                start,
                end,
                inclusive,
            } => format_range(f, start, end, *inclusive),
            Value::EnumVariant { variant_name, data } => {
                format_enum_variant(f, variant_name, data.as_ref())
            }
            Value::BuiltinFunction(name) => write!(f, "<builtin function: {name}>"),
        }
    }
}

/// Format an array value
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn format_array(f: &mut fmt::Formatter<'_>, arr: &[Value]) -> fmt::Result {
    write!(f, "[")?;
    for (i, val) in arr.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{val}")?;
    }
    write!(f, "]")
}

/// Format a tuple value
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn format_tuple(f: &mut fmt::Formatter<'_>, elements: &[Value]) -> fmt::Result {
    write!(f, "(")?;
    for (i, val) in elements.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{val}")?;
    }
    write!(f, ")")
}

/// Format a `DataFrame` value
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn format_dataframe(f: &mut fmt::Formatter<'_>, columns: &[DataFrameColumn]) -> fmt::Result {
    writeln!(f, "DataFrame with {} columns:", columns.len())?;
    for col in columns {
        writeln!(f, "  {}: {} rows", col.name, col.values.len())?;
    }
    Ok(())
}

/// Format an object value
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn format_object(
    f: &mut fmt::Formatter<'_>,
    obj: &std::collections::HashMap<String, Value>,
) -> fmt::Result {
    write!(f, "{{")?;
    for (i, (key, val)) in obj.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{key}: {val}")?;
    }
    write!(f, "}}")
}

/// Format a range value
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn format_range(
    f: &mut fmt::Formatter<'_>,
    start: &Value,
    end: &Value,
    inclusive: bool,
) -> fmt::Result {
    if inclusive {
        write!(f, "{start}..={end}")
    } else {
        write!(f, "{start}..{end}")
    }
}

/// Format an enum variant value
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn format_enum_variant(
    f: &mut fmt::Formatter<'_>,
    variant_name: &str,
    data: Option<&Vec<Value>>,
) -> fmt::Result {
    write!(f, "{variant_name}")?;
    if let Some(values) = data {
        if !values.is_empty() {
            write!(f, "(")?;
            for (i, val) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{val}")?;
            }
            write!(f, ")")?;
        }
    }
    Ok(())
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterError::TypeError(msg) => write!(f, "Type error: {msg}"),
            InterpreterError::RuntimeError(msg) => write!(f, "Runtime error: {msg}"),
            InterpreterError::StackOverflow => write!(f, "Stack overflow"),
            InterpreterError::StackUnderflow => write!(f, "Stack underflow"),
            InterpreterError::InvalidInstruction => write!(f, "Invalid instruction"),
            InterpreterError::DivisionByZero => write!(f, "Division by zero"),
            InterpreterError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            InterpreterError::Break(_) => write!(f, "Break outside of loop"),
            InterpreterError::Continue => write!(f, "Continue outside of loop"),
            InterpreterError::Return(_) => write!(f, "Return outside of function"),
        }
    }
}

impl std::error::Error for InterpreterError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_display_integer() {
        let val = Value::Integer(42);
        assert_eq!(val.to_string(), "42");
    }

    #[test]
    fn test_display_float() {
        let val = Value::Float(3.14);
        assert_eq!(val.to_string(), "3.14");
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_display_nil() {
        assert_eq!(Value::Nil.to_string(), "nil");
    }

    #[test]
    fn test_display_string() {
        let val = Value::from_string("hello".to_string());
        assert_eq!(val.to_string(), "\"hello\"");
    }

    #[test]
    fn test_display_array() {
        let val = Value::Array(Rc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        assert_eq!(val.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_display_tuple() {
        let val = Value::Tuple(Rc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        assert_eq!(val.to_string(), "(1, \"test\")");
    }

    #[test]
    fn test_display_object() {
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Integer(10));
        obj.insert("y".to_string(), Value::Integer(20));
        let val = Value::Object(Rc::new(obj));
        let display = val.to_string();
        assert!(display.contains("x: 10"));
        assert!(display.contains("y: 20"));
    }

    #[test]
    fn test_display_range() {
        let val = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(5)),
            inclusive: false,
        };
        assert_eq!(val.to_string(), "1..5");

        let val = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(5)),
            inclusive: true,
        };
        assert_eq!(val.to_string(), "1..=5");
    }

    #[test]
    fn test_interpreter_error_display() {
        assert_eq!(
            InterpreterError::TypeError("invalid type".to_string()).to_string(),
            "Type error: invalid type"
        );
        assert_eq!(
            InterpreterError::DivisionByZero.to_string(),
            "Division by zero"
        );
    }
}
