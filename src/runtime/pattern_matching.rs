//! Shared pattern matching utilities
//! Extracted to reduce duplication across interpreter and REPL

use crate::frontend::ast::{Literal, Pattern};
use crate::runtime::Value;

/// Match a literal pattern against a value
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
                if patterns.len() != values.len() {
                    return None;
                }
                
                let mut bindings = Vec::new();
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    let sub_bindings = match_pattern(pat, val)?;
                    bindings.extend(sub_bindings);
                }
                Some(bindings)
            } else {
                None
            }
        }
        
        Pattern::List(patterns) => {
            if let Value::List(values) = value {
                if patterns.len() != values.len() {
                    return None;
                }
                
                let mut bindings = Vec::new();
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    let sub_bindings = match_pattern(pat, val)?;
                    bindings.extend(sub_bindings);
                }
                Some(bindings)
            } else {
                None
            }
        }
        
        Pattern::Struct { name, fields, .. } => {
            if let Value::Object(obj_fields) = value {
                // Object doesn't store type name, so we can't match on it
                // Just extract bindings from fields
                let mut bindings = Vec::new();
                for _field in fields {
                    // Handle StructPatternField properly
                    // For now, just match by name
                    bindings.push((name.clone(), value.clone()));
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
        
        Pattern::Range { .. } => {
            // Range patterns need numeric comparison
            // For now, just fail to match
            None
        }
        
        Pattern::QualifiedName(_) => {
            // Qualified names need enum support
            // For now, just fail to match
            None
        }
        
        Pattern::Ok(_) | Pattern::Err(_) | Pattern::Some(_) | Pattern::None => {
            // Result/Option patterns not yet supported in Value enum
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
            f1.iter().all(|(k, v)| f2.get(k).map_or(false, |v2| values_equal(v, v2)))
        }
        _ => false,
    }
}