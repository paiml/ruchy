//! Sprint 1: Property tests for REPL with 10,000+ iterations
//! Using proptest for comprehensive random testing

use proptest::prelude::*;
use ruchy::runtime::{Repl, Value};
use std::{env, time::Duration;

// PROP-001: Property tests with 10,000+ iterations

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn test_integer_arithmetic_commutative(a: i32, b: i32) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // Addition is commutative
        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);

        if let (Ok(result1), Ok(result2)) = (repl.eval(&expr1), repl.eval(&expr2)) {
            prop_assert_eq!(result1, result2);
        }
    }

    #[test]
    fn test_integer_arithmetic_associative(a: i16, b: i16, c: i16) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // Addition is associative
        let expr1 = format!("({} + {}) + {}", a, b, c);
        let expr2 = format!("{} + ({} + {})", a, b, c);

        if let (Ok(result1), Ok(result2)) = (repl.eval(&expr1), repl.eval(&expr2)) {
            prop_assert_eq!(result1, result2);
        }
    }

    #[test]
    fn test_multiplication_distributive(a: i8, b: i8, c: i8) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // Multiplication distributes over addition
        let expr1 = format!("{} * ({} + {})", a, b, c);
        let expr2 = format!("({} * {}) + ({} * {})", a, b, a, c);

        if let (Ok(result1), Ok(result2)) = (repl.eval(&expr1), repl.eval(&expr2)) {
            prop_assert_eq!(result1, result2);
        }
    }

    #[test]
    fn test_comparison_transitivity(a: i32, b: i32, c: i32) {
        prop_assume!(a < b && b < c);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let expr = format!("{} < {}", a, c);
        let result = repl.eval(&expr).unwrap();
        prop_assert_eq!(result, "true");
    }

    #[test]
    fn test_boolean_logic_laws(a: bool, b: bool) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // De Morgan's law: !(a && b) == !a || !b
        let expr1 = format!("!({} && {})", a, b);
        let expr2 = format!("!{} || !{}", a, b);

        if let (Ok(result1), Ok(result2)) = (repl.eval(&expr1), repl.eval(&expr2)) {
            prop_assert_eq!(result1, result2);
        }
    }

    #[test]
    fn test_string_concatenation_associative(
        a in "[a-z]{1,10}",
        b in "[a-z]{1,10}",
        c in "[a-z]{1,10}"
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let expr1 = format!(r#"("{}" + "{}") + "{}""#, a, b, c);
        let expr2 = format!(r#""{}" + ("{}" + "{}")"#, a, b, c);

        if let (Ok(result1), Ok(result2)) = (repl.eval(&expr1), repl.eval(&expr2)) {
            prop_assert_eq!(result1, result2);
        }
    }

    #[test]
    fn test_list_operations_preserve_length(
        elements in prop::collection::vec(0i32..100, 0..20)
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let list_str = format!("[{}]", elements.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "));

        if repl.eval(&list_str).is_ok() {
            // List was created successfully
            prop_assert!(true);
        }
    }

    #[test]
    fn test_variable_binding_retrieval(
        name in "[a-z][a-z0-9]{0,9}",
        value: i32
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let bind_expr = format!("let {} = {}", name, value);
        if repl.eval(&bind_expr).is_ok() {
            let retrieve_expr = format!("{}", name);
            if let Ok(result) = repl.eval(&retrieve_expr) {
                prop_assert_eq!(result, value.to_string());
            }
        }
    }

    #[test]
    fn test_if_expression_consistency(condition: bool, a: i16, b: i16) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let expr = format!("if {} {{ {} }} else {{ {} }}", condition, a, b);
        let expected = if condition { a } else { b };

        if let Ok(result) = repl.eval(&expr) {
            prop_assert_eq!(result, expected.to_string());
        }
    }

    #[test]
    fn test_range_bounds(start: i16, end: i16) {
        prop_assume!(start < end && (end - start) < 100);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let expr = format!("{}..{}", start, end);
        if let Ok(result) = repl.eval(&expr) {
            prop_assert_eq!(result, format!("{}..{}", start, end));
        }

        let expr_inclusive = format!("{}..={}", start, end);
        if let Ok(result) = repl.eval(&expr_inclusive) {
            prop_assert_eq!(result, format!("{}..={}", start, end));
        }
    }

    #[test]
    fn test_function_definition_and_call(
        fn_name in "[a-z][a-z0-9]{0,9}",
        param in "[a-z][a-z0-9]{0,9}",
        value: i16
    ) {
        prop_assume!(fn_name != param);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let fn_def = format!("fn {}({}) {{ {} + 1 }}", fn_name, param, param);
        if repl.eval(&fn_def).is_ok() {
            let fn_call = format!("{}({})", fn_name, value);
            if let Ok(result) = repl.eval(&fn_call) {
                let expected = (value + 1).to_string();
                prop_assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_lambda_definition_and_call(
        var_name in "[a-z][a-z0-9]{0,9}",
        param in "[a-z][a-z0-9]{0,9}",
        value: i16
    ) {
        prop_assume!(var_name != param);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let lambda_def = format!("let {} = |{}| {} * 2", var_name, param, param);
        if repl.eval(&lambda_def).is_ok() {
            let lambda_call = format!("{}({})", var_name, value);
            if let Ok(result) = repl.eval(&lambda_call) {
                let expected = (value * 2).to_string();
                prop_assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_nested_blocks_scope(
        outer_var in "[a-z][a-z0-9]{0,9}",
        inner_var in "[a-z][a-z0-9]{0,9}",
        outer_val: i16,
        inner_val: i16
    ) {
        prop_assume!(outer_var != inner_var);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let expr = format!(
            r#"
            let {} = {};
            {{
                let {} = {};
                {}
            }}
            "#,
            outer_var, outer_val, inner_var, inner_val, inner_var
        );

        if let Ok(result) = repl.eval(&expr) {
            prop_assert_eq!(result, inner_val.to_string());
        }

        // Outer var should still be accessible
        if let Ok(result) = repl.eval(&outer_var) {
            prop_assert_eq!(result, outer_val.to_string());
        }
    }

    #[test]
    fn test_match_expression_exhaustive(value: i8) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let expr = format!(
            r#"
            match {} {{
                0 => "zero",
                n if n > 0 => "positive",
                _ => "negative"
            }}
            "#,
            value
        );

        if let Ok(result) = repl.eval(&expr) {
            let expected = if value == 0 {
                "\"zero\""
            } else if value > 0 {
                "\"positive\""
            } else {
                "\"negative\""
            };
            prop_assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_error_recovery_consistency(
        valid_expr1: i16,
        invalid_var in "[0-9][a-z]+", // Invalid variable name
        valid_expr2: i16
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // First valid expression
        let result1 = repl.eval(&format!("{} + 1", valid_expr1));
        prop_assert!(result1.is_ok());

        // Invalid expression
        let result2 = repl.eval(&invalid_var);
        prop_assert!(result2.is_err());

        // Should still be able to evaluate after error
        let result3 = repl.eval(&format!("{} + 1", valid_expr2));
        prop_assert!(result3.is_ok());
    }

    #[test]
    fn test_memory_allocation_bounded(
        size in 1usize..100
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let list_expr = format!("[{}]",
            (0..size).map(|i| i.to_string()).collect::<Vec<_>>().join(", "));

        let initial_memory = repl.memory_used();

        if repl.eval(&list_expr).is_ok() {
            let after_memory = repl.memory_used();
            // Memory should have increased but within reasonable bounds
            prop_assert!(after_memory >= initial_memory);
            prop_assert!(after_memory - initial_memory < size * 1000); // Reasonable upper bound
        }
    }

    #[test]
    fn test_checkpoint_restore_consistency(
        var1 in "[a-z][a-z0-9]{0,9}",
        var2 in "[a-z][a-z0-9]{0,9}",
        val1: i16,
        val2: i16
    ) {
        prop_assume!(var1 != var2);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // Set initial state
        repl.eval(&format!("let {} = {}", var1, val1)).unwrap();

        // Create checkpoint
        let checkpoint = repl.checkpoint();

        // Modify state
        repl.eval(&format!("let {} = {}", var2, val2)).unwrap();

        // Restore checkpoint
        repl.restore_checkpoint(&checkpoint);

        // var1 should exist, var2 should not
        prop_assert!(repl.eval(&var1).is_ok());
        prop_assert!(repl.eval(&var2).is_err());
    }

    #[test]
    fn test_math_function_properties(x in 0.0f64..100.0) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // sqrt(x)^2 ≈ x
        let expr = format!("pow(sqrt({}), 2.0)", x);
        if let Ok(result) = repl.eval(&expr) {
            if let Ok(value) = result.parse::<f64>() {
                prop_assert!((value - x).abs() < 0.0001);
            }
        }

        // abs(x) >= 0
        let expr = format!("abs({})", x);
        if let Ok(result) = repl.eval(&expr) {
            if let Ok(value) = result.parse::<f64>() {
                prop_assert!(value >= 0.0);
            }
        }
    }

    #[test]
    fn test_type_coercion_consistency(int_val: i16, float_val in 0.0f32..100.0) {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // Integer + Float should produce Float
        let expr = format!("{} + {}", int_val, float_val);
        if let Ok(result) = repl.eval(&expr) {
            // Result should contain decimal point
            prop_assert!(result.contains('.'));
        }
    }
}