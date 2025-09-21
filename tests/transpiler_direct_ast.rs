//! Simple transpiler tests with direct AST construction
//! Demonstrates bypassing parser limitations

#![allow(clippy::unwrap_used)]

use ruchy::frontend::ast::{Expr, ExprKind, Literal, MatchArm, Pattern, Span};
use ruchy::Transpiler;

/// Test simple match expression
#[test]
fn test_direct_match_construction() {
    let mut transpiler = Transpiler::new();

    // Create: match x { 1 => "one", _ => "other" }
    let match_expr = Expr {
        kind: ExprKind::Match {
            expr: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::Integer(1)),
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::String("one".to_string())),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::String("other".to_string())),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
    };

    let result = transpiler.transpile(&match_expr).unwrap();
    let code = result.to_string();

    assert!(code.contains("match"));
    assert!(code.contains("one"));
    assert!(code.contains("other"));
}

/// Test or-pattern construction
#[test]
fn test_or_pattern_direct() {
    let mut transpiler = Transpiler::new();

    // Create: match x { 1 | 2 => "small", _ => "large" }
    let match_expr = Expr {
        kind: ExprKind::Match {
            expr: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Or(vec![
                        Pattern::Literal(Literal::Integer(1)),
                        Pattern::Literal(Literal::Integer(2)),
                    ]),
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::String("small".to_string())),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::String("large".to_string())),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
    };

    // This tests or-patterns which parser doesn't support
    let result = transpiler.transpile(&match_expr).unwrap();
    let code = result.to_string();

    assert!(code.contains("small"));
    assert!(code.contains("large"));
}

/// Test Result pattern construction
#[test]
fn test_result_pattern_direct() {
    let mut transpiler = Transpiler::new();

    // Create: match result { Ok(x) => x, Err(_) => 0 }
    let match_expr = Expr {
        kind: ExprKind::Match {
            expr: Box::new(Expr {
                kind: ExprKind::Identifier("result".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Ok(Box::new(Pattern::Identifier("x".to_string()))),
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Identifier("x".to_string()),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Err(Box::new(Pattern::Wildcard)),
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::Integer(0)),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
    };

    let result = transpiler.transpile(&match_expr).unwrap();
    let code = result.to_string();

    assert!(code.contains("Ok"));
    assert!(code.contains("Err"));
}
