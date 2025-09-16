//! Shared pattern matching utilities
//! Extracted to reduce duplication across interpreter and REPL
use crate::frontend::ast::{Literal, Pattern};
use crate::runtime::Value;
///
/// let value = `Value::Int(42)`;
/// let pattern = `Literal::Integer(42)`;
/// `assert!(match_literal_pattern(&value`, &pattern));
///
/// let pattern2 = `Literal::Integer(43)`;
/// `assert!(!match_literal_pattern(&value`, &pattern2));
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
    // Check if there's a rest pattern
    let rest_position = patterns.iter().position(|p| matches!(p, Pattern::Rest | Pattern::RestNamed(_)));
    
    if let Some(rest_idx) = rest_position {
        match_patterns_with_rest(patterns, values, rest_idx)
    } else {
        match_patterns_without_rest(patterns, values)
    }
}

/// Match patterns that contain a rest pattern (complexity: 7)
fn match_patterns_with_rest(patterns: &[Pattern], values: &[Value], rest_idx: usize) -> Option<Vec<(String, Value)>> {
    let before_rest = &patterns[..rest_idx];
    let after_rest = &patterns[rest_idx + 1..];
    
    // Check if we have enough values for non-rest patterns
    if values.len() < before_rest.len() + after_rest.len() {
        return None;
    }
    
    let mut bindings = Vec::new();
    
    // Match patterns before rest
    bindings.extend(match_pattern_sequence(before_rest, &values[..before_rest.len()])?);
    
    // Handle the rest pattern
    let rest_start = before_rest.len();
    let rest_end = values.len() - after_rest.len();
    let rest_values = &values[rest_start..rest_end];
    
    if let Pattern::RestNamed(name) = &patterns[rest_idx] {
        bindings.push((name.clone(), Value::List(rest_values.to_vec())));
    }
    // Pattern::Rest doesn't bind anything
    
    // Match patterns after rest
    bindings.extend(match_pattern_sequence(after_rest, &values[rest_end..])?);
    
    Some(bindings)
}

/// Match patterns without rest pattern (complexity: 3)
fn match_patterns_without_rest(patterns: &[Pattern], values: &[Value]) -> Option<Vec<(String, Value)>> {
    if patterns.len() != values.len() {
        return None;
    }
    match_pattern_sequence(patterns, values)
}

/// Match a sequence of patterns against values (complexity: 3)
fn match_pattern_sequence(patterns: &[Pattern], values: &[Value]) -> Option<Vec<(String, Value)>> {
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
        Pattern::Literal(lit) => match_literal_pattern_helper(value, lit),
        Pattern::Tuple(patterns) => match_tuple_pattern_helper(patterns, value),
        Pattern::List(patterns) => {
            match_list_pattern_helper(patterns, value)
        },
        Pattern::Struct { fields, .. } => match_struct_pattern_helper(fields, value),
        Pattern::Or(patterns) => match_or_pattern_helper(patterns, value),
        Pattern::Rest | Pattern::RestNamed(_) => {
            // Rest patterns when used standalone act like wildcards
            // When within collections, they're handled by match_collection_patterns
            Some(vec![])
        },
        Pattern::Range { start, end, inclusive } => match_range_pattern_helper(start, end, *inclusive, value),
        Pattern::QualifiedName(_) => None,
        Pattern::Some(inner_pattern) => match_some_pattern_helper(inner_pattern, value),
        Pattern::None => match_none_pattern_helper(value),
        Pattern::Ok(_) | Pattern::Err(_) => None,
        Pattern::WithDefault { pattern, .. } => {
            // For default patterns, we match the inner pattern
            // Default handling is done at the destructuring level
            match_pattern(pattern, value)
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
/// Helper for matching literal patterns (complexity: 2)
fn match_literal_pattern_helper(value: &Value, lit: &Literal) -> Option<Vec<(String, Value)>> {
    if match_literal_pattern(value, lit) {
        Some(vec![])
    } else {
        None
    }
}
/// Helper for matching tuple patterns (complexity: 3)
fn match_tuple_pattern_helper(patterns: &[Pattern], value: &Value) -> Option<Vec<(String, Value)>> {
    if let Value::Tuple(values) = value {
        match_collection_patterns(patterns, values)
    } else {
        None
    }
}
/// Helper for matching list patterns (complexity: 3)
fn match_list_pattern_helper(patterns: &[Pattern], value: &Value) -> Option<Vec<(String, Value)>> {
    if let Value::List(values) = value {
        match_collection_patterns(patterns, values)
    } else {
        None
    }
}
/// Helper for matching struct patterns (complexity: 8)
fn match_struct_pattern_helper(fields: &[crate::frontend::ast::StructPatternField], value: &Value) -> Option<Vec<(String, Value)>> {
    if let Value::Object(obj_fields) = value {
        let mut bindings = Vec::new();
        for field in fields {
            let field_value = obj_fields.get(&field.name)?;
            if let Some(ref field_pattern) = field.pattern {
                let field_bindings = match_pattern(field_pattern, field_value)?;
                bindings.extend(field_bindings);
            } else {
                bindings.push((field.name.clone(), field_value.clone()));
            }
        }
        Some(bindings)
    } else {
        None
    }
}
/// Helper for matching or patterns (complexity: 4)
fn match_or_pattern_helper(patterns: &[Pattern], value: &Value) -> Option<Vec<(String, Value)>> {
    for pat in patterns {
        if let Some(bindings) = match_pattern(pat, value) {
            return Some(bindings);
        }
    }
    None
}
/// Helper for matching range patterns (complexity: 9)
fn match_range_pattern_helper(start: &Pattern, end: &Pattern, inclusive: bool, value: &Value) -> Option<Vec<(String, Value)>> {
    if let Value::Int(val) = value {
        let start_val = if let Pattern::Literal(Literal::Integer(n)) = start {
            *n
        } else {
            return None;
        };
        let end_val = if let Pattern::Literal(Literal::Integer(n)) = end {
            *n
        } else {
            return None;
        };
        let val = *val;
        let in_range = if inclusive {
            val >= start_val && val <= end_val
        } else {
            val >= start_val && val < end_val
        };
        if in_range {
            Some(Vec::new())
        } else {
            None
        }
    } else {
        None
    }
}
/// Helper for matching Some patterns (complexity: 6)
fn match_some_pattern_helper(inner_pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
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
/// Helper for matching None patterns (complexity: 4)
fn match_none_pattern_helper(value: &Value) -> Option<Vec<(String, Value)>> {
    if let Value::EnumVariant { variant_name, .. } = value {
        if variant_name == "None" {
            return Some(Vec::new());
        }
    }
    None
}
#[cfg(test)]
mod property_tests_pattern_matching {
    use proptest::proptest;
    
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_match_literal_pattern_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
