// TRANSPILER-136: Test pub fun preservation (not inlining)
// RED Phase: Write failing tests first

use ruchy::backend::transpiler::inline_expander::inline_small_functions;
use ruchy::frontend::ast::{Expr, ExprKind, Param, Pattern, Span, Type, TypeKind};

#[test]
fn test_pub_fun_not_inlined_simple() {
    // Test Case 1: Simple pub fun should NOT be inlined
    // Input: pub fun add(x: i32, y: i32) -> i32 { x + y }

    let pub_func = Expr::new(
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
            return_type: Some(Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            }),
            body: Box::new(Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        Span::default(),
                    )),
                    op: ruchy::frontend::ast::BinaryOp::Add,
                    right: Box::new(Expr::new(
                        ExprKind::Identifier("y".to_string()),
                        Span::default(),
                    )),
                },
                Span::default(),
            )),
            is_async: false,
            is_pub: true, // PUBLIC function
        },
        Span::default(),
    );

    let block = Expr::new(ExprKind::Block(vec![pub_func]), Span::default());

    let (result, inlined_set) = inline_small_functions(block);

    // ASSERTION 1: Public function should NOT be in inlined set
    assert!(
        !inlined_set.contains("add"),
        "BUG #136: pub fun 'add' was marked for inlining but should NOT be"
    );

    // ASSERTION 2: Public function definition should still exist in result
    if let ExprKind::Block(exprs) = result.kind {
        let has_add_func = exprs.iter().any(
            |e| matches!(&e.kind, ExprKind::Function { name, is_pub: true, .. } if name == "add"),
        );
        assert!(
            has_add_func,
            "BUG #136: pub fun 'add' definition missing from output"
        );
    } else {
        panic!("Expected Block expression");
    }
}

#[test]
fn test_private_fun_still_inlined() {
    // Test Case 2: Private fun should STILL be inlined (existing behavior)
    // Input: fun helper(x: i32) -> i32 { x + 1 }

    let private_func = Expr::new(
        ExprKind::Function {
            name: "helper".to_string(),
            type_params: vec![],
            params: vec![Param {
                pattern: Pattern::Identifier("x".to_string()),
                ty: Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: Some(Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            }),
            body: Box::new(Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        Span::default(),
                    )),
                    op: ruchy::frontend::ast::BinaryOp::Add,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(1, None)),
                        Span::default(),
                    )),
                },
                Span::default(),
            )),
            is_async: false,
            is_pub: false, // PRIVATE function
        },
        Span::default(),
    );

    let call = Expr::new(
        ExprKind::Call {
            func: Box::new(Expr::new(
                ExprKind::Identifier("helper".to_string()),
                Span::default(),
            )),
            args: vec![Expr::new(
                ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(5, None)),
                Span::default(),
            )],
        },
        Span::default(),
    );

    let block = Expr::new(ExprKind::Block(vec![private_func, call]), Span::default());

    let (result, inlined_set) = inline_small_functions(block);

    // ASSERTION: Private function SHOULD be inlined (existing behavior preserved)
    assert!(
        inlined_set.contains("helper"),
        "Private fun 'helper' should be marked for inlining"
    );

    // Call should be replaced with inlined body
    if let ExprKind::Block(exprs) = result.kind {
        // Second expr should be inlined (x + 1 with x=5)
        assert_eq!(exprs.len(), 2, "Expected function def + inlined call");
    } else {
        panic!("Expected Block expression");
    }
}

#[test]
fn test_pub_fun_library_crate() {
    // Test Case 3: Library crate with only pub fun (no main)
    // This is the actual use case from ruchy-lambda runtime-pure

    let get_endpoint = Expr::new(
        ExprKind::Function {
            name: "get_endpoint".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: Some(Type {
                kind: TypeKind::Named("String".to_string()),
                span: Span::default(),
            }),
            body: Box::new(Expr::new(
                ExprKind::Literal(ruchy::frontend::ast::Literal::String(
                    "127.0.0.1:9001".to_string(),
                )),
                Span::default(),
            )),
            is_async: false,
            is_pub: true,
        },
        Span::default(),
    );

    let next_event = Expr::new(
        ExprKind::Function {
            name: "next_event".to_string(),
            type_params: vec![],
            params: vec![Param {
                pattern: Pattern::Identifier("endpoint".to_string()),
                ty: Type {
                    kind: TypeKind::Named("&str".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: Some(Type {
                kind: TypeKind::Named("String".to_string()),
                span: Span::default(),
            }),
            body: Box::new(Expr::new(
                ExprKind::Literal(ruchy::frontend::ast::Literal::String(
                    "test-response".to_string(),
                )),
                Span::default(),
            )),
            is_async: false,
            is_pub: true,
        },
        Span::default(),
    );

    let block = Expr::new(
        ExprKind::Block(vec![get_endpoint, next_event]),
        Span::default(),
    );

    let (result, inlined_set) = inline_small_functions(block);

    // ASSERTION 1: Neither public function should be inlined
    assert!(
        !inlined_set.contains("get_endpoint"),
        "BUG #136: pub fun 'get_endpoint' should NOT be inlined"
    );
    assert!(
        !inlined_set.contains("next_event"),
        "BUG #136: pub fun 'next_event' should NOT be inlined"
    );

    // ASSERTION 2: Both function definitions should exist in output
    if let ExprKind::Block(exprs) = result.kind {
        assert_eq!(
            exprs.len(),
            2,
            "Both pub fun definitions should be preserved"
        );

        let has_get_endpoint = exprs
            .iter()
            .any(|e| matches!(&e.kind, ExprKind::Function { name, is_pub: true, .. } if name == "get_endpoint"));
        let has_next_event = exprs
            .iter()
            .any(|e| matches!(&e.kind, ExprKind::Function { name, is_pub: true, .. } if name == "next_event"));

        assert!(has_get_endpoint, "BUG #136: pub fun 'get_endpoint' missing");
        assert!(has_next_event, "BUG #136: pub fun 'next_event' missing");
    } else {
        panic!("Expected Block expression");
    }
}
