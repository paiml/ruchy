#![cfg(test)]
#![allow(warnings)]
//! Tests for control flow features in the interpreter

use ruchy::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern, Span};
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

/// Helper function to create test expressions
fn create_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::new(0, 10))
}

/// Test for loop with array iteration
#[test]
fn test_for_loop_array() {
    let mut interpreter = Interpreter::new();

    // for i in [1, 2, 3] { i * 2 }
    let for_expr = create_expr(ExprKind::For {
        var: "i".to_string(),
        pattern: None,
        iter: Box::new(create_expr(ExprKind::List(vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            create_expr(ExprKind::Literal(Literal::Integer(2))),
            create_expr(ExprKind::Literal(Literal::Integer(3))),
        ]))),
        body: Box::new(create_expr(ExprKind::Binary {
            left: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
            op: BinaryOp::Multiply,
            right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(2)))),
        })),
    });

    // The loop returns the last value
    let result = interpreter.eval_expr(&for_expr).unwrap();
    assert_eq!(result, Value::Integer(6)); // 3 * 2
}

/// Test for loop with range
#[test]
fn test_for_loop_range() {
    let mut interpreter = Interpreter::new();

    // for i in 0..3 { i }
    let for_expr = create_expr(ExprKind::For {
        var: "i".to_string(),
        pattern: None,
        iter: Box::new(create_expr(ExprKind::Range {
            start: Box::new(create_expr(ExprKind::Literal(Literal::Integer(0)))),
            end: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
            inclusive: false,
        })),
        body: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
    });

    let result = interpreter.eval_expr(&for_expr).unwrap();
    assert_eq!(result, Value::Integer(2)); // Last value is 2
}

/// Test while loop
#[test]
fn test_while_loop() {
    let mut interpreter = Interpreter::new();

    // let mut i = 0; while i < 3 { i = i + 1; i }
    let while_expr = create_expr(ExprKind::Let {
        name: "i".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(0)))),
        body: Box::new(create_expr(ExprKind::While {
            condition: Box::new(create_expr(ExprKind::Binary {
                left: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
                op: BinaryOp::Less,
                right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
            })),
            body: Box::new(create_expr(ExprKind::Block(vec![
                create_expr(ExprKind::Assign {
                    target: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
                    value: Box::new(create_expr(ExprKind::Binary {
                        left: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
                        op: BinaryOp::Add,
                        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(1)))),
                    })),
                }),
                create_expr(ExprKind::Identifier("i".to_string())),
            ]))),
        })),
        is_mutable: true,
    });

    let result = interpreter.eval_expr(&while_expr).unwrap();
    assert_eq!(result, Value::Integer(3));
}

/// Test match expression with literals
#[test]
fn test_match_literals() {
    let mut interpreter = Interpreter::new();

    // match 2 { 1 => "one", 2 => "two", _ => "other" }
    let match_expr = create_expr(ExprKind::Match {
        expr: Box::new(create_expr(ExprKind::Literal(Literal::Integer(2)))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1)),
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "one".to_string(),
                )))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(2)),
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "two".to_string(),
                )))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "other".to_string(),
                )))),
                span: Span::new(0, 10),
            },
        ],
    });

    let result = interpreter.eval_expr(&match_expr).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "two");
    } else {
        panic!("Expected string value");
    }
}

/// Test match with range patterns
#[test]
fn test_match_range() {
    let mut interpreter = Interpreter::new();

    // match 5 { 1..3 => "low", 3..7 => "mid", _ => "high" }
    let match_expr = create_expr(ExprKind::Match {
        expr: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Range {
                    start: Box::new(Pattern::Literal(Literal::Integer(1))),
                    end: Box::new(Pattern::Literal(Literal::Integer(3))),
                    inclusive: false,
                },
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "low".to_string(),
                )))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Range {
                    start: Box::new(Pattern::Literal(Literal::Integer(3))),
                    end: Box::new(Pattern::Literal(Literal::Integer(7))),
                    inclusive: false,
                },
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "mid".to_string(),
                )))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "high".to_string(),
                )))),
                span: Span::new(0, 10),
            },
        ],
    });

    let result = interpreter.eval_expr(&match_expr).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "mid");
    } else {
        panic!("Expected string value");
    }
}

/// Test assignment
#[test]
fn test_assignment() {
    let mut interpreter = Interpreter::new();

    // let x = 5; x = 10; x
    let assign_expr = create_expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        body: Box::new(create_expr(ExprKind::Block(vec![
            create_expr(ExprKind::Assign {
                target: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
                value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
            }),
            create_expr(ExprKind::Identifier("x".to_string())),
        ]))),
        is_mutable: true,
    });

    let result = interpreter.eval_expr(&assign_expr).unwrap();
    assert_eq!(result, Value::Integer(10));
}

/// Test compound assignment
#[test]
fn test_compound_assignment() {
    let mut interpreter = Interpreter::new();

    // let x = 5; x += 3; x
    let compound_expr = create_expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        body: Box::new(create_expr(ExprKind::Block(vec![
            create_expr(ExprKind::CompoundAssign {
                target: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
                op: BinaryOp::Add,
                value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
            }),
            create_expr(ExprKind::Identifier("x".to_string())),
        ]))),
        is_mutable: true,
    });

    let result = interpreter.eval_expr(&compound_expr).unwrap();
    assert_eq!(result, Value::Integer(8));
}

/// Test break in for loop
#[test]
fn test_for_loop_break() {
    let mut interpreter = Interpreter::new();

    // Check collecting values until break
    // for i in [1, 2, 3, 4, 5] { if i == 3 { break } else { i } }
    let for_expr = create_expr(ExprKind::For {
        var: "i".to_string(),
        pattern: None,
        iter: Box::new(create_expr(ExprKind::List(vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            create_expr(ExprKind::Literal(Literal::Integer(2))),
            create_expr(ExprKind::Literal(Literal::Integer(3))),
            create_expr(ExprKind::Literal(Literal::Integer(4))),
            create_expr(ExprKind::Literal(Literal::Integer(5))),
        ]))),
        body: Box::new(create_expr(ExprKind::If {
            condition: Box::new(create_expr(ExprKind::Binary {
                left: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
                op: BinaryOp::Equal,
                right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
            })),
            then_branch: Box::new(create_expr(ExprKind::Break { label: None })),
            else_branch: Some(Box::new(create_expr(ExprKind::Identifier("i".to_string())))),
        })),
    });

    // Should return the last value before break (2)
    let result = interpreter.eval_expr(&for_expr).unwrap();
    assert_eq!(result, Value::Integer(2));
}

/// Test continue in for loop
#[test]
fn test_for_loop_continue() {
    let mut interpreter = Interpreter::new();

    // Sum only even numbers
    // let sum = 0; for i in [1, 2, 3, 4] { if i % 2 == 1 { continue }; sum = sum + i }; sum
    let continue_expr = create_expr(ExprKind::Let {
        name: "sum".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(0)))),
        body: Box::new(create_expr(ExprKind::Block(vec![
            create_expr(ExprKind::For {
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(create_expr(ExprKind::List(vec![
                    create_expr(ExprKind::Literal(Literal::Integer(1))),
                    create_expr(ExprKind::Literal(Literal::Integer(2))),
                    create_expr(ExprKind::Literal(Literal::Integer(3))),
                    create_expr(ExprKind::Literal(Literal::Integer(4))),
                ]))),
                body: Box::new(create_expr(ExprKind::Block(vec![
                    create_expr(ExprKind::If {
                        condition: Box::new(create_expr(ExprKind::Binary {
                            left: Box::new(create_expr(ExprKind::Binary {
                                left: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
                                op: BinaryOp::Modulo,
                                right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(
                                    2,
                                )))),
                            })),
                            op: BinaryOp::Equal,
                            right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(1)))),
                        })),
                        then_branch: Box::new(create_expr(ExprKind::Continue { label: None })),
                        else_branch: None,
                    }),
                    create_expr(ExprKind::Assign {
                        target: Box::new(create_expr(ExprKind::Identifier("sum".to_string()))),
                        value: Box::new(create_expr(ExprKind::Binary {
                            left: Box::new(create_expr(ExprKind::Identifier("sum".to_string()))),
                            op: BinaryOp::Add,
                            right: Box::new(create_expr(ExprKind::Identifier("i".to_string()))),
                        })),
                    }),
                ]))),
            }),
            create_expr(ExprKind::Identifier("sum".to_string())),
        ]))),
        is_mutable: true,
    });

    let result = interpreter.eval_expr(&continue_expr).unwrap();
    assert_eq!(result, Value::Integer(6)); // 2 + 4
}

/// Test pattern matching with tuples
#[test]
fn test_match_tuple_pattern() {
    let mut interpreter = Interpreter::new();

    // match (1, 2) { (1, 2) => "match", _ => "no match" }
    let match_expr = create_expr(ExprKind::Match {
        expr: Box::new(create_expr(ExprKind::Tuple(vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            create_expr(ExprKind::Literal(Literal::Integer(2))),
        ]))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Integer(1)),
                    Pattern::Literal(Literal::Integer(2)),
                ]),
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "match".to_string(),
                )))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String(
                    "no match".to_string(),
                )))),
                span: Span::new(0, 10),
            },
        ],
    });

    let result = interpreter.eval_expr(&match_expr).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "match");
    } else {
        panic!("Expected string value");
    }
}
