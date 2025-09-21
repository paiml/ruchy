//! Pattern matching evaluation module
//!
//! This module handles all pattern matching operations including match expressions,
//! pattern guards, destructuring, and pattern binding.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Literal, Pattern};
use crate::runtime::{InterpreterError, Value};

/// Try to match a pattern against a value, returning bindings if successful
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
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
    if let Pattern::Literal(Literal::Integer(val)) = pattern {
        Ok(*val)
    } else {
        Err(InterpreterError::RuntimeError(
            "Range pattern requires integer literals".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn test_eval_literal(lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i) => Value::Integer(*i),
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
        let pattern = Pattern::Literal(Literal::Integer(42));
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_literal_pattern_no_match() {
        let pattern = Pattern::Literal(Literal::Integer(42));
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
        let value = Value::Tuple(Rc::from(vec![Value::Integer(1), Value::Integer(2)]));

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
            start: Box::new(Pattern::Literal(Literal::Integer(1))),
            end: Box::new(Pattern::Literal(Literal::Integer(5))),
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
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Literal(Literal::Integer(3)),
        ];
        let pattern = Pattern::Or(patterns);

        let value = Value::Integer(2);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_some());

        let value = Value::Integer(4);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal).unwrap();
        assert!(result.is_none());
    }
}
