//! Increment and decrement operator parsing
//!
//! Handles parsing of:
//! - Pre-increment: `++var`
//! - Pre-decrement: `--var`
//! - Constructor tokens: `Some`, `None`, `Ok`, `Err`, `Result`, `Option`
//! - Qualified constructors: `Option::Some`, `Result::Ok`
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, ParserState, Result};

/// Parse increment operator (++var or var++)
pub(in crate::frontend::parser) fn parse_increment_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume '++'

    // Parse the variable being incremented
    let variable = parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::PreIncrement {
            target: Box::new(variable),
        },
        span,
    ))
}

/// Parse decrement operator (--var or var--)
pub(in crate::frontend::parser) fn parse_decrement_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume '--'

    // Parse the variable being decremented
    let variable = parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::PreDecrement {
            target: Box::new(variable),
        },
        span,
    ))
}

/// Parse constructor tokens (Some, None, Ok, Err, Result, Option)
pub(in crate::frontend::parser) fn parse_constructor_token(
    state: &mut ParserState,
    token: Token,
    span: Span,
) -> Result<Expr> {
    let constructor_name = match token {
        Token::Some => "Some",
        Token::None => "None",
        Token::Ok => "Ok",
        Token::Err => "Err",
        Token::Result => "Result",
        Token::Option => "Option",
        _ => bail!("Expected constructor token, got: {token:?}"),
    };
    state.tokens.advance();

    // Check if this is a qualified name like Option::Some
    if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        if let Some((next_token, _)) = state.tokens.peek() {
            let variant_name = match next_token.clone() {
                Token::Some => "Some".to_string(),
                Token::None => "None".to_string(),
                Token::Ok => "Ok".to_string(),
                Token::Err => "Err".to_string(),
                Token::Identifier(name) => name,
                _ => bail!("Expected variant name after '::'"),
            };
            state.tokens.advance();
            let qualified_name = format!("{constructor_name}::{variant_name}");
            return Ok(Expr::new(ExprKind::Identifier(qualified_name), span));
        }
        bail!("Expected variant name after '::'");
    }

    Ok(Expr::new(
        ExprKind::Identifier(constructor_name.to_string()),
        span,
    ))
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_pre_increment() {
        let code = "++x";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Pre-increment should parse");
    }

    #[test]
    fn test_pre_decrement() {
        let code = "--x";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Pre-decrement should parse");
    }

    #[test]
    fn test_result_type_constructor() {
        let code = "Result";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Result type should parse");
    }

    #[test]
    fn test_option_type_constructor() {
        let code = "Option";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Option type should parse");
    }

    #[test]
    fn test_ok_variant() {
        let code = "Ok";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Ok variant should parse");
    }

    #[test]
    fn test_err_variant() {
        let code = "Err";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Err variant should parse");
    }

    #[test]
    fn test_qualified_constructor() {
        let code = "Option::Some";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Qualified constructor should parse");
    }

    // Test 8: Some variant (may need call syntax)
    #[test]
    fn test_some_variant() {
        // Some by itself needs call syntax or might be identifier
        let code = "let x = Some(1)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Some variant should parse");
    }

    // Test 9: None variant
    #[test]
    fn test_none_variant() {
        let code = "None";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "None variant should parse");
    }

    // Test 10: Result::Ok qualified
    #[test]
    fn test_result_ok_qualified() {
        let code = "Result::Ok";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Result::Ok should parse");
    }

    // Test 11: Result::Err qualified
    #[test]
    fn test_result_err_qualified() {
        let code = "Result::Err";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Result::Err should parse");
    }

    // Test 12: Option::None qualified
    #[test]
    fn test_option_none_qualified() {
        let code = "Option::None";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Option::None should parse");
    }

    // Test 13: Pre-increment with array access
    #[test]
    fn test_pre_increment_array() {
        let code = "++arr[0]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Pre-increment array should parse");
    }

    // Test 14: Pre-decrement with array access
    #[test]
    fn test_pre_decrement_array() {
        let code = "--arr[0]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Pre-decrement array should parse");
    }

    // Test 15: Pre-increment in expression
    #[test]
    fn test_pre_increment_in_expr() {
        let code = "let x = ++y";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Pre-increment in let should parse");
    }

    // Test 16: Pre-decrement in expression
    #[test]
    fn test_pre_decrement_in_expr() {
        let code = "let x = --y";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Pre-decrement in let should parse");
    }

    // Test 17: Some with call syntax
    #[test]
    fn test_some_call() {
        let code = "Some(42)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Some(42) should parse");
    }

    // Test 18: None in expression
    #[test]
    fn test_none_in_expr() {
        let code = "let x = None";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "None in let should parse");
    }

    // Test 19: Ok with call syntax
    #[test]
    fn test_ok_call() {
        let code = "Ok(value)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Ok(value) should parse");
    }

    // Test 20: Err with call syntax
    #[test]
    fn test_err_call() {
        let code = "Err(message)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Err(message) should parse");
    }

    // Test 21: Option::Some with call syntax
    #[test]
    fn test_option_some_call() {
        let code = "Option::Some(42)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Option::Some(42) should parse");
    }

    // Test 22: Multiple increments in block
    #[test]
    fn test_multiple_increments() {
        let code = "{ ++x; ++y }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Multiple increments should parse");
    }
}

#[cfg(test)]
mod property_tests {

    use crate::frontend::parser::Parser;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_increment_never_panics(var in "[a-z][a-z0-9_]{0,10}") {
            let code = format!("++{var}");
            let _ = Parser::new(&code).parse();
        }

        #[test]
        fn test_decrement_never_panics(var in "[a-z][a-z0-9_]{0,10}") {
            let code = format!("--{var}");
            let _ = Parser::new(&code).parse();
        }

        #[test]
        fn test_constructors_never_panic(constructor in "(Some|None|Ok|Err|Result|Option)") {
            let _ = Parser::new(&constructor).parse();
        }

        #[test]
        fn test_qualified_constructors_never_panic(
            container in "(Option|Result)",
            variant in "(Some|None|Ok|Err)"
        ) {
            let code = format!("{container}::{variant}");
            let _ = Parser::new(&code).parse();
        }
    }
}
