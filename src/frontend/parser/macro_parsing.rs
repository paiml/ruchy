//! Macro parsing utilities - Extracted to reduce complexity
//!
//! This module contains helper functions for parsing various macro types,
//! extracted from try_parse_macro_call to reduce its complexity from 105 to <10.

use super::{bail, parse_expr_recursive, Expr, ExprKind, ParserState, Result, Span, Token};
use crate::frontend::ast::Literal;

/// Parse df![] DataFrame macro (complexity: 7)
pub fn parse_dataframe_macro(state: &mut ParserState) -> Result<Option<Expr>> {
    state.tokens.advance(); // consume !

    if !matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
        return Ok(None); // Not df![], let regular macro parsing handle it
    }

    state.tokens.advance(); // consume [

    // Check for empty DataFrame df![]
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance(); // consume ]
        return Ok(Some(Expr::new(
            ExprKind::DataFrame {
                columns: Vec::new(),
            },
            Span::default(),
        )));
    }

    // Parse DataFrame columns
    let columns = super::collections::parse_dataframe_column_definitions(state)?;
    state.tokens.expect(&Token::RightBracket)?;

    Ok(Some(Expr::new(
        ExprKind::DataFrame { columns },
        Span::default(),
    )))
}

/// Parse sql!{} macro with special SQL content handling (complexity: 8)
pub fn parse_sql_macro(state: &mut ParserState, name: &str) -> Result<Expr> {
    state.tokens.advance(); // consume {

    let sql_content = collect_sql_content(state)?;

    Ok(Expr::new(
        ExprKind::Macro {
            name: name.to_string(),
            args: vec![Expr::new(
                ExprKind::Literal(Literal::String(sql_content)),
                Span::default(),
            )],
        },
        Span::default(),
    ))
}

/// Collect raw SQL content until closing brace (complexity: 7)
fn collect_sql_content(state: &mut ParserState) -> Result<String> {
    let mut sql_content = String::new();
    let mut depth = 1;

    while depth > 0 {
        if let Some((token, _)) = state.tokens.peek() {
            match token {
                Token::LeftBrace => depth += 1,
                Token::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        state.tokens.advance(); // consume final }
                        break;
                    }
                }
                _ => {}
            }

            if depth > 0 {
                let token_text = convert_token_to_sql(token);
                if !sql_content.is_empty() {
                    sql_content.push(' ');
                }
                sql_content.push_str(&token_text);
                state.tokens.advance();
            }
        } else {
            bail!("Unclosed SQL macro");
        }
    }

    Ok(sql_content)
}

/// Convert a token to its SQL text representation (complexity: 5)
fn convert_token_to_sql(token: &Token) -> String {
    match token {
        Token::Identifier(s) => s.clone(),
        Token::Integer(n) => n.to_string(),
        Token::Float(f) => f.to_string(),
        Token::String(s) => format!("'{s}'"),
        Token::Star => "*".to_string(),
        Token::Greater => ">".to_string(),
        Token::Less => "<".to_string(),
        Token::GreaterEqual => ">=".to_string(),
        Token::LessEqual => "<=".to_string(),
        Token::EqualEqual => "=".to_string(),
        Token::NotEqual => "!=".to_string(),
        Token::Comma => ",".to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
        Token::Dot => ".".to_string(),
        Token::Plus => "+".to_string(),
        Token::Minus => "-".to_string(),
        Token::Slash => "/".to_string(),
        _ => format!("{token:?}"), // Fallback for unhandled tokens
    }
}

/// Determine macro delimiter style and closing token (complexity: 4)
pub fn get_macro_delimiters(state: &mut ParserState) -> Option<(&'static str, Token)> {
    match state.tokens.peek() {
        Some((Token::LeftParen, _)) => {
            state.tokens.advance(); // consume (
            Some(("paren", Token::RightParen))
        }
        Some((Token::LeftBracket, _)) => {
            state.tokens.advance(); // consume [
            Some(("bracket", Token::RightBracket))
        }
        Some((Token::LeftBrace, _)) => {
            state.tokens.advance(); // consume {
            Some(("brace", Token::RightBrace))
        }
        _ => None,
    }
}

/// Parse macro arguments until closing token (complexity: 6)
pub fn parse_macro_arguments(state: &mut ParserState, closing_token: Token) -> Result<Vec<Expr>> {
    let mut args = Vec::new();

    while !matches!(state.tokens.peek(), Some((token, _)) if token == &closing_token) {
        args.push(parse_expr_recursive(state)?);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }

    state.tokens.expect(&closing_token)?;
    Ok(args)
}

/// Create a macro expression (complexity: 1)
pub fn create_macro_expr(name: String, args: Vec<Expr>) -> Expr {
    Expr::new(ExprKind::Macro { name, args }, Span::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_token_to_sql() {
        assert_eq!(convert_token_to_sql(&Token::Star), "*");
        assert_eq!(convert_token_to_sql(&Token::Integer(42)), "42");
        assert_eq!(
            convert_token_to_sql(&Token::String("test".to_string())),
            "'test'"
        );
        assert_eq!(convert_token_to_sql(&Token::Greater), ">");
        assert_eq!(convert_token_to_sql(&Token::GreaterEqual), ">=");
    }
}
