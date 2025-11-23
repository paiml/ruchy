//! Pattern matching evaluation module
//!
//! This module handles all pattern matching operations in the interpreter.
//! Provides comprehensive pattern matching for destructuring assignments,
//! match expressions, and function parameter binding.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Literal, Pattern, StructPatternField};
use crate::runtime::pattern_matching::values_equal;
use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;

/// Pattern matching result with variable bindings
#[derive(Debug, Clone)]
pub struct PatternMatchResult {
    pub matches: bool,
    pub bindings: HashMap<String, Value>,
}

impl PatternMatchResult {
    /// Create a successful match with bindings
    pub fn success(bindings: HashMap<String, Value>) -> Self {
        Self {
            matches: true,
            bindings,
        }
    }

    /// Create a failed match
    pub fn failure() -> Self {
        Self {
            matches: false,
            bindings: HashMap::new(),
        }
    }

    /// Create a successful match with no bindings
    pub fn success_no_bindings() -> Self {
        Self {
            matches: true,
            bindings: HashMap::new(),
        }
    }
}

/// Match a pattern against a value with comprehensive binding support
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn match_pattern(
    pattern: &Pattern,
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    match pattern {
        Pattern::Wildcard => Ok(PatternMatchResult::success_no_bindings()),

        Pattern::Identifier(name) => {
            let mut bindings = HashMap::new();
            bindings.insert(name.clone(), value.clone());
            Ok(PatternMatchResult::success(bindings))
        }

        Pattern::Literal(lit) => {
            let pattern_value = eval_pattern_literal(lit)?;
            if values_equal(&pattern_value, value) {
                Ok(PatternMatchResult::success_no_bindings())
            } else {
                Ok(PatternMatchResult::failure())
            }
        }

        Pattern::List(patterns) => match_array_pattern(patterns, value),
        Pattern::Tuple(patterns) => match_tuple_pattern(patterns, value),
        Pattern::Struct { name, fields, .. } => match_struct_pattern(name, fields, value),
        Pattern::TupleVariant { path, patterns } => {
            match_tuple_variant_pattern(path, patterns, value)
        }
        Pattern::Ok(inner_pattern) => match_ok_pattern(inner_pattern, value),
        Pattern::Err(inner_pattern) => match_err_pattern(inner_pattern, value),
        _ => Ok(PatternMatchResult::failure()), // Other patterns not implemented yet
    }
}

/// Match tuple variant pattern (enum variant destructuring)
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn match_tuple_variant_pattern(
    path: &[String],
    patterns: &[Pattern],
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    // Match against Value::EnumVariant { variant_name, data }
    if let Value::EnumVariant {
        variant_name, data, ..
    } = value
    {
        // Extract variant name from path (last element)
        // Path is like ["Result", "Ok"] -> we want "Ok"
        let expected_variant = path.last().map_or("", std::string::String::as_str);

        // Check if variant names match
        if variant_name != expected_variant {
            return Ok(PatternMatchResult::failure());
        }

        // Match data against patterns
        match data {
            Some(values) => {
                if values.len() != patterns.len() {
                    return Ok(PatternMatchResult::failure());
                }

                let mut all_bindings = HashMap::new();
                for (pattern, val) in patterns.iter().zip(values.iter()) {
                    let result = match_pattern(pattern, val)?;
                    if !result.matches {
                        return Ok(PatternMatchResult::failure());
                    }
                    all_bindings.extend(result.bindings);
                }
                Ok(PatternMatchResult::success(all_bindings))
            }
            None => {
                // Unit variant (no data) - should have no patterns
                if patterns.is_empty() {
                    Ok(PatternMatchResult::success_no_bindings())
                } else {
                    Ok(PatternMatchResult::failure())
                }
            }
        }
    } else {
        Ok(PatternMatchResult::failure())
    }
}

/// Helper: Match slice of patterns against slice of values
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn match_pattern_slice(
    patterns: &[Pattern],
    values: &[Value],
) -> Result<PatternMatchResult, InterpreterError> {
    if patterns.len() != values.len() {
        return Ok(PatternMatchResult::failure());
    }

    let mut all_bindings = HashMap::new();

    for (pattern, val) in patterns.iter().zip(values.iter()) {
        let result = match_pattern(pattern, val)?;
        if !result.matches {
            return Ok(PatternMatchResult::failure());
        }
        all_bindings.extend(result.bindings);
    }

    Ok(PatternMatchResult::success(all_bindings))
}

/// Match array pattern with support for destructuring
///
/// # Complexity
/// Cyclomatic complexity: 2 (reduced via helper extraction)
fn match_array_pattern(
    patterns: &[Pattern],
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    if let Value::Array(arr) = value {
        match_pattern_slice(patterns, arr)
    } else {
        Ok(PatternMatchResult::failure())
    }
}

/// Match tuple pattern with positional destructuring
///
/// # Complexity
/// Cyclomatic complexity: 2 (reduced via helper extraction)
fn match_tuple_pattern(
    patterns: &[Pattern],
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    if let Value::Tuple(elements) = value {
        match_pattern_slice(patterns, elements)
    } else {
        Ok(PatternMatchResult::failure())
    }
}

/// Match struct pattern with named field destructuring
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn match_struct_pattern(
    struct_name: &str,
    field_patterns: &[StructPatternField],
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    // Struct patterns not yet implemented in Value enum
    let _ = (struct_name, field_patterns, value);
    Ok(PatternMatchResult::failure())
}

/// Evaluate a literal pattern to its runtime value
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn eval_pattern_literal(literal: &Literal) -> Result<Value, InterpreterError> {
    match literal {
        Literal::Integer(n, _) => Ok(Value::Integer(*n)),
        Literal::Float(f) => Ok(Value::Float(*f)),
        Literal::String(s) => Ok(Value::from_string(s.clone())),
        Literal::Bool(b) => Ok(Value::Bool(*b)),
        Literal::Char(c) => Ok(Value::from_string(c.to_string())),
        Literal::Byte(b) => Ok(Value::Byte(*b)),
        Literal::Unit => Ok(Value::Nil),
        Literal::Null => Ok(Value::Nil),
    }
}

/// Extract variable bindings from pattern matching result
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn extract_pattern_bindings(
    pattern: &Pattern,
    value: &Value,
) -> Result<HashMap<String, Value>, InterpreterError> {
    let result = match_pattern(pattern, value)?;
    if result.matches {
        Ok(result.bindings)
    } else {
        Err(InterpreterError::RuntimeError(
            "Pattern does not match value".to_string(),
        ))
    }
}

/// Check if pattern is irrefutable (always matches)
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn is_irrefutable_pattern(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        Pattern::Identifier(_) => true,
        Pattern::List(patterns) => patterns.iter().all(is_irrefutable_pattern),
        Pattern::Tuple(patterns) => patterns.iter().all(is_irrefutable_pattern),
        Pattern::Struct { fields, .. } => fields
            .iter()
            .all(|field| field.pattern.as_ref().is_none_or(is_irrefutable_pattern)),
        _ => false,
    }
}

/// Pattern exhaustiveness checker for match expressions
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn check_pattern_exhaustiveness(
    patterns: &[Pattern],
    value_type: &str,
) -> Result<bool, InterpreterError> {
    // Simple exhaustiveness check - can be enhanced
    let has_wildcard = patterns.iter().any(|p| matches!(p, Pattern::Wildcard));
    let has_identifier = patterns.iter().any(|p| matches!(p, Pattern::Identifier(_)));

    if has_wildcard || has_identifier {
        return Ok(true);
    }

    match value_type {
        "bool" => {
            let has_true = patterns
                .iter()
                .any(|p| matches!(p, Pattern::Literal(Literal::Bool(true))));
            let has_false = patterns
                .iter()
                .any(|p| matches!(p, Pattern::Literal(Literal::Bool(false))));
            Ok(has_true && has_false)
        }
        "nil" => Ok(patterns
            .iter()
            .any(|p| matches!(p, Pattern::Literal(Literal::Unit | Literal::Null)))),
        _ => {
            // For other types, assume exhaustive if we have enough patterns
            Ok(patterns.len() >= 2)
        }
    }
}

/// Helper: Match Result pattern (Ok or Err)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn match_result_pattern(
    inner_pattern: &Pattern,
    value: &Value,
    expected_type: &str,
) -> Result<PatternMatchResult, InterpreterError> {
    let fields = match_extract_object_fields(value)?;

    if !match_has_type(fields, expected_type) {
        return Ok(PatternMatchResult::failure());
    }

    let data = match_extract_data_array(fields)?;
    if data.is_empty() {
        return Ok(PatternMatchResult::failure());
    }

    match_pattern(inner_pattern, &data[0])
}

/// Match Ok pattern - Result success case
///
/// # Complexity
/// Cyclomatic complexity: 1 (reduced via helper extraction)
fn match_ok_pattern(
    inner_pattern: &Pattern,
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    match_result_pattern(inner_pattern, value, "Ok")
}

/// Extract object fields from Value
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn match_extract_object_fields(
    value: &Value,
) -> Result<&std::collections::HashMap<String, Value>, InterpreterError> {
    if let Value::Object(fields) = value {
        Ok(fields)
    } else {
        Err(InterpreterError::RuntimeError(
            "Expected Object value for Result pattern".to_string(),
        ))
    }
}

/// Check if object has type field matching expected type
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn match_has_type(fields: &std::collections::HashMap<String, Value>, expected: &str) -> bool {
    if let Some(Value::String(type_str)) = fields.get("type") {
        &**type_str == expected
    } else {
        false
    }
}

/// Extract data array from object fields
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn match_extract_data_array(
    fields: &std::collections::HashMap<String, Value>,
) -> Result<&[Value], InterpreterError> {
    if let Some(Value::Array(data)) = fields.get("data") {
        Ok(data)
    } else {
        Err(InterpreterError::RuntimeError(
            "Expected data array in Result value".to_string(),
        ))
    }
}

/// Match Err pattern - Result error case
///
/// # Complexity
/// Cyclomatic complexity: 1 (reduced via helper extraction)
fn match_err_pattern(
    inner_pattern: &Pattern,
    value: &Value,
) -> Result<PatternMatchResult, InterpreterError> {
    match_result_pattern(inner_pattern, value, "Err")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard;
        let value = Value::Integer(42);

        let result = match_pattern(&pattern, &value).expect("operation should succeed in test");
        assert!(result.matches);
        assert!(result.bindings.is_empty());
    }

    #[test]
    fn test_identifier_pattern() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Integer(42);

        let result = match_pattern(&pattern, &value).expect("operation should succeed in test");
        assert!(result.matches);
        assert_eq!(result.bindings.get("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_literal_pattern() {
        let pattern = Pattern::Literal(Literal::Integer(42, None));

        let matching_value = Value::Integer(42);
        let result =
            match_pattern(&pattern, &matching_value).expect("operation should succeed in test");
        assert!(result.matches);

        let non_matching_value = Value::Integer(43);
        let result =
            match_pattern(&pattern, &non_matching_value).expect("operation should succeed in test");
        assert!(!result.matches);
    }

    #[test]
    fn test_array_pattern() {
        let patterns = vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Literal(Literal::Integer(2, None)),
        ];

        let pattern = Pattern::List(patterns);

        let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = match_pattern(&pattern, &value).expect("operation should succeed in test");
        assert!(result.matches);
        assert_eq!(result.bindings.get("x"), Some(&Value::Integer(1)));
    }

    #[test]
    fn test_values_equal() {
        // Test using imported pattern_matching::values_equal
        // Note: This function is strict - no mixed int/float comparison
        assert!(values_equal(&Value::Integer(42), &Value::Integer(42)));
        assert!(values_equal(&Value::Float(3.15), &Value::Float(3.15)));
        assert!(!values_equal(&Value::Integer(42), &Value::Float(42.0))); // Strict type matching
        assert!(!values_equal(&Value::Integer(42), &Value::Integer(43)));
    }

    #[test]
    fn test_irrefutable_patterns() {
        let wildcard = Pattern::Wildcard;
        assert!(is_irrefutable_pattern(&wildcard));

        let identifier = Pattern::Identifier("x".to_string());
        assert!(is_irrefutable_pattern(&identifier));

        let literal = Pattern::Literal(Literal::Integer(42, None));
        assert!(!is_irrefutable_pattern(&literal));
    }
}
