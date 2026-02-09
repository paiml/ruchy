// Auto-extracted from interpreter_tests.rs - Part 1
use super::*;

// ============== Literal Tests ==============

#[test]
fn test_eval_integer_literal() {
    let mut interp = Interpreter::new();
    let expr = make_int(42);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(42)
    );
}

#[test]
fn test_eval_negative_integer() {
    let mut interp = Interpreter::new();
    let expr = make_int(-100);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(-100)
    );
}

#[test]
fn test_eval_large_integer() {
    let mut interp = Interpreter::new();
    let expr = make_int(i64::MAX);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(i64::MAX)
    );
}

#[test]
fn test_eval_float_literal() {
    let mut interp = Interpreter::new();
    let expr = make_float(3.14159);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Float(f) => assert!((f - 3.14159).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_eval_bool_true() {
    let mut interp = Interpreter::new();
    let expr = make_bool(true);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_bool_false() {
    let mut interp = Interpreter::new();
    let expr = make_bool(false);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_eval_string_literal() {
    let mut interp = Interpreter::new();
    let expr = make_string("hello");
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), "hello"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_eval_empty_string() {
    let mut interp = Interpreter::new();
    let expr = make_string("");
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), ""),
        _ => panic!("Expected string"),
    }
}

// ============== Arithmetic Tests ==============

#[test]
fn test_eval_addition() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(10), AstBinaryOp::Add, make_int(5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(15)
    );
}

#[test]
fn test_eval_subtraction() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(10), AstBinaryOp::Subtract, make_int(3));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(7)
    );
}

#[test]
fn test_eval_multiplication() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(4), AstBinaryOp::Multiply, make_int(7));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(28)
    );
}

#[test]
fn test_eval_division() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(20), AstBinaryOp::Divide, make_int(4));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(5)
    );
}

#[test]
fn test_eval_modulo() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(2)
    );
}

#[test]
fn test_eval_float_addition() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Float(f) => assert!((f - 4.0).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_eval_mixed_int_float() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::Add, make_float(2.5));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

// ============== Comparison Tests ==============

#[test]
fn test_eval_equal_true() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::Equal, make_int(5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_equal_false() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::Equal, make_int(3));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_eval_not_equal() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::NotEqual, make_int(3));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_less_than() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(3), AstBinaryOp::Less, make_int(5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_less_than_false() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::Less, make_int(3));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_eval_greater_than() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(7), AstBinaryOp::Greater, make_int(2));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_less_equal() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_greater_equal() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

// ============== Logical Operator Tests ==============

#[test]
fn test_eval_and_true() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_and_false() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_eval_or_true() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_or_false() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_eval_not_true() {
    let mut interp = Interpreter::new();
    let expr = make_unary(UnaryOp::Not, make_bool(true));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_eval_not_false() {
    let mut interp = Interpreter::new();
    let expr = make_unary(UnaryOp::Not, make_bool(false));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_negate_int() {
    let mut interp = Interpreter::new();
    let expr = make_unary(UnaryOp::Negate, make_int(42));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(-42)
    );
}

#[test]
fn test_eval_negate_float() {
    let mut interp = Interpreter::new();
    let expr = make_unary(UnaryOp::Negate, make_float(3.14));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Float(f) => assert!((f - (-3.14)).abs() < 0.0001),
        _ => panic!("Expected float"),
    }
}

// ============== String Operation Tests ==============

#[test]
fn test_eval_string_concatenation() {
    let mut interp = Interpreter::new();
    let expr = make_binary(
        make_string("Hello"),
        AstBinaryOp::Add,
        make_string(" World"),
    );
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), "Hello World"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_eval_string_with_int() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_string("Value: "), AstBinaryOp::Add, make_int(42));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), "Value: 42"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_eval_string_equality() {
    let mut interp = Interpreter::new();
    let expr = make_binary(
        make_string("hello"),
        AstBinaryOp::Equal,
        make_string("hello"),
    );
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_eval_string_inequality() {
    let mut interp = Interpreter::new();
    let expr = make_binary(
        make_string("hello"),
        AstBinaryOp::NotEqual,
        make_string("world"),
    );
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

// ============== Variable Tests via Let ==============

#[test]
fn test_let_binding() {
    let mut interp = Interpreter::new();
    let expr = make_let("x", make_int(42), make_ident("x"));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(42)
    );
}

#[test]
fn test_let_with_computation() {
    let mut interp = Interpreter::new();
    let expr = make_let(
        "x",
        make_int(10),
        make_binary(make_ident("x"), AstBinaryOp::Multiply, make_int(2)),
    );
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(20)
    );
}

#[test]
fn test_nested_let() {
    let mut interp = Interpreter::new();
    let inner_let = make_let(
        "y",
        make_int(20),
        make_binary(make_ident("x"), AstBinaryOp::Add, make_ident("y")),
    );
    let expr = make_let("x", make_int(10), inner_let);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(30)
    );
}

#[test]
fn test_let_shadowing() {
    let mut interp = Interpreter::new();
    let inner_let = make_let("x", make_int(20), make_ident("x"));
    let expr = make_let("x", make_int(10), inner_let);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(20)
    );
}

#[test]
fn test_variable_not_found() {
    let mut interp = Interpreter::new();
    let expr = make_ident("undefined_var");
    assert!(interp.eval_expr(&expr).is_err());
}

// ============== Array Tests ==============

#[test]
fn test_eval_empty_array() {
    let mut interp = Interpreter::new();
    let expr = make_list(vec![]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Array(a) => assert!(a.is_empty()),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_eval_array_literal() {
    let mut interp = Interpreter::new();
    let expr = make_list(vec![make_int(1), make_int(2), make_int(3)]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Array(a) => {
            assert_eq!(a.len(), 3);
            assert_eq!(a[0], Value::Integer(1));
            assert_eq!(a[1], Value::Integer(2));
            assert_eq!(a[2], Value::Integer(3));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_eval_nested_array() {
    let mut interp = Interpreter::new();
    let inner = make_list(vec![make_int(1), make_int(2)]);
    let expr = make_list(vec![inner, make_list(vec![make_int(3), make_int(4)])]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Array(a) => {
            assert_eq!(a.len(), 2);
            match &a[0] {
                Value::Array(inner) => assert_eq!(inner.len(), 2),
                _ => panic!("Expected inner array"),
            }
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_array_index_access() {
    let mut interp = Interpreter::new();
    let arr = make_list(vec![make_int(10), make_int(20), make_int(30)]);
    let expr = make_let("arr", arr, make_index(make_ident("arr"), make_int(1)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(20)
    );
}

#[test]
fn test_array_negative_index() {
    let mut interp = Interpreter::new();
    let arr = make_list(vec![make_int(10), make_int(20), make_int(30)]);
    let expr = make_let("arr", arr, make_index(make_ident("arr"), make_int(-1)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(30)
    );
}

// ============== Tuple Tests ==============

#[test]
fn test_eval_tuple_literal() {
    let mut interp = Interpreter::new();
    let expr = make_tuple(vec![make_int(1), make_string("hello"), make_bool(true)]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 3);
            assert_eq!(t[0], Value::Integer(1));
        }
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_tuple_index_access() {
    let mut interp = Interpreter::new();
    let t = make_tuple(vec![make_int(10), make_int(20)]);
    let expr = make_let("t", t, make_index(make_ident("t"), make_int(0)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(10)
    );
}

// ============== If Expression Tests ==============

#[test]
fn test_if_true_branch() {
    let mut interp = Interpreter::new();
    let expr = make_if(make_bool(true), make_int(10), Some(make_int(20)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(10)
    );
}

#[test]
fn test_if_false_branch() {
    let mut interp = Interpreter::new();
    let expr = make_if(make_bool(false), make_int(10), Some(make_int(20)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(20)
    );
}

#[test]
fn test_if_no_else() {
    let mut interp = Interpreter::new();
    let expr = make_if(make_bool(false), make_int(10), None);
    assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Nil);
}

#[test]
fn test_nested_if() {
    let mut interp = Interpreter::new();
    let inner_if = make_if(make_bool(true), make_int(1), Some(make_int(2)));
    let expr = make_if(make_bool(true), inner_if, Some(make_int(3)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(1)
    );
}

#[test]
fn test_if_with_comparison() {
    let mut interp = Interpreter::new();
    let condition = make_binary(make_int(10), AstBinaryOp::Greater, make_int(5));
    let expr = make_if(condition, make_string("big"), Some(make_string("small")));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), "big"),
        _ => panic!("Expected string"),
    }
}

// ============== Block Expression Tests ==============

#[test]
fn test_empty_block() {
    let mut interp = Interpreter::new();
    let expr = make_block(vec![]);
    assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Nil);
}

#[test]
fn test_block_returns_last() {
    let mut interp = Interpreter::new();
    let expr = make_block(vec![make_int(1), make_int(2), make_int(3)]);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(3)
    );
}

// ============== Range Tests ==============

#[test]
fn test_exclusive_range() {
    let mut interp = Interpreter::new();
    let expr = make_range(make_int(0), make_int(5), false);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(*start, Value::Integer(0));
            assert_eq!(*end, Value::Integer(5));
            assert!(!inclusive);
        }
        _ => panic!("Expected range"),
    }
}

#[test]
fn test_inclusive_range() {
    let mut interp = Interpreter::new();
    let expr = make_range(make_int(0), make_int(5), true);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Range { inclusive, .. } => assert!(inclusive),
        _ => panic!("Expected range"),
    }
}

// ============== String Index Tests ==============

#[test]
fn test_string_index() {
    let mut interp = Interpreter::new();
    let s = make_string("hello");
    let expr = make_let("s", s, make_index(make_ident("s"), make_int(0)));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), "h"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_negative_index() {
    let mut interp = Interpreter::new();
    let s = make_string("hello");
    let expr = make_let("s", s, make_index(make_ident("s"), make_int(-1)));
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::String(s) => assert_eq!(s.as_ref(), "o"),
        _ => panic!("Expected string"),
    }
}

// ============== Error Handling Tests ==============

#[test]
fn test_division_by_zero() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(10), AstBinaryOp::Divide, make_int(0));
    assert!(interp.eval_expr(&expr).is_err());
}

#[test]
fn test_modulo_by_zero() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(10), AstBinaryOp::Modulo, make_int(0));
    assert!(interp.eval_expr(&expr).is_err());
}

// ============== Complex Expression Tests ==============

#[test]
fn test_nested_arithmetic() {
    let mut interp = Interpreter::new();
    // (2 + 3) * (4 - 1) = 5 * 3 = 15
    let left = make_binary(make_int(2), AstBinaryOp::Add, make_int(3));
    let right = make_binary(make_int(4), AstBinaryOp::Subtract, make_int(1));
    let expr = make_binary(left, AstBinaryOp::Multiply, right);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(15)
    );
}

#[test]
fn test_chained_comparisons() {
    let mut interp = Interpreter::new();
    // true && (5 > 3) = true
    let cmp = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
    let expr = make_binary(make_bool(true), AstBinaryOp::And, cmp);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_expression_in_if_condition() {
    let mut interp = Interpreter::new();
    // if (10 + 5) > 10 { 1 } else { 0 }
    let sum = make_binary(make_int(10), AstBinaryOp::Add, make_int(5));
    let condition = make_binary(sum, AstBinaryOp::Greater, make_int(10));
    let expr = make_if(condition, make_int(1), Some(make_int(0)));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(1)
    );
}

#[test]
fn test_complex_let_with_if() {
    let mut interp = Interpreter::new();
    // let x = 10 in if x > 5 { x * 2 } else { x }
    let condition = make_binary(make_ident("x"), AstBinaryOp::Greater, make_int(5));
    let then_branch = make_binary(make_ident("x"), AstBinaryOp::Multiply, make_int(2));
    let if_expr = make_if(condition, then_branch, Some(make_ident("x")));
    let expr = make_let("x", make_int(10), if_expr);
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(20)
    );
}

// ============== Additional Coverage Tests ==============

#[test]
fn test_power_operator() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(2), AstBinaryOp::Power, make_int(3));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(8)
    );
}

#[test]
fn test_bitwise_and() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(0b1100), AstBinaryOp::BitwiseAnd, make_int(0b1010));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(0b1000)
    );
}

#[test]
fn test_bitwise_or_cov5() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(0b1100), AstBinaryOp::BitwiseOr, make_int(0b1010));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(0b1110)
    );
}

#[test]
fn test_bitwise_xor_cov5() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(0b1100), AstBinaryOp::BitwiseXor, make_int(0b1010));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(0b0110)
    );
}

#[test]
fn test_left_shift() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(1), AstBinaryOp::LeftShift, make_int(4));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(16)
    );
}

#[test]
fn test_right_shift() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_int(16), AstBinaryOp::RightShift, make_int(2));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Integer(4)
    );
}

#[test]
fn test_deep_nesting() {
    let mut interp = Interpreter::new();
    // ((1 + 2) + 3) + 4 = 10
    let e1 = make_binary(make_int(1), AstBinaryOp::Add, make_int(2));
    let e2 = make_binary(e1, AstBinaryOp::Add, make_int(3));
    let e3 = make_binary(e2, AstBinaryOp::Add, make_int(4));
    assert_eq!(
        interp.eval_expr(&e3).expect("should succeed"),
        Value::Integer(10)
    );
}

#[test]
fn test_many_let_bindings() {
    let mut interp = Interpreter::new();
    // let a = 1 in let b = 2 in let c = 3 in a + b + c
    let sum_bc = make_binary(make_ident("b"), AstBinaryOp::Add, make_ident("c"));
    let sum_abc = make_binary(make_ident("a"), AstBinaryOp::Add, sum_bc);
    let let_c = make_let("c", make_int(3), sum_abc);
    let let_b = make_let("b", make_int(2), let_c);
    let let_a = make_let("a", make_int(1), let_b);
    assert_eq!(
        interp.eval_expr(&let_a).expect("should succeed"),
        Value::Integer(6)
    );
}

#[test]
fn test_mixed_types_in_array() {
    let mut interp = Interpreter::new();
    let expr = make_list(vec![
        make_int(1),
        make_float(2.5),
        make_bool(true),
        make_string("hi"),
    ]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Array(a) => assert_eq!(a.len(), 4),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_empty_tuple() {
    let mut interp = Interpreter::new();
    let expr = make_tuple(vec![]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Tuple(t) => assert!(t.is_empty()),
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_single_element_tuple() {
    let mut interp = Interpreter::new();
    let expr = make_tuple(vec![make_int(42)]);
    match interp.eval_expr(&expr).expect("should succeed") {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 1);
            assert_eq!(t[0], Value::Integer(42));
        }
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_float_comparison() {
    let mut interp = Interpreter::new();
    let expr = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_string_comparison_lt() {
    let mut interp = Interpreter::new();
    let expr = make_binary(
        make_string("apple"),
        AstBinaryOp::Less,
        make_string("banana"),
    );
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_short_circuit_and() {
    let mut interp = Interpreter::new();
    // false && (error) should not evaluate the right side
    let expr = make_binary(make_bool(false), AstBinaryOp::And, make_ident("undefined"));
    // This should succeed without error due to short-circuit
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(false)
    );
}

#[test]
fn test_short_circuit_or() {
    let mut interp = Interpreter::new();
    // true || (error) should not evaluate the right side
    let expr = make_binary(make_bool(true), AstBinaryOp::Or, make_ident("undefined"));
    // This should succeed without error due to short-circuit
    assert_eq!(
        interp.eval_expr(&expr).expect("should succeed"),
        Value::Bool(true)
    );
}

#[test]
fn test_interpreter_new_returns_valid() {
    let interp = Interpreter::new();
    // Just verify creation works
    drop(interp);
}

#[test]
fn test_multiple_operations_same_interpreter() {
    let mut interp = Interpreter::new();

    // First operation
    let e1 = make_int(10);
    assert_eq!(
        interp.eval_expr(&e1).expect("should succeed"),
        Value::Integer(10)
    );

    // Second operation
    let e2 = make_binary(make_int(20), AstBinaryOp::Add, make_int(30));
    assert_eq!(
        interp.eval_expr(&e2).expect("should succeed"),
        Value::Integer(50)
    );

    // Third operation with let
    let e3 = make_let("x", make_int(5), make_ident("x"));
    assert_eq!(
        interp.eval_expr(&e3).expect("should succeed"),
        Value::Integer(5)
    );
}

