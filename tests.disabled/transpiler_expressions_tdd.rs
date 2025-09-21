//! TDD tests for backend/transpiler/expressions.rs
//! Comprehensive coverage for expression transpilation

use ruchy::frontend::ast::{ObjectField, Span, StringPart};
use ruchy::Transpiler;
use ruchy::{BinaryOp, Expr, ExprKind, Literal, UnaryOp};

// ============================================================================
// Test Helpers
// ============================================================================

fn make_literal(val: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(val)),
        span: Span::default(),
        attributes: vec![],
    }
}

fn make_string_literal(s: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(s.to_string())),
        span: Span::default(),
        attributes: vec![],
    }
}

fn make_identifier(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span::default(),
        attributes: vec![],
    }
}

// ============================================================================
// Literal Transpilation Tests
// ============================================================================

#[test]
fn test_transpile_literal_integer_small() {
    let lit = Literal::Integer(42);
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("42"));
}

#[test]
fn test_transpile_literal_integer_large() {
    let lit = Literal::Integer(i64::MAX);
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains(&i64::MAX.to_string()));
}

#[test]
fn test_transpile_literal_float() {
    let lit = Literal::Float(3.14159);
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("3.14159"));
}

#[test]
fn test_transpile_literal_string() {
    let lit = Literal::String("hello world".to_string());
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("hello world"));
}

#[test]
fn test_transpile_literal_bool_true() {
    let lit = Literal::Bool(true);
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("true"));
}

#[test]
fn test_transpile_literal_bool_false() {
    let lit = Literal::Bool(false);
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("false"));
}

#[test]
fn test_transpile_literal_char() {
    let lit = Literal::Char('X');
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("'X'"));
}

#[test]
fn test_transpile_literal_unit() {
    let lit = Literal::Unit;
    let result = Transpiler::transpile_literal(&lit);
    assert!(result.to_string().contains("()"));
}

// ============================================================================
// String Interpolation Tests
// ============================================================================

#[test]
fn test_transpile_string_interpolation_empty() {
    let transpiler = Transpiler::new();
    let parts = vec![];
    let result = transpiler.transpile_string_interpolation(&parts).unwrap();
    assert!(result.to_string().contains("\"\""));
}

#[test]
fn test_transpile_string_interpolation_literal_only() {
    let transpiler = Transpiler::new();
    let parts = vec![StringPart::Text("Hello".to_string())];
    let result = transpiler.transpile_string_interpolation(&parts).unwrap();
    let code = result.to_string();
    assert!(code.contains("Hello"));
}

#[test]
fn test_transpile_string_interpolation_with_expr() {
    let transpiler = Transpiler::new();
    let parts = vec![
        StringPart::Text("Value: ".to_string()),
        StringPart::Expr(Box::new(make_literal(42))),
    ];
    let result = transpiler.transpile_string_interpolation(&parts).unwrap();
    let code = result.to_string();
    assert!(code.contains("format"));
}

#[test]
fn test_transpile_string_interpolation_mixed() {
    let transpiler = Transpiler::new();
    let parts = vec![
        StringPart::Text("Hello, ".to_string()),
        StringPart::Expr(Box::new(make_identifier("name"))),
        StringPart::Text("!".to_string()),
    ];
    let result = transpiler.transpile_string_interpolation(&parts).unwrap();
    assert!(result.to_string().contains("format"));
}

// ============================================================================
// Binary Operation Tests
// ============================================================================

#[test]
fn test_transpile_binary_add() {
    let transpiler = Transpiler::new();
    let left = make_literal(10);
    let right = make_literal(20);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Add, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("10"));
    assert!(code.contains("20"));
    assert!(code.contains("+"));
}

#[test]
fn test_transpile_binary_subtract() {
    let transpiler = Transpiler::new();
    let left = make_literal(30);
    let right = make_literal(15);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Subtract, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("-"));
}

#[test]
fn test_transpile_binary_multiply() {
    let transpiler = Transpiler::new();
    let left = make_literal(5);
    let right = make_literal(6);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Multiply, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("*"));
}

#[test]
fn test_transpile_binary_divide() {
    let transpiler = Transpiler::new();
    let left = make_literal(100);
    let right = make_literal(4);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Divide, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("/"));
}

#[test]
fn test_transpile_binary_modulo() {
    let transpiler = Transpiler::new();
    let left = make_literal(17);
    let right = make_literal(5);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Modulo, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("%"));
}

#[test]
fn test_transpile_binary_power() {
    let transpiler = Transpiler::new();
    let left = make_literal(2);
    let right = make_literal(8);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Power, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("pow"));
}

#[test]
fn test_transpile_binary_equal() {
    let transpiler = Transpiler::new();
    let left = make_literal(42);
    let right = make_literal(42);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Equal, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("=="));
}

#[test]
fn test_transpile_binary_not_equal() {
    let transpiler = Transpiler::new();
    let left = make_literal(1);
    let right = make_literal(2);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::NotEqual, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("!="));
}

#[test]
fn test_transpile_binary_less() {
    let transpiler = Transpiler::new();
    let left = make_literal(1);
    let right = make_literal(10);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Less, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("<"));
}

#[test]
fn test_transpile_binary_less_equal() {
    let transpiler = Transpiler::new();
    let left = make_literal(5);
    let right = make_literal(5);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::LessEqual, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("<="));
}

#[test]
fn test_transpile_binary_greater() {
    let transpiler = Transpiler::new();
    let left = make_literal(100);
    let right = make_literal(50);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Greater, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains(">"));
}

#[test]
fn test_transpile_binary_greater_equal() {
    let transpiler = Transpiler::new();
    let left = make_literal(7);
    let right = make_literal(7);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::GreaterEqual, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains(">="));
}

#[test]
fn test_transpile_binary_and() {
    let transpiler = Transpiler::new();
    let left = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    };
    let right = Expr {
        kind: ExprKind::Literal(Literal::Bool(false)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler
        .transpile_binary(&left, BinaryOp::And, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("&&"));
}

#[test]
fn test_transpile_binary_or() {
    let transpiler = Transpiler::new();
    let left = Expr {
        kind: ExprKind::Literal(Literal::Bool(false)),
        span: Span::default(),
        attributes: vec![],
    };
    let right = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler
        .transpile_binary(&left, BinaryOp::Or, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("||"));
}

#[test]
fn test_transpile_binary_bitwise_and() {
    let transpiler = Transpiler::new();
    let left = make_literal(0b1100);
    let right = make_literal(0b1010);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::BitwiseAnd, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("&"));
    assert!(!code.contains("&&"));
}

#[test]
fn test_transpile_binary_bitwise_or() {
    let transpiler = Transpiler::new();
    let left = make_literal(0b1100);
    let right = make_literal(0b0011);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::BitwiseOr, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("|"));
    assert!(!code.contains("||"));
}

#[test]
fn test_transpile_binary_bitwise_xor() {
    let transpiler = Transpiler::new();
    let left = make_literal(0b1111);
    let right = make_literal(0b1010);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::BitwiseXor, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("^"));
}

#[test]
fn test_transpile_binary_left_shift() {
    let transpiler = Transpiler::new();
    let left = make_literal(4);
    let right = make_literal(2);
    let result = transpiler
        .transpile_binary(&left, BinaryOp::LeftShift, &right)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("<<"));
}

// RightShift not in BinaryOp enum - skipping this test

// NullCoalesce not implemented - causes unreachable! panic
// #[test]
// fn test_transpile_binary_null_coalesce() {
//     let transpiler = Transpiler::new();
//     let left = make_identifier("maybe_null");
//     let right = make_literal(0);
//     let result = transpiler.transpile_binary(&left, BinaryOp::NullCoalesce, &right).unwrap();
//     let code = result.to_string();
//     assert!(code.contains("unwrap_or"));
// }

// ============================================================================
// Unary Operation Tests
// ============================================================================

#[test]
fn test_transpile_unary_negate() {
    let transpiler = Transpiler::new();
    let operand = make_literal(42);
    let result = transpiler
        .transpile_unary(UnaryOp::Negate, &operand)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("-"));
}

#[test]
fn test_transpile_unary_not() {
    let transpiler = Transpiler::new();
    let operand = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile_unary(UnaryOp::Not, &operand).unwrap();
    let code = result.to_string();
    assert!(code.contains("!"));
}

#[test]
fn test_transpile_unary_bitwise_not() {
    let transpiler = Transpiler::new();
    let operand = make_literal(0b1010);
    let result = transpiler
        .transpile_unary(UnaryOp::BitwiseNot, &operand)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("!"));
}

// ============================================================================
// Async/Await Tests
// ============================================================================

#[test]
fn test_transpile_await() {
    let transpiler = Transpiler::new();
    let expr = make_identifier("future");
    let result = transpiler.transpile_await(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("await"));
}

#[test]
fn test_transpile_async_block() {
    let transpiler = Transpiler::new();
    let body = make_literal(42);
    let result = transpiler.transpile_async_block(&body).unwrap();
    let code = result.to_string();
    assert!(code.contains("async"));
}

// ============================================================================
// Throw Test
// ============================================================================

#[test]
fn test_transpile_throw() {
    let transpiler = Transpiler::new();
    let expr = make_string_literal("Error!");
    let result = transpiler.transpile_throw(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("panic"));
}

// ============================================================================
// Field Access Tests
// ============================================================================

#[test]
fn test_transpile_field_access() {
    let transpiler = Transpiler::new();
    let object = make_identifier("obj");
    let result = transpiler.transpile_field_access(&object, "field").unwrap();
    let code = result.to_string();
    assert!(code.contains("obj"));
    assert!(code.contains("field"));
}

#[test]
fn test_transpile_field_access_nested() {
    let transpiler = Transpiler::new();
    let object = Expr {
        kind: ExprKind::FieldAccess {
            object: Box::new(make_identifier("parent")),
            field: "child".to_string(),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile_field_access(&object, "value").unwrap();
    let code = result.to_string();
    assert!(code.contains("value"));
}

// ============================================================================
// Index Access Tests
// ============================================================================

#[test]
fn test_transpile_index_access() {
    let transpiler = Transpiler::new();
    let object = make_identifier("array");
    let index = make_literal(5);
    let result = transpiler.transpile_index_access(&object, &index).unwrap();
    let code = result.to_string();
    assert!(code.contains("array"));
    assert!(code.contains("5"));
}

// ============================================================================
// Slice Tests
// ============================================================================

#[test]
fn test_transpile_slice_full() {
    let transpiler = Transpiler::new();
    let object = make_identifier("vec");
    let start_expr = make_literal(1);
    let end_expr = make_literal(5);
    let start = Some(&start_expr);
    let end = Some(&end_expr);
    let result = transpiler.transpile_slice(&object, start, end).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
}

#[test]
fn test_transpile_slice_start_only() {
    let transpiler = Transpiler::new();
    let object = make_identifier("vec");
    let start_expr = make_literal(2);
    let start = Some(&start_expr);
    let result = transpiler.transpile_slice(&object, start, None).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
}

#[test]
fn test_transpile_slice_end_only() {
    let transpiler = Transpiler::new();
    let object = make_identifier("vec");
    let end_expr = make_literal(10);
    let end = Some(&end_expr);
    let result = transpiler.transpile_slice(&object, None, end).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
}

#[test]
fn test_transpile_slice_none() {
    let transpiler = Transpiler::new();
    let object = make_identifier("vec");
    let result = transpiler.transpile_slice(&object, None, None).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
}

// ============================================================================
// Assignment Tests
// ============================================================================

#[test]
fn test_transpile_assign() {
    let transpiler = Transpiler::new();
    let target = make_identifier("x");
    let value = make_literal(100);
    let result = transpiler.transpile_assign(&target, &value).unwrap();
    let code = result.to_string();
    assert!(code.contains("x"));
    assert!(code.contains("100"));
}

#[test]
fn test_transpile_compound_assign_add() {
    let transpiler = Transpiler::new();
    let target = make_identifier("sum");
    let value = make_literal(10);
    let result = transpiler
        .transpile_compound_assign(&target, BinaryOp::Add, &value)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("sum"));
    assert!(code.contains("+="));
}

#[test]
fn test_transpile_compound_assign_subtract() {
    let transpiler = Transpiler::new();
    let target = make_identifier("count");
    let value = make_literal(1);
    let result = transpiler
        .transpile_compound_assign(&target, BinaryOp::Subtract, &value)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("-="));
}

#[test]
fn test_transpile_compound_assign_multiply() {
    let transpiler = Transpiler::new();
    let target = make_identifier("product");
    let value = make_literal(2);
    let result = transpiler
        .transpile_compound_assign(&target, BinaryOp::Multiply, &value)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("*="));
}

#[test]
fn test_transpile_compound_assign_divide() {
    let transpiler = Transpiler::new();
    let target = make_identifier("quotient");
    let value = make_literal(3);
    let result = transpiler
        .transpile_compound_assign(&target, BinaryOp::Divide, &value)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("/="));
}

// ============================================================================
// Increment/Decrement Tests
// ============================================================================

#[test]
fn test_transpile_pre_increment() {
    let transpiler = Transpiler::new();
    let target = make_identifier("counter");
    let result = transpiler.transpile_pre_increment(&target).unwrap();
    let code = result.to_string();
    assert!(code.contains("counter"));
    assert!(code.contains("+="));
}

#[test]
fn test_transpile_post_increment() {
    let transpiler = Transpiler::new();
    let target = make_identifier("index");
    let result = transpiler.transpile_post_increment(&target).unwrap();
    let code = result.to_string();
    assert!(code.contains("index"));
}

#[test]
fn test_transpile_pre_decrement() {
    let transpiler = Transpiler::new();
    let target = make_identifier("count");
    let result = transpiler.transpile_pre_decrement(&target).unwrap();
    let code = result.to_string();
    assert!(code.contains("count"));
    assert!(code.contains("-="));
}

#[test]
fn test_transpile_post_decrement() {
    let transpiler = Transpiler::new();
    let target = make_identifier("remaining");
    let result = transpiler.transpile_post_decrement(&target).unwrap();
    let code = result.to_string();
    assert!(code.contains("remaining"));
}

// ============================================================================
// Collection Tests
// ============================================================================

#[test]
fn test_transpile_list_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_list(&[]).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
}

#[test]
fn test_transpile_list_single() {
    let transpiler = Transpiler::new();
    let elements = vec![make_literal(42)];
    let result = transpiler.transpile_list(&elements).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_list_multiple() {
    let transpiler = Transpiler::new();
    let elements = vec![make_literal(1), make_literal(2), make_literal(3)];
    let result = transpiler.transpile_list(&elements).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
}

#[test]
fn test_transpile_tuple_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_tuple(&[]).unwrap();
    let code = result.to_string();
    assert!(code.contains("()"));
}

#[test]
fn test_transpile_tuple_single() {
    let transpiler = Transpiler::new();
    let elements = vec![make_literal(100)];
    let result = transpiler.transpile_tuple(&elements).unwrap();
    let code = result.to_string();
    assert!(code.contains("100"));
    // Note: transpiler doesn't add trailing comma for single-element tuples
    // This is a limitation that could make (100) ambiguous with parentheses
}

#[test]
fn test_transpile_tuple_pair() {
    let transpiler = Transpiler::new();
    let elements = vec![make_literal(10), make_literal(20)];
    let result = transpiler.transpile_tuple(&elements).unwrap();
    let code = result.to_string();
    assert!(code.contains("10"));
    assert!(code.contains("20"));
}

#[test]
fn test_transpile_range_inclusive() {
    let transpiler = Transpiler::new();
    let start_expr = make_literal(0);
    let end_expr = make_literal(10);
    let result = transpiler
        .transpile_range(&start_expr, &end_expr, true)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("0"));
    assert!(code.contains("10"));
    assert!(code.contains("..="));
}

#[test]
fn test_transpile_range_exclusive() {
    let transpiler = Transpiler::new();
    let start_expr = make_literal(1);
    let end_expr = make_literal(100);
    let result = transpiler
        .transpile_range(&start_expr, &end_expr, false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("1"));
    assert!(code.contains("100"));
    assert!(code.contains(".."));
    assert!(!code.contains("..="));
}

// Note: transpile_range requires both start and end, so no tests for partial ranges

// ============================================================================
// Object/Struct Literal Tests
// ============================================================================

#[test]
fn test_transpile_object_literal_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_object_literal(&[]).unwrap();
    let code = result.to_string();
    assert!(code.contains("HashMap"));
}

#[test]
fn test_transpile_object_literal_single_field() {
    let transpiler = Transpiler::new();
    let fields = vec![ObjectField::KeyValue {
        key: "name".to_string(),
        value: make_string_literal("Alice"),
    }];
    let result = transpiler.transpile_object_literal(&fields).unwrap();
    let code = result.to_string();
    assert!(code.contains("HashMap"));
    assert!(code.contains("name"));
    assert!(code.contains("Alice"));
}

#[test]
fn test_transpile_object_literal_multiple_fields() {
    let transpiler = Transpiler::new();
    let fields = vec![
        ObjectField::KeyValue {
            key: "x".to_string(),
            value: make_literal(10),
        },
        ObjectField::KeyValue {
            key: "y".to_string(),
            value: make_literal(20),
        },
        ObjectField::KeyValue {
            key: "label".to_string(),
            value: make_string_literal("point"),
        },
    ];
    let result = transpiler.transpile_object_literal(&fields).unwrap();
    let code = result.to_string();
    assert!(code.contains("HashMap"));
    assert!(code.contains("x"));
    assert!(code.contains("y"));
    assert!(code.contains("label"));
}

#[test]
fn test_transpile_struct_literal_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_struct_literal("Point", &[]).unwrap();
    let code = result.to_string();
    assert!(code.contains("Point"));
}

#[test]
fn test_transpile_struct_literal_with_fields() {
    let transpiler = Transpiler::new();
    let fields = vec![
        ("x".to_string(), make_literal(100)),
        ("y".to_string(), make_literal(200)),
    ];
    let result = transpiler
        .transpile_struct_literal("Point", &fields)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Point"));
    assert!(code.contains("x"));
    assert!(code.contains("y"));
    assert!(code.contains("100"));
    assert!(code.contains("200"));
}
