//! Function-related parsing (function definitions, lambdas, calls)

use super::{ParserState, *};
use crate::frontend::ast::{DataFrameOp, Literal};

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_function(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume fun

    // Check for async modifier - currently not implemented in lexer
    // When async keyword is added to lexer, this will be:
    // let is_async = state.tokens.check(&Token::Async);
    let is_async = false;

    // Parse function name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        "anonymous".to_string()
    };

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)?
    } else {
        Vec::new()
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

    // Parse body
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Function {
            name,
            type_params,
            params,
            return_type,
            body: Box::new(body),
            is_async,
        },
        start_span,
    ))
}

fn parse_lambda_params(state: &mut ParserState) -> Result<Vec<Param>> {
    let mut params = Vec::new();

    // Parse parameters until we hit a pipe or arrow
    loop {
        // Check if we've reached the end of parameters
        if matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
            break;
        }

        // Parse parameter name
        let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            break; // No more parameters
        };

        // Parse optional type annotation
        let ty = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            state.tokens.advance(); // consume :
            utils::parse_type(state)?
        } else {
            // Default to inferred type - use _ as placeholder
            Type {
                kind: TypeKind::Named("_".to_string()),
                span: Span { start: 0, end: 0 },
            }
        };

        params.push(Param {
            name,
            ty,
            span: Span { start: 0, end: 0 },
            is_mutable: false,
        });

        // Check for comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }

    Ok(params)
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_empty_lambda(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume ||

    // Check for fat arrow syntax: || => expr
    if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
        state.tokens.advance(); // consume =>
    }
    // Note: Regular lambda syntax (|| expr) is also supported without =>

    // Parse the body
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Lambda {
            params: Vec::new(),
            body: Box::new(body),
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
pub fn parse_lambda(state: &mut ParserState) -> Result<Expr> {
    let start_span = state
        .tokens
        .peek()
        .map_or(Span { start: 0, end: 0 }, |(_, s)| *s);

    // Check if it's backslash syntax (\x -> ...) or pipe syntax (|x| ...)
    if matches!(state.tokens.peek(), Some((Token::Backslash, _))) {
        state.tokens.advance(); // consume \

        // Parse parameters (simple identifiers separated by commas)
        let mut params = Vec::new();

        // Parse first parameter
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            params.push(Param {
                name: name.clone(),
                ty: Type {
                    kind: TypeKind::Named("Any".to_string()),
                    span: Span { start: 0, end: 0 },
                },
                span: Span { start: 0, end: 0 },
                is_mutable: false,
            });
            state.tokens.advance();

            // Parse additional parameters
            while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance(); // consume comma
                if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    params.push(Param {
                        name: name.clone(),
                        ty: Type {
                            kind: TypeKind::Named("Any".to_string()),
                            span: Span { start: 0, end: 0 },
                        },
                        span: Span { start: 0, end: 0 },
                        is_mutable: false,
                    });
                    state.tokens.advance();
                }
            }
        }

        // Expect arrow
        state.tokens.expect(&Token::Arrow)?;

        // Parse body
        let body = super::parse_expr_recursive(state)?;

        return Ok(Expr::new(
            ExprKind::Lambda {
                params,
                body: Box::new(body),
            },
            start_span,
        ));
    }

    // Otherwise, handle pipe syntax |x| ...
    state.tokens.advance(); // consume |

    // Handle || as a special case for empty parameter lambdas
    if matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        state.tokens.advance(); // consume second |
        
        // Check for fat arrow syntax: || => expr
        if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
            state.tokens.advance(); // consume =>
        }
        // Note: Regular lambda syntax (|| expr) is also supported without =>
        
        // Parse the body
        let body = super::parse_expr_recursive(state)?;
        return Ok(Expr::new(
            ExprKind::Lambda {
                params: Vec::new(),
                body: Box::new(body),
            },
            start_span,
        ));
    }

    // Parse parameters between pipes: |x, y|
    let params = parse_lambda_params(state)?;

    // Check for empty params with single |
    if !matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        bail!("Expected '|' after lambda parameters");
    }
    state.tokens.advance(); // consume |

    // Check for fat arrow syntax: |x| => expr
    if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
        state.tokens.advance(); // consume =>
    }
    // Note: Regular lambda syntax (|x| expr) is also supported without => 

    // Parse the body
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Lambda {
            params,
            body: Box::new(body),
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
pub fn parse_call(state: &mut ParserState, func: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume (

    let mut args = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        args.push(super::parse_expr_recursive(state)?);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightParen)?;

    Ok(Expr {
        kind: ExprKind::Call {
            func: Box::new(func),
            args,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
    })
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
#[allow(clippy::too_many_lines)]
pub fn parse_method_call(state: &mut ParserState, receiver: Expr) -> Result<Expr> {
    // Check for special postfix operators like .await
    if let Some((Token::Await, _)) = state.tokens.peek() {
        state.tokens.advance(); // consume await
        return Ok(Expr {
            kind: ExprKind::Await {
                expr: Box::new(receiver),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        });
    }

    // Parse method name
    let method = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected method name or 'await' after '.'");
    };

    // Check if this is a DataFrame operation method
    // Common DataFrame methods: filter, select, groupby, sort, head, tail, etc.
    let is_dataframe_method = matches!(
        method.as_str(),
        "filter"
            | "select"
            | "groupby"
            | "group_by"
            | "sort"
            | "sort_by"
            | "head"
            | "tail"
            | "limit"
            | "join"
            | "mean"
            | "sum"
            | "count"
            | "min"
            | "max"
            | "std"
            | "var"
            | "median"
    );

    // Check if it's a method call (with parentheses) or field access
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        // Method call
        state.tokens.advance(); // consume (

        let mut args = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            args.push(super::parse_expr_recursive(state)?);

            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance(); // consume comma
            } else {
                break;
            }
        }

        state.tokens.expect(&Token::RightParen)?;

        // Check if this is a DataFrame operation
        if is_dataframe_method {
            // Convert to DataFrame operation based on method name
            let operation = match method.as_str() {
                "filter" => {
                    if args.len() != 1 {
                        bail!("filter() expects exactly 1 argument");
                    }
                    DataFrameOp::Filter(Box::new(args.into_iter().next().expect("checked length")))
                }
                "select" => {
                    // Extract column names from arguments
                    let mut columns = Vec::new();
                    for arg in args {
                        match arg.kind {
                            // Handle bare identifiers: .select(age, name)
                            ExprKind::Identifier(name) => {
                                columns.push(name);
                            }
                            // Handle list literals: .select(["age", "name"])  
                            ExprKind::List(items) => {
                                for item in items {
                                    if let ExprKind::Literal(Literal::String(col_name)) = item.kind {
                                        columns.push(col_name);
                                    }
                                }
                            }
                            // Handle single string literals: .select("age")
                            ExprKind::Literal(Literal::String(col_name)) => {
                                columns.push(col_name);
                            }
                            _ => {}
                        }
                    }
                    DataFrameOp::Select(columns)
                }
                "groupby" | "group_by" => {
                    let columns = args
                        .into_iter()
                        .filter_map(|arg| {
                            if let ExprKind::Identifier(name) = arg.kind {
                                Some(name)
                            } else {
                                None
                            }
                        })
                        .collect();
                    DataFrameOp::GroupBy(columns)
                }
                "sort" | "sort_by" => {
                    let columns = args
                        .into_iter()
                        .filter_map(|arg| {
                            if let ExprKind::Identifier(name) = arg.kind {
                                Some(name)
                            } else {
                                None
                            }
                        })
                        .collect();
                    DataFrameOp::Sort(columns)
                }
                "head" => {
                    let n = if args.is_empty() {
                        5 // default
                    } else if args.len() == 1 {
                        if let ExprKind::Literal(Literal::Integer(n)) = args[0].kind {
                            usize::try_from(n.max(0)).unwrap_or(5)
                        } else {
                            bail!("head() expects an integer argument");
                        }
                    } else {
                        bail!("head() expects 0 or 1 arguments");
                    };
                    DataFrameOp::Head(n)
                }
                "tail" => {
                    let n = if args.is_empty() {
                        5 // default
                    } else if args.len() == 1 {
                        if let ExprKind::Literal(Literal::Integer(n)) = args[0].kind {
                            usize::try_from(n.max(0)).unwrap_or(5)
                        } else {
                            bail!("tail() expects an integer argument");
                        }
                    } else {
                        bail!("tail() expects 0 or 1 arguments");
                    };
                    DataFrameOp::Tail(n)
                }
                "limit" => {
                    if args.len() != 1 {
                        bail!("limit() expects exactly 1 argument");
                    }
                    if let ExprKind::Literal(Literal::Integer(n)) = args[0].kind {
                        DataFrameOp::Limit(usize::try_from(n.max(0)).unwrap_or(10))
                    } else {
                        bail!("limit() expects an integer argument");
                    }
                }
                _ => {
                    // For other methods, fall back to regular method call
                    return Ok(Expr {
                        kind: ExprKind::MethodCall {
                            receiver: Box::new(receiver),
                            method,
                            args,
                        },
                        span: Span { start: 0, end: 0 },
                        attributes: Vec::new(),
                    });
                }
            };

            Ok(Expr {
                kind: ExprKind::DataFrameOperation {
                    source: Box::new(receiver),
                    operation,
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
            })
        } else {
            Ok(Expr {
                kind: ExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method,
                    args,
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
            })
        }
    } else {
        // Field access
        Ok(Expr {
            kind: ExprKind::FieldAccess {
                object: Box::new(receiver),
                field: method,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        })
    }
}
