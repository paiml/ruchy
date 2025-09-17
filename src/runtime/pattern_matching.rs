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
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Pattern, StructPatternField};
    use crate::runtime::Value;
    use std::collections::HashMap;

    // Helper function to create test values
    fn create_test_values() -> Vec<Value> {
        vec![
            Value::Unit,
            Value::Int(42),
            Value::Float(3.14),
            Value::String("test".to_string()),
            Value::Bool(true),
            Value::Char('a'),
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Tuple(vec![Value::String("hello".to_string()), Value::Int(10)]),
        ]
    }

    // Test 1: Literal Pattern Matching
    #[test]
    fn test_literal_pattern_matching() {
        // Integer literal matching
        assert!(match_literal_pattern(&Value::Int(42), &Literal::Integer(42)));
        assert!(!match_literal_pattern(&Value::Int(42), &Literal::Integer(43)));

        // Float literal matching with epsilon
        assert!(match_literal_pattern(&Value::Float(3.14), &Literal::Float(3.14)));
        assert!(match_literal_pattern(&Value::Float(1.0), &Literal::Float(1.0 + f64::EPSILON / 2.0)));
        assert!(!match_literal_pattern(&Value::Float(1.0), &Literal::Float(1.5)));

        // String literal matching
        assert!(match_literal_pattern(&Value::String("hello".to_string()), &Literal::String("hello".to_string())));
        assert!(!match_literal_pattern(&Value::String("hello".to_string()), &Literal::String("world".to_string())));

        // Boolean literal matching
        assert!(match_literal_pattern(&Value::Bool(true), &Literal::Bool(true)));
        assert!(!match_literal_pattern(&Value::Bool(true), &Literal::Bool(false)));

        // Character literal matching
        assert!(match_literal_pattern(&Value::Char('a'), &Literal::Char('a')));
        assert!(!match_literal_pattern(&Value::Char('a'), &Literal::Char('b')));

        // Unit literal matching
        assert!(match_literal_pattern(&Value::Unit, &Literal::Unit));

        // Type mismatch should not match
        assert!(!match_literal_pattern(&Value::Int(42), &Literal::String("42".to_string())));
        assert!(!match_literal_pattern(&Value::String("true".to_string()), &Literal::Bool(true)));
    }

    // Test 2: Values Equality Function
    #[test]
    fn test_values_equal() {
        // Basic type equality
        assert!(values_equal(&Value::Int(42), &Value::Int(42)));
        assert!(!values_equal(&Value::Int(42), &Value::Int(43)));

        // Float equality with epsilon
        assert!(values_equal(&Value::Float(3.14), &Value::Float(3.14)));
        assert!(values_equal(&Value::Float(1.0), &Value::Float(1.0 + f64::EPSILON / 2.0)));
        assert!(!values_equal(&Value::Float(1.0), &Value::Float(1.5)));

        // String equality
        assert!(values_equal(&Value::String("test".to_string()), &Value::String("test".to_string())));
        assert!(!values_equal(&Value::String("test".to_string()), &Value::String("other".to_string())));

        // List equality (recursive)
        let list1 = Value::List(vec![Value::Int(1), Value::String("test".to_string())]);
        let list2 = Value::List(vec![Value::Int(1), Value::String("test".to_string())]);
        let list3 = Value::List(vec![Value::Int(1), Value::String("other".to_string())]);
        assert!(values_equal(&list1, &list2));
        assert!(!values_equal(&list1, &list3));

        // Tuple equality (recursive)
        let tuple1 = Value::Tuple(vec![Value::Bool(true), Value::Int(42)]);
        let tuple2 = Value::Tuple(vec![Value::Bool(true), Value::Int(42)]);
        let tuple3 = Value::Tuple(vec![Value::Bool(false), Value::Int(42)]);
        assert!(values_equal(&tuple1, &tuple2));
        assert!(!values_equal(&tuple1, &tuple3));

        // Different lengths should not be equal
        let short_list = Value::List(vec![Value::Int(1)]);
        let long_list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(!values_equal(&short_list, &long_list));

        // Different types should not be equal
        assert!(!values_equal(&Value::Int(42), &Value::String("42".to_string())));
        assert!(!values_equal(&Value::List(vec![]), &Value::Tuple(vec![])));
    }

    // Test 3: Simple Pattern Matching
    #[test]
    fn test_simple_pattern_matching() {
        // Wildcard pattern should match anything
        assert!(match_pattern(&Pattern::Wildcard, &Value::Int(42)).is_some());
        assert!(match_pattern(&Pattern::Wildcard, &Value::String("test".to_string())).is_some());
        assert!(match_pattern(&Pattern::Wildcard, &Value::Unit).is_some());

        // Variable pattern should bind value
        let binding = match_pattern(&Pattern::Identifier("x".to_string()), &Value::Int(42));
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert!(values_equal(&bindings[0].1, &Value::Int(42)));

        // Literal pattern matching
        let literal_pattern = Pattern::Literal(Literal::Integer(42));
        assert!(match_pattern(&literal_pattern, &Value::Int(42)).is_some());
        assert!(match_pattern(&literal_pattern, &Value::Int(43)).is_none());
    }

    // Test 4: Tuple Pattern Matching
    #[test]
    fn test_tuple_pattern_matching() {
        let tuple_value = Value::Tuple(vec![Value::Int(1), Value::String("test".to_string()), Value::Bool(true)]);

        // Exact tuple match
        let tuple_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Identifier("s".to_string()),
            Pattern::Literal(Literal::Bool(true))
        ]);

        let binding = match_pattern(&tuple_pattern, &tuple_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "s");
        assert!(values_equal(&bindings[0].1, &Value::String("test".to_string())));

        // Wrong tuple length should not match
        let wrong_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Identifier("s".to_string())
        ]);
        assert!(match_pattern(&wrong_pattern, &tuple_value).is_none());

        // Wrong element values should not match
        let mismatch_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Identifier("s".to_string()),
            Pattern::Literal(Literal::Bool(true))
        ]);
        assert!(match_pattern(&mismatch_pattern, &tuple_value).is_none());
    }

    // Test 5: List Pattern Matching
    #[test]
    fn test_list_pattern_matching() {
        let list_value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        // Exact list match with variables
        let list_pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Identifier("last".to_string())
        ]);

        let binding = match_pattern(&list_pattern, &list_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 2);

        // Check bindings (order may vary, so check both)
        let first_binding = bindings.iter().find(|(name, _)| name == "first").unwrap();
        let last_binding = bindings.iter().find(|(name, _)| name == "last").unwrap();
        assert!(values_equal(&first_binding.1, &Value::Int(1)));
        assert!(values_equal(&last_binding.1, &Value::Int(3)));

        // Empty list pattern
        let empty_pattern = Pattern::List(vec![]);
        assert!(match_pattern(&empty_pattern, &Value::List(vec![])).is_some());
        assert!(match_pattern(&empty_pattern, &list_value).is_none());

        // Wrong list length should not match
        let short_pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
        assert!(match_pattern(&short_pattern, &list_value).is_none());
    }

    // Test 6: Rest Pattern Matching
    #[test]
    fn test_rest_pattern_matching() {
        let list_value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);

        // Rest pattern at end
        let rest_pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Rest
        ]);

        let binding = match_pattern(&rest_pattern, &list_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "first");
        assert!(values_equal(&bindings[0].1, &Value::Int(1)));

        // Named rest pattern
        let named_rest_pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::RestNamed("middle".to_string()),
            Pattern::Identifier("last".to_string())
        ]);

        let binding = match_pattern(&named_rest_pattern, &list_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 3);

        let first_binding = bindings.iter().find(|(name, _)| name == "first").unwrap();
        let middle_binding = bindings.iter().find(|(name, _)| name == "middle").unwrap();
        let last_binding = bindings.iter().find(|(name, _)| name == "last").unwrap();

        assert!(values_equal(&first_binding.1, &Value::Int(1)));
        assert!(values_equal(&last_binding.1, &Value::Int(4)));
        // Middle should contain [2, 3]
        if let Value::List(middle_values) = &middle_binding.1 {
            assert_eq!(middle_values.len(), 2);
            assert!(values_equal(&middle_values[0], &Value::Int(2)));
            assert!(values_equal(&middle_values[1], &Value::Int(3)));
        } else {
            panic!("Expected middle binding to be a list");
        }
    }

    // Test 7: OR Pattern Matching
    #[test]
    fn test_or_pattern_matching() {
        let or_pattern = Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Literal(Literal::String("test".to_string()))
        ]);

        // Should match first option
        assert!(match_pattern(&or_pattern, &Value::Int(1)).is_some());

        // Should match second option
        assert!(match_pattern(&or_pattern, &Value::Int(2)).is_some());

        // Should match third option
        assert!(match_pattern(&or_pattern, &Value::String("test".to_string())).is_some());

        // Should not match non-matching values
        assert!(match_pattern(&or_pattern, &Value::Int(3)).is_none());
        assert!(match_pattern(&or_pattern, &Value::String("other".to_string())).is_none());
    }

    // Test 8: Option Pattern Matching (Some/None)
    #[test]
    fn test_option_pattern_matching() {
        // Some pattern matching
        let some_value = Value::Object({
            let mut map = HashMap::new();
            map.insert("Some".to_string(), Value::Int(42));
            map
        });

        let some_pattern = Pattern::Some(Box::new(Pattern::Identifier("value".to_string())));
        let binding = match_pattern(&some_pattern, &some_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "value");
        assert!(values_equal(&bindings[0].1, &Value::Int(42)));

        // None pattern matching
        let none_value = Value::Unit; // Assuming None is represented as Unit
        let none_pattern = Pattern::None;
        assert!(match_pattern(&none_pattern, &none_value).is_some());

        // None should not match Some
        assert!(match_pattern(&none_pattern, &some_value).is_none());
    }

    // Test 9: Struct Pattern Matching
    #[test]
    fn test_struct_pattern_matching() {
        let struct_value = Value::Object({
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String("Alice".to_string()));
            map.insert("age".to_string(), Value::Int(30));
            map.insert("active".to_string(), Value::Bool(true));
            map
        });

        let struct_pattern = Pattern::Struct {
            name: "Person".to_string(),
            fields: vec![
                StructPatternField {
                    name: "name".to_string(),
                    pattern: Pattern::Identifier("person_name".to_string()),
                    default_value: None,
                },
                StructPatternField {
                    name: "age".to_string(),
                    pattern: Pattern::Identifier("person_age".to_string()),
                    default_value: None,
                }
            ]
        };

        let binding = match_pattern(&struct_pattern, &struct_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 2);

        let name_binding = bindings.iter().find(|(name, _)| name == "person_name").unwrap();
        let age_binding = bindings.iter().find(|(name, _)| name == "person_age").unwrap();

        assert!(values_equal(&name_binding.1, &Value::String("Alice".to_string())));
        assert!(values_equal(&age_binding.1, &Value::Int(30)));
    }

    // Test 10: Range Pattern Matching Edge Cases
    #[test]
    fn test_range_pattern_edge_cases() {
        // Test with range values
        let range_value = Value::Range {
            start: 1,
            end: 10,
            inclusive: true
        };

        let matching_range = Value::Range {
            start: 1,
            end: 10,
            inclusive: true
        };

        let non_matching_range = Value::Range {
            start: 1,
            end: 10,
            inclusive: false
        };

        assert!(values_equal(&range_value, &matching_range));
        assert!(!values_equal(&range_value, &non_matching_range));

        // Test edge case with empty patterns/values
        assert!(match_pattern(&Pattern::Wildcard, &Value::List(vec![])).is_some());
        assert!(match_pattern(&Pattern::List(vec![]), &Value::List(vec![])).is_some());
    }

    // Test 11: Complex Nested Pattern Matching
    #[test]
    fn test_nested_pattern_matching() {
        // Nested tuple with list
        let complex_value = Value::Tuple(vec![
            Value::String("outer".to_string()),
            Value::List(vec![Value::Int(1), Value::Int(2)]),
            Value::Tuple(vec![Value::Bool(true), Value::Char('x')])
        ]);

        let nested_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("outer_str".to_string()),
            Pattern::List(vec![
                Pattern::Identifier("first_int".to_string()),
                Pattern::Identifier("second_int".to_string())
            ]),
            Pattern::Tuple(vec![
                Pattern::Literal(Literal::Bool(true)),
                Pattern::Identifier("inner_char".to_string())
            ])
        ]);

        let binding = match_pattern(&nested_pattern, &complex_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 4);

        let outer_binding = bindings.iter().find(|(name, _)| name == "outer_str").unwrap();
        let first_binding = bindings.iter().find(|(name, _)| name == "first_int").unwrap();
        let second_binding = bindings.iter().find(|(name, _)| name == "second_int").unwrap();
        let char_binding = bindings.iter().find(|(name, _)| name == "inner_char").unwrap();

        assert!(values_equal(&outer_binding.1, &Value::String("outer".to_string())));
        assert!(values_equal(&first_binding.1, &Value::Int(1)));
        assert!(values_equal(&second_binding.1, &Value::Int(2)));
        assert!(values_equal(&char_binding.1, &Value::Char('x')));
    }

    // Test 12: Pattern Matching Failure Cases
    #[test]
    fn test_pattern_matching_failures() {
        // Type mismatch: expecting tuple, got integer
        let tuple_pattern = Pattern::Tuple(vec![Pattern::Identifier("x".to_string())]);
        assert!(match_pattern(&tuple_pattern, &Value::Int(42)).is_none());

        // Type mismatch: expecting list, got string
        let list_pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
        assert!(match_pattern(&list_pattern, &Value::String("test".to_string())).is_none());

        // Length mismatch: pattern expects 3 elements, value has 2
        let long_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
            Pattern::Identifier("c".to_string())
        ]);
        let short_tuple = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        assert!(match_pattern(&long_pattern, &short_tuple).is_none());

        // Literal mismatch
        let literal_pattern = Pattern::Literal(Literal::Integer(42));
        assert!(match_pattern(&literal_pattern, &Value::Int(43)).is_none());
        assert!(match_pattern(&literal_pattern, &Value::String("42".to_string())).is_none());
    }
}
