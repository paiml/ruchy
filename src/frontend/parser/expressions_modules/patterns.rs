//! Pattern parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Pattern, Literal, Span};
use crate::frontend::ast::StructPatternField;
use anyhow::bail;

/// Parse a pattern
pub fn parse_pattern(state: &mut ParserState) -> Result<Pattern> {
    let pattern = parse_or_pattern(state)?;
    Ok(pattern)
}

/// Parse OR pattern (pattern | pattern | ...)
fn parse_or_pattern(state: &mut ParserState) -> Result<Pattern> {
    let mut patterns = vec![parse_primary_pattern(state)?];
    
    while state.peek_matches(&Token::Pipe) {
        state.advance();
        patterns.push(parse_primary_pattern(state)?);
    }
    
    if patterns.len() == 1 {
        Ok(patterns.into_iter().next().unwrap())
    } else {
        Ok(Pattern::Or(patterns))
    }
}

/// Parse primary pattern
fn parse_primary_pattern(state: &mut ParserState) -> Result<Pattern> {
    let (token, _span) = state.peek_token()?;
    
    match token {
        Token::Underscore => parse_wildcard_pattern(state),
        Token::Integer(_) | Token::Float(_) | Token::String(_) | 
        Token::Char(_) | Token::Bool(_) | Token::None => parse_literal_pattern(state),
        Token::LeftParen => parse_tuple_pattern(state),
        Token::LeftBracket => parse_list_pattern(state),
        Token::Identifier(_) => parse_identifier_or_struct_pattern(state),
        Token::DotDot => parse_rest_pattern(state),
        Token::Ok | Token::Err | Token::Some => parse_result_option_pattern(state),
        _ => bail!("Unexpected token in pattern: {:?}", token),
    }
}

/// Parse wildcard pattern (_)
fn parse_wildcard_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.expect_token(Token::Underscore)?;
    Ok(Pattern::Wildcard)
}

/// Parse literal pattern
fn parse_literal_pattern(state: &mut ParserState) -> Result<Pattern> {
    let (token, _) = state.next_token()?;
    
    let literal = match token {
        Token::Integer(val) => {
            let value = val.parse::<i64>()
                .map_err(|e| anyhow::anyhow!("Invalid integer in pattern: {}", e))?;
            Literal::Integer(value)
        }
        Token::Float(val) => {
            let value = val.parse::<f64>()
                .map_err(|e| anyhow::anyhow!("Invalid float in pattern: {}", e))?;
            Literal::Float(value)
        }
        Token::String(val) => Literal::String(val),
        Token::Char(val) => {
            let ch = val.chars().next()
                .ok_or_else(|| anyhow::anyhow!("Empty char literal in pattern"))?;
            Literal::Char(ch)
        }
        Token::Bool(val) => Literal::Bool(val),
        Token::None => Literal::None,
        _ => bail!("Expected literal in pattern"),
    };
    
    Ok(Pattern::Literal(literal))
}

/// Parse tuple pattern
pub fn parse_tuple_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.expect_token(Token::LeftParen)?;
    
    // Check for unit pattern ()
    if state.peek_matches(&Token::RightParen) {
        state.advance();
        return Ok(Pattern::Tuple(vec![]));
    }
    
    let mut patterns = vec![parse_pattern(state)?];
    
    while state.peek_matches(&Token::Comma) {
        state.advance();
        
        // Allow trailing comma
        if state.peek_matches(&Token::RightParen) {
            break;
        }
        
        patterns.push(parse_pattern(state)?);
    }
    
    state.expect_token(Token::RightParen)?;
    
    // Single element without comma is just a grouped pattern
    if patterns.len() == 1 && !state.last_was_comma() {
        Ok(patterns.into_iter().next().unwrap())
    } else {
        Ok(Pattern::Tuple(patterns))
    }
}

/// Parse list pattern
pub fn parse_list_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.expect_token(Token::LeftBracket)?;
    
    // Check for empty list []
    if state.peek_matches(&Token::RightBracket) {
        state.advance();
        return Ok(Pattern::List(vec![]));
    }
    
    let mut patterns = Vec::new();
    
    loop {
        // Check for rest pattern
        if state.peek_matches(&Token::DotDot) {
            patterns.push(parse_rest_pattern(state)?);
        } else {
            patterns.push(parse_pattern(state)?);
        }
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
        
        // Allow trailing comma
        if state.peek_matches(&Token::RightBracket) {
            break;
        }
    }
    
    state.expect_token(Token::RightBracket)?;
    Ok(Pattern::List(patterns))
}

/// Parse identifier or struct pattern
fn parse_identifier_or_struct_pattern(state: &mut ParserState) -> Result<Pattern> {
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected identifier in pattern");
    };
    
    // Check for struct pattern
    if state.peek_matches(&Token::LeftBrace) {
        parse_struct_pattern_fields(state, name)
    }
    // Check for qualified name (Enum::Variant)
    else if state.peek_matches(&Token::ColonColon) {
        parse_qualified_pattern(state, name)
    }
    // Simple identifier
    else {
        Ok(Pattern::Identifier(name))
    }
}

/// Parse struct pattern fields
fn parse_struct_pattern_fields(state: &mut ParserState, name: String) -> Result<Pattern> {
    state.expect_token(Token::LeftBrace)?;
    
    let mut fields = Vec::new();
    let mut has_rest = false;
    
    while !state.peek_matches(&Token::RightBrace) {
        if state.peek_matches(&Token::DotDot) {
            state.advance();
            has_rest = true;
            break;
        }
        
        let field = parse_struct_field_pattern(state)?;
        fields.push(field);
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightBrace)?;
    
    Ok(Pattern::Struct {
        name,
        fields,
        has_rest,
    })
}

/// Parse single struct field pattern
fn parse_struct_field_pattern(state: &mut ParserState) -> Result<StructPatternField> {
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected field name in struct pattern");
    };
    
    // Check for shorthand (just field name)
    if state.peek_matches(&Token::Comma) || state.peek_matches(&Token::RightBrace) {
        Ok(StructPatternField {
            name,
            pattern: None,
        })
    }
    // Full form (field: pattern)
    else {
        state.expect_token(Token::Colon)?;
        let pattern = parse_pattern(state)?;
        Ok(StructPatternField {
            name,
            pattern: Some(pattern),
        })
    }
}

/// Parse qualified pattern (Enum::Variant, etc.)
fn parse_qualified_pattern(state: &mut ParserState, first: String) -> Result<Pattern> {
    let mut parts = vec![first];
    
    while state.peek_matches(&Token::ColonColon) {
        state.advance();
        let (Token::Identifier(part), _) = state.next_token()? else {
            bail!("Expected identifier after ::");
        };
        parts.push(part);
    }
    
    Ok(Pattern::QualifiedName(parts))
}

/// Parse rest pattern (.. or ..name)
fn parse_rest_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.expect_token(Token::DotDot)?;
    
    // Check for named rest pattern
    if let Ok((Token::Identifier(name), _)) = state.peek_token() {
        state.advance();
        Ok(Pattern::RestNamed(name))
    } else {
        Ok(Pattern::Rest)
    }
}

/// Parse Result/Option pattern
fn parse_result_option_pattern(state: &mut ParserState) -> Result<Pattern> {
    let (token, _) = state.next_token()?;
    
    match token {
        Token::Ok => {
            let inner = parse_pattern_argument(state)?;
            Ok(Pattern::Ok(Box::new(inner)))
        }
        Token::Err => {
            let inner = parse_pattern_argument(state)?;
            Ok(Pattern::Err(Box::new(inner)))
        }
        Token::Some => {
            let inner = parse_pattern_argument(state)?;
            Ok(Pattern::Some(Box::new(inner)))
        }
        _ => bail!("Expected Ok, Err, or Some pattern"),
    }
}

/// Parse pattern argument (in parentheses)
fn parse_pattern_argument(state: &mut ParserState) -> Result<Pattern> {
    state.expect_token(Token::LeftParen)?;
    let pattern = parse_pattern(state)?;
    state.expect_token(Token::RightParen)?;
    Ok(pattern)
}

/// Parse range pattern
pub fn parse_range_pattern(state: &mut ParserState, start: Pattern) -> Result<Pattern> {
    let inclusive = if state.peek_matches(&Token::DotDotEquals) {
        state.advance();
        true
    } else if state.peek_matches(&Token::DotDot) {
        state.advance();
        false
    } else {
        return Ok(start);
    };
    
    let end = Box::new(parse_primary_pattern(state)?);
    
    Ok(Pattern::Range {
        start: Box::new(start),
        end,
        inclusive,
    })
}