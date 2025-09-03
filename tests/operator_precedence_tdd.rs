//! Comprehensive TDD test suite for operator_precedence.rs
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every operator precedence path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::frontend::Token;
use ruchy::frontend::parser::operator_precedence::{
    get_operator_info, is_postfix_operator, is_prefix_operator,
    get_postfix_precedence, should_continue_parsing,
    Precedence, Associativity
};

// ==================== ASSIGNMENT OPERATOR TESTS ====================

#[test]
fn test_precedence_assignment_equal() {
    let info = get_operator_info(&Token::Equal);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::ASSIGNMENT);
    assert_eq!(assoc, Associativity::Right);
}

#[test]
fn test_precedence_plus_equal() {
    let info = get_operator_info(&Token::PlusEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::ASSIGNMENT);
    assert_eq!(assoc, Associativity::Right);
}

#[test]
fn test_precedence_minus_equal() {
    let info = get_operator_info(&Token::MinusEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::ASSIGNMENT);
    assert_eq!(assoc, Associativity::Right);
}

// ==================== PIPELINE OPERATOR TESTS ====================

#[test]
fn test_precedence_pipeline() {
    let info = get_operator_info(&Token::Pipeline);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::PIPELINE);
    assert_eq!(assoc, Associativity::Left);
}

// ==================== LOGICAL OPERATOR TESTS ====================

#[test]
fn test_precedence_logical_or() {
    let info = get_operator_info(&Token::OrOr);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::LOGICAL_OR);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_logical_and() {
    let info = get_operator_info(&Token::AndAnd);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::LOGICAL_AND);
    assert_eq!(assoc, Associativity::Left);
}

// ==================== EQUALITY OPERATOR TESTS ====================

#[test]
fn test_precedence_equal_equal() {
    let info = get_operator_info(&Token::EqualEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::EQUALITY);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_not_equal() {
    let info = get_operator_info(&Token::NotEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::EQUALITY);
    assert_eq!(assoc, Associativity::Left);
}

// ==================== COMPARISON OPERATOR TESTS ====================

#[test]
fn test_precedence_less_than() {
    let info = get_operator_info(&Token::Less);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::COMPARISON);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_greater_than() {
    let info = get_operator_info(&Token::Greater);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::COMPARISON);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_less_equal() {
    let info = get_operator_info(&Token::LessEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::COMPARISON);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_greater_equal() {
    let info = get_operator_info(&Token::GreaterEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::COMPARISON);
    assert_eq!(assoc, Associativity::Left);
}

// ==================== BITWISE OPERATOR TESTS ====================

#[test]
fn test_precedence_bitwise_or() {
    let info = get_operator_info(&Token::Pipe);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::BITWISE_OR);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_bitwise_xor() {
    let info = get_operator_info(&Token::Caret);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::BITWISE_XOR);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_bitwise_and() {
    let info = get_operator_info(&Token::Ampersand);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::BITWISE_AND);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_left_shift() {
    let info = get_operator_info(&Token::LeftShift);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::SHIFT);
    assert_eq!(assoc, Associativity::Left);
}

// ==================== RANGE OPERATOR TESTS ====================

#[test]
fn test_precedence_range() {
    let info = get_operator_info(&Token::DotDot);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::RANGE);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_range_inclusive() {
    let info = get_operator_info(&Token::DotDotEqual);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::RANGE);
    assert_eq!(assoc, Associativity::Left);
}

// ==================== ARITHMETIC OPERATOR TESTS ====================

#[test]
fn test_precedence_plus() {
    let info = get_operator_info(&Token::Plus);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::ADDITIVE);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_minus() {
    let info = get_operator_info(&Token::Minus);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::ADDITIVE);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_star() {
    let info = get_operator_info(&Token::Star);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::MULTIPLICATIVE);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_slash() {
    let info = get_operator_info(&Token::Slash);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::MULTIPLICATIVE);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_percent() {
    let info = get_operator_info(&Token::Percent);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::MULTIPLICATIVE);
    assert_eq!(assoc, Associativity::Left);
}

#[test]
fn test_precedence_power() {
    let info = get_operator_info(&Token::Power);
    assert!(info.is_some());
    let (prec, assoc) = info.unwrap();
    assert_eq!(prec, Precedence::POWER);
    assert_eq!(assoc, Associativity::Right);
}

// ==================== PREFIX OPERATOR TESTS ====================

#[test]
fn test_is_prefix_bang() {
    assert!(is_prefix_operator(&Token::Bang));
}

#[test]
fn test_is_prefix_minus() {
    assert!(is_prefix_operator(&Token::Minus));
}

#[test]
fn test_is_prefix_tilde() {
    assert!(is_prefix_operator(&Token::Tilde));
}

#[test]
fn test_is_prefix_ampersand() {
    assert!(is_prefix_operator(&Token::Ampersand));
}

#[test]
fn test_is_prefix_star() {
    assert!(is_prefix_operator(&Token::Star));
}

#[test]
fn test_is_not_prefix_plus() {
    assert!(!is_prefix_operator(&Token::Plus));
}

// ==================== POSTFIX OPERATOR TESTS ====================

#[test]
fn test_is_postfix_question() {
    assert!(is_postfix_operator(&Token::Question));
}

#[test]
fn test_is_postfix_dot() {
    assert!(is_postfix_operator(&Token::Dot));
}

#[test]
fn test_is_postfix_safe_nav() {
    assert!(is_postfix_operator(&Token::SafeNav));
}

#[test]
fn test_is_postfix_left_paren() {
    assert!(is_postfix_operator(&Token::LeftParen));
}

#[test]
fn test_is_postfix_left_bracket() {
    assert!(is_postfix_operator(&Token::LeftBracket));
}

#[test]
fn test_is_not_postfix_plus() {
    assert!(!is_postfix_operator(&Token::Plus));
}

// ==================== POSTFIX PRECEDENCE TESTS ====================

#[test]
fn test_postfix_precedence_dot() {
    assert_eq!(get_postfix_precedence(&Token::Dot), Precedence::MEMBER);
}

#[test]
fn test_postfix_precedence_safe_nav() {
    assert_eq!(get_postfix_precedence(&Token::SafeNav), Precedence::MEMBER);
}

#[test]
fn test_postfix_precedence_left_paren() {
    assert_eq!(get_postfix_precedence(&Token::LeftParen), Precedence::CALL);
}

#[test]
fn test_postfix_precedence_left_bracket() {
    assert_eq!(get_postfix_precedence(&Token::LeftBracket), Precedence::CALL);
}

#[test]
fn test_postfix_precedence_question() {
    assert_eq!(get_postfix_precedence(&Token::Question), Precedence::POSTFIX);
}

// ==================== PRECEDENCE ORDERING TESTS ====================

#[test]
fn test_precedence_ordering() {
    // Verify that precedence levels are correctly ordered
    assert!(Precedence::ASSIGNMENT < Precedence::PIPELINE);
    assert!(Precedence::PIPELINE < Precedence::LOGICAL_OR);
    assert!(Precedence::LOGICAL_OR < Precedence::LOGICAL_AND);
    assert!(Precedence::LOGICAL_AND < Precedence::EQUALITY);
    assert!(Precedence::EQUALITY < Precedence::COMPARISON);
    assert!(Precedence::COMPARISON < Precedence::BITWISE_OR);
    assert!(Precedence::BITWISE_OR < Precedence::BITWISE_XOR);
    assert!(Precedence::BITWISE_XOR < Precedence::BITWISE_AND);
    assert!(Precedence::BITWISE_AND < Precedence::SHIFT);
    assert!(Precedence::SHIFT < Precedence::RANGE);
    assert!(Precedence::RANGE < Precedence::ADDITIVE);
    assert!(Precedence::ADDITIVE < Precedence::MULTIPLICATIVE);
    assert!(Precedence::MULTIPLICATIVE < Precedence::POWER);
    assert!(Precedence::POWER < Precedence::UNARY);
    assert!(Precedence::UNARY < Precedence::POSTFIX);
    assert!(Precedence::POSTFIX < Precedence::CALL);
    assert!(Precedence::CALL < Precedence::MEMBER);
}

// ==================== SHOULD CONTINUE PARSING TESTS ====================

#[test]
fn test_should_continue_parsing_higher_precedence() {
    // Should continue if operator has higher precedence
    assert!(should_continue_parsing(&Token::Star, Precedence::ADDITIVE));
}

#[test]
fn test_should_continue_parsing_equal_precedence() {
    // Should continue if operator has equal precedence
    assert!(should_continue_parsing(&Token::Plus, Precedence::ADDITIVE));
}

#[test]
fn test_should_not_continue_parsing_lower_precedence() {
    // Should not continue if operator has lower precedence
    assert!(!should_continue_parsing(&Token::Plus, Precedence::MULTIPLICATIVE));
}

#[test]
fn test_should_not_continue_parsing_non_operator() {
    // Should not continue for non-operators
    assert!(!should_continue_parsing(&Token::Identifier("x".to_string()), Precedence::ADDITIVE));
}

// ==================== UNKNOWN TOKEN TESTS ====================

#[test]
fn test_unknown_token_returns_none() {
    let info = get_operator_info(&Token::Identifier("foo".to_string()));
    assert!(info.is_none());
}

#[test]
fn test_unknown_postfix_returns_zero() {
    assert_eq!(get_postfix_precedence(&Token::Identifier("foo".to_string())), Precedence(0));
}

// Run all tests with: cargo test operator_precedence_tdd --test operator_precedence_tdd