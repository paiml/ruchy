//! TDD Tests for eval_prim refactoring
//!
//! These tests establish the behavior contract for eval_prim before refactoring.
//! By writing tests first, we ensure the refactoring doesn't break functionality.
//!
//! NOTE: Currently disabled - testing transpiler functionality that doesn't exist

/*
use ruchy::runtime::{Interpreter, Value};
use std::rc::Rc;
use ruchy::frontend::ast::{Expr, ExprKind, Literal};
use std::rc::Rc;
use ruchy::transpiler::CoreExpr;
use std::rc::Rc;
use proptest::prelude::*;
use std::rc::Rc;

#[cfg(test)]
mod eval_prim_tdd_tests {
    use super::*;

    // Helper function to create a test interpreter
    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // Helper to create literal expressions
    pub fn int_lit(n: i64) -> Box<Expr> {
        Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(n)),
            span: Default::default(),
            attributes: Vec::new(),
        })
    }

    pub fn float_lit(f: f64) -> Box<Expr> {
        Box::new(Expr {
            kind: ExprKind::Literal(Literal::Float(f)),
            span: Default::default(),
            attributes: Vec::new(),
        })
    }

    pub fn bool_lit(b: bool) -> Box<Expr> {
        Box::new(Expr {
            kind: ExprKind::Literal(Literal::Bool(b)),
            span: Default::default(),
            attributes: Vec::new(),
        })
    }

    pub fn string_lit(s: &str) -> Box<Expr> {
        Box::new(Expr {
            kind: ExprKind::Literal(Literal::String(s.to_string())),
            span: Default::default(),
            attributes: Vec::new(),
        })
    }

    // Test arithmetic operations
    mod arithmetic_tests {
        use super::*;

        #[test]
        fn test_add_integers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Add, vec![int_lit(5), int_lit(3)]);
            assert_eq!(result, Ok(Value::Integer(8)));
        }

        #[test]
        fn test_add_floats() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Add, vec![float_lit(5.5), float_lit(3.2)]);
            match result {
                Ok(Value::Float(f)) => assert!((f - 8.7).abs() < 0.001),
                _ => panic!("Expected float result"),
            }
        }

        #[test]
        fn test_add_mixed_numbers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Add, vec![int_lit(5), float_lit(3.2)]);
            match result {
                Ok(Value::Float(f)) => assert!((f - 8.2).abs() < 0.001),
                _ => panic!("Expected float result"),
            }
        }

        #[test]
        fn test_subtract_integers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Sub, vec![int_lit(10), int_lit(3)]);
            assert_eq!(result, Ok(Value::Integer(7)));
        }

        #[test]
        fn test_multiply_integers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Mul, vec![int_lit(4), int_lit(3)]);
            assert_eq!(result, Ok(Value::Integer(12)));
        }

        #[test]
        fn test_divide_integers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Div, vec![int_lit(10), int_lit(3)]);
            assert_eq!(result, Ok(Value::Integer(3)));
        }

        #[test]
        fn test_divide_by_zero() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Div, vec![int_lit(10), int_lit(0)]);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Division by zero"));
        }

        #[test]
        fn test_modulo() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Mod, vec![int_lit(10), int_lit(3)]);
            assert_eq!(result, Ok(Value::Integer(1)));
        }

        #[test]
        fn test_power() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Pow, vec![int_lit(2), int_lit(3)]);
            assert_eq!(result, Ok(Value::Integer(8)));
        }
    }

    // Test comparison operations
    mod comparison_tests {
        use super::*;

        #[test]
        fn test_equal_integers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Eq, vec![int_lit(5), int_lit(5)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_not_equal_integers() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Ne, vec![int_lit(5), int_lit(3)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_less_than() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Lt, vec![int_lit(3), int_lit(5)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_less_than_or_equal() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Le, vec![int_lit(5), int_lit(5)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_greater_than() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Gt, vec![int_lit(5), int_lit(3)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_greater_than_or_equal() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Ge, vec![int_lit(5), int_lit(5)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }
    }

    // Test logical operations
    mod logical_tests {
        use super::*;

        #[test]
        fn test_logical_and() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::And, vec![bool_lit(true), bool_lit(true)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_logical_or() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Or, vec![bool_lit(false), bool_lit(true)]);
            assert_eq!(result, Ok(Value::Bool(true)));
        }

        #[test]
        fn test_logical_not() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Not, vec![bool_lit(true)]);
            assert_eq!(result, Ok(Value::Bool(false)));
        }
    }

    // Test string operations
    mod string_tests {
        use super::*;

        #[test]
        fn test_string_concat() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Concat, vec![string_lit("Hello, "), string_lit("World!")]);
            assert_eq!(result, Ok(Value::String(Rc::new("Hello, World!".to_string()))));
        }

        #[test]
        fn test_string_length() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Len, vec![string_lit("Hello")]);
            assert_eq!(result, Ok(Value::Integer(5)));
        }
    }

    // Test error cases
    mod error_tests {
        use super::*;

        #[test]
        fn test_type_mismatch_add() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Add, vec![int_lit(5), bool_lit(true)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_wrong_arity() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Add, vec![int_lit(5)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_operation_on_type() {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(Prim::Mod, vec![string_lit("hello"), string_lit("world")]);
            assert!(result.is_err());
        }
    }
}

// Property-based tests for eval_prim
#[cfg(test)]
mod eval_prim_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_add_commutative(a: i32, b: i32) {
            let mut interp1 = setup_interpreter();
            let mut interp2 = setup_interpreter();
            
            let result1 = interp1.eval_prim(
                Prim::Add, 
                vec![int_lit(a as i64), int_lit(b as i64)]
            );
            let result2 = interp2.eval_prim(
                Prim::Add,
                vec![int_lit(b as i64), int_lit(a as i64)]
            );
            
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_add_associative(a: i8, b: i8, c: i8) {
            let mut interp = setup_interpreter();
            
            // (a + b) + c
            let ab = match interp.eval_prim(Prim::Add, vec![int_lit(a as i64), int_lit(b as i64)]) {
                Ok(Value::Integer(n)) => n,
                _ => return Ok(()),
            };
            let result1 = interp.eval_prim(Prim::Add, vec![int_lit(ab), int_lit(c as i64)]);
            
            // a + (b + c)
            let bc = match interp.eval_prim(Prim::Add, vec![int_lit(b as i64), int_lit(c as i64)]) {
                Ok(Value::Integer(n)) => n,
                _ => return Ok(()),
            };
            let result2 = interp.eval_prim(Prim::Add, vec![int_lit(a as i64), int_lit(bc)]);
            
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_multiply_by_zero(n: i32) {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(
                Prim::Mul,
                vec![int_lit(n as i64), int_lit(0)]
            );
            prop_assert_eq!(result, Ok(Value::Integer(0)));
        }

        #[test]
        fn test_divide_by_one(n: i32) {
            let mut interp = setup_interpreter();
            let result = interp.eval_prim(
                Prim::Div,
                vec![int_lit(n as i64), int_lit(1)]
            );
            prop_assert_eq!(result, Ok(Value::Integer(n as i64)));
        }

        #[test]
        fn test_comparison_trichotomy(a: i32, b: i32) {
            let mut interp = setup_interpreter();
            
            let lt = interp.eval_prim(Prim::Lt, vec![int_lit(a as i64), int_lit(b as i64)]);
            let eq = interp.eval_prim(Prim::Eq, vec![int_lit(a as i64), int_lit(b as i64)]);
            let gt = interp.eval_prim(Prim::Gt, vec![int_lit(a as i64), int_lit(b as i64)]);
            
            // Exactly one should be true
            let results = vec![lt, eq, gt];
            let true_count = results.iter()
                .filter(|r| matches!(r, Ok(Value::Bool(true))))
                .count();
            
            prop_assert_eq!(true_count, 1);
        }

        #[test]
        fn test_boolean_operations_consistency(a: bool, b: bool) {
            let mut interp = setup_interpreter();
            
            // De Morgan's law: !(a && b) == !a || !b
            let and_result = interp.eval_prim(Prim::And, vec![bool_lit(a), bool_lit(b)]);
            let not_and = match and_result {
                Ok(Value::Bool(v)) => interp.eval_prim(Prim::Not, vec![bool_lit(v)]),
                _ => return Ok(()),
            };
            
            let not_a = match interp.eval_prim(Prim::Not, vec![bool_lit(a)]) {
                Ok(Value::Bool(v)) => v,
                _ => return Ok(()),
            };
            let not_b = match interp.eval_prim(Prim::Not, vec![bool_lit(b)]) {
                Ok(Value::Bool(v)) => v,
                _ => return Ok(()),
            };
            let or_not = interp.eval_prim(Prim::Or, vec![bool_lit(not_a), bool_lit(not_b)]);
            
            prop_assert_eq!(not_and, or_not);
        }

        #[test]
        fn test_no_panic_on_random_input(
            prim_idx: u8,
            args: Vec<u8>
        ) {
            // This test ensures eval_prim never panics on any input
            let primitives = vec![
                Prim::Add, Prim::Sub, Prim::Mul, Prim::Div, Prim::Mod,
                Prim::Eq, Prim::Ne, Prim::Lt, Prim::Le, Prim::Gt, Prim::Ge,
                Prim::And, Prim::Or, Prim::Not,
            ];
            
            let prim = &primitives[prim_idx as usize % primitives.len()];
            let mut interp = setup_interpreter();
            
            let exprs: Vec<Box<Expr>> = args.iter()
                .take(3) // Limit to reasonable number of args
                .map(|&n| int_lit(n as i64))
                .collect();
            
            // Should not panic, may return error
            let _ = interp.eval_prim(prim.clone(), exprs);
        }
    }

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }
}*/
