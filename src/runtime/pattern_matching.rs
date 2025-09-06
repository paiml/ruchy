//! Shared pattern matching utilities
//! Extracted to reduce duplication across interpreter and REPL

use crate::frontend::ast::{Literal, Pattern};
use crate::runtime::Value;

/// Match a literal pattern against a value
///
/// # Examples
///
/// ```
/// use ruchy::runtime::pattern_matching::match_literal_pattern;
/// use ruchy::runtime::Value;
/// use ruchy::frontend::ast::Literal;
///
/// let value = Value::Int(42);
/// let pattern = Literal::Integer(42);
/// assert!(match_literal_pattern(&value, &pattern));
///
/// let pattern2 = Literal::Integer(43);
/// assert!(!match_literal_pattern(&value, &pattern2));
/// ```
pub fn match_literal_pattern(value: &Value, literal: &Literal) -> bool {
    match (value, literal) {
        (Value::Unit, Literal::Unit) => true,
        (Value::Int(v), Literal::Integer(p)) => v == p,
        (Value::Float(v), Literal::Float(p)) => (v - p).abs() < f64::EPSILON,
        (Value::String(v), Literal::String(p)) => v == p,
        (Value::Bool(v), Literal::Bool(p)) => v == p,
        (Value::Char(v), Literal::Char(p)) => v == p,
        _ => false,
    }
}

/// Helper function to match collection patterns (tuple or list)
fn match_collection_patterns(patterns: &[Pattern], values: &[Value]) -> Option<Vec<(String, Value)>> {
    if patterns.len() != values.len() {
        return None;
    }
    
    let mut bindings = Vec::new();
    for (pat, val) in patterns.iter().zip(values.iter()) {
        let sub_bindings = match_pattern(pat, val)?;
        bindings.extend(sub_bindings);
    }
    Some(bindings)
}

/// Match a pattern against a value, returning bindings if successful
pub fn match_pattern(pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match pattern {
        Pattern::Wildcard => Some(vec![]),
        
        Pattern::Identifier(name) => Some(vec![(name.clone(), value.clone())]),
        
        Pattern::Literal(lit) => {
            if match_literal_pattern(value, lit) {
                Some(vec![])
            } else {
                None
            }
        }
        
        Pattern::Tuple(patterns) => {
            if let Value::Tuple(values) = value {
                match_collection_patterns(patterns, values)
            } else {
                None
            }
        }
        
        Pattern::List(patterns) => {
            if let Value::List(values) = value {
                match_collection_patterns(patterns, values)
            } else {
                None
            }
        }
        
        Pattern::Struct { fields, .. } => {
            if let Value::Object(obj_fields) = value {
                let mut bindings = Vec::new();
                
                // Check each field pattern
                for field in fields {
                    // Get the value for this field
                    let field_value = obj_fields.get(&field.name)?;
                    
                    // Match the pattern if provided
                    if let Some(ref field_pattern) = field.pattern {
                        let field_bindings = match_pattern(field_pattern, field_value)?;
                        bindings.extend(field_bindings);
                    } else {
                        // Shorthand: field name is the binding
                        bindings.push((field.name.clone(), field_value.clone()));
                    }
                }
                
                Some(bindings)
            } else {
                None
            }
        }
        
        Pattern::Or(patterns) => {
            for pat in patterns {
                if let Some(bindings) = match_pattern(pat, value) {
                    return Some(bindings);
                }
            }
            None
        }
        
        Pattern::Rest => {
            // Rest patterns match anything
            Some(vec![])
        }
        
        Pattern::RestNamed(name) => {
            // Rest pattern with binding
            Some(vec![(name.clone(), value.clone())])
        }
        
        Pattern::Range { start, end, inclusive } => {
            // Range patterns for numeric values
            if let Value::Int(val) = value {
                // Extract start and end values from patterns
                let start_val = if let Pattern::Literal(Literal::Integer(n)) = &**start {
                    *n
                } else {
                    return None; // Start must be a literal integer
                };
                
                let end_val = if let Pattern::Literal(Literal::Integer(n)) = &**end {
                    *n
                } else {
                    return None; // End must be a literal integer
                };
                
                let val = *val;
                let in_range = if *inclusive {
                    val >= start_val && val <= end_val
                } else {
                    val >= start_val && val < end_val
                };
                
                if in_range {
                    Some(Vec::new()) // No variable bindings for range patterns
                } else {
                    None
                }
            } else {
                None // Range patterns only match integers
            }
        }
        
        Pattern::QualifiedName(_) => {
            // Qualified names need enum support
            // For now, just fail to match
            None
        }
        
        Pattern::Some(inner_pattern) => {
            // Match EnumVariant with variant_name "Some"
            if let Value::EnumVariant { variant_name, data, .. } = value {
                if variant_name == "Some" {
                    if let Some(ref variant_data) = data {
                        if !variant_data.is_empty() {
                            return match_pattern(inner_pattern, &variant_data[0]);
                        }
                    }
                }
            }
            None
        }
        
        Pattern::None => {
            // Match EnumVariant with variant_name "None"
            if let Value::EnumVariant { variant_name, .. } = value {
                if variant_name == "None" {
                    return Some(Vec::new());
                }
            }
            None
        }
        
        Pattern::Ok(_) | Pattern::Err(_) => {
            // Result patterns not yet fully supported
            None
        }
    }
}

/// Check if two values are equal (for pattern matching)
pub fn values_equal(v1: &Value, v2: &Value) -> bool {
    match (v1, v2) {
        (Value::Unit, Value::Unit) => true,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Char(a), Value::Char(b)) => a == b,
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Tuple(a), Value::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(f1), Value::Object(f2)) => {
            f1.len() == f2.len() && 
            f1.iter().all(|(k, v)| f2.get(k).is_some_and(|v2| values_equal(v, v2)))
        }
        (Value::Range { start: s1, end: e1, inclusive: i1 }, 
         Value::Range { start: s2, end: e2, inclusive: i2 }) => {
            s1 == s2 && e1 == e2 && i1 == i2
        }
        _ => false,
    }
}