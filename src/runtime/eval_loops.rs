//! Loop evaluation module
//!
//! This module handles all loop operations including for loops, while loops,
//! and iteration over arrays and ranges.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, Pattern};
use crate::runtime::{InterpreterError, Value};

/// Evaluate a for loop over an iterable
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_for_loop<F>(
    var: &str,
    pattern: Option<&Pattern>,
    iter_value: Value,
    body: &Expr,
    set_variable: impl FnMut(String, Value),
    eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    match iter_value {
        Value::Array(arr) => eval_for_array(var, pattern, &arr, body, set_variable, eval_expr),
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            // Extract integers from the boxed values
            let start_int = match start.as_ref() {
                Value::Integer(i) => *i,
                _ => {
                    return Err(InterpreterError::TypeError(
                        "Range start must be an integer".to_string(),
                    ))
                }
            };
            let end_int = match end.as_ref() {
                Value::Integer(i) => *i,
                _ => {
                    return Err(InterpreterError::TypeError(
                        "Range end must be an integer".to_string(),
                    ))
                }
            };
            eval_for_range(
                var,
                pattern,
                start_int,
                end_int,
                inclusive,
                body,
                set_variable,
                eval_expr,
            )
        }
        _ => Err(InterpreterError::TypeError(
            "For loop requires an iterable (array or range)".to_string(),
        )),
    }
}

/// Evaluate a for loop over an array
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_for_array<F>(
    var: &str,
    pattern: Option<&Pattern>,
    arr: &[Value],
    body: &Expr,
    mut set_variable: impl FnMut(String, Value),
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut last_value = Value::Nil;
    for item in arr {
        bind_loop_variable(&mut set_variable, var, pattern, item);

        match handle_loop_iteration(body, &mut eval_expr)? {
            LoopAction::Continue(value) => last_value = value,
            LoopAction::Break => break,
            LoopAction::Return => return eval_expr(body),
        }
    }
    Ok(last_value)
}

fn bind_loop_variable(
    set_variable: &mut impl FnMut(String, Value),
    var: &str,
    _pattern: Option<&Pattern>,
    item: &Value,
) {
    // Pattern matching for destructuring would go here in the future
    set_variable(var.to_string(), item.clone());
}

enum LoopAction {
    Continue(Value),
    Break,
    Return,
}

fn handle_loop_iteration<F>(body: &Expr, eval_expr: &mut F) -> Result<LoopAction, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    match eval_expr(body) {
        Ok(value) => Ok(LoopAction::Continue(value)),
        Err(InterpreterError::Break(None, _)) => Ok(LoopAction::Break),
        Err(InterpreterError::Continue(_)) => Ok(LoopAction::Continue(Value::Nil)),
        Err(InterpreterError::Return(_)) => Ok(LoopAction::Return),
        Err(InterpreterError::RuntimeError(msg)) if msg == "break" => Ok(LoopAction::Break),
        Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {
            Ok(LoopAction::Continue(Value::Nil))
        }
        Err(e) => Err(e),
    }
}

/// Evaluate a for loop over a range
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_for_range<F>(
    var: &str,
    pattern: Option<&Pattern>,
    start: i64,
    end: i64,
    inclusive: bool,
    body: &Expr,
    mut set_variable: impl FnMut(String, Value),
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut last_value = Value::Nil;
    let range_iter = create_range_iterator(start, end, inclusive);

    for i in range_iter {
        let item = Value::Integer(i);
        bind_loop_variable(&mut set_variable, var, pattern, &item);

        match handle_loop_iteration(body, &mut eval_expr)? {
            LoopAction::Continue(value) => last_value = value,
            LoopAction::Break => break,
            LoopAction::Return => return eval_expr(body),
        }
    }
    Ok(last_value)
}

fn create_range_iterator(start: i64, end: i64, inclusive: bool) -> Box<dyn Iterator<Item = i64>> {
    if inclusive {
        Box::new(start..=end)
    } else {
        Box::new(start..end)
    }
}

/// Evaluate a while loop
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_while_loop<F>(
    condition: &Expr,
    body: &Expr,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut last_value = Value::Nil;
    loop {
        if !eval_condition(condition, &mut eval_expr)? {
            break;
        }

        match handle_loop_iteration(body, &mut eval_expr)? {
            LoopAction::Continue(value) => last_value = value,
            LoopAction::Break => break,
            LoopAction::Return => return eval_expr(body),
        }
    }
    Ok(last_value)
}

fn eval_condition<F>(condition: &Expr, eval_expr: &mut F) -> Result<bool, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let cond_value = eval_expr(condition)?;
    Ok(is_truthy(&cond_value))
}

/// Evaluate an infinite loop (loop { ... })
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_loop<F>(body: &Expr, mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    loop {
        match handle_infinite_loop_iteration(body, &mut eval_expr)? {
            InfiniteLoopAction::Continue => {}
            InfiniteLoopAction::Break(value) => return Ok(value),
            InfiniteLoopAction::Return => return eval_expr(body),
        }
    }
}

enum InfiniteLoopAction {
    Continue,
    Break(Value),
    Return,
}

fn handle_infinite_loop_iteration<F>(
    body: &Expr,
    eval_expr: &mut F,
) -> Result<InfiniteLoopAction, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    match eval_expr(body) {
        Ok(_) => Ok(InfiniteLoopAction::Continue),
        Err(InterpreterError::Break(None, value)) => Ok(InfiniteLoopAction::Break(value)),
        Err(InterpreterError::Continue(_)) => Ok(InfiniteLoopAction::Continue),
        Err(InterpreterError::Return(_)) => Ok(InfiniteLoopAction::Return),
        Err(InterpreterError::RuntimeError(msg)) if msg == "break" => {
            Ok(InfiniteLoopAction::Break(Value::Nil))
        }
        Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {
            Ok(InfiniteLoopAction::Continue)
        }
        Err(e) => Err(e),
    }
}

/// Check if a value is truthy
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Nil => false,
        Value::Integer(i) => *i != 0,
        Value::Float(f) => *f != 0.0 && !f.is_nan(),
        _ => true, // Non-nil values are truthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal};
    use std::sync::Arc;

    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Default::default(),
        )
    }

    #[test]
    fn test_for_loop_array() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        let body = make_literal_expr(10);
        let mut last_set = Value::Nil;

        let result = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| {
                last_set = val;
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(10));
        assert_eq!(last_set, Value::Integer(3)); // Last iteration value
    }

    #[test]
    fn test_for_loop_range_exclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(4)),
            inclusive: false,
        };

        let body = make_literal_expr(10);
        let mut count = 0;

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| {
                count += 1;
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(10));
        assert_eq!(count, 3); // 1, 2, 3
    }

    #[test]
    fn test_for_loop_range_inclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(3)),
            inclusive: true,
        };

        let body = make_literal_expr(10);
        let mut count = 0;

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| {
                count += 1;
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(10));
        assert_eq!(count, 3); // 1, 2, 3
    }

    #[test]
    fn test_for_loop_break() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);

        let result = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, _val| {
                *count.borrow_mut() += 1;
            },
            |_expr| {
                if *count.borrow() == 2 {
                    Err(InterpreterError::RuntimeError("break".to_string()))
                } else {
                    Ok(Value::Integer(10))
                }
            },
        )
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(10));
        assert_eq!(*count.borrow(), 2); // Stopped at 2
    }

    #[test]
    fn test_while_loop() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let mut condition_checks = 0;

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                // This is the condition
                condition_checks += 1;
                if condition_checks > 3 {
                    Ok(Value::Bool(false)) // Stop after 3 iterations
                } else {
                    Ok(Value::Bool(true)) // Continue
                }
            } else {
                // This is the body
                Ok(Value::Integer(42)) // Body result
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(42));
        assert_eq!(condition_checks, 4); // 3 iterations + 1 final check
    }

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy(&Value::Bool(true)));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(!is_truthy(&Value::Nil));
        assert!(!is_truthy(&Value::Integer(0)));
        assert!(is_truthy(&Value::Integer(1)));
        assert!(!is_truthy(&Value::Float(0.0)));
        assert!(is_truthy(&Value::Float(1.0)));
        assert!(is_truthy(&Value::from_string("test".to_string())));
    }
}
