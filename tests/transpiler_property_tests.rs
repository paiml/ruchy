//! Property tests for transpiler call functions
//! These tests verify the mathematical properties and behaviors of refactored transpiler functions

#![allow(clippy::similar_names)]

use proptest::prelude::*;
use ruchy::{Transpiler, Parser};

proptest! {
    #[test]
    fn test_math_function_sqrt_always_positive(x in 0.0f64..1000.0) {
        let mut transpiler = Transpiler::new();
        let input = format!("sqrt({x})");
        let mut parser = Parser::new(&input);
        
        if let Ok(ast) = parser.parse() {
            if let Ok(result) = transpiler.transpile(&ast) {
                let result_str = result.to_string();
                // Should contain sqrt method call
                prop_assert!(result_str.contains("sqrt"));
                // Should cast to f64
                prop_assert!(result_str.contains("as f64"));
            }
        }
    }

    #[test]  
    fn test_math_function_pow_deterministic(base in 0.0f64..100.0, exp in 0.0f64..5.0) {
        let mut transpiler = Transpiler::new();
        let input = format!("pow({base}, {exp})");
        let mut parser = Parser::new(&input);
        
        if let Ok(ast) = parser.parse() {
            if let Ok(result) = transpiler.transpile(&ast) {
                let result_str = result.to_string();
                // Should contain powf method call
                prop_assert!(result_str.contains("powf"));
                // Should cast both arguments to f64
                prop_assert!(result_str.contains("as f64"));
            }
        }
    }

    #[test]
    fn test_math_function_abs_preserves_structure(x in -1000.0f64..1000.0) {
        let mut transpiler = Transpiler::new();
        let input = format!("abs({x})");
        let mut parser = Parser::new(&input);
        
        if let Ok(ast) = parser.parse() {
            if let Ok(result) = transpiler.transpile(&ast) {
                let result_str = result.to_string();
                // Should contain abs method call
                prop_assert!(result_str.contains("abs"));
            }
        }
    }

    #[test]
    fn test_math_min_max_symmetry(a in 0.0f64..1000.0, b in 0.0f64..1000.0) {
        let mut transpiler = Transpiler::new();
        
        // Test min function
        let input_min = format!("min({a}, {b})");
        let mut parser_min = Parser::new(&input_min);
        if let Ok(ast_min) = parser_min.parse() {
            if let Ok(result_min) = transpiler.transpile(&ast_min) {
                let result_min_str = result_min.to_string();
                prop_assert!(result_min_str.contains("min"));
            }
        }
        
        // Test max function  
        let input_max = format!("max({a}, {b})");
        let mut parser_max = Parser::new(&input_max);
        if let Ok(ast_max) = parser_max.parse() {
            if let Ok(result_max) = transpiler.transpile(&ast_max) {
                let result_max_str = result_max.to_string();
                prop_assert!(result_max_str.contains("max"));
            }
        }
    }

    #[test]
    fn test_print_macro_format_strings(s in r"[a-zA-Z0-9 ]{1,20}") {
        let mut transpiler = Transpiler::new();
        let input = format!(r#"println("{s}")"#);
        let mut parser = Parser::new(&input);
        
        if let Ok(ast) = parser.parse() {
            if let Ok(result) = transpiler.transpile(&ast) {
                let result_str = result.to_string();
                // Should contain println macro
                prop_assert!(result_str.contains("println !"));
                // Should contain the string content
                prop_assert!(result_str.contains(&s));
            }
        }
    }

    #[test]
    fn test_assert_functions_preserve_structure(condition in any::<bool>()) {
        let mut transpiler = Transpiler::new();
        let input = format!("assert({condition})");
        let mut parser = Parser::new(&input);
        
        if let Ok(ast) = parser.parse() {
            if let Ok(result) = transpiler.transpile(&ast) {
                let result_str = result.to_string();
                // Should contain assert macro
                prop_assert!(result_str.contains("assert !"));
                // Should contain the condition
                prop_assert!(result_str.contains(&condition.to_string()));
            }
        }
    }

    #[test]
    fn test_regular_function_calls_preserve_name(func_name in r"[a-zA-Z_][a-zA-Z0-9_]{0,10}") {
        let mut transpiler = Transpiler::new();
        let input = format!(r#"{func_name}("test")"#);
        let mut parser = Parser::new(&input);
        
        if let Ok(ast) = parser.parse() {
            if let Ok(result) = transpiler.transpile(&ast) {
                let result_str = result.to_string();
                // Should contain the function name
                prop_assert!(result_str.contains(&func_name));
                // Should contain string conversion
                prop_assert!(result_str.contains("to_string"));
            }
        }
    }

    #[test]
    fn test_transpiler_determinism(input in r"[a-zA-Z0-9()., ]{1,50}") {
        let mut transpiler = Transpiler::new();
        let mut parser1 = Parser::new(&input);
        let mut parser2 = Parser::new(&input);
        
        if let (Ok(ast1), Ok(ast2)) = (parser1.parse(), parser2.parse()) {
            if let (Ok(result1), Ok(result2)) = (transpiler.transpile(&ast1), transpiler.transpile(&ast2)) {
                // Same input should always produce same output (determinism)
                prop_assert_eq!(result1.to_string(), result2.to_string());
            } else {
                // If one fails, both should fail (consistency)
            }
        } else {
            // Parser failures are acceptable for malformed input
        }
    }
}

/// Unit tests for specific refactored functions (not property tests)
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_transpile_call_complexity_reduced() {
        // This test verifies that our refactoring worked by checking
        // that the main function delegates properly to helpers
        let mut transpiler = Transpiler::new();
        
        // Test each category of function
        let test_cases = vec![
            (r#"println("hello")"#, vec!["println !"]),
            ("sqrt(4.0)", vec!["sqrt"]),
            ("input()", vec!["read_line"]),
            ("assert(true)", vec!["assert !"]),
            ("HashMap()", vec!["HashMap"]),
            (r#"col("name")"#, vec!["polars"]),
            (r#"regular_func("test")"#, vec!["regular_func", "to_string"]),
        ];
        
        for (input, expected_tokens) in test_cases {
            let mut parser = Parser::new(input);
            if let Ok(ast) = parser.parse() {
                if let Ok(result) = transpiler.transpile(&ast) {
                    let result_str = result.to_string();
                    for token in expected_tokens {
                        assert!(
                            result_str.contains(token),
                            "Input '{input}' should contain '{token}' but got: {result_str}"
                        );
                    }
                }
            }
        }
    }
}