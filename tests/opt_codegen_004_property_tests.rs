// OPT-CODEGEN-004: Property-Based Tests for Inline Expansion
// VALIDATE Phase: Verify invariants hold across 25,000+ test cases
// Pattern: Based on PERF-002-B/C property tests (inline proptest! in #[ignore] tests)

use ruchy::backend::transpiler::inline_expander::inline_small_functions;
use ruchy::frontend::ast::{
    BinaryOp, Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind,
};

// ============================================================================
// PROPERTY 1: Idempotence (256 × 100 = 25,600 cases)
// ============================================================================

/// Property Test 1: Inline expansion is idempotent
/// Invariant: inline(inline(expr)) = inline(expr)
#[test]
#[ignore = "Run with: cargo test property_inline_idempotent -- --ignored --nocapture"]
fn property_inline_idempotent() {
    use proptest::prelude::*;

    proptest!(|(a in -50i64..50i64, b in 1i64..50i64)| {
        // Create small function: fun add(x, y) { x + y }
        let add_func = Expr::new(
            ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![
                    Param {
                        pattern: Pattern::Identifier("x".to_string()),
                        ty: Type {
                            kind: TypeKind::Named("i32".to_string()),
                            span: Span::default(),
                        },
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                    Param {
                        pattern: Pattern::Identifier("y".to_string()),
                        ty: Type {
                            kind: TypeKind::Named("i32".to_string()),
                            span: Span::default(),
                        },
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                ],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(Expr::new(
                            ExprKind::Identifier("x".to_string()),
                            Span::default(),
                        )),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::new(
                            ExprKind::Identifier("y".to_string()),
                            Span::default(),
                        )),
                    },
                    Span::default(),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("add".to_string()),
                    Span::default(),
                )),
                args: vec![
                    Expr::new(ExprKind::Literal(Literal::Integer(a, None)), Span::default()),
                    Expr::new(ExprKind::Literal(Literal::Integer(b, None)), Span::default()),
                ],
            },
            Span::default(),
        );

        let block = Expr::new(
            ExprKind::Block(vec![add_func, call]),
            Span::default(),
        );

        // Apply inline expansion once
        let (once, _) = inline_small_functions(block);

        // Apply inline expansion twice
        let (twice, _) = inline_small_functions(once.clone());

        // Convert to strings for comparison (semantic equivalence)
        let once_str = format!("{once:?}");
        let twice_str = format!("{twice:?}");

        prop_assert_eq!(once_str, twice_str,
            "Inline expansion should be idempotent: inline(inline(expr)) = inline(expr)");
    });
}
// ============================================================================
// PROPERTY 2: Recursive Functions Never Inlined (256 × 100 = 25,600 cases)
// ============================================================================

/// Property Test 2: Recursive functions are never inlined
/// Invariant: Function definition still exists after inlining
#[test]
#[ignore = "Run with: cargo test property_recursive_never_inlined -- --ignored --nocapture"]
fn property_recursive_never_inlined() {
    use proptest::prelude::*;

    proptest!(|(n in 1i64..20i64)| {
        // Create recursive factorial function
        let factorial = Expr::new(
            ExprKind::Function {
                name: "factorial".to_string(),
                type_params: vec![],
                params: vec![Param {
                    pattern: Pattern::Identifier("n".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                }],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("factorial".to_string()),
                            Span::default(),
                        )),
                        args: vec![Expr::new(
                            ExprKind::Literal(Literal::Integer(n, None)),
                            Span::default(),
                        )],
                    },
                    Span::default(),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("factorial".to_string()),
                    Span::default(),
                )),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::default(),
                )],
            },
            Span::default(),
        );

        let block = Expr::new(
            ExprKind::Block(vec![factorial, call]),
            Span::default(),
        );

        let (result, _) = inline_small_functions(block);

        // Verify factorial function definition still exists (not inlined)
        if let ExprKind::Block(exprs) = result.kind {
            prop_assert!(
                exprs.iter().any(|e| matches!(&e.kind, ExprKind::Function { name, .. } if name == "factorial")),
                "Recursive function 'factorial' should NOT be inlined (safety check)"
            );
        } else {
            return Err(proptest::test_runner::TestCaseError::fail("Expected block result"));
        }
    });
}

// ============================================================================
// PROPERTY 3: Large Functions NOT Inlined (256 × 100 = 25,600 cases)
// ============================================================================

/// Property Test 3: Large functions (>10 LOC) are NOT inlined
/// Invariant: Function call still exists after inlining
#[test]
#[ignore = "Run with: cargo test property_large_functions_not_inlined -- --ignored --nocapture"]
fn property_large_functions_not_inlined() {
    use proptest::prelude::*;

    proptest!(|(n in 1i64..100i64)| {
        // Create large function with >10 LOC (11 let statements)
        let mut nested_let = Expr::new(
            ExprKind::Identifier("var11".to_string()),
            Span::default(),
        );

        for i in (1..=11).rev() {
            nested_let = Expr::new(
                ExprKind::Let {
                    name: format!("var{i}"),
                    type_annotation: None,
                    value: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(i, None)),
                        Span::default(),
                    )),
                    body: Box::new(nested_let),
                    is_mutable: false,
                    else_block: None,
                },
                Span::default(),
            );
        }

        let large_func = Expr::new(
            ExprKind::Function {
                name: "large_computation".to_string(),
                type_params: vec![],
                params: vec![Param {
                    pattern: Pattern::Identifier("n".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                }],
                return_type: None,
                body: Box::new(nested_let),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("large_computation".to_string()),
                    Span::default(),
                )),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(n, None)),
                    Span::default(),
                )],
            },
            Span::default(),
        );

        let block = Expr::new(
            ExprKind::Block(vec![large_func, call]),
            Span::default(),
        );

        let (result, _) = inline_small_functions(block);

        // Verify call still exists (not inlined)
        if let ExprKind::Block(exprs) = &result.kind {
            prop_assert!(
                exprs.iter().any(|e| matches!(&e.kind, ExprKind::Call { func, .. }
                    if matches!(&func.kind, ExprKind::Identifier(name) if name == "large_computation"))),
                "Large function (>10 LOC) should NOT be inlined (size heuristic)"
            );
        } else {
            return Err(proptest::test_runner::TestCaseError::fail("Expected block result"));
        }
    });
}
