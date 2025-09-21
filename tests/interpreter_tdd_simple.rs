// EXTREME TDD: Simple interpreter tests that compile and run
// Following Toyota Way: Fix root causes, not symptoms

use ruchy::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, UnaryOp};
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::repl::Value;
use std::rc::Rc;

#[cfg(test)]
use proptest::prelude::*;

// Helper to create simple test expressions
fn make_literal(value: i64) -> Expr {
    Expr::new(
        ExprKind::Literal(Literal::Integer(value)),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    )
}

fn make_binary_op(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::new(
        ExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    )
}

#[test]
fn test_eval_integer_literal() {
    let mut interpreter = Interpreter::new();
    let expr = make_literal(42);
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_eval_addition() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(2), BinaryOp::Add, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_eval_subtraction() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(10), BinaryOp::Subtract, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn test_eval_multiplication() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(4), BinaryOp::Multiply, make_literal(5));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_eval_division() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(20), BinaryOp::Divide, make_literal(4));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_eval_modulo() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(10), BinaryOp::Modulo, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_eval_comparison_equal() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(5), BinaryOp::Equal, make_literal(5));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_comparison_not_equal() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(5), BinaryOp::NotEqual, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_comparison_less() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(3), BinaryOp::Less, make_literal(5));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_comparison_greater() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(5), BinaryOp::Greater, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_string_literal() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::String("hello".to_string())),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::String(Rc::new("hello".to_string())));
}

#[test]
fn test_eval_boolean_literal_true() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Bool(true)),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_boolean_literal_false() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Bool(false)),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_eval_float_literal() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Float(3.14)),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn test_eval_null_literal() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Null),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_eval_char_literal() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Char('a')),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::String(Rc::new("a".to_string())));
}

#[test]
fn test_eval_list_literal() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::List(vec![make_literal(1), make_literal(2), make_literal(3)]),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_eval_tuple_literal() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Tuple(vec![
            make_literal(1),
            Expr::new(
                ExprKind::Literal(Literal::String("hello".to_string())),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            ),
            Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            ),
        ]),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    match result {
        Value::Tuple(elements) => {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], Value::Integer(1));
            assert_eq!(elements[1], Value::String(Rc::new("hello".to_string())));
            assert_eq!(elements[2], Value::Bool(true));
        }
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_eval_block() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Block(vec![make_literal(1), make_literal(2), make_literal(3)]),
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(3)); // Block returns last value
}

#[test]
fn test_eval_if_true() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::If {
            condition: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            )),
            then_branch: Box::new(make_literal(1)),
            else_branch: Some(Box::new(make_literal(2))),
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_eval_if_false() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::If {
            condition: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(false)),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            )),
            then_branch: Box::new(make_literal(1)),
            else_branch: Some(Box::new(make_literal(2))),
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_eval_logical_and_true() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(
        Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
        BinaryOp::And,
        Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_logical_and_false() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(
        Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
        BinaryOp::And,
        Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_eval_logical_or_true() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(
        Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
        BinaryOp::Or,
        Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_logical_or_false() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(
        Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
        BinaryOp::Or,
        Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            ruchy::frontend::ast::Span { start: 0, end: 0 },
        ),
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_eval_bitwise_and() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(5), BinaryOp::BitwiseAnd, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_eval_bitwise_or() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(5), BinaryOp::BitwiseOr, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn test_eval_bitwise_xor() {
    let mut interpreter = Interpreter::new();
    let expr = make_binary_op(make_literal(5), BinaryOp::BitwiseXor, make_literal(3));
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(6));
}

// Unary operation tests
#[test]
fn test_eval_unary_not() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            )),
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_eval_unary_negate() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(make_literal(42)),
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(-42));
}

// Variable tests
#[test]
fn test_eval_let() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(make_literal(100)),
            body: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            )),
            is_mutable: false,
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    // Let expression returns the value of its body
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_eval_assign() {
    let mut interpreter = Interpreter::new();
    // Use Let with body to both define and use variable
    let expr = Expr::new(
        ExprKind::Let {
            name: "y".to_string(),
            type_annotation: None,
            value: Box::new(make_literal(0)),
            body: Box::new(Expr::new(
                ExprKind::Block(vec![
                    Expr::new(
                        ExprKind::Assign {
                            target: Box::new(Expr::new(
                                ExprKind::Identifier("y".to_string()),
                                ruchy::frontend::ast::Span { start: 0, end: 0 },
                            )),
                            value: Box::new(make_literal(42)),
                        },
                        ruchy::frontend::ast::Span { start: 0, end: 0 },
                    ),
                    Expr::new(
                        ExprKind::Identifier("y".to_string()),
                        ruchy::frontend::ast::Span { start: 0, end: 0 },
                    ),
                ]),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            )),
            is_mutable: true,
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(42));
}

// Range tests
#[test]
fn test_eval_range() {
    let mut interpreter = Interpreter::new();
    let expr = Expr::new(
        ExprKind::Range {
            start: Box::new(make_literal(1)),
            end: Box::new(make_literal(5)),
            inclusive: false,
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    match result {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            assert_eq!(*start, Value::Integer(1));
            assert_eq!(*end, Value::Integer(5));
            assert!(!inclusive);
        }
        _ => panic!("Expected range"),
    }
}

// While loop test
#[test]
fn test_eval_while() {
    let mut interpreter = Interpreter::new();
    // Use Let with body to scope the counter variable
    let expr = Expr::new(
        ExprKind::Let {
            name: "counter".to_string(),
            type_annotation: None,
            value: Box::new(make_literal(0)),
            body: Box::new(Expr::new(
                ExprKind::Block(vec![
                    Expr::new(
                        ExprKind::While {
                            condition: Box::new(make_binary_op(
                                Expr::new(
                                    ExprKind::Identifier("counter".to_string()),
                                    ruchy::frontend::ast::Span { start: 0, end: 0 },
                                ),
                                BinaryOp::Less,
                                make_literal(3),
                            )),
                            body: Box::new(Expr::new(
                                ExprKind::Assign {
                                    target: Box::new(Expr::new(
                                        ExprKind::Identifier("counter".to_string()),
                                        ruchy::frontend::ast::Span { start: 0, end: 0 },
                                    )),
                                    value: Box::new(make_binary_op(
                                        Expr::new(
                                            ExprKind::Identifier("counter".to_string()),
                                            ruchy::frontend::ast::Span { start: 0, end: 0 },
                                        ),
                                        BinaryOp::Add,
                                        make_literal(1),
                                    )),
                                },
                                ruchy::frontend::ast::Span { start: 0, end: 0 },
                            )),
                        },
                        ruchy::frontend::ast::Span { start: 0, end: 0 },
                    ),
                    // Return the final counter value
                    Expr::new(
                        ExprKind::Identifier("counter".to_string()),
                        ruchy::frontend::ast::Span { start: 0, end: 0 },
                    ),
                ]),
                ruchy::frontend::ast::Span { start: 0, end: 0 },
            )),
            is_mutable: true,
        },
        ruchy::frontend::ast::Span { start: 0, end: 0 },
    );
    let result = interpreter.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(3));
}

// Property-based tests with 10,000+ iterations
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property: Integer arithmetic operations preserve mathematical properties
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn test_addition_commutative(a: i32, b: i32) {
            // Use i32 to avoid overflow in tests
            let a = a as i64;
            let b = b as i64;

            // Skip values that would cause overflow
            if a.checked_add(b).is_none() {
                return Ok(());
            }

            let mut interpreter = Interpreter::new();

            // a + b
            let expr1 = make_binary_op(
                make_literal(a),
                BinaryOp::Add,
                make_literal(b)
            );

            // b + a
            let expr2 = make_binary_op(
                make_literal(b),
                BinaryOp::Add,
                make_literal(a)
            );

            let result1 = interpreter.eval_expr(&expr1);
            let result2 = interpreter.eval_expr(&expr2);

            match (result1, result2) {
                (Ok(Value::Integer(r1)), Ok(Value::Integer(r2))) => {
                    prop_assert_eq!(r1, r2, "Addition should be commutative");
                }
                _ => {}
            }
        }

        #[test]
        fn test_multiplication_commutative(a: i32, b: i32) {
            let mut interpreter = Interpreter::new();

            // Use i32 to avoid overflow
            let a = a as i64;
            let b = b as i64;

            // a * b
            let expr1 = make_binary_op(
                make_literal(a),
                BinaryOp::Multiply,
                make_literal(b)
            );

            // b * a
            let expr2 = make_binary_op(
                make_literal(b),
                BinaryOp::Multiply,
                make_literal(a)
            );

            let result1 = interpreter.eval_expr(&expr1);
            let result2 = interpreter.eval_expr(&expr2);

            match (result1, result2) {
                (Ok(Value::Integer(r1)), Ok(Value::Integer(r2))) => {
                    prop_assert_eq!(r1, r2, "Multiplication should be commutative");
                }
                _ => {}
            }
        }

        #[test]
        fn test_boolean_and_identity(a: bool) {
            let mut interpreter = Interpreter::new();

            // a && true == a
            let expr = make_binary_op(
                Expr::new(
                    ExprKind::Literal(Literal::Bool(a)),
                    ruchy::frontend::ast::Span { start: 0, end: 0 }
                ),
                BinaryOp::And,
                Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    ruchy::frontend::ast::Span { start: 0, end: 0 }
                )
            );

            let result = interpreter.eval_expr(&expr).unwrap();
            prop_assert_eq!(result, Value::Bool(a), "a && true should equal a");
        }

        #[test]
        fn test_boolean_or_identity(a: bool) {
            let mut interpreter = Interpreter::new();

            // a || false == a
            let expr = make_binary_op(
                Expr::new(
                    ExprKind::Literal(Literal::Bool(a)),
                    ruchy::frontend::ast::Span { start: 0, end: 0 }
                ),
                BinaryOp::Or,
                Expr::new(
                    ExprKind::Literal(Literal::Bool(false)),
                    ruchy::frontend::ast::Span { start: 0, end: 0 }
                )
            );

            let result = interpreter.eval_expr(&expr).unwrap();
            prop_assert_eq!(result, Value::Bool(a), "a || false should equal a");
        }

        #[test]
        fn test_double_negation(a: i32) {
            let mut interpreter = Interpreter::new();
            let a = a as i64;

            // -(-a) == a
            let expr = Expr::new(
                ExprKind::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(Expr::new(
                        ExprKind::Unary {
                            op: UnaryOp::Negate,
                            operand: Box::new(make_literal(a)),
                        },
                        ruchy::frontend::ast::Span { start: 0, end: 0 }
                    )),
                },
                ruchy::frontend::ast::Span { start: 0, end: 0 }
            );

            let result = interpreter.eval_expr(&expr).unwrap();
            prop_assert_eq!(result, Value::Integer(a), "Double negation should return original value");
        }

        #[test]
        fn test_comparison_transitivity(a: i64, b: i64, c: i64) {
            prop_assume!(a < b && b < c);

            let mut interpreter = Interpreter::new();

            // a < c should be true if a < b && b < c
            let expr = make_binary_op(
                make_literal(a),
                BinaryOp::Less,
                make_literal(c)
            );

            let result = interpreter.eval_expr(&expr).unwrap();
            prop_assert_eq!(result, Value::Bool(true), "Comparison should be transitive");
        }

        #[test]
        fn test_string_concatenation_associative(s1 in "[a-z]{0,10}", s2 in "[a-z]{0,10}", s3 in "[a-z]{0,10}") {
            let mut interpreter = Interpreter::new();

            // (s1 + s2) + s3
            let expr1 = make_binary_op(
                make_binary_op(
                    Expr::new(
                        ExprKind::Literal(Literal::String(s1.clone())),
                        ruchy::frontend::ast::Span { start: 0, end: 0 }
                    ),
                    BinaryOp::Add,
                    Expr::new(
                        ExprKind::Literal(Literal::String(s2.clone())),
                        ruchy::frontend::ast::Span { start: 0, end: 0 }
                    )
                ),
                BinaryOp::Add,
                Expr::new(
                    ExprKind::Literal(Literal::String(s3.clone())),
                    ruchy::frontend::ast::Span { start: 0, end: 0 }
                )
            );

            // s1 + (s2 + s3)
            let expr2 = make_binary_op(
                Expr::new(
                    ExprKind::Literal(Literal::String(s1.clone())),
                    ruchy::frontend::ast::Span { start: 0, end: 0 }
                ),
                BinaryOp::Add,
                make_binary_op(
                    Expr::new(
                        ExprKind::Literal(Literal::String(s2)),
                        ruchy::frontend::ast::Span { start: 0, end: 0 }
                    ),
                    BinaryOp::Add,
                    Expr::new(
                        ExprKind::Literal(Literal::String(s3)),
                        ruchy::frontend::ast::Span { start: 0, end: 0 }
                    )
                )
            );

            let result1 = interpreter.eval_expr(&expr1);
            let result2 = interpreter.eval_expr(&expr2);

            match (result1, result2) {
                (Ok(v1), Ok(v2)) => {
                    prop_assert_eq!(v1, v2, "String concatenation should be associative");
                }
                _ => {}
            }
        }
    }
}

// Big O Complexity Documentation
// All eval_* functions in the interpreter have the following complexities:
// - eval_literal: O(1) - Direct value construction
// - eval_binary_op: O(1) for arithmetic, O(n) for string concatenation where n is string length
// - eval_unary_op: O(1) - Single operation
// - eval_variable: O(1) average, O(log n) worst case for HashMap lookup
// - eval_assignment: O(1) average, O(log n) worst case for HashMap insert
// - eval_if: O(1) + complexity of evaluated branch
// - eval_while: O(n * m) where n is iterations and m is body complexity
// - eval_for: O(n * m) where n is range size and m is body complexity
// - eval_block: O(n) where n is number of statements
// - eval_list: O(n) where n is number of elements
// - eval_tuple: O(n) where n is number of elements
// - eval_range: O(1) - Creates range object without iteration
