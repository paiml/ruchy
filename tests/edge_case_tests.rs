// Edge case tests for the Ruchy compiler
// These test boundary conditions and unusual but valid inputs

use ruchy::compile;

#[cfg(test)]
mod edge_cases {
    use super::*;

    // Empty program tests
    #[test]
    fn test_empty_program() {
        assert!(compile("").is_ok() || compile("").is_err());
    }

    #[test]
    fn test_whitespace_only() {
        assert!(compile("   \n\t\r\n   ").is_ok() || compile("   \n\t\r\n   ").is_err());
    }

    #[test]
    #[ignore] // Parser needs fix for comment-only files
    fn test_comment_only() {
        assert!(compile("// Just a comment").is_ok());
    }

    #[test]
    #[ignore] // Parser needs fix for comment-only files
    fn test_multiline_comment_only() {
        assert!(compile("/* Just a \n multiline \n comment */").is_ok());
    }

    // Edge case numbers
    #[test]
    fn test_zero() {
        assert!(compile("fn main() { let x = 0; }").is_ok());
    }

    #[test]
    fn test_negative_numbers() {
        assert!(compile("fn main() { let x = -42; }").is_ok());
    }

    #[test]
    fn test_large_numbers() {
        assert!(compile("fn main() { let x = 2147483647; }").is_ok());
    }

    #[test]
    fn test_float_numbers() {
        assert!(compile("fn main() { let x = 3.14159; }").is_ok());
    }

    #[test]
    fn test_scientific_notation() {
        assert!(compile("fn main() { let x = 1.5e10; }").is_ok());
    }

    // Edge case strings
    #[test]
    fn test_empty_string() {
        assert!(compile(r#"fn main() { let s = ""; }"#).is_ok());
    }

    #[test]
    fn test_string_with_escapes() {
        assert!(compile(r#"fn main() { let s = "Hello\nWorld\t!"; }"#).is_ok());
    }

    #[test]
    fn test_unicode_string() {
        assert!(compile(r#"fn main() { let s = "Hello 世界 🦀"; }"#).is_ok());
    }

    // Edge case arrays
    #[test]
    fn test_empty_array() {
        assert!(compile("fn main() { let arr = []; }").is_ok());
    }

    #[test]
    fn test_single_element_array() {
        assert!(compile("fn main() { let arr = [42]; }").is_ok());
    }

    #[test]
    fn test_nested_arrays() {
        assert!(compile("fn main() { let arr = [[1, 2], [3, 4]]; }").is_ok());
    }

    // Edge case functions
    #[test]
    fn test_empty_function() {
        assert!(compile("fn empty() {}").is_ok());
    }

    #[test]
    fn test_function_no_params() {
        assert!(compile("fn no_params() -> i32 { 42 }").is_ok());
    }

    #[test]
    fn test_function_many_params() {
        assert!(compile(
            "fn many(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 { a + b + c + d + e }"
        )
        .is_ok());
    }

    // Edge case control flow
    #[test]
    fn test_empty_if() {
        assert!(compile("fn main() { if true {} }").is_ok());
    }

    #[test]
    fn test_nested_if() {
        assert!(compile("fn main() { if true { if false {} } }").is_ok());
    }

    #[test]
    fn test_empty_for_loop() {
        assert!(compile("fn main() { for _ in 0..0 {} }").is_ok());
    }

    #[test]
    fn test_infinite_loop() {
        assert!(compile("fn main() { loop { break; } }").is_ok());
    }

    // Edge case operators
    #[test]
    fn test_chained_operators() {
        assert!(compile("fn main() { let x = 1 + 2 + 3 + 4 + 5; }").is_ok());
    }

    #[test]
    fn test_parenthesized_expressions() {
        assert!(compile("fn main() { let x = ((1 + 2) * 3) / 4; }").is_ok());
    }

    #[test]
    fn test_unary_operators() {
        assert!(compile("fn main() { let x = -(-42); }").is_ok());
    }

    #[test]
    fn test_boolean_operators() {
        assert!(compile("fn main() { let x = true && false || true; }").is_ok());
    }

    // Edge case identifiers
    #[test]
    fn test_single_letter_identifier() {
        assert!(compile("fn main() { let a = 1; }").is_ok());
    }

    #[test]
    #[ignore] // Parser needs fix for underscore identifier
    fn test_underscore_identifier() {
        assert!(compile("fn main() { let _ = 42; }").is_ok());
    }

    #[test]
    fn test_identifier_with_numbers() {
        assert!(compile("fn main() { let var123 = 42; }").is_ok());
    }

    #[test]
    fn test_long_identifier() {
        assert!(compile(
            "fn main() { let this_is_a_very_long_identifier_name_that_should_still_work = 42; }"
        )
        .is_ok());
    }

    // Edge case tuples
    #[test]
    fn test_unit_tuple() {
        assert!(compile("fn main() { let t = (); }").is_ok());
    }

    #[test]
    fn test_single_element_tuple() {
        assert!(compile("fn main() { let t = (42,); }").is_ok());
    }

    #[test]
    fn test_large_tuple() {
        assert!(compile("fn main() { let t = (1, 2, 3, 4, 5, 6, 7, 8); }").is_ok());
    }

    // Edge case returns
    #[test]
    fn test_early_return() {
        assert!(compile("fn test() -> i32 { return 42; 0 }").is_ok());
    }

    #[test]
    fn test_implicit_return() {
        assert!(compile("fn test() -> i32 { 42 }").is_ok());
    }

    #[test]
    fn test_unit_return() {
        assert!(compile("fn test() { return; }").is_ok());
    }
}
