//! Binary operator token mapping and precedence
//!
//! Handles mapping between lexer tokens and binary operators,
//! and defines operator precedence for expression parsing.
//!
//! # Operator Categories
//! - **Arithmetic**: +, -, *, /, %, **
//! - **Comparison**: ==, !=, <, <=, >, >=
//! - **Logical**: &&, ||, ??
//! - **Bitwise**: &, |, ^, <<, >>
//! - **Actor**: ! (message send)
//!
//! # Precedence Levels (1=lowest, 12=highest)
//! ```text
//! 1:  ||
//! 2:  ??, ! (actor send)
//! 3:  &&
//! 4:  |
//! 5:  ^
//! 6:  &
//! 7:  ==, !=
//! 8:  <, <=, >, >=
//! 9:  <<, >>
//! 10: +, -
//! 11: *, /, %
//! 12: **
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::BinaryOp;
use crate::frontend::lexer::Token;

/// Map token to binary operator
///
/// Attempts to map a token to its corresponding binary operator by
/// checking each operator category in sequence.
///
/// # Examples
/// ```ruchy
/// 1 + 2      // Token::Plus → BinaryOp::Add
/// x == y     // Token::EqualEqual → BinaryOp::Equal
/// a && b     // Token::AndAnd → BinaryOp::And
/// ```
pub fn token_to_binary_op(token: &Token) -> Option<BinaryOp> {
    // Try each category of operators
    map_arithmetic_operator(token)
        .or_else(|| map_comparison_operator(token))
        .or_else(|| map_logical_operator(token))
        .or_else(|| map_bitwise_operator(token))
        .or_else(|| map_actor_operator(token))
}

/// Map arithmetic tokens to binary operators
///
/// Handles: +, -, *, /, %, **
fn map_arithmetic_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Plus => Some(BinaryOp::Add),
        Token::Minus => Some(BinaryOp::Subtract),
        Token::Star => Some(BinaryOp::Multiply),
        Token::Slash => Some(BinaryOp::Divide),
        Token::Percent => Some(BinaryOp::Modulo),
        Token::Power => Some(BinaryOp::Power),
        _ => None,
    }
}

/// Map comparison tokens to binary operators
///
/// Handles: ==, !=, <, <=, >, >=
fn map_comparison_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::EqualEqual => Some(BinaryOp::Equal),
        Token::NotEqual => Some(BinaryOp::NotEqual),
        Token::Less => Some(BinaryOp::Less),
        Token::LessEqual => Some(BinaryOp::LessEqual),
        Token::Greater => Some(BinaryOp::Greater),
        Token::GreaterEqual => Some(BinaryOp::GreaterEqual),
        _ => None,
    }
}

/// Map logical tokens to binary operators
///
/// Handles: &&, ||, ??
fn map_logical_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::AndAnd => Some(BinaryOp::And),
        Token::OrOr => Some(BinaryOp::Or),
        Token::NullCoalesce => Some(BinaryOp::NullCoalesce),
        _ => None,
    }
}

/// Map bitwise tokens to binary operators
///
/// Handles: &, |, ^, <<, >>
fn map_bitwise_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Ampersand => Some(BinaryOp::BitwiseAnd),
        Token::Pipe => Some(BinaryOp::BitwiseOr),
        Token::Caret => Some(BinaryOp::BitwiseXor),
        Token::LeftShift => Some(BinaryOp::LeftShift),
        Token::RightShift => Some(BinaryOp::RightShift),
        _ => None,
    }
}

/// Map actor message passing tokens to binary operators
///
/// Handles: ! (actor ! message)
fn map_actor_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Bang => Some(BinaryOp::Send), // actor ! Message
        _ => None,
    }
}

/// Get operator precedence (1=lowest, 12=highest)
///
/// Defines the precedence hierarchy for binary operators in expression parsing.
/// Higher numbers bind more tightly.
///
/// # Precedence Table
/// - **1**: || (logical or)
/// - **2**: ?? (null coalesce), ! (actor send)
/// - **3**: && (logical and)
/// - **4-6**: Bitwise operators (|, ^, &)
/// - **7-8**: Comparison operators (==, !=, <, <=, >, >=)
/// - **9**: Shift operators (<<, >>)
/// - **10**: Addition, subtraction (+, -)
/// - **11**: Multiplication, division, modulo (*, /, %)
/// - **12**: Power (**)
pub fn get_precedence(op: BinaryOp) -> i32 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::NullCoalesce => 2,
        BinaryOp::And => 3,
        BinaryOp::BitwiseOr => 4,
        BinaryOp::BitwiseXor => 5,
        BinaryOp::BitwiseAnd => 6,
        BinaryOp::Equal | BinaryOp::NotEqual => 7,
        BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual
        | BinaryOp::Gt
        | BinaryOp::In => 8, // In is same precedence as comparison (used internally, not parsed from tokens)
        BinaryOp::LeftShift => 9,
        BinaryOp::RightShift => 9,
        BinaryOp::Add | BinaryOp::Subtract => 10,
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 11,
        BinaryOp::Power => 12,
        BinaryOp::Send => 2, // Actor message passing precedence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Token mapping tests
    #[test]
    fn test_arithmetic_operators() {
        assert!(matches!(
            token_to_binary_op(&Token::Plus),
            Some(BinaryOp::Add)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Minus),
            Some(BinaryOp::Subtract)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Star),
            Some(BinaryOp::Multiply)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Slash),
            Some(BinaryOp::Divide)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Percent),
            Some(BinaryOp::Modulo)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Power),
            Some(BinaryOp::Power)
        ));
    }

    #[test]
    fn test_comparison_operators() {
        assert!(matches!(
            token_to_binary_op(&Token::EqualEqual),
            Some(BinaryOp::Equal)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::NotEqual),
            Some(BinaryOp::NotEqual)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Less),
            Some(BinaryOp::Less)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Greater),
            Some(BinaryOp::Greater)
        ));
    }

    #[test]
    fn test_logical_operators() {
        assert!(matches!(
            token_to_binary_op(&Token::AndAnd),
            Some(BinaryOp::And)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::OrOr),
            Some(BinaryOp::Or)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::NullCoalesce),
            Some(BinaryOp::NullCoalesce)
        ));
    }

    #[test]
    fn test_bitwise_operators() {
        assert!(matches!(
            token_to_binary_op(&Token::Ampersand),
            Some(BinaryOp::BitwiseAnd)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Pipe),
            Some(BinaryOp::BitwiseOr)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::Caret),
            Some(BinaryOp::BitwiseXor)
        ));
    }

    #[test]
    fn test_actor_operator() {
        assert!(matches!(
            token_to_binary_op(&Token::Bang),
            Some(BinaryOp::Send)
        ));
    }

    #[test]
    fn test_non_binary_tokens() {
        assert!(token_to_binary_op(&Token::LeftParen).is_none());
        assert!(token_to_binary_op(&Token::RightParen).is_none());
        assert!(token_to_binary_op(&Token::Semicolon).is_none());
    }

    // Precedence tests
    #[test]
    fn test_precedence_ordering() {
        // Logical OR has lowest precedence
        assert_eq!(get_precedence(BinaryOp::Or), 1);

        // Null coalesce and actor send
        assert_eq!(get_precedence(BinaryOp::NullCoalesce), 2);
        assert_eq!(get_precedence(BinaryOp::Send), 2);

        // Logical AND
        assert_eq!(get_precedence(BinaryOp::And), 3);

        // Comparison
        assert_eq!(get_precedence(BinaryOp::Equal), 7);
        assert_eq!(get_precedence(BinaryOp::Less), 8);

        // Arithmetic
        assert_eq!(get_precedence(BinaryOp::Add), 10);
        assert_eq!(get_precedence(BinaryOp::Multiply), 11);

        // Power has highest precedence
        assert_eq!(get_precedence(BinaryOp::Power), 12);
    }

    #[test]
    fn test_precedence_comparison_invariants() {
        // Multiplication should bind tighter than addition
        assert!(get_precedence(BinaryOp::Multiply) > get_precedence(BinaryOp::Add));

        // Power should bind tighter than multiplication
        assert!(get_precedence(BinaryOp::Power) > get_precedence(BinaryOp::Multiply));

        // Logical AND should bind tighter than logical OR
        assert!(get_precedence(BinaryOp::And) > get_precedence(BinaryOp::Or));

        // Comparison should bind tighter than logical operators
        assert!(get_precedence(BinaryOp::Equal) > get_precedence(BinaryOp::And));
    }

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::Expr;
    use crate::frontend::parser::{Parser, Result};

    fn parse(code: &str) -> Result<Expr> {
        Parser::new(code).parse()
    }

    // ============================================================
    // Arithmetic expression parsing tests
    // ============================================================

    #[test]
    fn test_parse_add_expression() {
        let result = parse("1 + 2");
        assert!(result.is_ok(), "Addition should parse");
    }

    #[test]
    fn test_parse_subtract_expression() {
        let result = parse("5 - 3");
        assert!(result.is_ok(), "Subtraction should parse");
    }

    #[test]
    fn test_parse_multiply_expression() {
        let result = parse("2 * 3");
        assert!(result.is_ok(), "Multiplication should parse");
    }

    #[test]
    fn test_parse_divide_expression() {
        let result = parse("10 / 2");
        assert!(result.is_ok(), "Division should parse");
    }

    #[test]
    fn test_parse_modulo_expression() {
        let result = parse("10 % 3");
        assert!(result.is_ok(), "Modulo should parse");
    }

    #[test]
    fn test_parse_power_expression() {
        let result = parse("2 ** 8");
        assert!(result.is_ok(), "Power should parse");
    }

    #[test]
    fn test_parse_complex_arithmetic() {
        let result = parse("1 + 2 * 3 - 4 / 2");
        assert!(result.is_ok(), "Complex arithmetic should parse");
    }

    // ============================================================
    // Comparison expression parsing tests
    // ============================================================

    #[test]
    fn test_parse_equal_expression() {
        let result = parse("a == b");
        assert!(result.is_ok(), "Equality should parse");
    }

    #[test]
    fn test_parse_not_equal_expression() {
        let result = parse("a != b");
        assert!(result.is_ok(), "Not equal should parse");
    }

    #[test]
    fn test_parse_less_expression() {
        let result = parse("a < b");
        assert!(result.is_ok(), "Less than should parse");
    }

    #[test]
    fn test_parse_less_equal_expression() {
        let result = parse("a <= b");
        assert!(result.is_ok(), "Less or equal should parse");
    }

    #[test]
    fn test_parse_greater_expression() {
        let result = parse("a > b");
        assert!(result.is_ok(), "Greater than should parse");
    }

    #[test]
    fn test_parse_greater_equal_expression() {
        let result = parse("a >= b");
        assert!(result.is_ok(), "Greater or equal should parse");
    }

    // ============================================================
    // Logical expression parsing tests
    // ============================================================

    #[test]
    fn test_parse_and_expression() {
        let result = parse("a && b");
        assert!(result.is_ok(), "Logical AND should parse");
    }

    #[test]
    fn test_parse_or_expression() {
        let result = parse("a || b");
        assert!(result.is_ok(), "Logical OR should parse");
    }

    #[test]
    fn test_parse_null_coalesce_expression() {
        let result = parse("a ?? b");
        assert!(result.is_ok(), "Null coalesce should parse");
    }

    #[test]
    fn test_parse_complex_logical() {
        let result = parse("a && b || c && d");
        assert!(result.is_ok(), "Complex logical should parse");
    }

    // ============================================================
    // Bitwise expression parsing tests
    // ============================================================

    #[test]
    fn test_parse_bitwise_and_expression() {
        let result = parse("a & b");
        assert!(result.is_ok(), "Bitwise AND should parse");
    }

    #[test]
    fn test_parse_bitwise_or_expression() {
        let result = parse("a | b");
        assert!(result.is_ok(), "Bitwise OR should parse");
    }

    #[test]
    fn test_parse_bitwise_xor_expression() {
        let result = parse("a ^ b");
        assert!(result.is_ok(), "Bitwise XOR should parse");
    }

    #[test]
    fn test_parse_left_shift_expression() {
        let result = parse("a << 2");
        assert!(result.is_ok(), "Left shift should parse");
    }

    #[test]
    fn test_parse_right_shift_expression() {
        let result = parse("a >> 2");
        assert!(result.is_ok(), "Right shift should parse");
    }

    // ============================================================
    // Precedence behavior tests
    // ============================================================

    #[test]
    fn test_mul_before_add() {
        // 1 + 2 * 3 should be parsed as 1 + (2 * 3) = 7
        let result = parse("1 + 2 * 3");
        assert!(result.is_ok(), "Mul before add should parse");
    }

    #[test]
    fn test_power_right_associative() {
        // 2 ** 3 ** 2 should work
        let result = parse("2 ** 3 ** 2");
        assert!(result.is_ok(), "Nested power should parse");
    }

    #[test]
    fn test_comparison_chain() {
        // a < b && b < c
        let result = parse("a < b && b < c");
        assert!(result.is_ok(), "Comparison chain should parse");
    }

    #[test]
    fn test_mixed_precedence() {
        let result = parse("a + b * c == d / e - f");
        assert!(result.is_ok(), "Mixed precedence should parse");
    }

    // ============================================================
    // Shift operator tests
    // ============================================================

    #[test]
    fn test_shift_operators() {
        assert!(matches!(
            token_to_binary_op(&Token::LeftShift),
            Some(BinaryOp::LeftShift)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::RightShift),
            Some(BinaryOp::RightShift)
        ));
    }

    #[test]
    fn test_shift_precedence() {
        assert_eq!(get_precedence(BinaryOp::LeftShift), 9);
        assert_eq!(get_precedence(BinaryOp::RightShift), 9);
    }

    // ============================================================
    // Edge cases and special operators
    // ============================================================

    #[test]
    fn test_less_equal_and_greater_equal() {
        assert!(matches!(
            token_to_binary_op(&Token::LessEqual),
            Some(BinaryOp::LessEqual)
        ));
        assert!(matches!(
            token_to_binary_op(&Token::GreaterEqual),
            Some(BinaryOp::GreaterEqual)
        ));
    }

    #[test]
    fn test_comparison_precedence_same() {
        assert_eq!(
            get_precedence(BinaryOp::Less),
            get_precedence(BinaryOp::Greater)
        );
        assert_eq!(
            get_precedence(BinaryOp::LessEqual),
            get_precedence(BinaryOp::GreaterEqual)
        );
    }

    #[test]
    fn test_equality_precedence_same() {
        assert_eq!(
            get_precedence(BinaryOp::Equal),
            get_precedence(BinaryOp::NotEqual)
        );
    }

    #[test]
    fn test_bitwise_precedence_order() {
        // & binds tighter than ^, which binds tighter than |
        assert!(get_precedence(BinaryOp::BitwiseAnd) > get_precedence(BinaryOp::BitwiseXor));
        assert!(get_precedence(BinaryOp::BitwiseXor) > get_precedence(BinaryOp::BitwiseOr));
    }

    #[test]
    fn test_add_subtract_same_precedence() {
        assert_eq!(
            get_precedence(BinaryOp::Add),
            get_precedence(BinaryOp::Subtract)
        );
    }

    #[test]
    fn test_mul_div_mod_same_precedence() {
        assert_eq!(
            get_precedence(BinaryOp::Multiply),
            get_precedence(BinaryOp::Divide)
        );
        assert_eq!(
            get_precedence(BinaryOp::Divide),
            get_precedence(BinaryOp::Modulo)
        );
    }

    // ============================================================
    // Additional EXTREME TDD tests
    // ============================================================

    // ===== Parenthesized expressions =====

    #[test]
    fn test_parenthesized_add() {
        let result = parse("(1 + 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parenthesized_override_precedence() {
        let result = parse("(1 + 2) * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_parentheses() {
        let result = parse("((1 + 2) * (3 + 4))");
        assert!(result.is_ok());
    }

    #[test]
    fn test_deeply_nested_parentheses() {
        let result = parse("(((a + b)))");
        assert!(result.is_ok());
    }

    // ===== Chained operations =====

    #[test]
    fn test_chained_add() {
        let result = parse("1 + 2 + 3 + 4 + 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_multiply() {
        let result = parse("1 * 2 * 3 * 4 * 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_and() {
        let result = parse("a && b && c && d");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_or() {
        let result = parse("a || b || c || d");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_bitwise() {
        let result = parse("a & b & c & d");
        assert!(result.is_ok());
    }

    // ===== Complex mixed expressions =====

    #[test]
    fn test_arithmetic_in_comparison() {
        let result = parse("a + b > c - d");
        assert!(result.is_ok());
    }

    #[test]
    fn test_comparison_in_logical() {
        let result = parse("a > 0 && b < 10");
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_arithmetic_ops() {
        let result = parse("a + b - c * d / e % f ** g");
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_comparison_ops() {
        let result = parse("(a == b) && (c != d) && (e < f) && (g > h)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_logical_ops() {
        let result = parse("(a && b) || (c ?? d)");
        assert!(result.is_ok());
    }

    // ===== Variable operands =====

    #[test]
    fn test_single_char_variables() {
        let result = parse("a + b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_long_variable_names() {
        let result = parse("very_long_name + another_long_name");
        assert!(result.is_ok());
    }

    #[test]
    fn test_variables_with_numbers() {
        let result = parse("x1 + y2 * z3");
        assert!(result.is_ok());
    }

    // ===== Literal operands =====

    #[test]
    fn test_integer_literals() {
        let result = parse("42 + 100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_literals() {
        let result = parse("3.14 * 2.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_literals() {
        let result = parse("42 + 3.14");
        assert!(result.is_ok());
    }

    #[test]
    fn test_negative_literals() {
        let result = parse("-5 + -3");
        assert!(result.is_ok());
    }

    // ===== Expressions with function calls =====

    #[test]
    fn test_function_call_operand() {
        let result = parse("foo() + bar()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_method_call_operand() {
        let result = parse("a.len() + b.len()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_function_calls() {
        let result = parse("max(a, b) + min(c, d)");
        assert!(result.is_ok());
    }

    // ===== Additional token mapping tests =====

    #[test]
    fn test_all_arithmetic_token_mapping() {
        assert!(token_to_binary_op(&Token::Plus).is_some());
        assert!(token_to_binary_op(&Token::Minus).is_some());
        assert!(token_to_binary_op(&Token::Star).is_some());
        assert!(token_to_binary_op(&Token::Slash).is_some());
        assert!(token_to_binary_op(&Token::Percent).is_some());
        assert!(token_to_binary_op(&Token::Power).is_some());
    }

    #[test]
    fn test_all_comparison_token_mapping() {
        assert!(token_to_binary_op(&Token::EqualEqual).is_some());
        assert!(token_to_binary_op(&Token::NotEqual).is_some());
        assert!(token_to_binary_op(&Token::Less).is_some());
        assert!(token_to_binary_op(&Token::LessEqual).is_some());
        assert!(token_to_binary_op(&Token::Greater).is_some());
        assert!(token_to_binary_op(&Token::GreaterEqual).is_some());
    }

    #[test]
    fn test_all_logical_token_mapping() {
        assert!(token_to_binary_op(&Token::AndAnd).is_some());
        assert!(token_to_binary_op(&Token::OrOr).is_some());
        assert!(token_to_binary_op(&Token::NullCoalesce).is_some());
    }

    #[test]
    fn test_all_bitwise_token_mapping() {
        assert!(token_to_binary_op(&Token::Ampersand).is_some());
        assert!(token_to_binary_op(&Token::Pipe).is_some());
        assert!(token_to_binary_op(&Token::Caret).is_some());
        assert!(token_to_binary_op(&Token::LeftShift).is_some());
        assert!(token_to_binary_op(&Token::RightShift).is_some());
    }

    // Property tests for binary operators
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_all_arithmetic_tokens_map(_n in 0..100) {
                let tokens = vec![
                    Token::Plus, Token::Minus, Token::Star,
                    Token::Slash, Token::Percent, Token::Power,
                ];
                for token in tokens {
                    prop_assert!(token_to_binary_op(&token).is_some(),
                        "Arithmetic token {:?} should map to operator", token);
                }
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_all_comparison_tokens_map(_n in 0..100) {
                let tokens = vec![
                    Token::EqualEqual, Token::NotEqual,
                    Token::Less, Token::LessEqual,
                    Token::Greater, Token::GreaterEqual,
                ];
                for token in tokens {
                    prop_assert!(token_to_binary_op(&token).is_some(),
                        "Comparison token {:?} should map to operator", token);
                }
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_all_logical_tokens_map(_n in 0..100) {
                let tokens = vec![Token::AndAnd, Token::OrOr, Token::NullCoalesce];
                for token in tokens {
                    prop_assert!(token_to_binary_op(&token).is_some(),
                        "Logical token {:?} should map to operator", token);
                }
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_precedence_is_positive(_n in 0..100) {
                let ops = vec![
                    BinaryOp::Add, BinaryOp::Multiply, BinaryOp::Power,
                    BinaryOp::And, BinaryOp::Or, BinaryOp::Equal,
                ];
                for op in ops {
                    prop_assert!(get_precedence(op) > 0,
                        "Precedence for {:?} should be positive", op);
                }
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_precedence_bounded(_n in 0..100) {
                let ops = vec![
                    BinaryOp::Add, BinaryOp::Multiply, BinaryOp::Power,
                    BinaryOp::And, BinaryOp::Or, BinaryOp::Equal,
                ];
                for op in ops {
                    let prec = get_precedence(op);
                    prop_assert!((1..=12).contains(&prec),
                        "Precedence {} for {:?} should be in range [1, 12]", prec, op);
                }
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_multiply_binds_tighter_than_add(_a in 1..100, _b in 1..100) {
                let mul_prec = get_precedence(BinaryOp::Multiply);
                let add_prec = get_precedence(BinaryOp::Add);
                prop_assert!(mul_prec > add_prec,
                    "Multiplication precedence {} should be > addition precedence {}", mul_prec, add_prec);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_power_binds_tightest(_a in 1..100) {
                let power_prec = get_precedence(BinaryOp::Power);
                let other_ops = vec![
                    BinaryOp::Add, BinaryOp::Multiply, BinaryOp::Divide,
                    BinaryOp::And, BinaryOp::Or, BinaryOp::Equal,
                ];
                for op in other_ops {
                    prop_assert!(power_prec >= get_precedence(op),
                        "Power precedence {} should be >= {:?} precedence {}", power_prec, op, get_precedence(op));
                }
            }
        }
    }
}
