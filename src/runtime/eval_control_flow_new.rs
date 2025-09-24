//! Control flow evaluation module
//!
//! This module handles evaluation of all control flow constructs including
//! if expressions, loops, match statements, and pattern matching.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, Literal, MatchArm, Pattern};
use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate an if expression with optional else branch
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_if_expr<F>(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let condition_val = eval_expr(condition)?;

    if condition_val.is_truthy() {
        eval_expr(then_branch)
    } else if let Some(else_expr) = else_branch {
        eval_expr(else_expr)
    } else {
        Ok(Value::Nil)
    }
}

/// Evaluate a let expression with variable binding
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_let_expr<F1, F2>(
    name: &str,
    value: &Expr,
    _body: &Expr,
    mut eval_expr: F1,
    mut with_variable: F2,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(
        &str,
        Value,
        &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
    ) -> Result<Value, InterpreterError>,
{
    let val = eval_expr(value)?;
    with_variable(name, val, &mut eval_expr)
}

/// Evaluate a for loop with iterator and body
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_for_loop<F1, F2>(
    var: &str,
    iter: &Expr,
    _body: &Expr,
    mut eval_expr: F1,
    mut with_variable: F2,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(
        &str,
        Value,
        &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
    ) -> Result<Value, InterpreterError>,
{
    let iter_val = eval_expr(iter)?;
    let mut last_val = Value::Nil;

    match iter_val {
        Value::Array(arr) => {
            for item in arr.iter() {
                match with_variable(var, item.clone(), &mut eval_expr) {
                    Ok(val) => last_val = val,
                    Err(InterpreterError::Break(val)) => return Ok(val),
                    Err(InterpreterError::Continue) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            let start_val = match *start {
                Value::Integer(i) => i,
                _ => {
                    return Err(InterpreterError::TypeError(
                        "Range start must be integer".to_string(),
                    ))
                }
            };
            let end_val = match *end {
                Value::Integer(i) => i,
                _ => {
                    return Err(InterpreterError::TypeError(
                        "Range end must be integer".to_string(),
                    ))
                }
            };

            let range: Box<dyn Iterator<Item = i64>> = if inclusive {
                Box::new(start_val..=end_val)
            } else {
                Box::new(start_val..end_val)
            };

            for i in range {
                match with_variable(var, Value::Integer(i), &mut eval_expr) {
                    Ok(val) => last_val = val,
                    Err(InterpreterError::Break(val)) => return Ok(val),
                    Err(InterpreterError::Continue) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        _ => {
            return Err(InterpreterError::TypeError(format!(
                "Cannot iterate over {}",
                iter_val.type_name()
            )))
        }
    }

    Ok(last_val)
}

/// Evaluate a while loop with condition and body
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn eval_while_loop<F>(
    condition: &Expr,
    body: &Expr,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut last_val = Value::Nil;

    loop {
        let condition_val = eval_expr(condition)?;
        if !condition_val.is_truthy() {
            break;
        }

        match eval_expr(body) {
            Ok(val) => last_val = val,
            Err(InterpreterError::Break(val)) => return Ok(val),
            Err(InterpreterError::Continue) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(last_val)
}

/// Evaluate a match expression with pattern matching
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn eval_match<F1, F2>(
    expr: &Expr,
    arms: &[MatchArm],
    mut eval_expr: F1,
    mut pattern_matches: F2,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(&Pattern, &Value) -> Result<bool, InterpreterError>,
{
    let value = eval_expr(expr)?;

    for arm in arms {
        if pattern_matches(&arm.pattern, &value)? {
            if let Some(ref guard) = arm.guard {
                let guard_val = eval_expr(guard)?;
                if !guard_val.is_truthy() {
                    continue;
                }
            }
            return eval_expr(&arm.body);
        }
    }

    Err(InterpreterError::RuntimeError(
        "No matching pattern found in match expression".to_string(),
    ))
}

/// Evaluate a block expression (sequence of statements)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_block_expr<F>(statements: &[Expr], mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut last_val = Value::Nil;

    for stmt in statements {
        match eval_expr(stmt) {
            Ok(val) => last_val = val,
            Err(InterpreterError::Return(val)) => return Ok(val),
            Err(e) => return Err(e),
        }
    }

    Ok(last_val)
}

/// Evaluate a list expression (array literal)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_list_expr<F>(elements: &[Expr], mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut values = Vec::new();

    for element in elements {
        values.push(eval_expr(element)?);
    }

    Ok(Value::from_array(values))
}

/// Evaluate a tuple expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_tuple_expr<F>(elements: &[Expr], mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut values = Vec::new();

    for element in elements {
        values.push(eval_expr(element)?);
    }

    Ok(Value::Tuple(Rc::from(values.as_slice())))
}

/// Evaluate a range expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_range_expr<F>(
    start: &Expr,
    end: &Expr,
    inclusive: bool,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let start_val = eval_expr(start)?;
    let end_val = eval_expr(end)?;

    Ok(Value::Range {
        start: Box::new(start_val),
        end: Box::new(end_val),
        inclusive,
    })
}

/// Evaluate an array initialization expression
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn eval_array_init_expr<F>(
    value_expr: &Expr,
    size_expr: &Expr,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let value = eval_expr(value_expr)?;
    let size_val = eval_expr(size_expr)?;

    if let Value::Integer(size) = size_val {
        if size >= 0 {
            let mut values = Vec::new();
            for _ in 0..size {
                values.push(value.clone());
            }
            Ok(Value::from_array(values))
        } else {
            Err(InterpreterError::RuntimeError(
                "Array size must be non-negative".to_string(),
            ))
        }
    } else {
        Err(InterpreterError::TypeError(
            "Array size must be integer".to_string(),
        ))
    }
}

/// Evaluate a return expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_return_expr<F>(
    value: Option<&Expr>,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let return_value = if let Some(expr) = value {
        eval_expr(expr)?
    } else {
        Value::Nil
    };

    Err(InterpreterError::Return(return_value))
}

// Additional control flow utilities

/// Check if a pattern matches a value (simplified version)
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
// =============================================================================
// COMPLEXITY REFACTORING: Pattern Matching Helper Functions
// Target: Reduce pattern_matches_simple complexity from 12 → ≤10
// =============================================================================

/// Match wildcard patterns (always matches)
/// Complexity: 1
pub fn match_wildcard_pattern(_value: &Value) -> bool {
    true // Wildcard always matches
}

/// Match literal patterns
/// Complexity: 3
pub fn match_literal_pattern(lit: &Literal, value: &Value) -> Result<bool, InterpreterError> {
    let pattern_val = crate::runtime::eval_literal::eval_literal(lit);
    Ok(pattern_val == *value)
}

/// Match identifier patterns (always matches, binds variable)
/// Complexity: 1
pub fn match_identifier_pattern(_name: &str, _value: &Value) -> bool {
    true // Identifier always matches, binds the variable
}

/// Match list patterns recursively
/// Complexity: 6
pub fn match_list_pattern(patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
    if let Value::Array(arr) = value {
        if patterns.len() != arr.len() {
            return Ok(false);
        }
        for (pat, val) in patterns.iter().zip(arr.iter()) {
            if !pattern_matches_simple(pat, val)? {
                return Ok(false);
            }
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Match tuple patterns recursively
/// Complexity: 6
pub fn match_tuple_pattern(patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
    if let Value::Tuple(elements) = value {
        if patterns.len() != elements.len() {
            return Ok(false);
        }
        for (pat, val) in patterns.iter().zip(elements.iter()) {
            if !pattern_matches_simple(pat, val)? {
                return Ok(false);
            }
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Pattern matching with complexity reduced from 12 → 5
/// Complexity: 5 (down from 12)
pub fn pattern_matches_simple(pattern: &Pattern, value: &Value) -> Result<bool, InterpreterError> {
    match pattern {
        Pattern::Wildcard => Ok(match_wildcard_pattern(value)),
        Pattern::Literal(lit) => match_literal_pattern(lit, value),
        Pattern::Identifier(name) => Ok(match_identifier_pattern(name, value)),
        Pattern::List(patterns) => match_list_pattern(patterns, value),
        Pattern::Tuple(patterns) => match_tuple_pattern(patterns, value),
        _ => Ok(false), // Other patterns not implemented yet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span};

    #[test]
    fn test_eval_if_expr() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Bool(true)),  // condition
                2 => Ok(Value::Integer(42)), // then branch
                _ => panic!("Unexpected call"),
            }
        };

        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4),
        );
        let then_branch = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42)),
            Span::new(5, 7),
        );

        let result = eval_if_expr(&condition, &then_branch, None, eval_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_list_expr() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            Ok(Value::Integer(call_count))
        };

        let elements = vec![
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(1)),
                Span::new(0, 1),
            ),
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(2)),
                Span::new(3, 4),
            ),
        ];

        let result = eval_list_expr(&elements, eval_expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_block_expr() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            Ok(Value::Integer(call_count * 10))
        };

        let statements = vec![
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(1)),
                Span::new(0, 1),
            ),
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(2)),
                Span::new(3, 4),
            ),
        ];

        let result = eval_block_expr(&statements, eval_expr).unwrap();
        assert_eq!(result, Value::Integer(20)); // Last statement result
    }

    #[test]
    fn test_pattern_matches_simple() {
        let wildcard_pattern = Pattern::Wildcard;
        assert!(pattern_matches_simple(&wildcard_pattern, &Value::Integer(42)).unwrap());

        let literal_pattern = Pattern::Literal(Literal::Integer(42));
        assert!(pattern_matches_simple(&literal_pattern, &Value::Integer(42)).unwrap());
        assert!(!pattern_matches_simple(&literal_pattern, &Value::Integer(43)).unwrap());
    }
}
