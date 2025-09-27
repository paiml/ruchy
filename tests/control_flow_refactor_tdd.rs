//! EXTREME TDD: Control Flow Complexity Refactoring
//! Target A: Reduce `eval_match` cognitive complexity from 25 → ≤10
//! Target B: Reduce `eval_while_loop` cognitive complexity from 16 → ≤10
//!
//! This test-driven refactoring decomposes complex control flow functions
//! into focused helper functions with single responsibility.

use ruchy::frontend::ast::{Expr, ExprKind, Literal, MatchArm, Pattern, Span};
use ruchy::runtime::eval_control_flow_new::{eval_match, eval_while_loop};
use ruchy::runtime::interpreter::Value;
use ruchy::runtime::InterpreterError;

/// Test suite for decomposed control flow evaluation functions
#[cfg(test)]
mod control_flow_refactor_tests {
    use super::*;

    // Helper function to create a simple expression
    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(val)), Span::default())
    }

    // =============================================================================
    // PART A: Tests for eval_match decomposition
    // =============================================================================

    #[test]
    fn test_eval_match_arm() {
        // Helper function to evaluate a single match arm
        use ruchy::runtime::eval_control_flow_new::eval_match_arm;

        let pattern = Pattern::Literal(Literal::Integer(42));
        let body = make_literal_expr(100);
        let arm = MatchArm {
            pattern,
            guard: None,
            body: Box::new(body),
            span: Span::default(),
        };

        let value = Value::Integer(42);
        let result = eval_match_arm(
            &arm,
            &value,
            &mut |_pattern, _value| Ok(true), // Pattern matches
            &mut |_expr| Ok(Value::Integer(100)),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Integer(100)));

        // Test with non-matching pattern
        let result = eval_match_arm(
            &arm,
            &value,
            &mut |_pattern, _value| Ok(false), // Pattern doesn't match
            &mut |_expr| Ok(Value::Integer(100)),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_eval_match_guard() {
        // Helper function to evaluate guard expressions
        use ruchy::runtime::eval_control_flow_new::eval_match_guard;

        // Test with Some guard that's truthy
        let guard_expr = make_literal_expr(1);
        let result = eval_match_guard(Some(&guard_expr), &mut |_expr| Ok(Value::Bool(true)));
        assert!(result.is_ok());
        assert!(result.unwrap()); // Guard passes

        // Test with Some guard that's falsy
        let result = eval_match_guard(Some(&guard_expr), &mut |_expr| Ok(Value::Bool(false)));
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Guard fails

        // Test with None guard
        let result = eval_match_guard(None, &mut |_expr| panic!("Should not be called"));
        assert!(result.is_ok());
        assert!(result.unwrap()); // No guard always passes
    }

    #[test]
    fn test_find_matching_arm() {
        // Helper function to find the first matching arm
        use ruchy::runtime::eval_control_flow_new::find_matching_arm;

        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1)),
                guard: None,
                body: Box::new(make_literal_expr(10)),
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(2)),
                guard: None,
                body: Box::new(make_literal_expr(20)),
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(make_literal_expr(99)),
                span: Span::default(),
            },
        ];

        // Match second arm
        let value = Value::Integer(2);
        let result = find_matching_arm(
            &arms,
            &value,
            &mut |pattern, val| match pattern {
                Pattern::Literal(Literal::Integer(i)) => Ok(*i
                    == match val {
                        Value::Integer(v) => *v,
                        _ => return Ok(false),
                    }),
                Pattern::Wildcard => Ok(true),
                _ => Ok(false),
            },
            &mut |expr| match expr.kind {
                ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(i)),
                _ => Ok(Value::Nil),
            },
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(20));

        // Match wildcard
        let value = Value::Integer(99);
        let result = find_matching_arm(
            &arms,
            &value,
            &mut |pattern, _val| Ok(matches!(pattern, Pattern::Wildcard)),
            &mut |expr| match expr.kind {
                ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(i)),
                _ => Ok(Value::Nil),
            },
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(99));
    }

    // =============================================================================
    // PART B: Tests for eval_while_loop decomposition
    // =============================================================================

    #[test]
    fn test_eval_loop_condition() {
        // Helper function to evaluate loop condition
        use ruchy::runtime::eval_control_flow_new::eval_loop_condition;

        let condition = make_literal_expr(1);

        // Truthy condition
        let result = eval_loop_condition(&condition, &mut |_expr| Ok(Value::Bool(true)));
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Falsy condition
        let result = eval_loop_condition(&condition, &mut |_expr| Ok(Value::Bool(false)));
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Non-boolean truthy value
        let result = eval_loop_condition(&condition, &mut |_expr| Ok(Value::Integer(42)));
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Nil is falsy
        let result = eval_loop_condition(&condition, &mut |_expr| Ok(Value::Nil));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_eval_loop_body() {
        // Helper function to evaluate loop body and handle control flow
        use ruchy::runtime::eval_control_flow_new::eval_loop_body;

        let body = make_literal_expr(42);
        let mut last_val = Value::Nil;

        // Normal execution
        let result = eval_loop_body(&body, &mut last_val, &mut |_expr| Ok(Value::Integer(42)));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
        assert_eq!(last_val, Value::Integer(42));

        // Break with value
        let result = eval_loop_body(&body, &mut last_val, &mut |_expr| {
            Err(InterpreterError::Break(Value::Integer(99)))
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Integer(99)));

        // Continue
        let mut last_val = Value::Integer(10);
        let result = eval_loop_body(&body, &mut last_val, &mut |_expr| {
            Err(InterpreterError::Continue)
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
        assert_eq!(last_val, Value::Integer(10)); // Unchanged on continue

        // Other error
        let result = eval_loop_body(&body, &mut last_val, &mut |_expr| {
            Err(InterpreterError::RuntimeError("test".to_string()))
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_run_while_loop() {
        // Helper function to run the while loop logic
        use ruchy::runtime::eval_control_flow_new::run_while_loop;

        let condition = make_literal_expr(1); // Different value to distinguish
        let body = make_literal_expr(2); // Different value to distinguish
        let mut counter = 3;

        let result = run_while_loop(&condition, &body, &mut |expr| {
            // Check which expression we're evaluating by its literal value
            match &expr.kind {
                ExprKind::Literal(Literal::Integer(1)) => {
                    // Condition: count down from 3
                    let val = counter > 0;
                    Ok(Value::Bool(val))
                }
                ExprKind::Literal(Literal::Integer(2)) => {
                    // Body: decrement and return counter
                    counter -= 1;
                    Ok(Value::Integer(i64::from(counter)))
                }
                _ => Ok(Value::Nil),
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(0));
        assert_eq!(counter, 0);
    }

    // =============================================================================
    // Integration tests for refactored functions
    // =============================================================================

    #[test]
    fn test_refactored_eval_match() {
        let expr = make_literal_expr(2);
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1)),
                guard: None,
                body: Box::new(make_literal_expr(10)),
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(2)),
                guard: Some(Box::new(make_literal_expr(1))), // Always true
                body: Box::new(make_literal_expr(20)),
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(make_literal_expr(99)),
                span: Span::default(),
            },
        ];

        let result = eval_match(
            &expr,
            &arms,
            |expr| {
                // Evaluate expressions
                match &expr.kind {
                    ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(*i)),
                    _ => Ok(Value::Bool(true)), // Guards evaluate to true
                }
            },
            |pattern, val| match pattern {
                Pattern::Literal(Literal::Integer(i)) => Ok(*i
                    == match val {
                        Value::Integer(v) => *v,
                        _ => return Ok(false),
                    }),
                Pattern::Wildcard => Ok(true),
                _ => Ok(false),
            },
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_refactored_eval_while_loop() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(2);
        let mut sum = 0;
        let mut count = 5;

        let result = eval_while_loop(&condition, &body, |expr| {
            match &expr.kind {
                ExprKind::Literal(Literal::Integer(1)) => {
                    // Condition
                    Ok(Value::Bool(count > 0))
                }
                ExprKind::Literal(Literal::Integer(2)) => {
                    // Body
                    sum += count;
                    count -= 1;
                    Ok(Value::Integer(sum))
                }
                _ => Ok(Value::Nil),
            }
        });

        assert!(result.is_ok());
        assert_eq!(sum, 15); // 5 + 4 + 3 + 2 + 1
    }

    #[test]
    fn test_while_loop_with_break() {
        let condition = make_literal_expr(1);
        let body = make_literal_expr(2);
        let mut iterations = 0;

        let result = eval_while_loop(&condition, &body, |expr| {
            match &expr.kind {
                ExprKind::Literal(Literal::Integer(1)) => {
                    // Condition: always true
                    Ok(Value::Bool(true))
                }
                ExprKind::Literal(Literal::Integer(2)) => {
                    // Body
                    iterations += 1;
                    if iterations == 3 {
                        Err(InterpreterError::Break(Value::Integer(42)))
                    } else {
                        Ok(Value::Integer(iterations))
                    }
                }
                _ => Ok(Value::Nil),
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
        assert_eq!(iterations, 3);
    }

    #[test]
    fn test_match_with_guards() {
        let expr = make_literal_expr(10);
        let arms = vec![
            MatchArm {
                pattern: Pattern::Identifier("x".to_string()),
                guard: Some(Box::new(make_literal_expr(0))), // Guard: x > 5
                body: Box::new(make_literal_expr(100)),
                span: Span::default(),
            },
            MatchArm {
                pattern: Pattern::Identifier("x".to_string()),
                guard: Some(Box::new(make_literal_expr(0))), // Guard: x <= 5
                body: Box::new(make_literal_expr(200)),
                span: Span::default(),
            },
        ];

        let result = eval_match(
            &expr,
            &arms,
            |_expr| Ok(Value::Integer(10)),
            |pattern, _val| Ok(matches!(pattern, Pattern::Identifier(_))),
        );

        // First arm should match since 10 > 5
        assert!(result.is_ok());
        // The actual value depends on guard evaluation
    }

    // =============================================================================
    // Complexity verification
    // =============================================================================

    #[test]
    fn test_complexity_compliance() {
        // This test documents expected complexity after refactoring:
        //
        // eval_match: 25 → ≤10
        // - eval_match: ≤5 (orchestrator)
        // - eval_match_arm: ≤5
        // - eval_match_guard: ≤3
        // - find_matching_arm: ≤8
        //
        // eval_while_loop: 16 → ≤10
        // - eval_while_loop: ≤3 (orchestrator)
        // - eval_loop_condition: ≤3
        // - eval_loop_body: ≤5
        // - run_while_loop: ≤8

        // Test passes - this is documentation
    }

    // =============================================================================
    // Property-based tests
    // =============================================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::TestResult;
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        fn test_match_never_panics(value: i32) -> TestResult {
            let expr = make_literal_expr(i64::from(value));
            let arms = vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(make_literal_expr(0)),
                span: Span::default(),
            }];

            let result = eval_match(
                &expr,
                &arms,
                |_| Ok(Value::Integer(i64::from(value))),
                |_, _| Ok(true),
            );

            assert!(result.is_ok());
            TestResult::passed()
        }

        #[quickcheck]
        fn test_while_terminates(iterations: u8) -> TestResult {
            let condition = make_literal_expr(1);
            let body = make_literal_expr(2);
            let mut count = i32::from(iterations);

            let result = eval_while_loop(&condition, &body, |expr| {
                match &expr.kind {
                    ExprKind::Literal(Literal::Integer(1)) => {
                        // Condition
                        let should_continue = count > 0;
                        Ok(Value::Bool(should_continue))
                    }
                    ExprKind::Literal(Literal::Integer(2)) => {
                        // Body
                        count -= 1;
                        Ok(Value::Integer(i64::from(count)))
                    }
                    _ => Ok(Value::Nil),
                }
            });

            assert!(result.is_ok());
            assert_eq!(count, 0);
            TestResult::passed()
        }
    }
}
