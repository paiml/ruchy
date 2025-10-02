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
/// Cyclomatic complexity: 8 (within Toyota Way limits)
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
        // Handle pattern matching if present
        if let Some(_pat) = pattern {
            // Pattern matching for destructuring would go here
            // For now, just bind to var
            set_variable(var.to_string(), item.clone());
        } else {
            // Simple variable binding
            set_variable(var.to_string(), item.clone());
        }

        // Execute body
        match eval_expr(body) {
            Ok(value) => last_value = value,
            Err(InterpreterError::Break(_)) => break,
            Err(InterpreterError::Continue) => {}
            Err(InterpreterError::Return(_)) => return eval_expr(body), // Propagate Return
            Err(InterpreterError::RuntimeError(msg)) if msg == "break" => break,
            Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {}
            Err(e) => return Err(e),
        }
    }
    Ok(last_value)
}

/// Evaluate a for loop over a range
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
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

    let range_iter: Box<dyn Iterator<Item = i64>> = if inclusive {
        Box::new(start..=end)
    } else {
        Box::new(start..end)
    };

    for i in range_iter {
        let item = Value::Integer(i);
        // Handle pattern matching if present
        if let Some(_pat) = pattern {
            // Pattern matching for destructuring would go here
            // For now, just bind to var
            set_variable(var.to_string(), item);
        } else {
            // Simple variable binding
            set_variable(var.to_string(), item);
        }

        // Execute body
        match eval_expr(body) {
            Ok(value) => last_value = value,
            Err(InterpreterError::Break(_)) => break,
            Err(InterpreterError::Continue) => {}
            Err(InterpreterError::Return(_)) => return eval_expr(body), // Propagate Return
            Err(InterpreterError::RuntimeError(msg)) if msg == "break" => break,
            Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {}
            Err(e) => return Err(e),
        }
    }
    Ok(last_value)
}

/// Evaluate a while loop
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
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
        let cond_value = eval_expr(condition)?;
        if !is_truthy(&cond_value) {
            break;
        }

        match eval_expr(body) {
            Ok(value) => last_value = value,
            Err(InterpreterError::Break(_)) => break,
            Err(InterpreterError::Continue) => {}
            Err(InterpreterError::Return(_)) => return eval_expr(body), // Propagate Return
            Err(InterpreterError::RuntimeError(msg)) if msg == "break" => break,
            Err(InterpreterError::RuntimeError(msg)) if msg == "continue" => {}
            Err(e) => return Err(e),
        }
    }
    Ok(last_value)
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
    use std::rc::Rc;

    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(val)), Default::default())
    }

    #[test]
    fn test_for_loop_array() {
        let arr = Value::Array(Rc::from(vec![
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
        .unwrap();

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
        .unwrap();

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
        .unwrap();

        assert_eq!(result, Value::Integer(10));
        assert_eq!(count, 3); // 1, 2, 3
    }

    #[test]
    fn test_for_loop_break() {
        let arr = Value::Array(Rc::from(vec![
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
        .unwrap();

        assert_eq!(result, Value::Integer(10));
        assert_eq!(*count.borrow(), 2); // Stopped at 2
    }

    #[test]
    fn test_while_loop() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let mut condition_checks = 0;

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1))) {
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
        .unwrap();

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
