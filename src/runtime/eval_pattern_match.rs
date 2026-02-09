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
/// Match a sequence of patterns against corresponding values, returning combined bindings
fn match_pattern_sequence(
    patterns: &[Pattern],
    values: &[Value],
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    let mut bindings = Vec::new();
    for (pattern, val) in patterns.iter().zip(values.iter()) {
        if let Some(b) = try_pattern_match(pattern, val, eval_literal)? {
            bindings.extend(b);
        } else {
            return Ok(None);
        }
    }
    Ok(Some(bindings))
}

/// Match a list pattern with a rest element (e.g., [first, ..rest, last])
fn try_match_list_with_rest(
    patterns: &[Pattern],
    rest_idx: usize,
    array_values: &[Value],
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    let before_rest = &patterns[..rest_idx];
    let after_rest = &patterns[rest_idx + 1..];
    let min_len = before_rest.len() + after_rest.len();

    if array_values.len() < min_len {
        return Ok(None);
    }

    let mut all_bindings = match match_pattern_sequence(before_rest, array_values, eval_literal)? {
        Some(b) => b,
        None => return Ok(None),
    };

    let rest_end = array_values.len() - after_rest.len();
    let rest_values: Vec<Value> = array_values[rest_idx..rest_end].to_vec();

    if let Pattern::RestNamed(name) = &patterns[rest_idx] {
        all_bindings.push((name.clone(), Value::Array(rest_values.into())));
    }

    match match_pattern_sequence(after_rest, &array_values[rest_end..], eval_literal)? {
        Some(b) => {
            all_bindings.extend(b);
            Ok(Some(all_bindings))
        }
        None => Ok(None),
    }
}

fn try_match_list_pattern(
    patterns: &[Pattern],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    let array_values = match value {
        Value::Array(a) => a,
        _ => return Ok(None),
    };

    let rest_pos = patterns
        .iter()
        .position(|p| matches!(p, Pattern::Rest | Pattern::RestNamed(_)));

    if let Some(rest_idx) = rest_pos {
        try_match_list_with_rest(patterns, rest_idx, array_values, eval_literal)
    } else if patterns.len() != array_values.len() {
        Ok(None)
    } else {
        match_pattern_sequence(patterns, array_values, eval_literal)
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
    if let Value::EnumVariant {
        variant_name, data, ..
    } = value
    {
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
    if let Value::EnumVariant {
        variant_name, data, ..
    } = value
    {
        if variant_name == "None" && data.is_none() {
            return Ok(Some(vec![]));
        }
    }
    Ok(None)
}

/// Try to match an Ok pattern
///
/// Supports both `EnumVariant` (Issue #85) and legacy Object representations
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
/// Extract the first data element from a Result-like value (Ok or Err variant)
fn extract_result_variant_data<'a>(value: &'a Value, expected_variant: &str) -> Option<&'a Value> {
    // EnumVariant representation (Issue #85)
    if let Value::EnumVariant {
        variant_name, data, ..
    } = value
    {
        if variant_name == expected_variant {
            return data.as_ref().and_then(|v| v.first());
        }
    }
    // Legacy Object representation {data: [x], type: "Ok"/"Err"}
    if let Value::Object(fields) = value {
        if let Some(Value::String(type_str)) = fields.get("type") {
            if &**type_str == expected_variant {
                if let Some(Value::Array(data)) = fields.get("data") {
                    return data.first();
                }
            }
        }
    }
    None
}

fn try_match_ok_pattern(
    inner_pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    match extract_result_variant_data(value, "Ok") {
        Some(data) => try_pattern_match(inner_pattern, data, eval_literal),
        None => Ok(None),
    }
}

fn try_match_err_pattern(
    inner_pattern: &Pattern,
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    match extract_result_variant_data(value, "Err") {
        Some(data) => try_pattern_match(inner_pattern, data, eval_literal),
        None => Ok(None),
    }
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
    if let Value::EnumVariant {
        variant_name, data, ..
    } = value
    {
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
    let (variant_name, data) = match value {
        Value::EnumVariant {
            variant_name, data, ..
        } => (variant_name, data),
        _ => return Ok(None),
    };

    if path.last() != Some(variant_name) {
        return Ok(None);
    }

    let variant_data = match data {
        Some(d) => d,
        None => return Ok(None),
    };

    if patterns.len() != variant_data.len() {
        return Ok(None);
    }

    match_pattern_sequence(patterns, variant_data, eval_literal)
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
fn match_struct_field(
    field_pattern: &StructPatternField,
    field_value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    match &field_pattern.pattern {
        Some(pattern) => try_pattern_match(pattern, field_value, eval_literal),
        None => Ok(Some(vec![(field_pattern.name.clone(), field_value.clone())])),
    }
}

fn try_match_struct_pattern(
    struct_name: &str,
    field_patterns: &[StructPatternField],
    value: &Value,
    eval_literal: &dyn Fn(&Literal) -> Value,
) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
    let struct_fields = match value {
        Value::Struct { name, fields } if name == struct_name => fields,
        Value::Object(fields) => fields,
        _ => return Ok(None),
    };

    let mut all_bindings = Vec::new();
    for field_pattern in field_patterns {
        let field_value = match struct_fields.get(&field_pattern.name) {
            Some(v) => v,
            None => return Ok(None),
        };

        match match_struct_field(field_pattern, field_value, eval_literal)? {
            Some(bindings) => all_bindings.extend(bindings),
            None => return Ok(None),
        }
    }

    Ok(Some(all_bindings))
}


#[cfg(test)]
#[path = "eval_pattern_match_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "eval_pattern_match_prop_tests.rs"]
mod property_tests;

#[cfg(test)]
#[path = "eval_pattern_match_coverage_tests.rs"]
mod coverage_tests;
