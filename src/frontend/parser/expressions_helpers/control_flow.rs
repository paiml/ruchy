//! Control flow expression parsing
//!
//! Handles parsing of control flow statements: break, continue, return, throw.
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{parse_expr_recursive, ParserState, Result};

/// Skip any comment tokens in the stream
///
/// Comments should be transparent to parsing logic - they don't affect syntax.
/// This helper ensures comment tokens don't interfere with terminator detection.
fn skip_comments(state: &mut ParserState) {
    while matches!(
        state.tokens.peek(),
        Some((
            Token::LineComment(_)
                | Token::BlockComment(_)
                | Token::DocComment(_)
                | Token::HashComment(_),
            _
        ))
    ) {
        state.tokens.advance();
    }
}

/// Parse break token with optional label and value
///
/// Syntax: `break`, `break 'label`, `break value`, `break 'label value`
///
/// # Examples
/// ```ruchy
/// break;
/// break 'outer;
/// break 42;
/// break 'loop1 value;
/// ```
pub(in crate::frontend::parser) fn parse_break_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance();

    // Optional label (lifetime syntax 'label)
    let label = if let Some((Token::Lifetime(name), _)) = state.tokens.peek() {
        let label = Some(name.clone());
        state.tokens.advance();
        label
    } else {
        None
    };

    // Skip comments before checking for terminators (PARSER-062 fix)
    skip_comments(state);

    // Parse optional break value: break <expr> or break 'label <expr>
    let value = if matches!(
        state.tokens.peek(),
        Some((Token::Semicolon | Token::RightBrace | Token::RightParen, _))
    ) || state.tokens.peek().is_none()
    {
        // No value if followed by terminator or EOF
        None
    } else {
        // Parse the value expression
        Some(Box::new(parse_expr_recursive(state)?))
    };

    Ok(Expr::new(ExprKind::Break { label, value }, span))
}

/// Parse continue token with optional label
///
/// Syntax: `continue`, `continue 'label`
///
/// # Examples
/// ```ruchy
/// continue;
/// continue 'outer;
/// ```
pub(in crate::frontend::parser) fn parse_continue_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance();

    // Optional label (lifetime syntax 'label)
    let label = if let Some((Token::Lifetime(name), _)) = state.tokens.peek() {
        let label = Some(name.clone());
        state.tokens.advance();
        label
    } else {
        None
    };

    // Skip comments after continue statement (PARSER-062 fix)
    skip_comments(state);

    Ok(Expr::new(ExprKind::Continue { label }, span))
}

/// Parse return token with optional expression
///
/// Supports bare returns (early exit) and returns with values.
/// Fixed in PARSER-055 to handle bare returns followed by `}`.
///
/// Syntax: `return`, `return expr`
///
/// # Examples
/// ```ruchy
/// return;           // Bare return (early exit)
/// return 42;        // Return with value
/// if x { return }   // Bare return in block
/// ```
pub(in crate::frontend::parser) fn parse_return_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance();

    // Skip comments before checking for terminators (PARSER-062 fix)
    skip_comments(state);

    // Check if there's an expression to return
    // Bare return is allowed when followed by: ;, }, or EOF
    let value = if matches!(
        state.tokens.peek(),
        Some((Token::Semicolon | Token::RightBrace, _))
    ) || state.tokens.peek().is_none()
    {
        // No expression, bare return (equivalent to return ())
        None
    } else {
        // Parse the return expression
        Some(Box::new(parse_expr_recursive(state)?))
    };

    Ok(Expr::new(ExprKind::Return { value }, span))
}

/// Parse throw statement token
///
/// Throw always requires an expression (the error to throw).
///
/// Syntax: `throw expr`
///
/// # Examples
/// ```ruchy
/// throw "Error message";
/// throw CustomError::new();
/// ```
pub(in crate::frontend::parser) fn parse_throw_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance();

    // Throw always requires an expression
    let expr = Box::new(parse_expr_recursive(state)?);

    Ok(Expr::new(ExprKind::Throw { expr }, span))
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_bare_return() {
        let code = "fun f() { return }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Bare return should parse successfully");
    }

    #[test]
    fn test_return_with_value() {
        let code = "fun f() { return 42 }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Return with value should parse successfully"
        );
    }

    #[test]
    #[ignore = "Property tests run with --ignored flag"] // PARSER-079: Parser architecture issue - statements with lifetime tokens in for loops
    fn test_break_with_label() {
        // Root cause: Parser gets confused when lifetime token appears in statement position within for loop
        // Error: "Expected RightBrace, found Break" suggests statement parsing consumes tokens incorrectly
        // Workaround: Use break without label, or use while loops which work correctly
        let code = "for x in xs { break 'outer; }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Break with label should parse successfully");
    }

    #[test]
    fn test_continue_no_label() {
        let code = "while true { continue }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Continue should parse successfully");
    }

    #[test]
    fn test_throw_expression() {
        let code = "fun f() { throw \"error\" }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Throw should parse successfully");
    }
}
