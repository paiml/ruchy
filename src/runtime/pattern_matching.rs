//! Shared pattern matching utilities
//! Extracted to reduce duplication across interpreter and REPL
use crate::frontend::ast::{Literal, Pattern};
use crate::runtime::Value;
use std::collections::HashMap;

#[cfg(test)]
use std::sync::Arc;
///
/// let value = `Value::Integer(42)`;
/// let pattern = `Literal::Integer(42, None)`;
/// `assert!(match_literal_pattern(&value`, &pattern));
///
/// let pattern2 = `Literal::Integer(43, None)`;
/// `assert!(!match_literal_pattern(&value`, &pattern2));
/// ```
pub fn match_literal_pattern(value: &Value, literal: &Literal) -> bool {
    match (value, literal) {
        (Value::Nil, Literal::Unit) => true,
        (Value::Integer(v), Literal::Integer(p, _)) => v == p,
        (Value::Float(v), Literal::Float(p)) => (v - p).abs() < f64::EPSILON,
        (Value::String(v), Literal::String(p)) => &**v == p,
        (Value::Bool(v), Literal::Bool(p)) => v == p,
        // Char variant not available in current Value enum
        _ => false,
    }
}
/// Helper function to match collection patterns (tuple or list)
fn match_collection_patterns(
    patterns: &[Pattern],
    values: &[Value],
) -> Option<Vec<(String, Value)>> {
    // Check if there's a rest pattern
    let rest_position = patterns
        .iter()
        .position(|p| matches!(p, Pattern::Rest | Pattern::RestNamed(_)));

    if let Some(rest_idx) = rest_position {
        match_patterns_with_rest(patterns, values, rest_idx)
    } else {
        match_patterns_without_rest(patterns, values)
    }
}

/// Match patterns that contain a rest pattern (complexity: 7)
fn match_patterns_with_rest(
    patterns: &[Pattern],
    values: &[Value],
    rest_idx: usize,
) -> Option<Vec<(String, Value)>> {
    let before_rest = &patterns[..rest_idx];
    let after_rest = &patterns[rest_idx + 1..];

    // Check if we have enough values for non-rest patterns
    if values.len() < before_rest.len() + after_rest.len() {
        return None;
    }

    let mut bindings = Vec::new();

    // Match patterns before rest
    bindings.extend(match_pattern_sequence(
        before_rest,
        &values[..before_rest.len()],
    )?);

    // Handle the rest pattern
    let rest_start = before_rest.len();
    let rest_end = values.len() - after_rest.len();
    let rest_values = &values[rest_start..rest_end];

    if let Pattern::RestNamed(name) = &patterns[rest_idx] {
        bindings.push((name.clone(), Value::from_array(rest_values.to_vec())));
    }
    // Pattern::Rest doesn't bind anything

    // Match patterns after rest
    bindings.extend(match_pattern_sequence(after_rest, &values[rest_end..])?);

    Some(bindings)
}

/// Match patterns without rest pattern (complexity: 3)
fn match_patterns_without_rest(
    patterns: &[Pattern],
    values: &[Value],
) -> Option<Vec<(String, Value)>> {
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
        Pattern::List(patterns) => match_list_pattern_helper(patterns, value),
        Pattern::Struct { fields, .. } => match_struct_pattern_helper(fields, value),
        Pattern::Or(patterns) => match_or_pattern_helper(patterns, value),
        Pattern::Rest | Pattern::RestNamed(_) => {
            // Rest patterns when used standalone act like wildcards
            // When within collections, they're handled by match_collection_patterns
            Some(vec![])
        }
        Pattern::AtBinding { name, pattern } => {
            // @ bindings both bind the value and match the pattern
            if let Some(mut bindings) = match_pattern(pattern, value) {
                bindings.push((name.clone(), value.clone()));
                Some(bindings)
            } else {
                None
            }
        }
        Pattern::Range {
            start,
            end,
            inclusive,
        } => match_range_pattern_helper(start, end, *inclusive, value),
        Pattern::QualifiedName(_) => None,
        Pattern::TupleVariant { path: _, patterns } => {
            // For enum tuple variants, match like tuple destructuring
            match_tuple_pattern_helper(patterns, value)
        }
        Pattern::Some(inner_pattern) => match_some_pattern_helper(inner_pattern, value),
        Pattern::None => match_none_pattern_helper(value),
        Pattern::Ok(inner_pattern) => match_ok_pattern_helper(inner_pattern, value),
        Pattern::Err(inner_pattern) => match_err_pattern_helper(inner_pattern, value),
        Pattern::WithDefault { pattern, .. } => {
            // For default patterns, we match the inner pattern
            // Default handling is done at the destructuring level
            match_pattern(pattern, value)
        }
        Pattern::Mut(inner_pattern) => {
            // Mut patterns match the inner pattern
            // Mutability is handled at the environment binding level
            match_pattern(inner_pattern, value)
        }
    }
}
/// Check if two values are equal (for pattern matching)
pub fn values_equal(v1: &Value, v2: &Value) -> bool {
    match (v1, v2) {
        (Value::Nil, Value::Nil) => true,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        // (Value::Char(a), Value::Char(b)) => a == b, // Char variant not available in current Value enum
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Tuple(a), Value::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(f1), Value::Object(f2)) => {
            f1.len() == f2.len()
                && f1
                    .iter()
                    .all(|(k, v)| f2.get(k).is_some_and(|v2| values_equal(v, v2)))
        }
        (
            Value::Range {
                start: s1,
                end: e1,
                inclusive: i1,
            },
            Value::Range {
                start: s2,
                end: e2,
                inclusive: i2,
            },
        ) => values_equal(s1, s2) && values_equal(e1, e2) && i1 == i2,
        (
            Value::EnumVariant {
                variant_name: n1,
                data: d1,
                ..
            },
            Value::EnumVariant {
                variant_name: n2,
                data: d2,
                ..
            },
        ) => {
            n1 == n2
                && match (d1, d2) {
                    (Some(v1), Some(v2)) => {
                        v1.len() == v2.len()
                            && v1.iter().zip(v2.iter()).all(|(x, y)| values_equal(x, y))
                    }
                    (None, None) => true,
                    _ => false,
                }
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
    if let Value::Array(values) = value {
        match_collection_patterns(patterns, values)
    } else {
        None
    }
}
/// Helper for matching struct patterns (complexity: 8)
fn match_struct_pattern_helper(
    fields: &[crate::frontend::ast::StructPatternField],
    value: &Value,
) -> Option<Vec<(String, Value)>> {
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
fn match_range_pattern_helper(
    start: &Pattern,
    end: &Pattern,
    inclusive: bool,
    value: &Value,
) -> Option<Vec<(String, Value)>> {
    if let Value::Integer(val) = value {
        let start_val = if let Pattern::Literal(Literal::Integer(n, _)) = start {
            *n
        } else {
            return None;
        };
        let end_val = if let Pattern::Literal(Literal::Integer(n, _)) = end {
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
fn match_some_pattern_helper(
    inner_pattern: &Pattern,
    value: &Value,
) -> Option<Vec<(String, Value)>> {
    if let Value::EnumVariant {
        variant_name, data, ..
    } = value
    {
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

/// Helper for matching Ok patterns (complexity: 3)
fn match_ok_pattern_helper(inner_pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
    let fields = extract_object_fields(value)?;

    if !is_ok_type(fields) {
        return None;
    }

    let data = extract_ok_data(fields)?;
    if data.is_empty() {
        return None;
    }

    match_pattern(inner_pattern, &data[0])
}

fn extract_object_fields(value: &Value) -> Option<&HashMap<String, Value>> {
    if let Value::Object(fields) = value {
        Some(fields)
    } else {
        None
    }
}

fn is_ok_type(fields: &HashMap<String, Value>) -> bool {
    if let Some(Value::String(type_str)) = fields.get("type") {
        &**type_str == "Ok"
    } else {
        false
    }
}

fn extract_ok_data(fields: &HashMap<String, Value>) -> Option<&[Value]> {
    if let Some(Value::Array(data)) = fields.get("data") {
        Some(data)
    } else {
        None
    }
}

/// Helper for matching Err patterns (complexity: 6)
fn match_err_pattern_helper(
    inner_pattern: &Pattern,
    value: &Value,
) -> Option<Vec<(String, Value)>> {
    // Err(x) creates an Object: {data: [x], __type: "Message", type: "Err"}
    if let Value::Object(fields) = value {
        if let Some(Value::String(type_str)) = fields.get("type") {
            if &**type_str == "Err" {
                if let Some(Value::Array(data)) = fields.get("data") {
                    if !data.is_empty() {
                        return match_pattern(inner_pattern, &data[0]);
                    }
                }
            }
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
            Value::Nil,
            Value::Integer(42),
            Value::Float(3.15),
            Value::from_string("test".to_string()),
            Value::Bool(true),
            // Value::Char('a'), // Char variant not available in current Value enum
            Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)])),
            Value::Tuple(Arc::from(vec![
                Value::from_string("hello".to_string()),
                Value::Integer(10),
            ])),
        ]
    }

    // Test 1: Literal Pattern Matching
    #[test]
    fn test_literal_pattern_matching() {
        // Integer literal matching
        assert!(match_literal_pattern(
            &Value::Integer(42),
            &Literal::Integer(42, None)
        ));
        assert!(!match_literal_pattern(
            &Value::Integer(42),
            &Literal::Integer(43, None)
        ));

        // Float literal matching with epsilon
        assert!(match_literal_pattern(
            &Value::Float(3.15),
            &Literal::Float(3.15)
        ));
        assert!(match_literal_pattern(
            &Value::Float(1.0),
            &Literal::Float(1.0 + f64::EPSILON / 2.0)
        ));
        assert!(!match_literal_pattern(
            &Value::Float(1.0),
            &Literal::Float(1.5)
        ));

        // String literal matching
        assert!(match_literal_pattern(
            &Value::from_string("hello".to_string()),
            &Literal::String("hello".to_string())
        ));
        assert!(!match_literal_pattern(
            &Value::from_string("hello".to_string()),
            &Literal::String("world".to_string())
        ));

        // Boolean literal matching
        assert!(match_literal_pattern(
            &Value::Bool(true),
            &Literal::Bool(true)
        ));
        assert!(!match_literal_pattern(
            &Value::Bool(true),
            &Literal::Bool(false)
        ));

        // Character literal matching
        // assert!(match_literal_pattern(&Value::Char('a'), &Literal::Char('a'))); // Char variant not available
        // assert!(!match_literal_pattern(&Value::Char('a'), &Literal::Char('b'))); // Char variant not available

        // Unit literal matching
        assert!(match_literal_pattern(&Value::Nil, &Literal::Unit));

        // Type mismatch should not match
        assert!(!match_literal_pattern(
            &Value::Integer(42),
            &Literal::String("42".to_string())
        ));
        assert!(!match_literal_pattern(
            &Value::from_string("true".to_string()),
            &Literal::Bool(true)
        ));
    }

    // Test 2: Values Equality Function
    #[test]
    fn test_values_equal() {
        // Basic type equality
        assert!(values_equal(&Value::Integer(42), &Value::Integer(42)));
        assert!(!values_equal(&Value::Integer(42), &Value::Integer(43)));

        // Float equality with epsilon
        assert!(values_equal(&Value::Float(3.15), &Value::Float(3.15)));
        assert!(values_equal(
            &Value::Float(1.0),
            &Value::Float(1.0 + f64::EPSILON / 2.0)
        ));
        assert!(!values_equal(&Value::Float(1.0), &Value::Float(1.5)));

        // String equality
        assert!(values_equal(
            &Value::from_string("test".to_string()),
            &Value::from_string("test".to_string())
        ));
        assert!(!values_equal(
            &Value::from_string("test".to_string()),
            &Value::from_string("other".to_string())
        ));

        // List equality (recursive)
        let list1 = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        let list2 = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]));
        let list3 = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("other".to_string()),
        ]));
        assert!(values_equal(&list1, &list2));
        assert!(!values_equal(&list1, &list3));

        // Tuple equality (recursive)
        let tuple1 = Value::Tuple(Arc::from(vec![Value::Bool(true), Value::Integer(42)]));
        let tuple2 = Value::Tuple(Arc::from(vec![Value::Bool(true), Value::Integer(42)]));
        let tuple3 = Value::Tuple(Arc::from(vec![Value::Bool(false), Value::Integer(42)]));
        assert!(values_equal(&tuple1, &tuple2));
        assert!(!values_equal(&tuple1, &tuple3));

        // Different lengths should not be equal
        let short_list = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let long_list = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(!values_equal(&short_list, &long_list));

        // Different types should not be equal
        assert!(!values_equal(
            &Value::Integer(42),
            &Value::from_string("42".to_string())
        ));
        assert!(!values_equal(
            &Value::from_array(vec![]),
            &Value::Tuple(Arc::from(vec![]))
        ));
    }

    // Test 3: Simple Pattern Matching
    #[test]
    fn test_simple_pattern_matching() {
        // Wildcard pattern should match anything
        assert!(match_pattern(&Pattern::Wildcard, &Value::Integer(42)).is_some());
        assert!(
            match_pattern(&Pattern::Wildcard, &Value::from_string("test".to_string())).is_some()
        );
        assert!(match_pattern(&Pattern::Wildcard, &Value::Nil).is_some());

        // Variable pattern should bind value
        let binding = match_pattern(&Pattern::Identifier("x".to_string()), &Value::Integer(42));
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert!(values_equal(&bindings[0].1, &Value::Integer(42)));

        // Literal pattern matching
        let literal_pattern = Pattern::Literal(Literal::Integer(42, None));
        assert!(match_pattern(&literal_pattern, &Value::Integer(42)).is_some());
        assert!(match_pattern(&literal_pattern, &Value::Integer(43)).is_none());
    }

    // Test 4: Tuple Pattern Matching
    #[test]
    fn test_tuple_pattern_matching() {
        let tuple_value = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
            Value::Bool(true),
        ]));

        // Exact tuple match
        let tuple_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Identifier("s".to_string()),
            Pattern::Literal(Literal::Bool(true)),
        ]);

        let binding = match_pattern(&tuple_pattern, &tuple_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "s");
        assert!(values_equal(
            &bindings[0].1,
            &Value::from_string("test".to_string())
        ));

        // Wrong tuple length should not match
        let wrong_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Identifier("s".to_string()),
        ]);
        assert!(match_pattern(&wrong_pattern, &tuple_value).is_none());

        // Wrong element values should not match
        let mismatch_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(2, None)),
            Pattern::Identifier("s".to_string()),
            Pattern::Literal(Literal::Bool(true)),
        ]);
        assert!(match_pattern(&mismatch_pattern, &tuple_value).is_none());
    }

    // Test 5: List Pattern Matching
    #[test]
    fn test_list_pattern_matching() {
        let list_value = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        // Exact list match with variables
        let list_pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Literal(Literal::Integer(2, None)),
            Pattern::Identifier("last".to_string()),
        ]);

        let binding = match_pattern(&list_pattern, &list_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 2);

        // Check bindings (order may vary, so check both)
        let first_binding = bindings.iter().find(|(name, _)| name == "first").unwrap();
        let last_binding = bindings.iter().find(|(name, _)| name == "last").unwrap();
        assert!(values_equal(&first_binding.1, &Value::Integer(1)));
        assert!(values_equal(&last_binding.1, &Value::Integer(3)));

        // Empty list pattern
        let empty_pattern = Pattern::List(vec![]);
        assert!(match_pattern(&empty_pattern, &Value::from_array(vec![])).is_some());
        assert!(match_pattern(&empty_pattern, &list_value).is_none());

        // Wrong list length should not match
        let short_pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
        assert!(match_pattern(&short_pattern, &list_value).is_none());
    }

    // Test 6: Rest Pattern Matching
    #[test]
    fn test_rest_pattern_matching() {
        let list_value = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]));

        // Rest pattern at end
        let rest_pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Rest,
        ]);

        let binding = match_pattern(&rest_pattern, &list_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "first");
        assert!(values_equal(&bindings[0].1, &Value::Integer(1)));

        // Named rest pattern
        let named_rest_pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::RestNamed("middle".to_string()),
            Pattern::Identifier("last".to_string()),
        ]);

        let binding = match_pattern(&named_rest_pattern, &list_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 3);

        let first_binding = bindings.iter().find(|(name, _)| name == "first").unwrap();
        let middle_binding = bindings.iter().find(|(name, _)| name == "middle").unwrap();
        let last_binding = bindings.iter().find(|(name, _)| name == "last").unwrap();

        assert!(values_equal(&first_binding.1, &Value::Integer(1)));
        assert!(values_equal(&last_binding.1, &Value::Integer(4)));
        // Middle should contain [2, 3]
        if let Value::Array(middle_values) = &middle_binding.1 {
            assert_eq!(middle_values.len(), 2);
            assert!(values_equal(&middle_values[0], &Value::Integer(2)));
            assert!(values_equal(&middle_values[1], &Value::Integer(3)));
        } else {
            panic!("Expected middle binding to be a list");
        }
    }

    // Test 7: OR Pattern Matching
    #[test]
    fn test_or_pattern_matching() {
        let or_pattern = Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
            Pattern::Literal(Literal::String("test".to_string())),
        ]);

        // Should match first option
        assert!(match_pattern(&or_pattern, &Value::Integer(1)).is_some());

        // Should match second option
        assert!(match_pattern(&or_pattern, &Value::Integer(2)).is_some());

        // Should match third option
        assert!(match_pattern(&or_pattern, &Value::from_string("test".to_string())).is_some());

        // Should not match non-matching values
        assert!(match_pattern(&or_pattern, &Value::Integer(3)).is_none());
        assert!(match_pattern(&or_pattern, &Value::from_string("other".to_string())).is_none());
    }

    // Test 8: Option Pattern Matching (Some/None)
    #[test]
    fn test_option_pattern_matching() {
        // Some pattern matching - should use EnumVariant representation
        let some_value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };

        let some_pattern = Pattern::Some(Box::new(Pattern::Identifier("value".to_string())));
        let binding = match_pattern(&some_pattern, &some_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "value");
        assert!(values_equal(&bindings[0].1, &Value::Integer(42)));

        // None pattern matching - should use EnumVariant representation
        let none_value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };
        let none_pattern = Pattern::None;
        assert!(match_pattern(&none_pattern, &none_value).is_some());

        // None should not match Some
        assert!(match_pattern(&none_pattern, &some_value).is_none());
    }

    // Test 9: Struct Pattern Matching
    #[test]
    fn test_struct_pattern_matching() {
        let struct_value = Value::Object(Arc::new({
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::from_string("Alice".to_string()));
            map.insert("age".to_string(), Value::Integer(30));
            map.insert("active".to_string(), Value::Bool(true));
            map
        }));

        let struct_pattern = Pattern::Struct {
            name: "Person".to_string(),
            fields: vec![
                StructPatternField {
                    name: "name".to_string(),
                    pattern: Some(Pattern::Identifier("person_name".to_string())),
                },
                StructPatternField {
                    name: "age".to_string(),
                    pattern: Some(Pattern::Identifier("person_age".to_string())),
                },
            ],
            has_rest: false,
        };

        let binding = match_pattern(&struct_pattern, &struct_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 2);

        let name_binding = bindings
            .iter()
            .find(|(name, _)| name == "person_name")
            .unwrap();
        let age_binding = bindings
            .iter()
            .find(|(name, _)| name == "person_age")
            .unwrap();

        assert!(values_equal(
            &name_binding.1,
            &Value::from_string("Alice".to_string())
        ));
        assert!(values_equal(&age_binding.1, &Value::Integer(30)));
    }

    // Test 10: Range Pattern Matching Edge Cases
    #[test]
    fn test_range_pattern_edge_cases() {
        // Test with range values
        let range_value = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };

        let matching_range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };

        let non_matching_range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };

        assert!(values_equal(&range_value, &matching_range));
        assert!(!values_equal(&range_value, &non_matching_range));

        // Test edge case with empty patterns/values
        assert!(match_pattern(&Pattern::Wildcard, &Value::from_array(vec![])).is_some());
        assert!(match_pattern(&Pattern::List(vec![]), &Value::from_array(vec![])).is_some());
    }

    // Test 11: Complex Nested Pattern Matching
    #[test]
    fn test_nested_pattern_matching() {
        // Nested tuple with list
        let complex_value = Value::Tuple(Arc::from(vec![
            Value::from_string("outer".to_string()),
            Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)])),
            Value::Tuple(Arc::from(vec![Value::Bool(true)])),
        ]));

        let nested_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("outer_str".to_string()),
            Pattern::List(vec![
                Pattern::Identifier("first_int".to_string()),
                Pattern::Identifier("second_int".to_string()),
            ]),
            Pattern::Tuple(vec![Pattern::Literal(Literal::Bool(true))]),
        ]);

        let binding = match_pattern(&nested_pattern, &complex_value);
        assert!(binding.is_some());
        let bindings = binding.unwrap();
        assert_eq!(bindings.len(), 3); // Only 3 bindings now

        let outer_binding = bindings
            .iter()
            .find(|(name, _)| name == "outer_str")
            .unwrap();
        let first_binding = bindings
            .iter()
            .find(|(name, _)| name == "first_int")
            .unwrap();
        let second_binding = bindings
            .iter()
            .find(|(name, _)| name == "second_int")
            .unwrap();

        assert!(values_equal(
            &outer_binding.1,
            &Value::from_string("outer".to_string())
        ));
        assert!(values_equal(&first_binding.1, &Value::Integer(1)));
        assert!(values_equal(&second_binding.1, &Value::Integer(2)));
    }

    // Test 12: Pattern Matching Failure Cases
    #[test]
    fn test_pattern_matching_failures() {
        // Type mismatch: expecting tuple, got integer
        let tuple_pattern = Pattern::Tuple(vec![Pattern::Identifier("x".to_string())]);
        assert!(match_pattern(&tuple_pattern, &Value::Integer(42)).is_none());

        // Type mismatch: expecting list, got string
        let list_pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
        assert!(match_pattern(&list_pattern, &Value::from_string("test".to_string())).is_none());

        // Length mismatch: pattern expects 3 elements, value has 2
        let long_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
            Pattern::Identifier("c".to_string()),
        ]);
        let short_tuple = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        assert!(match_pattern(&long_pattern, &short_tuple).is_none());

        // Literal mismatch
        let literal_pattern = Pattern::Literal(Literal::Integer(42, None));
        assert!(match_pattern(&literal_pattern, &Value::Integer(43)).is_none());
        assert!(match_pattern(&literal_pattern, &Value::from_string("42".to_string())).is_none());
    }
}
