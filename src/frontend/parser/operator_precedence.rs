//! Operator precedence handling for the Ruchy parser
//!
//! This module defines operator precedence and associativity rules
//! ensuring proper parsing of complex expressions.
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
    pub const MESSAGE_SEND: Precedence = Precedence(15); // actor ! Message
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
        // Assignment and actor operators (right-associative)
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
        | Token::LeftArrow
        | Token::ActorQuery => Option::Some((Precedence::ASSIGNMENT, Right)),
        // Message send operator (left-associative: actor ! Message)
        Token::Bang => Option::Some((Precedence::MESSAGE_SEND, Left)),
        // Pipeline operator (left-associative)
        Token::Pipeline => Option::Some((Precedence::PIPELINE, Left)),
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
        Token::RightShift => Option::Some((Precedence::SHIFT, Left)),
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
        | Token::SafeNav     // ?. (optional chaining)
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
        | Token::Plus        // + (unary plus - identity)
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
        Token::Dot | Token::SafeNav => Precedence::MEMBER,
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

    // Sprint 8 Phase 1: Mutation test gap coverage
    // Target: 21 MISSED → 0 MISSED (21% → 80%+ catch rate)

    #[test]
    fn test_all_operator_match_arms() {
        // Test gaps: verify all match arms in get_operator_info
        assert!(get_operator_info(&Token::Pipeline).is_some(), "Pipeline");
        assert!(get_operator_info(&Token::OrOr).is_some(), "OrOr");
        assert!(get_operator_info(&Token::AndAnd).is_some(), "AndAnd");
        assert!(
            get_operator_info(&Token::EqualEqual).is_some(),
            "EqualEqual"
        );
        assert!(get_operator_info(&Token::NotEqual).is_some(), "NotEqual");
        assert!(get_operator_info(&Token::Less).is_some(), "Less");
        assert!(get_operator_info(&Token::Greater).is_some(), "Greater");
        assert!(get_operator_info(&Token::LessEqual).is_some(), "LessEqual");
        assert!(
            get_operator_info(&Token::GreaterEqual).is_some(),
            "GreaterEqual"
        );
        assert!(
            get_operator_info(&Token::Pipe).is_some(),
            "Pipe (bitwise OR)"
        );
        assert!(get_operator_info(&Token::Caret).is_some(), "Caret (XOR)");
        assert!(
            get_operator_info(&Token::Ampersand).is_some(),
            "Ampersand (bitwise AND)"
        );
        assert!(get_operator_info(&Token::LeftShift).is_some(), "LeftShift");
        assert!(
            get_operator_info(&Token::RightShift).is_some(),
            "RightShift"
        );
        assert!(get_operator_info(&Token::DotDot).is_some(), "DotDot");
        assert!(
            get_operator_info(&Token::DotDotEqual).is_some(),
            "DotDotEqual"
        );
        assert!(get_operator_info(&Token::Star).is_some(), "Star");
        assert!(get_operator_info(&Token::Slash).is_some(), "Slash");
        assert!(get_operator_info(&Token::Percent).is_some(), "Percent");
        assert!(get_operator_info(&Token::Bang).is_some(), "Bang");
    }

    #[test]
    fn test_all_postfix_match_arms() {
        // Test gaps: verify all match arms in get_postfix_precedence
        assert_eq!(
            get_postfix_precedence(&Token::Dot),
            Precedence::MEMBER,
            "Dot"
        );
        assert_eq!(
            get_postfix_precedence(&Token::SafeNav),
            Precedence::MEMBER,
            "SafeNav"
        );
        assert_eq!(
            get_postfix_precedence(&Token::LeftParen),
            Precedence::CALL,
            "LeftParen"
        );
        assert_eq!(
            get_postfix_precedence(&Token::LeftBracket),
            Precedence::CALL,
            "LeftBracket"
        );
        assert_eq!(
            get_postfix_precedence(&Token::Question),
            Precedence::POSTFIX,
            "Question"
        );
        assert_eq!(
            get_postfix_precedence(&Token::Increment),
            Precedence::POSTFIX,
            "Increment"
        );
        assert_eq!(
            get_postfix_precedence(&Token::Decrement),
            Precedence::POSTFIX,
            "Decrement"
        );
    }

    #[test]
    fn test_is_prefix_operator_returns_false_for_non_prefix() {
        // Test gap: verify is_prefix_operator returns false (not just true)
        assert!(!is_prefix_operator(&Token::Slash), "Slash is not prefix");
        assert!(!is_prefix_operator(&Token::Dot), "Dot is not prefix");
        assert!(
            !is_prefix_operator(&Token::EqualEqual),
            "EqualEqual is not prefix"
        );
    }

    #[test]
    fn test_is_prefix_operator_returns_true_for_prefix() {
        // Test gap: verify is_prefix_operator returns true (not just false)
        assert!(is_prefix_operator(&Token::Bang), "Bang is prefix");
        assert!(
            is_prefix_operator(&Token::Minus),
            "Minus (negation) is prefix"
        );
        assert!(
            is_prefix_operator(&Token::Plus),
            "Plus (identity) is prefix"
        );
        assert!(is_prefix_operator(&Token::Tilde), "Tilde is prefix");
        assert!(
            is_prefix_operator(&Token::Ampersand),
            "Ampersand (ref) is prefix"
        );
        assert!(is_prefix_operator(&Token::Star), "Star (deref) is prefix");
        assert!(is_prefix_operator(&Token::Increment), "Increment is prefix");
        assert!(is_prefix_operator(&Token::Decrement), "Decrement is prefix");
    }

    #[test]
    fn test_should_continue_parsing_precedence_comparison() {
        // Test gap: verify >= condition (not < or ==)
        assert!(
            should_continue_parsing(&Token::Plus, Precedence(100)),
            "Should continue when current > min"
        );
        assert!(
            should_continue_parsing(&Token::Plus, Precedence(120)),
            "Should continue when current == min"
        );
        assert!(
            !should_continue_parsing(&Token::Plus, Precedence(130)),
            "Should stop when current < min"
        );
    }

    #[test]
    fn test_should_continue_parsing_returns_false_for_non_operators() {
        // Test gap: verify function returns false (not true)
        assert!(
            !should_continue_parsing(&Token::Semicolon, Precedence(0)),
            "Non-operator should return false"
        );
        assert!(
            !should_continue_parsing(&Token::Comma, Precedence(0)),
            "Non-operator should return false"
        );
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[test]
    fn test_is_prefix_operator_not_stub() {
        // MISSED: replace is_prefix_operator -> bool with true (line 100)

        // Test that function returns true for prefix operators
        assert!(is_prefix_operator(&Token::Bang), "Bang should be prefix");
        assert!(is_prefix_operator(&Token::Minus), "Minus should be prefix");
        assert!(
            is_prefix_operator(&Token::Plus),
            "Plus should be prefix (identity)"
        );
        assert!(is_prefix_operator(&Token::Tilde), "Tilde should be prefix");

        // Test that function returns false for non-prefix operators (proves not stub returning true)
        assert!(
            !is_prefix_operator(&Token::Slash),
            "Slash should NOT be prefix"
        );
        assert!(!is_prefix_operator(&Token::Dot), "Dot should NOT be prefix");
        assert!(
            !is_prefix_operator(&Token::Equal),
            "Equal should NOT be prefix"
        );
    }

    #[test]
    fn test_get_operator_info_ampersand_match_arm() {
        // MISSED: delete match arm Token::Ampersand in get_operator_info (line 71)

        let result = get_operator_info(&Token::Ampersand);
        assert!(result.is_some(), "Ampersand should have operator info");

        let (prec, assoc) = result.unwrap();
        assert_eq!(
            prec,
            Precedence::BITWISE_AND,
            "Ampersand should have BITWISE_AND precedence"
        );
        assert_eq!(
            assoc,
            Associativity::Left,
            "Ampersand should be left-associative"
        );
    }

    #[test]
    fn test_get_operator_info_left_shift_match_arm() {
        // MISSED: delete match arm Token::LeftShift in get_operator_info (line 72)

        let result = get_operator_info(&Token::LeftShift);
        assert!(result.is_some(), "LeftShift should have operator info");

        let (prec, assoc) = result.unwrap();
        assert_eq!(
            prec,
            Precedence::SHIFT,
            "LeftShift should have SHIFT precedence"
        );
        assert_eq!(
            assoc,
            Associativity::Left,
            "LeftShift should be left-associative"
        );
    }

    #[test]
    fn test_get_operator_info_right_shift_match_arm() {
        // MISSED: delete match arm Token::RightShift in get_operator_info (line 73)

        let result = get_operator_info(&Token::RightShift);
        assert!(result.is_some(), "RightShift should have operator info");

        let (prec, assoc) = result.unwrap();
        assert_eq!(
            prec,
            Precedence::SHIFT,
            "RightShift should have SHIFT precedence"
        );
        assert_eq!(
            assoc,
            Associativity::Left,
            "RightShift should be left-associative"
        );
    }
}
