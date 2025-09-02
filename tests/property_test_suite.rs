//! Property-based tests for Ruchy core algorithms
//!
//! These tests verify mathematical invariants and properties that should hold
//! for all valid inputs, using the proptest framework.

#![allow(warnings)]  // Allow all warnings for test files

use proptest::prelude::*;
use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;
use ruchy::runtime::interpreter::{Interpreter, Value};
use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, Span};

/// Property: Parser should never panic on any input
proptest! {
    #[test]
    fn parser_never_panics(input in ".*") {
        let mut parser = Parser::new(&input);
        // Test parser without panic catching - just verify it doesn't crash
        let result = parser.parse();
        // Either succeeds or fails gracefully
        prop_assert!(result.is_ok() || result.is_err());
    }
}

/// Property: Valid expressions should always transpile to valid Rust
proptest! {
    #[test]
    fn valid_expressions_transpile(
        a in 0i32..1000,
        b in 0i32..1000,
        op in prop_oneof![
            Just(BinaryOp::Add),
            Just(BinaryOp::Subtract),
            Just(BinaryOp::Multiply)
        ]
    ) {
        let mut transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Binary {
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(a as i64)),
                Span::new(0, 5)
            )),
            op,
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(b as i64)),
                Span::new(6, 11)
            ))
        }, Span::new(0, 11));
        
        let result = transpiler.transpile_expr(&expr);
        prop_assert!(result.is_ok());
    }
}

/// Property: Integer arithmetic should be associative where applicable
proptest! {
    #[test]
    fn integer_arithmetic_associative(
        a in 0i64..100,
        b in 0i64..100,
        c in 0i64..100
    ) {
        let mut interpreter = Interpreter::new();
        
        // Create (a + b) + c
        let expr1 = create_add_expr(create_add_expr(
            create_int_expr(a),
            create_int_expr(b)
        ), create_int_expr(c));
        
        // Create a + (b + c)
        let expr2 = create_add_expr(create_int_expr(a), create_add_expr(
            create_int_expr(b),
            create_int_expr(c)
        ));
        
        let result1 = interpreter.eval_expr(&expr1);
        let result2 = interpreter.eval_expr(&expr2);
        
        if let (Ok(val1), Ok(val2)) = (result1, result2) {
            prop_assert_eq!(val1, val2);
        }
    }
}

/// Property: Multiplication by zero should always yield zero
proptest! {
    #[test]
    fn multiplication_by_zero(n in i64::MIN..i64::MAX) {
        let mut interpreter = Interpreter::new();
        
        let expr = create_mul_expr(create_int_expr(n), create_int_expr(0));
        let result = interpreter.eval_expr(&expr);
        
        if let Ok(Value::Integer(val)) = result {
            prop_assert_eq!(val, 0);
        }
    }
}

/// Property: Identity operations
proptest! {
    #[test]
    fn identity_operations(n in i64::MIN/2..i64::MAX/2) {
        let mut interpreter = Interpreter::new();
        
        // n + 0 = n
        let add_zero = create_add_expr(create_int_expr(n), create_int_expr(0));
        if let Ok(Value::Integer(result)) = interpreter.eval_expr(&add_zero) {
            prop_assert_eq!(result, n);
        }
        
        // n * 1 = n
        let mul_one = create_mul_expr(create_int_expr(n), create_int_expr(1));
        if let Ok(Value::Integer(result)) = interpreter.eval_expr(&mul_one) {
            prop_assert_eq!(result, n);
        }
    }
}

/// Property: Boolean operations should be consistent
proptest! {
    #[test]
    fn boolean_consistency(a in any::<bool>(), b in any::<bool>()) {
        let mut interpreter = Interpreter::new();
        
        // De Morgan's law: !(a && b) = !a || !b
        let left_expr = create_not_expr(create_and_expr(
            create_bool_expr(a),
            create_bool_expr(b)
        ));
        
        let right_expr = create_or_expr(
            create_not_expr(create_bool_expr(a)),
            create_not_expr(create_bool_expr(b))
        );
        
        let left_result = interpreter.eval_expr(&left_expr);
        let right_result = interpreter.eval_expr(&right_expr);
        
        if let (Ok(Value::Bool(left)), Ok(Value::Bool(right))) = (left_result, right_result) {
            prop_assert_eq!(left, right);
        }
    }
}

/// Property: String concatenation should be associative
proptest! {
    #[test]
    fn string_concatenation_associative(
        s1 in "[a-zA-Z]{0,10}",
        s2 in "[a-zA-Z]{0,10}",
        s3 in "[a-zA-Z]{0,10}"
    ) {
        let mut interpreter = Interpreter::new();
        
        // (s1 + s2) + s3
        let expr1 = create_str_concat(
            create_str_concat(create_str_expr(s1.clone()), create_str_expr(s2.clone())),
            create_str_expr(s3.clone())
        );
        
        // s1 + (s2 + s3)
        let expr2 = create_str_concat(
            create_str_expr(s1),
            create_str_concat(create_str_expr(s2), create_str_expr(s3))
        );
        
        let result1 = interpreter.eval_expr(&expr1);
        let result2 = interpreter.eval_expr(&expr2);
        
        if let (Ok(val1), Ok(val2)) = (result1, result2) {
            prop_assert_eq!(val1, val2);
        }
    }
}

/// Property: Transpiler should be deterministic
proptest! {
    #[test]
    fn transpiler_deterministic(n in 0i32..1000) {
        let mut transpiler = Transpiler::new();
        let expr = create_int_expr(n as i64);
        
        let result1 = transpiler.transpile_expr(&expr);
        let result2 = transpiler.transpile_expr(&expr);
        
        match (result1, result2) {
            (Ok(code1), Ok(code2)) => {
                prop_assert_eq!(code1.to_string(), code2.to_string());
            }
            (Err(_), Err(_)) => {
                // Both errors is also deterministic
                prop_assert!(true);
            }
            _ => {
                // One success, one error is non-deterministic
                prop_assert!(false, "Transpiler is non-deterministic");
            }
        }
    }
}

// Helper functions to create test expressions
fn create_int_expr(n: i64) -> Expr {
    Expr::new(ExprKind::Literal(Literal::Integer(n)), Span::new(0, 10))
}

fn create_bool_expr(b: bool) -> Expr {
    Expr::new(ExprKind::Literal(Literal::Bool(b)), Span::new(0, 10))
}

fn create_str_expr(s: String) -> Expr {
    Expr::new(ExprKind::Literal(Literal::String(s)), Span::new(0, 10))
}

fn create_add_expr(left: Expr, right: Expr) -> Expr {
    Expr::new(ExprKind::Binary {
        left: Box::new(left),
        op: BinaryOp::Add,
        right: Box::new(right)
    }, Span::new(0, 20))
}

fn create_mul_expr(left: Expr, right: Expr) -> Expr {
    Expr::new(ExprKind::Binary {
        left: Box::new(left),
        op: BinaryOp::Multiply,
        right: Box::new(right)
    }, Span::new(0, 20))
}

fn create_and_expr(left: Expr, right: Expr) -> Expr {
    Expr::new(ExprKind::Binary {
        left: Box::new(left),
        op: BinaryOp::And,
        right: Box::new(right)
    }, Span::new(0, 20))
}

fn create_or_expr(left: Expr, right: Expr) -> Expr {
    Expr::new(ExprKind::Binary {
        left: Box::new(left),
        op: BinaryOp::Or,
        right: Box::new(right)
    }, Span::new(0, 20))
}

fn create_not_expr(operand: Expr) -> Expr {
    Expr::new(ExprKind::Unary {
        op: ruchy::frontend::ast::UnaryOp::Not,
        operand: Box::new(operand)
    }, Span::new(0, 20))
}

fn create_str_concat(left: Expr, right: Expr) -> Expr {
    Expr::new(ExprKind::Binary {
        left: Box::new(left),
        op: BinaryOp::Add, // String concatenation uses Add
        right: Box::new(right)
    }, Span::new(0, 20))
}