//! Operator precedence handling for the Ruchy parser
//!
//! This module defines operator precedence and associativity rules
//! ensuring proper parsing of complex expressions.

#![allow(dead_code)]

use crate::frontend::lexer::Token;

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

/// Operator precedence levels (higher = tighter binding)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precedence(pub i32);

impl Precedence {
    // Precedence levels from lowest to highest
    pub const ASSIGNMENT: Precedence = Precedence(10);
    pub const PIPELINE: Precedence = Precedence(20);
    pub const LOGICAL_OR: Precedence = Precedence(30);
    pub const LOGICAL_AND: Precedence = Precedence(40);
    pub const EQUALITY: Precedence = Precedence(50);
    pub const COMPARISON: Precedence = Precedence(60);
    pub const BITWISE_OR: Precedence = Precedence(70);
    pub const BITWISE_XOR: Precedence = Precedence(80);
    pub const BITWISE_AND: Precedence = Precedence(90);
    pub const SHIFT: Precedence = Precedence(100);
    pub const RANGE: Precedence = Precedence(110);
    pub const ADDITIVE: Precedence = Precedence(120);
    pub const MULTIPLICATIVE: Precedence = Precedence(130);
    pub const POWER: Precedence = Precedence(140);
    pub const UNARY: Precedence = Precedence(150);
    pub const POSTFIX: Precedence = Precedence(160);
    pub const CALL: Precedence = Precedence(170);
    pub const MEMBER: Precedence = Precedence(180);
}

/// Get operator precedence and associativity
pub fn get_operator_info(token: &Token) -> Option<(Precedence, Associativity)> {
    use Associativity::{Left, Right};

    match token {
        // Assignment operators (right-associative)
        Token::Equal
        | Token::PlusEqual
        | Token::MinusEqual
        | Token::StarEqual
        | Token::SlashEqual
        | Token::PercentEqual
        | Token::AmpersandEqual
        | Token::PipeEqual
        | Token::CaretEqual
        | Token::LeftShiftEqual
 => Option::Some((Precedence::ASSIGNMENT, Right)),

        // Pipeline operator (left-associative)
        Token::Pipeline => Option::Some((Precedence::PIPELINE, Left)),

        // Actor operators (right-associative, like assignment)
        Token::LeftArrow | Token::ActorQuery => Option::Some((Precedence::ASSIGNMENT, Right)),

        // Logical operators
        Token::OrOr => Option::Some((Precedence::LOGICAL_OR, Left)),
        Token::AndAnd => Option::Some((Precedence::LOGICAL_AND, Left)),

        // Equality operators
        Token::EqualEqual | Token::NotEqual => Option::Some((Precedence::EQUALITY, Left)),

        // Comparison operators
        Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual => {
            Option::Some((Precedence::COMPARISON, Left))
        }

        // Bitwise operators
        Token::Pipe => Option::Some((Precedence::BITWISE_OR, Left)),
        Token::Caret => Option::Some((Precedence::BITWISE_XOR, Left)),
        Token::Ampersand => Option::Some((Precedence::BITWISE_AND, Left)),
        Token::LeftShift => Option::Some((Precedence::SHIFT, Left)),

        // Range operators
        Token::DotDot | Token::DotDotEqual => Option::Some((Precedence::RANGE, Left)),

        // Arithmetic operators
        Token::Plus | Token::Minus => Option::Some((Precedence::ADDITIVE, Left)),
        Token::Star | Token::Slash | Token::Percent => {
            Option::Some((Precedence::MULTIPLICATIVE, Left))
        }
        Token::Power => Option::Some((Precedence::POWER, Right)),

        _ => Option::None,
    }
}

/// Check if token is a postfix operator
pub fn is_postfix_operator(token: &Token) -> bool {
    matches!(
        token,
        Token::Question      // ? (try operator)
        | Token::Increment   // ++
        | Token::Decrement   // --
        | Token::Dot         // . (member access)
        | Token::LeftParen   // ( (function call)
        | Token::LeftBracket // [ (indexing)
    )
}

/// Check if token is a prefix operator
pub fn is_prefix_operator(token: &Token) -> bool {
    matches!(
        token,
        Token::Bang          // ! (logical not)
        | Token::Minus       // - (negation)
        | Token::Tilde       // ~ (bitwise not)
        | Token::Ampersand   // & (reference)
        | Token::Star        // * (dereference)
        | Token::Increment   // ++ (pre-increment)
        | Token::Decrement // -- (pre-decrement)
    )
}

/// Get postfix operator precedence
pub fn get_postfix_precedence(token: &Token) -> Precedence {
    match token {
        Token::Dot => Precedence::MEMBER,
        Token::LeftParen | Token::LeftBracket => Precedence::CALL,
        Token::Question | Token::Increment | Token::Decrement => Precedence::POSTFIX,
        _ => Precedence(0),
    }
}

/// Check if we should continue parsing based on precedence
pub fn should_continue_parsing(current_token: &Token, min_precedence: Precedence) -> bool {
    if let Option::Some((prec, _)) = get_operator_info(current_token) {
        prec.0 >= min_precedence.0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_precedence_ordering() {
        // Verify precedence ordering
        assert!(Precedence::ASSIGNMENT < Precedence::PIPELINE);
        assert!(Precedence::PIPELINE < Precedence::LOGICAL_OR);
        assert!(Precedence::LOGICAL_OR < Precedence::LOGICAL_AND);
        assert!(Precedence::ADDITIVE < Precedence::MULTIPLICATIVE);
        assert!(Precedence::MULTIPLICATIVE < Precedence::POWER);
        assert!(Precedence::POWER < Precedence::POSTFIX);
        assert!(Precedence::POSTFIX < Precedence::CALL);
    }

    #[test]
    fn test_postfix_operators() {
        assert!(is_postfix_operator(&Token::Question));
        assert!(is_postfix_operator(&Token::Dot));
        assert!(is_postfix_operator(&Token::LeftParen));
        assert!(!is_postfix_operator(&Token::Plus));
    }

    #[test]
    fn test_operator_associativity() {
        // Assignment is right-associative
        let (_, assoc) = get_operator_info(&Token::Equal).expect("Equal should have operator info");
        assert_eq!(assoc, Associativity::Right);

        // Addition is left-associative
        let (_, assoc) = get_operator_info(&Token::Plus).expect("Plus should have operator info");
        assert_eq!(assoc, Associativity::Left);

        // Power is right-associative
        let (_, assoc) = get_operator_info(&Token::Power).expect("Power should have operator info");
        assert_eq!(assoc, Associativity::Right);
    }
}
