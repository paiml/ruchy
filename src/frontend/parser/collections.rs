//! Collections parsing (lists, dataframes, comprehensions, blocks, object literals)
use super::{bail, expressions, functions, Expr, ExprKind, ParserState, Result, Span, Token};
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

    // Check if this might be a comprehension (set or dict)
    if let Ok(comprehension) = try_parse_comprehension(state, start_span) {
        return Ok(comprehension);
    }

    // Check if this might be an object literal
    if is_object_literal(state) {
        return parse_object_literal_body(state, start_span);
    }

    // Try to parse as block first (priority for function bodies)
    if let Ok(block_result) = try_parse_block_expressions(state, start_span) {
        return Ok(block_result);
    }

    // Check if this might be a set literal (fallback for explicit sets)
    if let Ok(set_literal) = try_parse_set_literal(state, start_span) {
        return Ok(set_literal);
    }

    // Final fallback - parse as empty block
    // PARSER-063: Skip comments before expecting closing brace
    skip_comments(state);
    state.tokens.expect(&Token::RightBrace)?;
    Ok(create_block_result(Vec::new(), start_span))
}

/// Try to parse as block expressions with backtracking (complexity: 5)
fn try_parse_block_expressions(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    // Save position for backtracking
    let saved_position = state.tokens.position();

    if let Ok(exprs) = parse_block_expressions(state, start_span) {
        // PARSER-063: Skip comments before checking for closing brace
        // This is critical - comments before } would cause backtracking otherwise
        skip_comments(state);

        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            state.tokens.advance(); // consume }
            Ok(create_block_result(exprs, start_span))
        } else {
            // Failed to find closing brace, backtrack
            state.tokens.set_position(saved_position);
            bail!("Not a valid block - missing closing brace")
        }
    } else {
        // Failed to parse as block, backtrack
        state.tokens.set_position(saved_position);
        bail!("Not a valid block expression")
    }
}

/// Skip any comment tokens in the stream (PARSER-063)
///
/// Comments should be transparent to parsing logic - they don't affect syntax.
fn skip_comments(state: &mut ParserState) {
    while matches!(
        state.tokens.peek(),
        Some((
            Token::LineComment(_)
                | Token::BlockComment(_)
                | Token::DocComment(_)
                | Token::HashComment(_),
            _
        ))
    ) {
        state.tokens.advance();
    }
}

/// Parse all expressions within a block (complexity: 8)
/// Made public for use by async block parsing (PARSER-056)
pub(in crate::frontend::parser) fn parse_block_expressions(
    state: &mut ParserState,
    start_span: Span,
) -> Result<Vec<Expr>> {
    let mut exprs = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // PARSER-063: Skip comments before parsing each expression in the block
        skip_comments(state);

        // Check again after skipping comments
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }

        let expr = parse_next_block_expression(state, start_span)?;
        exprs.push(expr);
        consume_optional_semicolon(state);
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }
    }
    Ok(exprs)
}
/// Parse the next expression in a block, handling attributes and let statements (complexity: 10)
/// DEFECT-PARSER-006: Now parses attributes before expressions in block bodies
fn parse_next_block_expression(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    // Parse attributes before the expression (same pattern as top-level parsing in core.rs:55)
    let _attributes = super::utils::parse_attributes(state)?;

    let mut expr = if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        parse_potential_let_statement(state, start_span)?
    } else {
        super::parse_expr_recursive(state)?
    };

    // Attach attributes to specific expression types that support them
    match &mut expr.kind {
        ExprKind::Function { .. } | ExprKind::Struct { .. } | ExprKind::Class { .. } => {
            // Attributes are already attached by the underlying parser
            // No action needed - attributes are stored in the AST
        }
        _ => {
            // For other expression types, attributes are parsed but may be ignored
            // This prevents "Unexpected token: AttributeStart" errors
        }
    }

    Ok(expr)
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
fn create_let_statement_expression(
    state: &mut ParserState,
    let_info: LetBindingInfo,
    start_span: Span,
) -> Result<Expr> {
    consume_optional_semicolon(state);
    let body = parse_remaining_block_body(state, start_span)?;
    Ok(Expr::new(
        ExprKind::Let {
            name: let_info.name,
            type_annotation: None,
            value: Box::new(let_info.value),
            body: Box::new(body),
            is_mutable: false,
            else_block: None, // Block-level let doesn't support let-else
        },
        start_span,
    ))
}
/// Parse remaining expressions as block body (complexity: 8)
/// PARSER-081 FIX: Must use `parse_next_block_expression` to handle sequential let statements
fn parse_remaining_block_body(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    let mut body_exprs = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Use parse_next_block_expression to properly handle let statements
        body_exprs.push(parse_next_block_expression(state, start_span)?);
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
/// Consume optional semicolon and skip trailing comments (complexity: 3)
/// PARSER-054: Must skip comments after semicolons to avoid parse errors
fn consume_optional_semicolon(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
        state.tokens.advance();
        // Skip any trailing comments after the semicolon
        while matches!(
            state.tokens.peek(),
            Some((
                Token::LineComment(_)
                    | Token::BlockComment(_)
                    | Token::DocComment(_)
                    | Token::HashComment(_),
                _
            ))
        ) {
            state.tokens.advance();
        }
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
    // Empty braces {} are object literals, not empty blocks
    if is_empty_braces(state) {
        return true;
    }
    if is_spread_operator(state) {
        return true;
    }
    check_for_object_key_separator(state)
}

fn is_empty_braces(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::RightBrace, _)))
}

fn is_spread_operator(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::DotDotDot, _)))
}

fn check_for_object_key_separator(state: &mut ParserState) -> bool {
    // PARSER-DEFECT-018: Check if token can be an object key (including keywords)
    if let Some((token, _)) = state.tokens.peek() {
        if can_be_object_key(token) {
            let saved_pos = state.tokens.position();
            state.tokens.advance(); // skip key token

            let has_separator = matches!(
                state.tokens.peek(),
                Some((Token::Colon | Token::FatArrow, _))
            );

            let is_dict_comprehension = has_separator
                && matches!(state.tokens.peek(), Some((Token::Colon, _)))
                && lookahead_for_comprehension(state);

            state.tokens.set_position(saved_pos);
            return has_separator && !is_dict_comprehension;
        }
    }
    false
}

/// Check if a token can be used as an object key (PARSER-DEFECT-018)
/// This includes identifiers, strings, and keywords
fn can_be_object_key(token: &Token) -> bool {
    matches!(
        token,
        // PARSER-082: Allow atoms as object keys
        Token::Identifier(_) | Token::String(_) | Token::RawString(_) | Token::Atom(_)
    ) || control_flow_token_to_key(token).is_some()
        || declaration_token_to_key(token).is_some()
        || type_token_to_key(token).is_some()
        || module_token_to_key(token).is_some()
        || async_error_token_to_key(token).is_some()
}

fn lookahead_for_comprehension(state: &mut ParserState) -> bool {
    let saved_pos = state.tokens.position();
    state.tokens.advance(); // consume the colon

    let found_for = scan_for_comprehension_keyword(state);

    state.tokens.set_position(saved_pos);
    found_for
}

fn scan_for_comprehension_keyword(state: &mut ParserState) -> bool {
    let mut token_count = 0;

    while token_count < 20 {
        match state.tokens.peek() {
            Some((Token::For, _)) => return true,
            Some((Token::RightBrace, _)) => return false,
            Some((Token::Comma, _)) => return false,
            Some(_) => {
                state.tokens.advance();
                token_count += 1;
            }
            None => return false,
        }
    }
    false
}
/// Parse an object key, handling identifiers, strings, and reserved words
/// Complexity: 8 (extracted from `parse_object_literal_body`)
fn parse_object_key(state: &mut ParserState) -> Result<String> {
    if let Some((token, _)) = state.tokens.peek() {
        let key = token_to_object_key(token)?;
        state.tokens.advance();
        Ok(key)
    } else {
        bail!("Expected key in object literal")
    }
}
/// Convert a token to an object key string
/// Complexity: 7 (simple match statement)
/// Extract method: Handle control flow keywords as object keys - complexity: 4
fn control_flow_token_to_key(token: &Token) -> Option<String> {
    match token {
        Token::If => Some("if".to_string()),
        Token::Else => Some("else".to_string()),
        Token::For => Some("for".to_string()),
        Token::While => Some("while".to_string()),
        Token::Loop => Some("loop".to_string()),
        Token::Match => Some("match".to_string()),
        Token::Break => Some("break".to_string()),
        Token::Continue => Some("continue".to_string()),
        Token::Return => Some("return".to_string()),
        _ => None,
    }
}

/// Extract method: Handle declaration keywords as object keys - complexity: 6
fn declaration_token_to_key(token: &Token) -> Option<String> {
    match token {
        Token::Let => Some("let".to_string()),
        Token::Var => Some("var".to_string()),
        Token::Const => Some("const".to_string()),
        Token::Static => Some("static".to_string()),
        Token::Pub => Some("pub".to_string()),
        Token::Mut => Some("mut".to_string()),
        Token::Fun => Some("fun".to_string()),
        Token::Fn => Some("fn".to_string()),
        _ => None,
    }
}

/// Extract method: Handle type-related keywords as object keys - complexity: 3
fn type_token_to_key(token: &Token) -> Option<String> {
    match token {
        Token::Type => Some("type".to_string()),
        Token::Struct => Some("struct".to_string()),
        Token::Enum => Some("enum".to_string()),
        Token::Impl => Some("impl".to_string()),
        Token::Trait => Some("trait".to_string()),
        _ => None,
    }
}

/// Extract method: Handle module-related keywords as object keys - complexity: 4
fn module_token_to_key(token: &Token) -> Option<String> {
    match token {
        Token::Module => Some("module".to_string()),
        Token::Import => Some("import".to_string()),
        Token::Export => Some("export".to_string()),
        Token::Use => Some("use".to_string()),
        Token::As => Some("as".to_string()),
        Token::From => Some("from".to_string()),
        Token::Self_ => Some("self".to_string()),
        Token::Super => Some("super".to_string()),
        Token::Crate => Some("crate".to_string()),
        Token::In => Some("in".to_string()),
        Token::Where => Some("where".to_string()),
        _ => None,
    }
}

/// Extract method: Handle async/error handling keywords as object keys - complexity: 3
fn async_error_token_to_key(token: &Token) -> Option<String> {
    match token {
        Token::Async => Some("async".to_string()),
        Token::Await => Some("await".to_string()),
        Token::Try => Some("try".to_string()),
        Token::Catch => Some("catch".to_string()),
        Token::Throw => Some("throw".to_string()),
        _ => None,
    }
}

/// Refactored `token_to_object_key` using Extract Method pattern
/// Complexity reduced from 50 to 8 by extracting helper functions
fn try_keyword_as_object_key(token: &Token) -> Option<String> {
    type KeywordMapper = fn(&Token) -> Option<String>;
    let mappers: &[KeywordMapper] = &[
        control_flow_token_to_key,
        declaration_token_to_key,
        type_token_to_key,
        module_token_to_key,
        async_error_token_to_key,
    ];
    for mapper in mappers {
        if let Some(key) = mapper(token) {
            return Some(key);
        }
    }
    None
}

fn token_to_object_key(token: &Token) -> Result<String> {
    match token {
        Token::Identifier(name) => Ok(name.clone()),
        Token::String(s) | Token::RawString(s) => Ok(s.clone()),
        // PARSER-082: Allow atoms as object keys (for IaC-style configuration)
        Token::Atom(s) => Ok(format!(":{s}")),
        // Allow reserved words as object keys - delegated to helper functions
        _ => try_keyword_as_object_key(token)
            .ok_or_else(|| anyhow::anyhow!("Expected identifier or string key in object literal")),
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
        parse_single_object_field(state, &mut fields)?;
        handle_object_field_separator(state)?;
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::ObjectLiteral { fields }, start_span))
}
/// Parse a single object field (either spread or key-value) - complexity: 6
fn parse_single_object_field(state: &mut ParserState, fields: &mut Vec<ObjectField>) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
        parse_object_spread_field(state, fields)
    } else {
        parse_object_key_value_field(state, fields)
    }
}
/// Parse object spread field (...expr) - complexity: 3
fn parse_object_spread_field(state: &mut ParserState, fields: &mut Vec<ObjectField>) -> Result<()> {
    state.tokens.advance(); // consume ...
    let expr = super::parse_expr_recursive(state)?;
    fields.push(ObjectField::Spread { expr });
    Ok(())
}
/// Parse object key-value field (key: value or key => value) - complexity: 5
fn parse_object_key_value_field(
    state: &mut ParserState,
    fields: &mut Vec<ObjectField>,
) -> Result<()> {
    let key = parse_object_key(state)?;
    // Accept either : or => for object key-value pairs (book compatibility)
    if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
        state.tokens.advance(); // consume =>
    } else {
        state.tokens.expect(&Token::Colon)?;
    }
    let value = super::parse_expr_recursive(state)?;
    fields.push(ObjectField::KeyValue { key, value });
    Ok(())
}
/// Handle comma separator between object fields - complexity: 4
fn handle_object_field_separator(state: &mut ParserState) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        Ok(())
    } else if !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        bail!("Expected comma or closing brace in object literal")
    } else {
        Ok(())
    }
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
            end: expr.span.end,
        };
        Ok(Expr::new(
            ExprKind::Spread {
                expr: Box::new(expr),
            },
            span,
        ))
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
/// Parse comprehension expression, handling ranges but stopping at comprehension keywords
fn parse_comprehension_expr(state: &mut ParserState) -> Result<Expr> {
    // Parse left side
    let mut left = parse_condition_term(state)?;

    // Handle binary operators including ranges, but stop at comprehension keywords
    while let Some((token, _)) = state.tokens.peek() {
        match token {
            // Stop at comprehension keywords
            Token::For | Token::If | Token::RightBracket => break,

            // Handle range operators specially
            Token::DotDot | Token::DotDotEqual => {
                let is_inclusive = matches!(token, Token::DotDotEqual);
                state.tokens.advance(); // consume .. or ..=
                let end = parse_condition_term(state)?;
                left = Expr::new(
                    ExprKind::Range {
                        start: Box::new(left),
                        end: Box::new(end),
                        inclusive: is_inclusive,
                    },
                    Span::default(),
                );
            }

            // Handle other binary operators
            Token::Greater
            | Token::Less
            | Token::GreaterEqual
            | Token::LessEqual
            | Token::EqualEqual
            | Token::NotEqual
            | Token::AndAnd
            | Token::OrOr
            | Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent => {
                let op = expressions::token_to_binary_op(token).expect("checked: valid op");
                state.tokens.advance(); // consume operator
                let right = parse_condition_term(state)?;
                left = Expr::new(
                    ExprKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    Span::default(),
                );
            }

            _ => break, // Stop at unknown tokens
        }
    }

    Ok(left)
}

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
            | Token::OrOr
            | Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent => {
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
        expr = match token {
            Token::Dot => parse_dot_operation(state, expr)?,
            Token::LeftParen => functions::parse_call(state, expr)?,
            _ => break, // Stop at other tokens
        };
    }
    Ok(expr)
}
/// Parse dot operation (field access or method call) - complexity: 8
fn parse_dot_operation(state: &mut ParserState, expr: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume .
    let Some((Token::Identifier(name), _)) = state.tokens.peek() else {
        return Ok(expr); // No identifier after dot
    };
    let name = name.clone();
    state.tokens.advance();
    // Check if it's a method call or field access
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_method_call(state, expr, name)
    } else {
        Ok(create_field_access(expr, name))
    }
}
/// Parse method call arguments (complexity: 5)
fn parse_method_call(state: &mut ParserState, receiver: Expr, method: String) -> Result<Expr> {
    state.tokens.advance(); // consume (
    let args = parse_method_arguments(state)?;
    state.tokens.expect(&Token::RightParen)?;
    Ok(Expr::new(
        ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method,
            args,
        },
        Span { start: 0, end: 0 },
    ))
}
/// Parse method call arguments (complexity: 4)
fn parse_method_arguments(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut args = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        args.push(super::parse_expr_recursive(state)?);
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }
    Ok(args)
}
/// Create field access expression (complexity: 1)
fn create_field_access(object: Expr, field: String) -> Expr {
    Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(object),
            field,
        },
        Span { start: 0, end: 0 },
    )
}
pub fn parse_list_comprehension(
    state: &mut ParserState,
    start_span: Span,
    element: Expr,
) -> Result<Expr> {
    let mut clauses = Vec::new();

    // Parse first for clause (required)
    state.tokens.expect(&Token::For)?;
    clauses.push(parse_for_clause(state)?);

    // Parse additional for clauses
    while matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance();
        clauses.push(parse_for_clause(state)?);
    }

    state.tokens.expect(&Token::RightBracket)?;

    Ok(Expr::new(
        ExprKind::ListComprehension {
            element: Box::new(element),
            clauses,
        },
        start_span,
    ))
}

fn parse_for_clause(state: &mut ParserState) -> Result<crate::frontend::ast::ComprehensionClause> {
    let variable = parse_comprehension_variable(state)?;
    state.tokens.expect(&Token::In)?;
    let iterable = parse_comprehension_iterable(state)?;

    let condition = if matches!(state.tokens.peek(), Some((Token::If, _))) {
        state.tokens.advance();
        Some(Box::new(parse_condition_expr(state)?))
    } else {
        None
    };

    Ok(crate::frontend::ast::ComprehensionClause {
        variable,
        iterable: Box::new(iterable),
        condition,
    })
}

/// Parse comprehension iterable, stopping at 'for', 'if', or ']'
fn parse_comprehension_iterable(state: &mut ParserState) -> Result<Expr> {
    // Parse an expression but stop at keywords that end the iterable
    // We need to handle ranges (0..100) but stop at 'if', 'for', or ']'
    let mut expr = parse_comprehension_expr(state)?;

    // Check for method calls or field access
    while let Some((token, _)) = state.tokens.peek() {
        match token {
            Token::For | Token::If | Token::RightBracket => break,
            Token::Dot => {
                // Allow method chaining
                // Just consume the dot and parse the next part manually
                state.tokens.advance(); // consume dot
                if let Some((Token::Identifier(method), _)) = state.tokens.peek() {
                    let method_name = method.clone();
                    state.tokens.advance();
                    // For now, just treat it as a field access
                    expr = Expr::new(
                        ExprKind::FieldAccess {
                            object: Box::new(expr),
                            field: method_name,
                        },
                        Span::default(),
                    );
                }
            }
            _ => break,
        }
    }

    Ok(expr)
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
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::String(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(name)
        }
        _ => bail!("Expected column name (identifier or string) in DataFrame literal"),
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
/// Parse column definitions loop (complexity: 6)
pub fn parse_dataframe_column_definitions(state: &mut ParserState) -> Result<Vec<DataFrameColumn>> {
    let mut columns = Vec::new();
    loop {
        // Check for trailing comma (empty item after comma)
        if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
            break;
        }

        let col_name = parse_dataframe_column_name(state)?;
        parse_single_dataframe_column(state, col_name, &mut columns)?;
        if !handle_dataframe_column_continuation(state, &mut columns)? {
            break;
        }
    }
    Ok(columns)
}
/// Parse a single `DataFrame` column (either new or legacy syntax) - complexity: 5
fn parse_single_dataframe_column(
    state: &mut ParserState,
    col_name: String,
    columns: &mut Vec<DataFrameColumn>,
) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
        // New syntax: col => [values]
        let values = parse_dataframe_column_values(state)?;
        columns.push(DataFrameColumn {
            name: col_name,
            values,
        });
    } else if is_dataframe_legacy_syntax_token(state) {
        columns.push(handle_dataframe_legacy_syntax_column(col_name));
    } else {
        bail!("Expected '=>' or ',' after column name in DataFrame literal");
    }
    Ok(())
}
/// Check if current token indicates legacy `DataFrame` syntax - complexity: 4
fn is_dataframe_legacy_syntax_token(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::Comma, _)))
        || matches!(state.tokens.peek(), Some((Token::Semicolon, _)))
        || matches!(state.tokens.peek(), Some((Token::RightBracket, _)))
}
/// Handle `DataFrame` column continuation tokens - complexity: 5
fn handle_dataframe_column_continuation(
    state: &mut ParserState,
    columns: &mut Vec<DataFrameColumn>,
) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        Ok(true)
    } else if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
        // Legacy row-based syntax
        state.tokens.advance();
        parse_legacy_dataframe_rows(state, columns)?;
        Ok(false)
    } else {
        Ok(false)
    }
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

/// Parse an expression but stop at 'for' keyword (for comprehensions)
fn parse_comprehension_element(state: &mut ParserState) -> Result<Expr> {
    // Parse the expression but stop at 'for' keyword
    // We can't use parse_expr_recursive directly because it will consume 'for'
    // as part of a for loop

    // Start with a prefix expression
    let mut expr = super::expressions::parse_prefix(state)?;

    // Continue parsing operators but stop at 'for'
    loop {
        // Check for 'for' keyword - stop here for comprehensions
        if matches!(state.tokens.peek(), Some((Token::For, _))) {
            break;
        }

        // Try to parse postfix operators
        let prev_expr = expr.clone();
        expr = super::handle_postfix_operators(state, expr)?;
        if expr != prev_expr {
            continue; // Made progress with postfix
        }

        // Try to parse infix operators with precedence 0
        if let Some(new_expr) = super::try_handle_infix_operators(state, expr.clone(), 0)? {
            expr = new_expr;
            continue;
        }

        // No more operators to parse
        break;
    }

    Ok(expr)
}

/// Try to parse set or dict comprehension from { ... } syntax
/// Returns Ok(comprehension) if successful, Err if not a comprehension
fn try_parse_comprehension(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    // Quick lookahead check to see if this might be a comprehension
    // Look for patterns like: identifier for, identifier: identifier for
    if !looks_like_comprehension(state) {
        bail!("Not a comprehension - doesn't match comprehension pattern");
    }

    // Save parser state for backtracking
    let saved_position = state.tokens.position();

    // Parse the first expression more carefully
    // We need to parse a full expression that could include operators,
    // but we need to stop at 'for' keyword
    let first_expr = match parse_comprehension_element(state) {
        Ok(expr) => expr,
        Err(e) => {
            state.tokens.set_position(saved_position);
            bail!("Not a comprehension - failed to parse first expression: {e}");
        }
    };

    // Check what comes next to determine comprehension type
    match state.tokens.peek() {
        Some((Token::For, _)) => {
            // This is a set comprehension: {expr for x in iter}
            parse_set_comprehension_continuation(state, first_expr, start_span)
        }
        Some((Token::Colon, _)) => {
            // This might be a dict comprehension: {key: value for x in iter}
            state.tokens.advance(); // consume :
            let value_expr = match parse_comprehension_element(state) {
                Ok(expr) => expr,
                Err(e) => {
                    state.tokens.set_position(saved_position);
                    bail!("Not a dict comprehension - failed to parse value: {e}");
                }
            };

            // Check for 'for' keyword
            if matches!(state.tokens.peek(), Some((Token::For, _))) {
                parse_dict_comprehension_continuation(state, first_expr, value_expr, start_span)
            } else {
                // Not a comprehension, restore state
                state.tokens.set_position(saved_position);
                bail!("Not a dict comprehension - no 'for' keyword");
            }
        }
        _ => {
            // Not a comprehension, restore state
            state.tokens.set_position(saved_position);
            bail!("Not a comprehension - no 'for' or ':' after first expression");
        }
    }
}

/// Quick lookahead to determine if this might be a comprehension
/// Looks for patterns: x for, x: y for, etc.
/// Classify a token during comprehension lookahead scanning
enum ComprehensionLookahead {
    FoundFor,
    NotComprehension,
    OpenBracket,
    CloseBracket { at_top_level: bool },
    Continue,
    End,
}

fn classify_comprehension_token(
    token: Option<&(Token, Span)>,
    nesting_depth: usize,
    token_count: usize,
) -> ComprehensionLookahead {
    match token {
        Some((Token::For, _)) if nesting_depth == 0 => {
            if token_count == 0 {
                ComprehensionLookahead::NotComprehension
            } else {
                ComprehensionLookahead::FoundFor
            }
        }
        Some((Token::LeftBrace | Token::LeftParen | Token::LeftBracket, _)) => {
            ComprehensionLookahead::OpenBracket
        }
        Some((Token::RightBrace | Token::RightParen | Token::RightBracket, _)) => {
            ComprehensionLookahead::CloseBracket {
                at_top_level: nesting_depth == 0,
            }
        }
        Some((Token::Semicolon | Token::Let | Token::Var, _)) if nesting_depth == 0 => {
            ComprehensionLookahead::NotComprehension
        }
        Some(_) => ComprehensionLookahead::Continue,
        None => ComprehensionLookahead::End,
    }
}

fn looks_like_comprehension(state: &mut ParserState) -> bool {
    let saved_pos = state.tokens.position();
    let mut token_count = 0;
    let mut found_for = false;
    let mut nesting_depth: usize = 0;

    while token_count < 20 && !found_for {
        match classify_comprehension_token(state.tokens.peek(), nesting_depth, token_count) {
            ComprehensionLookahead::FoundFor => {
                found_for = true;
                break;
            }
            ComprehensionLookahead::NotComprehension | ComprehensionLookahead::End => break,
            ComprehensionLookahead::OpenBracket => {
                nesting_depth += 1;
                state.tokens.advance();
                token_count += 1;
            }
            ComprehensionLookahead::CloseBracket { at_top_level } => {
                if at_top_level {
                    break;
                }
                nesting_depth -= 1;
                state.tokens.advance();
                token_count += 1;
            }
            ComprehensionLookahead::Continue => {
                state.tokens.advance();
                token_count += 1;
            }
        }
    }

    state.tokens.set_position(saved_pos);
    found_for
}

/// Parse the continuation of a set comprehension after detecting {expr for
fn parse_set_comprehension_continuation(
    state: &mut ParserState,
    element: Expr,
    start_span: Span,
) -> Result<Expr> {
    let mut clauses = Vec::new();

    // Parse first for clause (required)
    state.tokens.expect(&Token::For)?;
    clauses.push(parse_for_clause(state)?);

    // Parse additional for clauses
    while matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance();
        clauses.push(parse_for_clause(state)?);
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        crate::frontend::ast::ExprKind::SetComprehension {
            element: Box::new(element),
            clauses,
        },
        start_span,
    ))
}

/// Parse the continuation of a dict comprehension after detecting {key: value for
fn parse_dict_comprehension_continuation(
    state: &mut ParserState,
    key: Expr,
    value: Expr,
    start_span: Span,
) -> Result<Expr> {
    let mut clauses = Vec::new();

    // Parse first for clause (required)
    state.tokens.expect(&Token::For)?;
    clauses.push(parse_for_clause(state)?);

    // Parse additional for clauses
    while matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance();
        clauses.push(parse_for_clause(state)?);
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        crate::frontend::ast::ExprKind::DictComprehension {
            key: Box::new(key),
            value: Box::new(value),
            clauses,
        },
        start_span,
    ))
}

/// Parse comprehension variable - supports patterns like Some(x), (a, b), or simple identifiers (complexity: 8)
pub fn parse_comprehension_variable(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::LeftParen, _)) => parse_tuple_pattern(state),
        Some((Token::Identifier(_), _)) => parse_identifier_pattern(state),
        Some((Token::Some, _)) => parse_option_some_pattern(state),
        Some((Token::None, _)) => parse_option_none_pattern(state),
        Some((Token::Ok, _)) => parse_result_ok_pattern(state),
        Some((Token::Err, _)) => parse_result_err_pattern(state),
        _ => bail!("Expected pattern in comprehension variable"),
    }
}

fn parse_tuple_pattern(state: &mut ParserState) -> Result<String> {
    state.tokens.advance(); // consume (
    let mut pattern_str = String::from("(");

    // Parse first element
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        pattern_str.push_str(name);
        state.tokens.advance();
    } else {
        bail!("Expected identifier in tuple pattern");
    }

    // Parse remaining elements
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        pattern_str.push_str(", ");

        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            pattern_str.push_str(name);
            state.tokens.advance();
        } else {
            bail!("Expected identifier after comma in tuple pattern");
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    pattern_str.push(')');
    Ok(pattern_str)
}

fn parse_identifier_pattern(state: &mut ParserState) -> Result<String> {
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        n.clone()
    } else {
        bail!("Expected identifier")
    };
    state.tokens.advance();

    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_constructor_pattern(state, &name)
    } else {
        Ok(name)
    }
}

fn parse_constructor_pattern(state: &mut ParserState, name: &str) -> Result<String> {
    state.tokens.advance(); // consume (
    let mut pattern_str = format!("{name}(");

    if let Some((Token::Identifier(inner), _)) = state.tokens.peek() {
        pattern_str.push_str(inner);
        state.tokens.advance();
    }

    state.tokens.expect(&Token::RightParen)?;
    pattern_str.push(')');
    Ok(pattern_str)
}

fn parse_option_some_pattern(state: &mut ParserState) -> Result<String> {
    state.tokens.advance(); // consume Some
    state.tokens.expect(&Token::LeftParen)?;

    let inner = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier inside Some pattern");
    };

    state.tokens.expect(&Token::RightParen)?;
    Ok(format!("Some({inner})"))
}

fn parse_option_none_pattern(state: &mut ParserState) -> Result<String> {
    state.tokens.advance();
    Ok("None".to_string())
}

fn parse_result_ok_pattern(state: &mut ParserState) -> Result<String> {
    state.tokens.advance(); // consume Ok
    state.tokens.expect(&Token::LeftParen)?;

    let inner = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier inside Ok pattern");
    };

    state.tokens.expect(&Token::RightParen)?;
    Ok(format!("Ok({inner})"))
}

fn parse_result_err_pattern(state: &mut ParserState) -> Result<String> {
    state.tokens.advance(); // consume Err
    state.tokens.expect(&Token::LeftParen)?;

    let inner = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier inside Err pattern");
    };

    state.tokens.expect(&Token::RightParen)?;
    Ok(format!("Err({inner})"))
}

/// Try to parse a set literal: {expr, expr, ...}
/// Result of checking whether a token starts a statement (not a set element)
fn is_statement_start_token(token: Option<&(Token, Span)>) -> bool {
    matches!(
        token,
        Some((
            Token::Let | Token::If | Token::For | Token::While | Token::Return,
            _
        ))
    )
}

/// After parsing one set element expression, classify what follows
enum SetElementSuffix {
    Comma,
    TrailingComma,
    End,
    NotSet(&'static str),
}

fn classify_set_element_suffix(state: &mut ParserState) -> SetElementSuffix {
    match state.tokens.peek() {
        Some((Token::Comma, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                SetElementSuffix::TrailingComma
            } else {
                SetElementSuffix::Comma
            }
        }
        Some((Token::RightBrace, _)) => SetElementSuffix::End,
        Some((Token::Semicolon, _)) => SetElementSuffix::NotSet("contains semicolon"),
        Some((Token::For, _)) => SetElementSuffix::NotSet("looks like comprehension"),
        _ => SetElementSuffix::NotSet("unexpected token after expression"),
    }
}

fn try_parse_set_literal(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    let saved_position = state.tokens.position();

    if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        state.tokens.advance();
        return Ok(Expr::new(ExprKind::Set(Vec::new()), start_span));
    }

    let mut elements = Vec::new();

    loop {
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }

        if is_statement_start_token(state.tokens.peek()) {
            state.tokens.set_position(saved_position);
            bail!("Not a set literal - contains statements");
        }

        let expr = if let Ok(expr) = super::parse_expr_recursive(state) {
            expr
        } else {
            state.tokens.set_position(saved_position);
            bail!("Not a set literal - failed to parse expression");
        };

        match classify_set_element_suffix(state) {
            SetElementSuffix::Comma => elements.push(expr),
            SetElementSuffix::TrailingComma | SetElementSuffix::End => {
                elements.push(expr);
                break;
            }
            SetElementSuffix::NotSet(reason) => {
                state.tokens.set_position(saved_position);
                bail!("Not a set literal - {reason}");
            }
        }
    }

    if elements.is_empty() {
        state.tokens.set_position(saved_position);
        bail!("Not a set literal - no elements");
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Set(elements), start_span))
}

#[cfg(test)]
#[path = "collections_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "collections_mutation_tests.rs"]
mod mutation_tests;
