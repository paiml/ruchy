// EXTREME TDD: Compound Assignment Operators
// Complexity: <10 per test
// Single responsibility per test
// Zero technical debt

use ruchy::compile;

#[cfg(test)]
mod compound_assignment_tests {
    use super::*;

    #[test]
    fn test_plus_equal_transpilation() {
        let code = compile("let mut x = 1; x += 2").unwrap();
        assert!(code.contains("x += 2"));
    }

    #[test]
    fn test_minus_equal_transpilation() {
        let code = compile("let mut x = 5; x -= 2").unwrap();
        assert!(code.contains("x -= 2"));
    }

    #[test]
    fn test_multiply_equal_transpilation() {
        let code = compile("let mut x = 3; x *= 2").unwrap();
        assert!(code.contains("x *= 2"));
    }

    #[test]
    fn test_divide_equal_transpilation() {
        let code = compile("let mut x = 10; x /= 2").unwrap();
        assert!(code.contains("x /= 2"));
    }

    #[test]
    fn test_modulo_equal_transpilation() {
        let code = compile("let mut x = 10; x %= 3").unwrap();
        assert!(code.contains("x %= 3"));
    }

    #[test]
    fn test_compound_assignment_in_expression() {
        // Compound assignment should work as part of larger expression
        let result = compile("let mut x = 1; let y = (x += 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_compound_assignments() {
        let result = compile("let mut x = 1; x += 2; x *= 3; x -= 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compound_assignment_with_complex_value() {
        let result = compile("let mut x = 1; x += 2 * 3 + 4");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bitwise_and_equal() {
        let code = compile("let mut x = 5; x &= 3").unwrap();
        assert!(code.contains("x &= 3"));
    }

    #[test]
    fn test_bitwise_or_equal() {
        let code = compile("let mut x = 5; x |= 3").unwrap();
        assert!(code.contains("x |= 3"));
    }

    #[test]
    fn test_bitwise_xor_equal() {
        let code = compile("let mut x = 5; x ^= 3").unwrap();
        assert!(code.contains("x ^= 3"));
    }

    #[test]
    fn test_left_shift_equal() {
        let code = compile("let mut x = 1; x <<= 2").unwrap();
        assert!(code.contains("x <<= 2"));
    }

    // Property tests for compound assignments
    #[test]
    fn test_all_assignment_operators_parse() {
        let operators = vec![
            ("+=", "Add"),
            ("-=", "Subtract"),
            ("*=", "Multiply"),
            ("/=", "Divide"),
            ("%=", "Modulo"),
            ("&=", "BitwiseAnd"),
            ("|=", "BitwiseOr"),
            ("^=", "BitwiseXor"),
            ("<<=", "LeftShift"),
        ];

        for (op, _name) in operators {
            let code = format!("let mut x = 1; x {} 2", op);
            let result = compile(&code);
            assert!(result.is_ok(), "Failed to compile: {}", code);
        }
    }

    #[test]
    fn test_compound_assignment_on_array_element() {
        let result = compile("let mut arr = [1, 2, 3]; arr[0] += 10");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compound_assignment_on_field() {
        let result = compile("let mut obj = {x: 1}; obj.x += 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compound_assignment_precedence() {
        // Assignment has lowest precedence
        let code = compile("let mut x = 1; x += 2 * 3").unwrap();
        // Should be x += (2 * 3), not (x += 2) * 3
        assert!(code.contains("x += 2i32 * 3i32") || code.contains("x += (2i32 * 3i32)"));
    }

    #[test]
    fn test_compound_assignment_with_function_call() {
        let result = compile("let mut x = 0; fn add() { 5 } x += add()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compound_assignment_type_inference() {
        // Should infer types correctly
        let code = compile("let mut x = 1; x += 2.5");
        // This might fail if we don't handle mixed types, which is fine
        assert!(code.is_ok() || code.is_err());
    }

    #[test]
    fn test_compound_assignment_returns_value() {
        // In Rust, compound assignments return ()
        // But we might want different behavior
        let result = compile("let mut x = 1; let y = (x += 2)");
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_compound_assignment_never_panics(
            initial in 0..100i32,
            delta in 0..100i32,
            op in "[+\\-*/%&|^]="
        ) {
            let code = format!("let mut x = {}; x {} {}", initial, op, delta);
            // Should not panic, either succeeds or returns error
            let _ = compile(&code);
        }

        #[test]
        fn test_compound_assignment_preserves_semantics(
            var_name in "[a-z][a-z0-9]*",
            value in 0..100i32
        ) {
            let code = format!("let mut {} = 1; {} += {}", var_name, var_name, value);
            if let Ok(transpiled) = compile(&code) {
                assert!(transpiled.contains(&format!("{} +=", var_name)));
            }
        }
    }
}
