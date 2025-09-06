//! Literal parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Literal, Span, Parser};
use anyhow::bail;

/// Parse a literal expression
pub fn parse_literal(state: &mut ParserState) -> Result<Expr> {
    let (token, span) = state.next_token()?;
    
    match token {
        Token::Integer(val) => parse_integer(val, span),
        Token::Float(val) => parse_float(val, span),
        Token::String(val) => parse_string(val, span),
        Token::FString(val) => parse_fstring(val, span),
        Token::Char(val) => parse_char(val, span),
        Token::Bool(val) => parse_bool(val, span),
        Token::None => parse_none(span),
        _ => bail!("Expected literal, got {:?}", token),
    }
}

/// Parse integer literal
fn parse_integer(val: String, span: Span) -> Result<Expr> {
    let value = val.parse::<i64>()
        .map_err(|e| anyhow::anyhow!("Invalid integer literal '{}': {}", val, e))?;
    
    Ok(Expr {
        kind: ExprKind::Literal(Literal::Integer(value)),
        span,
        attributes: vec![],
    })
}

/// Parse float literal
fn parse_float(val: String, span: Span) -> Result<Expr> {
    let value = val.parse::<f64>()
        .map_err(|e| anyhow::anyhow!("Invalid float literal '{}': {}", val, e))?;
    
    Ok(Expr {
        kind: ExprKind::Literal(Literal::Float(value)),
        span,
        attributes: vec![],
    })
}

/// Parse string literal
fn parse_string(val: String, span: Span) -> Result<Expr> {
    Ok(Expr {
        kind: ExprKind::Literal(Literal::String(val)),
        span,
        attributes: vec![],
    })
}

/// Parse f-string (formatted string)
fn parse_fstring(val: String, span: Span) -> Result<Expr> {
    // Parse f-string into string interpolation parts
    let parts = parse_fstring_parts(&val)?;
    
    Ok(Expr {
        kind: ExprKind::StringInterpolation { parts },
        span,
        attributes: vec![],
    })
}

/// Parse f-string into parts
fn parse_fstring_parts(input: &str) -> Result<Vec<crate::frontend::ast::StringPart>> {
    use crate::frontend::ast::StringPart;
    
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                current.push('{');
            } else {
                // Save text part if any
                if !current.is_empty() {
                    parts.push(StringPart::Text(current.clone()));
                    current.clear();
                }
                
                // Extract expression string
                let expr_str = extract_interpolation(&mut chars)?;
                
                // Parse the expression string into an actual Expr
                let expr = parse_expression_from_string(&expr_str)?;
                parts.push(StringPart::Expr(Box::new(expr)));
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                current.push('}');
            } else {
                bail!("Unmatched '}}' in f-string");
            }
        } else {
            current.push(ch);
        }
    }
    
    // Add remaining text
    if !current.is_empty() {
        parts.push(StringPart::Text(current));
    }
    
    Ok(parts)
}

/// Parse expression from string (for f-string interpolations)
fn parse_expression_from_string(expr_str: &str) -> Result<Expr> {
    // Create a new parser with just the expression string
    let mut parser = Parser::new(expr_str);
    parser.parse_expr()
}

/// Extract interpolation expression from f-string
fn extract_interpolation(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String> {
    let mut expr = String::new();
    let mut depth = 1;
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            depth += 1;
            expr.push(ch);
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(expr);
            }
            expr.push(ch);
        } else {
            expr.push(ch);
        }
    }
    
    bail!("Unclosed interpolation in f-string")
}

/// Parse character literal
fn parse_char(val: String, span: Span) -> Result<Expr> {
    let ch = parse_char_value(&val)?;
    
    Ok(Expr {
        kind: ExprKind::Literal(Literal::Char(ch)),
        span,
        attributes: vec![],
    })
}

/// Parse character value from string
fn parse_char_value(val: &str) -> Result<char> {
    if val.is_empty() {
        bail!("Empty character literal");
    }
    
    let chars: Vec<char> = val.chars().collect();
    if chars.len() == 1 {
        Ok(chars[0])
    } else if chars.len() == 2 && chars[0] == '\\' {
        // Handle escape sequences
        match chars[1] {
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '\\' => Ok('\\'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            '0' => Ok('\0'),
            _ => bail!("Invalid escape sequence '\\{}'", chars[1]),
        }
    } else {
        bail!("Character literal must be exactly one character, got '{}'", val)
    }
}

/// Parse boolean literal
fn parse_bool(val: bool, span: Span) -> Result<Expr> {
    Ok(Expr {
        kind: ExprKind::Literal(Literal::Bool(val)),
        span,
        attributes: vec![],
    })
}

/// Parse None literal
fn parse_none(span: Span) -> Result<Expr> {
    Ok(Expr {
        kind: ExprKind::Literal(Literal::None),
        span,
        attributes: vec![],
    })
}

/// Parse numeric literal with optional suffix
pub fn parse_numeric_literal(state: &mut ParserState) -> Result<Expr> {
    let (token, span) = state.peek_token()?;
    
    match token {
        Token::Integer(val) => {
            state.advance();
            
            // Check for suffix (i32, i64, f32, f64, etc.)
            if let Ok((Token::Identifier(suffix), _)) = state.peek_token() {
                if is_numeric_suffix(&suffix) {
                    state.advance();
                    parse_suffixed_number(&val, &suffix, span)
                } else {
                    parse_integer(val, span)
                }
            } else {
                parse_integer(val, span)
            }
        }
        Token::Float(val) => {
            state.advance();
            parse_float(val, span)
        }
        _ => bail!("Expected numeric literal"),
    }
}

/// Check if string is a valid numeric suffix
fn is_numeric_suffix(s: &str) -> bool {
    matches!(s, "i8" | "i16" | "i32" | "i64" | "i128" |
             "u8" | "u16" | "u32" | "u64" | "u128" |
             "f32" | "f64" | "isize" | "usize")
}

/// Parse number with type suffix
fn parse_suffixed_number(val: &str, suffix: &str, span: Span) -> Result<Expr> {
    // For now, just parse as regular integer/float
    // Type checking will handle the suffix
    if suffix.starts_with('f') {
        let value = val.parse::<f64>()
            .map_err(|e| anyhow::anyhow!("Invalid float literal '{}': {}", val, e))?;
        Ok(Expr {
            kind: ExprKind::Literal(Literal::Float(value)),
            span,
            attributes: vec![],
        })
    } else {
        let value = val.parse::<i64>()
            .map_err(|e| anyhow::anyhow!("Invalid integer literal '{}': {}", val, e))?;
        Ok(Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span,
            attributes: vec![],
        })
    }
}