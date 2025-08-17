//! Collections parsing (lists, dataframes, comprehensions, blocks, object literals)

use super::{ParserState, *};
use crate::frontend::ast::ObjectField;

/// Parse a block expression or object literal
///
/// Blocks are sequences of expressions enclosed in braces `{}`. This function
/// intelligently detects whether the content represents a block of statements
/// or an object literal based on the syntax patterns.
///
/// # Examples
///
/// ```
/// // Block with multiple statements
/// { let x = 5; x + 1 }
///
/// // Object literal
/// { name: "John", age: 30 }
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
                                value: Box::new(value),
                                body: Box::new(body),
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

    Ok(Expr::new(ExprKind::Block(exprs), start_span))
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

    // Check for identifier/string followed by colon
    match state.tokens.peek() {
        Some((Token::Identifier(_) | Token::String(_), _)) => {
            // Look ahead for colon
            let saved_pos = state.tokens.position();
            state.tokens.advance(); // skip identifier/string
            let has_colon = matches!(state.tokens.peek(), Some((Token::Colon, _)));
            state.tokens.set_position(saved_pos); // restore position
            has_colon
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
/// // Key-value pairs
/// { name: "John", age: 30 }
///
/// // With spread
/// { ...defaults, name: "Jane" }
///
/// // String keys
/// { "complex key": value }
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
                Some((Token::String(s), _)) => {
                    let key = s.clone();
                    state.tokens.advance();
                    key
                }
                _ => bail!("Expected identifier or string key in object literal"),
            };

            state.tokens.expect(&Token::Colon)?;
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
/// // Simple list
/// [1, 2, 3]
///
/// // Empty list
/// []
///
/// // List comprehension
/// [x * 2 for x in numbers]
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
/// // Basic comprehension
/// [x * 2 for x in range(10)]
///
/// // With condition
/// [x for x in numbers if x > 0]
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
/// DataFrame {
///     name, age, city;
///     "Alice", 30, "NYC";
///     "Bob", 25, "LA"
/// }
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
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume DataFrame

    // Support both df![] and df{} syntax
    let use_bracket_syntax = if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
        state.tokens.advance(); // consume !
        state.tokens.expect(&Token::LeftBracket)?;
        true
    } else {
        state.tokens.expect(&Token::LeftBrace)?;
        false
    };

    // Parse column names (first row should be column identifiers)
    let mut columns = Vec::new();
    let mut rows = Vec::new();

    let end_token = if use_bracket_syntax {
        Token::RightBracket
    } else {
        Token::RightBrace
    };

    loop {
        // Check for end token
        if let Some((ref t, _)) = state.tokens.peek() {
            if *t == end_token {
                break;
            }
        }

        if !columns.is_empty() {
            // Expect semicolon between rows
            if !matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
                bail!("Expected ';' between DataFrame rows");
            }
            state.tokens.advance(); // consume ;

            // Check for trailing semicolon
            if let Some((ref t, _)) = state.tokens.peek() {
                if *t == end_token {
                    break;
                }
            }
        }

        let mut row = Vec::new();

        if columns.is_empty() {
            // Parse column names
            loop {
                // Check for end of columns
                if let Some((ref t, _)) = state.tokens.peek() {
                    if *t == Token::Semicolon || *t == end_token {
                        break;
                    }
                }
                if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    columns.push(name.clone());
                    state.tokens.advance();
                } else {
                    bail!("Expected column name");
                }

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                } else {
                    break;
                }
            }
        } else {
            // Parse data values
            loop {
                // Check for end of row
                if let Some((ref t, _)) = state.tokens.peek() {
                    if *t == Token::Semicolon || *t == end_token {
                        break;
                    }
                }
                row.push(super::parse_expr_recursive(state)?);

                // Check for comma or end of row
                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                } else {
                    break;
                }
            }
            rows.push(row);
        }
    }

    state.tokens.expect(&end_token)?;

    Ok(Expr::new(ExprKind::DataFrame { columns, rows }, start_span))
}
