//! Comprehensive test suite for backend transpiler statements
//! Target: Increase coverage for src/backend/transpiler/statements.rs

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, Span, UnaryOp};

#[test]
fn test_transpile_simple_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 0));
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("42"));
}

#[test]
fn test_transpile_string_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::String("hello".to_string())),
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("hello"));
}

#[test]
fn test_transpile_boolean_literals() {
    let transpiler = Transpiler::new();

    let true_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
    let result = transpiler.transpile(&true_expr).unwrap();
    assert!(result.to_string().contains("true"));

    let false_expr = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::new(0, 0));
    let result = transpiler.transpile(&false_expr).unwrap();
    assert!(result.to_string().contains("false"));
}

#[test]
fn test_transpile_identifier() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(ExprKind::Identifier("my_var".to_string()), Span::new(0, 0));
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("my_var"));
}

#[test]
fn test_transpile_binary_addition() {
    let transpiler = Transpiler::new();
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
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains('1') && output.contains('2'));
}

#[test]
fn test_transpile_binary_operations() {
    let transpiler = Transpiler::new();
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Subtract,
        BinaryOp::Multiply,
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::Greater,
    ];

    for op in ops {
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10)),
                    Span::new(0, 0),
                )),
                op,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 0),
                )),
            },
            Span::new(0, 0),
        );
        let result = transpiler.transpile(&expr);
        assert!(result.is_ok(), "Failed to transpile {op:?}");
    }
}

#[test]
fn test_transpile_unary_operations() {
    let transpiler = Transpiler::new();

    let neg_expr = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(42)),
                Span::new(0, 0),
            )),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&neg_expr).unwrap();
    assert!(result.to_string().contains("42"));

    let not_expr = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                Span::new(0, 0),
            )),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&not_expr).unwrap();
    assert!(result.to_string().contains('!'));
}

#[test]
fn test_transpile_block_expression() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::Block(vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 0)),
            Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(0, 0)),
        ]),
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains('1') && output.contains('2'));
}

#[test]
fn test_transpile_if_expression() {
    let transpiler = Transpiler::new();
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
            else_branch: Some(Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2)),
                Span::new(0, 0),
            ))),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains("if"));
}

#[test]
fn test_transpile_let_binding() {
    let transpiler = Transpiler::new();
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
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains("let") && output.contains('x'));
}

#[test]
fn test_transpile_mutable_binding() {
    let transpiler = Transpiler::new();
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
            is_mutable: true,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains("mut"));
}

#[test]
fn test_transpile_function_call() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::Call {
            func: Box::new(Expr::new(
                ExprKind::Identifier("println".to_string()),
                Span::new(0, 0),
            )),
            args: vec![Expr::new(
                ExprKind::Literal(Literal::String("Hello".to_string())),
                Span::new(0, 0),
            )],
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains("println"));
}

#[test]
fn test_transpile_list_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::List(vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 0)),
            Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(0, 0)),
            Expr::new(ExprKind::Literal(Literal::Integer(3)), Span::new(0, 0)),
        ]),
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    // Lists are transpiled as vec![] or similar
    assert!(output.contains('1') && output.contains('2') && output.contains('3'));
}

#[test]
fn test_transpile_tuple_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::Tuple(vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 0)),
            Expr::new(
                ExprKind::Literal(Literal::String("test".to_string())),
                Span::new(0, 0),
            ),
        ]),
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains('(') && output.contains(')'));
}

#[test]
fn test_transpile_assignment() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::Assign {
            target: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                Span::new(0, 0),
            )),
            value: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(100)),
                Span::new(0, 0),
            )),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    let output = result.to_string();
    assert!(output.contains('x') && output.contains("100"));
}

#[test]
fn test_transpile_while_loop() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::While {
            condition: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                Span::new(0, 0),
            )),
            body: Box::new(Expr::new(ExprKind::Block(vec![]), Span::new(0, 0))),
            label: None,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("while"));
}

#[test]
fn test_transpile_for_loop() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::For {
            var: "i".to_string(),
            pattern: Some(Pattern::Identifier("i".to_string())),
            iter: Box::new(Expr::new(
                ExprKind::Identifier("range".to_string()),
                Span::new(0, 0),
            )),
            body: Box::new(Expr::new(ExprKind::Block(vec![]), Span::new(0, 0))),
            label: None,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("for"));
}

#[test]
fn test_transpile_return_statement() {
    let transpiler = Transpiler::new();
    let expr = Expr::new(
        ExprKind::Return {
            value: Some(Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(42)),
                Span::new(0, 0),
            ))),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&expr).unwrap();
    assert!(result.to_string().contains("return"));
}

#[test]
fn test_transpile_break_continue() {
    let transpiler = Transpiler::new();

    let break_expr = Expr::new(
        ExprKind::Break {
            label: None,
            value: None,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile(&break_expr).unwrap();
    assert!(result.to_string().contains("break"));

    let continue_expr = Expr::new(ExprKind::Continue { label: None }, Span::new(0, 0));
    let result = transpiler.transpile(&continue_expr).unwrap();
    assert!(result.to_string().contains("continue"));
}

#[test]
fn test_transpile_complex_expression() {
    let transpiler = Transpiler::new();
    // Create a complex nested expression: if (x > 5) { x * 2 } else { x + 1 }
    let expr = Expr::new(
        ExprKind::If {
            condition: Box::new(Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        Span::new(0, 0),
                    )),
                    op: BinaryOp::Greater,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(5)),
                        Span::new(0, 0),
                    )),
                },
                Span::new(0, 0),
            )),
            then_branch: Box::new(Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        Span::new(0, 0),
                    )),
                    op: BinaryOp::Multiply,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(2)),
                        Span::new(0, 0),
                    )),
                },
                Span::new(0, 0),
            )),
            else_branch: Some(Box::new(Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        Span::new(0, 0),
                    )),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(1)),
                        Span::new(0, 0),
                    )),
                },
                Span::new(0, 0),
            ))),
        },
        Span::new(0, 0),
    );

    let result = transpiler.transpile(&expr);
    assert!(result.is_ok(), "Failed to transpile complex expression");
    let output = result.unwrap().to_string();
    assert!(output.contains("if") && output.contains("else"));
}
