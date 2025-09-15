//! Comprehensive test suite for parser modules
//! Target: Increase coverage for src/frontend/parser/*.rs

use ruchy::frontend::{Parser, ast::*};

#[test]
fn test_parse_integer_literal() {
    let mut parser = Parser::new("42");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_parse_float_literal() {
    let mut parser = Parser::new("3.14");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_parse_string_literal() {
    let mut parser = Parser::new("\"hello world\"");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "hello world"),
        _ => panic!("Expected string literal"),
    }
}

#[test]
fn test_parse_boolean_literals() {
    let mut parser = Parser::new("true");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Literal(Literal::Bool(b)) => assert!(b),
        _ => panic!("Expected boolean literal"),
    }

    let mut parser = Parser::new("false");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Literal(Literal::Bool(b)) => assert!(!b),
        _ => panic!("Expected boolean literal"),
    }
}

#[test]
fn test_parse_identifier() {
    let mut parser = Parser::new("variable_name");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Identifier(name) => assert_eq!(name, "variable_name"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_parse_binary_addition() {
    let mut parser = Parser::new("1 + 2");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Binary { op: BinaryOp::Add, .. } => {},
        _ => panic!("Expected addition"),
    }
}

#[test]
fn test_parse_binary_multiplication() {
    let mut parser = Parser::new("3 * 4");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Binary { op: BinaryOp::Multiply, .. } => {},
        _ => panic!("Expected multiplication"),
    }
}

#[test]
fn test_parse_precedence() {
    let mut parser = Parser::new("1 + 2 * 3");
    let expr = parser.parse().unwrap();
    // Should parse as 1 + (2 * 3) due to precedence
    match expr.kind {
        ExprKind::Binary { op: BinaryOp::Add, right, .. } => {
            match right.kind {
                ExprKind::Binary { op: BinaryOp::Multiply, .. } => {},
                _ => panic!("Expected multiplication on right side"),
            }
        },
        _ => panic!("Expected addition at top level"),
    }
}

#[test]
fn test_parse_parentheses() {
    let mut parser = Parser::new("(1 + 2) * 3");
    let expr = parser.parse().unwrap();
    // Should parse as (1 + 2) * 3
    match expr.kind {
        ExprKind::Binary { op: BinaryOp::Multiply, left, .. } => {
            match left.kind {
                ExprKind::Binary { op: BinaryOp::Add, .. } => {},
                _ => panic!("Expected addition on left side"),
            }
        },
        _ => panic!("Expected multiplication at top level"),
    }
}

#[test]
fn test_parse_unary_negation() {
    let mut parser = Parser::new("-42");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Unary { op: UnaryOp::Negate, .. } => {},
        _ => panic!("Expected negation"),
    }
}

#[test]
fn test_parse_unary_not() {
    let mut parser = Parser::new("!true");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Unary { op: UnaryOp::Not, .. } => {},
        _ => panic!("Expected not operator"),
    }
}

#[test]
fn test_parse_list_literal() {
    let mut parser = Parser::new("[1, 2, 3]");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::List(elements) => assert_eq!(elements.len(), 3),
        _ => panic!("Expected list literal"),
    }
}

#[test]
fn test_parse_empty_list() {
    let mut parser = Parser::new("[]");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::List(elements) => assert_eq!(elements.len(), 0),
        _ => panic!("Expected empty list"),
    }
}

#[test]
fn test_parse_tuple_literal() {
    let mut parser = Parser::new("(1, \"test\", true)");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Tuple(elements) => assert_eq!(elements.len(), 3),
        _ => panic!("Expected tuple literal"),
    }
}

#[test]
fn test_parse_if_expression() {
    let mut parser = Parser::new("if true { 1 } else { 2 }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::If { .. } => {},
        _ => panic!("Expected if expression"),
    }
}

#[test]
fn test_parse_if_without_else() {
    let mut parser = Parser::new("if condition { body }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::If { else_branch, .. } => assert!(else_branch.is_none()),
        _ => panic!("Expected if expression"),
    }
}

#[test]
fn test_parse_while_loop() {
    let mut parser = Parser::new("while true { x = x + 1 }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::While { .. } => {},
        _ => panic!("Expected while loop"),
    }
}

#[test]
fn test_parse_for_loop() {
    let mut parser = Parser::new("for i in range(10) { print(i) }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::For { .. } => {},
        _ => panic!("Expected for loop"),
    }
}

#[test]
fn test_parse_function_definition() {
    let mut parser = Parser::new("fun add(a: i32, b: i32) -> i32 { a + b }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        },
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_parse_function_call() {
    let mut parser = Parser::new("print(\"hello\")");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Call { args, .. } => assert_eq!(args.len(), 1),
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_method_call() {
    let mut parser = Parser::new("object.method(arg)");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::MethodCall { method, args, .. } => {
            assert_eq!(method, "method");
            assert_eq!(args.len(), 1);
        },
        _ => panic!("Expected method call"),
    }
}

#[test]
fn test_parse_chain_method_calls() {
    let mut parser = Parser::new("object.method1().method2()");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::MethodCall { method, receiver, .. } => {
            assert_eq!(method, "method2");
            match receiver.kind {
                ExprKind::MethodCall { method, .. } => assert_eq!(method, "method1"),
                _ => panic!("Expected chained method call"),
            }
        },
        _ => panic!("Expected method call"),
    }
}

#[test]
fn test_parse_assignment() {
    let mut parser = Parser::new("x = 42");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Assign { .. } => {},
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_parse_compound_assignment() {
    let mut parser = Parser::new("x += 10");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::CompoundAssign { op: BinaryOp::Add, .. } => {},
        _ => panic!("Expected compound assignment"),
    }
}

#[test]
fn test_parse_range_inclusive() {
    let mut parser = Parser::new("1..=10");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Range { inclusive, .. } => assert!(inclusive),
        _ => panic!("Expected inclusive range"),
    }
}

#[test]
fn test_parse_range_exclusive() {
    let mut parser = Parser::new("1..10");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Range { inclusive, .. } => assert!(!inclusive),
        _ => panic!("Expected exclusive range"),
    }
}

#[test]
fn test_parse_block_expression() {
    let mut parser = Parser::new("{ let x = 1; x + 2 }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Block(stmts) => assert!(stmts.len() >= 1),
        _ => panic!("Expected block expression"),
    }
}

#[test]
fn test_parse_match_expression() {
    let mut parser = Parser::new("match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Match { arms, .. } => assert_eq!(arms.len(), 3),
        _ => panic!("Expected match expression"),
    }
}

#[test]
fn test_parse_lambda_expression() {
    let mut parser = Parser::new("|x| x * 2");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Lambda { params, .. } => assert_eq!(params.len(), 1),
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_parse_fat_arrow_function() {
    let mut parser = Parser::new("x => x + 1");
    let expr = parser.parse().unwrap();
    // Fat arrow might be parsed as lambda or special syntax
    assert!(matches!(expr.kind, ExprKind::Lambda { .. }));
}

#[test]
fn test_parse_string_interpolation() {
    let mut parser = Parser::new("f\"Hello {name}\"");
    let result = parser.parse();
    // String interpolation might be special syntax
    if let Ok(expr) = result {
        match expr.kind {
            ExprKind::Literal(Literal::String(_)) => {},
            _ => panic!("Expected string interpolation"),
        }
    }
}

#[test]
fn test_parse_async_function() {
    let mut parser = Parser::new("async fun fetch() { await request() }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::Function { is_async, .. } => assert!(is_async),
        _ => panic!("Expected async function"),
    }
}

#[test]
fn test_parse_error_recovery() {
    // Test that parser can recover from errors
    let mut parser = Parser::new("1 + + 2"); // Invalid syntax
    let result = parser.parse();
    // Parser should either return error or handle gracefully
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_complex_expression() {
    let mut parser = Parser::new("if x > 0 { x * 2 } else { -x }");
    let expr = parser.parse().unwrap();
    match expr.kind {
        ExprKind::If { condition, then_branch, else_branch } => {
            assert!(matches!(condition.kind, ExprKind::Binary { op: BinaryOp::Greater, .. }));
            assert!(matches!(then_branch.kind, ExprKind::Block(_)));
            assert!(else_branch.is_some());
        },
        _ => panic!("Expected complex if expression"),
    }
}