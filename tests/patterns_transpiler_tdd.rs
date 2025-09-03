//! Comprehensive TDD test suite for patterns.rs transpiler module
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every pattern type must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Pattern, MatchArm, Span, StructPatternField, BinaryOp};

/// Helper function to create a test transpiler
fn create_test_transpiler() -> Transpiler {
    Transpiler::new()
}

/// Helper to create a simple identifier expression
fn create_identifier(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span::new(0, name.len()),
        attributes: vec![],
    }
}

/// Helper to create a literal integer expression
fn create_integer(value: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(value)),
        span: Span::new(0, 0),
        attributes: vec![],
    }
}

/// Helper to create a literal string expression
fn create_string(value: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(value.to_string())),
        span: Span::new(0, value.len()),
        attributes: vec![],
    }
}

// ==================== MATCH EXPRESSION TESTS ====================

#[test]
fn test_transpile_match_simple() {
    let transpiler = create_test_transpiler();
    let expr = create_identifier("x");
    let arms = vec![
        MatchArm {
            pattern: Pattern::Literal(Literal::Integer(1)),
            guard: None,
            body: Box::new(create_string("one")),
            span: Span::new(0, 0),
        },
        MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(create_string("other")),
            span: Span::new(0, 0),
        },
    ];
    
    let result = transpiler.transpile_match(&expr, &arms);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("match"));
    assert!(code.contains("x"));
    assert!(code.contains("1") && code.contains("=>"));
    assert!(code.contains("_") && code.contains("=>"));
}

#[test]
fn test_transpile_match_with_guard() {
    let transpiler = create_test_transpiler();
    let expr = create_identifier("n");
    let arms = vec![
        MatchArm {
            pattern: Pattern::Identifier("x".to_string()),
            guard: Some(Box::new(Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::Greater,
                    left: Box::new(create_identifier("x")),
                    right: Box::new(create_integer(10)),
                },
                span: Span::new(0, 0),
                attributes: vec![],
            })),
            body: Box::new(create_string("big")),
            span: Span::new(0, 0),
        },
        MatchArm {
            pattern: Pattern::Identifier("x".to_string()),
            guard: None,
            body: Box::new(create_string("small")),
            span: Span::new(0, 0),
        },
    ];
    
    let result = transpiler.transpile_match(&expr, &arms);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("match"));
    assert!(code.contains("if"));
    assert!(code.contains(">"));
}

#[test]
fn test_transpile_match_multiple_arms() {
    let transpiler = create_test_transpiler();
    let expr = create_identifier("color");
    let arms = vec![
        MatchArm {
            pattern: Pattern::Literal(Literal::String("red".to_string())),
            guard: None,
            body: Box::new(create_integer(1)),
            span: Span::new(0, 0),
        },
        MatchArm {
            pattern: Pattern::Literal(Literal::String("green".to_string())),
            guard: None,
            body: Box::new(create_integer(2)),
            span: Span::new(0, 0),
        },
        MatchArm {
            pattern: Pattern::Literal(Literal::String("blue".to_string())),
            guard: None,
            body: Box::new(create_integer(3)),
            span: Span::new(0, 0),
        },
        MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(create_integer(0)),
            span: Span::new(0, 0),
        },
    ];
    
    let result = transpiler.transpile_match(&expr, &arms);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("red"));
    assert!(code.contains("green"));
    assert!(code.contains("blue"));
    assert!(code.contains("_"));
}

// ==================== PATTERN TESTS ====================

#[test]
fn test_transpile_pattern_wildcard() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Wildcard;
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert_eq!(code.trim(), "_");
}

#[test]
fn test_transpile_pattern_literal_integer() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Literal(Literal::Integer(42));
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_pattern_literal_string() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Literal(Literal::String("hello".to_string()));
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("hello"));
}

#[test]
fn test_transpile_pattern_literal_bool() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Literal(Literal::Bool(true));
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("true"));
}

#[test]
fn test_transpile_pattern_identifier() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Identifier("value".to_string());
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("value"));
}

#[test]
fn test_transpile_pattern_qualified_name() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::QualifiedName(vec![
        "Ordering".to_string(),
        "Less".to_string(),
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Ordering"));
    assert!(code.contains("Less"));
    assert!(code.contains("::"));
}

#[test]
fn test_transpile_pattern_tuple() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string()),
        Pattern::Wildcard,
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("("));
    assert!(code.contains("x"));
    assert!(code.contains("y"));
    assert!(code.contains("_"));
    assert!(code.contains(")"));
}

#[test]
fn test_transpile_pattern_list_empty() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::List(vec![]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("["));
    assert!(code.contains("]"));
}

#[test]
fn test_transpile_pattern_list_simple() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::List(vec![
        Pattern::Literal(Literal::Integer(1)),
        Pattern::Literal(Literal::Integer(2)),
        Pattern::Literal(Literal::Integer(3)),
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("["));
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
    assert!(code.contains("]"));
}

#[test]
fn test_transpile_pattern_list_with_rest() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::List(vec![
        Pattern::Identifier("first".to_string()),
        Pattern::Rest,
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("["));
    assert!(code.contains("first"));
    assert!(code.contains(".."));
    assert!(code.contains("]"));
}

#[test]
fn test_transpile_pattern_list_with_rest_named() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::List(vec![
        Pattern::Identifier("head".to_string()),
        Pattern::RestNamed("tail".to_string()),
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("["));
    assert!(code.contains("head"));
    assert!(code.contains(".."));
    assert!(code.contains("tail"));
    assert!(code.contains("]"));
}

#[test]
fn test_transpile_pattern_struct_empty() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![],
        has_rest: false,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Point"));
    assert!(code.contains("{"));
    assert!(code.contains("}"));
}

#[test]
fn test_transpile_pattern_struct_with_fields() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Struct {
        name: "Person".to_string(),
        fields: vec![
            StructPatternField {
                name: "name".to_string(),
                pattern: Some(Pattern::Identifier("n".to_string())),
            },
            StructPatternField {
                name: "age".to_string(),
                pattern: Some(Pattern::Identifier("a".to_string())),
            },
        ],
        has_rest: false,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Person"));
    assert!(code.contains("name"));
    assert!(code.contains("age"));
}

#[test]
fn test_transpile_pattern_struct_shorthand() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Struct {
        name: "Config".to_string(),
        fields: vec![
            StructPatternField {
                name: "host".to_string(),
                pattern: None, // Shorthand
            },
            StructPatternField {
                name: "port".to_string(),
                pattern: None, // Shorthand
            },
        ],
        has_rest: false,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Config"));
    assert!(code.contains("host"));
    assert!(code.contains("port"));
}

#[test]
fn test_transpile_pattern_struct_with_rest() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Struct {
        name: "User".to_string(),
        fields: vec![
            StructPatternField {
                name: "id".to_string(),
                pattern: Some(Pattern::Identifier("user_id".to_string())),
            },
        ],
        has_rest: true,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("User"));
    assert!(code.contains("id"));
    assert!(code.contains(".."));
}

#[test]
fn test_transpile_pattern_or() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Or(vec![
        Pattern::Literal(Literal::Integer(1)),
        Pattern::Literal(Literal::Integer(2)),
        Pattern::Literal(Literal::Integer(3)),
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
    assert!(code.contains("|"));
}

#[test]
fn test_transpile_pattern_range_exclusive() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1))),
        end: Box::new(Pattern::Literal(Literal::Integer(10))),
        inclusive: false,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("1"));
    assert!(code.contains(".."));
    assert!(code.contains("10"));
    assert!(!code.contains("..="));
}

#[test]
fn test_transpile_pattern_range_inclusive() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1))),
        end: Box::new(Pattern::Literal(Literal::Integer(10))),
        inclusive: true,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("1"));
    assert!(code.contains("..=") || code.contains(".. ="));
    assert!(code.contains("10"));
}

#[test]
fn test_transpile_pattern_rest() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Rest;
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains(".."));
}

#[test]
fn test_transpile_pattern_rest_named() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::RestNamed("rest".to_string());
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains(".."));
    assert!(code.contains("rest"));
}

#[test]
fn test_transpile_pattern_ok() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Ok(Box::new(Pattern::Identifier("value".to_string())));
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Ok"));
    assert!(code.contains("value"));
}

#[test]
fn test_transpile_pattern_err() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Err(Box::new(Pattern::Identifier("error".to_string())));
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Err"));
    assert!(code.contains("error"));
}

#[test]
fn test_transpile_pattern_some() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Some"));
    assert!(code.contains("x"));
}

#[test]
fn test_transpile_pattern_none() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::None;
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("None"));
}

// ==================== COMPLEX PATTERN TESTS ====================

#[test]
fn test_transpile_pattern_nested_tuple() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Tuple(vec![
        Pattern::Tuple(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
        ]),
        Pattern::Identifier("z".to_string()),
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("(("));
    assert!(code.contains("x"));
    assert!(code.contains("y"));
    assert!(code.contains("z"));
}

#[test]
fn test_transpile_pattern_nested_struct() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Struct {
        name: "Outer".to_string(),
        fields: vec![
            StructPatternField {
                name: "inner".to_string(),
                pattern: Some(Pattern::Struct {
                    name: "Inner".to_string(),
                    fields: vec![
                        StructPatternField {
                            name: "value".to_string(),
                            pattern: Some(Pattern::Identifier("v".to_string())),
                        },
                    ],
                    has_rest: false,
                }),
            },
        ],
        has_rest: false,
    };
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Outer"));
    assert!(code.contains("inner"));
    assert!(code.contains("Inner"));
    assert!(code.contains("value"));
}

#[test]
fn test_transpile_pattern_complex_or() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Or(vec![
        Pattern::Some(Box::new(Pattern::Identifier("x".to_string()))),
        Pattern::None,
        Pattern::Ok(Box::new(Pattern::Wildcard)),
    ]);
    
    let result = transpiler.transpile_pattern(&pattern);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("Some"));
    assert!(code.contains("None"));
    assert!(code.contains("Ok"));
    assert!(code.contains("|"));
}

// Run all tests with: cargo test patterns_transpiler_tdd --test patterns_transpiler_tdd