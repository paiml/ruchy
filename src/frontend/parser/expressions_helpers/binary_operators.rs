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
        | BinaryOp::Gt => 8,
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

    // Property tests for binary operators
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
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
            #[ignore]
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
            #[ignore]
            fn prop_all_logical_tokens_map(_n in 0..100) {
                let tokens = vec![Token::AndAnd, Token::OrOr, Token::NullCoalesce];
                for token in tokens {
                    prop_assert!(token_to_binary_op(&token).is_some(),
                        "Logical token {:?} should map to operator", token);
                }
            }

            #[test]
            #[ignore]
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
            #[ignore]
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
            #[ignore]
            fn prop_multiply_binds_tighter_than_add(_a in 1..100, _b in 1..100) {
                let mul_prec = get_precedence(BinaryOp::Multiply);
                let add_prec = get_precedence(BinaryOp::Add);
                prop_assert!(mul_prec > add_prec,
                    "Multiplication precedence {} should be > addition precedence {}", mul_prec, add_prec);
            }

            #[test]
            #[ignore]
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
