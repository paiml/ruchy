//! Comprehensive test suite for quality modules
//! Target: Increase coverage for src/quality/*.rs

use ruchy::quality::formatter::Formatter;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span, BinaryOp, Param, Pattern, Type, TypeKind};

#[test]
fn test_formatter_new() {
    let formatter = Formatter::new();
    assert!(formatter.format(&Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 0),
    )).is_ok());
}

#[test]
fn test_formatter_default() {
    let formatter = Formatter::default();
    assert!(formatter.format(&Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 0),
    )).is_ok());
}

#[test]
fn test_format_integer_literal() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(123)),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert_eq!(result, "123");
}

#[test]
fn test_format_float_literal() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Float(3.14)),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert_eq!(result, "3.14");
}

#[test]
fn test_format_string_literal() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::String("hello".to_string())),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert_eq!(result, "\"hello\"");
}

#[test]
fn test_format_bool_literals() {
    let formatter = Formatter::new();

    let true_expr = Expr::new(
        ExprKind::Literal(Literal::Bool(true)),
        Span::new(0, 0),
    );
    assert_eq!(formatter.format(&true_expr).unwrap(), "true");

    let false_expr = Expr::new(
        ExprKind::Literal(Literal::Bool(false)),
        Span::new(0, 0),
    );
    assert_eq!(formatter.format(&false_expr).unwrap(), "false");
}

#[test]
fn test_format_char_literal() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Char('a')),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert_eq!(result, "'a'");
}

#[test]
fn test_format_unit_literal() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Unit),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert_eq!(result, "()");
}

#[test]
fn test_format_identifier() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Identifier("my_variable".to_string()),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert_eq!(result, "my_variable");
}

#[test]
fn test_format_block_empty() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Block(vec![]),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("{") && result.contains("}"));
}

#[test]
fn test_format_block_with_expressions() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Block(vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1)),
                Span::new(0, 0),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2)),
                Span::new(0, 0),
            ),
        ]),
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("1") && result.contains("2"));
    assert!(result.contains("{") && result.contains("}"));
}

#[test]
fn test_formatter_binary_expression() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(1)),
                Span::new(0, 0),
            )),
            op: BinaryOp::Add,
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2)),
                Span::new(0, 0),
            )),
        },
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("1") && result.contains("2") && result.contains("+"));
}

#[test]
fn test_formatter_if_expression() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::If {
            condition: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                Span::new(0, 0),
            )),
            then_branch: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(1)),
                Span::new(0, 0),
            )),
            else_branch: None,
        },
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("if") && result.contains("true") && result.contains("1"));
}

#[test]
fn test_formatter_if_else_expression() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::If {
            condition: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(false)),
                Span::new(0, 0),
            )),
            then_branch: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(1)),
                Span::new(0, 0),
            )),
            else_branch: Some(Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2)),
                Span::new(0, 0),
            ))),
        },
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("if") && result.contains("else"));
    assert!(result.contains("1") && result.contains("2"));
}

#[test]
fn test_formatter_let_expression() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(42)),
                Span::new(0, 0),
            )),
            body: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                Span::new(0, 0),
            )),
            is_mutable: false,
        },
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("let") && result.contains("x") && result.contains("42"));
}

#[test]
fn test_formatter_function_simple() {
    let formatter = Formatter::new();
    let expr = Expr::new(
        ExprKind::Function {
            name: "add".to_string(),
            type_params: vec![],
            params: vec![
                Param {
                    pattern: Pattern::Identifier("a".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::new(0, 0),
                    },
                    span: Span::new(0, 0),
                    is_mutable: false,
                    default_value: None,
                },
                Param {
                    pattern: Pattern::Identifier("b".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::new(0, 0),
                    },
                    span: Span::new(0, 0),
                    is_mutable: false,
                    default_value: None,
                },
            ],
            return_type: Some(Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 0),
            }),
            body: Box::new(Expr::new(
                ExprKind::Block(vec![]),
                Span::new(0, 0),
            )),
            is_async: false,
            is_pub: false,
        },
        Span::new(0, 0),
    );
    let result = formatter.format(&expr).unwrap();
    assert!(result.contains("fun add"));
    assert!(result.contains("a: i32") && result.contains("b: i32"));
}

#[test]
fn test_formatter_fallback_for_unsupported() {
    let formatter = Formatter::new();
    // Test with a complex expression that might not be fully implemented
    let expr = Expr::new(
        ExprKind::Loop {
            body: Box::new(Expr::new(
                ExprKind::Block(vec![]),
                Span::new(0, 0),
            )),
        },
        Span::new(0, 0),
    );
    let result = formatter.format(&expr);
    // Should not panic, should return something
    assert!(result.is_ok());
}

// Test quality gates
use ruchy::quality::{QualityGates, QualityMetrics, QualityThresholds, QualityReport};

#[test]
fn test_quality_gates_new() {
    let gates = QualityGates::new();
    assert!(gates.get_metrics().test_coverage >= 0.0);
}

#[test]
fn test_quality_gates_with_thresholds() {
    let thresholds = QualityThresholds {
        min_test_coverage: 80.0,
        max_complexity: 10,
        max_satd: 0,
        max_clippy_warnings: 0,
        min_doc_coverage: 90.0,
    };
    let gates = QualityGates::with_thresholds(thresholds);
    let thresholds_ref = gates.get_thresholds();
    assert_eq!(thresholds_ref.min_test_coverage, 80.0);
}

#[test]
fn test_quality_metrics_check_pass() {
    let mut gates = QualityGates::new();
    let metrics = QualityMetrics {
        test_coverage: 85.0,
        cyclomatic_complexity: 5,
        cognitive_complexity: 5,
        satd_count: 0,
        clippy_warnings: 0,
        documentation_coverage: 95.0,
        unsafe_blocks: 0,
    };
    gates.update_metrics(metrics);

    match gates.check() {
        Ok(report) => match report {
            QualityReport::Pass => {}, // Expected
            _ => panic!("Expected quality check to pass"),
        },
        Err(_) => panic!("Expected quality check to pass"),
    }
}

#[test]
fn test_quality_metrics_check_fail() {
    let mut gates = QualityGates::new();
    let metrics = QualityMetrics {
        test_coverage: 30.0, // Below threshold
        cyclomatic_complexity: 15, // Above threshold
        cognitive_complexity: 15, // Above threshold
        satd_count: 10, // Above threshold
        clippy_warnings: 10, // Above threshold
        documentation_coverage: 20.0, // Below threshold
        unsafe_blocks: 5, // Above threshold
    };
    gates.update_metrics(metrics);

    match gates.check() {
        Err(report) => match report {
            QualityReport::Fail { violations: _ } => {}, // Expected
            _ => panic!("Expected quality check to fail"),
        },
        Ok(_) => panic!("Expected quality check to fail"),
    }
}