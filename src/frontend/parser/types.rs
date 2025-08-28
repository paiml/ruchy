//! Type-related parsing (struct, trait, impl)

use super::{ParserState, *};

/// Parse an enum definition
///
/// # Errors
///
/// Returns an error if the enum syntax is invalid
pub fn parse_enum(state: &mut ParserState) -> Result<Expr> {
    parse_enum_with_visibility(state, false)
}

pub fn parse_enum_with_visibility(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume enum

    // Parse enum name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected enum name");
    };

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)?
    } else {
        Vec::new()
    };

    // Parse enum variants
    state.tokens.expect(&Token::LeftBrace)?;

    let mut variants = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse variant name
        let variant_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected variant name");
        };

        // Check for tuple variant (has parentheses)
        let fields = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            state.tokens.advance(); // consume (

            let mut field_types = Vec::new();
            while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                field_types.push(utils::parse_type(state)?);

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                } else {
                    break;
                }
            }

            state.tokens.expect(&Token::RightParen)?;
            Some(field_types)
        } else {
            None // Unit variant
        };

        variants.push(EnumVariant {
            name: variant_name,
            fields,
        });

        // Handle comma or end of enum
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Enum {
            name,
            type_params,
            variants,
            is_pub,
        },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_struct(state: &mut ParserState) -> Result<Expr> {
    parse_struct_with_visibility(state, false)
}

pub fn parse_struct_with_visibility(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume struct

    // Parse struct name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected struct name");
    };

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)?
    } else {
        Vec::new()
    };

    // Parse struct fields
    state.tokens.expect(&Token::LeftBrace)?;

    let mut fields = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse field visibility (pub is optional)
        let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
            state.tokens.advance();
            true
        } else {
            false
        };

        // Parse field name
        let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name");
        };

        // Parse type annotation (optional for shorthand syntax)
        let ty = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            state.tokens.advance(); // consume :
            utils::parse_type(state)?
        } else {
            // Default to Any type for shorthand syntax like { x, y }
            Type {
                kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                span: crate::frontend::ast::Span { start: 0, end: 0 },
            }
        };

        fields.push(StructField {
            name: field_name,
            ty,
            is_pub,
        });

        // Handle comma or end of struct
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Struct {
            name,
            type_params,
            fields,
            is_pub,
        },
        start_span,
    ))
}

pub fn parse_struct_literal(
    state: &mut ParserState,
    name: String,
    start_span: Span,
) -> Result<Expr> {
    state.tokens.expect(&Token::LeftBrace)?;

    let mut fields = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse field name
        let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name");
        };

        // Parse colon and value
        state.tokens.expect(&Token::Colon)?;
        let value = super::parse_expr_recursive(state)?;

        fields.push((field_name, value));

        // Handle comma or end of struct literal
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::StructLiteral { name, fields },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_trait(state: &mut ParserState) -> Result<Expr> {
    parse_trait_with_visibility(state, false)
}

pub fn parse_trait_with_visibility(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume trait

    // Parse trait name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected trait name");
    };

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)?
    } else {
        Vec::new()
    };

    // Parse trait body
    state.tokens.expect(&Token::LeftBrace)?;

    let mut methods = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse method
        let method = parse_trait_method(state)?;
        methods.push(method);

        // Handle semicolon or comma
        if matches!(
            state.tokens.peek(),
            Some((Token::Semicolon | Token::Comma, _))
        ) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Trait {
            name,
            type_params,
            methods,
            is_pub,
        },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_trait_method(state: &mut ParserState) -> Result<TraitMethod> {
    // Parse optional pub keyword
    let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance(); // consume pub
        true
    } else {
        false
    };

    // Parse fn or fun keyword
    if !matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
        bail!("Expected 'fun' or 'fn' keyword");
    }
    state.tokens.advance(); // consume fun/fn

    // Parse method name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected method name");
    };

    // Parse parameters
    let params = utils::parse_params(state)?;

    // Parse return type if present
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Check for method body (default implementation) or just signature
    let body = if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        Some(Box::new(super::parse_expr_recursive(state)?))
    } else {
        None
    };

    Ok(TraitMethod {
        name,
        params,
        return_type,
        body,
        is_pub,
    })
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_impl(state: &mut ParserState) -> Result<Expr> {
    parse_impl_with_visibility(state, false)
}

pub fn parse_impl_with_visibility(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume impl

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)?
    } else {
        Vec::new()
    };

    // Parse trait name (optional) and for_type
    let (trait_name, for_type) =
        if let Some((Token::Identifier(first_name), _)) = state.tokens.peek() {
            let first_name = first_name.clone();
            state.tokens.advance();

            if matches!(state.tokens.peek(), Some((Token::For, _))) {
                // impl TraitName for TypeName
                state.tokens.advance(); // consume for
                if let Some((Token::Identifier(type_name), _)) = state.tokens.peek() {
                    let type_name = type_name.clone();
                    state.tokens.advance();
                    (Some(first_name), type_name)
                } else {
                    bail!("Expected type name after 'for'");
                }
            } else {
                // impl TypeName (inherent impl)
                (None, first_name)
            }
        } else {
            bail!("Expected identifier after 'impl'");
        };

    // Parse impl body
    state.tokens.expect(&Token::LeftBrace)?;

    let mut methods = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse method implementation
        let method = parse_impl_method(state)?;
        methods.push(method);

        // Skip optional semicolons
        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Impl {
            type_params,
            trait_name,
            for_type,
            methods,
            is_pub,
        },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_impl_method(state: &mut ParserState) -> Result<ImplMethod> {
    // Parse optional pub keyword
    let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance(); // consume pub
        true
    } else {
        false
    };

    // Parse fn or fun keyword
    if !matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
        bail!("Expected 'fun' or 'fn' keyword");
    }
    state.tokens.advance(); // consume fun/fn

    // Parse method name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected method name");
    };

    // Parse parameters
    let params = utils::parse_params(state)?;

    // Parse return type if present
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse method body (required for impl)
    let body = super::parse_expr_recursive(state)?;

    Ok(ImplMethod {
        name,
        params,
        return_type,
        body: Box::new(body),
        is_pub,
    })
}

/// Parse extension methods: `extend Type { ... }`
///
/// # Errors
///
/// Returns an error if the parsing fails
pub fn parse_extend(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume extend

    // Parse target type
    let target_type = if let Some((Token::Identifier(type_name), _)) = state.tokens.peek() {
        let type_name = type_name.clone();
        state.tokens.advance();
        type_name
    } else {
        bail!("Expected type name after 'extend'");
    };

    // Parse extension body
    state.tokens.expect(&Token::LeftBrace)?;

    let mut methods = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse method implementation (reuse impl method parser)
        let method = parse_impl_method(state)?;
        methods.push(method);

        // Skip optional semicolons
        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Extension {
            target_type,
            methods,
        },
        start_span,
    ))
}
