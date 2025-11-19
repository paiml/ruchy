//! Control flow evaluation module
//!
//! This module handles evaluation of all control flow constructs including
//! if expressions, loops, match statements, and pattern matching.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, Literal, MatchArm, Pattern};
use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

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
/// Cyclomatic complexity: 3 (reduced from 8, cognitive complexity from 42 → ≤10)
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

    match &iter_val {
        Value::Array(_) => eval_array_iteration(&iter_val, var, &mut with_variable, &mut eval_expr),
        Value::Range { .. } => {
            eval_range_iteration(&iter_val, var, &mut with_variable, &mut eval_expr)
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot iterate over {}",
            iter_val.type_name()
        ))),
    }
}

// =============================================================================
// COMPLEXITY REFACTORING: While Loop Helper Functions
// Target: Reduce eval_while_loop cognitive complexity from 16 → ≤10
// =============================================================================

/// Evaluate loop condition
/// Complexity: ≤3
pub fn eval_loop_condition<F>(condition: &Expr, eval_expr: &mut F) -> Result<bool, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    Ok(eval_expr(condition)?.is_truthy())
}

/// Evaluate loop body and handle control flow
/// Complexity: ≤5
pub fn eval_loop_body<F>(
    body: &Expr,
    last_val: &mut Value,
    eval_expr: &mut F,
) -> Result<Option<Value>, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    match eval_expr(body) {
        Ok(val) => {
            *last_val = val;
            Ok(None)
        }
        Err(InterpreterError::Break(None, val)) => Ok(Some(val)),
        Err(InterpreterError::Continue(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Run the while loop with separated concerns
/// Complexity: ≤8
pub fn run_while_loop<F>(
    condition: &Expr,
    body: &Expr,
    eval_expr: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut last_val = Value::Nil;

    loop {
        if !eval_loop_condition(condition, eval_expr)? {
            break;
        }

        if let Some(break_val) = eval_loop_body(body, &mut last_val, eval_expr)? {
            return Ok(break_val);
        }
    }

    Ok(last_val)
}

/// Evaluate a while loop with condition and body
///
/// # Complexity
/// Cyclomatic complexity: 1 (reduced from 5, cognitive from 16 → ≤3)
pub fn eval_while_loop<F>(
    condition: &Expr,
    body: &Expr,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    run_while_loop(condition, body, &mut eval_expr)
}

// =============================================================================
// COMPLEXITY REFACTORING: Match Expression Helper Functions
// Target: Reduce eval_match cognitive complexity from 25 → ≤10
// =============================================================================

/// Evaluate a single match arm
/// Complexity: ≤5
pub fn eval_match_arm<F1, F2>(
    arm: &MatchArm,
    value: &Value,
    pattern_matches: &mut F2,
    eval_expr: &mut F1,
) -> Result<Option<Value>, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(&Pattern, &Value) -> Result<bool, InterpreterError>,
{
    if pattern_matches(&arm.pattern, value)? && eval_match_guard(arm.guard.as_deref(), eval_expr)? {
        return Ok(Some(eval_expr(&arm.body)?));
    }
    Ok(None)
}

/// Evaluate guard expression if present
/// Complexity: ≤3
pub fn eval_match_guard<F>(
    guard: Option<&Expr>,
    eval_expr: &mut F,
) -> Result<bool, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    if let Some(guard_expr) = guard {
        Ok(eval_expr(guard_expr)?.is_truthy())
    } else {
        Ok(true) // No guard means always pass
    }
}

/// Find the first matching arm and evaluate it
/// Complexity: ≤8
pub fn find_matching_arm<F1, F2>(
    arms: &[MatchArm],
    value: &Value,
    pattern_matches: &mut F2,
    eval_expr: &mut F1,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(&Pattern, &Value) -> Result<bool, InterpreterError>,
{
    for arm in arms {
        if let Some(result) = eval_match_arm(arm, value, pattern_matches, eval_expr)? {
            return Ok(result);
        }
    }

    Err(InterpreterError::RuntimeError(
        "No matching pattern found in match expression".to_string(),
    ))
}

/// Evaluate a match expression with pattern matching
///
/// # Complexity
/// Cyclomatic complexity: 2 (reduced from 6, cognitive from 25 → ≤5)
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
    find_matching_arm(arms, &value, &mut pattern_matches, &mut eval_expr)
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
        last_val = eval_expr(stmt)?; // Propagate all errors including Return
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

    Ok(Value::Tuple(Arc::from(values.as_slice())))
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

// =============================================================================
// COMPLEXITY REFACTORING: For Loop Helper Functions
// Target: Reduce eval_for_loop cognitive complexity from 42 → ≤10
// =============================================================================

/// Handle array iteration in for loops
/// Complexity: ≤8
pub fn eval_array_iteration<F1, F2>(
    array: &Value,
    var: &str,
    with_variable: &mut F2,
    eval_expr: &mut F1,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(
        &str,
        Value,
        &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
    ) -> Result<Value, InterpreterError>,
{
    if let Value::Array(arr) = array {
        let mut last_val = Value::Nil;
        for item in arr.iter() {
            let should_continue =
                execute_iteration_step(var, item.clone(), with_variable, eval_expr, &mut last_val)?;
            if !should_continue {
                break;
            }
        }
        Ok(last_val)
    } else {
        Err(InterpreterError::TypeError(format!(
            "Expected array, got {}",
            array.type_name()
        )))
    }
}

/// Execute iteration body with loop control handling
/// Complexity: ≤5
fn execute_iteration_step<F1, F2>(
    loop_var: &str,
    value: Value,
    with_variable: &mut F2,
    eval_expr: &mut F1,
    last_val: &mut Value,
) -> Result<bool, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(
        &str,
        Value,
        &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
    ) -> Result<Value, InterpreterError>,
{
    match with_variable(loop_var, value, eval_expr) {
        Ok(result_val) => {
            *last_val = result_val;
            Ok(true) // Continue iteration
        }
        Err(InterpreterError::Break(None, break_val)) => {
            *last_val = break_val;
            Ok(false) // Stop iteration
        }
        Err(InterpreterError::Continue(_)) => Ok(true), // Continue iteration
        Err(e) => Err(e),
    }
}

/// Handle range iteration in for loops
/// Complexity: ≤5
pub fn eval_range_iteration<F1, F2>(
    range: &Value,
    var: &str,
    with_variable: &mut F2,
    eval_expr: &mut F1,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr) -> Result<Value, InterpreterError>,
    F2: FnMut(
        &str,
        Value,
        &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
    ) -> Result<Value, InterpreterError>,
{
    if let Value::Range { .. } = range {
        let (start_val, end_val, inclusive) = extract_range_bounds(range)?;
        let iter = create_range_iterator(start_val, end_val, inclusive);

        let mut last_val = Value::Nil;
        for i in iter {
            let should_continue = execute_iteration_step(
                var,
                Value::Integer(i),
                with_variable,
                eval_expr,
                &mut last_val,
            )?;
            if !should_continue {
                break;
            }
        }
        Ok(last_val)
    } else {
        Err(InterpreterError::TypeError(format!(
            "Expected range, got {}",
            range.type_name()
        )))
    }
}

/// Extract integer from a value
/// Complexity: ≤2
fn value_to_integer(value: &Value, context: &str) -> Result<i64, InterpreterError> {
    match value {
        Value::Integer(i) => Ok(*i),
        _ => Err(InterpreterError::TypeError(format!(
            "{context} must be integer"
        ))),
    }
}

/// Extract integer bounds from a range value
/// Complexity: ≤5
pub fn extract_range_bounds(range: &Value) -> Result<(i64, i64, bool), InterpreterError> {
    if let Value::Range {
        start,
        end,
        inclusive,
    } = range
    {
        let start_val = value_to_integer(start, "Range start")?;
        let end_val = value_to_integer(end, "Range end")?;
        Ok((start_val, end_val, *inclusive))
    } else {
        Err(InterpreterError::TypeError(
            "Expected range value".to_string(),
        ))
    }
}

/// Handle loop control flow (break/continue)
/// Complexity: ≤5
pub fn handle_loop_control(
    result: Result<Value, InterpreterError>,
    last_val: &mut Value,
) -> Result<Option<Value>, InterpreterError> {
    match result {
        Ok(val) => {
            *last_val = val;
            Ok(None)
        }
        Err(InterpreterError::Break(None, val)) => Ok(Some(val)),
        Err(InterpreterError::Continue(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Create an iterator from range bounds
/// Complexity: ≤3
pub fn create_range_iterator(
    start: i64,
    end: i64,
    inclusive: bool,
) -> Box<dyn Iterator<Item = i64>> {
    if inclusive {
        Box::new(start..=end)
    } else {
        Box::new(start..end)
    }
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

/// Check if patterns match values element-wise
/// Complexity: ≤5
fn patterns_match_values(patterns: &[Pattern], values: &[Value]) -> Result<bool, InterpreterError> {
    if patterns.len() != values.len() {
        return Ok(false);
    }
    for (pat, val) in patterns.iter().zip(values.iter()) {
        if !pattern_matches_simple(pat, val)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Match list patterns recursively
/// Complexity: ≤3
pub fn match_list_pattern(patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
    match value {
        Value::Array(arr) => patterns_match_values(patterns, arr),
        _ => Ok(false),
    }
}

/// Match tuple patterns recursively
/// Complexity: ≤3
pub fn match_tuple_pattern(patterns: &[Pattern], value: &Value) -> Result<bool, InterpreterError> {
    match value {
        Value::Tuple(elements) => patterns_match_values(patterns, elements),
        _ => Ok(false),
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
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
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
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(1, None)),
                Span::new(0, 1),
            ),
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(2, None)),
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
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(1, None)),
                Span::new(0, 1),
            ),
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(2, None)),
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

        let literal_pattern = Pattern::Literal(Literal::Integer(42, None));
        assert!(pattern_matches_simple(&literal_pattern, &Value::Integer(42)).unwrap());
        assert!(!pattern_matches_simple(&literal_pattern, &Value::Integer(43)).unwrap());
    }

    // ===== TUPLE EXPRESSION TESTS =====

    #[test]
    fn test_eval_tuple_expr_basic() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            Ok(Value::Integer(call_count))
        };

        let elements = vec![
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(1, None)),
                Span::new(0, 1),
            ),
            Expr::new(
                crate::frontend::ast::ExprKind::Literal(Literal::Integer(2, None)),
                Span::new(2, 3),
            ),
        ];

        let result = eval_tuple_expr(&elements, eval_expr).unwrap();
        if let Value::Tuple(tuple) = result {
            assert_eq!(tuple.len(), 2);
            assert_eq!(tuple[0], Value::Integer(1));
            assert_eq!(tuple[1], Value::Integer(2));
        } else {
            panic!("Expected tuple result");
        }
    }

    #[test]
    fn test_eval_tuple_expr_empty() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };
        let result = eval_tuple_expr(&[], eval_expr).unwrap();
        if let Value::Tuple(tuple) = result {
            assert_eq!(tuple.len(), 0);
        } else {
            panic!("Expected empty tuple");
        }
    }

    // ===== RANGE EXPRESSION TESTS =====

    #[test]
    fn test_eval_range_expr_inclusive() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Integer(1)),  // start
                2 => Ok(Value::Integer(10)), // end
                _ => panic!("Unexpected call"),
            }
        };

        let start = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(1, None)),
            Span::new(0, 1),
        );
        let end = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(10, None)),
            Span::new(2, 3),
        );

        let result = eval_range_expr(&start, &end, true, eval_expr).unwrap();
        if let Value::Range {
            start: s,
            end: e,
            inclusive,
        } = result
        {
            assert_eq!(*s, Value::Integer(1));
            assert_eq!(*e, Value::Integer(10));
            assert!(inclusive);
        } else {
            panic!("Expected range");
        }
    }

    #[test]
    fn test_eval_range_expr_exclusive() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Integer(0)),
                2 => Ok(Value::Integer(5)),
                _ => panic!("Unexpected call"),
            }
        };

        let start = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(0, None)),
            Span::new(0, 1),
        );
        let end = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(5, None)),
            Span::new(2, 3),
        );

        let result = eval_range_expr(&start, &end, false, eval_expr).unwrap();
        if let Value::Range { inclusive, .. } = result {
            assert!(!inclusive);
        } else {
            panic!("Expected range");
        }
    }

    // ===== LOOP CONDITION TESTS =====

    #[test]
    fn test_eval_loop_condition_true() {
        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4),
        );
        let mut eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Bool(true)) };
        let result = eval_loop_condition(&condition, &mut eval_expr).unwrap();
        assert!(result);
    }

    #[test]
    fn test_eval_loop_condition_false() {
        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 5),
        );
        let mut eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Bool(false)) };
        let result = eval_loop_condition(&condition, &mut eval_expr).unwrap();
        assert!(!result);
    }

    // ===== PATTERN MATCHING HELPERS TESTS =====

    #[test]
    fn test_match_wildcard_pattern() {
        assert!(match_wildcard_pattern(&Value::Integer(42)));
        assert!(match_wildcard_pattern(&Value::Bool(true)));
        assert!(match_wildcard_pattern(&Value::Nil));
    }

    #[test]
    fn test_match_literal_pattern_integer() {
        let lit = Literal::Integer(42, None);
        assert!(match_literal_pattern(&lit, &Value::Integer(42)).unwrap());
        assert!(!match_literal_pattern(&lit, &Value::Integer(43)).unwrap());
    }

    #[test]
    fn test_match_literal_pattern_bool() {
        let lit_true = Literal::Bool(true);
        assert!(match_literal_pattern(&lit_true, &Value::Bool(true)).unwrap());
        assert!(!match_literal_pattern(&lit_true, &Value::Bool(false)).unwrap());
    }

    #[test]
    fn test_match_literal_pattern_string() {
        let lit = Literal::String("hello".to_string());
        assert!(match_literal_pattern(&lit, &Value::String(Arc::from("hello"))).unwrap());
        assert!(!match_literal_pattern(&lit, &Value::String(Arc::from("world"))).unwrap());
    }

    #[test]
    fn test_match_identifier_pattern() {
        assert!(match_identifier_pattern("x", &Value::Integer(42)));
        assert!(match_identifier_pattern("foo", &Value::Bool(true)));
    }

    #[test]
    fn test_match_list_pattern_basic() {
        let patterns = vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
        ];
        let arr = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(match_list_pattern(&patterns, &Value::Array(arr)).unwrap());
    }

    #[test]
    fn test_match_list_pattern_length_mismatch() {
        let patterns = vec![Pattern::Literal(Literal::Integer(1, None))];
        let arr = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(!match_list_pattern(&patterns, &Value::Array(arr)).unwrap());
    }

    #[test]
    fn test_match_tuple_pattern_basic() {
        let patterns = vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Wildcard,
        ];
        let tuple = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(match_tuple_pattern(&patterns, &Value::Tuple(tuple)).unwrap());
    }

    #[test]
    fn test_match_tuple_pattern_length_mismatch() {
        let patterns = vec![Pattern::Wildcard];
        let tuple = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(!match_tuple_pattern(&patterns, &Value::Tuple(tuple)).unwrap());
    }

    // ===== RANGE HELPERS TESTS =====

    #[test]
    fn test_extract_range_bounds_inclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };
        let (start, end, inclusive) = extract_range_bounds(&range).unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 10);
        assert!(inclusive);
    }

    #[test]
    fn test_extract_range_bounds_exclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(5)),
            inclusive: false,
        };
        let (start, end, inclusive) = extract_range_bounds(&range).unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 5);
        assert!(!inclusive);
    }

    #[test]
    fn test_extract_range_bounds_non_range() {
        let result = extract_range_bounds(&Value::Integer(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_range_iterator_inclusive() {
        let iter = create_range_iterator(1, 3, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_create_range_iterator_exclusive() {
        let iter = create_range_iterator(1, 3, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn test_create_range_iterator_empty_when_start_gt_end() {
        // Rust ranges are empty when start > end (no reverse iteration)
        let iter = create_range_iterator(5, 3, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, Vec::<i64>::new()); // Empty range
    }

    // ===== IF EXPRESSION EDGE CASES =====

    #[test]
    fn test_eval_if_expr_false_no_else() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            if call_count == 1 {
                Ok(Value::Bool(false)) // condition is false
            } else {
                panic!("Should not evaluate then branch");
            }
        };

        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 5),
        );
        let then_branch = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(6, 8),
        );

        let result = eval_if_expr(&condition, &then_branch, None, eval_expr).unwrap();
        assert_eq!(result, Value::Nil); // No else branch, returns Nil
    }

    #[test]
    fn test_eval_if_expr_with_else() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Bool(false)), // condition is false
                2 => Ok(Value::Integer(99)), // else branch
                _ => panic!("Unexpected call"),
            }
        };

        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 5),
        );
        let then_branch = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(6, 8),
        );
        let else_branch = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(99, None)),
            Span::new(14, 16),
        );

        let result = eval_if_expr(&condition, &then_branch, Some(&else_branch), eval_expr).unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    // ===== BLOCK EXPRESSION EDGE CASES =====

    #[test]
    fn test_eval_block_expr_empty() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };
        let result = eval_block_expr(&[], eval_expr).unwrap();
        assert_eq!(result, Value::Nil); // Empty block returns Nil
    }

    #[test]
    fn test_eval_block_expr_single_statement() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };
        let statements = vec![Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        )];
        let result = eval_block_expr(&statements, eval_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== LIST EXPRESSION EDGE CASES =====

    #[test]
    fn test_eval_list_expr_empty() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };
        let result = eval_list_expr(&[], eval_expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected empty array");
        }
    }

    // ===== ARRAY INIT EXPRESSION TESTS =====

    #[test]
    fn test_eval_array_init_expr_basic() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Integer(42)), // element
                2 => Ok(Value::Integer(3)),  // size
                _ => panic!("Unexpected call"),
            }
        };

        let element = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        );
        let size = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(3, None)),
            Span::new(4, 5),
        );

        let result = eval_array_init_expr(&element, &size, eval_expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(42));
            assert_eq!(arr[1], Value::Integer(42));
            assert_eq!(arr[2], Value::Integer(42));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_eval_array_init_expr_zero_size() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Integer(42)),
                2 => Ok(Value::Integer(0)),
                _ => panic!("Unexpected call"),
            }
        };

        let element = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        );
        let size = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(0, None)),
            Span::new(4, 5),
        );

        let result = eval_array_init_expr(&element, &size, eval_expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected empty array");
        }
    }

    #[test]
    fn test_eval_array_init_expr_invalid_size() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Integer(42)),
                2 => Ok(Value::Bool(true)), // Invalid size type
                _ => panic!("Unexpected call"),
            }
        };

        let element = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        );
        let size = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(true)),
            Span::new(4, 8),
        );

        let result = eval_array_init_expr(&element, &size, eval_expr);
        assert!(result.is_err());
    }

    // ===== RETURN EXPRESSION TESTS =====

    #[test]
    fn test_eval_return_expr() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };

        let value = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        );
        let result = eval_return_expr(Some(&value), eval_expr);
        assert!(result.is_err()); // Return creates an error with value
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::Integer(42));
        } else {
            panic!("Expected return error");
        }
    }

    #[test]
    fn test_eval_return_expr_no_value() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };

        let result = eval_return_expr(None, eval_expr);
        assert!(result.is_err());
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::Nil);
        } else {
            panic!("Expected return error with nil");
        }
    }
}
