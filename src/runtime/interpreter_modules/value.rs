//! Value representation and operations
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;
use crate::frontend::ast::Expr;

/// Runtime value representation using safe enum approach
/// Alternative to tagged pointers that respects project's `unsafe_code = "forbid"`
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit float
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Nil/null value
    Nil,
    /// String value (reference-counted for efficiency)
    String(Rc<String>),
    /// Array of values
    Array(Rc<Vec<Value>>),
    /// Tuple of values
    Tuple(Rc<Vec<Value>>),
    /// Function closure
    Closure {
        params: Vec<String>,
        body: Rc<Expr>,
        env: Rc<HashMap<String, Value>>, // Captured environment
    },
}

impl Value {
    /// Create integer value
    pub fn from_i64(i: i64) -> Self {
        Value::Integer(i)
    }

    /// Create float value
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
        Value::String(Rc::new(s))
    }

    /// Create array value
    pub fn from_array(values: Vec<Value>) -> Self {
        Value::Array(Rc::new(values))
    }

    /// Create tuple value
    pub fn from_tuple(values: Vec<Value>) -> Self {
        Value::Tuple(Rc::new(values))
    }

    /// Check if value is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0 && !f.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            _ => true,
        }
    }

    /// Try to convert to integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Try to convert to float
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Try to convert to boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => Some(self.is_truthy()),
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Bool(_) => "boolean",
            Value::Nil => "nil",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Closure { .. } => "closure",
        }
    }

    /// Check if two values are equal
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            _ => false,
        }
    }

    /// Compare values for ordering
    pub fn compare(&self, other: &Value) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
            (Value::Float(a), Value::Float(b)) => {
                if a < b {
                    Some(Ordering::Less)
                } else if a > b {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Equal)
                }
            }
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            (Value::Integer(a), Value::Float(b)) => {
                let af = *a as f64;
                if af < *b {
                    Some(Ordering::Less)
                } else if af > *b {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Equal)
                }
            }
            (Value::Float(a), Value::Integer(b)) => {
                let bf = *b as f64;
                if a < &bf {
                    Some(Ordering::Less)
                } else if a > &bf {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Equal)
                }
            }
            _ => None,
        }
    }

    /// Add two values
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a.wrapping_add(*b)))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, b))))
            }
            _ => Err(format!("Cannot add {} and {}", self.type_name(), other.type_name())),
        }
    }

    /// Subtract two values
    pub fn subtract(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a.wrapping_sub(*b)))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(format!("Cannot subtract {} from {}", other.type_name(), self.type_name())),
        }
    }

    /// Multiply two values
    pub fn multiply(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a.wrapping_mul(*b)))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(format!("Cannot multiply {} and {}", self.type_name(), other.type_name())),
        }
    }

    /// Divide two values
    pub fn divide(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (_, Value::Integer(0)) | (_, Value::Float(f)) if *f == 0.0 => {
                Err("Division by zero".to_string())
            }
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
            _ => Err(format!("Cannot divide {} by {}", self.type_name(), other.type_name())),
        }
    }

    /// Modulo operation
    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            _ => Err(format!("Cannot compute {} modulo {}", self.type_name(), other.type_name())),
        }
    }

    /// Power operation
    pub fn power(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b < 0 {
                    Ok(Value::Float((*a as f64).powf(*b as f64)))
                } else {
                    Ok(Value::Integer(a.pow(*b as u32)))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powi(*b as i32))),
            _ => Err(format!("Cannot compute {} to the power of {}", self.type_name(), other.type_name())),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => {
                if fl.fract() == 0.0 && fl.abs() < 1e10 {
                    write!(f, "{:.1}", fl)
                } else {
                    write!(f, "{}", fl)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(values) => {
                write!(f, "[")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Tuple(values) => {
                write!(f, "(")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                if values.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Closure { .. } => write!(f, "<closure>"),
        }
    }
}