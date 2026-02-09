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

        let result = eval_if_expr(&condition, &then_branch, None, eval_expr)
            .expect("operation should succeed in test");
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

        let result =
            eval_list_expr(&elements, eval_expr).expect("operation should succeed in test");
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

        let result =
            eval_block_expr(&statements, eval_expr).expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(20)); // Last statement result
    }

    #[test]
    fn test_pattern_matches_simple() {
        let wildcard_pattern = Pattern::Wildcard;
        assert!(
            pattern_matches_simple(&wildcard_pattern, &Value::Integer(42))
                .expect("operation should succeed in test")
        );

        let literal_pattern = Pattern::Literal(Literal::Integer(42, None));
        assert!(
            pattern_matches_simple(&literal_pattern, &Value::Integer(42))
                .expect("operation should succeed in test")
        );
        assert!(
            !pattern_matches_simple(&literal_pattern, &Value::Integer(43))
                .expect("operation should succeed in test")
        );
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

        let result =
            eval_tuple_expr(&elements, eval_expr).expect("operation should succeed in test");
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
        let result = eval_tuple_expr(&[], eval_expr).expect("operation should succeed in test");
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

        let result = eval_range_expr(&start, &end, true, eval_expr)
            .expect("operation should succeed in test");
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

        let result = eval_range_expr(&start, &end, false, eval_expr)
            .expect("operation should succeed in test");
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
        let result = eval_loop_condition(&condition, &mut eval_expr)
            .expect("operation should succeed in test");
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
        let result = eval_loop_condition(&condition, &mut eval_expr)
            .expect("operation should succeed in test");
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
        assert!(match_literal_pattern(&lit, &Value::Integer(42))
            .expect("operation should succeed in test"));
        assert!(!match_literal_pattern(&lit, &Value::Integer(43))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_match_literal_pattern_bool() {
        let lit_true = Literal::Bool(true);
        assert!(match_literal_pattern(&lit_true, &Value::Bool(true))
            .expect("operation should succeed in test"));
        assert!(!match_literal_pattern(&lit_true, &Value::Bool(false))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_match_literal_pattern_string() {
        let lit = Literal::String("hello".to_string());
        assert!(
            match_literal_pattern(&lit, &Value::String(Arc::from("hello")))
                .expect("operation should succeed in test")
        );
        assert!(
            !match_literal_pattern(&lit, &Value::String(Arc::from("world")))
                .expect("operation should succeed in test")
        );
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
        assert!(match_list_pattern(&patterns, &Value::Array(arr))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_match_list_pattern_length_mismatch() {
        let patterns = vec![Pattern::Literal(Literal::Integer(1, None))];
        let arr = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(!match_list_pattern(&patterns, &Value::Array(arr))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_match_tuple_pattern_basic() {
        let patterns = vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Wildcard,
        ];
        let tuple = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(match_tuple_pattern(&patterns, &Value::Tuple(tuple))
            .expect("operation should succeed in test"));
    }

    #[test]
    fn test_match_tuple_pattern_length_mismatch() {
        let patterns = vec![Pattern::Wildcard];
        let tuple = Arc::from([Value::Integer(1), Value::Integer(2)]);
        assert!(!match_tuple_pattern(&patterns, &Value::Tuple(tuple))
            .expect("operation should succeed in test"));
    }

    // ===== RANGE HELPERS TESTS =====

    #[test]
    fn test_extract_range_bounds_inclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(10)),
            inclusive: true,
        };
        let (start, end, inclusive) =
            extract_range_bounds(&range).expect("operation should succeed in test");
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
        let (start, end, inclusive) =
            extract_range_bounds(&range).expect("operation should succeed in test");
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

        let result = eval_if_expr(&condition, &then_branch, None, eval_expr)
            .expect("operation should succeed in test");
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

        let result = eval_if_expr(&condition, &then_branch, Some(&else_branch), eval_expr)
            .expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(99));
    }

    // ===== BLOCK EXPRESSION EDGE CASES =====

    #[test]
    fn test_eval_block_expr_empty() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };
        let result = eval_block_expr(&[], eval_expr).expect("operation should succeed in test");
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
        let result =
            eval_block_expr(&statements, eval_expr).expect("operation should succeed in test");
        assert_eq!(result, Value::Integer(42));
    }

    // ===== LIST EXPRESSION EDGE CASES =====

    #[test]
    fn test_eval_list_expr_empty() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };
        let result = eval_list_expr(&[], eval_expr).expect("operation should succeed in test");
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

        let result = eval_array_init_expr(&element, &size, eval_expr)
            .expect("operation should succeed in test");
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

        let result = eval_array_init_expr(&element, &size, eval_expr)
            .expect("operation should succeed in test");
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

    // ===== ADDITIONAL UNIQUE TESTS =====

    #[test]
    fn test_extract_range_bounds_invalid_type() {
        let not_range = Value::Integer(42);
        let result = extract_range_bounds(&not_range);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_range_iterator_negative() {
        let range = create_range_iterator(-5, 0, false);
        let values: Vec<_> = range.collect();
        assert_eq!(values, vec![-5, -4, -3, -2, -1]);
    }

    #[test]
    fn test_handle_loop_control_break_with_value() {
        let mut last_val = Value::Nil;
        let result = handle_loop_control(
            Err(InterpreterError::Break(None, Value::Integer(99))),
            &mut last_val,
        );
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_handle_loop_control_continue_preserves_last_val() {
        let mut last_val = Value::Integer(42);
        let result = handle_loop_control(Err(InterpreterError::Continue(None)), &mut last_val);
        assert!(result.unwrap().is_none());
        assert_eq!(last_val, Value::Integer(42));
    }

    #[test]
    fn test_handle_loop_control_updates_last_val() {
        let mut last_val = Value::Nil;
        let result = handle_loop_control(Ok(Value::Integer(42)), &mut last_val);
        assert!(result.unwrap().is_none());
        assert_eq!(last_val, Value::Integer(42));
    }

    #[test]
    fn test_handle_loop_control_propagates_error() {
        let mut last_val = Value::Nil;
        let result = handle_loop_control(
            Err(InterpreterError::RuntimeError("test".to_string())),
            &mut last_val,
        );
        assert!(result.is_err());
    }

    // === EXTREME TDD Round 126 - Additional Coverage Tests ===

    #[test]
    fn test_eval_if_expr_true_branch_r126() {
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Bool(true)) };

        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4),
        );
        let then_branch = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(5, 7),
        );

        let result = eval_if_expr(&condition, &then_branch, None, eval_expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_if_expr_false_no_else_r126() {
        let mut call_count = 0;
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            call_count += 1;
            if call_count == 1 {
                Ok(Value::Bool(false))
            } else {
                Ok(Value::Integer(42))
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

        let result = eval_if_expr(&condition, &then_branch, None, eval_expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[test]
    fn test_eval_loop_condition_true_r126() {
        let mut eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Bool(true)) };

        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4),
        );

        let result = eval_loop_condition(&condition, &mut eval_expr);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_eval_loop_condition_false_r126() {
        let mut eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Bool(false)) };

        let condition = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 5),
        );

        let result = eval_loop_condition(&condition, &mut eval_expr);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_eval_loop_body_normal_r126() {
        let mut eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Integer(42)) };

        let body = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        );
        let mut last_val = Value::Nil;

        let result = eval_loop_body(&body, &mut last_val, &mut eval_expr);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(last_val, Value::Integer(42));
    }

    #[test]
    fn test_eval_loop_body_break_r126() {
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Err(InterpreterError::Break(None, Value::Integer(99)))
        };

        let body = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(99, None)),
            Span::new(0, 2),
        );
        let mut last_val = Value::Nil;

        let result = eval_loop_body(&body, &mut last_val, &mut eval_expr);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_eval_loop_body_continue_r126() {
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Err(InterpreterError::Continue(None))
        };

        let body = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
        );
        let mut last_val = Value::Integer(10);

        let result = eval_loop_body(&body, &mut last_val, &mut eval_expr);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        // last_val unchanged after continue
    }

    #[test]
    fn test_create_range_iterator_inclusive_r126() {
        let range = create_range_iterator(1, 5, true);
        let values: Vec<_> = range.collect();
        assert_eq!(values.len(), 5);
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_create_range_iterator_exclusive_r126() {
        let range = create_range_iterator(1, 5, false);
        let values: Vec<_> = range.collect();
        assert_eq!(values.len(), 4);
        assert_eq!(values, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_create_range_iterator_empty_r126() {
        let range = create_range_iterator(5, 5, false);
        let values: Vec<_> = range.collect();
        assert!(values.is_empty());
    }

    #[test]
    fn test_create_range_iterator_single_inclusive_r126() {
        let range = create_range_iterator(5, 5, true);
        let values: Vec<_> = range.collect();
        assert_eq!(values.len(), 1);
        assert_eq!(values, vec![5]);
    }

    #[test]
    fn test_handle_loop_control_break_no_value_r126() {
        let mut last_val = Value::Integer(10);
        let result = handle_loop_control(
            Err(InterpreterError::Break(None, Value::Nil)),
            &mut last_val,
        );
        assert!(result.is_ok());
        let opt = result.unwrap();
        assert!(opt.is_some());
        assert_eq!(opt.unwrap(), Value::Nil);
    }

    #[test]
    fn test_eval_return_expr_with_float_r126() {
        let eval_expr =
            |_expr: &Expr| -> Result<Value, InterpreterError> { Ok(Value::Float(3.14)) };

        let value = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::Float(3.14)),
            Span::new(0, 4),
        );
        let result = eval_return_expr(Some(&value), eval_expr);
        assert!(result.is_err());
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::Float(3.14));
        }
    }

    #[test]
    fn test_eval_return_expr_with_string_r126() {
        let eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::String(Arc::from("hello")))
        };

        let value = Expr::new(
            crate::frontend::ast::ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(0, 7),
        );
        let result = eval_return_expr(Some(&value), eval_expr);
        assert!(result.is_err());
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::String(Arc::from("hello")));
        }
    }
