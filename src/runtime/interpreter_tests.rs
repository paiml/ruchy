//! Tests for the interpreter module
//!
//! This module contains all tests for the interpreter, extracted from interpreter.rs
//! for maintainability and to reduce the main module size.

use super::*;
use crate::frontend::ast::{BinaryOp as AstBinaryOp, Expr, ExprKind, Literal, Span};
use crate::frontend::Parser;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a simple literal expression
    fn make_literal(val: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(val, None)),
            span: Span::default(),
        }
    }

    // Helper to create a binary expression
    fn make_binary(left: Expr, op: AstBinaryOp, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: Span::default(),
        }
    }

    #[test]
    fn test_eval_literal() {
        let mut interp = Interpreter::new();
        let expr = make_literal(42);
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut interp = Interpreter::new();

        // Test addition
        let expr = make_binary(make_literal(10), AstBinaryOp::Add, make_literal(5));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(15));

        // Test subtraction
        let expr = make_binary(make_literal(10), AstBinaryOp::Subtract, make_literal(3));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(7));

        // Test multiplication
        let expr = make_binary(make_literal(4), AstBinaryOp::Multiply, make_literal(7));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(28));

        // Test division
        let expr = make_binary(make_literal(20), AstBinaryOp::Divide, make_literal(4));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_eval_comparison() {
        let mut interp = Interpreter::new();

        // Test equality
        let expr = make_binary(make_literal(5), AstBinaryOp::Equal, make_literal(5));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        let expr = make_binary(make_literal(5), AstBinaryOp::NotEqual, make_literal(3));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Test less than
        let expr = make_binary(make_literal(3), AstBinaryOp::Less, make_literal(5));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Test greater than
        let expr = make_binary(make_literal(7), AstBinaryOp::Greater, make_literal(2));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_variables() {
        let mut interp = Interpreter::new();

        // Set a variable
        interp.env_set("x".to_string(), Value::Integer(42));

        // Create identifier expression
        let expr = Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: Span::default(),
        };

        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_string_operations() {
        let mut interp = Interpreter::new();

        // Create string literals
        let hello = Expr {
            kind: ExprKind::Literal(Literal::String("Hello".to_string())),
            span: Span::default(),
        };

        let world = Expr {
            kind: ExprKind::Literal(Literal::String(" World".to_string())),
            span: Span::default(),
        };

        // Test string concatenation
        let expr = make_binary(hello, AstBinaryOp::Add, world);
        let result = interp.eval_expr(&expr).unwrap();

        match result {
            Value::String(s) => assert_eq!(&**s, "Hello World"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_array_operations() {
        let mut interp = Interpreter::new();

        // Create array literal
        let arr = Expr {
            kind: ExprKind::List(vec![make_literal(1), make_literal(2), make_literal(3)]),
            span: Span::default(),
        };

        let result = interp.eval_expr(&arr).unwrap();
        match result {
            Value::Array(a) => {
                assert_eq!(a.len(), 3);
                assert_eq!(a[0], Value::Integer(1));
                assert_eq!(a[1], Value::Integer(2));
                assert_eq!(a[2], Value::Integer(3));
            }
            _ => panic!("Expected array value"),
        }
    }

    #[test]
    fn test_if_expression() {
        let mut interp = Interpreter::new();

        // if true { 10 } else { 20 }
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Bool(true)),
                    span: Span::default(),
                }),
                then_branch: Box::new(make_literal(10)),
                else_branch: Some(Box::new(make_literal(20))),
            },
            span: Span::default(),
        };

        assert_eq!(interp.eval_expr(&if_expr).unwrap(), Value::Integer(10));

        // if false { 10 } else { 20 }
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Bool(false)),
                    span: Span::default(),
                }),
                then_branch: Box::new(make_literal(10)),
                else_branch: Some(Box::new(make_literal(20))),
            },
            span: Span::default(),
        };

        assert_eq!(interp.eval_expr(&if_expr).unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_type_feedback() {
        let mut interp = Interpreter::new();

        // Set variables with different types
        interp.env_set("x".to_string(), Value::Integer(42));
        interp.env_set("y".to_string(), Value::Float(3.14));
        interp.env_set("z".to_string(), Value::from_string("test".to_string()));

        // Access variables multiple times to trigger type feedback
        for _ in 0..5 {
            let expr = Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
            };
            assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(42));
        }

        // Check type feedback state
        let tf = &interp.type_feedback;
        assert!(tf.variable_types.contains_key("x"));
    }

    // Add more tests as needed...
}
