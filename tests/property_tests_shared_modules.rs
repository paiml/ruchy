//! Property-based tests for shared modules
//! Tests invariants and properties that should always hold

use proptest::prelude::*;
use std::rc::Rc;
use ruchy::runtime::{Value, binary_ops::evaluate_binary_op};
use std::rc::Rc;
use ruchy::frontend::ast::BinaryOp;
use std::rc::Rc;

#[cfg(test)]
mod binary_ops_properties {
    use super::*;

    proptest! {
        #[test]
        fn test_int_addition_commutative(a: i64, b: i64) {
            // Skip overflow cases
            if a.checked_add(b).is_none() {
                return Ok(());
            }
            
            let lhs = Value::Integer(a);
            let rhs = Value::Integer(b);
            
            let result1 = evaluate_binary_op(&BinaryOp::Add, &lhs, &rhs).unwrap();
            let result2 = evaluate_binary_op(&BinaryOp::Add, &rhs, &lhs).unwrap();
            
            assert_eq!(result1, result2, "Addition should be commutative");
        }

        #[test]
        fn test_int_multiplication_commutative(a: i64, b: i64) {
            // Skip overflow cases
            if a.checked_mul(b).is_none() {
                return Ok(());
            }
            
            let lhs = Value::Integer(a);
            let rhs = Value::Integer(b);
            
            let result1 = evaluate_binary_op(&BinaryOp::Multiply, &lhs, &rhs).unwrap();
            let result2 = evaluate_binary_op(&BinaryOp::Multiply, &rhs, &lhs).unwrap();
            
            assert_eq!(result1, result2, "Multiplication should be commutative");
        }

        #[test]
        fn test_string_concatenation_associative(a: String, b: String, c: String) {
            let val_a = Value::String(Rc::new(a.clone()));
            let val_b = Value::String(Rc::new(b.clone()));
            let val_c = Value::String(Rc::new(c.clone()));
            
            // (a + b) + c
            let ab = evaluate_binary_op(&BinaryOp::Add, &val_a, &val_b).unwrap();
            let abc1 = evaluate_binary_op(&BinaryOp::Add, &ab, &val_c).unwrap();
            
            // a + (b + c)
            let bc = evaluate_binary_op(&BinaryOp::Add, &val_b, &val_c).unwrap();
            let abc2 = evaluate_binary_op(&BinaryOp::Add, &val_a, &bc).unwrap();
            
            assert_eq!(abc1, abc2, "String concatenation should be associative");
        }

        #[test]
        fn test_boolean_and_identity(a: bool) {
            let val = Value::Bool(a);
            let true_val = Value::Bool(true);
            
            let result = evaluate_binary_op(&BinaryOp::And, &val, &true_val).unwrap();
            
            assert_eq!(result, val, "AND with true should be identity");
        }

        #[test]
        fn test_boolean_or_identity(a: bool) {
            let val = Value::Bool(a);
            let false_val = Value::Bool(false);
            
            let result = evaluate_binary_op(&BinaryOp::Or, &val, &false_val).unwrap();
            
            assert_eq!(result, val, "OR with false should be identity");
        }

        #[test]
        fn test_division_by_one_identity(a: i64) {
            let val = Value::Integer(a);
            let one = Value::Integer(1);
            
            let result = evaluate_binary_op(&BinaryOp::Divide, &val, &one).unwrap();
            
            assert_eq!(result, val, "Division by 1 should be identity");
        }

        #[test]
        fn test_comparison_reflexivity(a: i64) {
            let val = Value::Integer(a);
            
            let eq_result = evaluate_binary_op(&BinaryOp::Equal, &val, &val).unwrap();
            assert_eq!(eq_result, Value::Bool(true), "Value should equal itself");
            
            let le_result = evaluate_binary_op(&BinaryOp::LessEqual, &val, &val).unwrap();
            assert_eq!(le_result, Value::Bool(true), "Value should be <= itself");
            
            let ge_result = evaluate_binary_op(&BinaryOp::GreaterEqual, &val, &val).unwrap();
            assert_eq!(ge_result, Value::Bool(true), "Value should be >= itself");
        }

        #[test]
        fn test_comparison_antisymmetry(a: i64, b: i64) {
            if a == b {
                return Ok(()); // Skip equal values
            }
            
            let val_a = Value::Integer(a);
            let val_b = Value::Integer(b);
            
            let less = evaluate_binary_op(&BinaryOp::Less, &val_a, &val_b).unwrap();
            let greater = evaluate_binary_op(&BinaryOp::Greater, &val_a, &val_b).unwrap();
            
            // If a < b, then !(a > b)
            if let (Value::Bool(l), Value::Bool(g)) = (less, greater) {
                assert_ne!(l && g, true, "Value cannot be both less than and greater than another");
            }
        }
    }
}

#[cfg(test)]
mod pattern_matching_properties {
    use ruchy::runtime::pattern_matching::{match_literal_pattern, values_equal};
    use ruchy::frontend::ast::Literal;
    use super::*;

    proptest! {
        #[test]
        fn test_literal_pattern_reflexivity(n: i64) {
            let value = Value::Integer(n);
            let pattern = Literal::Integer(n);
            
            assert!(match_literal_pattern(&value, &pattern), 
                    "Literal should match itself");
        }

        #[test]
        fn test_values_equal_reflexivity(n: i64) {
            let value = Value::Integer(n);
            
            assert!(values_equal(&value, &value), 
                    "Value should equal itself");
        }

        #[test]
        fn test_values_equal_symmetry(a: i64, b: i64) {
            let val_a = Value::Integer(a);
            let val_b = Value::Integer(b);
            
            let eq1 = values_equal(&val_a, &val_b);
            let eq2 = values_equal(&val_b, &val_a);
            
            assert_eq!(eq1, eq2, "Equality should be symmetric");
        }

        #[test]
        fn test_string_values_equal(s: String) {
            let val1 = Value::String(Rc::new(s.clone()));
            let val2 = Value::String(s);
            
            assert!(values_equal(&val1, &val2), 
                    "Same strings should be equal");
        }
    }
}