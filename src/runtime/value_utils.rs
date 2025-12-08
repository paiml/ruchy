//! Value utility methods module
//!
//! This module contains utility methods for the Value type including
//! constructors, type checking, and conversion methods.
//! Extracted for maintainability and following Toyota Way principles.

use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

impl Value {
    /// Create an integer value from an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::Value;
    ///
    /// let val = Value::from_i64(42);
    /// assert_eq!(val.as_i64().unwrap(), 42);
    /// ```
    pub fn from_i64(i: i64) -> Self {
        Value::Integer(i)
    }

    /// Create a float value from an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::Value;
    ///
    /// let val = Value::from_f64(3.15);
    /// assert_eq!(val.as_f64().unwrap(), 3.15);
    /// ```
    pub fn from_f64(f: f64) -> Self {
        Value::Float(f)
    }

    /// Create boolean value
    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }

    /// Create nil value
    pub fn nil() -> Self {
        Value::Nil
    }

    /// Create string value
    pub fn from_string(s: String) -> Self {
        Value::String(Arc::from(s))
    }

    /// Create array value
    pub fn from_array(arr: Vec<Value>) -> Self {
        Value::Array(Arc::from(arr))
    }

    /// Create object value
    pub fn from_object(obj: HashMap<String, Value>) -> Self {
        Value::Object(Arc::new(obj))
    }

    /// Create range value
    pub fn from_range(start: Value, end: Value, inclusive: bool) -> Self {
        Value::Range {
            start: Box::new(start),
            end: Box::new(end),
            inclusive,
        }
    }

    /// Create enum variant value
    pub fn from_enum_variant(
        enum_name: String,
        variant_name: String,
        data: Option<Vec<Value>>,
    ) -> Self {
        Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        }
    }

    /// Check if value is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if value is truthy.
    ///
    /// In Ruchy, only `false` and `nil` are falsy. All other values,
    /// including `0` and empty strings, are truthy.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::Value;
    ///
    /// assert!(Value::from_i64(0).is_truthy());
    /// assert!(Value::from_string("".to_string()).is_truthy());
    /// assert!(!Value::Bool(false).is_truthy());
    /// assert!(!Value::Nil.is_truthy());
    /// ```
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    /// Extract integer value
    /// # Errors
    /// Returns error if the value is not an integer
    pub fn as_i64(&self) -> Result<i64, InterpreterError> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(InterpreterError::TypeError(format!(
                "Expected integer, got {}",
                self.type_name()
            ))),
        }
    }

    /// Extract float value
    /// # Errors
    /// Returns error if the value is not a float
    pub fn as_f64(&self) -> Result<f64, InterpreterError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(InterpreterError::TypeError(format!(
                "Expected float, got {}",
                self.type_name()
            ))),
        }
    }

    /// Extract boolean value
    /// # Errors
    /// Returns error if the value is not a boolean
    pub fn as_bool(&self) -> Result<bool, InterpreterError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(InterpreterError::TypeError(format!(
                "Expected boolean, got {}",
                self.type_name()
            ))),
        }
    }

    /// Get type name for debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Bool(_) => "boolean",
            Value::Byte(_) => "byte",
            Value::Nil => "nil",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Closure { .. } => "function",
            Value::DataFrame { .. } => "dataframe",
            Value::Object(_) => "object",
            Value::ObjectMut(_) => "object",
            Value::Range { .. } => "range",
            Value::EnumVariant { .. } => "enum_variant",
            Value::BuiltinFunction(_) => "builtin_function",
            Value::Struct { .. } => "struct",
            Value::Class { .. } => "class",
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => "html_document",
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => "html_element",
            Value::Atom(_) => "atom",
        }
    }

    /// Add two values (for bytecode VM)
    ///
    /// Supports: Integer + Integer, Float + Float, mixed numeric types, String + String, Array + Array
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_add(*b)
                .map(Value::Integer)
                .ok_or_else(|| "Integer overflow in addition".to_string()),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::from_string(format!("{}{}", a.as_ref(), b.as_ref())))
            }
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.as_ref().to_vec();
                result.extend_from_slice(b.as_ref());
                Ok(Value::from_array(result))
            }
            _ => Err(format!(
                "Cannot add {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    /// Subtract two values (for bytecode VM)
    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_sub(*b)
                .map(Value::Integer)
                .ok_or_else(|| "Integer overflow in subtraction".to_string()),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(format!(
                "Cannot subtract {} from {}",
                other.type_name(),
                self.type_name()
            )),
        }
    }

    /// Multiply two values (for bytecode VM)
    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_mul(*b)
                .map(Value::Integer)
                .ok_or_else(|| "Integer overflow in multiplication".to_string()),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(format!(
                "Cannot multiply {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    /// Divide two values (for bytecode VM)
    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Integer(a / b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float(a / b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float(*a as f64 / b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Float(a / *b as f64))
            }
            _ => Err(format!(
                "Cannot divide {} by {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    /// Modulo operation (for bytecode VM)
    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                Ok(Value::Integer(a % b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("Modulo by zero".to_string());
                }
                Ok(Value::Float(a % b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err("Modulo by zero".to_string());
                }
                Ok(Value::Float(*a as f64 % b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                Ok(Value::Float(a % *b as f64))
            }
            _ => Err(format!(
                "Cannot modulo {} by {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    /// Compare values with less-than operator (for bytecode VM)
    pub fn less_than(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a < b,
            (Value::Float(a), Value::Float(b)) => a < b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) < *b,
            (Value::Float(a), Value::Integer(b)) => *a < (*b as f64),
            _ => false,
        }
    }

    /// Compare values with less-than-or-equal operator (for bytecode VM)
    pub fn less_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a <= b,
            (Value::Float(a), Value::Float(b)) => a <= b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) <= *b,
            (Value::Float(a), Value::Integer(b)) => *a <= (*b as f64),
            _ => false,
        }
    }

    /// Compare values with greater-than operator (for bytecode VM)
    pub fn greater_than(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a > b,
            (Value::Float(a), Value::Float(b)) => a > b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) > *b,
            (Value::Float(a), Value::Integer(b)) => *a > (*b as f64),
            _ => false,
        }
    }

    /// Compare values with greater-than-or-equal operator (for bytecode VM)
    pub fn greater_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a >= b,
            (Value::Float(a), Value::Float(b)) => a >= b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) >= *b,
            (Value::Float(a), Value::Integer(b)) => *a >= (*b as f64),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_constructors() {
        let int_val = Value::from_i64(42);
        assert_eq!(int_val.as_i64().unwrap(), 42);

        let float_val = Value::from_f64(3.15);
        assert_eq!(float_val.as_f64().unwrap(), 3.15);

        let bool_val = Value::from_bool(true);
        assert!(bool_val.as_bool().unwrap());

        let nil_val = Value::nil();
        assert!(nil_val.is_nil());

        let string_val = Value::from_string("test".to_string());
        match string_val {
            Value::String(s) => assert_eq!(s.as_ref(), "test"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_is_truthy() {
        assert!(Value::from_i64(0).is_truthy());
        assert!(Value::from_i64(1).is_truthy());
        assert!(Value::from_string(String::new()).is_truthy());
        assert!(Value::from_string("test".to_string()).is_truthy());
        assert!(Value::from_bool(true).is_truthy());
        assert!(!Value::from_bool(false).is_truthy());
        assert!(!Value::nil().is_truthy());
    }

    #[test]
    fn test_type_name() {
        assert_eq!(Value::from_i64(42).type_name(), "integer");
        assert_eq!(Value::from_f64(3.15).type_name(), "float");
        assert_eq!(Value::from_bool(true).type_name(), "boolean");
        assert_eq!(Value::nil().type_name(), "nil");
        assert_eq!(Value::from_string("test".to_string()).type_name(), "string");
        assert_eq!(Value::from_array(vec![]).type_name(), "array");
    }

    #[test]
    fn test_value_extraction_errors() {
        let int_val = Value::from_i64(42);
        assert!(int_val.as_f64().is_err());
        assert!(int_val.as_bool().is_err());

        let float_val = Value::from_f64(3.15);
        assert!(float_val.as_i64().is_err());
        assert!(float_val.as_bool().is_err());

        let bool_val = Value::from_bool(true);
        assert!(bool_val.as_i64().is_err());
        assert!(bool_val.as_f64().is_err());
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[test]
    fn test_is_nil_not_stub() {
        // MISSED: replace Value::is_nil -> bool with true

        // Test Nil value returns true
        assert!(Value::Nil.is_nil(), "Nil should return true");

        // Test non-Nil values return false (proves not always true)
        assert!(!Value::Integer(0).is_nil(), "Integer should return false");
        assert!(!Value::Bool(false).is_nil(), "Bool should return false");
        assert!(
            !Value::from_string(String::new()).is_nil(),
            "String should return false"
        );
    }
}
