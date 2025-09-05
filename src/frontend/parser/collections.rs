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
/// Parse a block expression { ... } (complexity: 7)
///
/// Handles both regular blocks and let-statement conversion to let-expressions
pub fn parse_block(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume {

    // Check if this might be an object literal
    if is_object_literal(state) {
        return parse_object_literal_body(state, start_span);
    }

    let exprs = parse_block_expressions(state, start_span)?;
    state.tokens.expect(&Token::RightBrace)?;
    
    Ok(create_block_result(exprs, start_span))
}

/// Parse all expressions within a block (complexity: 8)
fn parse_block_expressions(state: &mut ParserState, start_span: Span) -> Result<Vec<Expr>> {
    let mut exprs = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let expr = parse_next_block_expression(state, start_span)?;
        exprs.push(expr);
        
        consume_optional_semicolon(state);
        
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }
    }
    
    Ok(exprs)
}

/// Parse the next expression in a block, handling let statements (complexity: 9)
fn parse_next_block_expression(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        parse_potential_let_statement(state, start_span)
    } else {
        super::parse_expr_recursive(state)
    }
}

/// Handle potential let statement with lookahead (complexity: 10)
fn parse_potential_let_statement(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    let saved_pos = state.tokens.position();
    state.tokens.advance(); // consume let
    
    if let Some(let_info) = try_parse_let_binding(state)? {
        if is_let_expression(state) {
            // Let expression - restore and parse normally
            state.tokens.set_position(saved_pos);
            super::parse_expr_recursive(state)
        } else {
            // Let statement - convert to let expression
            create_let_statement_expression(state, let_info, start_span)
        }
    } else {
        // Not a valid let - restore and parse as expression
        state.tokens.set_position(saved_pos);
        super::parse_expr_recursive(state)
    }
}

/// Try to parse let binding info (complexity: 6)
fn try_parse_let_binding(state: &mut ParserState) -> Result<Option<LetBindingInfo>> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        
        if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
            state.tokens.advance(); // consume =
            let value = super::parse_expr_recursive(state)?;
            return Ok(Some(LetBindingInfo { name, value }));
        }
    }
    Ok(None)
}

/// Check if this is a let expression (has 'in' keyword) (complexity: 2)
fn is_let_expression(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::In, _)))
}

/// Create let statement expression from binding info (complexity: 8)
fn create_let_statement_expression(state: &mut ParserState, let_info: LetBindingInfo, start_span: Span) -> Result<Expr> {
    consume_optional_semicolon(state);
    
    let body = parse_remaining_block_body(state, start_span)?;
    
    Ok(Expr::new(
        ExprKind::Let {
            name: let_info.name,
            type_annotation: None,
            value: Box::new(let_info.value),
            body: Box::new(body),
            is_mutable: false,
        },
        start_span,
    ))
}

/// Parse remaining expressions as block body (complexity: 8)
fn parse_remaining_block_body(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    let mut body_exprs = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        body_exprs.push(super::parse_expr_recursive(state)?);
        
        consume_optional_semicolon(state);
        
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }
    }
    
    Ok(create_body_expression(body_exprs, start_span))
}

/// Create body expression from parsed expressions (complexity: 4)
fn create_body_expression(body_exprs: Vec<Expr>, start_span: Span) -> Expr {
    if body_exprs.is_empty() {
        Expr::new(ExprKind::Literal(Literal::Unit), start_span)
    } else if body_exprs.len() == 1 {
        body_exprs.into_iter().next().expect("checked: len == 1")
    } else {
        Expr::new(ExprKind::Block(body_exprs), start_span)
    }
}

/// Create final block result (complexity: 3)
fn create_block_result(exprs: Vec<Expr>, start_span: Span) -> Expr {
    if exprs.is_empty() {
        Expr::new(ExprKind::Literal(Literal::Unit), start_span)
    } else {
        Expr::new(ExprKind::Block(exprs), start_span)
    }
}

/// Consume optional semicolon (complexity: 2)
fn consume_optional_semicolon(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
        state.tokens.advance();
    }
}

/// Information about a let binding (complexity: 1)
#[derive(Debug, Clone)]
struct LetBindingInfo {
    name: String,
    value: Expr,
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
            // Allow reserved words as keys in object literals
            let key = if let Some((token, _)) = state.tokens.peek() {
                match token {
                    Token::Identifier(name) => {
                        let key = name.clone();
                        state.tokens.advance();
                        key
                    }
                    Token::String(s) | Token::RawString(s) => {
                        let key = s.clone();
                        state.tokens.advance();
                        key
                    }
                    // Allow reserved words as object keys
                    Token::Command => {
                        state.tokens.advance();
                        "command".to_string()
                    }
                    Token::Type => {
                        state.tokens.advance();
                        "type".to_string()
                    }
                    Token::Module => {
                        state.tokens.advance();
                        "module".to_string()
                    }
                    Token::Import => {
                        state.tokens.advance();
                        "import".to_string()
                    }
                    Token::Export => {
                        state.tokens.advance();
                        "export".to_string()
                    }
                    Token::Fun => {
                        state.tokens.advance();
                        "fun".to_string()
                    }
                    Token::Fn => {
                        state.tokens.advance();
                        "fn".to_string()
                    }
                    Token::Return => {
                        state.tokens.advance();
                        "return".to_string()
                    }
                    Token::If => {
                        state.tokens.advance();
                        "if".to_string()
                    }
                    Token::Else => {
                        state.tokens.advance();
                        "else".to_string()
                    }
                    Token::For => {
                        state.tokens.advance();
                        "for".to_string()
                    }
                    Token::While => {
                        state.tokens.advance();
                        "while".to_string()
                    }
                    Token::Loop => {
                        state.tokens.advance();
                        "loop".to_string()
                    }
                    Token::Match => {
                        state.tokens.advance();
                        "match".to_string()
                    }
                    Token::Let => {
                        state.tokens.advance();
                        "let".to_string()
                    }
                    Token::Var => {
                        state.tokens.advance();
                        "var".to_string()
                    }
                    Token::Const => {
                        state.tokens.advance();
                        "const".to_string()
                    }
                    Token::Static => {
                        state.tokens.advance();
                        "static".to_string()
                    }
                    Token::Pub => {
                        state.tokens.advance();
                        "pub".to_string()
                    }
                    Token::Mut => {
                        state.tokens.advance();
                        "mut".to_string()
                    }
                    Token::Struct => {
                        state.tokens.advance();
                        "struct".to_string()
                    }
                    Token::Enum => {
                        state.tokens.advance();
                        "enum".to_string()
                    }
                    Token::Impl => {
                        state.tokens.advance();
                        "impl".to_string()
                    }
                    Token::Trait => {
                        state.tokens.advance();
                        "trait".to_string()
                    }
                    Token::Use => {
                        state.tokens.advance();
                        "use".to_string()
                    }
                    Token::As => {
                        state.tokens.advance();
                        "as".to_string()
                    }
                    Token::In => {
                        state.tokens.advance();
                        "in".to_string()
                    }
                    Token::Where => {
                        state.tokens.advance();
                        "where".to_string()
                    }
                    Token::Async => {
                        state.tokens.advance();
                        "async".to_string()
                    }
                    Token::Await => {
                        state.tokens.advance();
                        "await".to_string()
                    }
                    Token::Try => {
                        state.tokens.advance();
                        "try".to_string()
                    }
                    Token::Catch => {
                        state.tokens.advance();
                        "catch".to_string()
                    }
                    Token::Throw => {
                        state.tokens.advance();
                        "throw".to_string()
                    }
                    Token::Break => {
                        state.tokens.advance();
                        "break".to_string()
                    }
                    Token::Continue => {
                        state.tokens.advance();
                        "continue".to_string()
                    }
                    Token::State => {
                        state.tokens.advance();
                        "state".to_string()
                    }
                    _ => bail!("Expected identifier or string key in object literal"),
                }
            } else {
                bail!("Expected key in object literal")
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

    // Parse the first element (checking for spread syntax)
    let first_element = parse_list_element(state)?;

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

        elements.push(parse_list_element(state)?);
    }

    state.tokens.expect(&Token::RightBracket)?;

    Ok(Expr::new(ExprKind::List(elements), start_span))
}

/// Parse a single list element, handling both regular expressions and spread syntax
fn parse_list_element(state: &mut ParserState) -> Result<Expr> {
    // Check for spread syntax (...)
    if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
        let start_pos = state.tokens.advance().expect("checked above").1.start; // consume ...
        let expr = super::parse_expr_recursive(state)?;
        let span = Span { 
            start: start_pos, 
            end: expr.span.end 
        };
        Ok(Expr::new(ExprKind::Spread { expr: Box::new(expr) }, span))
    } else {
        // Regular element
        super::parse_expr_recursive(state)
    }
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
/// Parse `DataFrame` header: df![ (complexity: 3)
fn parse_dataframe_header(state: &mut ParserState) -> Result<Span> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume df
    state.tokens.expect(&Token::Bang)?;
    state.tokens.expect(&Token::LeftBracket)?;
    Ok(start_span)
}

/// Parse column name identifier (complexity: 3)
fn parse_dataframe_column_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected column name in DataFrame literal");
    }
}

/// Parse column values after => (complexity: 4)
fn parse_dataframe_column_values(state: &mut ParserState) -> Result<Vec<Expr>> {
    state.tokens.expect(&Token::FatArrow)?; // consume =>
    
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
    
    Ok(value_vec)
}

/// Handle legacy syntax column (complexity: 3)
fn handle_dataframe_legacy_syntax_column(col_name: String) -> DataFrameColumn {
    // Legacy syntax: just column names, then semicolon and rows
    // For backward compatibility, create empty column for now
    DataFrameColumn {
        name: col_name,
        values: Vec::new(),
    }
}

/// Parse column definitions loop (complexity: 8)
fn parse_dataframe_column_definitions(state: &mut ParserState) -> Result<Vec<DataFrameColumn>> {
    let mut columns = Vec::new();
    
    loop {
        let col_name = parse_dataframe_column_name(state)?;
        
        // Check for => or legacy syntax
        if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
            // New syntax: col => [values]
            let values = parse_dataframe_column_values(state)?;
            columns.push(DataFrameColumn {
                name: col_name,
                values,
            });
        } else if matches!(state.tokens.peek(), Some((Token::Comma, _)))
            || matches!(state.tokens.peek(), Some((Token::Semicolon, _)))
            || matches!(state.tokens.peek(), Some((Token::RightBracket, _)))
        {
            columns.push(handle_dataframe_legacy_syntax_column(col_name));
        } else {
            bail!("Expected '=>' or ',' after column name in DataFrame literal");
        }
        
        // Check for continuation
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            // Legacy row-based syntax
            state.tokens.advance();
            parse_legacy_dataframe_rows(state, &mut columns)?;
            break;
        } else {
            break;
        }
    }
    
    Ok(columns)
}

/// Create final `DataFrame` expression (complexity: 3)
fn create_dataframe_result(columns: Vec<DataFrameColumn>, start_span: Span) -> Result<Expr> {
    Ok(Expr::new(ExprKind::DataFrame { columns }, start_span))
}

/// Parse `DataFrame` literal: df![...] (complexity: 6)
pub fn parse_dataframe(state: &mut ParserState) -> Result<Expr> {
    let start_span = parse_dataframe_header(state)?;
    
    // Check for empty DataFrame df![]
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance();
        return create_dataframe_result(Vec::new(), start_span);
    }
    
    // Parse column definitions
    let columns = parse_dataframe_column_definitions(state)?;
    
    state.tokens.expect(&Token::RightBracket)?;
    create_dataframe_result(columns, start_span)
}

/// Parse legacy row-based `DataFrame` syntax for backward compatibility
#[allow(clippy::ptr_arg)] // We need to mutate the Vec, not just read it
fn parse_legacy_dataframe_rows(
    state: &mut ParserState,
    columns: &mut Vec<DataFrameColumn>,
) -> Result<()> {
    let rows = parse_all_dataframe_rows(state)?;
    populate_dataframe_columns(columns, &rows);
    Ok(())
}

/// Parse all dataframe rows (complexity: 2)
fn parse_all_dataframe_rows(state: &mut ParserState) -> Result<Vec<Vec<Expr>>> {
    let mut rows = Vec::new();
    
    loop {
        if is_end_bracket(state) {
            break;
        }
        
        let row = parse_single_dataframe_row(state)?;
        add_non_empty_row(&mut rows, row);
        
        if !consume_row_separator(state) {
            break;
        }
    }
    
    Ok(rows)
}

/// Check if current token is end bracket (complexity: 1)
fn is_end_bracket(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::RightBracket, _)))
}

/// Parse a single dataframe row (complexity: 2)
fn parse_single_dataframe_row(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut row = Vec::new();
    
    loop {
        if is_row_boundary(state) {
            break;
        }
        
        row.push(super::parse_expr_recursive(state)?);
        
        if !consume_value_separator(state) {
            break;
        }
    }
    
    Ok(row)
}

/// Check if current token is a row boundary (complexity: 2)
fn is_row_boundary(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::Semicolon, _)))
        || matches!(state.tokens.peek(), Some((Token::RightBracket, _)))
}

/// Consume comma separator if present (complexity: 2)
fn consume_value_separator(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Consume semicolon row separator if present (complexity: 2)
fn consume_row_separator(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Add non-empty row to collection (complexity: 2)
fn add_non_empty_row(rows: &mut Vec<Vec<Expr>>, row: Vec<Expr>) {
    if !row.is_empty() {
        rows.push(row);
    }
}

/// Populate columns from row data (complexity: 3)
fn populate_dataframe_columns(columns: &mut [DataFrameColumn], rows: &[Vec<Expr>]) {
    for (col_idx, column) in columns.iter_mut().enumerate() {
        for row in rows {
            if col_idx < row.len() {
                column.values.push(row[col_idx].clone());
            }
        }
    }
}
