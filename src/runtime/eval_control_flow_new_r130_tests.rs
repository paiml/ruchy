    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, MatchArm, Span};

    // EXTREME TDD Round 130: eval_control_flow_new.rs coverage boost
    // Target: 75.42% -> 90%+

    // Helper functions for creating test expressions
    fn make_lit_int(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Span::new(0, 0),
        )
    }

    fn make_lit_bool(val: bool) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Bool(val)), Span::new(0, 0))
    }

    fn make_unit_expr() -> Expr {
        Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))
    }

    // ==================== eval_for_loop tests ====================

    #[test]
    fn test_eval_for_loop_non_iterable_r130() {
        let iter_expr = make_lit_int(42); // Can't iterate over integer

        let result = eval_for_loop(
            "x",
            &iter_expr,
            &make_unit_expr(),
            |_| Ok(Value::Integer(42)),
            |_, _, _| Ok(Value::Nil),
        );

        assert!(result.is_err());
        if let Err(InterpreterError::TypeError(msg)) = result {
            assert!(msg.contains("Cannot iterate over"));
        }
    }

    #[test]
    fn test_eval_for_loop_string_not_iterable_r130() {
        let result = eval_for_loop(
            "x",
            &make_unit_expr(),
            &make_unit_expr(),
            |_| Ok(Value::String(Arc::from("hello"))),
            |_, _, _| Ok(Value::Nil),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_for_loop_bool_not_iterable_r130() {
        let result = eval_for_loop(
            "x",
            &make_unit_expr(),
            &make_unit_expr(),
            |_| Ok(Value::Bool(true)),
            |_, _, _| Ok(Value::Nil),
        );

        assert!(result.is_err());
    }

    // ==================== eval_loop_body tests ====================

    #[test]
    fn test_eval_loop_body_break_with_label_r130() {
        let body = make_lit_int(99);
        let mut last_val = Value::Nil;

        // Simulate a break with value
        let result = eval_loop_body(&body, &mut last_val, &mut |_| {
            Err(InterpreterError::Break(None, Value::Integer(42)))
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Integer(42)));
    }

    #[test]
    fn test_eval_loop_body_continue_r130() {
        let body = make_lit_int(99);
        let mut last_val = Value::Nil;

        let result = eval_loop_body(&body, &mut last_val, &mut |_| {
            Err(InterpreterError::Continue(None))
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_eval_loop_body_runtime_error_r130() {
        let body = make_unit_expr();
        let mut last_val = Value::Nil;

        let result = eval_loop_body(&body, &mut last_val, &mut |_| {
            Err(InterpreterError::RuntimeError("test error".to_string()))
        });

        assert!(result.is_err());
    }

    // ==================== run_while_loop tests ====================

    #[test]
    fn test_run_while_loop_immediately_false_r130() {
        let condition = make_lit_bool(false);
        let body = make_lit_int(42);

        let result = run_while_loop(&condition, &body, &mut |expr: &Expr| {
            if let ExprKind::Literal(Literal::Bool(b)) = &expr.kind {
                Ok(Value::Bool(*b))
            } else {
                Ok(Value::Nil)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil); // Never executed body
    }

    // ==================== eval_match_guard tests ====================

    #[test]
    fn test_eval_match_guard_true_r130() {
        let guard = make_lit_bool(true);

        let result = eval_match_guard(Some(&guard), &mut |_| Ok(Value::Bool(true)));

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_eval_match_guard_false_r130() {
        let guard = make_lit_bool(false);

        let result = eval_match_guard(Some(&guard), &mut |_| Ok(Value::Bool(false)));

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_eval_match_guard_none_r130() {
        let result = eval_match_guard(None, &mut |_| Ok(Value::Bool(true)));

        assert!(result.is_ok());
        assert!(result.unwrap()); // No guard means it passes
    }

    // ==================== eval_match_arm tests ====================

    #[test]
    fn test_eval_match_arm_non_matching_r130() {
        let arm = MatchArm {
            pattern: Pattern::Literal(Literal::Integer(1, None)),
            guard: None,
            body: Box::new(make_lit_int(100)),
            span: Span::new(0, 0),
        };

        let result = eval_match_arm(
            &arm,
            &Value::Integer(2), // Different value
            &mut |_pat, _val| Ok(false),
            &mut |_| Ok(Value::Integer(100)),
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No match
    }

    #[test]
    fn test_eval_match_arm_matching_r130() {
        let arm = MatchArm {
            pattern: Pattern::Literal(Literal::Integer(42, None)),
            guard: None,
            body: Box::new(make_lit_int(100)),
            span: Span::new(0, 0),
        };

        let result = eval_match_arm(
            &arm,
            &Value::Integer(42),
            &mut |_pat, _val| Ok(true),
            &mut |_| Ok(Value::Integer(100)),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Integer(100)));
    }

    // ==================== find_matching_arm tests ====================

    #[test]
    fn test_find_matching_arm_no_match_r130() {
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1, None)),
                guard: None,
                body: Box::new(make_lit_int(100)),
                span: Span::new(0, 0),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(2, None)),
                guard: None,
                body: Box::new(make_lit_int(200)),
                span: Span::new(0, 0),
            },
        ];

        let result = find_matching_arm(
            &arms,
            &Value::Integer(3), // No matching arm
            &mut |_pat, _val| Ok(false),
            &mut |_| Ok(Value::Nil),
        );

        // find_matching_arm returns an error (NonExhaustiveMatch) when no arm matches
        assert!(result.is_err());
    }

    #[test]
    fn test_find_matching_arm_first_match_r130() {
        let arms = vec![
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(make_lit_int(100)),
                span: Span::new(0, 0),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(make_lit_int(200)),
                span: Span::new(0, 0),
            },
        ];

        let result = find_matching_arm(
            &arms,
            &Value::Integer(42),
            &mut |_pat, _val| Ok(true), // All match
            &mut |_| Ok(Value::Integer(100)),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(100)); // First match wins
    }

    // ==================== eval_block_expr tests ====================

    #[test]
    fn test_eval_block_expr_many_statements_r130() {
        let stmts = vec![
            make_lit_int(1),
            make_lit_int(2),
            make_lit_int(3),
            make_lit_int(4),
            make_lit_int(5),
        ];

        let result = eval_block_expr(&stmts, |expr| {
            if let ExprKind::Literal(Literal::Integer(n, _)) = &expr.kind {
                Ok(Value::Integer(*n))
            } else {
                Ok(Value::Nil)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(5)); // Last value
    }

    #[test]
    fn test_eval_block_expr_early_error_r130() {
        let stmts = vec![make_lit_int(1), make_lit_int(2), make_lit_int(3)];

        let mut call_count = 0;
        let result = eval_block_expr(&stmts, |_expr| {
            call_count += 1;
            if call_count == 2 {
                Err(InterpreterError::RuntimeError("test error".to_string()))
            } else {
                Ok(Value::Nil)
            }
        });

        assert!(result.is_err());
    }

    // ==================== eval_list_expr tests ====================

    #[test]
    fn test_eval_list_expr_mixed_types_r130() {
        let elements = vec![make_lit_int(1), make_lit_bool(true)];

        let mut i = 0;
        let result = eval_list_expr(&elements, |_expr| {
            i += 1;
            if i == 1 {
                Ok(Value::Integer(1))
            } else {
                Ok(Value::Bool(true))
            }
        });

        assert!(result.is_ok());
        if let Value::Array(arr) = result.unwrap() {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Bool(true));
        } else {
            panic!("Expected Array");
        }
    }

    // ==================== eval_tuple_expr tests ====================

    #[test]
    fn test_eval_tuple_expr_nested_r130() {
        let elements = vec![make_lit_int(1), make_lit_int(2)];

        let result = eval_tuple_expr(&elements, |expr| {
            if let ExprKind::Literal(Literal::Integer(n, _)) = &expr.kind {
                Ok(Value::Integer(*n))
            } else {
                Ok(Value::Nil)
            }
        });

        assert!(result.is_ok());
        if let Value::Tuple(t) = result.unwrap() {
            assert_eq!(t.len(), 2);
        } else {
            panic!("Expected Tuple");
        }
    }

    // ==================== value_to_integer tests ====================

    #[test]
    fn test_value_to_integer_valid_r130() {
        let result = value_to_integer(&Value::Integer(42), "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_value_to_integer_invalid_r130() {
        let result = value_to_integer(&Value::String(Arc::from("hello")), "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_value_to_integer_float_r130() {
        let result = value_to_integer(&Value::Float(3.14), "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_value_to_integer_nil_r130() {
        let result = value_to_integer(&Value::Nil, "test");
        assert!(result.is_err());
    }

    // ==================== extract_range_bounds tests ====================

    #[test]
    fn test_extract_range_bounds_non_integer_start_r130() {
        let range = Value::Range {
            start: Box::new(Value::String(Arc::from("a"))),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };

        let result = extract_range_bounds(&range);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_range_bounds_non_integer_end_r130() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Float(10.5)),
            inclusive: false,
        };

        let result = extract_range_bounds(&range);
        assert!(result.is_err());
    }

    // ==================== handle_loop_control tests ====================

    #[test]
    fn test_handle_loop_control_ok_value_r130() {
        let mut last_val = Value::Nil;

        let result = handle_loop_control(Ok(Value::Integer(42)), &mut last_val);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No break - returns None
        assert_eq!(last_val, Value::Integer(42));
    }

    #[test]
    fn test_handle_loop_control_break_r130() {
        let mut last_val = Value::Nil;

        let result = handle_loop_control(
            Err(InterpreterError::Break(None, Value::Integer(100))),
            &mut last_val,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_some()); // Break occurred - returns Some
    }

    #[test]
    fn test_handle_loop_control_continue_r130() {
        let mut last_val = Value::Integer(50);

        let result = handle_loop_control(Err(InterpreterError::Continue(None)), &mut last_val);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Continue - returns None
        assert_eq!(last_val, Value::Integer(50)); // Unchanged
    }

    #[test]
    fn test_handle_loop_control_return_r130() {
        let mut last_val = Value::Nil;

        let result = handle_loop_control(
            Err(InterpreterError::Return(Value::Integer(999))),
            &mut last_val,
        );

        assert!(result.is_err());
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::Integer(999));
        }
    }

    // ==================== create_range_iterator tests ====================

    #[test]
    fn test_create_range_iterator_large_range_r130() {
        let iter: Vec<i64> = create_range_iterator(0, 100, false).collect();
        assert_eq!(iter.len(), 100);
        assert_eq!(iter[0], 0);
        assert_eq!(iter[99], 99);
    }

    #[test]
    fn test_create_range_iterator_single_inclusive_r130() {
        let iter: Vec<i64> = create_range_iterator(5, 5, true).collect();
        assert_eq!(iter, vec![5]);
    }

    #[test]
    fn test_create_range_iterator_single_exclusive_r130() {
        let iter: Vec<i64> = create_range_iterator(5, 5, false).collect();
        assert!(iter.is_empty());
    }

    #[test]
    fn test_create_range_iterator_negative_r130() {
        let iter: Vec<i64> = create_range_iterator(-5, 0, false).collect();
        assert_eq!(iter, vec![-5, -4, -3, -2, -1]);
    }

    // ==================== pattern_matches_simple tests ====================

    #[test]
    fn test_pattern_matches_simple_wildcard_r130() {
        let result = pattern_matches_simple(&Pattern::Wildcard, &Value::Integer(42));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_pattern_matches_simple_identifier_r130() {
        let result =
            pattern_matches_simple(&Pattern::Identifier("x".to_string()), &Value::Integer(42));
        assert!(result.is_ok());
        assert!(result.unwrap()); // Identifier always matches
    }

    #[test]
    fn test_pattern_matches_simple_literal_match_r130() {
        let result = pattern_matches_simple(
            &Pattern::Literal(Literal::Integer(42, None)),
            &Value::Integer(42),
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_pattern_matches_simple_literal_no_match_r130() {
        let result = pattern_matches_simple(
            &Pattern::Literal(Literal::Integer(42, None)),
            &Value::Integer(99),
        );
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_pattern_matches_simple_tuple_r130() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = Value::Tuple(Arc::new([Value::Integer(1), Value::Integer(2)]));

        let result = pattern_matches_simple(&pattern, &value);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_pattern_matches_simple_list_r130() {
        let pattern = Pattern::List(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = Value::from_array(vec![Value::Integer(1), Value::Integer(2)]);

        let result = pattern_matches_simple(&pattern, &value);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // ==================== match_literal_pattern tests ====================

    #[test]
    fn test_match_literal_pattern_float_match_r130() {
        let result = match_literal_pattern(&Literal::Float(3.14), &Value::Float(3.14));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_float_no_match_r130() {
        let result = match_literal_pattern(&Literal::Float(3.14), &Value::Float(2.71));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_unit_r130() {
        let result = match_literal_pattern(&Literal::Unit, &Value::Nil);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_type_mismatch_r130() {
        // Integer literal against String value
        let result =
            match_literal_pattern(&Literal::Integer(42, None), &Value::String(Arc::from("42")));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // === EXTREME TDD Round 161 - Control Flow Coverage Push ===

    #[test]
    fn test_match_literal_pattern_float_r161() {
        let result = match_literal_pattern(&Literal::Float(3.14), &Value::Float(3.14));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_float_mismatch_r161() {
        let result = match_literal_pattern(&Literal::Float(3.14), &Value::Float(2.71));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_bool_true_r161() {
        let result = match_literal_pattern(&Literal::Bool(true), &Value::Bool(true));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_bool_false_r161() {
        let result = match_literal_pattern(&Literal::Bool(false), &Value::Bool(false));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_bool_mismatch_r161() {
        let result = match_literal_pattern(&Literal::Bool(true), &Value::Bool(false));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_string_r161() {
        let result = match_literal_pattern(
            &Literal::String("hello".to_string()),
            &Value::from_string("hello".to_string()),
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_string_mismatch_r161() {
        let result = match_literal_pattern(
            &Literal::String("hello".to_string()),
            &Value::from_string("world".to_string()),
        );
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_char_r161() {
        let result =
            match_literal_pattern(&Literal::Char('a'), &Value::from_string("a".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_char_mismatch_r161() {
        let result =
            match_literal_pattern(&Literal::Char('a'), &Value::from_string("b".to_string()));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_byte_r161() {
        let result = match_literal_pattern(&Literal::Byte(255), &Value::Byte(255));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_byte_mismatch_r161() {
        let result = match_literal_pattern(&Literal::Byte(255), &Value::Byte(0));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_nil_r161() {
        let result = match_literal_pattern(&Literal::Null, &Value::Nil);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_nil_vs_int_r161() {
        let result = match_literal_pattern(&Literal::Null, &Value::Integer(0));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_negative_int_r161() {
        let result = match_literal_pattern(&Literal::Integer(-42, None), &Value::Integer(-42));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_large_int_r161() {
        let result =
            match_literal_pattern(&Literal::Integer(i64::MAX, None), &Value::Integer(i64::MAX));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_int_vs_float_r161() {
        // Integer literal vs Float value - should not match
        let result = match_literal_pattern(&Literal::Integer(42, None), &Value::Float(42.0));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_float_vs_int_r161() {
        // Float literal vs Integer value - should not match
        let result = match_literal_pattern(&Literal::Float(42.0), &Value::Integer(42));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_empty_string_r161() {
        let result = match_literal_pattern(
            &Literal::String("".to_string()),
            &Value::from_string("".to_string()),
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_unicode_string_r161() {
        let result = match_literal_pattern(
            &Literal::String("日本語".to_string()),
            &Value::from_string("日本語".to_string()),
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_string_vs_nil_r161() {
        let result = match_literal_pattern(&Literal::String("test".to_string()), &Value::Nil);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_bool_vs_int_r161() {
        let result = match_literal_pattern(&Literal::Bool(true), &Value::Integer(1));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_int_zero_r161() {
        let result = match_literal_pattern(&Literal::Integer(0, None), &Value::Integer(0));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_float_zero_r161() {
        let result = match_literal_pattern(&Literal::Float(0.0), &Value::Float(0.0));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_float_negative_r161() {
        let result = match_literal_pattern(&Literal::Float(-3.14), &Value::Float(-3.14));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_float_infinity_r161() {
        let result =
            match_literal_pattern(&Literal::Float(f64::INFINITY), &Value::Float(f64::INFINITY));
        assert!(result.is_ok());
        // Note: Infinity == Infinity should be true
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_byte_zero_r161() {
        let result = match_literal_pattern(&Literal::Byte(0), &Value::Byte(0));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_char_newline_r161() {
        let result =
            match_literal_pattern(&Literal::Char('\n'), &Value::from_string("\n".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_match_literal_pattern_char_unicode_r161() {
        let result =
            match_literal_pattern(&Literal::Char('日'), &Value::from_string("日".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
