//! Tests for the transpiler module
//! Focus on testing code generation functionality

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::*;

fn create_test_span() -> Span {
    Span::new(0, 1)
}

fn create_literal(lit: Literal) -> Expr {
    Expr {
        kind: ExprKind::Literal(lit),
        span: create_test_span(),
        attributes: vec![],
    }
}

fn create_identifier(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: create_test_span(),
        attributes: vec![],
    }
}

fn create_binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        span: create_test_span(),
        attributes: vec![],
    }
}

fn create_unary(op: UnaryOp, operand: Expr) -> Expr {
    Expr {
        kind: ExprKind::Unary {
            op,
            operand: Box::new(operand),
        },
        span: create_test_span(),
        attributes: vec![],
    }
}

#[test]
fn test_transpile_integer_literal() {
    let transpiler = Transpiler::new();
    let expr = create_literal(Literal::Integer(42));

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.to_string().contains("42"));
}

#[test]
fn test_transpile_float_literal() {
    let transpiler = Transpiler::new();
    let expr = create_literal(Literal::Float(3.14));

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());

    let code = result.unwrap();
    let code_str = code.to_string();
    assert!(code_str.contains("3.14") || code_str.contains("3_14"));
}

#[test]
fn test_transpile_bool_literals() {
    let transpiler = Transpiler::new();

    // Test true
    let true_expr = create_literal(Literal::Bool(true));
    let result = transpiler.transpile(&true_expr);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("true"));

    // Test false
    let false_expr = create_literal(Literal::Bool(false));
    let result = transpiler.transpile(&false_expr);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("false"));
}

#[test]
fn test_transpile_string_literal() {
    let transpiler = Transpiler::new();
    let expr = create_literal(Literal::String("hello".to_string()));

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.to_string().contains("hello"));
}

#[test]
fn test_transpile_identifier() {
    let transpiler = Transpiler::new();
    let expr = create_identifier("my_variable");

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.to_string().contains("my_variable"));
}

#[test]
fn test_transpile_addition() {
    let transpiler = Transpiler::new();
    let left = create_literal(Literal::Integer(1));
    let right = create_literal(Literal::Integer(2));
    let expr = create_binary(left, BinaryOp::Add, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());

    let code = result.unwrap().to_string();
    assert!(code.contains("1") || code.contains("2"));
}

#[test]
fn test_transpile_subtraction() {
    let transpiler = Transpiler::new();
    let left = create_literal(Literal::Integer(10));
    let right = create_literal(Literal::Integer(5));
    let expr = create_binary(left, BinaryOp::Subtract, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_multiplication() {
    let transpiler = Transpiler::new();
    let left = create_literal(Literal::Integer(3));
    let right = create_literal(Literal::Integer(4));
    let expr = create_binary(left, BinaryOp::Multiply, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_division() {
    let transpiler = Transpiler::new();
    let left = create_literal(Literal::Integer(10));
    let right = create_literal(Literal::Integer(2));
    let expr = create_binary(left, BinaryOp::Divide, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_comparison() {
    let transpiler = Transpiler::new();

    // Test less than
    let left = create_literal(Literal::Integer(1));
    let right = create_literal(Literal::Integer(2));
    let expr = create_binary(left, BinaryOp::Less, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_equality() {
    let transpiler = Transpiler::new();

    let left = create_literal(Literal::Integer(42));
    let right = create_literal(Literal::Integer(42));
    let expr = create_binary(left, BinaryOp::Equal, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_logical_and() {
    let transpiler = Transpiler::new();

    let left = create_literal(Literal::Bool(true));
    let right = create_literal(Literal::Bool(false));
    let expr = create_binary(left, BinaryOp::And, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_logical_or() {
    let transpiler = Transpiler::new();

    let left = create_literal(Literal::Bool(false));
    let right = create_literal(Literal::Bool(true));
    let expr = create_binary(left, BinaryOp::Or, right);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_unary_negation() {
    let transpiler = Transpiler::new();
    let operand = create_literal(Literal::Integer(42));
    let expr = create_unary(UnaryOp::Negate, operand);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_unary_not() {
    let transpiler = Transpiler::new();
    let operand = create_literal(Literal::Bool(true));
    let expr = create_unary(UnaryOp::Not, operand);

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_block() {
    let transpiler = Transpiler::new();
    let statements = vec![
        create_literal(Literal::Integer(1)),
        create_literal(Literal::Integer(2)),
    ];
    let expr = Expr {
        kind: ExprKind::Block(statements),
        span: create_test_span(),
        attributes: vec![],
    };

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_if_expression() {
    let transpiler = Transpiler::new();
    let condition = create_literal(Literal::Bool(true));
    let then_branch = create_literal(Literal::Integer(1));
    let else_branch = Some(Box::new(create_literal(Literal::Integer(2))));

    let expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        },
        span: create_test_span(),
        attributes: vec![],
    };

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_transpile_integer_never_panics(n in i64::MIN..i64::MAX) {
            let transpiler = Transpiler::new();
            let expr = create_literal(Literal::Integer(n));

            // Should never panic
            let _ = transpiler.transpile(&expr);
        }

        #[test]
        fn prop_transpile_float_never_panics(f in any::<f64>()) {
            let transpiler = Transpiler::new();
            let expr = create_literal(Literal::Float(f));

            // Should never panic
            let _ = transpiler.transpile(&expr);
        }

        #[test]
        fn prop_transpile_string_never_panics(s in ".*") {
            let transpiler = Transpiler::new();
            let expr = create_literal(Literal::String(s));

            // Should never panic
            let _ = transpiler.transpile(&expr);
        }
    }
}
