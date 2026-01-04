//! Effect system parsing - SPEC-001-I, SPEC-001-J
use super::{bail, utils, Expr, ExprKind, ParserState, Result, Token};
use crate::frontend::ast::{EffectHandler, EffectOperation, Pattern};

pub fn parse_effect(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked").1;
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let n = n.clone();
            state.tokens.advance();
            n
        }
        _ => bail!("Expected effect name"),
    };
    state.tokens.expect(&Token::LeftBrace)?;
    let mut operations = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let op_name = match state.tokens.peek() {
            Some((Token::Identifier(n), _)) => {
                let n = n.clone();
                state.tokens.advance();
                n
            }
            _ => bail!("Expected operation name"),
        };
        let params = utils::parse_params(state)?;
        let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
            state.tokens.advance();
            Some(utils::parse_type(state)?)
        } else {
            None
        };
        operations.push(EffectOperation {
            name: op_name,
            params,
            return_type,
        });
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Effect { name, operations }, start_span))
}

/// Parse a single handler case: operation(params) => body
fn parse_single_handler(state: &mut ParserState) -> Result<EffectHandler> {
    let operation = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let n = n.clone();
            state.tokens.advance();
            n
        }
        _ => bail!("Expected operation name in handler"),
    };
    let params = parse_handler_params(state)?;
    state.tokens.expect(&Token::FatArrow)?;
    let body = Box::new(super::parse_expr_recursive(state)?);
    Ok(EffectHandler {
        operation,
        params,
        body,
    })
}

/// SPEC-001-J: Parse effect handler expression
/// Syntax: handle expr with { operation => body, operation(params) => body }
pub fn parse_handler(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked").1;
    let expr = Box::new(super::parse_expr_recursive(state)?);
    state.tokens.expect(&Token::With)?;
    state.tokens.expect(&Token::LeftBrace)?;

    let mut handlers = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        handlers.push(parse_single_handler(state)?);
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Handle { expr, handlers }, start_span))
}

fn parse_handler_params(state: &mut ParserState) -> Result<Vec<Pattern>> {
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        state.tokens.advance();
        let mut params = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            let param_name = match state.tokens.peek() {
                Some((Token::Identifier(n), _)) => {
                    let n = n.clone();
                    state.tokens.advance();
                    n
                }
                _ => bail!("Expected parameter name"),
            };
            params.push(Pattern::Identifier(param_name));
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            }
        }
        state.tokens.expect(&Token::RightParen)?;
        Ok(params)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::parser::Parser;

    /// Helper to parse effect/handler code through the full parser
    fn parse_code(code: &str) -> crate::frontend::parser::Result<crate::frontend::ast::Expr> {
        Parser::new(code).parse()
    }

    // ==================== parse_effect tests ====================

    #[test]
    fn test_parse_effect_simple() {
        let result = parse_code("effect Logger { log(msg: String) }");
        assert!(result.is_ok(), "Failed to parse simple effect: {result:?}");
    }

    #[test]
    fn test_parse_effect_multiple_operations() {
        let result = parse_code("effect IO { read() -> String, write(data: String) }");
        assert!(
            result.is_ok(),
            "Failed to parse multi-op effect: {result:?}"
        );
    }

    #[test]
    fn test_parse_effect_empty_operations() {
        let result = parse_code("effect Empty { }");
        assert!(result.is_ok(), "Failed to parse empty effect: {result:?}");
    }

    #[test]
    fn test_parse_effect_with_return_type() {
        let result = parse_code("effect State { get() -> i32, set(val: i32) }");
        assert!(
            result.is_ok(),
            "Failed to parse effect with return: {result:?}"
        );
    }

    #[test]
    fn test_parse_effect_no_params() {
        let result = parse_code("effect Simple { action() }");
        assert!(
            result.is_ok(),
            "Failed to parse effect without params: {result:?}"
        );
    }

    #[test]
    fn test_parse_effect_multiple_params() {
        let result = parse_code("effect Math { add(a: i32, b: i32) -> i32 }");
        assert!(
            result.is_ok(),
            "Failed to parse effect with multiple params: {result:?}"
        );
    }

    // ==================== parse_handler tests ====================

    #[test]
    fn test_parse_handler_simple() {
        let result = parse_code("handle computation with { log => println(\"logged\") }");
        assert!(result.is_ok(), "Failed to parse simple handler: {result:?}");
    }

    #[test]
    fn test_parse_handler_with_params() {
        let result = parse_code("handle expr with { write(x) => print(x) }");
        assert!(
            result.is_ok(),
            "Failed to parse handler with params: {result:?}"
        );
    }

    #[test]
    fn test_parse_handler_multiple() {
        let result = parse_code("handle x with { get => 42, set(v) => v }");
        assert!(
            result.is_ok(),
            "Failed to parse multiple handlers: {result:?}"
        );
    }

    #[test]
    fn test_parse_handler_empty() {
        let result = parse_code("handle expr with { }");
        assert!(result.is_ok(), "Failed to parse empty handler: {result:?}");
    }

    #[test]
    fn test_parse_handler_multiple_params() {
        let result = parse_code("handle x with { op(a, b, c) => a + b + c }");
        assert!(
            result.is_ok(),
            "Failed to parse multi-param handler: {result:?}"
        );
    }

    #[test]
    fn test_parse_handler_complex_body() {
        let result = parse_code("handle comp with { read => { let x = 42; x } }");
        assert!(
            result.is_ok(),
            "Failed to parse handler with block body: {result:?}"
        );
    }

    #[test]
    fn test_parse_handler_nested_expr() {
        let result = parse_code("handle (1 + 2) with { op => 3 }");
        assert!(
            result.is_ok(),
            "Failed to parse handler with nested expr: {result:?}"
        );
    }

    // ==================== Error case tests ====================

    #[test]
    fn test_parse_effect_missing_name() {
        let result = parse_code("effect { }");
        assert!(result.is_err(), "Should fail on missing effect name");
    }

    #[test]
    fn test_parse_effect_missing_brace() {
        let result = parse_code("effect Logger log()");
        assert!(result.is_err(), "Should fail on missing brace");
    }

    #[test]
    fn test_parse_handler_missing_with() {
        let result = parse_code("handle expr { }");
        assert!(result.is_err(), "Should fail on missing 'with' keyword");
    }

    #[test]
    fn test_parse_handler_missing_arrow() {
        let result = parse_code("handle expr with { op body }");
        assert!(result.is_err(), "Should fail on missing '=>'");
    }
}
