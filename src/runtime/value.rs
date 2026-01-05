//! Value Type - Core Runtime Value Representation
//!
//! This module contains the Value enum and related types for the Ruchy runtime.
//! Extracted from interpreter.rs for EXTREME TDD modularization.
//!
//! **EXTREME TDD**: This module contains ONLY the Value type definitions.
//! Utility methods are in value_utils.rs, formatting in value_format.rs.

use crate::frontend::ast::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// `DataFrame` column representation for the interpreter
#[derive(Debug, Clone)]
pub struct DataFrameColumn {
    pub name: String,
    pub values: Vec<Value>,
}

/// Runtime value representation using safe enum approach.
///
/// `Value` represents all runtime values in the Ruchy interpreter. This is an
/// enum-based approach that avoids unsafe code while maintaining good performance
/// through strategic use of `Rc` for heap-allocated data.
///
/// # Examples
///
/// ```
/// use ruchy::runtime::interpreter::Value;
///
/// let int_val = Value::from_i64(42);
/// let str_val = Value::from_string("hello".to_string());
/// let arr_val = Value::from_array(vec![int_val.clone(), str_val]);
/// ```
#[derive(Clone, Debug)]
pub enum Value {
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit float
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Byte value (0-255)
    Byte(u8),
    /// Nil/null value
    Nil,
    /// Atom value (interned identifier)
    Atom(String),
    /// String value (reference-counted for efficiency, thread-safe)
    String(Arc<str>),
    /// Array of values
    Array(Arc<[Value]>),
    /// Tuple of values
    Tuple(Arc<[Value]>),
    /// Function closure
    /// RUNTIME-DEFAULT-PARAMS: Params now store (name, `default_value`) to support default parameters
    Closure {
        params: Vec<(String, Option<Arc<Expr>>)>, // (param_name, default_value)
        body: Arc<Expr>,
        env: Rc<RefCell<HashMap<String, Value>>>, // Shared mutable reference (ISSUE-119)
    },
    /// `DataFrame` value
    DataFrame { columns: Vec<DataFrameColumn> },
    /// Object/HashMap value for key-value mappings (immutable)
    Object(Arc<HashMap<String, Value>>),
    /// Mutable object with interior mutability (for actors and classes, thread-safe)
    ObjectMut(Arc<std::sync::Mutex<HashMap<String, Value>>>),
    /// Range value for representing ranges
    Range {
        start: Box<Value>,
        end: Box<Value>,
        inclusive: bool,
    },
    /// Enum variant value
    EnumVariant {
        enum_name: String,    // The enum type (e.g., "LogLevel")
        variant_name: String, // The variant (e.g., "Debug", "Info")
        data: Option<Vec<Value>>,
    },
    /// Built-in function reference
    BuiltinFunction(String),
    /// Struct instance (value type with named fields)
    /// Thread-safe via Arc, value semantics via cloning
    Struct {
        name: String,
        fields: Arc<HashMap<String, Value>>,
    },
    /// Class instance (reference type with methods)
    /// Thread-safe via Arc, reference semantics, mutable via `RwLock`
    /// Identity-based equality (pointer comparison)
    Class {
        class_name: String,
        fields: Arc<std::sync::RwLock<HashMap<String, Value>>>,
        methods: Arc<HashMap<String, Value>>, // method name -> Closure
    },
    /// HTML document (HTTP-002-C)
    #[cfg(not(target_arch = "wasm32"))]
    HtmlDocument(crate::stdlib::html::HtmlDocument),
    /// HTML element (HTTP-002-C)
    #[cfg(not(target_arch = "wasm32"))]
    HtmlElement(crate::stdlib::html::HtmlElement),
}

// Manual PartialEq implementation because Mutex doesn't implement PartialEq
// ObjectMut uses identity-based equality (Arc pointer comparison) since it represents mutable state
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Atom(a), Value::Atom(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Arc::ptr_eq(a, b) || **a == **b,
            (Value::ObjectMut(a), Value::ObjectMut(b)) => Arc::ptr_eq(a, b), // Identity-based
            (
                Value::Struct {
                    name: n1,
                    fields: f1,
                },
                Value::Struct {
                    name: n2,
                    fields: f2,
                },
            ) => {
                n1 == n2 && **f1 == **f2 // Value equality (compare fields)
            }
            (Value::Class { fields: f1, .. }, Value::Class { fields: f2, .. }) => {
                Arc::ptr_eq(f1, f2) // Identity-based: same instance only
            }
            (Value::Nil, Value::Nil) => true,
            (Value::Byte(a), Value::Byte(b)) => a == b,
            #[cfg(not(target_arch = "wasm32"))]
            (Value::HtmlDocument(_), Value::HtmlDocument(_)) => false, // Documents compared by identity
            #[cfg(not(target_arch = "wasm32"))]
            (Value::HtmlElement(_), Value::HtmlElement(_)) => false, // Elements compared by identity
            _ => false, // Different variants are not equal
        }
    }
}

impl Value {
    /// Get the type ID for this value for caching purposes
    ///
    /// # Complexity
    /// Cyclomatic complexity: 10 (within Toyota Way limits, just barely)
    pub fn type_id(&self) -> std::any::TypeId {
        use std::any::TypeId;
        match self {
            Value::Integer(_) => TypeId::of::<i64>(),
            Value::Float(_) => TypeId::of::<f64>(),
            Value::Bool(_) => TypeId::of::<bool>(),
            Value::Byte(_) => TypeId::of::<u8>(),
            Value::String(_) => TypeId::of::<String>(),
            Value::Atom(_) => TypeId::of::<crate::frontend::lexer::Token>(), // Use Token as proxy type ID
            Value::Nil => TypeId::of::<()>(),
            Value::Array(_) => TypeId::of::<Vec<Value>>(),
            Value::Tuple(_) => TypeId::of::<(Value,)>(), // Generic tuple marker
            Value::Closure { .. } => TypeId::of::<fn()>(), // Generic closure marker
            Value::DataFrame { .. } => TypeId::of::<DataFrameColumn>(),
            Value::Object(_) => TypeId::of::<HashMap<String, Value>>(),
            Value::ObjectMut(_) => TypeId::of::<HashMap<String, Value>>(),
            Value::Range { .. } => TypeId::of::<std::ops::Range<i64>>(),
            Value::EnumVariant { .. } => TypeId::of::<(String, Option<Vec<Value>>)>(),
            Value::BuiltinFunction(_) => TypeId::of::<fn()>(),
            Value::Struct { .. } => TypeId::of::<HashMap<String, Value>>(),
            Value::Class { .. } => TypeId::of::<HashMap<String, Value>>(),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlDocument(_) => TypeId::of::<crate::stdlib::html::HtmlDocument>(),
            #[cfg(not(target_arch = "wasm32"))]
            Value::HtmlElement(_) => TypeId::of::<crate::stdlib::html::HtmlElement>(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_integer_equality() {
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Integer(43));
    }

    #[test]
    fn test_value_float_equality() {
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_ne!(Value::Float(3.14), Value::Float(2.71));
    }

    #[test]
    fn test_value_bool_equality() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Bool(false), Value::Bool(false));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_value_nil_equality() {
        assert_eq!(Value::Nil, Value::Nil);
    }

    #[test]
    fn test_value_byte_equality() {
        assert_eq!(Value::Byte(255), Value::Byte(255));
        assert_ne!(Value::Byte(0), Value::Byte(1));
    }

    #[test]
    fn test_value_string_equality() {
        let s1 = Value::String(Arc::from("hello"));
        let s2 = Value::String(Arc::from("hello"));
        let s3 = Value::String(Arc::from("world"));
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_value_atom_equality() {
        assert_eq!(Value::Atom("foo".to_string()), Value::Atom("foo".to_string()));
        assert_ne!(Value::Atom("foo".to_string()), Value::Atom("bar".to_string()));
    }

    #[test]
    fn test_value_array_equality() {
        let a1 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let a2 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let a3 = Value::Array(Arc::from(vec![Value::Integer(3)]));
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }

    #[test]
    fn test_value_tuple_equality() {
        let t1 = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Bool(true)]));
        let t2 = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Bool(true)]));
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_value_object_equality() {
        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), Value::Integer(1));
        let o1 = Value::Object(Arc::new(map1.clone()));
        let o2 = Value::Object(Arc::new(map1));
        assert_eq!(o1, o2);
    }

    #[test]
    fn test_value_object_mut_identity() {
        let map = HashMap::new();
        let o1 = Value::ObjectMut(Arc::new(std::sync::Mutex::new(map.clone())));
        let o2 = Value::ObjectMut(Arc::new(std::sync::Mutex::new(map)));
        // ObjectMut uses identity equality - different Arcs are not equal
        assert_ne!(o1, o2);
    }

    #[test]
    fn test_value_struct_equality() {
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Integer(10));
        let s1 = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(fields.clone()),
        };
        let s2 = Value::Struct {
            name: "Point".to_string(),
            fields: Arc::new(fields),
        };
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_value_class_identity() {
        let fields1 = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let fields2 = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let methods = Arc::new(HashMap::new());

        let c1 = Value::Class {
            class_name: "MyClass".to_string(),
            fields: fields1.clone(),
            methods: methods.clone(),
        };
        let c2 = Value::Class {
            class_name: "MyClass".to_string(),
            fields: fields2,
            methods: methods.clone(),
        };
        let c3 = Value::Class {
            class_name: "MyClass".to_string(),
            fields: fields1,
            methods: methods,
        };

        // Different instances are not equal
        assert_ne!(c1, c2);
        // Same fields Arc means same instance
        assert_eq!(c1, c3);
    }

    #[test]
    fn test_value_different_variants_not_equal() {
        assert_ne!(Value::Integer(42), Value::Float(42.0));
        assert_ne!(Value::Bool(true), Value::Integer(1));
        assert_ne!(Value::Nil, Value::Integer(0));
    }

    #[test]
    fn test_value_type_id() {
        let int_val = Value::Integer(42);
        let float_val = Value::Float(3.14);
        let bool_val = Value::Bool(true);

        assert_ne!(int_val.type_id(), float_val.type_id());
        assert_ne!(float_val.type_id(), bool_val.type_id());
        assert_eq!(int_val.type_id(), Value::Integer(0).type_id());
    }

    #[test]
    fn test_dataframe_column_clone() {
        let col = DataFrameColumn {
            name: "test".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2)],
        };
        let cloned = col.clone();
        assert_eq!(cloned.name, "test");
        assert_eq!(cloned.values.len(), 2);
    }
}
