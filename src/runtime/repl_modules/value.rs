//! Value enum and its implementations
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use std::collections::{HashMap, HashSet};
use std::fmt;
use crate::frontend::ast::Expr;

/// Runtime value for evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    DataFrame {
        columns: Vec<DataFrameColumn>,
    },
    Object(HashMap<String, Value>),
    HashMap(HashMap<Value, Value>),
    HashSet(HashSet<Value>),
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    EnumVariant {
        enum_name: String,
        variant_name: String,
        data: Option<Vec<Value>>,
    },
    Unit,
    Nil,
}

/// DataFrame column representation for pretty printing
#[derive(Debug, Clone, PartialEq)]
pub struct DataFrameColumn {
    pub name: String,
    pub values: Vec<Value>,
}

// Manual Eq implementation for Value
impl Eq for Value {}

// Manual Hash implementation for Value
impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Int(i) => {
                0.hash(state);
                i.hash(state);
            }
            Value::Float(f) => {
                1.hash(state);
                f.to_bits().hash(state);
            }
            Value::String(s) => {
                2.hash(state);
                s.hash(state);
            }
            Value::Bool(b) => {
                3.hash(state);
                b.hash(state);
            }
            Value::Char(c) => {
                4.hash(state);
                c.hash(state);
            }
            Value::List(l) => {
                5.hash(state);
                l.hash(state);
            }
            Value::Tuple(t) => {
                6.hash(state);
                t.hash(state);
            }
            Value::Unit => {
                7.hash(state);
            }
            Value::Nil => {
                8.hash(state);
            }
            Value::Range { start, end, inclusive } => {
                9.hash(state);
                start.hash(state);
                end.hash(state);
                inclusive.hash(state);
            }
            Value::EnumVariant { enum_name, variant_name, data } => {
                10.hash(state);
                enum_name.hash(state);
                variant_name.hash(state);
                data.hash(state);
            }
            _ => {
                // Other types cannot be hashed
                11.hash(state);
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::List(l) => format_list(f, l),
            Value::Tuple(t) => format_tuple(f, t),
            Value::Unit => write!(f, "()"),
            Value::Nil => write!(f, "None"),
            Value::Function { name, .. } => write!(f, "<function {}>", name),
            Value::Lambda { .. } => write!(f, "<lambda>"),
            Value::DataFrame { columns } => format_dataframe(f, columns),
            Value::Object(fields) => format_object(f, fields),
            Value::HashMap(map) => format_hashmap(f, map),
            Value::HashSet(set) => format_hashset(f, set),
            Value::Range { start, end, inclusive } => {
                if *inclusive {
                    write!(f, "{}..={}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            Value::EnumVariant { enum_name, variant_name, data } => {
                format_enum_variant(f, enum_name, variant_name, data)
            }
        }
    }
}

// Helper functions for Display implementation (complexity: ≤10)
fn format_list(f: &mut fmt::Formatter<'_>, list: &[Value]) -> fmt::Result {
    write!(f, "[")?;
    for (i, v) in list.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", v)?;
    }
    write!(f, "]")
}

fn format_tuple(f: &mut fmt::Formatter<'_>, tuple: &[Value]) -> fmt::Result {
    write!(f, "(")?;
    for (i, v) in tuple.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", v)?;
    }
    if tuple.len() == 1 {
        write!(f, ",")?;
    }
    write!(f, ")")
}

fn format_object(f: &mut fmt::Formatter<'_>, fields: &HashMap<String, Value>) -> fmt::Result {
    write!(f, "{{")?;
    let mut first = true;
    for (k, v) in fields {
        if !first {
            write!(f, ", ")?;
        }
        write!(f, "{}: {}", k, v)?;
        first = false;
    }
    write!(f, "}}")
}

fn format_hashmap(f: &mut fmt::Formatter<'_>, map: &HashMap<Value, Value>) -> fmt::Result {
    write!(f, "{{")?;
    let mut first = true;
    for (k, v) in map {
        if !first {
            write!(f, ", ")?;
        }
        write!(f, "{}: {}", k, v)?;
        first = false;
    }
    write!(f, "}}")
}

fn format_hashset(f: &mut fmt::Formatter<'_>, set: &HashSet<Value>) -> fmt::Result {
    write!(f, "{{")?;
    let mut first = true;
    for v in set {
        if !first {
            write!(f, ", ")?;
        }
        write!(f, "{}", v)?;
        first = false;
    }
    write!(f, "}}")
}

fn format_dataframe(f: &mut fmt::Formatter<'_>, columns: &[DataFrameColumn]) -> fmt::Result {
    writeln!(f, "DataFrame [")?;
    for col in columns {
        writeln!(f, "  {}: {:?}", col.name, col.values)?;
    }
    write!(f, "]")
}

fn format_enum_variant(
    f: &mut fmt::Formatter<'_>,
    enum_name: &str,
    variant_name: &str,
    data: &Option<Vec<Value>>
) -> fmt::Result {
    write!(f, "{}::{}", enum_name, variant_name)?;
    if let Some(values) = data {
        write!(f, "(")?;
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
        }
        write!(f, ")")?;
    }
    Ok(())
}

impl Value {
    /// Get the type name of the value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "str",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::List(_) => "list",
            Value::Tuple(_) => "tuple",
            Value::Function { .. } => "function",
            Value::Lambda { .. } => "lambda",
            Value::DataFrame { .. } => "dataframe",
            Value::Object(_) => "object",
            Value::HashMap(_) => "hashmap",
            Value::HashSet(_) => "hashset",
            Value::Range { .. } => "range",
            Value::EnumVariant { .. } => "enum_variant",
            Value::Unit => "unit",
            Value::Nil => "nil",
        }
    }

    /// Check if value is truthy for boolean context
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil | Value::Unit => false,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0 && !f.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::List(l) | Value::Tuple(l) => !l.is_empty(),
            Value::HashMap(m) => !m.is_empty(),
            Value::HashSet(s) => !s.is_empty(),
            _ => true,
        }
    }
}