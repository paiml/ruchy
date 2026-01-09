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

    #[test]
    fn test_eval_loop_with_break() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            if *iterations.borrow() >= 5 {
                Err(InterpreterError::Break(None, Value::Integer(100)))
            } else {
                Ok(Value::Integer(42))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(100)); // Break value
        assert_eq!(*iterations.borrow(), 5);
    }

    #[test]
    fn test_for_loop_invalid_iterable() {
        let body = make_literal_expr(10);

        let result = eval_for_loop(
            "x",
            None,
            Value::Integer(42), // Not iterable
            &body,
            |_name, _val| {},
            |_expr| Ok(Value::Integer(10)),
        );

        assert!(result.is_err());
        match result {
            Err(InterpreterError::TypeError(msg)) => {
                assert!(msg.contains("iterable"));
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_for_loop_range_invalid_start() {
        let range = Value::Range {
            start: Box::new(Value::from_string("not a number".to_string())),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        let body = make_literal_expr(10);

        let result = eval_for_loop("x", None, range, &body, |_name, _val| {}, |_expr| {
            Ok(Value::Integer(10))
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_for_loop_range_invalid_end() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Bool(true)),
            inclusive: false,
        };
        let body = make_literal_expr(10);

        let result = eval_for_loop("x", None, range, &body, |_name, _val| {}, |_expr| {
            Ok(Value::Integer(10))
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_for_loop_empty_array() {
        let arr = Value::Array(Arc::from(vec![]));
        let body = make_literal_expr(10);

        let result = eval_for_loop("x", None, arr, &body, |_name, _val| {}, |_expr| {
            Ok(Value::Integer(10))
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Nil); // Empty array returns Nil
    }

    #[test]
    fn test_while_loop_false_condition() {
        let condition = make_literal_expr(0);
        let body = make_literal_expr(42);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(0, None))) {
                Ok(Value::Bool(false)) // Condition is false immediately
            } else {
                Ok(Value::Integer(42))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Nil); // Never executed body
    }

    #[test]
    fn test_is_truthy_empty_string() {
        // Empty strings are truthy (non-nil values are truthy)
        assert!(is_truthy(&Value::from_string("".to_string())));
    }

    #[test]
    fn test_is_truthy_array() {
        let empty_arr = Value::Array(Arc::from(vec![]));
        let non_empty_arr = Value::Array(Arc::from(vec![Value::Integer(1)]));

        // All arrays are truthy (non-nil values are truthy)
        assert!(is_truthy(&empty_arr));
        assert!(is_truthy(&non_empty_arr));
    }

    #[test]
    fn test_is_truthy_float_nan() {
        // NaN is falsy
        assert!(!is_truthy(&Value::Float(f64::NAN)));
    }

    // New tests for Round 93

    // Test 19: for loop with continue
    #[test]
    fn test_for_loop_continue() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);
        let sum = std::cell::RefCell::new(0);

        let result = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| {
                *count.borrow_mut() += 1;
                if let Value::Integer(i) = val {
                    *sum.borrow_mut() += i;
                }
            },
            |_expr| {
                if *count.borrow() == 2 {
                    Err(InterpreterError::Continue(Default::default()))
                } else {
                    Ok(Value::Integer(10))
                }
            },
        )
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(10));
        assert_eq!(*count.borrow(), 3); // All iterations run
    }

    // Test 20: while loop with break
    #[test]
    fn test_while_loop_break() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                Ok(Value::Bool(true))
            } else {
                *iterations.borrow_mut() += 1;
                if *iterations.borrow() >= 3 {
                    Err(InterpreterError::Break(None, Value::Integer(99)))
                } else {
                    Ok(Value::Integer(42))
                }
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(42)); // Last successful body result
        assert_eq!(*iterations.borrow(), 3);
    }

    // Test 21: while loop with continue
    #[test]
    fn test_while_loop_continue() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);
        let condition_checks = std::cell::RefCell::new(0);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                *condition_checks.borrow_mut() += 1;
                if *condition_checks.borrow() > 5 {
                    Ok(Value::Bool(false))
                } else {
                    Ok(Value::Bool(true))
                }
            } else {
                *iterations.borrow_mut() += 1;
                if *iterations.borrow() % 2 == 0 {
                    Err(InterpreterError::Continue(Default::default()))
                } else {
                    Ok(Value::Integer(42))
                }
            }
        })
        .expect("operation should succeed in test");

        // Last successful body result was 42, continue returns Nil but doesn't override last_value
        assert_eq!(result, Value::Integer(42));
    }

    // Test 22: infinite loop with continue
    #[test]
    fn test_loop_continue() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            let i = *iterations.borrow();
            if i < 3 {
                Err(InterpreterError::Continue(Default::default()))
            } else if i == 3 {
                Err(InterpreterError::Break(None, Value::Integer(99)))
            } else {
                Ok(Value::Integer(42))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(99));
        assert_eq!(*iterations.borrow(), 3);
    }

    // Test 23: range iterator exclusive
    #[test]
    fn test_create_range_iterator_exclusive() {
        let iter = create_range_iterator(0, 5, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4]);
    }

    // Test 24: range iterator inclusive
    #[test]
    fn test_create_range_iterator_inclusive() {
        let iter = create_range_iterator(0, 5, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4, 5]);
    }

    // Test 25: range iterator empty
    #[test]
    fn test_create_range_iterator_empty() {
        let iter = create_range_iterator(5, 5, false);
        let values: Vec<i64> = iter.collect();
        assert!(values.is_empty());
    }

    // Test 26: range iterator single element
    #[test]
    fn test_create_range_iterator_single() {
        let iter = create_range_iterator(5, 5, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![5]);
    }

    // Test 27: is_truthy with negative float
    #[test]
    fn test_is_truthy_negative_float() {
        assert!(is_truthy(&Value::Float(-1.0)));
        assert!(!is_truthy(&Value::Float(-0.0)));
    }

    // Test 28: is_truthy with negative integer
    #[test]
    fn test_is_truthy_negative_integer() {
        assert!(is_truthy(&Value::Integer(-1)));
        assert!(is_truthy(&Value::Integer(-100)));
    }

    // Test 29: for loop error propagation
    #[test]
    fn test_for_loop_error_propagation() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let body = make_literal_expr(10);

        let result = eval_for_loop("x", None, arr, &body, |_name, _val| {}, |_expr| {
            Err(InterpreterError::RuntimeError("custom error".to_string()))
        });

        assert!(result.is_err());
        if let Err(InterpreterError::RuntimeError(msg)) = result {
            assert_eq!(msg, "custom error");
        } else {
            panic!("Expected RuntimeError");
        }
    }

    // Test 30: while loop error propagation
    #[test]
    fn test_while_loop_error_propagation() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                Ok(Value::Bool(true))
            } else {
                Err(InterpreterError::RuntimeError("body error".to_string()))
            }
        });

        assert!(result.is_err());
    }

    // Test 31: infinite loop error propagation
    #[test]
    fn test_loop_error_propagation() {
        let body = make_literal_expr(42);

        let result = eval_loop(&body, |_expr| {
            Err(InterpreterError::RuntimeError("infinite loop error".to_string()))
        });

        assert!(result.is_err());
        if let Err(InterpreterError::RuntimeError(msg)) = result {
            assert_eq!(msg, "infinite loop error");
        } else {
            panic!("Expected RuntimeError");
        }
    }

    // Test 32: for loop with Break enum variant
    #[test]
    fn test_for_loop_break_enum() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
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
                if *count.borrow() == 1 {
                    Err(InterpreterError::Break(None, Value::Nil))
                } else {
                    Ok(Value::Integer(10))
                }
            },
        )
        .expect("operation should succeed in test");

        assert_eq!(*count.borrow(), 1);
        // Break on first iteration means last_value is still Nil
        assert_eq!(result, Value::Nil);
    }

    // Test 33: for loop with Return handling
    #[test]
    fn test_for_loop_return() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let body = make_literal_expr(10);

        let result = eval_for_loop("x", None, arr, &body, |_name, _val| {}, |_expr| {
            Err(InterpreterError::Return(Value::Integer(42)))
        });

        // Return is handled by re-evaluating body
        assert!(result.is_err());
    }

    // Test 34: while loop condition error
    #[test]
    fn test_while_loop_condition_error() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                Err(InterpreterError::RuntimeError("condition error".to_string()))
            } else {
                Ok(Value::Integer(42))
            }
        });

        assert!(result.is_err());
    }

    // Test 35: infinite loop with runtime "break" string
    #[test]
    fn test_loop_runtime_break_string() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            if *iterations.borrow() >= 2 {
                Err(InterpreterError::RuntimeError("break".to_string()))
            } else {
                Ok(Value::Integer(42))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Nil); // RuntimeError "break" returns Nil
    }

    // Test 36: infinite loop with runtime "continue" string
    #[test]
    fn test_loop_runtime_continue_string() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            let i = *iterations.borrow();
            if i == 1 {
                Err(InterpreterError::RuntimeError("continue".to_string()))
            } else if i >= 2 {
                Err(InterpreterError::Break(None, Value::Integer(77)))
            } else {
                Ok(Value::Integer(42))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Integer(77));
    }
}

// ============================================================================
// EXTREME TDD Round 133: Additional comprehensive tests
// Target: 33 → 50+ tests
// ============================================================================
#[cfg(test)]
mod round_133_tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal};
    use std::sync::Arc;

    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Default::default(),
        )
    }

    // --- is_truthy edge cases ---
    #[test]
    fn test_is_truthy_float_positive_infinity() {
        assert!(is_truthy(&Value::Float(f64::INFINITY)));
    }

    #[test]
    fn test_is_truthy_float_negative_infinity() {
        assert!(is_truthy(&Value::Float(f64::NEG_INFINITY)));
    }

    #[test]
    fn test_is_truthy_integer_max() {
        assert!(is_truthy(&Value::Integer(i64::MAX)));
    }

    #[test]
    fn test_is_truthy_integer_min() {
        assert!(is_truthy(&Value::Integer(i64::MIN)));
    }

    #[test]
    fn test_is_truthy_tuple() {
        let tuple = Value::Tuple(Arc::from(vec![].as_slice()));
        assert!(is_truthy(&tuple));
    }

    #[test]
    fn test_is_truthy_object() {
        let obj = Value::Object(Arc::new(std::collections::HashMap::new()));
        assert!(is_truthy(&obj));
    }

    // --- create_range_iterator edge cases ---
    #[test]
    fn test_range_iterator_negative_start() {
        let iter = create_range_iterator(-5, 0, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![-5, -4, -3, -2, -1]);
    }

    #[test]
    fn test_range_iterator_all_negative() {
        let iter = create_range_iterator(-10, -5, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![-10, -9, -8, -7, -6, -5]);
    }

    #[test]
    fn test_range_iterator_reversed_empty() {
        // When start > end, range is empty
        let iter = create_range_iterator(5, 0, false);
        let values: Vec<i64> = iter.collect();
        assert!(values.is_empty());
    }

    // --- for loop edge cases ---
    #[test]
    fn test_for_loop_single_element_array() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(42)]));
        let body = make_literal_expr(100);
        let mut bound_value = Value::Nil;

        let result = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| {
                bound_value = val;
            },
            |_expr| Ok(Value::Integer(100)),
        ).unwrap();

        assert_eq!(result, Value::Integer(100));
        assert_eq!(bound_value, Value::Integer(42));
    }

    #[test]
    fn test_for_loop_range_zero_to_zero_exclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(0)),
            inclusive: false,
        };
        let body = make_literal_expr(10);
        let mut count = 0;

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| { count += 1; },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(result, Value::Nil); // No iterations
        assert_eq!(count, 0);
    }

    #[test]
    fn test_for_loop_range_zero_to_zero_inclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(0)),
            inclusive: true,
        };
        let body = make_literal_expr(10);
        let mut count = 0;

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| { count += 1; },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(result, Value::Integer(10)); // One iteration
        assert_eq!(count, 1);
    }

    #[test]
    fn test_for_loop_large_range() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(100)),
            inclusive: false,
        };
        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| { *count.borrow_mut() += 1; },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(result, Value::Integer(10));
        assert_eq!(*count.borrow(), 100);
    }

    // --- while loop edge cases ---
    #[test]
    fn test_while_loop_with_string_condition() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let checks = std::cell::RefCell::new(0);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                *checks.borrow_mut() += 1;
                if *checks.borrow() > 3 {
                    Ok(Value::Bool(false)) // Stop after 3 iterations
                } else {
                    // Return string "hello" - non-empty string is truthy
                    Ok(Value::from_string("hello".to_string()))
                }
            } else {
                Ok(Value::Integer(42))
            }
        });

        // This will run until condition returns false
        assert!(result.is_ok());
        assert_eq!(*checks.borrow(), 4); // 3 truthy + 1 falsy check
    }

    // --- infinite loop edge cases ---
    #[test]
    fn test_loop_immediate_break() {
        let body = make_literal_expr(42);

        let result = eval_loop(&body, |_expr| {
            Err(InterpreterError::Break(None, Value::Integer(0)))
        }).unwrap();

        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_loop_break_with_value() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            if *iterations.borrow() == 10 {
                Err(InterpreterError::Break(None, Value::from_string("done".to_string())))
            } else {
                Ok(Value::Integer(42))
            }
        }).unwrap();

        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "done"),
            _ => panic!("Expected string"),
        }
        assert_eq!(*iterations.borrow(), 10);
    }

    // --- handle_loop_iteration edge cases ---
    #[test]
    fn test_for_loop_continue_enum() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, _val| { *count.borrow_mut() += 1; },
            |_expr| Err(InterpreterError::Continue(Default::default())),
        ).unwrap();

        assert_eq!(*count.borrow(), 3); // All iterations still run
    }
}

// ============================================================================
// EXTREME TDD Round 135: Additional comprehensive tests
// Target: 50 → 65+ tests
// ============================================================================
#[cfg(test)]
mod round_135_tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal};
    use std::sync::Arc;

    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Default::default(),
        )
    }

    // --- is_truthy additional edge cases ---
    #[test]
    fn test_is_truthy_byte_values() {
        // Bytes are always truthy (fall into _ => true case)
        assert!(is_truthy(&Value::Byte(0)));
        assert!(is_truthy(&Value::Byte(1)));
        assert!(is_truthy(&Value::Byte(255)));
    }

    #[test]
    fn test_is_truthy_atom() {
        assert!(is_truthy(&Value::Atom("ok".to_string())));
        assert!(is_truthy(&Value::Atom("".to_string()))); // Even empty atom is truthy
    }

    #[test]
    fn test_is_truthy_enum_variant() {
        let variant = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(1)]),
        };
        assert!(is_truthy(&variant));
    }

    #[test]
    fn test_is_truthy_closure() {
        let closure = Value::Closure {
            params: vec![],
            body: Arc::from(make_literal_expr(0)),
            env: std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
        };
        assert!(is_truthy(&closure));
    }

    // --- create_range_iterator edge cases ---
    #[test]
    fn test_range_iterator_single_element_inclusive() {
        let iter = create_range_iterator(5, 5, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![5]);
    }

    #[test]
    fn test_range_iterator_large_range() {
        let iter = create_range_iterator(0, 1000, false);
        let count = iter.count();
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_range_iterator_negative_to_positive() {
        let iter = create_range_iterator(-3, 3, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![-3, -2, -1, 0, 1, 2, 3]);
    }

    // --- for loop edge cases ---
    #[test]
    fn test_for_loop_nested_array() {
        let inner1 = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let inner2 = Value::Array(Arc::from(vec![Value::Integer(2)]));
        let arr = Value::Array(Arc::from(vec![inner1, inner2]));
        let body = make_literal_expr(10);
        let mut count = 0;

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, _val| { count += 1; },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_for_loop_tracks_variable_name() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let body = make_literal_expr(10);
        let bound_names = std::cell::RefCell::new(vec![]);

        let _ = eval_for_loop(
            "item",
            None,
            arr,
            &body,
            |name, _val| { bound_names.borrow_mut().push(name.to_string()); },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(*bound_names.borrow(), vec!["item", "item"]);
    }

    #[test]
    fn test_for_loop_mixed_values() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::from_string("hello".to_string()),
            Value::Bool(true),
        ]));
        let body = make_literal_expr(10);
        let values = std::cell::RefCell::new(vec![]);

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| { values.borrow_mut().push(val); },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(values.borrow().len(), 3);
    }

    // --- while loop edge cases ---
    #[test]
    fn test_while_loop_false_condition() {
        let condition = make_literal_expr(0);
        let body = make_literal_expr(42);
        let mut condition_checked = false;

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(0, None))) {
                condition_checked = true;
                Ok(Value::Bool(false))
            } else {
                Ok(Value::Integer(42))
            }
        }).unwrap();

        assert!(condition_checked);
        assert_eq!(result, Value::Nil); // No iterations
    }

    #[test]
    fn test_while_loop_multiple_iterations() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                *iterations.borrow_mut() += 1;
                if *iterations.borrow() > 5 {
                    Ok(Value::Bool(false)) // Stop after 5 iterations
                } else {
                    Ok(Value::Bool(true))
                }
            } else {
                Ok(Value::Integer(42))
            }
        }).unwrap();

        assert_eq!(*iterations.borrow(), 6); // 5 true + 1 false
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_while_loop_continue() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let condition_checks = std::cell::RefCell::new(0);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                *condition_checks.borrow_mut() += 1;
                if *condition_checks.borrow() > 5 {
                    Ok(Value::Bool(false))
                } else {
                    Ok(Value::Bool(true))
                }
            } else {
                Err(InterpreterError::Continue(Default::default()))
            }
        }).unwrap();

        // Continue causes loop to check condition again
        assert_eq!(*condition_checks.borrow(), 6);
        assert_eq!(result, Value::Nil);
    }

    // --- infinite loop edge cases ---
    #[test]
    fn test_loop_break_immediate() {
        let body = make_literal_expr(42);

        let result = eval_loop(&body, |_expr| {
            Err(InterpreterError::Break(None, Value::Integer(123)))
        }).unwrap();

        assert_eq!(result, Value::Integer(123));
    }

    #[test]
    fn test_loop_continue_then_break() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            let i = *iterations.borrow();
            if i < 3 {
                Err(InterpreterError::Continue(Default::default()))
            } else {
                Err(InterpreterError::Break(None, Value::Integer(i)))
            }
        }).unwrap();

        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_loop_return_propagates() {
        let body = make_literal_expr(42);

        let result = eval_loop(&body, |_expr| {
            Err(InterpreterError::Return(Value::Float(3.14)))
        });

        assert!(result.is_err());
        match result {
            Err(InterpreterError::Return(val)) => assert_eq!(val, Value::Float(3.14)),
            _ => panic!("Expected Return error"),
        }
    }

    // --- for loop with array of tuples ---
    #[test]
    fn test_for_loop_array_of_tuples() {
        let t1 = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let t2 = Value::Tuple(Arc::from(vec![Value::Integer(3), Value::Integer(4)]));
        let arr = Value::Array(Arc::from(vec![t1, t2]));
        let body = make_literal_expr(10);
        let values = std::cell::RefCell::new(vec![]);

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| {
                values.borrow_mut().push(val);
            },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(values.borrow().len(), 2);
    }

    #[test]
    fn test_for_loop_deeply_nested_arrays() {
        let inner = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let outer = Value::Array(Arc::from(vec![inner.clone(), inner]));
        let body = make_literal_expr(10);
        let mut count = 0;

        let _ = eval_for_loop(
            "x",
            None,
            outer,
            &body,
            |_name, _val| { count += 1; },
            |_expr| Ok(Value::Integer(10)),
        ).unwrap();

        assert_eq!(count, 2); // Two inner arrays
    }

    // === EXTREME TDD Round 139 tests ===

    #[test]
    fn test_is_truthy_integer_zero_is_false() {
        // 0 is falsy
        assert!(!is_truthy(&Value::Integer(0)));
    }

    #[test]
    fn test_is_truthy_integer_positive() {
        assert!(is_truthy(&Value::Integer(42)));
    }

    #[test]
    fn test_is_truthy_integer_negative() {
        assert!(is_truthy(&Value::Integer(-1)));
    }

    #[test]
    fn test_is_truthy_float_zero_is_false() {
        // 0.0 is falsy
        assert!(!is_truthy(&Value::Float(0.0)));
    }

    #[test]
    fn test_is_truthy_float_non_zero() {
        assert!(is_truthy(&Value::Float(1.5)));
    }

    #[test]
    fn test_is_truthy_string_non_empty() {
        assert!(is_truthy(&Value::from_string("hello".to_string())));
    }

    #[test]
    fn test_is_truthy_nil() {
        assert!(!is_truthy(&Value::Nil));
    }
}

// ============================================================================
// EXTREME TDD Round 157: Additional edge case tests
// Target: 85 → 100+ tests
// ============================================================================
#[cfg(test)]
mod round_157_tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal};
    use std::sync::Arc;

    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Default::default(),
        )
    }

    // --- LoopAction enum edge cases ---
    #[test]
    fn test_for_loop_with_break_enum_immediate() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let body = make_literal_expr(10);

        let result = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, _val| {},
            |_expr| Err(InterpreterError::Break(None, Value::Integer(77))),
        )
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Nil); // Break on first iteration
    }

    #[test]
    fn test_for_loop_with_continue_all_iterations() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
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
            |_expr| Err(InterpreterError::Continue(Default::default())),
        )
        .expect("operation should succeed in test");

        assert_eq!(*count.borrow(), 5);
        assert_eq!(result, Value::Nil);
    }

    // --- Range edge cases ---
    #[test]
    fn test_for_loop_range_large_values() {
        let range = Value::Range {
            start: Box::new(Value::Integer(i64::MAX - 5)),
            end: Box::new(Value::Integer(i64::MAX - 2)),
            inclusive: false,
        };
        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| {
                *count.borrow_mut() += 1;
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(*count.borrow(), 3);
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_for_loop_range_negative_large() {
        let range = Value::Range {
            start: Box::new(Value::Integer(i64::MIN + 2)),
            end: Box::new(Value::Integer(i64::MIN + 5)),
            inclusive: true,
        };
        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);

        let result = eval_for_loop(
            "x",
            None,
            range,
            &body,
            |_name, _val| {
                *count.borrow_mut() += 1;
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(*count.borrow(), 4); // -9223372036854775806 to -9223372036854775803 inclusive
        assert_eq!(result, Value::Integer(10));
    }

    // --- while loop edge cases ---
    #[test]
    fn test_while_loop_with_break_enum() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                Ok(Value::Bool(true))
            } else {
                Err(InterpreterError::Break(None, Value::Integer(88)))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_while_loop_integer_truthy_condition() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_while_loop(&condition, &body, |expr| {
            if matches!(expr.kind, ExprKind::Literal(Literal::Integer(1, None))) {
                *iterations.borrow_mut() += 1;
                if *iterations.borrow() > 5 {
                    Ok(Value::Integer(0)) // 0 is falsy
                } else {
                    Ok(Value::Integer(1)) // Non-zero is truthy
                }
            } else {
                Ok(Value::Integer(42))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(*iterations.borrow(), 6);
        assert_eq!(result, Value::Integer(42));
    }

    // --- Infinite loop edge cases ---
    #[test]
    fn test_loop_multiple_continues_then_break() {
        let body = make_literal_expr(42);
        let iterations = std::cell::RefCell::new(0);

        let result = eval_loop(&body, |_expr| {
            *iterations.borrow_mut() += 1;
            let i = *iterations.borrow();
            if i < 10 {
                Err(InterpreterError::Continue(Default::default()))
            } else {
                Err(InterpreterError::Break(None, Value::Integer(999)))
            }
        })
        .expect("operation should succeed in test");

        assert_eq!(*iterations.borrow(), 10);
        assert_eq!(result, Value::Integer(999));
    }

    #[test]
    fn test_loop_with_return_handling() {
        let body = make_literal_expr(42);

        let result = eval_loop(&body, |_expr| {
            Err(InterpreterError::Return(Value::Float(3.14)))
        });

        assert!(result.is_err());
        match result {
            Err(InterpreterError::Return(val)) => {
                assert_eq!(val, Value::Float(3.14));
            }
            _ => panic!("Expected Return error"),
        }
    }

    // --- is_truthy comprehensive tests ---
    #[test]
    fn test_is_truthy_range_value() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        assert!(is_truthy(&range)); // Non-nil is truthy
    }

    #[test]
    fn test_is_truthy_closure_value() {
        let closure = Value::Closure {
            params: vec![],
            body: Arc::from(make_literal_expr(0)),
            env: std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
        };
        assert!(is_truthy(&closure));
    }

    #[test]
    fn test_is_truthy_object_value() {
        let obj = Value::Object(Arc::new(std::collections::HashMap::new()));
        assert!(is_truthy(&obj));
    }

    #[test]
    fn test_is_truthy_byte_value() {
        assert!(is_truthy(&Value::Byte(0)));
        assert!(is_truthy(&Value::Byte(255)));
    }

    // --- for loop with various array element types ---
    #[test]
    fn test_for_loop_array_of_bools() {
        let arr = Value::Array(Arc::from(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
        ]));
        let body = make_literal_expr(10);
        let values = std::cell::RefCell::new(vec![]);

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| {
                values.borrow_mut().push(val);
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(values.borrow().len(), 3);
    }

    #[test]
    fn test_for_loop_array_of_floats() {
        let arr = Value::Array(Arc::from(vec![
            Value::Float(1.1),
            Value::Float(2.2),
            Value::Float(3.3),
        ]));
        let body = make_literal_expr(10);
        let sum = std::cell::RefCell::new(0.0);

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, val| {
                if let Value::Float(f) = val {
                    *sum.borrow_mut() += f;
                }
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert!((*sum.borrow() - 6.6).abs() < 0.001);
    }

    #[test]
    fn test_for_loop_array_of_strings() {
        let arr = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
            Value::from_string("c".to_string()),
        ]));
        let body = make_literal_expr(10);
        let count = std::cell::RefCell::new(0);

        let _ = eval_for_loop(
            "x",
            None,
            arr,
            &body,
            |_name, _val| {
                *count.borrow_mut() += 1;
            },
            |_expr| Ok(Value::Integer(10)),
        )
        .expect("operation should succeed in test");

        assert_eq!(*count.borrow(), 3);
    }

    // --- create_range_iterator edge cases ---
    #[test]
    fn test_create_range_iterator_zero_start() {
        let iter = create_range_iterator(0, 3, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![0, 1, 2]);
    }

    #[test]
    fn test_create_range_iterator_negative_to_negative() {
        let iter = create_range_iterator(-3, -1, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![-3, -2, -1]);
    }

    #[test]
    fn test_create_range_iterator_empty_when_start_equals_end_exclusive() {
        let iter = create_range_iterator(10, 10, false);
        let values: Vec<i64> = iter.collect();
        assert!(values.is_empty());
    }

    #[test]
    fn test_create_range_iterator_single_when_start_equals_end_inclusive() {
        let iter = create_range_iterator(10, 10, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![10]);
    }
}
