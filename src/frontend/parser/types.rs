//! Type-related parsing - minimal version with only used functions
use super::{bail, Expr, ExprKind, ParserState, Result, Span, Token};

// Helper: Parse update syntax base expression (complexity: 3)
fn parse_struct_base(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::DotDot, _))) {
        state.tokens.advance(); // consume ..
        let base = Some(Box::new(super::parse_expr_recursive(state)?));
        // After base expression, we should only have optional comma and closing brace
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
        Ok(base)
    } else {
        Ok(None)
    }
}

// Helper: Parse field name identifier (complexity: 2)
fn parse_field_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected field name");
    }
}

// Helper: Parse field value with shorthand support (complexity: 2)
fn parse_field_value(state: &mut ParserState, field_name: &str, start_span: Span) -> Result<Expr> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume :
        super::parse_expr_recursive(state)
    } else {
        // Field shorthand: use field name as identifier expression
        Ok(Expr::new(
            ExprKind::Identifier(field_name.to_string()),
            start_span,
        ))
    }
}

// Helper: Consume optional trailing comma (complexity: 2)
fn consume_trailing_comma(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

// Main function: Parse struct literal (complexity: 4)
pub fn parse_struct_literal(
    state: &mut ParserState,
    name: String,
    start_span: Span,
) -> Result<Expr> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();
    let mut base = None;

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Check for update syntax ..expr
        if let Some(base_expr) = parse_struct_base(state)? {
            base = Some(base_expr);
            break;
        }

        // Parse field name
        let field_name = parse_field_name(state)?;

        // Parse colon and value, or use field shorthand
        let value = parse_field_value(state, &field_name, start_span)?;
        fields.push((field_name, value));

        // Handle comma or end of struct literal
        if !consume_trailing_comma(state) {
            break;
        }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(
        ExprKind::StructLiteral { name, fields, base },
        start_span,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_struct_literal_empty() {
        let mut state = ParserState::new("{ }");
        let result = parse_struct_literal(&mut state, "Point".to_string(), Span::default());
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::StructLiteral { name, fields, base } = expr.kind {
            assert_eq!(name, "Point");
            assert!(fields.is_empty());
            assert!(base.is_none());
        } else {
            panic!("Expected StructLiteral");
        }
    }

    #[test]
    fn test_parse_struct_literal_single_field() {
        let mut state = ParserState::new("{ x: 10 }");
        let result = parse_struct_literal(&mut state, "Point".to_string(), Span::default());
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::StructLiteral { name, fields, .. } = expr.kind {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "x");
        } else {
            panic!("Expected StructLiteral");
        }
    }

    #[test]
    fn test_parse_struct_literal_multiple_fields() {
        let mut state = ParserState::new("{ x: 10, y: 20 }");
        let result = parse_struct_literal(&mut state, "Point".to_string(), Span::default());
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::StructLiteral { fields, .. } = expr.kind {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
        } else {
            panic!("Expected StructLiteral");
        }
    }

    #[test]
    fn test_parse_struct_literal_trailing_comma() {
        let mut state = ParserState::new("{ x: 10, y: 20, }");
        let result = parse_struct_literal(&mut state, "Point".to_string(), Span::default());
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::StructLiteral { fields, .. } = expr.kind {
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected StructLiteral");
        }
    }

    #[test]
    fn test_parse_struct_literal_field_shorthand() {
        let mut state = ParserState::new("{ x, y }");
        let result = parse_struct_literal(&mut state, "Point".to_string(), Span::default());
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::StructLiteral { fields, .. } = expr.kind {
            assert_eq!(fields.len(), 2);
            // Field shorthand: x becomes (x, Identifier(x))
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
        } else {
            panic!("Expected StructLiteral");
        }
    }

    #[test]
    fn test_consume_trailing_comma() {
        let mut state = ParserState::new(", x");
        let result = consume_trailing_comma(&mut state);
        assert!(result);
    }

    #[test]
    fn test_consume_trailing_comma_none() {
        let mut state = ParserState::new("x");
        let result = consume_trailing_comma(&mut state);
        assert!(!result);
    }
}
