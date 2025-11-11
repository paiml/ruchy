//! Struct definition parsing
//!
//! Handles parsing of struct (record type) definitions:
//! - Named structs: `struct Point { x: f64, y: f64 }`
//! - Tuple structs: `struct Color(u8, u8, u8)`
//! - Unit structs: `struct Marker`
//! - Generic structs: `struct Container<T> { value: T }`
//! - Field visibility: `pub`, `pub(crate)`, `private`
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{ClassMethod, Expr, ExprKind, SelfType, Span, StructField, Type, Visibility};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

pub(in crate::frontend::parser) fn parse_struct_variant(
    state: &mut ParserState,
    name: String,
    type_params: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
    match state.tokens.peek() {
        Some((Token::LeftParen, _)) => {
            let fields = parse_tuple_struct_fields(state)?;
            Ok(Expr::new(
                ExprKind::TupleStruct {
                    name,
                    type_params,
                    fields,
                    derives: Vec::new(),
                    is_pub: false,
                },
                start_span,
            ))
        }
        Some((Token::LeftBrace, _)) => {
            let (fields, methods) = parse_struct_fields(state)?;
            Ok(Expr::new(
                ExprKind::Struct {
                    name,
                    type_params,
                    fields,
                    methods,
                    derives: Vec::new(),
                    is_pub: false,
                },
                start_span,
            ))
        }
        _ => Ok(Expr::new(
            ExprKind::Struct {
                name,
                type_params,
                fields: Vec::new(),
                methods: Vec::new(),
                derives: Vec::new(),
                is_pub: false,
            },
            start_span,
        )),
    }
}

pub(in crate::frontend::parser) fn parse_struct_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected struct name after 'struct'");
    }
}

fn parse_tuple_struct_fields(state: &mut ParserState) -> Result<Vec<Type>> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        fields.push(utils::parse_type(state)?);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(fields)
}

fn parse_struct_fields(state: &mut ParserState) -> Result<(Vec<StructField>, Vec<ClassMethod>)> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // DEFECT-PARSER-007: Skip comments before member declaration
        while matches!(state.tokens.peek(), Some((Token::LineComment(_) | Token::BlockComment(_) | Token::DocComment(_) | Token::HashComment(_), _))) {
            state.tokens.advance();
        }

        // PARSER-147: Check if this is a method definition (with or without pub)
        if is_method_definition(state) {
            let method = parse_struct_method_with_visibility(state)?;
            methods.push(method);
        } else {
            // Parse field
            let (visibility, is_mut) = parse_struct_field_modifiers(state)?;
            let (field_name, field_type, default_value) = parse_single_struct_field(state)?;

            fields.push(StructField {
                name: field_name,
                ty: field_type,
                visibility,
                is_mut,
                default_value,
                decorators: vec![],
            });
        }

        // DEFECT-PARSER-007: Skip any inline comments after member definition
        while matches!(state.tokens.peek(), Some((Token::LineComment(_) | Token::BlockComment(_) | Token::DocComment(_) | Token::HashComment(_), _))) {
            state.tokens.advance();
        }

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();

            // Skip comments after comma (allows multiline definitions with comments)
            while matches!(state.tokens.peek(), Some((Token::LineComment(_) | Token::BlockComment(_) | Token::DocComment(_) | Token::HashComment(_), _))) {
                state.tokens.advance();
            }
        }
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok((fields, methods))
}

fn parse_struct_field_modifiers(state: &mut ParserState) -> Result<(Visibility, bool)> {
    let visibility = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        parse_pub_visibility(state)?
    } else if matches!(state.tokens.peek(), Some((Token::Private, _))) {
        parse_private_keyword(state);
        Visibility::Private
    } else {
        Visibility::Private
    };

    let is_mut = parse_mut_modifier(state);
    Ok((visibility, is_mut))
}

fn parse_pub_visibility(state: &mut ParserState) -> Result<Visibility> {
    state.tokens.expect(&Token::Pub)?;

    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_scoped_visibility(state)
    } else {
        Ok(Visibility::Public)
    }
}

fn parse_scoped_visibility(state: &mut ParserState) -> Result<Visibility> {
    state.tokens.expect(&Token::LeftParen)?;

    // PARSER-074: Match Token::Crate and Token::Super (not Identifier)
    let visibility = match state.tokens.peek() {
        Some((Token::Crate, _)) => {
            state.tokens.advance();
            Visibility::PubCrate
        }
        Some((Token::Super, _)) => {
            state.tokens.advance();
            Visibility::PubSuper
        }
        _ => Visibility::Public,
    };

    state.tokens.expect(&Token::RightParen)?;
    Ok(visibility)
}

fn parse_mut_modifier(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

fn parse_private_keyword(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Private, _))) {
        state.tokens.advance();
    }
}

pub(in crate::frontend::parser) fn parse_single_struct_field(state: &mut ParserState) -> Result<(String, Type, Option<Expr>)> {
    let field_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected field name");
    };

    state.tokens.expect(&Token::Colon)?;
    let field_type = utils::parse_type(state)?;

    let default_value = if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance();
        Some(parse_expr_recursive(state)?)
    } else {
        None
    };

    Ok((field_name, field_type, default_value))
}

// PARSER-147: Helper to detect if next tokens are a method definition
fn is_method_definition(state: &mut ParserState) -> bool {
    // Check for: fun/fn OR pub fun/fn
    match state.tokens.peek() {
        Some((Token::Fun | Token::Fn, _)) => true,
        Some((Token::Pub, _)) => {
            // Lookahead: check if next token after pub is fun/fn
            matches!(state.tokens.peek_ahead(1), Some((Token::Fun | Token::Fn, _)))
        }
        _ => false,
    }
}

// PARSER-147: Parse method with optional pub visibility modifier
fn parse_struct_method_with_visibility(state: &mut ParserState) -> Result<ClassMethod> {
    // Parse optional pub keyword
    let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance();
        true
    } else {
        false
    };

    // Parse the method (reuse existing logic)
    let mut method = parse_struct_method(state)?;

    // Update visibility
    method.is_pub = is_pub;

    Ok(method)
}

fn parse_struct_method(state: &mut ParserState) -> Result<ClassMethod> {
    // Expect 'fun' or 'fn' keyword
    match state.tokens.peek() {
        Some((Token::Fun, _)) => {
            state.tokens.advance();
        }
        Some((Token::Fn, _)) => {
            state.tokens.advance();
        }
        _ => bail!("Expected 'fun' or 'fn' keyword for method definition"),
    }

    // Parse method name
    let method_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected method name after 'fun'");
    };

    // Parse parameter list (including self parameter)
    let params = utils::parse_params(state)?;

    // Determine self type from first parameter
    let self_type = determine_self_type(&params);

    // Parse optional return type
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse method body
    let body = Box::new(parse_expr_recursive(state)?);

    Ok(ClassMethod {
        name: method_name,
        params,
        return_type,
        body,
        is_pub: false,
        is_static: matches!(self_type, SelfType::None),
        is_override: false,
        is_final: false,
        is_abstract: false,
        is_async: false,
        self_type,
    })
}

fn determine_self_type(params: &[crate::frontend::ast::Param]) -> SelfType {
    if !params.is_empty() && params[0].name() == "self" {
        use crate::frontend::ast::TypeKind;
        match &params[0].ty.kind {
            TypeKind::Reference { is_mut: true, .. } => SelfType::MutBorrowed,
            TypeKind::Reference { is_mut: false, .. } => SelfType::Borrowed,
            _ => SelfType::Owned,
        }
    } else {
        SelfType::None
    }
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_named_struct() {
        let code = "struct Point { x: f64, y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Named struct should parse");
    }

    #[test]
    fn test_tuple_struct() {
        let code = "struct Color(u8, u8, u8)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple struct should parse");
    }

    #[test]
    fn test_unit_struct() {
        let code = "struct Marker";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Unit struct should parse");
    }

    #[test]
    fn test_generic_struct() {
        let code = "struct Container<T> { value: T }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic struct should parse");
    }

    #[test]
    fn test_pub_field() {
        let code = "struct Point { pub x: f64, y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with pub field should parse");
    }

    #[test]
    fn test_mut_field() {
        let code = "struct Counter { mut count: i32 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with mut field should parse");
    }

    #[test]
    fn test_field_with_default() {
        let code = "struct Config { timeout: i32 = 30 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with default value should parse");
    }
}
