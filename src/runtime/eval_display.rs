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
            Value::Byte(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Array(arr) => format_array(f, arr),
            Value::Tuple(elements) => format_tuple(f, elements),
            Value::Closure { .. } => write!(f, "<function>"),
            Value::DataFrame { columns } => format_dataframe(f, columns),
            Value::Object(obj) => format_object(f, obj),
            Value::ObjectMut(cell) => {
                let obj = cell
                    .lock()
                    .expect("Mutex poisoned in Value::ObjectMut Display - indicates panic in another thread");
                format_object(f, &obj)
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => format_range(f, start, end, *inclusive),
            Value::EnumVariant {
                variant_name, data, ..
            } => format_enum_variant(f, variant_name, data.as_ref()),
            Value::BuiltinFunction(name) => write!(f, "<builtin function: {name}>"),
            Value::Struct { name, fields } => format_struct(f, name, fields),
            Value::Class {
                class_name, fields, ..
            } => format_class(f, class_name, fields),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => write!(f, "<HtmlDocument>"),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => write!(f, "<HtmlElement>"),
            Value::Atom(s) => write!(f, ":{s}"),
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
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Determinism
/// Keys are sorted to ensure deterministic output across multiple runs.
/// This is critical for property testing and reproducible behavior.
fn format_object(
    f: &mut fmt::Formatter<'_>,
    obj: &std::collections::HashMap<String, Value>,
) -> fmt::Result {
    write!(f, "{{")?;

    // Sort keys for deterministic output (DEFECT-DICT-DETERMINISM fix)
    let mut keys: Vec<&String> = obj.keys().collect();
    keys.sort();

    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        let val = &obj[*key];
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
/// Format enum variant with optional data
/// Complexity: 3 (reduced by extracting value formatting)
fn format_enum_variant(
    f: &mut fmt::Formatter<'_>,
    variant_name: &str,
    data: Option<&Vec<Value>>,
) -> fmt::Result {
    write!(f, "{variant_name}")?;
    if let Some(values) = data {
        format_enum_data(f, values)?;
    }
    Ok(())
}

/// Format enum variant data values
/// Complexity: 2 (conditional + delegation)
fn format_enum_data(f: &mut fmt::Formatter<'_>, values: &[Value]) -> fmt::Result {
    if values.is_empty() {
        return Ok(());
    }
    write!(f, "(")?;
    write_comma_separated_values(f, values)?;
    write!(f, ")")
}

/// Write comma-separated values
/// Complexity: 3 (iteration + conditional separator)
fn write_comma_separated_values(f: &mut fmt::Formatter<'_>, values: &[Value]) -> fmt::Result {
    for (i, val) in values.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{val}")?;
    }
    Ok(())
}

/// Format a struct value
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Determinism
/// Keys are sorted to ensure deterministic output across multiple runs.
fn format_struct(
    f: &mut fmt::Formatter<'_>,
    name: &str,
    fields: &std::collections::HashMap<String, Value>,
) -> fmt::Result {
    write!(f, "{name} {{")?;

    // Sort keys for deterministic output
    let mut keys: Vec<&String> = fields.keys().collect();
    keys.sort();

    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        let val = &fields[*key];
        write!(f, "{key}: {val}")?;
    }
    write!(f, "}}")
}

/// Format a class instance value
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Determinism
/// Keys are sorted to ensure deterministic output across multiple runs.
fn format_class(
    f: &mut fmt::Formatter<'_>,
    class_name: &str,
    fields: &std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, Value>>>,
) -> fmt::Result {
    write!(f, "{class_name} {{")?;

    // Sort keys for deterministic output
    let fields_read = fields
        .read()
        .expect("RwLock poisoned in format_class - indicates panic in another thread");
    let mut keys: Vec<&String> = fields_read.keys().collect();
    keys.sort();

    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        let val = &fields_read[*key];
        write!(f, "{key}: {val}")?;
    }
    write!(f, "}}")
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
            InterpreterError::Break(Some(label), _) => {
                write!(f, "Break '{label}' outside of matching loop")
            }
            InterpreterError::Break(None, _) => write!(f, "Break outside of loop"),
            InterpreterError::Continue(Some(label)) => {
                write!(f, "Continue '{label}' outside of matching loop")
            }
            InterpreterError::Continue(None) => write!(f, "Continue outside of loop"),
            InterpreterError::Return(_) => write!(f, "Return outside of function"),
            InterpreterError::Throw(value) => write!(f, "Uncaught exception: {value:?}"),
            InterpreterError::AssertionFailed(msg) => write!(f, "Assertion failed: {msg}"),
            InterpreterError::RecursionLimitExceeded(depth, max) => {
                write!(
                    f,
                    "Recursion limit exceeded: depth {depth} exceeds maximum {max}\n\
                     Hint: Possible infinite recursion detected. Check for:\n\
                     - Functions calling themselves without base case\n\
                     - Mutual recursion between functions\n\
                     - Very deep call chains"
                )
            }
        }
    }
}

impl std::error::Error for InterpreterError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};

    #[test]
    fn test_display_integer() {
        let val = Value::Integer(42);
        assert_eq!(val.to_string(), "42");
    }

    #[test]
    fn test_display_integer_negative() {
        let val = Value::Integer(-100);
        assert_eq!(val.to_string(), "-100");
    }

    #[test]
    fn test_display_float() {
        let val = Value::Float(3.15);
        assert_eq!(val.to_string(), "3.15");
    }

    #[test]
    fn test_display_float_whole_number() {
        let val = Value::Float(5.0);
        assert_eq!(val.to_string(), "5.0");
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_display_byte() {
        let val = Value::Byte(255);
        assert_eq!(val.to_string(), "255");
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
        let val = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        assert_eq!(val.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_display_array_empty() {
        let val = Value::Array(Arc::from(vec![]));
        assert_eq!(val.to_string(), "[]");
    }

    #[test]
    fn test_display_tuple() {
        let val = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        assert_eq!(val.to_string(), "(1, \"test\")");
    }

    #[test]
    fn test_display_tuple_empty() {
        let val = Value::Tuple(Arc::from(vec![]));
        assert_eq!(val.to_string(), "()");
    }

    #[test]
    fn test_display_object() {
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Integer(10));
        obj.insert("y".to_string(), Value::Integer(20));
        let val = Value::Object(Arc::new(obj));
        let display = val.to_string();
        assert!(display.contains("x: 10"));
        assert!(display.contains("y: 20"));
    }

    #[test]
    fn test_display_object_empty() {
        let obj = HashMap::new();
        let val = Value::Object(Arc::new(obj));
        assert_eq!(val.to_string(), "{}");
    }

    #[test]
    fn test_display_object_mut() {
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Value::Integer(42));
        let val = Value::ObjectMut(Arc::new(Mutex::new(obj)));
        let display = val.to_string();
        assert!(display.contains("key: 42"));
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
    fn test_display_closure() {
        let val = Value::Closure {
            params: vec![],
            body: Arc::new(crate::frontend::ast::Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(0, None)),
                crate::frontend::ast::Span::default(),
            )),
            env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
        };
        assert_eq!(val.to_string(), "<function>");
    }

    #[test]
    fn test_display_builtin_function() {
        let val = Value::BuiltinFunction("println".to_string());
        assert_eq!(val.to_string(), "<builtin function: println>");
    }

    #[test]
    fn test_display_atom() {
        let val = Value::Atom("ok".to_string());
        assert_eq!(val.to_string(), ":ok");
    }

    #[test]
    fn test_display_enum_variant_no_data() {
        let val = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };
        assert_eq!(val.to_string(), "None");
    }

    #[test]
    fn test_display_enum_variant_with_data() {
        let val = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        assert_eq!(val.to_string(), "Some(42)");
    }

    #[test]
    fn test_display_enum_variant_empty_data() {
        let val = Value::EnumVariant {
            enum_name: "Test".to_string(),
            variant_name: "Empty".to_string(),
            data: Some(vec![]),
        };
        assert_eq!(val.to_string(), "Empty");
    }

    #[test]
    fn test_display_struct() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::from_string("test".to_string()));
        fields.insert("age".to_string(), Value::Integer(25));
        let val = Value::Struct {
            name: "Person".to_string(),
            fields: Arc::new(fields),
        };
        let display = val.to_string();
        assert!(display.starts_with("Person {"));
        assert!(display.contains("age: 25"));
        assert!(display.contains("name: \"test\""));
    }

    #[test]
    fn test_display_class() {
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), Value::Integer(100));
        let val = Value::Class {
            class_name: "MyClass".to_string(),
            fields: Arc::new(RwLock::new(fields)),
            methods: Arc::new(HashMap::new()),
        };
        let display = val.to_string();
        assert!(display.starts_with("MyClass {"));
        assert!(display.contains("value: 100"));
    }

    #[test]
    fn test_display_dataframe() {
        let columns = vec![
            DataFrameColumn {
                name: "id".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "name".to_string(),
                values: vec![Value::from_string("a".to_string())],
            },
        ];
        let val = Value::DataFrame { columns };
        let display = val.to_string();
        assert!(display.contains("DataFrame with 2 columns"));
        assert!(display.contains("id: 2 rows"));
        assert!(display.contains("name: 1 rows"));
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

    #[test]
    fn test_interpreter_error_runtime() {
        assert_eq!(
            InterpreterError::RuntimeError("something failed".to_string()).to_string(),
            "Runtime error: something failed"
        );
    }

    #[test]
    fn test_interpreter_error_stack_overflow() {
        assert_eq!(InterpreterError::StackOverflow.to_string(), "Stack overflow");
    }

    #[test]
    fn test_interpreter_error_stack_underflow() {
        assert_eq!(InterpreterError::StackUnderflow.to_string(), "Stack underflow");
    }

    #[test]
    fn test_interpreter_error_invalid_instruction() {
        assert_eq!(InterpreterError::InvalidInstruction.to_string(), "Invalid instruction");
    }

    #[test]
    fn test_interpreter_error_index_out_of_bounds() {
        assert_eq!(InterpreterError::IndexOutOfBounds.to_string(), "Index out of bounds");
    }

    #[test]
    fn test_interpreter_error_break_with_label() {
        let err = InterpreterError::Break(Some("outer".to_string()), Value::Nil);
        assert_eq!(err.to_string(), "Break 'outer' outside of matching loop");
    }

    #[test]
    fn test_interpreter_error_break_no_label() {
        let err = InterpreterError::Break(None, Value::Nil);
        assert_eq!(err.to_string(), "Break outside of loop");
    }

    #[test]
    fn test_interpreter_error_continue_with_label() {
        let err = InterpreterError::Continue(Some("inner".to_string()));
        assert_eq!(err.to_string(), "Continue 'inner' outside of matching loop");
    }

    #[test]
    fn test_interpreter_error_continue_no_label() {
        let err = InterpreterError::Continue(None);
        assert_eq!(err.to_string(), "Continue outside of loop");
    }

    #[test]
    fn test_interpreter_error_return() {
        let err = InterpreterError::Return(Value::Integer(5));
        assert_eq!(err.to_string(), "Return outside of function");
    }

    #[test]
    fn test_interpreter_error_throw() {
        let err = InterpreterError::Throw(Value::from_string("error!".to_string()));
        assert!(err.to_string().contains("Uncaught exception"));
    }

    #[test]
    fn test_interpreter_error_assertion_failed() {
        let err = InterpreterError::AssertionFailed("expected true".to_string());
        assert_eq!(err.to_string(), "Assertion failed: expected true");
    }

    #[test]
    fn test_interpreter_error_recursion_limit() {
        let err = InterpreterError::RecursionLimitExceeded(1000, 500);
        let display = err.to_string();
        assert!(display.contains("Recursion limit exceeded"));
        assert!(display.contains("1000"));
        assert!(display.contains("500"));
    }

    #[test]
    fn test_format_tuple_helper() {
        let elements = vec![Value::Integer(1)];
        let val = Value::Tuple(Arc::from(elements));
        assert_eq!(val.to_string(), "(1)");
    }

    #[test]
    fn test_format_nested_array() {
        let inner = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let outer = Value::Array(Arc::from(vec![inner, Value::Integer(3)]));
        assert_eq!(outer.to_string(), "[[1, 2], 3]");
    }

    #[test]
    fn test_format_range_with_floats() {
        let val = Value::Range {
            start: Box::new(Value::Float(1.5)),
            end: Box::new(Value::Float(3.5)),
            inclusive: false,
        };
        assert_eq!(val.to_string(), "1.5..3.5");
    }

    #[test]
    fn test_interpreter_error_is_error_trait() {
        let err: &dyn std::error::Error = &InterpreterError::DivisionByZero;
        assert!(err.to_string().contains("Division"));
    }
}
