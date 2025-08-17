//! Parsing utilities and helper functions

use super::{ParserState, *};

pub fn parse_params(state: &mut ParserState) -> Result<Vec<Param>> {
    state.tokens.expect(&Token::LeftParen)?;

    let mut params = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected parameter name");
        };

        // Type annotation is optional for gradual typing
        let ty = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            state.tokens.advance(); // consume :
            parse_type(state)?
        } else {
            // Default to 'Any' type for untyped parameters
            Type {
                kind: TypeKind::Named("Any".to_string()),
                span: Span { start: 0, end: 0 },
            }
        };

        params.push(Param {
            name,
            ty,
            span: Span { start: 0, end: 0 },
        });

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(params)
}

pub fn parse_type_parameters(state: &mut ParserState) -> Result<Vec<String>> {
    state.tokens.expect(&Token::Less)?;

    let mut type_params = Vec::new();

    // Parse first type parameter
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        type_params.push(name.clone());
        state.tokens.advance();
    }

    // Parse additional type parameters
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            type_params.push(name.clone());
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::Greater)?;
    Ok(type_params)
}

pub fn parse_type(state: &mut ParserState) -> Result<Type> {
    let span = Span { start: 0, end: 0 }; // Simplified for now

    match state.tokens.peek() {
        Some((Token::LeftBracket, _)) => {
            state.tokens.advance(); // consume [
            let inner = parse_type(state)?;
            state.tokens.expect(&Token::RightBracket)?;
            // List type: [T]
            Ok(Type {
                kind: TypeKind::List(Box::new(inner)),
                span,
            })
        }
        Some((Token::LeftParen, _)) => {
            state.tokens.advance(); // consume (
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                // Unit type: ()
                state.tokens.advance();
                Ok(Type {
                    kind: TypeKind::Named("()".to_string()),
                    span,
                })
            } else {
                // Function type: (T1, T2) -> T3
                let mut param_types = Vec::new();
                param_types.push(parse_type(state)?);

                while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                    param_types.push(parse_type(state)?);
                }

                state.tokens.expect(&Token::RightParen)?;
                state.tokens.expect(&Token::Arrow)?;
                let ret_type = parse_type(state)?;

                Ok(Type {
                    kind: TypeKind::Function {
                        params: param_types,
                        ret: Box::new(ret_type),
                    },
                    span,
                })
            }
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();

            // Check for generic types: Vec<T>, Result<T, E>
            if matches!(state.tokens.peek(), Some((Token::Less, _))) {
                state.tokens.advance(); // consume <

                let mut type_params = Vec::new();
                type_params.push(parse_type(state)?);

                while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                    type_params.push(parse_type(state)?);
                }

                state.tokens.expect(&Token::Greater)?;

                // Use Generic TypeKind for parameterized types
                Ok(Type {
                    kind: TypeKind::Generic {
                        base: name,
                        params: type_params,
                    },
                    span,
                })
            } else {
                Ok(Type {
                    kind: TypeKind::Named(name),
                    span,
                })
            }
        }
        _ => bail!("Expected type"),
    }
}

pub fn parse_import(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume import/use

    let mut path_parts = Vec::new();

    // Parse the path (e.g., std::io::prelude)
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        path_parts.push(name.clone());
        state.tokens.advance();

        while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
            // Check for ::
            state.tokens.advance(); // consume ::
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                path_parts.push(name.clone());
                state.tokens.advance();
            }
        }
    }

    // Check for specific imports like ::{Read, Write}
    let items = if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        state.tokens.expect(&Token::LeftBrace)?; // consume {

        let mut items = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                items.push(name.clone());
                state.tokens.advance();

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        state.tokens.expect(&Token::RightBrace)?;
        items
    } else {
        Vec::new()
    };

    let path = path_parts.join("::");
    let span = start_span; // simplified for now

    Ok(Expr::new(ExprKind::Import { path, items }, span))
}

pub fn parse_attributes(state: &mut ParserState) -> Result<Vec<Attribute>> {
    let mut attributes = Vec::new();

    while matches!(state.tokens.peek(), Some((Token::Hash, _))) {
        state.tokens.advance(); // consume #

        if !matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
            bail!("Expected '[' after '#'");
        }
        state.tokens.advance(); // consume [

        let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected attribute name");
        };

        let mut args = Vec::new();
        if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            state.tokens.advance(); // consume (

            while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                if let Some((Token::Identifier(arg), _)) = state.tokens.peek() {
                    args.push(arg.clone());
                    state.tokens.advance();

                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            state.tokens.advance(); // consume )
        }

        let end_span = state.tokens.advance().expect("Expected ']' token").1; // consume ]

        attributes.push(Attribute {
            name,
            args,
            span: end_span,
        });
    }

    Ok(attributes)
}

/// Parse string interpolation from a string containing {expr} patterns
pub fn parse_string_interpolation(_state: &mut ParserState, s: &str) -> Result<Vec<StringPart>> {
    let mut parts = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current_text = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.peek() == Some(&'{') => {
                // Escaped brace: {{
                chars.next(); // consume second '{'
                current_text.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                // Escaped brace: }}
                chars.next(); // consume second '}'
                current_text.push('}');
            }
            '{' => {
                // Start of interpolation
                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                // Collect expression until closing '}'
                let mut expr_text = String::new();
                let mut brace_count = 1;

                for expr_ch in chars.by_ref() {
                    match expr_ch {
                        '{' => {
                            brace_count += 1;
                            expr_text.push(expr_ch);
                        }
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                break;
                            }
                            expr_text.push(expr_ch);
                        }
                        _ => expr_text.push(expr_ch),
                    }
                }

                // Parse the expression
                let mut expr_parser = super::core::Parser::new(&expr_text);
                match expr_parser.parse() {
                    Ok(expr) => parts.push(StringPart::Expr(Box::new(expr))),
                    Err(_) => {
                        // Fallback to text if parsing fails
                        parts.push(StringPart::Text(format!("{{{expr_text}}}")));
                    }
                }
            }
            _ => current_text.push(ch),
        }
    }

    // Add remaining text
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }

    Ok(parts)
}
