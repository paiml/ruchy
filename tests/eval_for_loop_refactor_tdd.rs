//! EXTREME TDD: `eval_for_loop` Complexity Refactoring
//! Target: Reduce `eval_for_loop` cognitive complexity from 42 → ≤10
//!
//! This test-driven refactoring decomposes `eval_for_loop` into focused helper functions,
//! each with single responsibility and complexity ≤10.

use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
use ruchy::runtime::eval_control_flow_new::eval_for_loop;
use ruchy::runtime::interpreter::Value;
use ruchy::runtime::InterpreterError;

/// Test suite for decomposed for loop evaluation functions
#[cfg(test)]
mod eval_for_loop_refactor_tests {
    use super::*;

    // Helper function to create a simple expression
    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(val)), Span::default())
    }

    // =============================================================================
    // STEP 1: Tests for individual helper functions (will guide implementation)
    // =============================================================================

    #[test]
    fn test_eval_array_iteration() {
        // Helper function to handle array iteration
        use ruchy::runtime::eval_control_flow_new::eval_array_iteration;

        let array =
            Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into());

        let mut last_val = Value::Nil;
        let mut eval_count = 0;

        let result = eval_array_iteration(
            &array,
            "x",
            &mut |_var: &str, val: Value, _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                eval_count += 1;
                last_val = val.clone();
                Ok(val)
            },
            &mut |_expr: &Expr| Ok(Value::Integer(42)),
        );

        assert!(result.is_ok());
        assert_eq!(eval_count, 3);
        assert_eq!(last_val, Value::Integer(3));
    }

    #[test]
    fn test_eval_range_iteration() {
        // Helper function to handle range iteration
        use ruchy::runtime::eval_control_flow_new::eval_range_iteration;

        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(4)),
            inclusive: false,
        };

        let mut last_val = Value::Nil;
        let mut eval_count = 0;

        let result = eval_range_iteration(
            &range,
            "i",
            &mut |_var: &str, val: Value, _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                eval_count += 1;
                last_val = val.clone();
                Ok(val)
            },
            &mut |_expr: &Expr| Ok(Value::Integer(42)),
        );

        assert!(result.is_ok());
        assert_eq!(eval_count, 3); // 1, 2, 3 (not 4 because exclusive)
        assert_eq!(last_val, Value::Integer(3));
    }

    #[test]
    fn test_extract_range_bounds() {
        // Helper function to extract integer bounds from range values
        use ruchy::runtime::eval_control_flow_new::extract_range_bounds;

        let good_range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };

        let result = extract_range_bounds(&good_range);
        assert!(result.is_ok());
        let (start, end, inclusive) = result.unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 10);
        assert!(inclusive);

        // Test with non-integer start
        let bad_start_range = Value::Range {
            start: Box::new(Value::Float(1.5)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };

        let result = extract_range_bounds(&bad_start_range);
        assert!(result.is_err());

        // Test with non-integer end
        let bad_end_range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::String("ten".to_string().into())),
            inclusive: false,
        };

        let result = extract_range_bounds(&bad_end_range);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_loop_control() {
        // Helper function to handle break/continue control flow
        use ruchy::runtime::eval_control_flow_new::handle_loop_control;

        // Test normal iteration
        let result = handle_loop_control(Ok(Value::Integer(42)), &mut Value::Nil);
        assert_eq!(result, Ok(None));

        // Test break
        let result = handle_loop_control(
            Err(InterpreterError::Break(Value::Integer(99))),
            &mut Value::Nil,
        );
        assert_eq!(result, Ok(Some(Value::Integer(99))));

        // Test continue
        let result = handle_loop_control(Err(InterpreterError::Continue), &mut Value::Nil);
        assert_eq!(result, Ok(None));

        // Test other errors
        let result = handle_loop_control(
            Err(InterpreterError::RuntimeError("test error".to_string())),
            &mut Value::Nil,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_range_iterator() {
        // Helper function to create iterator from range bounds
        use ruchy::runtime::eval_control_flow_new::create_range_iterator;

        // Inclusive range
        let iter = create_range_iterator(1, 3, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![1, 2, 3]);

        // Exclusive range
        let iter = create_range_iterator(1, 3, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![1, 2]);

        // Empty range
        let iter = create_range_iterator(5, 3, false);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, Vec::<i64>::new());

        // Single element inclusive
        let iter = create_range_iterator(5, 5, true);
        let values: Vec<i64> = iter.collect();
        assert_eq!(values, vec![5]);
    }

    // =============================================================================
    // STEP 2: Integration tests for the refactored main function
    // =============================================================================

    #[test]
    fn test_refactored_eval_for_loop_array() {
        let var = "x";
        let array_expr = make_literal_expr(0); // Dummy expr
        let body_expr = make_literal_expr(0); // Dummy expr

        let mut eval_count = 0;
        let array_value =
            Value::Array(vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)].into());

        let result = eval_for_loop(
            var,
            &array_expr,
            &body_expr,
            |_expr| Ok(array_value.clone()),
            |_var, val, _eval| {
                eval_count += 1;
                Ok(val)
            },
        );

        assert!(result.is_ok());
        assert_eq!(eval_count, 3);
        assert_eq!(result.unwrap(), Value::Integer(30));
    }

    #[test]
    fn test_refactored_eval_for_loop_range() {
        let var = "i";
        let range_expr = make_literal_expr(0); // Dummy expr
        let body_expr = make_literal_expr(0); // Dummy expr

        let mut sum = 0;
        let range_value = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(5)),
            inclusive: false,
        };

        let result = eval_for_loop(
            var,
            &range_expr,
            &body_expr,
            |_expr| Ok(range_value.clone()),
            |_var, val, _eval| {
                if let Value::Integer(i) = val {
                    sum += i;
                }
                Ok(val)
            },
        );

        assert!(result.is_ok());
        assert_eq!(sum, 10); // 1 + 2 + 3 + 4
    }

    #[test]
    fn test_for_loop_with_break() {
        let var = "x";
        let array_expr = make_literal_expr(0);
        let body_expr = make_literal_expr(0);

        let array_value =
            Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into());

        let mut iteration_count = 0;
        let result = eval_for_loop(
            var,
            &array_expr,
            &body_expr,
            |_expr| Ok(array_value.clone()),
            |_var, val, _eval| {
                iteration_count += 1;
                if let Value::Integer(2) = val {
                    Err(InterpreterError::Break(Value::Integer(99)))
                } else {
                    Ok(val)
                }
            },
        );

        assert!(result.is_ok());
        assert_eq!(iteration_count, 2); // Should break on second iteration
        assert_eq!(result.unwrap(), Value::Integer(99));
    }

    #[test]
    fn test_for_loop_with_continue() {
        let var = "x";
        let array_expr = make_literal_expr(0);
        let body_expr = make_literal_expr(0);

        let array_value =
            Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into());

        let mut sum = 0;
        let result = eval_for_loop(
            var,
            &array_expr,
            &body_expr,
            |_expr| Ok(array_value.clone()),
            |_var, val, _eval| {
                if let Value::Integer(i) = val {
                    if i == 2 {
                        return Err(InterpreterError::Continue);
                    }
                    sum += i;
                }
                Ok(val)
            },
        );

        assert!(result.is_ok());
        assert_eq!(sum, 4); // 1 + 3 (skipped 2)
    }

    #[test]
    fn test_for_loop_invalid_iterator() {
        let var = "x";
        let invalid_expr = make_literal_expr(0);
        let body_expr = make_literal_expr(0);

        let result = eval_for_loop(
            var,
            &invalid_expr,
            &body_expr,
            |_expr| Ok(Value::Integer(42)), // Not iterable
            |_var, _val, _eval| Ok(Value::Nil),
        );

        assert!(result.is_err());
        if let Err(InterpreterError::TypeError(msg)) = result {
            assert!(msg.contains("Cannot iterate"));
        } else {
            panic!("Expected TypeError");
        }
    }

    // =============================================================================
    // STEP 3: Complexity verification tests
    // =============================================================================

    #[test]
    fn test_complexity_compliance_verification() {
        // This test documents that after refactoring, all functions have complexity ≤10
        // Manual verification required via PMAT analysis:
        //
        // Expected after refactoring:
        // - eval_for_loop: ≤10 (down from 42 cognitive complexity)
        // - eval_array_iteration: ≤8
        // - eval_range_iteration: ≤8
        // - extract_range_bounds: ≤5
        // - handle_loop_control: ≤5
        // - create_range_iterator: ≤3
        //
        // Total cognitive complexity reduction: 42 → distributed across helpers

        // Test passes without assertion - this is documentation
    }

    // =============================================================================
    // STEP 4: Property-based testing for robustness
    // =============================================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::TestResult;
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        #[allow(clippy::needless_pass_by_value)]
        fn test_for_loop_never_panics_on_valid_arrays(values: Vec<i32>) -> TestResult {
            let array = Value::Array(
                values
                    .iter()
                    .map(|&i| Value::Integer(i64::from(i)))
                    .collect::<Vec<_>>()
                    .into(),
            );

            let dummy_expr = make_literal_expr(0);
            let result = eval_for_loop(
                "x",
                &dummy_expr,
                &dummy_expr,
                |_| Ok(array.clone()),
                |_, val, _| Ok(val),
            );

            assert!(result.is_ok());
            TestResult::passed()
        }

        #[quickcheck]
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        fn test_range_iteration_correctness(start: i8, end: i8) -> TestResult {
            // Limit range to prevent huge iterations
            let start = i64::from(start);
            let end = i64::from(end);

            if (end - start).abs() > 100 {
                return TestResult::discard();
            }

            let range = Value::Range {
                start: Box::new(Value::Integer(start)),
                end: Box::new(Value::Integer(end)),
                inclusive: false,
            };

            let dummy_expr = make_literal_expr(0);
            let mut count = 0;

            let result = eval_for_loop(
                "i",
                &dummy_expr,
                &dummy_expr,
                |_| Ok(range.clone()),
                |_, _, _| {
                    count += 1;
                    Ok(Value::Nil)
                },
            );

            assert!(result.is_ok());
            let expected_count = if end > start {
                (end - start) as usize
            } else {
                0
            };
            assert_eq!(count, expected_count);

            TestResult::passed()
        }
    }
}
