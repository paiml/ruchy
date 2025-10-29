//! Pattern matching evaluation module
//!
//! This module handles all pattern matching operations including match expressions,
//! pattern guards, destructuring, and pattern binding.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Literal, Pattern, StructPatternField};
use crate::runtime::{InterpreterError, Value};

/// Try to match a pattern against a value, returning bindings if successful
///
/// # Complexity
/// Cyclomatic complexity: 10 (at Toyota Way limit - added Some/None support)
pub fn try_pattern_match(
    pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    match pattern {
        Pattern::Wildcard => Ok(Some(vec![])),
        Pattern::Literal(lit) => {
            if match_literal_pattern(lit, value, eval_literal)? {
                Ok(Some(vec![]))
            } else {
                Ok(None)
            }
        }
        Pattern::Identifier(name) => {
            // Always matches and binds the value to the identifier
            Ok(Some(vec![(name.clone(), value.clone())]))
        }
        Pattern::Tuple(patterns) => try_match_tuple_pattern(patterns, value, eval_literal),
        Pattern::List(patterns) => try_match_list_pattern(patterns, value, eval_literal),
        Pattern::Or(patterns) => try_match_or_pattern(patterns, value, eval_literal),
        Pattern::Range {
            start,
            end,
            inclusive,
        } => {
            if match_range_pattern(start, end, *inclusive, value)? {
                Ok(Some(vec![]))
            } else {
                Ok(None)
            }
        }
        Pattern::AtBinding { pattern, name } => {
            if let Some(mut bindings) = try_pattern_match(pattern, value, eval_literal)? {
                bindings.push((name.clone(), value.clone()));
                Ok(Some(bindings))
            } else {
                Ok(None)
            }
        }
        Pattern::Some(inner_pattern) => try_match_some_pattern(inner_pattern, value, eval_literal),
        Pattern::None => try_match_none_pattern(value),
        Pattern::Ok(inner_pattern) => try_match_ok_pattern(inner_pattern, value, eval_literal),
        Pattern::Err(inner_pattern) => try_match_err_pattern(inner_pattern, value, eval_literal),
        Pattern::QualifiedName(path) => try_match_qualified_name_pattern(path, value),
        Pattern::TupleVariant { path, patterns } => {
            try_match_tuple_variant_pattern(path, patterns, value, eval_literal)
        }
        Pattern::Struct { name, fields, .. } => {
            try_match_struct_pattern(name, fields, value, eval_literal)
        }
        _ => Ok(None), // Other patterns not yet implemented
    }
}

/// Check if a pattern matches a value (legacy compatibility)
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
pub fn pattern_matches(
    pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<bool, InterpreterError> {
    Ok(try_pattern_match(pattern, value, eval_literal)?.is_some())
}

/// Try to match a tuple pattern
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn try_match_tuple_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    if let Value::Tuple(tuple_values) = value {
        if patterns.len() != tuple_values.len() {
            return Ok(None);
        }

        let mut all_bindings = Vec::new();
        for (pattern, val) in patterns.iter().zip(tuple_values.iter()) {
            if let Some(bindings) = try_pattern_match(pattern, val, &eval_literal)? {
                all_bindings.extend(bindings);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(all_bindings))
    } else {
        Ok(None)
    }
}

/// Try to match a list pattern
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn try_match_list_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    if let Value::Array(array_values) = value {
        if patterns.len() != array_values.len() {
            return Ok(None);
        }

        let mut all_bindings = Vec::new();
        for (pattern, val) in patterns.iter().zip(array_values.iter()) {
            if let Some(bindings) = try_pattern_match(pattern, val, &eval_literal)? {
                all_bindings.extend(bindings);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(all_bindings))
    } else {
        Ok(None)
    }
}

/// Try to match an or pattern
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn try_match_or_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    for pattern in patterns {
        if let Some(bindings) = try_pattern_match(pattern, value, &eval_literal)? {
            return Ok(Some(bindings));
        }
    }
    Ok(None)
}

/// Try to match a Some pattern
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn try_match_some_pattern(
    inner_pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    if let Value::EnumVariant { variant_name, data, .. } = value {
        if variant_name == "Some" {
            if let Some(values) = data {
                if values.len() == 1 {
                    return try_pattern_match(inner_pattern, &values[0], eval_literal);
                }
            }
        }
    }
    Ok(None)
}

/// Try to match a None pattern
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn try_match_none_pattern(value: &Value) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    if let Value::EnumVariant { variant_name, data, .. } = value {
        if variant_name == "None" && data.is_none() {
            return Ok(Some(vec![]));
        }
    }
    Ok(None)
}

/// Try to match an Ok pattern
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn try_match_ok_pattern(
    inner_pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    // Ok(x) creates an Object: {data: [x], __type: "Message", type: "Ok"}
    if let Value::Object(fields) = value {
        if let Some(Value::String(type_str)) = fields.get("type") {
            if &**type_str == "Ok" {
                if let Some(Value::Array(data)) = fields.get("data") {
                    if !data.is_empty() {
                        return try_pattern_match(inner_pattern, &data[0], eval_literal);
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Try to match an Err pattern
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn try_match_err_pattern(
    inner_pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    // Err(x) creates an Object: {data: [x], __type: "Message", type: "Err"}
    if let Value::Object(fields) = value {
        if let Some(Value::String(type_str)) = fields.get("type") {
            if &**type_str == "Err" {
                if let Some(Value::Array(data)) = fields.get("data") {
                    if !data.is_empty() {
                        return try_pattern_match(inner_pattern, &data[0], eval_literal);
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Try to match a qualified name pattern (unit enum variant)
///
/// Matches patterns like `Status::Success` against `EnumVariant` values
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn try_match_qualified_name_pattern(
    path: &[String],
    value: &Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    if let Value::EnumVariant { variant_name, data, .. } = value {
        // Check if variant is unit (no data) and name matches
        if data.is_none() && path.last() == Some(variant_name) {
            return Ok(Some(vec![]));
        }
    }
    Ok(None)
}

/// Try to match a tuple variant pattern (enum with data)
///
/// Matches patterns like `Response::Error(msg)` against `EnumVariant` values
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn try_match_tuple_variant_pattern(
    path: &[String],
    patterns: &[Pattern],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    if let Value::EnumVariant { variant_name, data, .. } = value {
        // Check if variant name matches the last component of path
        if path.last() == Some(variant_name) {
            // Check if variant has data
            if let Some(variant_data) = data {
                // Match the number of patterns with data elements
                if patterns.len() != variant_data.len() {
                    return Ok(None);
                }

                // Try to match each pattern against corresponding data element
                let mut all_bindings = Vec::new();
                for (pattern, data_val) in patterns.iter().zip(variant_data.iter()) {
                    if let Some(bindings) = try_pattern_match(pattern, data_val, eval_literal)? {
                        all_bindings.extend(bindings);
                    } else {
                        return Ok(None);
                    }
                }
                return Ok(Some(all_bindings));
            }
        }
    }
    Ok(None)
}

/// Match a literal pattern
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn match_literal_pattern(
    lit: &Literal,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<bool, InterpreterError> {
    let lit_value = eval_literal(lit);
    Ok(lit_value == *value)
}

/// Match a tuple pattern (legacy)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn match_tuple_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: impl Fn(&Literal) -> Value,
) -> Result<bool, InterpreterError> {
    if let Value::Tuple(elements) = value {
        match_sequence_patterns(patterns, elements, eval_literal)
    } else {
        Ok(false)
    }
}

/// Match a list pattern (legacy)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn match_list_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: impl Fn(&Literal) -> Value,
) -> Result<bool, InterpreterError> {
    if let Value::Array(elements) = value {
        match_sequence_patterns(patterns, elements, eval_literal)
    } else {
        Ok(false)
    }
}

/// Match a sequence of patterns against elements
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn match_sequence_patterns(
    patterns: &[Pattern],
    elements: &[Value],
    eval_literal: impl Fn(&Literal) -> Value,
) -> Result<bool, InterpreterError> {
    if patterns.len() != elements.len() {
        return Ok(false);
    }
    for (pat, val) in patterns.iter().zip(elements.iter()) {
        if !pattern_matches(pat, val, &eval_literal)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Match an or pattern (legacy)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn match_or_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: impl Fn(&Literal) -> Value,
) -> Result<bool, InterpreterError> {
    for pat in patterns {
        if pattern_matches(pat, value, &eval_literal)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Match a range pattern
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn match_range_pattern(
    start: &Pattern,
    end: &Pattern,
    inclusive: bool,
    value: &Value,
) -> Result<bool, InterpreterError> {
    if let Value::Integer(i) = value {
        let start_val = extract_integer_from_pattern(start)?;
        let end_val = extract_integer_from_pattern(end)?;

        if inclusive {
            Ok(*i >= start_val && *i <= end_val)
        } else {
            Ok(*i >= start_val && *i < end_val)
        }
    } else {
        Ok(false)
    }
}

/// Extract an integer from a literal pattern
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn extract_integer_from_pattern(pattern: &Pattern) -> Result<i64, InterpreterError> {
    if let Pattern::Literal(Literal::Integer(val, _)) = pattern {
        Ok(*val)
    } else {
        Err(InterpreterError::RuntimeError(
            "Range pattern requires integer literals".to_string(),
        ))
    }
}

/// Try to match a struct pattern
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn try_match_struct_pattern(
    struct_name: &str,
    field_patterns: &[StructPatternField],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    // Support both Value::Struct and Value::Object (duck typing)
    let struct_fields = match value {
        Value::Struct { name, fields } => {
            // Check struct name matches (case-sensitive)
            if name != struct_name {
                return Ok(None);
            }
            fields
        }
        Value::Object(fields) => {
            // Objects can match struct patterns (duck typing)
            fields
        }
        _ => return Ok(None),
    };

    // Match each field pattern
    let mut all_bindings = Vec::new();
    for field_pattern in field_patterns {
        // Get the field value from the struct
        let field_value = match struct_fields.get(&field_pattern.name) {
            Some(v) => v,
            None => return Ok(None), // Field not found
        };

        // Match the field pattern (if specified)
        if let Some(ref pattern) = field_pattern.pattern {
            if let Some(bindings) = try_pattern_match(pattern, field_value, eval_literal)? {
                all_bindings.extend(bindings);
            } else {
                return Ok(None);
            }
        } else {
            // No pattern specified, bind field name directly
            all_bindings.push((field_pattern.name.clone(), field_value.clone()));
        }
    }

    Ok(Some(all_bindings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn test_eval_literal(lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i, _) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Unit => Value::Nil,
            _ => Value::Nil,
        }
    }

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard;
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_identifier_pattern() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(42));
    }

    #[test]
    fn test_literal_pattern_match() {
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_literal_pattern_no_match() {
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let value = Value::Integer(43);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_tuple_pattern() {
        let patterns = vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
        ];
        let pattern = Pattern::Tuple(patterns);
        let value = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(1));
        assert_eq!(bindings[1].0, "y");
        assert_eq!(bindings[1].1, Value::Integer(2));
    }

    #[test]
    fn test_range_pattern_inclusive() {
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
            inclusive: true,
        };

        let value = Value::Integer(3);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());

        let value = Value::Integer(5);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());

        let value = Value::Integer(6);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_or_pattern() {
        let patterns = vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
            Pattern::Literal(Literal::Integer(3, None)),
        ];
        let pattern = Pattern::Or(patterns);

        let value = Value::Integer(2);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());

        let value = Value::Integer(4);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_some_pattern_match() {
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(42));
    }

    #[test]
    fn test_some_pattern_no_match_on_none() {
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_none_pattern_match() {
        let pattern = Pattern::None;
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_none_pattern_no_match_on_some() {
        let pattern = Pattern::None;
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_qualified_name_unit_variant_match() {
        let pattern = Pattern::QualifiedName(vec!["Status".to_string(), "Success".to_string()]);
        let value = Value::EnumVariant {
            enum_name: "Status".to_string(),
            variant_name: "Success".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert!(bindings.is_empty()); // Unit variants have no bindings
    }

    #[test]
    fn test_qualified_name_unit_variant_no_match_wrong_name() {
        let pattern = Pattern::QualifiedName(vec!["Status".to_string(), "Success".to_string()]);
        let value = Value::EnumVariant {
            enum_name: "Status".to_string(),
            variant_name: "Failed".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_qualified_name_no_match_on_tuple_variant() {
        let pattern = Pattern::QualifiedName(vec!["Response".to_string(), "Error".to_string()]);
        let value = Value::EnumVariant {
            enum_name: "Response".to_string(),
            variant_name: "Error".to_string(),
            data: Some(vec![Value::from_string("failed".to_string())]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none()); // Should not match - has data
    }

    #[test]
    fn test_tuple_variant_single_element() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Response".to_string(), "Error".to_string()],
            patterns: vec![Pattern::Identifier("msg".to_string())],
        };
        let value = Value::EnumVariant {
            enum_name: "Response".to_string(),
            variant_name: "Error".to_string(),
            data: Some(vec![Value::from_string("failed".to_string())]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "msg");
        assert_eq!(bindings[0].1, Value::from_string("failed".to_string()));
    }

    #[test]
    fn test_tuple_variant_multiple_elements() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Point".to_string(), "Pos".to_string()],
            patterns: vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ],
        };
        let value = Value::EnumVariant {
            enum_name: "Point".to_string(),
            variant_name: "Pos".to_string(),
            data: Some(vec![Value::Integer(10), Value::Integer(20)]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(10));
        assert_eq!(bindings[1].0, "y");
        assert_eq!(bindings[1].1, Value::Integer(20));
    }

    #[test]
    fn test_tuple_variant_no_match_wrong_variant_name() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Message".to_string(), "Move".to_string()],
            patterns: vec![Pattern::Identifier("dir".to_string())],
        };
        let value = Value::EnumVariant {
            enum_name: "Message".to_string(),
            variant_name: "Quit".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_tuple_variant_no_match_arity_mismatch() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Point".to_string(), "Pos".to_string()],
            patterns: vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ],
        };
        let value = Value::EnumVariant {
            enum_name: "Point".to_string(),
            variant_name: "Pos".to_string(),
            data: Some(vec![Value::Integer(10)]), // Only 1 element, pattern expects 2
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn test_eval_literal(lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i, _) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Unit => Value::Nil,
            _ => Value::Nil,
        }
    }

    proptest! {
        /// Property: Wildcard pattern always matches any value
        #[test]
        fn prop_wildcard_always_matches(value: i64) {
            let pattern = Pattern::Wildcard;
            let val = Value::Integer(value);
            let result = try_pattern_match(&pattern, &val, &test_eval_literal).unwrap();
            prop_assert!(result.is_some());
            prop_assert_eq!(result.unwrap().len(), 0); // No bindings
        }

        /// Property: Identifier pattern always matches and binds
        #[test]
        fn prop_identifier_always_binds(name in "[a-z]{1,10}", value: i64) {
            let pattern = Pattern::Identifier(name.clone());
            let val = Value::Integer(value);
            let result = try_pattern_match(&pattern, &val, &test_eval_literal).unwrap();
            prop_assert!(result.is_some());
            let bindings = result.unwrap();
            prop_assert_eq!(bindings.len(), 1);
            prop_assert_eq!(&bindings[0].0, &name);
            prop_assert_eq!(&bindings[0].1, &Value::Integer(value));
        }

        /// Property: Literal pattern matches only exact values
        #[test]
        fn prop_literal_exact_match(target: i64, test: i64) {
            let pattern = Pattern::Literal(Literal::Integer(target, None));
            let val = Value::Integer(test);
            let result = try_pattern_match(&pattern, &val, &test_eval_literal).unwrap();

            if target == test {
                prop_assert!(result.is_some());
            } else {
                prop_assert!(result.is_none());
            }
        }

        /// Property: Tuple pattern matches only correct arity
        #[test]
        fn prop_tuple_arity_must_match(size in 1usize..5) {
            let patterns: Vec<Pattern> = (0..size)
                .map(|i| Pattern::Identifier(format!("x{}", i)))
                .collect();
            let pattern = Pattern::Tuple(patterns);

            // Correct arity - should match
            let values: Vec<Value> = (0..size).map(|i| Value::Integer(i as i64)).collect();
            let val = Value::Tuple(std::sync::Arc::from(values.clone()));
            let result = try_pattern_match(&pattern, &val, &test_eval_literal).unwrap();
            prop_assert!(result.is_some());

            // Wrong arity (one less) - should not match
            if size > 1 {
                let wrong_values: Vec<Value> = (0..size-1).map(|i| Value::Integer(i as i64)).collect();
                let wrong_val = Value::Tuple(std::sync::Arc::from(wrong_values));
                let result = try_pattern_match(&pattern, &wrong_val, &test_eval_literal).unwrap();
                prop_assert!(result.is_none());
            }
        }

        /// Property: EnumVariant unit pattern matches only unit variants
        #[test]
        fn prop_unit_variant_no_data(variant_name in "[A-Z][a-z]{1,10}") {
            let pattern = Pattern::QualifiedName(vec!["Enum".to_string(), variant_name.clone()]);

            // Unit variant (no data) - should match
            let val_unit = Value::EnumVariant {
                enum_name: "Enum".to_string(),
                variant_name: variant_name.clone(),
                data: None,
            };
            let result = try_pattern_match(&pattern, &val_unit, &test_eval_literal).unwrap();
            prop_assert!(result.is_some());

            // Tuple variant (with data) - should NOT match
            let val_tuple = Value::EnumVariant {
                enum_name: "Enum".to_string(),
                variant_name: variant_name.clone(),
                data: Some(vec![Value::Integer(42)]),
            };
            let result = try_pattern_match(&pattern, &val_tuple, &test_eval_literal).unwrap();
            prop_assert!(result.is_none());
        }

        /// Property: TupleVariant pattern binds all elements
        #[test]
        fn prop_tuple_variant_binds_all(count in 1usize..4, values in prop::collection::vec(any::<i64>(), 1..4)) {
            let count = count.min(values.len());
            let patterns: Vec<Pattern> = (0..count)
                .map(|i| Pattern::Identifier(format!("x{}", i)))
                .collect();

            let pattern = Pattern::TupleVariant {
                path: vec!["Type".to_string(), "Variant".to_string()],
                patterns,
            };

            let variant_values: Vec<Value> = values[..count].iter()
                .map(|&v| Value::Integer(v))
                .collect();

            let val = Value::EnumVariant {
                enum_name: "Type".to_string(),
                variant_name: "Variant".to_string(),
                data: Some(variant_values.clone()),
            };

            let result = try_pattern_match(&pattern, &val, &test_eval_literal).unwrap();
            prop_assert!(result.is_some());

            let bindings = result.unwrap();
            prop_assert_eq!(bindings.len(), count);

            for i in 0..count {
                prop_assert_eq!(&bindings[i].0, &format!("x{}", i));
                prop_assert_eq!(&bindings[i].1, &Value::Integer(values[i]));
            }
        }

        /// Property: Or pattern matches if ANY subpattern matches
        #[test]
        fn prop_or_pattern_any_match(target: i64, options in prop::collection::vec(any::<i64>(), 1..5)) {
            let patterns: Vec<Pattern> = options.iter()
                .map(|&v| Pattern::Literal(Literal::Integer(v, None)))
                .collect();

            let pattern = Pattern::Or(patterns);
            let val = Value::Integer(target);
            let result = try_pattern_match(&pattern, &val, &test_eval_literal).unwrap();

            if options.contains(&target) {
                prop_assert!(result.is_some());
            } else {
                prop_assert!(result.is_none());
            }
        }

        /// Property: Pattern matching never panics (robustness)
        #[test]
        fn prop_never_panics(value: i64) {
            let patterns = vec![
                Pattern::Wildcard,
                Pattern::Identifier("x".to_string()),
                Pattern::Literal(Literal::Integer(value, None)),
                Pattern::QualifiedName(vec!["Type".to_string(), "Variant".to_string()]),
            ];

            let val = Value::Integer(value);

            for pattern in patterns {
                // Should not panic
                let _ = try_pattern_match(&pattern, &val, &test_eval_literal);
            }
        }
    }
}
