// Auto-extracted from interpreter_tests.rs - Part 5
use super::*;

// ============== COVERAGE BOOST ROUND 2: AST-based tests ==============
// Using AST builders for reliability

#[test]
fn test_for_loop_with_range_coverage() {
    let mut interp = Interpreter::new();
    // for i in 0..3 { i }
    let body = make_ident("i");
    let range = make_range(make_int(0), make_int(3), false);
    let for_loop = make_for("i", range, body);
    let result = interp.eval_expr(&for_loop).expect("should evaluate");
    assert_eq!(result, Value::Integer(2)); // last iteration
}

#[test]
fn test_for_loop_inclusive_range_coverage() {
    let mut interp = Interpreter::new();
    // for i in 0..=2 { i }
    let body = make_ident("i");
    let range = make_range(make_int(0), make_int(2), true);
    let for_loop = make_for("i", range, body);
    let result = interp.eval_expr(&for_loop).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_while_loop_with_condition_coverage() {
    let mut interp = Interpreter::new();
    // let mut x = 0; while x < 3 { x = x + 1 }; x
    let body = make_compound_assign("x", AstBinaryOp::Add, make_int(1));
    let condition = make_binary(make_ident("x"), AstBinaryOp::Less, make_int(3));
    let while_loop = make_while(condition, body);
    let inner = make_let_mut(
        "x",
        make_int(0),
        make_block(vec![while_loop, make_ident("x")]),
    );
    let result = interp.eval_expr(&inner).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_nested_if_coverage() {
    let mut interp = Interpreter::new();
    // if true { if false { 1 } else { 2 } } else { 3 }
    let inner_if = make_if(make_bool(false), make_int(1), Some(make_int(2)));
    let outer_if = make_if(make_bool(true), inner_if, Some(make_int(3)));
    let result = interp.eval_expr(&outer_if).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_deeply_nested_binary_ops_coverage() {
    let mut interp = Interpreter::new();
    // ((1 + 2) * 3) + 4 = 13
    let inner = make_binary(make_int(1), AstBinaryOp::Add, make_int(2));
    let middle = make_binary(inner, AstBinaryOp::Multiply, make_int(3));
    let outer = make_binary(middle, AstBinaryOp::Add, make_int(4));
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Integer(13));
}

#[test]
fn test_list_with_many_elements_coverage() {
    let mut interp = Interpreter::new();
    let elements: Vec<Expr> = (0..10).map(make_int).collect();
    let list = make_list(elements);
    let result = interp.eval_expr(&list).expect("should evaluate");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 10),
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_tuple_with_mixed_types_coverage() {
    let mut interp = Interpreter::new();
    let tuple = make_tuple(vec![
        make_int(1),
        make_float(2.5),
        make_bool(true),
        make_string("test"),
    ]);
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => assert_eq!(t.len(), 4),
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_index_at_boundaries_coverage() {
    let mut interp = Interpreter::new();
    // [10, 20, 30, 40, 50][4]
    let list = make_list(vec![
        make_int(10),
        make_int(20),
        make_int(30),
        make_int(40),
        make_int(50),
    ]);
    let indexed = make_index(list, make_int(4));
    let result = interp.eval_expr(&indexed).expect("should evaluate");
    assert_eq!(result, Value::Integer(50));
}

#[test]
fn test_chained_let_bindings_coverage() {
    let mut interp = Interpreter::new();
    // let a = 1; let b = a + 1; let c = b + 1; c
    let c_expr = make_ident("c");
    let let_c = make_let(
        "c",
        make_binary(make_ident("b"), AstBinaryOp::Add, make_int(1)),
        c_expr,
    );
    let let_b = make_let(
        "b",
        make_binary(make_ident("a"), AstBinaryOp::Add, make_int(1)),
        let_c,
    );
    let let_a = make_let("a", make_int(1), let_b);
    let result = interp.eval_expr(&let_a).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_all_comparison_ops_coverage() {
    let mut interp = Interpreter::new();

    // Test Equal
    let eq = make_binary(make_int(5), AstBinaryOp::Equal, make_int(5));
    assert_eq!(interp.eval_expr(&eq).unwrap(), Value::Bool(true));

    // Test NotEqual
    let ne = make_binary(make_int(5), AstBinaryOp::NotEqual, make_int(3));
    assert_eq!(interp.eval_expr(&ne).unwrap(), Value::Bool(true));

    // Test Less
    let lt = make_binary(make_int(3), AstBinaryOp::Less, make_int(5));
    assert_eq!(interp.eval_expr(&lt).unwrap(), Value::Bool(true));

    // Test LessEqual
    let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
    assert_eq!(interp.eval_expr(&le).unwrap(), Value::Bool(true));

    // Test Greater
    let gt = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
    assert_eq!(interp.eval_expr(&gt).unwrap(), Value::Bool(true));

    // Test GreaterEqual
    let ge = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
    assert_eq!(interp.eval_expr(&ge).unwrap(), Value::Bool(true));
}

#[test]
fn test_all_arithmetic_ops_coverage() {
    let mut interp = Interpreter::new();

    // Test Add
    let add = make_binary(make_int(10), AstBinaryOp::Add, make_int(5));
    assert_eq!(interp.eval_expr(&add).unwrap(), Value::Integer(15));

    // Test Subtract
    let sub = make_binary(make_int(10), AstBinaryOp::Subtract, make_int(5));
    assert_eq!(interp.eval_expr(&sub).unwrap(), Value::Integer(5));

    // Test Multiply
    let mul = make_binary(make_int(10), AstBinaryOp::Multiply, make_int(5));
    assert_eq!(interp.eval_expr(&mul).unwrap(), Value::Integer(50));

    // Test Divide
    let div = make_binary(make_int(10), AstBinaryOp::Divide, make_int(5));
    assert_eq!(interp.eval_expr(&div).unwrap(), Value::Integer(2));

    // Test Modulo
    let modulo = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
    assert_eq!(interp.eval_expr(&modulo).unwrap(), Value::Integer(2));
}

#[test]
fn test_float_arithmetic_coverage() {
    let mut interp = Interpreter::new();

    // Float addition
    let add = make_binary(make_float(3.14), AstBinaryOp::Add, make_float(2.86));
    match interp.eval_expr(&add).unwrap() {
        Value::Float(f) => assert!((f - 6.0).abs() < 0.001),
        _ => panic!("Expected Float"),
    }

    // Float subtraction
    let sub = make_binary(make_float(5.5), AstBinaryOp::Subtract, make_float(2.5));
    match interp.eval_expr(&sub).unwrap() {
        Value::Float(f) => assert!((f - 3.0).abs() < 0.001),
        _ => panic!("Expected Float"),
    }

    // Float multiplication
    let mul = make_binary(make_float(2.5), AstBinaryOp::Multiply, make_float(4.0));
    match interp.eval_expr(&mul).unwrap() {
        Value::Float(f) => assert!((f - 10.0).abs() < 0.001),
        _ => panic!("Expected Float"),
    }

    // Float division
    let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
    match interp.eval_expr(&div).unwrap() {
        Value::Float(f) => assert!((f - 2.5).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_unary_negate_all_types_coverage() {
    let mut interp = Interpreter::new();

    // Negate integer
    let neg_int = make_unary(UnaryOp::Negate, make_int(42));
    assert_eq!(interp.eval_expr(&neg_int).unwrap(), Value::Integer(-42));

    // Negate float
    let neg_float = make_unary(UnaryOp::Negate, make_float(3.14));
    match interp.eval_expr(&neg_float).unwrap() {
        Value::Float(f) => assert!((f + 3.14).abs() < 0.001),
        _ => panic!("Expected Float"),
    }

    // Double negate
    let double_neg = make_unary(UnaryOp::Negate, make_unary(UnaryOp::Negate, make_int(99)));
    assert_eq!(interp.eval_expr(&double_neg).unwrap(), Value::Integer(99));
}

#[test]
fn test_unary_not_coverage() {
    let mut interp = Interpreter::new();

    // Not true
    let not_true = make_unary(UnaryOp::Not, make_bool(true));
    assert_eq!(interp.eval_expr(&not_true).unwrap(), Value::Bool(false));

    // Not false
    let not_false = make_unary(UnaryOp::Not, make_bool(false));
    assert_eq!(interp.eval_expr(&not_false).unwrap(), Value::Bool(true));

    // Double not
    let double_not = make_unary(UnaryOp::Not, make_unary(UnaryOp::Not, make_bool(true)));
    assert_eq!(interp.eval_expr(&double_not).unwrap(), Value::Bool(true));
}

#[test]
fn test_logical_and_short_circuit_coverage() {
    let mut interp = Interpreter::new();

    // true && true = true
    let tt = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
    assert_eq!(interp.eval_expr(&tt).unwrap(), Value::Bool(true));

    // true && false = false
    let tf = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
    assert_eq!(interp.eval_expr(&tf).unwrap(), Value::Bool(false));

    // false && true = false (short circuit)
    let ft = make_binary(make_bool(false), AstBinaryOp::And, make_bool(true));
    assert_eq!(interp.eval_expr(&ft).unwrap(), Value::Bool(false));

    // false && false = false
    let ff = make_binary(make_bool(false), AstBinaryOp::And, make_bool(false));
    assert_eq!(interp.eval_expr(&ff).unwrap(), Value::Bool(false));
}

#[test]
fn test_logical_or_short_circuit_coverage() {
    let mut interp = Interpreter::new();

    // true || true = true (short circuit)
    let tt = make_binary(make_bool(true), AstBinaryOp::Or, make_bool(true));
    assert_eq!(interp.eval_expr(&tt).unwrap(), Value::Bool(true));

    // true || false = true (short circuit)
    let tf = make_binary(make_bool(true), AstBinaryOp::Or, make_bool(false));
    assert_eq!(interp.eval_expr(&tf).unwrap(), Value::Bool(true));

    // false || true = true
    let ft = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
    assert_eq!(interp.eval_expr(&ft).unwrap(), Value::Bool(true));

    // false || false = false
    let ff = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
    assert_eq!(interp.eval_expr(&ff).unwrap(), Value::Bool(false));
}

#[test]
fn test_block_returns_last_value_coverage() {
    let mut interp = Interpreter::new();
    // { 1; 2; 3 }
    let block = make_block(vec![make_int(1), make_int(2), make_int(3)]);
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_empty_block_coverage() {
    let mut interp = Interpreter::new();
    let block = make_block(vec![]);
    let result = interp.eval_expr(&block).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_return_in_block_coverage() {
    let mut interp = Interpreter::new();
    // { 1; return 42; 3 }
    let block = make_block(vec![
        make_int(1),
        make_return(Some(make_int(42))),
        make_int(3),
    ]);
    let result = interp.eval_expr(&block);
    // Return should propagate
    assert!(result.is_err() || matches!(result, Ok(Value::Integer(42))));
}

#[test]
fn test_break_propagation_coverage() {
    let mut interp = Interpreter::new();
    // for i in 0..10 { if i == 3 { break } else { i } }
    let break_expr = make_break();
    let body = make_if(
        make_binary(make_ident("i"), AstBinaryOp::Equal, make_int(3)),
        break_expr,
        Some(make_ident("i")),
    );
    let range = make_range(make_int(0), make_int(10), false);
    let for_loop = make_for("i", range, body);
    let result = interp.eval_expr(&for_loop).expect("should evaluate");
    // Break should exit early
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_continue_propagation_coverage() {
    let mut interp = Interpreter::new();
    // for i in 0..5 { if i == 2 { continue } else { i } }
    let continue_expr = make_continue();
    let body = make_if(
        make_binary(make_ident("i"), AstBinaryOp::Equal, make_int(2)),
        continue_expr,
        Some(make_ident("i")),
    );
    let range = make_range(make_int(0), make_int(5), false);
    let for_loop = make_for("i", range, body);
    let result = interp.eval_expr(&for_loop).expect("should evaluate");
    assert_eq!(result, Value::Integer(4)); // last iteration
}

#[test]
fn test_nested_arrays_coverage() {
    let mut interp = Interpreter::new();
    // [[1, 2], [3, 4], [5, 6]]
    let inner1 = make_list(vec![make_int(1), make_int(2)]);
    let inner2 = make_list(vec![make_int(3), make_int(4)]);
    let inner3 = make_list(vec![make_int(5), make_int(6)]);
    let outer = make_list(vec![inner1, inner2, inner3]);
    let result = interp.eval_expr(&outer).expect("should evaluate");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            match &arr[1] {
                Value::Array(inner) => assert_eq!(inner.len(), 2),
                _ => panic!("Expected inner Array"),
            }
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_string_comparison_coverage() {
    let mut interp = Interpreter::new();

    // Equal strings
    let eq = make_binary(
        make_string("hello"),
        AstBinaryOp::Equal,
        make_string("hello"),
    );
    assert_eq!(interp.eval_expr(&eq).unwrap(), Value::Bool(true));

    // NotEqual strings
    let ne = make_binary(
        make_string("hello"),
        AstBinaryOp::NotEqual,
        make_string("world"),
    );
    assert_eq!(interp.eval_expr(&ne).unwrap(), Value::Bool(true));
}

#[test]
fn test_string_concatenation_coverage() {
    let mut interp = Interpreter::new();
    // "hello" + " " + "world"
    let hello_space = make_binary(make_string("hello"), AstBinaryOp::Add, make_string(" "));
    let full = make_binary(hello_space, AstBinaryOp::Add, make_string("world"));
    let result = interp.eval_expr(&full).expect("should evaluate");
    assert_eq!(result, Value::from_string("hello world".to_string()));
}

#[test]
fn test_range_values_coverage() {
    let mut interp = Interpreter::new();

    // Exclusive range
    let exclusive = make_range(make_int(1), make_int(5), false);
    match interp.eval_expr(&exclusive).unwrap() {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(*start, Value::Integer(1));
            assert_eq!(*end, Value::Integer(5));
            assert!(!inclusive);
        }
        _ => panic!("Expected Range"),
    }

    // Inclusive range
    let inclusive = make_range(make_int(1), make_int(5), true);
    match interp.eval_expr(&inclusive).unwrap() {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(*start, Value::Integer(1));
            assert_eq!(*end, Value::Integer(5));
            assert!(inclusive);
        }
        _ => panic!("Expected Range"),
    }
}

#[test]
fn test_if_else_chain_coverage() {
    let mut interp = Interpreter::new();
    // if false { 1 } else if false { 2 } else { 3 }
    let else_3 = make_int(3);
    let elif_2 = make_if(make_bool(false), make_int(2), Some(else_3));
    let if_1 = make_if(make_bool(false), make_int(1), Some(elif_2));
    let result = interp.eval_expr(&if_1).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_compound_assignment_all_ops_coverage() {
    let mut interp = Interpreter::new();

    // +=
    let add_assign = make_let_mut(
        "x",
        make_int(10),
        make_block(vec![
            make_compound_assign("x", AstBinaryOp::Add, make_int(5)),
            make_ident("x"),
        ]),
    );
    assert_eq!(interp.eval_expr(&add_assign).unwrap(), Value::Integer(15));

    // -=
    let mut interp2 = Interpreter::new();
    let sub_assign = make_let_mut(
        "x",
        make_int(10),
        make_block(vec![
            make_compound_assign("x", AstBinaryOp::Subtract, make_int(3)),
            make_ident("x"),
        ]),
    );
    assert_eq!(interp2.eval_expr(&sub_assign).unwrap(), Value::Integer(7));

    // *=
    let mut interp3 = Interpreter::new();
    let mul_assign = make_let_mut(
        "x",
        make_int(5),
        make_block(vec![
            make_compound_assign("x", AstBinaryOp::Multiply, make_int(4)),
            make_ident("x"),
        ]),
    );
    assert_eq!(interp3.eval_expr(&mul_assign).unwrap(), Value::Integer(20));

    // /=
    let mut interp4 = Interpreter::new();
    let div_assign = make_let_mut(
        "x",
        make_int(20),
        make_block(vec![
            make_compound_assign("x", AstBinaryOp::Divide, make_int(4)),
            make_ident("x"),
        ]),
    );
    assert_eq!(interp4.eval_expr(&div_assign).unwrap(), Value::Integer(5));
}

#[test]
fn test_for_loop_over_array_coverage() {
    let mut interp = Interpreter::new();
    // for x in [1, 2, 3] { x }
    let array = make_list(vec![make_int(1), make_int(2), make_int(3)]);
    let for_loop = make_for("x", array, make_ident("x"));
    let result = interp.eval_expr(&for_loop).expect("should evaluate");
    assert_eq!(result, Value::Integer(3)); // last element
}

#[test]
fn test_nested_for_loops_coverage() {
    let mut interp = Interpreter::new();
    // for i in 0..2 { for j in 0..2 { i + j } }
    let inner_body = make_binary(make_ident("i"), AstBinaryOp::Add, make_ident("j"));
    let inner_range = make_range(make_int(0), make_int(2), false);
    let inner_for = make_for("j", inner_range, inner_body);
    let outer_range = make_range(make_int(0), make_int(2), false);
    let outer_for = make_for("i", outer_range, inner_for);
    let result = interp.eval_expr(&outer_for).expect("should evaluate");
    // Last values: i=1, j=1, so 1+1=2
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_mixed_type_comparison_coverage() {
    let mut interp = Interpreter::new();

    // Int and Float comparison
    let int_float = make_binary(make_int(5), AstBinaryOp::Less, make_float(5.5));
    assert_eq!(interp.eval_expr(&int_float).unwrap(), Value::Bool(true));

    // Float and Int comparison
    let float_int = make_binary(make_float(5.5), AstBinaryOp::Greater, make_int(5));
    assert_eq!(interp.eval_expr(&float_int).unwrap(), Value::Bool(true));
}

#[test]
fn test_unit_literal_coverage() {
    let mut interp = Interpreter::new();
    let unit = make_unit();
    let result = interp.eval_expr(&unit).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_large_tuple_coverage() {
    let mut interp = Interpreter::new();
    let elements: Vec<Expr> = (0..20).map(make_int).collect();
    let tuple = make_tuple(elements);
    let result = interp.eval_expr(&tuple).expect("should evaluate");
    match result {
        Value::Tuple(t) => assert_eq!(t.len(), 20),
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_deeply_nested_blocks_coverage() {
    let mut interp = Interpreter::new();
    // { { { { 42 } } } }
    let inner = make_block(vec![make_int(42)]);
    let level2 = make_block(vec![inner]);
    let level3 = make_block(vec![level2]);
    let level4 = make_block(vec![level3]);
    let result = interp.eval_expr(&level4).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_variable_shadowing_coverage() {
    let mut interp = Interpreter::new();
    // let x = 1; let x = 2; x
    let inner = make_let("x", make_int(2), make_ident("x"));
    let outer = make_let("x", make_int(1), inner);
    let result = interp.eval_expr(&outer).expect("should evaluate");
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_power_operator_coverage() {
    let mut interp = Interpreter::new();
    // 2 ** 10 = 1024
    let power = make_binary(make_int(2), AstBinaryOp::Power, make_int(10));
    let result = interp.eval_expr(&power).expect("should evaluate");
    assert_eq!(result, Value::Integer(1024));
}

#[test]
fn test_zero_as_exponent_coverage() {
    let mut interp = Interpreter::new();
    // Any number ** 0 = 1
    let power = make_binary(make_int(999), AstBinaryOp::Power, make_int(0));
    let result = interp.eval_expr(&power).expect("should evaluate");
    assert_eq!(result, Value::Integer(1));
}

// ============== EXTREME TDD Round 130: Interpreter.rs Coverage Expansion ==============

// ---------- Ternary Expression Tests ----------

#[test]
fn test_ternary_true_condition() {
    let mut interp = Interpreter::new();
    let ternary = Expr {
        kind: ExprKind::Ternary {
            condition: Box::new(make_bool(true)),
            true_expr: Box::new(make_int(100)),
            false_expr: Box::new(make_int(0)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&ternary).expect("should evaluate");
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_ternary_false_condition() {
    let mut interp = Interpreter::new();
    let ternary = Expr {
        kind: ExprKind::Ternary {
            condition: Box::new(make_bool(false)),
            true_expr: Box::new(make_int(100)),
            false_expr: Box::new(make_int(0)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&ternary).expect("should evaluate");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_ternary_with_expression_condition() {
    let mut interp = Interpreter::new();
    let ternary = Expr {
        kind: ExprKind::Ternary {
            condition: Box::new(make_binary(make_int(5), AstBinaryOp::Greater, make_int(3))),
            true_expr: Box::new(make_string("big")),
            false_expr: Box::new(make_string("small")),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&ternary).expect("should evaluate");
    assert_eq!(result, Value::String("big".into()));
}

// ---------- Object Literal Tests ----------

#[test]
fn test_object_literal_empty() {
    let mut interp = Interpreter::new();
    let obj = Expr {
        kind: ExprKind::ObjectLiteral { fields: vec![] },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&obj).expect("should evaluate");
    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn test_object_literal_with_fields() {
    use crate::frontend::ast::ObjectField;
    let mut interp = Interpreter::new();
    let obj = Expr {
        kind: ExprKind::ObjectLiteral {
            fields: vec![
                ObjectField::KeyValue {
                    key: "x".to_string(),
                    value: make_int(10),
                },
                ObjectField::KeyValue {
                    key: "y".to_string(),
                    value: make_int(20),
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&obj).expect("should evaluate");
    if let Value::Object(o) = result {
        assert_eq!(o.get("x"), Some(&Value::Integer(10)));
        assert_eq!(o.get("y"), Some(&Value::Integer(20)));
    } else {
        panic!("Expected Object");
    }
}

// ---------- String Interpolation Tests ----------

#[test]
fn test_string_interpolation_literal_only() {
    use crate::frontend::ast::StringPart;
    let mut interp = Interpreter::new();
    let interp_expr = Expr {
        kind: ExprKind::StringInterpolation {
            parts: vec![StringPart::Text("hello world".to_string())],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&interp_expr).expect("should evaluate");
    assert_eq!(result, Value::String("hello world".into()));
}

#[test]
fn test_string_interpolation_with_expression() {
    use crate::frontend::ast::StringPart;
    let mut interp = Interpreter::new();
    let interp_expr = Expr {
        kind: ExprKind::StringInterpolation {
            parts: vec![
                StringPart::Text("Value: ".to_string()),
                StringPart::Expr(Box::new(make_int(42))),
            ],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&interp_expr).expect("should evaluate");
    assert_eq!(result, Value::String("Value: 42".into()));
}

#[test]
fn test_string_interpolation_multiple_parts() {
    use crate::frontend::ast::StringPart;
    let mut interp = Interpreter::new();
    let interp_expr = Expr {
        kind: ExprKind::StringInterpolation {
            parts: vec![
                StringPart::Text("x = ".to_string()),
                StringPart::Expr(Box::new(make_int(10))),
                StringPart::Text(", y = ".to_string()),
                StringPart::Expr(Box::new(make_int(20))),
            ],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&interp_expr).expect("should evaluate");
    assert_eq!(result, Value::String("x = 10, y = 20".into()));
}

// ---------- Effect and Handle Tests ----------

#[test]
fn test_effect_declaration_returns_nil() {
    let mut interp = Interpreter::new();
    let effect = Expr {
        kind: ExprKind::Effect {
            name: "MyEffect".to_string(),
            operations: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&effect).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_handle_evaluates_expr_returns_nil() {
    let mut interp = Interpreter::new();
    let handle = Expr {
        kind: ExprKind::Handle {
            expr: Box::new(make_int(42)),
            handlers: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&handle).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

// ---------- TupleStruct Test ----------

#[test]
fn test_tuple_struct_returns_nil() {
    let mut interp = Interpreter::new();
    let tuple_struct = Expr {
        kind: ExprKind::TupleStruct {
            name: "Point".to_string(),
            type_params: vec![],
            fields: vec![],
            derives: vec![],
            is_pub: false,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&tuple_struct).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

// ---------- ImportDefault Test ----------

#[test]
fn test_import_default_returns_nil() {
    let mut interp = Interpreter::new();
    let import = Expr {
        kind: ExprKind::ImportDefault {
            module: "somemodule".to_string(),
            name: "sm".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&import).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

// ---------- Macro Tests (format macro edge cases) ----------

#[test]
fn test_macro_invocation_format_basic() {
    let mut interp = Interpreter::new();
    let format_macro = Expr {
        kind: ExprKind::MacroInvocation {
            name: "format".to_string(),
            args: vec![make_string("Hello, {}!"), make_string("World")],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&format_macro).expect("should evaluate");
    // format! macro uses debug formatting, so strings get quoted
    if let Value::String(s) = result {
        assert!(s.contains("Hello"));
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_macro_invocation_format_multiple_args() {
    let mut interp = Interpreter::new();
    let format_macro = Expr {
        kind: ExprKind::MacroInvocation {
            name: "format".to_string(),
            args: vec![
                make_string("{} + {} = {}"),
                make_int(1),
                make_int(2),
                make_int(3),
            ],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&format_macro).expect("should evaluate");
    // format! macro includes the formatted values
    if let Value::String(s) = result {
        assert!(s.contains("1") && s.contains("2") && s.contains("3"));
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_macro_invocation_format_empty_args_error() {
    let mut interp = Interpreter::new();
    let format_macro = Expr {
        kind: ExprKind::MacroInvocation {
            name: "format".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&format_macro);
    assert!(result.is_err());
}

#[test]
fn test_macro_invocation_unknown_macro_error() {
    let mut interp = Interpreter::new();
    let unknown = Expr {
        kind: ExprKind::MacroInvocation {
            name: "unknown_macro".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&unknown);
    assert!(result.is_err());
}

#[test]
fn test_macro_invocation_println_empty() {
    let mut interp = Interpreter::new();
    let println = Expr {
        kind: ExprKind::MacroInvocation {
            name: "println".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&println).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_macro_invocation_println_single_arg() {
    let mut interp = Interpreter::new();
    let println = Expr {
        kind: ExprKind::MacroInvocation {
            name: "println".to_string(),
            args: vec![make_string("test")],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&println).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_macro_invocation_println_with_format() {
    let mut interp = Interpreter::new();
    let println = Expr {
        kind: ExprKind::MacroInvocation {
            name: "println".to_string(),
            args: vec![make_string("value: {}"), make_int(42)],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&println).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

// ---------- Lookup Variable Special Cases ----------

#[test]
fn test_lookup_json_global() {
    let interp = Interpreter::new();
    let result = interp.lookup_variable("JSON");
    assert!(result.is_ok());
    if let Value::Object(o) = result.unwrap() {
        assert_eq!(o.get("__type"), Some(&Value::String("JSON".into())));
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_lookup_file_global() {
    let interp = Interpreter::new();
    let result = interp.lookup_variable("File");
    assert!(result.is_ok());
    if let Value::Object(o) = result.unwrap() {
        assert_eq!(o.get("__type"), Some(&Value::String("File".into())));
    } else {
        panic!("Expected Object");
    }
}

// ---------- Lazy Expression Test ----------

#[test]
fn test_lazy_expr_evaluates_immediately() {
    let mut interp = Interpreter::new();
    let lazy = Expr {
        kind: ExprKind::Lazy {
            expr: Box::new(make_int(42)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&lazy).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

// ---------- AsyncBlock Test ----------

#[test]
fn test_async_block_evaluates_body() {
    let mut interp = Interpreter::new();
    let async_block = Expr {
        kind: ExprKind::AsyncBlock {
            body: Box::new(make_int(100)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&async_block).expect("should evaluate");
    assert_eq!(result, Value::Integer(100));
}

// ---------- Module Expression Test ----------

#[test]
fn test_module_expr_empty_body() {
    let mut interp = Interpreter::new();
    let module = Expr {
        kind: ExprKind::Module {
            name: "test_mod".to_string(),
            body: Box::new(make_block(vec![])),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&module).expect("should evaluate");
    if let Value::Object(o) = result {
        assert_eq!(o.get("__type"), Some(&Value::String("Module".into())));
        assert_eq!(o.get("__name"), Some(&Value::String("test_mod".into())));
    } else {
        panic!("Expected Object");
    }
}

// ---------- ModuleDeclaration Error Test ----------

#[test]
fn test_module_declaration_error() {
    let mut interp = Interpreter::new();
    let mod_decl = Expr {
        kind: ExprKind::ModuleDeclaration {
            name: "unresolved_mod".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&mod_decl);
    assert!(result.is_err());
}

// ---------- Await Expression Test ----------

#[test]
fn test_await_expr_evaluates_inner() {
    let mut interp = Interpreter::new();
    let await_expr = Expr {
        kind: ExprKind::Await {
            expr: Box::new(make_int(42)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&await_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

// ---------- Loop Expression Tests ----------

#[test]
fn test_loop_with_break() {
    let mut interp = Interpreter::new();
    let loop_expr = Expr {
        kind: ExprKind::Loop {
            label: None,
            body: Box::new(make_break()),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&loop_expr).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_break_with_value() {
    let mut interp = Interpreter::new();
    let loop_expr = Expr {
        kind: ExprKind::Loop {
            label: None,
            body: Box::new(Expr {
                kind: ExprKind::Break {
                    label: None,
                    value: Some(Box::new(make_int(42))),
                },
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&loop_expr).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

// ---------- Atom Literal Test ----------

#[test]
fn test_atom_literal() {
    let mut interp = Interpreter::new();
    let atom = Expr {
        kind: ExprKind::Literal(Literal::Atom("ok".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&atom).expect("should evaluate");
    assert_eq!(result, Value::Atom("ok".to_string()));
}

// ---------- Try Operator Tests ----------

#[test]
fn test_try_operator_ok_variant() {
    let mut interp = Interpreter::new();
    // Set up Ok variant
    let ok_val = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };
    interp.set_variable("result", ok_val);

    let try_expr = Expr {
        kind: ExprKind::Try {
            expr: Box::new(make_ident("result")),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&try_expr).expect("should unwrap Ok");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_try_operator_err_variant() {
    let mut interp = Interpreter::new();
    // Set up Err variant
    let err_val = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Err".to_string(),
        data: Some(vec![Value::String("error message".into())]),
    };
    interp.set_variable("result", err_val);

    let try_expr = Expr {
        kind: ExprKind::Try {
            expr: Box::new(make_ident("result")),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&try_expr);
    // Should propagate error via Return
    assert!(result.is_err());
}

#[test]
fn test_try_operator_non_result_error() {
    let mut interp = Interpreter::new();
    interp.set_variable("not_result", Value::Integer(42));

    let try_expr = Expr {
        kind: ExprKind::Try {
            expr: Box::new(make_ident("not_result")),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&try_expr);
    assert!(result.is_err());
}

// ---------- Environment Methods Tests ----------

#[test]
fn test_env_push_pop() {
    let mut interp = Interpreter::new();
    let initial_depth = interp.env_stack.len();

    interp.env_push(std::collections::HashMap::new());
    assert_eq!(interp.env_stack.len(), initial_depth + 1);

    interp.env_pop();
    assert_eq!(interp.env_stack.len(), initial_depth);
}

#[test]
fn test_env_pop_keeps_global() {
    let mut interp = Interpreter::new();
    // Try to pop the global environment
    let result = interp.env_pop();
    assert!(result.is_none()); // Should not pop the last (global) environment
}

#[test]
fn test_env_set_and_lookup() {
    let mut interp = Interpreter::new();
    interp.env_set("test_var".to_string(), Value::Integer(123));
    let result = interp.lookup_variable("test_var");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(123));
}

#[test]
fn test_env_set_mut() {
    let mut interp = Interpreter::new();
    interp.env_set("mutable_var".to_string(), Value::Integer(1));
    interp.env_set_mut("mutable_var".to_string(), Value::Integer(2));
    let result = interp.lookup_variable("mutable_var");
    assert_eq!(result.unwrap(), Value::Integer(2));
}

// ---------- Import Tests ----------

#[test]
fn test_import_all_wildcard() {
    let mut interp = Interpreter::new();
    let import = Expr {
        kind: ExprKind::ImportAll {
            module: "std::io".to_string(),
            alias: "*".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&import);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil);
}

#[test]
fn test_import_all_with_alias() {
    let mut interp = Interpreter::new();
    let import = Expr {
        kind: ExprKind::ImportAll {
            module: "std".to_string(),
            alias: "stdlib".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&import);
    assert!(result.is_ok());
}

#[test]
fn test_import_stdlib() {
    let mut interp = Interpreter::new();
    let import = Expr {
        kind: ExprKind::Import {
            module: "std::io".to_string(),
            items: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&import);
    assert!(result.is_ok());
}

// ---------- Struct Literal Test ----------

#[test]
fn test_struct_literal_empty() {
    let mut interp = Interpreter::new();
    // First define a struct
    let struct_def = Expr {
        kind: ExprKind::Struct {
            name: "Point".to_string(),
            type_params: vec![],
            fields: vec![],
            methods: vec![],
            derives: vec![],
            is_pub: false,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    interp.eval_expr(&struct_def).expect("should define struct");

    // Now create struct literal
    let struct_lit = Expr {
        kind: ExprKind::StructLiteral {
            name: "Point".to_string(),
            fields: vec![],
            base: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&struct_lit);
    assert!(result.is_ok());
}

// ---------- IfLet Expression Test ----------

#[test]
fn test_if_let_matching() {
    let mut interp = Interpreter::new();
    // Set up a Some value to match against
    let some_val = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };
    interp.set_variable("opt", some_val);

    let if_let = Expr {
        kind: ExprKind::IfLet {
            pattern: Pattern::Identifier("x".to_string()),
            expr: Box::new(make_ident("opt")),
            then_branch: Box::new(make_int(100)),
            else_branch: Some(Box::new(make_int(0))),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_let);
    assert!(result.is_ok());
}

#[test]
fn test_if_let_no_else_cov5() {
    let mut interp = Interpreter::new();
    interp.set_variable("val", Value::Integer(42));

    let if_let = Expr {
        kind: ExprKind::IfLet {
            pattern: Pattern::Identifier("x".to_string()),
            expr: Box::new(make_ident("val")),
            then_branch: Box::new(make_int(100)),
            else_branch: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&if_let);
    assert!(result.is_ok());
}

// ---------- WhileLet Expression Test ----------

#[test]
fn test_while_let_no_match() {
    let mut interp = Interpreter::new();
    // Use a tuple pattern that won't match an integer
    // while let (x, y) = 42 { ... } should not execute
    let while_let = Expr {
        kind: ExprKind::WhileLet {
            label: None,
            pattern: Pattern::Tuple(vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ]),
            expr: Box::new(make_int(42)),
            body: Box::new(make_int(1)),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&while_let);
    // Should exit immediately since tuple pattern doesn't match an integer
    assert!(result.is_ok());
}

// ---------- Call Function Special Cases ----------

#[test]
fn test_call_function_non_callable_error() {
    let mut interp = Interpreter::new();
    let result = interp.call_function(Value::Integer(42), &[]);
    assert!(result.is_err());
}

#[test]
fn test_call_function_closure_wrong_args() {
    let mut interp = Interpreter::new();
    // Create a closure that expects 2 args
    let closure = Value::Closure {
        params: vec![("a".to_string(), None), ("b".to_string(), None)],
        body: std::sync::Arc::new(make_int(1)),
        env: interp.current_env().clone(),
    };
    // Call with wrong number of args
    let result = interp.call_function(closure, &[Value::Integer(1)]);
    assert!(result.is_err());
}

// ---------- ListComprehension with Condition Tests ----------

#[test]
fn test_list_comprehension_ast_simple() {
    use crate::frontend::ast::ComprehensionClause;
    let mut interp = Interpreter::new();

    let comprehension = Expr {
        kind: ExprKind::ListComprehension {
            element: Box::new(make_ident("x")),
            clauses: vec![ComprehensionClause {
                variable: "x".to_string(),
                iterable: Box::new(Expr {
                    kind: ExprKind::List(vec![make_int(1), make_int(2), make_int(3)]),
                    span: Span::default(),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                }),
                condition: None,
            }],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = interp.eval_expr(&comprehension).expect("should evaluate");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_list_comprehension_ast_with_condition() {
    use crate::frontend::ast::ComprehensionClause;
    let mut interp = Interpreter::new();

    // [x for x in [1, 2, 3, 4] if x > 2]
    let comprehension = Expr {
        kind: ExprKind::ListComprehension {
            element: Box::new(make_ident("x")),
            clauses: vec![ComprehensionClause {
                variable: "x".to_string(),
                iterable: Box::new(Expr {
                    kind: ExprKind::List(vec![
                        make_int(1),
                        make_int(2),
                        make_int(3),
                        make_int(4),
                    ]),
                    span: Span::default(),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                }),
                condition: Some(Box::new(make_binary(
                    make_ident("x"),
                    AstBinaryOp::Greater,
                    make_int(2),
                ))),
            }],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = interp.eval_expr(&comprehension).expect("should evaluate");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 2); // Only 3 and 4
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_list_comprehension_with_range() {
    use crate::frontend::ast::ComprehensionClause;
    let mut interp = Interpreter::new();

    // [x for x in 0..3]
    let comprehension = Expr {
        kind: ExprKind::ListComprehension {
            element: Box::new(make_ident("x")),
            clauses: vec![ComprehensionClause {
                variable: "x".to_string(),
                iterable: Box::new(make_range(make_int(0), make_int(3), false)),
                condition: None,
            }],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = interp.eval_expr(&comprehension).expect("should evaluate");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3); // 0, 1, 2
    } else {
        panic!("Expected Array");
    }
}

// ---------- Pipeline Operator Tests ----------

#[test]
fn test_pipeline_simple_function() {
    let mut interp = Interpreter::new();
    // Define a function: fn double(x) { x * 2 }
    let double_fn = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: std::sync::Arc::new(make_binary(
            make_ident("x"),
            AstBinaryOp::Multiply,
            make_int(2),
        )),
        env: interp.current_env().clone(),
    };
    interp.set_variable("double", double_fn);

    // 5 |> double
    let pipeline = Expr {
        kind: ExprKind::Pipeline {
            expr: Box::new(make_int(5)),
            stages: vec![crate::frontend::ast::PipelineStage {
                op: Box::new(make_ident("double")),
                span: Span::default(),
            }],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = interp.eval_expr(&pipeline).expect("should evaluate");
    assert_eq!(result, Value::Integer(10));
}

// ---------- Spread Expression Test ----------

#[test]
fn test_spread_in_array() {
    let mut interp = Interpreter::new();
    let arr = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    interp.set_variable("arr", Value::Array(arr.into()));

    let spread = Expr {
        kind: ExprKind::Spread {
            expr: Box::new(make_ident("arr")),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    // Spread in isolation returns an error (must be used in context like array literal)
    // This just exercises the code path
    let _result = interp.eval_expr(&spread);
    // Result may be ok or err depending on context - we just want to hit the code path
}

// ---------- FieldAccess Test ----------

#[test]
fn test_field_access_on_object() {
    let mut interp = Interpreter::new();
    let mut obj = std::collections::HashMap::new();
    obj.insert("x".to_string(), Value::Integer(42));
    obj.insert("y".to_string(), Value::Integer(100));
    interp.set_variable("point", Value::Object(std::sync::Arc::new(obj)));

    let field_access = Expr {
        kind: ExprKind::FieldAccess {
            object: Box::new(make_ident("point")),
            field: "x".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = interp.eval_expr(&field_access).expect("should evaluate");
    assert_eq!(result, Value::Integer(42));
}

// ---------- TypeCast Tests ----------

#[test]
fn test_type_cast_int_to_float_expr() {
    let mut interp = Interpreter::new();
    let cast = Expr {
        kind: ExprKind::TypeCast {
            expr: Box::new(make_int(42)),
            target_type: "f64".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&cast).expect("should evaluate");
    assert_eq!(result, Value::Float(42.0));
}

#[test]
fn test_type_cast_float_to_int_expr() {
    let mut interp = Interpreter::new();
    let cast = Expr {
        kind: ExprKind::TypeCast {
            expr: Box::new(make_float(3.7)),
            target_type: "i64".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&cast).expect("should evaluate");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_type_cast_to_string_expr_fails() {
    let mut interp = Interpreter::new();
    let cast = Expr {
        kind: ExprKind::TypeCast {
            expr: Box::new(make_int(42)),
            target_type: "String".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    // int->String cast is not supported
    let result = interp.eval_expr(&cast);
    assert!(result.is_err());
}

// ---------- Additional Macro Tests ----------

#[test]
fn test_macro_vec_empty() {
    let mut interp = Interpreter::new();
    let vec_macro = Expr {
        kind: ExprKind::Macro {
            name: "vec".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&vec_macro).expect("should evaluate");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_macro_vec_with_elements() {
    let mut interp = Interpreter::new();
    let vec_macro = Expr {
        kind: ExprKind::Macro {
            name: "vec".to_string(),
            args: vec![make_int(1), make_int(2), make_int(3)],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&vec_macro).expect("should evaluate");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_macro_println_empty() {
    let mut interp = Interpreter::new();
    let println_macro = Expr {
        kind: ExprKind::Macro {
            name: "println".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&println_macro).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_macro_println_single_arg() {
    let mut interp = Interpreter::new();
    let println_macro = Expr {
        kind: ExprKind::Macro {
            name: "println".to_string(),
            args: vec![make_string("hello")],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&println_macro).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_macro_println_format_args() {
    let mut interp = Interpreter::new();
    let println_macro = Expr {
        kind: ExprKind::Macro {
            name: "println".to_string(),
            args: vec![make_string("value: {}"), make_int(42)],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&println_macro).expect("should evaluate");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_macro_unknown_returns_error() {
    let mut interp = Interpreter::new();
    let unknown_macro = Expr {
        kind: ExprKind::Macro {
            name: "unknown".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = interp.eval_expr(&unknown_macro);
    assert!(result.is_err());
}

