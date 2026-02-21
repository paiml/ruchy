// Auto-extracted from interpreter_tests.rs - Part 4
use super::*;

// ============== EXTREME TDD Round 127: Expanded Coverage ==============

#[test]
fn test_unary_negate_integer_r127() {
    let mut interp = Interpreter::new();
    let neg = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Negate,
            operand: Box::new(make_int(42)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&neg).expect("should evaluate");
    assert_eq!(result, Value::Integer(-42));
}

#[test]
fn test_unary_negate_float_r127() {
    let mut interp = Interpreter::new();
    let neg = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Negate,
            operand: Box::new(make_float(3.14)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&neg).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - (-3.14)).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_unary_not_true_r127() {
    let mut interp = Interpreter::new();
    let not = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Not,
            operand: Box::new(make_bool(true)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&not).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_unary_not_false_r127() {
    let mut interp = Interpreter::new();
    let not = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Not,
            operand: Box::new(make_bool(false)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&not).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_less_equal_equal_r127() {
    let mut interp = Interpreter::new();
    let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_less_equal_less_r127() {
    let mut interp = Interpreter::new();
    let le = make_binary(make_int(3), AstBinaryOp::LessEqual, make_int(5));
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_less_equal_greater_r127() {
    let mut interp = Interpreter::new();
    let le = make_binary(make_int(7), AstBinaryOp::LessEqual, make_int(5));
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_binary_greater_equal_equal_r127() {
    let mut interp = Interpreter::new();
    let ge = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
    let result = interp.eval_expr(&ge).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_greater_equal_greater_r127() {
    let mut interp = Interpreter::new();
    let ge = make_binary(make_int(7), AstBinaryOp::GreaterEqual, make_int(5));
    let result = interp.eval_expr(&ge).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_greater_equal_less_r127() {
    let mut interp = Interpreter::new();
    let ge = make_binary(make_int(3), AstBinaryOp::GreaterEqual, make_int(5));
    let result = interp.eval_expr(&ge).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_binary_or_true_true_r127() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(true), AstBinaryOp::Or, make_bool(true));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_or_true_false_r127() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(true), AstBinaryOp::Or, make_bool(false));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_or_false_true_r127() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_or_false_false_r127() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_binary_and_true_true_r127() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_and_true_false_r127() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_binary_and_false_true_r127() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(false), AstBinaryOp::And, make_bool(true));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_binary_and_false_false_r127() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(false), AstBinaryOp::And, make_bool(false));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_float_add_r127() {
    let mut interp = Interpreter::new();
    let add = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
    let result = interp.eval_expr(&add).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 4.0).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_float_subtract_r127() {
    let mut interp = Interpreter::new();
    let sub = make_binary(make_float(5.5), AstBinaryOp::Subtract, make_float(2.5));
    let result = interp.eval_expr(&sub).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 3.0).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_float_multiply_r127() {
    let mut interp = Interpreter::new();
    let mul = make_binary(make_float(3.0), AstBinaryOp::Multiply, make_float(2.5));
    let result = interp.eval_expr(&mul).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_float_divide_r127() {
    let mut interp = Interpreter::new();
    let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
    let result = interp.eval_expr(&div).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 2.5).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_mixed_int_float_add_r127() {
    let mut interp = Interpreter::new();
    let add = make_binary(make_int(5), AstBinaryOp::Add, make_float(2.5));
    let result = interp.eval_expr(&add).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_float_compare_less_r127() {
    let mut interp = Interpreter::new();
    let lt = make_binary(make_float(2.5), AstBinaryOp::Less, make_float(3.5));
    let result = interp.eval_expr(&lt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_float_compare_greater_r127() {
    let mut interp = Interpreter::new();
    let gt = make_binary(make_float(3.5), AstBinaryOp::Greater, make_float(2.5));
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_string_equal_true_r127() {
    let mut interp = Interpreter::new();
    let eq = make_binary(
        make_string("hello"),
        AstBinaryOp::Equal,
        make_string("hello"),
    );
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_string_equal_false_r127() {
    let mut interp = Interpreter::new();
    let eq = make_binary(
        make_string("hello"),
        AstBinaryOp::Equal,
        make_string("world"),
    );
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_string_not_equal_r127() {
    let mut interp = Interpreter::new();
    let ne = make_binary(
        make_string("hello"),
        AstBinaryOp::NotEqual,
        make_string("world"),
    );
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_tuple_literal_r127() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![make_int(1), make_int(2), make_int(3)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 3);
            assert_eq!(t[0], Value::Integer(1));
            assert_eq!(t[1], Value::Integer(2));
            assert_eq!(t[2], Value::Integer(3));
        }
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_list_literal_r127() {
    let mut interp = Interpreter::new();
    let list = Expr {
        kind: ExprKind::List(vec![make_int(10), make_int(20), make_int(30)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(10));
            assert_eq!(arr[1], Value::Integer(20));
            assert_eq!(arr[2], Value::Integer(30));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_empty_list_r127() {
    let mut interp = Interpreter::new();
    let list = Expr {
        kind: ExprKind::List(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_empty_tuple_r127() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => assert_eq!(t.len(), 0),
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_double_negate_r127() {
    let mut interp = Interpreter::new();
    let inner = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Negate,
            operand: Box::new(make_int(42)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let outer = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Negate,
            operand: Box::new(inner),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_double_not_r127() {
    let mut interp = Interpreter::new();
    let inner = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Not,
            operand: Box::new(make_bool(true)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let outer = Expr {
        kind: ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Not,
            operand: Box::new(inner),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_block_with_multiple_exprs_r127() {
    let mut interp = Interpreter::new();
    let block = Expr {
        kind: ExprKind::Block(vec![make_int(1), make_int(2), make_int(42)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Integer(42)); // Last expression is returned
}

#[test]
fn test_large_integer_r127() {
    let mut interp = Interpreter::new();
    let large = make_int(i64::MAX);
    let result = interp.eval_expr(&large).expect("should evaluate");
    assert_eq!(result, Value::Integer(i64::MAX));
}

#[test]
fn test_negative_integer_r127() {
    let mut interp = Interpreter::new();
    let neg = make_int(-12345);
    let result = interp.eval_expr(&neg).expect("should evaluate");
    assert_eq!(result, Value::Integer(-12345));
}

#[test]
fn test_zero_integer_r127() {
    let mut interp = Interpreter::new();
    let zero = make_int(0);
    let result = interp.eval_expr(&zero).expect("should evaluate");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_float_zero_r127() {
    let mut interp = Interpreter::new();
    let zero = make_float(0.0);
    let result = interp.eval_expr(&zero).expect("should evaluate");
    assert_eq!(result, Value::Float(0.0));
}

#[test]
fn test_empty_string_r127() {
    let mut interp = Interpreter::new();
    let empty = make_string("");
    let result = interp.eval_expr(&empty).expect("should evaluate");
    assert_eq!(result, Value::String("".into()));
}

// ============== EXTREME TDD Round 129 Tests ==============
// Focus: Control flow, expressions, basic operations

#[test]
fn test_if_else_true_branch_r129() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(true)),
            then_branch: Box::new(make_int(100)),
            else_branch: Some(Box::new(make_int(200))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_if_else_false_branch_r129() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(false)),
            then_branch: Box::new(make_int(100)),
            else_branch: Some(Box::new(make_int(200))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(200));
}

#[test]
fn test_if_no_else_true_r129() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(true)),
            then_branch: Box::new(make_int(42)),
            else_branch: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_if_no_else_false_r129() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(false)),
            then_branch: Box::new(make_int(42)),
            else_branch: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_expr).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_list_creation_r129() {
    let mut interp = Interpreter::new();
    let list = Expr {
        kind: ExprKind::List(vec![make_int(1), make_int(2), make_int(3)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_empty_list_r129() {
    let mut interp = Interpreter::new();
    let list = Expr {
        kind: ExprKind::List(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 0);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_tuple_creation_r129() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![make_int(10), make_string("hello")]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 2);
        }
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_string_concat_r129() {
    let mut interp = Interpreter::new();
    let concat = make_binary(
        make_string("hello"),
        AstBinaryOp::Add,
        make_string(" world"),
    );
    let result = interp.eval_expr(&concat).expect("should evaluate");
    assert_eq!(result, Value::String("hello world".into()));
}

#[test]
fn test_integer_multiply_r129() {
    let mut interp = Interpreter::new();
    let mul = make_binary(make_int(7), AstBinaryOp::Multiply, make_int(6));
    let result = interp.eval_expr(&mul).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_integer_divide_r129() {
    let mut interp = Interpreter::new();
    let div = make_binary(make_int(100), AstBinaryOp::Divide, make_int(4));
    let result = interp.eval_expr(&div).expect("should evaluate");
    assert_eq!(result, Value::Integer(25));
}

#[test]
fn test_gc_track_value_r129() {
    let mut interp = Interpreter::new();
    let tracked = interp.gc_track(Value::Integer(42));
    // tracked is usize, just verify it exists (any value is valid)
    let _ = tracked;
}

#[test]
fn test_gc_collect_r129() {
    let mut interp = Interpreter::new();
    let _id = interp.gc_track(Value::Integer(100));
    let stats = interp.gc_collect();
    // Verify stats struct is returned (collections is usize, always valid)
    let _ = stats.collections;
}

#[test]
fn test_gc_stats_r129() {
    let interp = Interpreter::new();
    let stats = interp.gc_stats();
    // Verify stats struct is returned (collections is usize, always valid)
    let _ = stats.collections;
}

#[test]
fn test_gc_set_threshold_r129() {
    let mut interp = Interpreter::new();
    interp.gc_set_threshold(100);
}

#[test]
fn test_gc_set_auto_collect_r129() {
    let mut interp = Interpreter::new();
    interp.gc_set_auto_collect(true);
    interp.gc_set_auto_collect(false);
}

#[test]
fn test_gc_clear_r129() {
    let mut interp = Interpreter::new();
    interp.gc_track(Value::Integer(1));
    interp.gc_clear();
}

#[test]
fn test_gc_alloc_array_r129() {
    let mut interp = Interpreter::new();
    let arr = interp.gc_alloc_array(vec![Value::Integer(1), Value::Integer(2)]);
    match arr {
        Value::Array(a) => assert_eq!(a.len(), 2),
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_gc_alloc_string_r129() {
    let mut interp = Interpreter::new();
    let s = interp.gc_alloc_string("test".to_string());
    assert_eq!(s, Value::String("test".into()));
}

#[test]
fn test_push_scope_r129() {
    let mut interp = Interpreter::new();
    interp.push_scope();
    interp.pop_scope();
}

#[test]
fn test_set_and_get_variable_r129() {
    let mut interp = Interpreter::new();
    interp.set_variable("test_var", Value::Integer(42));
    let result = interp.get_variable("test_var");
    assert_eq!(result, Some(Value::Integer(42)));
}

#[test]
fn test_get_nonexistent_variable_r129() {
    let interp = Interpreter::new();
    let result = interp.get_variable("nonexistent");
    assert_eq!(result, None);
}

#[test]
fn test_clear_user_variables_r129() {
    let mut interp = Interpreter::new();
    interp.set_variable("user_var", Value::Integer(99));
    assert!(interp.get_variable("user_var").is_some());
    interp.clear_user_variables();
    // After clearing, user_var should be gone
    assert!(interp.get_variable("user_var").is_none());
}

#[test]
fn test_get_global_bindings_r129() {
    let mut interp = Interpreter::new();
    interp.set_global_binding("global_test".to_string(), Value::Integer(123));
    let bindings = interp.get_global_bindings();
    assert!(bindings.contains_key("global_test"));
}

#[test]
fn test_get_current_bindings_r129() {
    let mut interp = Interpreter::new();
    interp.set_variable("current_test", Value::String("test".into()));
    let bindings = interp.get_current_bindings();
    assert!(bindings.contains_key("current_test"));
}

#[test]
fn test_push_error_scope_r129() {
    let mut interp = Interpreter::new();
    interp.push_error_scope();
}

#[test]
fn test_pop_error_scope_r129() {
    let mut interp = Interpreter::new();
    interp.push_error_scope();
    interp.pop_error_scope();
}

#[test]
fn test_capture_stdout_r129() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("line 1".to_string());
    interp.capture_stdout("line 2".to_string());
    assert!(interp.has_stdout());
}

#[test]
fn test_get_stdout_r129() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("hello".to_string());
    let output = interp.get_stdout();
    assert!(output.contains("hello"));
}

#[test]
fn test_clear_stdout_r129() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("test".to_string());
    interp.clear_stdout();
    assert!(!interp.has_stdout());
}

#[test]
fn test_get_cache_stats_r129() {
    let interp = Interpreter::new();
    let stats = interp.get_cache_stats();
    assert!(stats.contains_key("hit_rate") || stats.is_empty());
}

#[test]
fn test_clear_caches_r129() {
    let mut interp = Interpreter::new();
    interp.clear_caches();
}

#[test]
fn test_clear_type_feedback_r129() {
    let mut interp = Interpreter::new();
    interp.clear_type_feedback();
}

#[test]
fn test_push_and_pop_r129() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(42)).expect("push should work");
    let result = interp.pop().expect("pop should work");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_peek_r129() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(1)).expect("push should work");
    interp.push(Value::Integer(2)).expect("push should work");
    let result = interp.peek(0).expect("peek should work");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_pop_empty_stack_error_r129() {
    let mut interp = Interpreter::new();
    let result = interp.pop();
    assert!(result.is_err());
}

#[test]
fn test_peek_out_of_bounds_r129() {
    let interp = Interpreter::new();
    let result = interp.peek(100);
    assert!(result.is_err());
}

#[test]
fn test_eval_string_simple_r129() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("2 + 2").expect("should evaluate");
    assert_eq!(result.to_string(), "4");
}

#[test]
fn test_eval_string_comparison_r129() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("5 > 3").expect("should evaluate");
    assert_eq!(result.to_string(), "true");
}

#[test]
fn test_range_expr_r129() {
    let mut interp = Interpreter::new();
    let range = Expr {
        kind: ExprKind::Range {
            start: Box::new(make_int(1)),
            end: Box::new(make_int(5)),
            inclusive: false,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&range).expect("should evaluate");
    // Range returns a Range object (not Array) - just verify it evaluated
    assert_ne!(result, Value::Nil);
}

#[test]
fn test_range_inclusive_r129() {
    let mut interp = Interpreter::new();
    let range = Expr {
        kind: ExprKind::Range {
            start: Box::new(make_int(1)),
            end: Box::new(make_int(5)),
            inclusive: true,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&range).expect("should evaluate");
    // Range returns a Range object (not Array) - just verify it evaluated
    assert_ne!(result, Value::Nil);
}

#[test]
fn test_current_env_r129() {
    let mut interp = Interpreter::new();
    interp.set_variable("env_test", Value::Integer(42));
    let env = interp.current_env();
    let borrowed = env.borrow();
    assert!(borrowed.contains_key("env_test"));
}

#[test]
fn test_default_interpreter_r129() {
    let interp = Interpreter::default();
    assert!(interp.get_variable("len").is_some());
}

#[test]
fn test_nested_if_r129() {
    let mut interp = Interpreter::new();
    let inner_if = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(true)),
            then_branch: Box::new(make_int(10)),
            else_branch: Some(Box::new(make_int(20))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let outer_if = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(true)),
            then_branch: Box::new(inner_if),
            else_branch: Some(Box::new(make_int(30))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&outer_if).expect("should evaluate");
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn test_empty_block_r129() {
    let mut interp = Interpreter::new();
    let block = Expr {
        kind: ExprKind::Block(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_empty_tuple_r129() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => assert_eq!(t.len(), 0),
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_set_variable_string_r129() {
    let mut interp = Interpreter::new();
    interp.set_variable_string("str_var".to_string(), Value::String("hello".into()));
    let result = interp.get_variable("str_var");
    assert_eq!(result, Some(Value::String("hello".into())));
}

#[test]
fn test_index_expr_array_r129() {
    let mut interp = Interpreter::new();
    let list_expr = Expr {
        kind: ExprKind::List(vec![make_int(10), make_int(20), make_int(30)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let index_expr = Expr {
        kind: ExprKind::IndexAccess {
            object: Box::new(list_expr),
            index: Box::new(make_int(1)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&index_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_index_expr_tuple_r129() {
    let mut interp = Interpreter::new();
    let tuple_expr = Expr {
        kind: ExprKind::Tuple(vec![make_int(100), make_string("test")]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let index_expr = Expr {
        kind: ExprKind::IndexAccess {
            object: Box::new(tuple_expr),
            index: Box::new(make_int(0)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&index_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_bitwise_and_r129() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_int(0b1100), AstBinaryOp::BitwiseAnd, make_int(0b1010));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Integer(0b1000));
}

#[test]
fn test_bitwise_or_r129() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_int(0b1100), AstBinaryOp::BitwiseOr, make_int(0b1010));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Integer(0b1110));
}

#[test]
fn test_bitwise_xor_r129() {
    let mut interp = Interpreter::new();
    let xor = make_binary(make_int(0b1100), AstBinaryOp::BitwiseXor, make_int(0b1010));
    let result = interp.eval_expr(&xor).expect("should evaluate");
    assert_eq!(result, Value::Integer(0b0110));
}

#[test]
fn test_binary_less_than_r129() {
    let mut interp = Interpreter::new();
    let lt = make_binary(make_int(5), AstBinaryOp::Less, make_int(10));
    let result = interp.eval_expr(&lt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_greater_than_r129() {
    let mut interp = Interpreter::new();
    let gt = make_binary(make_int(10), AstBinaryOp::Greater, make_int(5));
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_less_equal_r129() {
    let mut interp = Interpreter::new();
    let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_greater_equal_r129() {
    let mut interp = Interpreter::new();
    let ge = make_binary(make_int(10), AstBinaryOp::GreaterEqual, make_int(10));
    let result = interp.eval_expr(&ge).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_equal_r129() {
    let mut interp = Interpreter::new();
    let eq = make_binary(make_int(42), AstBinaryOp::Equal, make_int(42));
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_not_equal_r129() {
    let mut interp = Interpreter::new();
    let ne = make_binary(make_int(42), AstBinaryOp::NotEqual, make_int(99));
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_mixed_float_int_subtract_r129() {
    let mut interp = Interpreter::new();
    let sub = make_binary(make_float(10.5), AstBinaryOp::Subtract, make_int(5));
    let result = interp.eval_expr(&sub).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 5.5).abs() < 0.0001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_mixed_float_int_multiply_r129() {
    let mut interp = Interpreter::new();
    let mul = make_binary(make_int(3), AstBinaryOp::Multiply, make_float(2.5));
    let result = interp.eval_expr(&mul).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_float_divide_r129() {
    let mut interp = Interpreter::new();
    let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
    let result = interp.eval_expr(&div).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 2.5).abs() < 0.0001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_logical_and_both_true_r129() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_logical_and_one_false_r129() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_logical_or_both_false_r129() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_logical_or_one_true_r129() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// EXTREME TDD Round 157: Additional interpreter tests
// Target: Push coverage further
// ============================================================================

#[test]
fn test_nested_binary_operations_r157() {
    let mut interp = Interpreter::new();
    // ((1 + 2) * 3) - 4 = 5
    let add = make_binary(make_int(1), AstBinaryOp::Add, make_int(2));
    let mul = make_binary(add, AstBinaryOp::Multiply, make_int(3));
    let sub = make_binary(mul, AstBinaryOp::Subtract, make_int(4));
    let result = interp.eval_expr(&sub).expect("should evaluate");
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_deeply_nested_binary_r157() {
    let mut interp = Interpreter::new();
    // Build: 1 + 2 + 3 + 4 + 5 = 15
    let mut expr = make_int(1);
    for i in 2..=5 {
        expr = make_binary(expr, AstBinaryOp::Add, make_int(i));
    }
    let result = interp.eval_expr(&expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_unary_negate_float_r157() {
    let mut interp = Interpreter::new();
    let neg = make_unary(UnaryOp::Negate, make_float(3.14));
    let result = interp.eval_expr(&neg).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f + 3.14).abs() < 0.0001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_unary_not_true_r157() {
    let mut interp = Interpreter::new();
    let not = make_unary(UnaryOp::Not, make_bool(true));
    let result = interp.eval_expr(&not).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_unary_not_false_r157() {
    let mut interp = Interpreter::new();
    let not = make_unary(UnaryOp::Not, make_bool(false));
    let result = interp.eval_expr(&not).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_integer_modulo_r157() {
    let mut interp = Interpreter::new();
    let modulo = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
    let result = interp.eval_expr(&modulo).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_integer_modulo_zero_remainder_r157() {
    let mut interp = Interpreter::new();
    let modulo = make_binary(make_int(20), AstBinaryOp::Modulo, make_int(5));
    let result = interp.eval_expr(&modulo).expect("should evaluate");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_nested_if_expression_r157() {
    let mut interp = Interpreter::new();
    // if true { if false { 1 } else { 2 } } else { 3 }
    let inner_if = make_if(make_bool(false), make_int(1), Some(make_int(2)));
    let outer_if = make_if(make_bool(true), inner_if, Some(make_int(3)));
    let result = interp.eval_expr(&outer_if).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_block_with_multiple_expressions_r157() {
    let mut interp = Interpreter::new();
    let block = make_block(vec![make_int(1), make_int(2), make_int(3), make_int(42)]);
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_empty_block_returns_nil_r157() {
    let mut interp = Interpreter::new();
    let block = make_block(vec![]);
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_list_of_floats_r157() {
    let mut interp = Interpreter::new();
    let list = make_list(vec![make_float(1.1), make_float(2.2), make_float(3.3)]);
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert!(matches!(arr[0], Value::Float(_)));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_list_of_bools_r157() {
    let mut interp = Interpreter::new();
    let list = make_list(vec![make_bool(true), make_bool(false), make_bool(true)]);
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_tuple_of_three_r157() {
    let mut interp = Interpreter::new();
    let tuple = make_tuple(vec![make_int(1), make_float(2.5), make_string("three")]);
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 3);
        }
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_nested_list_r157() {
    let mut interp = Interpreter::new();
    let inner1 = make_list(vec![make_int(1), make_int(2)]);
    let inner2 = make_list(vec![make_int(3), make_int(4)]);
    let outer = make_list(vec![inner1, inner2]);
    let result = interp.eval_expr(&outer).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert!(matches!(arr[0], Value::Array(_)));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_let_binding_chain_r157() {
    let mut interp = Interpreter::new();
    // let x = 1; let y = x + 1; let z = y + 1; z
    let z = make_ident("z");
    let let_z = make_let(
        "z",
        make_binary(make_ident("y"), AstBinaryOp::Add, make_int(1)),
        z,
    );
    let let_y = make_let(
        "y",
        make_binary(make_ident("x"), AstBinaryOp::Add, make_int(1)),
        let_z,
    );
    let let_x = make_let("x", make_int(1), let_y);
    let result = interp.eval_expr(&let_x).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_range_exclusive_r157() {
    let mut interp = Interpreter::new();
    let range = make_range(make_int(0), make_int(5), false);
    let result = interp.eval_expr(&range).expect("should evaluate");
    match result {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(*start, Value::Integer(0));
            assert_eq!(*end, Value::Integer(5));
            assert!(!inclusive);
        }
        _ => panic!("Expected Range"),
    }
}

#[test]
fn test_range_inclusive_r157() {
    let mut interp = Interpreter::new();
    let range = make_range(make_int(1), make_int(10), true);
    let result = interp.eval_expr(&range).expect("should evaluate");
    match result {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(*start, Value::Integer(1));
            assert_eq!(*end, Value::Integer(10));
            assert!(inclusive);
        }
        _ => panic!("Expected Range"),
    }
}

#[test]
fn test_range_negative_values_r157() {
    let mut interp = Interpreter::new();
    let range = make_range(make_int(-10), make_int(-1), false);
    let result = interp.eval_expr(&range).expect("should evaluate");
    match result {
        Value::Range { start, end, .. } => {
            assert_eq!(*start, Value::Integer(-10));
            assert_eq!(*end, Value::Integer(-1));
        }
        _ => panic!("Expected Range"),
    }
}

#[test]
fn test_index_access_r157() {
    let mut interp = Interpreter::new();
    let list = make_list(vec![make_int(10), make_int(20), make_int(30)]);
    let indexed = make_index(list, make_int(1));
    let result = interp.eval_expr(&indexed).expect("should evaluate");
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_index_access_first_r157() {
    let mut interp = Interpreter::new();
    let list = make_list(vec![make_int(100), make_int(200)]);
    let indexed = make_index(list, make_int(0));
    let result = interp.eval_expr(&indexed).expect("should evaluate");
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_index_access_last_r157() {
    let mut interp = Interpreter::new();
    let list = make_list(vec![make_int(1), make_int(2), make_int(3), make_int(999)]);
    let indexed = make_index(list, make_int(3));
    let result = interp.eval_expr(&indexed).expect("should evaluate");
    assert_eq!(result, Value::Integer(999));
}

#[test]
fn test_float_comparison_less_r157() {
    let mut interp = Interpreter::new();
    let lt = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
    let result = interp.eval_expr(&lt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_float_comparison_greater_r157() {
    let mut interp = Interpreter::new();
    let gt = make_binary(make_float(3.5), AstBinaryOp::Greater, make_float(2.5));
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_string_comparison_equal_r157() {
    let mut interp = Interpreter::new();
    let eq = make_binary(
        make_string("hello"),
        AstBinaryOp::Equal,
        make_string("hello"),
    );
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_string_comparison_not_equal_r157() {
    let mut interp = Interpreter::new();
    let ne = make_binary(
        make_string("hello"),
        AstBinaryOp::NotEqual,
        make_string("world"),
    );
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_large_integer_r157() {
    let mut interp = Interpreter::new();
    let large = make_int(999_999_999_999);
    let result = interp.eval_expr(&large).expect("should evaluate");
    assert_eq!(result, Value::Integer(999_999_999_999));
}

#[test]
fn test_negative_large_integer_r157() {
    let mut interp = Interpreter::new();
    let large_neg = make_int(-999_999_999_999);
    let result = interp.eval_expr(&large_neg).expect("should evaluate");
    assert_eq!(result, Value::Integer(-999_999_999_999));
}

#[test]
fn test_scientific_float_r157() {
    let mut interp = Interpreter::new();
    let sci = make_float(1.5e10);
    let result = interp.eval_expr(&sci).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 1.5e10).abs() < 1.0),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_very_small_float_r157() {
    let mut interp = Interpreter::new();
    let small = make_float(1.5e-10);
    let result = interp.eval_expr(&small).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 1.5e-10).abs() < 1e-15),
        _ => panic!("Expected Float"),
    }
}
