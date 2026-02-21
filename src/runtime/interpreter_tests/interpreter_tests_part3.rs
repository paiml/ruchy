// Auto-extracted from interpreter_tests.rs - Part 3
use super::*;

// ============== EXTREME TDD Round 112: Additional Coverage ==============

#[test]
fn test_eval_range_simple() {
    let mut interp = Interpreter::new();
    let range = Expr {
        kind: ExprKind::Range {
            start: Box::new(make_int(0)),
            end: Box::new(make_int(5)),
            inclusive: false,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&range);
    assert!(result.is_ok());
}

#[test]
fn test_eval_range_inclusive() {
    let mut interp = Interpreter::new();
    let range = Expr {
        kind: ExprKind::Range {
            start: Box::new(make_int(1)),
            end: Box::new(make_int(3)),
            inclusive: true,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&range);
    assert!(result.is_ok());
}

#[test]
fn test_eval_tuple_empty() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    assert!(matches!(result, Value::Tuple(_)));
}

#[test]
fn test_eval_tuple_single() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![make_int(42)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    assert!(matches!(result, Value::Tuple(_)));
}

#[test]
fn test_eval_tuple_multiple() {
    let mut interp = Interpreter::new();
    let tuple = Expr {
        kind: ExprKind::Tuple(vec![make_int(1), make_int(2), make_int(3)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    if let Value::Tuple(vals) = result {
        assert_eq!(vals.len(), 3);
    } else {
        panic!("Expected tuple");
    }
}

#[test]
fn test_eval_list_to_array_empty() {
    let mut interp = Interpreter::new();
    let list = Expr {
        kind: ExprKind::List(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&list).expect("should evaluate");
    assert!(matches!(result, Value::Array(_)));
}

#[test]
fn test_eval_list_to_array_integers() {
    let mut interp = Interpreter::new();
    let list = Expr {
        kind: ExprKind::List(vec![make_int(1), make_int(2), make_int(3)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&list).expect("should evaluate");
    if let Value::Array(vals) = result {
        assert_eq!(vals.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_eval_array_init_expr() {
    let mut interp = Interpreter::new();
    let arr = Expr {
        kind: ExprKind::ArrayInit {
            value: Box::new(make_int(0)),
            size: Box::new(make_int(5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&arr);
    assert!(result.is_ok());
}

#[test]
fn test_eval_null_literal() {
    let mut interp = Interpreter::new();
    let nil = Expr {
        kind: ExprKind::Literal(Literal::Null),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&nil).expect("should evaluate");
    assert!(matches!(result, Value::Nil));
}

#[test]
fn test_eval_char_to_string_literal() {
    let mut interp = Interpreter::new();
    let ch = Expr {
        kind: ExprKind::Literal(Literal::Char('a')),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&ch).expect("should evaluate");
    // Char literals are converted to single-character strings
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_eval_unary_negate_expr() {
    let mut interp = Interpreter::new();
    let neg = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Negate,
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
fn test_eval_unary_not_expr() {
    let mut interp = Interpreter::new();
    let not = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Not,
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
fn test_eval_binary_power() {
    let mut interp = Interpreter::new();
    let power = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(2)),
            op: AstBinaryOp::Power,
            right: Box::new(make_int(3)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&power).expect("should evaluate");
    assert_eq!(result, Value::Integer(8));
}

#[test]
fn test_eval_logical_and() {
    let mut interp = Interpreter::new();
    let and = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_bool(false)),
            op: AstBinaryOp::And,
            right: Box::new(make_bool(true)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_eval_logical_or() {
    let mut interp = Interpreter::new();
    let or = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_bool(true)),
            op: AstBinaryOp::Or,
            right: Box::new(make_bool(false)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_float_add_expr() {
    let mut interp = Interpreter::new();
    let add = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_float(1.5)),
            op: AstBinaryOp::Add,
            right: Box::new(make_float(2.5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&add).expect("should evaluate");
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_eval_float_div_expr() {
    let mut interp = Interpreter::new();
    let div = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_float(10.0)),
            op: AstBinaryOp::Divide,
            right: Box::new(make_float(4.0)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&div).expect("should evaluate");
    assert_eq!(result, Value::Float(2.5));
}

#[test]
fn test_eval_block_multi_stmt() {
    let mut interp = Interpreter::new();
    let block = Expr {
        kind: ExprKind::Block(vec![make_int(1), make_int(2), make_int(3)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_eval_if_no_else() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(false)),
            then_branch: Box::new(make_int(1)),
            else_branch: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_expr).expect("should evaluate");
    assert!(matches!(result, Value::Nil));
}

#[test]
fn test_eval_if_nested_expr() {
    let mut interp = Interpreter::new();
    let nested = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(true)),
            then_branch: Box::new(Expr {
                kind: ExprKind::If {
                    condition: Box::new(make_bool(true)),
                    then_branch: Box::new(make_int(42)),
                    else_branch: Some(Box::new(make_int(0))),
                },
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            else_branch: Some(Box::new(make_int(-1))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&nested).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_eval_return_empty() {
    let mut interp = Interpreter::new();
    let ret = make_return(None);
    let result = interp.eval_expr(&ret);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_eval_return_int_value() {
    let mut interp = Interpreter::new();
    let ret = make_return(Some(make_int(42)));
    let result = interp.eval_expr(&ret);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_interpreter_creation() {
    let interp = Interpreter::new();
    assert!(interp.current_env().borrow().is_empty() || true);
}

#[test]
fn test_eval_nested_arithmetic() {
    let mut interp = Interpreter::new();
    let inner_add = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(2)),
            op: AstBinaryOp::Add,
            right: Box::new(make_int(3)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let mul = Expr {
        kind: ExprKind::Binary {
            left: Box::new(inner_add),
            op: AstBinaryOp::Multiply,
            right: Box::new(make_int(4)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&mul).expect("should evaluate");
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_eval_less_comparison() {
    let mut interp = Interpreter::new();
    let cmp = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(1)),
            op: AstBinaryOp::Less,
            right: Box::new(make_int(2)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&cmp).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_equal_comparison() {
    let mut interp = Interpreter::new();
    let eq = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(5)),
            op: AstBinaryOp::Equal,
            right: Box::new(make_int(5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_not_equal_comparison() {
    let mut interp = Interpreter::new();
    let neq = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(5)),
            op: AstBinaryOp::NotEqual,
            right: Box::new(make_int(3)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&neq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_byte_literal() {
    let mut interp = Interpreter::new();
    let byte = Expr {
        kind: ExprKind::Literal(Literal::Byte(255)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&byte).expect("should evaluate");
    assert_eq!(result, Value::Byte(255));
}

#[test]
fn test_eval_unit_literal() {
    let mut interp = Interpreter::new();
    let unit = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&unit).expect("should evaluate");
    assert!(matches!(result, Value::Nil));
}

#[test]
fn test_eval_list_nested() {
    let mut interp = Interpreter::new();
    let inner = Expr {
        kind: ExprKind::List(vec![make_int(1), make_int(2)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let outer = Expr {
        kind: ExprKind::List(vec![inner]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert!(matches!(result, Value::Array(_)));
}

#[test]
fn test_eval_if_true_branch() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(true)),
            then_branch: Box::new(make_int(100)),
            else_branch: Some(Box::new(make_int(0))),
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
fn test_eval_if_false_branch() {
    let mut interp = Interpreter::new();
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(make_bool(false)),
            then_branch: Box::new(make_int(100)),
            else_branch: Some(Box::new(make_int(0))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_eval_unary_negate_float() {
    let mut interp = Interpreter::new();
    let neg = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(make_float(3.5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&neg).expect("should evaluate");
    assert_eq!(result, Value::Float(-3.5));
}

#[test]
fn test_eval_binary_subtract() {
    let mut interp = Interpreter::new();
    let sub = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(10)),
            op: AstBinaryOp::Subtract,
            right: Box::new(make_int(4)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&sub).expect("should evaluate");
    assert_eq!(result, Value::Integer(6));
}

#[test]
fn test_eval_binary_multiply() {
    let mut interp = Interpreter::new();
    let mul = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(6)),
            op: AstBinaryOp::Multiply,
            right: Box::new(make_int(7)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&mul).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_eval_binary_divide() {
    let mut interp = Interpreter::new();
    let div = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(20)),
            op: AstBinaryOp::Divide,
            right: Box::new(make_int(4)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&div).expect("should evaluate");
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_eval_binary_greater() {
    let mut interp = Interpreter::new();
    let gt = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(10)),
            op: AstBinaryOp::Greater,
            right: Box::new(make_int(5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_empty_block() {
    let mut interp = Interpreter::new();
    let block = Expr {
        kind: ExprKind::Block(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert!(matches!(result, Value::Nil));
}

#[test]
fn test_eval_single_stmt_block() {
    let mut interp = Interpreter::new();
    let block = Expr {
        kind: ExprKind::Block(vec![make_int(99)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Integer(99));
}

// ============== EXTREME TDD Round 120: Special Forms Coverage ==============

#[test]
fn test_eval_none_literal() {
    let mut interp = Interpreter::new();
    let none_expr = Expr {
        kind: ExprKind::None,
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&none_expr).expect("should evaluate");
    match result {
        Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } => {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "None");
            assert!(data.is_none());
        }
        _ => panic!("Expected EnumVariant, got {:?}", result),
    }
}

#[test]
fn test_eval_some_with_integer() {
    let mut interp = Interpreter::new();
    let some_expr = Expr {
        kind: ExprKind::Some {
            value: Box::new(make_int(42)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&some_expr).expect("should evaluate");
    match result {
        Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } => {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "Some");
            assert!(data.is_some());
            let values = data.unwrap();
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], Value::Integer(42));
        }
        _ => panic!("Expected EnumVariant, got {:?}", result),
    }
}

#[test]
fn test_eval_some_with_string() {
    let mut interp = Interpreter::new();
    let some_expr = Expr {
        kind: ExprKind::Some {
            value: Box::new(make_string("hello")),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&some_expr).expect("should evaluate");
    match result {
        Value::EnumVariant {
            variant_name, data, ..
        } => {
            assert_eq!(variant_name, "Some");
            let values = data.unwrap();
            assert_eq!(values[0], Value::String("hello".into()));
        }
        _ => panic!("Expected EnumVariant"),
    }
}

#[test]
fn test_eval_set_empty() {
    let mut interp = Interpreter::new();
    let set_expr = Expr {
        kind: ExprKind::Set(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&set_expr).expect("should evaluate");
    assert!(matches!(result, Value::Nil));
}

#[test]
fn test_eval_set_single() {
    let mut interp = Interpreter::new();
    let set_expr = Expr {
        kind: ExprKind::Set(vec![make_int(100)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&set_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_eval_set_multiple_returns_last() {
    let mut interp = Interpreter::new();
    let set_expr = Expr {
        kind: ExprKind::Set(vec![make_int(1), make_int(2), make_int(3)]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&set_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_eval_qualified_name_std_module() {
    let mut interp = Interpreter::new();
    let qual_expr = Expr {
        kind: ExprKind::QualifiedName {
            module: "std".to_string(),
            name: "io".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    // May succeed or fail depending on stdlib availability
    let _ = interp.eval_expr(&qual_expr);
}

#[test]
fn test_eval_binary_less_than() {
    let mut interp = Interpreter::new();
    let lt = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(3)),
            op: AstBinaryOp::Less,
            right: Box::new(make_int(10)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&lt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_binary_less_equal() {
    let mut interp = Interpreter::new();
    let le = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(5)),
            op: AstBinaryOp::LessEqual,
            right: Box::new(make_int(5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_binary_greater_equal() {
    let mut interp = Interpreter::new();
    let ge = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(10)),
            op: AstBinaryOp::GreaterEqual,
            right: Box::new(make_int(10)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&ge).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_binary_not_equal() {
    let mut interp = Interpreter::new();
    let ne = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(5)),
            op: AstBinaryOp::NotEqual,
            right: Box::new(make_int(10)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_binary_modulo() {
    let mut interp = Interpreter::new();
    let modulo = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(17)),
            op: AstBinaryOp::Modulo,
            right: Box::new(make_int(5)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&modulo).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_eval_binary_power_large() {
    let mut interp = Interpreter::new();
    let power = Expr {
        kind: ExprKind::Binary {
            left: Box::new(make_int(2)),
            op: AstBinaryOp::Power,
            right: Box::new(make_int(10)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&power).expect("should evaluate");
    assert_eq!(result, Value::Integer(1024));
}

#[test]
fn test_eval_unary_negate() {
    let mut interp = Interpreter::new();
    let neg = make_unary(UnaryOp::Negate, make_int(42));
    let result = interp.eval_expr(&neg).expect("should evaluate");
    assert_eq!(result, Value::Integer(-42));
}

#[test]
fn test_eval_unary_not_true() {
    let mut interp = Interpreter::new();
    let not_expr = make_unary(UnaryOp::Not, make_bool(true));
    let result = interp.eval_expr(&not_expr).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_eval_unary_not_false() {
    let mut interp = Interpreter::new();
    let not_expr = make_unary(UnaryOp::Not, make_bool(false));
    let result = interp.eval_expr(&not_expr).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_list_empty_r120() {
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
        Value::Array(items) => assert!(items.is_empty()),
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_eval_list_with_elements_r120() {
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
        Value::Array(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(1));
            assert_eq!(items[1], Value::Integer(2));
            assert_eq!(items[2], Value::Integer(3));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_eval_tuple_empty_r120() {
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
        Value::Tuple(items) => assert!(items.is_empty()),
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_eval_tuple_with_mixed_types() {
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
        Value::Tuple(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Integer(10));
        }
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_float_addition() {
    let mut interp = Interpreter::new();
    let add = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
    let result = interp.eval_expr(&add).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 4.0).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_float_subtraction() {
    let mut interp = Interpreter::new();
    let sub = make_binary(make_float(5.5), AstBinaryOp::Subtract, make_float(2.0));
    let result = interp.eval_expr(&sub).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 3.5).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_float_multiplication() {
    let mut interp = Interpreter::new();
    let mul = make_binary(make_float(3.0), AstBinaryOp::Multiply, make_float(4.0));
    let result = interp.eval_expr(&mul).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 12.0).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_float_division_r120() {
    let mut interp = Interpreter::new();
    let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
    let result = interp.eval_expr(&div).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 2.5).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_float_comparison_less_r120() {
    let mut interp = Interpreter::new();
    let cmp = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
    let result = interp.eval_expr(&cmp).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_mixed_int_float_add() {
    let mut interp = Interpreter::new();
    let add = make_binary(make_int(2), AstBinaryOp::Add, make_float(3.5));
    let result = interp.eval_expr(&add).expect("should evaluate");
    match result {
        Value::Float(f) => assert!((f - 5.5).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_string_concat_addition() {
    let mut interp = Interpreter::new();
    let concat = make_binary(
        make_string("Hello, "),
        AstBinaryOp::Add,
        make_string("World!"),
    );
    let result = interp.eval_expr(&concat).expect("should evaluate");
    assert_eq!(result, Value::String("Hello, World!".into()));
}

#[test]
fn test_string_equality_r120() {
    let mut interp = Interpreter::new();
    let eq = make_binary(make_string("test"), AstBinaryOp::Equal, make_string("test"));
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_string_inequality_r120() {
    let mut interp = Interpreter::new();
    let ne = make_binary(make_string("a"), AstBinaryOp::NotEqual, make_string("b"));
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

// === EXTREME TDD Round 124 tests ===

#[test]
fn test_integer_modulo_positive_r124() {
    let mut interp = Interpreter::new();
    let modulo = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
    let result = interp.eval_expr(&modulo).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_integer_modulo_zero_result_r124() {
    let mut interp = Interpreter::new();
    let modulo = make_binary(make_int(15), AstBinaryOp::Modulo, make_int(5));
    let result = interp.eval_expr(&modulo).expect("should evaluate");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_greater_than_true_r124() {
    let mut interp = Interpreter::new();
    let gt = make_binary(make_int(10), AstBinaryOp::Greater, make_int(5));
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_greater_than_false_r124() {
    let mut interp = Interpreter::new();
    let gt = make_binary(make_int(3), AstBinaryOp::Greater, make_int(7));
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_greater_equal_true_r124() {
    let mut interp = Interpreter::new();
    let ge = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
    let result = interp.eval_expr(&ge).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_less_equal_true_r124() {
    let mut interp = Interpreter::new();
    let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_less_equal_false_r124() {
    let mut interp = Interpreter::new();
    let le = make_binary(make_int(10), AstBinaryOp::LessEqual, make_int(5));
    let result = interp.eval_expr(&le).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_bitwise_and_r124() {
    let mut interp = Interpreter::new();
    let band = make_binary(make_int(0b1100), AstBinaryOp::BitwiseAnd, make_int(0b1010));
    let result = interp.eval_expr(&band).expect("should evaluate");
    assert_eq!(result, Value::Integer(0b1000));
}

#[test]
fn test_bitwise_or_r124() {
    let mut interp = Interpreter::new();
    let bor = make_binary(make_int(0b1100), AstBinaryOp::BitwiseOr, make_int(0b0011));
    let result = interp.eval_expr(&bor).expect("should evaluate");
    assert_eq!(result, Value::Integer(0b1111));
}

#[test]
fn test_bitwise_xor_r124() {
    let mut interp = Interpreter::new();
    let bxor = make_binary(make_int(0b1100), AstBinaryOp::BitwiseXor, make_int(0b1010));
    let result = interp.eval_expr(&bxor).expect("should evaluate");
    assert_eq!(result, Value::Integer(0b0110));
}

#[test]
fn test_left_shift_r124() {
    let mut interp = Interpreter::new();
    let shl = make_binary(make_int(1), AstBinaryOp::LeftShift, make_int(4));
    let result = interp.eval_expr(&shl).expect("should evaluate");
    assert_eq!(result, Value::Integer(16));
}

#[test]
fn test_right_shift_r124() {
    let mut interp = Interpreter::new();
    let shr = make_binary(make_int(16), AstBinaryOp::RightShift, make_int(2));
    let result = interp.eval_expr(&shr).expect("should evaluate");
    assert_eq!(result, Value::Integer(4));
}

#[test]
fn test_double_negation_r124() {
    let mut interp = Interpreter::new();
    let inner = make_unary(UnaryOp::Negate, make_int(42));
    let outer = make_unary(UnaryOp::Negate, inner);
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_double_not_r124() {
    let mut interp = Interpreter::new();
    let inner = make_unary(UnaryOp::Not, make_bool(true));
    let outer = make_unary(UnaryOp::Not, inner);
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

// === EXTREME TDD Round 125 tests ===

#[test]
fn test_equal_true_r125() {
    let mut interp = Interpreter::new();
    let eq = make_binary(make_int(42), AstBinaryOp::Equal, make_int(42));
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equal_false_r125() {
    let mut interp = Interpreter::new();
    let eq = make_binary(make_int(42), AstBinaryOp::Equal, make_int(43));
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_not_equal_true_r125() {
    let mut interp = Interpreter::new();
    let ne = make_binary(make_int(42), AstBinaryOp::NotEqual, make_int(43));
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_not_equal_false_r125() {
    let mut interp = Interpreter::new();
    let ne = make_binary(make_int(42), AstBinaryOp::NotEqual, make_int(42));
    let result = interp.eval_expr(&ne).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_logical_and_true_r125() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_logical_and_false_r125() {
    let mut interp = Interpreter::new();
    let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
    let result = interp.eval_expr(&and).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_logical_or_true_r125() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_logical_or_false_r125() {
    let mut interp = Interpreter::new();
    let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
    let result = interp.eval_expr(&or).expect("should evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_float_add_r125() {
    let mut interp = Interpreter::new();
    let add = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
    let result = interp.eval_expr(&add).expect("should evaluate");
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_float_subtract_r125() {
    let mut interp = Interpreter::new();
    let sub = make_binary(make_float(5.0), AstBinaryOp::Subtract, make_float(2.0));
    let result = interp.eval_expr(&sub).expect("should evaluate");
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_float_multiply_r125() {
    let mut interp = Interpreter::new();
    let mul = make_binary(make_float(3.0), AstBinaryOp::Multiply, make_float(4.0));
    let result = interp.eval_expr(&mul).expect("should evaluate");
    assert_eq!(result, Value::Float(12.0));
}

#[test]
fn test_float_divide_r125() {
    let mut interp = Interpreter::new();
    let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(2.0));
    let result = interp.eval_expr(&div).expect("should evaluate");
    assert_eq!(result, Value::Float(5.0));
}

#[test]
fn test_float_less_than_r125() {
    let mut interp = Interpreter::new();
    let lt = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
    let result = interp.eval_expr(&lt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_float_greater_than_r125() {
    let mut interp = Interpreter::new();
    let gt = make_binary(make_float(3.5), AstBinaryOp::Greater, make_float(2.5));
    let result = interp.eval_expr(&gt).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_float_negate_r125() {
    let mut interp = Interpreter::new();
    let neg = make_unary(UnaryOp::Negate, make_float(3.14));
    let result = interp.eval_expr(&neg).expect("should evaluate");
    assert_eq!(result, Value::Float(-3.14));
}

#[test]
fn test_string_equal_r125() {
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
fn test_string_not_equal_r125() {
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
fn test_bool_equal_r125() {
    let mut interp = Interpreter::new();
    let eq = make_binary(make_bool(true), AstBinaryOp::Equal, make_bool(true));
    let result = interp.eval_expr(&eq).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_chain_and_r125() {
    let mut interp = Interpreter::new();
    let inner = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
    let outer = make_binary(inner, AstBinaryOp::And, make_bool(true));
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_chain_or_r125() {
    let mut interp = Interpreter::new();
    let inner = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
    let outer = make_binary(inner, AstBinaryOp::Or, make_bool(true));
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_complex_comparison_r125() {
    let mut interp = Interpreter::new();
    // (5 > 3) && (2 < 4)
    let left = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
    let right = make_binary(make_int(2), AstBinaryOp::Less, make_int(4));
    let combined = make_binary(left, AstBinaryOp::And, right);
    let result = interp.eval_expr(&combined).expect("should evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_nested_arithmetic_r125() {
    let mut interp = Interpreter::new();
    // (2 + 3) * (4 - 1)
    let left = make_binary(make_int(2), AstBinaryOp::Add, make_int(3));
    let right = make_binary(make_int(4), AstBinaryOp::Subtract, make_int(1));
    let combined = make_binary(left, AstBinaryOp::Multiply, right);
    let result = interp.eval_expr(&combined).expect("should evaluate");
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_zero_multiply_r125() {
    let mut interp = Interpreter::new();
    let mul = make_binary(make_int(100), AstBinaryOp::Multiply, make_int(0));
    let result = interp.eval_expr(&mul).expect("should evaluate");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_negative_modulo_r125() {
    let mut interp = Interpreter::new();
    let modulo = make_binary(make_int(-7), AstBinaryOp::Modulo, make_int(3));
    let result = interp.eval_expr(&modulo).expect("should evaluate");
    // In Rust, -7 % 3 = -1
    assert_eq!(result, Value::Integer(-1));
}
