//! Basic TDD tests for backend/transpiler - achieving coverage
//! Focus on simple, core functionality that doesn't require complex AST construction

use ruchy::Transpiler;
use ruchy::{Expr, ExprKind, Literal};
use ruchy::frontend::ast::Span;

// Helper to create a simple literal expression
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

// ============================================================================
// Core Transpiler Tests
// ============================================================================

#[test]
fn test_transpiler_new() {
    let transpiler = Transpiler::new();
    assert!(!transpiler.in_async_context);
    assert!(transpiler.mutable_vars.is_empty());
    assert!(transpiler.function_signatures.is_empty());
}

#[test]
fn test_transpiler_default() {
    let transpiler = Transpiler::default();
    assert!(!transpiler.in_async_context);
}

// ============================================================================
// Literal Transpilation Tests
// ============================================================================

#[test]
fn test_transpile_integer() {
    let transpiler = Transpiler::new();
    let expr = make_literal(42);
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("42"));
}

#[test]
fn test_transpile_string() {
    let transpiler = Transpiler::new();
    let expr = make_string_literal("hello");
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("hello"));
}

#[test]
fn test_transpile_bool_true() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("true"));
}

#[test]
fn test_transpile_bool_false() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Bool(false)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("false"));
}

#[test]
fn test_transpile_float() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Float(3.14)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("3.14") || code.contains("3.14f64"));
}

#[test]
fn test_transpile_char() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Char('x')),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("'x'"));
}

#[test]
fn test_transpile_unit() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("()"));
}

// ============================================================================
// Identifier Tests
// ============================================================================

#[test]
fn test_transpile_identifier() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("my_var".to_string()),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("my_var"));
}

// ============================================================================
// Block and List Tests
// ============================================================================

#[test]
fn test_transpile_empty_block() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Block(vec![]),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_block_single_expr() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Block(vec![make_literal(42)]),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("42"));
}

#[test]
fn test_transpile_block_multiple_exprs() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Block(vec![
            make_literal(1),
            make_literal(2),
            make_literal(3),
        ]),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
}

#[test]
fn test_transpile_empty_list() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::List(vec![]),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("vec"));
}

#[test]
fn test_transpile_list_with_elements() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::List(vec![
            make_literal(10),
            make_literal(20),
        ]),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
    assert!(code.contains("10"));
    assert!(code.contains("20"));
}

// ============================================================================
// Program Generation Tests
// ============================================================================

#[test]
fn test_transpile_to_program() {
    let mut transpiler = Transpiler::new();
    let expr = make_literal(42);
    let result = transpiler.transpile_to_program(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("fn main"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_to_string() {
    let transpiler = Transpiler::new();
    let expr = make_literal(123);
    let result = transpiler.transpile_to_string(&expr).unwrap();
    // transpile_to_string just returns the expression, not wrapped in main
    assert!(result.contains("123"));
}

#[test]
fn test_transpile_minimal() {
    let transpiler = Transpiler::new();
    let expr = make_literal(999);
    let result = transpiler.transpile_minimal(&expr).unwrap();
    assert!(result.contains("999"));
}

// ============================================================================
// Mutability Analysis Tests  
// ============================================================================

#[test]
fn test_analyze_mutability_empty() {
    let mut transpiler = Transpiler::new();
    transpiler.analyze_mutability(&[]);
    assert!(transpiler.mutable_vars.is_empty());
}

#[test]
fn test_analyze_mutability_literal() {
    let mut transpiler = Transpiler::new();
    transpiler.analyze_mutability(&[make_literal(42)]);
    assert!(transpiler.mutable_vars.is_empty());
}

#[test]
fn test_analyze_mutability_assign() {
    let mut transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            value: Box::new(make_literal(10)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    transpiler.analyze_mutability(&[expr]);
    assert!(transpiler.mutable_vars.contains("x"));
}

// ============================================================================
// Async Context Tests
// ============================================================================

#[test]
fn test_async_context_default() {
    let transpiler = Transpiler::new();
    assert!(!transpiler.in_async_context);
}

#[test]
fn test_async_context_set() {
    let mut transpiler = Transpiler::new();
    transpiler.in_async_context = true;
    assert!(transpiler.in_async_context);
}

// ============================================================================
// Control Flow Tests
// ============================================================================

#[test]
fn test_transpile_if_without_else() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
            }),
            then_branch: Box::new(make_literal(1)),
            else_branch: None,
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("if"));
}

#[test]
fn test_transpile_if_with_else() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(false)),
                span: Span::default(),
                attributes: vec![],
            }),
            then_branch: Box::new(make_literal(1)),
            else_branch: Some(Box::new(make_literal(2))),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("if"));
    assert!(code.contains("else"));
}

#[test]
fn test_transpile_while() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::While {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
            }),
            body: Box::new(make_literal(1)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("while"));
}

// ============================================================================
// Binary Operations Tests
// ============================================================================

#[test]
fn test_transpile_add() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            op: ruchy::BinaryOp::Add,
            left: Box::new(make_literal(5)),
            right: Box::new(make_literal(3)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("5"));
    assert!(code.contains("3"));
    assert!(code.contains("+"));
}

#[test]
fn test_transpile_subtract() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            op: ruchy::BinaryOp::Subtract,
            left: Box::new(make_literal(10)),
            right: Box::new(make_literal(4)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("10"));
    assert!(code.contains("4"));
    assert!(code.contains("-"));
}

#[test]
fn test_transpile_multiply() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            op: ruchy::BinaryOp::Multiply,
            left: Box::new(make_literal(6)),
            right: Box::new(make_literal(7)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("6"));
    assert!(code.contains("7"));
    assert!(code.contains("*"));
}

// ============================================================================
// Unary Operations Tests
// ============================================================================

#[test]
fn test_transpile_negate() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Unary {
            op: ruchy::UnaryOp::Negate,
            operand: Box::new(make_literal(5)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("-") || code.contains("neg"));
    assert!(code.contains("5"));
}

#[test]
fn test_transpile_not() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Unary {
            op: ruchy::UnaryOp::Not,
            operand: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
            }),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("!"));
    assert!(code.contains("true"));
}

// ============================================================================
// Break and Continue Tests
// ============================================================================

#[test]
fn test_transpile_break() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Break { label: None },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("break"));
}

#[test]
fn test_transpile_break_with_label() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Break {
            label: Some("outer".to_string()),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("break"));
}

#[test]
fn test_transpile_continue() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Continue { label: None },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("continue"));
}

// ============================================================================
// Transpile Expression Test
// ============================================================================

#[test]
fn test_transpile_expr_public_method() {
    let transpiler = Transpiler::new();
    let expr = make_literal(100);
    let result = transpiler.transpile_expr(&expr).unwrap();
    assert!(result.to_string().contains("100"));
}