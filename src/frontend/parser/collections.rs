//! Collections parsing (lists, dataframes, comprehensions, blocks, object literals)

use super::{ParserState, *};
use crate::frontend::ast::{DataFrameColumn, Literal, ObjectField};

/// Parse a block expression or object literal
///
/// Blocks are sequences of expressions enclosed in braces `{}`. This function
/// intelligently detects whether the content represents a block of statements
/// or an object literal based on the syntax patterns.
///
/// # Examples
///
/// ```
/// use ruchy::Parser;
///
/// let input = "{ let x = 5; x + 1 }";
/// let mut parser = Parser::new(input);
/// let result = parser.parse();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The opening brace is missing (should be handled by caller)
/// - Failed to parse any expression within the block
/// - Missing closing brace
/// - Invalid object literal syntax when detected as object
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_block(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume {

    // Check if this might be an object literal
    // Object literals have: identifier/string : expr, or ...expr patterns
    // Blocks have statements and expressions
    if is_object_literal(state) {
        return parse_object_literal_body(state, start_span);
    }

    let mut exprs = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Check if this is a let statement (let without 'in')
        if matches!(state.tokens.peek(), Some((Token::Let, _))) {
            // Peek ahead to see if this is a let-statement or let-expression
            let saved_pos = state.tokens.position();
            state.tokens.advance(); // consume let

            // Parse variable name
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let name = name.clone();
                state.tokens.advance();

                // Check for =
                if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
                    state.tokens.advance(); // consume =

                    // Parse the value expression
                    let value = super::parse_expr_recursive(state)?;

                    // Check if followed by 'in' (let-expression) or semicolon/brace (let-statement)
                    if matches!(state.tokens.peek(), Some((Token::In, _))) {
                        // It's a let-expression, restore position and parse normally
                        state.tokens.set_position(saved_pos);
                        exprs.push(super::parse_expr_recursive(state)?);
                    } else {
                        // It's a let-statement, create a synthetic let-in that binds for the rest of the block
                        // Consume optional semicolon
                        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
                            state.tokens.advance();
                        }

                        // Parse the rest of the block as the body
                        let mut body_exprs = Vec::new();
                        while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                            body_exprs.push(super::parse_expr_recursive(state)?);

                            if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
                                state.tokens.advance();
                            }

                            if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                                break;
                            }
                        }

                        // Create the body expression
                        let body = if body_exprs.is_empty() {
                            Expr::new(ExprKind::Literal(Literal::Unit), start_span)
                        } else if body_exprs.len() == 1 {
                            body_exprs.into_iter().next().expect("checked: len == 1")
                        } else {
                            Expr::new(ExprKind::Block(body_exprs), start_span)
                        };

                        // Create let-in expression
                        exprs.push(Expr::new(
                            ExprKind::Let {
                                name,
                                type_annotation: None,
                                value: Box::new(value),
                                body: Box::new(body),
                                is_mutable: false,
                            },
                            start_span,
                        ));
                        break; // The let consumed the rest of the block
                    }
                } else {
                    // Not a valid let, restore and parse as expression
                    state.tokens.set_position(saved_pos);
                    exprs.push(super::parse_expr_recursive(state)?);
                }
            } else {
                // Not a valid let, restore and parse as expression
                state.tokens.set_position(saved_pos);
                exprs.push(super::parse_expr_recursive(state)?);
            }
        } else {
            exprs.push(super::parse_expr_recursive(state)?);
        }

        // Optional semicolon
        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            state.tokens.advance();
        }

        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    // Empty blocks should be unit literals
    if exprs.is_empty() {
        Ok(Expr::new(ExprKind::Literal(Literal::Unit), start_span))
    } else {
        Ok(Expr::new(ExprKind::Block(exprs), start_span))
    }
}

/// Check if the current position looks like an object literal
///
/// Analyzes the upcoming tokens to determine if the content should be parsed
/// as an object literal rather than a regular block. Object literals have
/// specific patterns like `key: value` pairs or spread operators `...expr`.
///
/// # Examples
///
/// Returns `true` for patterns like:
/// - `{ name: "John" }`
/// - `{ ...other }`
/// - `{ "key": value }`
///
/// Returns `false` for:
/// - `{ x + 1 }`
/// - `{ let x = 5 }`
/// - `{ }`
///
/// # Errors
///
/// Returns an error if token stream operations fail during lookahead.
fn is_object_literal(state: &mut ParserState) -> bool {
    // Peek at the next few tokens to determine if this is an object literal
    // Object literal patterns:
    // 1. { key: value, ... }
    // 2. { ...spread }
    // 3. { } (empty)

    // Empty braces could be either - default to block
    if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        return false;
    }

    // Check for spread operator
    if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
        return true;
    }

    // Check for identifier/string followed by colon or fat arrow (book compatibility)
    match state.tokens.peek() {
        Some((Token::Identifier(_) | Token::String(_) | Token::RawString(_), _)) => {
            // Look ahead for colon or fat arrow
            let saved_pos = state.tokens.position();
            state.tokens.advance(); // skip identifier/string
            let has_separator = matches!(
                state.tokens.peek(),
                Some((Token::Colon | Token::FatArrow, _))
            );
            state.tokens.set_position(saved_pos); // restore position
            has_separator
        }
        _ => false,
    }
}

/// Parse the body of an object literal after the opening brace
///
/// Parses the contents of an object literal including key-value pairs and
/// spread expressions. Handles both string and identifier keys.
///
/// # Examples
///
/// ```
/// use ruchy::Parser;
///
/// let input = r#"{ name: "John", age: 30 }"#;
/// let mut parser = Parser::new(input);
/// let result = parser.parse();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Invalid key type (neither identifier nor string)
/// - Missing colon after key
/// - Failed to parse value expression
/// - Missing comma between fields
/// - Missing closing brace
fn parse_object_literal_body(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Check for spread operator
        if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
            state.tokens.advance(); // consume ...
            let expr = super::parse_expr_recursive(state)?;
            fields.push(ObjectField::Spread { expr });
        } else {
            // Parse key-value pair
            let key = match state.tokens.peek() {
                Some((Token::Identifier(name), _)) => {
                    let key = name.clone();
                    state.tokens.advance();
                    key
                }
                Some((Token::String(s) | Token::RawString(s), _)) => {
                    let key = s.clone();
                    state.tokens.advance();
                    key
                }
                _ => bail!("Expected identifier or string key in object literal"),
            };

            // Accept either : or => for object key-value pairs (book compatibility)
            if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                state.tokens.advance(); // consume =>
            } else {
                state.tokens.expect(&Token::Colon)?;
            }
            let value = super::parse_expr_recursive(state)?;
            fields.push(ObjectField::KeyValue { key, value });
        }

        // Check for comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else if !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            bail!("Expected comma or closing brace in object literal");
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(ExprKind::ObjectLiteral { fields }, start_span))
}

/// Parse a list expression or list comprehension
///
/// Parses list literals enclosed in brackets `[]`. Automatically detects
/// list comprehensions when the `for` keyword is encountered after the
/// first element.
///
/// # Examples
///
/// ```
/// use ruchy::Parser;
///
/// let input = "[1, 2, 3]";
/// let mut parser = Parser::new(input);
/// let result = parser.parse();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Failed to parse any element expression
/// - Missing closing bracket
/// - Invalid list comprehension syntax
/// - Malformed comma-separated elements
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_list(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume [

    // Check for empty list
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance(); // consume ]
        return Ok(Expr::new(ExprKind::List(Vec::new()), start_span));
    }

    // Parse the first element
    let first_element = super::parse_expr_recursive(state)?;

    // Check if this is a list comprehension by looking for 'for'
    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        return parse_list_comprehension(state, start_span, first_element);
    }

    // Regular list - continue parsing elements
    let mut elements = vec![first_element];

    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma

        if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
            break; // trailing comma
        }

        elements.push(super::parse_expr_recursive(state)?);
    }

    state.tokens.expect(&Token::RightBracket)?;

    Ok(Expr::new(ExprKind::List(elements), start_span))
}

/// Parse a list comprehension after the element expression
///
/// Parses the remaining parts of a list comprehension: the `for` clause,
/// variable binding, iterable expression, and optional `if` condition.
///
/// # Examples
///
/// ```
/// use ruchy::Parser;
///
/// let input = "[x * 2 for x in range(10)]";
/// let mut parser = Parser::new(input);
/// let result = parser.parse();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Missing `for` keyword
/// - Invalid variable name
/// - Missing `in` keyword
/// - Failed to parse iterable expression
/// - Failed to parse condition expression (when present)
/// - Missing closing bracket
///
/// Parse a condition expression for list comprehension that stops at ]
fn parse_condition_expr(state: &mut ParserState) -> Result<Expr> {
    // Save the current position in case we need to backtrack
    let _start_pos = state.tokens.position();

    // Try to parse an expression, but we need to be careful about ]
    // We'll parse terms and operators manually to avoid consuming ]
    let mut left = parse_condition_term(state)?;

    // Check for comparison operators
    while let Some((token, _)) = state.tokens.peek() {
        match token {
            Token::Greater
            | Token::Less
            | Token::GreaterEqual
            | Token::LessEqual
            | Token::EqualEqual
            | Token::NotEqual
            | Token::AndAnd
            | Token::OrOr => {
                let op = expressions::token_to_binary_op(token).expect("checked: valid op");
                state.tokens.advance(); // consume operator
                let right = parse_condition_term(state)?;
                left = Expr::new(
                    ExprKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    Span { start: 0, end: 0 },
                );
            }
            _ => break, // Stop at closing bracket or any other token
        }
    }

    Ok(left)
}

/// Parse a single term in a condition expression
fn parse_condition_term(state: &mut ParserState) -> Result<Expr> {
    // Parse a primary expression (identifier, literal, call, etc.)
    let mut expr = expressions::parse_prefix(state)?;

    // Handle postfix operations like method calls and field access
    while let Some((token, _)) = state.tokens.peek() {
        match token {
            Token::Dot => {
                state.tokens.advance(); // consume .
                if let Some((Token::Identifier(method), _)) = state.tokens.peek() {
                    let method = method.clone();
                    state.tokens.advance();

                    // Check for method call
                    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                        state.tokens.advance(); // consume (
                        let mut args = Vec::new();

                        while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                            args.push(super::parse_expr_recursive(state)?);
                            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                                state.tokens.advance();
                            } else {
                                break;
                            }
                        }

                        state.tokens.expect(&Token::RightParen)?;
                        expr = Expr::new(
                            ExprKind::MethodCall {
                                receiver: Box::new(expr),
                                method,
                                args,
                            },
                            Span { start: 0, end: 0 },
                        );
                    } else {
                        // Field access
                        expr = Expr::new(
                            ExprKind::FieldAccess {
                                object: Box::new(expr),
                                field: method,
                            },
                            Span { start: 0, end: 0 },
                        );
                    }
                }
            }
            Token::LeftParen => {
                // Function call
                expr = functions::parse_call(state, expr)?;
            }
            _ => break, // Stop at other tokens
        }
    }

    Ok(expr)
}

pub fn parse_list_comprehension(
    state: &mut ParserState,
    start_span: Span,
    element: Expr,
) -> Result<Expr> {
    // We've already parsed the element expression
    // Now expect: for variable in iterable [if condition]

    state.tokens.expect(&Token::For)?;

    // Parse variable name
    let variable = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected variable name in list comprehension");
    };

    state.tokens.expect(&Token::In)?;

    // Parse iterable expression
    let iterable = super::parse_expr_recursive(state)?;

    // Check for optional if condition
    let condition = if matches!(state.tokens.peek(), Some((Token::If, _))) {
        state.tokens.advance(); // consume 'if'
                                // Parse condition expression - this needs to stop at the closing bracket
                                // We'll parse a simple expression that stops at ]
        let cond = parse_condition_expr(state)?;
        Some(Box::new(cond))
    } else {
        None
    };

    state.tokens.expect(&Token::RightBracket)?;

    Ok(Expr::new(
        ExprKind::ListComprehension {
            element: Box::new(element),
            variable,
            iterable: Box::new(iterable),
            condition,
        },
        start_span,
    ))
}

/// Parse a `DataFrame` literal expression
///
/// Parses `DataFrame` literals with column headers and data rows. The first
/// row defines column names, subsequent rows contain data values.
///
/// # Examples
///
/// ```
/// use ruchy::Parser;
///
/// let input = r#"df![name => ["Alice", "Bob"], age => [30, 25]]"#;
/// let mut parser = Parser::new(input);
/// let result = parser.parse();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Missing opening brace after `DataFrame`
/// - Invalid column name (must be identifier)
/// - Missing semicolon between rows
/// - Failed to parse data value expressions
/// - Missing closing brace
/// - Inconsistent number of values per row
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_dataframe(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume df

    // Expect ! after df
    state.tokens.expect(&Token::Bang)?;
    state.tokens.expect(&Token::LeftBracket)?;

    let mut columns = Vec::new();

    // Check for empty DataFrame df![]
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance();
        return Ok(Expr::new(ExprKind::DataFrame { columns }, start_span));
    }

    // Parse column definitions using the new syntax: df![col => values, ...]
    loop {
        // Parse column name
        let col_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected column name in DataFrame literal");
        };

        // Check for => or semicolon (legacy syntax support)
        if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
            // New syntax: col => [values]
            state.tokens.advance(); // consume =>

            // Parse values - could be a list or individual values
            let values = if matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
                // Values are in a list
                parse_list(state)?
            } else {
                // Parse individual expression
                super::parse_expr_recursive(state)?
            };

            // Convert to vector of expressions
            let value_vec = match values.kind {
                ExprKind::List(exprs) => exprs,
                _ => vec![values],
            };

            columns.push(DataFrameColumn {
                name: col_name,
                values: value_vec,
            });
        } else if matches!(state.tokens.peek(), Some((Token::Comma, _)))
            || matches!(state.tokens.peek(), Some((Token::Semicolon, _)))
            || matches!(state.tokens.peek(), Some((Token::RightBracket, _)))
        {
            // Legacy syntax: just column names, then semicolon and rows
            // For backward compatibility, create empty column for now
            columns.push(DataFrameColumn {
                name: col_name,
                values: Vec::new(),
            });
        } else {
            bail!("Expected '=>' or ',' after column name in DataFrame literal");
        }

        // Check for continuation
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            // Legacy row-based syntax
            state.tokens.advance();
            // Parse legacy rows if present
            parse_legacy_dataframe_rows(state, &mut columns)?;
            break;
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightBracket)?;

    Ok(Expr::new(ExprKind::DataFrame { columns }, start_span))
}

/// Parse legacy row-based `DataFrame` syntax for backward compatibility
#[allow(clippy::ptr_arg)] // We need to mutate the Vec, not just read it
fn parse_legacy_dataframe_rows(
    state: &mut ParserState,
    columns: &mut Vec<DataFrameColumn>,
) -> Result<()> {
    let mut rows: Vec<Vec<Expr>> = Vec::new();

    loop {
        // Check for end bracket
        if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
            break;
        }

        let mut row = Vec::new();

        // Parse row values
        loop {
            if matches!(state.tokens.peek(), Some((Token::Semicolon, _)))
                || matches!(state.tokens.peek(), Some((Token::RightBracket, _)))
            {
                break;
            }

            row.push(super::parse_expr_recursive(state)?);

            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            } else {
                break;
            }
        }

        if !row.is_empty() {
            rows.push(row);
        }

        // Check for row separator
        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }

    // Convert rows to column-based format
    if !rows.is_empty() {
        for (col_idx, column) in columns.iter_mut().enumerate() {
            for row in &rows {
                if col_idx < row.len() {
                    column.values.push(row[col_idx].clone());
                }
            }
        }
    }

    Ok(())
}
