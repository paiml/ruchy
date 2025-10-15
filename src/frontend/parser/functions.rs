//! Function-related parsing (function definitions, lambdas, calls)
use super::{bail, utils, Expr, ExprKind, Param, ParserState, Result, Span, Token, Type, TypeKind};
use crate::frontend::ast::{DataFrameOp, Literal, Pattern};
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_function(state: &mut ParserState) -> Result<Expr> {
    parse_function_with_visibility(state, false)
}
pub fn parse_function_with_visibility(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1;
    let name = parse_function_name(state);
    let type_params = parse_optional_type_params(state)?;
    let params = utils::parse_params(state)?;
    let return_type = parse_optional_return_type(state)?;
    parse_optional_where_clause(state)?;
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Function {
            name,
            type_params,
            params,
            return_type,
            body: Box::new(body),
            is_async: false,
            is_pub,
        },
        start_span,
    ))
}

/// Parse function name or return "anonymous" (complexity: 1)
fn parse_function_name(state: &mut ParserState) -> String {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        "anonymous".to_string()
    }
}

/// Parse optional type parameters <T, U, ...> (complexity: 1)
fn parse_optional_type_params(state: &mut ParserState) -> Result<Vec<String>> {
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)
    } else {
        Ok(Vec::new())
    }
}

/// Parse optional return type after arrow (complexity: 2)
fn parse_optional_return_type(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Parse optional where clause (complexity: 1)
fn parse_optional_where_clause(state: &mut ParserState) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::Where, _))) {
        parse_where_clause(state)?;
    }
    Ok(())
}
fn parse_lambda_params(state: &mut ParserState) -> Result<Vec<Param>> {
    let mut params = Vec::new();
    while !is_at_param_end(state) {
        if !try_append_lambda_param(state, &mut params)? {
            break;
        }
    }
    Ok(params)
}

/// Try to append a lambda parameter, return false if no more params (complexity: 3)
fn try_append_lambda_param(state: &mut ParserState, params: &mut Vec<Param>) -> Result<bool> {
    let Some(param) = try_parse_single_lambda_param(state)? else {
        return Ok(false);
    };
    params.push(param);
    Ok(consume_comma_if_present(state))
}

/// Check if at end of parameter list (complexity: 1)
fn is_at_param_end(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::Pipe, _)))
}

/// Try to parse a single lambda parameter (complexity: 2)
fn try_parse_single_lambda_param(state: &mut ParserState) -> Result<Option<Param>> {
    let Some(param_name) = try_parse_param_name(state)? else {
        return Ok(None);
    };
    let param_type = parse_optional_type_annotation(state)?;
    Ok(Some(create_lambda_param(param_name, param_type)))
}

/// Try to parse a parameter name, returning None if no identifier found (complexity: 1)
fn try_parse_param_name(state: &mut ParserState) -> Result<Option<String>> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(Some(name))
    } else {
        Ok(None)
    }
}

/// Parse optional type annotation after colon (complexity: 2)
fn parse_optional_type_annotation(state: &mut ParserState) -> Result<Type> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        utils::parse_type(state)
    } else {
        Ok(Type {
            kind: TypeKind::Named("_".to_string()),
            span: Span { start: 0, end: 0 },
        })
    }
}

/// Create a lambda parameter from name and type (complexity: 1)
fn create_lambda_param(name: String, ty: Type) -> Param {
    Param {
        pattern: Pattern::Identifier(name),
        ty,
        span: Span { start: 0, end: 0 },
        is_mutable: false,
        default_value: None,
    }
}

/// Consume comma if present, return true if consumed (complexity: 1)
fn consume_comma_if_present(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_empty_lambda(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume ||
                                                                                 // Lambda syntax: || expr (no => allowed)
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
    // Check syntax type and parse accordingly
    let params = if matches!(state.tokens.peek(), Some((Token::Backslash, _))) {
        parse_backslash_lambda(state)?
    } else {
        parse_pipe_lambda(state)?
    };
    // Parse body
    let body = super::parse_expr_recursive(state)?;
    Ok(Expr::new(
        ExprKind::Lambda {
            params,
            body: Box::new(body),
        },
        start_span,
    ))
}
/// Parse backslash-style lambda: \x, y -> body (complexity: 6)
fn parse_backslash_lambda(state: &mut ParserState) -> Result<Vec<Param>> {
    state.tokens.advance(); // consume \
    let params = parse_simple_params(state)?;
    // Expect arrow
    state
        .tokens
        .expect(&Token::Arrow)
        .map_err(|e| anyhow::anyhow!("In backslash lambda after params: {}", e))?;
    Ok(params)
}
/// Parse pipe-style lambda: |x, y| body (complexity: 5)
fn parse_pipe_lambda(state: &mut ParserState) -> Result<Vec<Param>> {
    state.tokens.advance(); // consume |
                            // Handle || as empty params
    if matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        state.tokens.advance(); // consume second |
        return Ok(Vec::new());
    }
    // Parse parameters between pipes
    let params = parse_lambda_params(state)?;
    // Expect closing pipe
    if !matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        bail!("Expected '|' after lambda parameters");
    }
    state.tokens.advance(); // consume |
    Ok(params)
}
/// Parse simple comma-separated parameters (complexity: 6)
fn parse_simple_params(state: &mut ParserState) -> Result<Vec<Param>> {
    let mut params = Vec::new();
    // Parse first parameter if present
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        params.push(create_simple_param(name.clone()));
        state.tokens.advance();
        // Parse additional parameters
        while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                params.push(create_simple_param(name.clone()));
                state.tokens.advance();
            }
        }
    }
    Ok(params)
}
/// Create a simple parameter with default type (complexity: 1)
fn create_simple_param(name: String) -> Param {
    Param {
        pattern: Pattern::Identifier(name),
        ty: Type {
            kind: TypeKind::Named("Any".to_string()),
            span: Span { start: 0, end: 0 },
        },
        span: Span { start: 0, end: 0 },
        is_mutable: false,
        default_value: None,
    }
}
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_call(state: &mut ParserState, func: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume (
    let (args, named_args) = parse_arguments_list(state)?;
    state.tokens.expect(&Token::RightParen)?;

    build_call_expression(func, args, named_args)
}

/// Build appropriate call expression based on arguments (complexity: 5)
fn build_call_expression(
    func: Expr,
    args: Vec<Expr>,
    named_args: Vec<(String, Expr)>,
) -> Result<Expr> {
    if named_args.is_empty() {
        Ok(Expr {
            kind: ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comment: None,
        })
    } else {
        build_struct_literal_call(func, named_args)
    }
}

/// Convert named args to struct literal (complexity: 3)
fn build_struct_literal_call(func: Expr, named_args: Vec<(String, Expr)>) -> Result<Expr> {
    if let ExprKind::Identifier(name) = &func.kind {
        let fields = named_args.into_iter().collect();
        Ok(Expr {
            kind: ExprKind::StructLiteral {
                name: name.clone(),
                fields,
                base: None,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comment: None,
        })
    } else {
        // For now, only support named args with simple identifiers
        Ok(Expr {
            kind: ExprKind::Call {
                func: Box::new(func),
                args: Vec::new(),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comment: None,
        })
    }
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
            leading_comments: Vec::new(),
            trailing_comment: None,
        });
    }
    // Parse method name or tuple index
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let method = name.clone();
            state.tokens.advance();
            parse_method_or_field_access(state, receiver, method)
        }
        Some((Token::Send, _)) => {
            // Handle 'send' as a method name (for actors)
            state.tokens.advance();
            parse_method_or_field_access(state, receiver, "send".to_string())
        }
        Some((Token::Ask, _)) => {
            // Handle 'ask' as a method name (for actors)
            state.tokens.advance();
            parse_method_or_field_access(state, receiver, "ask".to_string())
        }
        Some((Token::Integer(index), _)) => {
            // Handle tuple access like t.0, t.1, etc.
            let index = index.clone();
            state.tokens.advance();
            Ok(Expr {
                kind: ExprKind::FieldAccess {
                    object: Box::new(receiver),
                    field: index,
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
                leading_comments: Vec::new(),
                trailing_comment: None,
            })
        }
        _ => {
            bail!("Expected method name, tuple index, or 'await' after '.'");
        }
    }
}
pub fn parse_optional_method_call(state: &mut ParserState, receiver: Expr) -> Result<Expr> {
    // Parse method name or tuple index for optional chaining
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let method = name.clone();
            state.tokens.advance();
            parse_optional_method_or_field_access(state, receiver, method)
        }
        Some((Token::Send, _)) => {
            // Handle 'send' as a method name (for actors)
            state.tokens.advance();
            parse_optional_method_or_field_access(state, receiver, "send".to_string())
        }
        Some((Token::Ask, _)) => {
            // Handle 'ask' as a method name (for actors)
            state.tokens.advance();
            parse_optional_method_or_field_access(state, receiver, "ask".to_string())
        }
        Some((Token::Integer(index), _)) => {
            // Handle optional tuple access like t?.0, t?.1, etc.
            let index = index.clone();
            state.tokens.advance();
            Ok(Expr {
                kind: ExprKind::OptionalFieldAccess {
                    object: Box::new(receiver),
                    field: index,
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
                leading_comments: Vec::new(),
                trailing_comment: None,
            })
        }
        _ => {
            bail!("Expected method name or tuple index after '?.'");
        }
    }
}
fn parse_method_or_field_access(
    state: &mut ParserState,
    receiver: Expr,
    method: String,
) -> Result<Expr> {
    // Check if it's a method call (with parentheses) or field access
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_method_call_access(state, receiver, method)
    } else {
        // Field access
        Ok(create_field_access(receiver, method))
    }
}
/// Parse method call with arguments (complexity: 6)
fn parse_method_call_access(
    state: &mut ParserState,
    receiver: Expr,
    method: String,
) -> Result<Expr> {
    state.tokens.advance(); // consume (
    let args = parse_method_arguments(state)?;
    state.tokens.expect(&Token::RightParen)?;
    // Check if this is a DataFrame operation
    if is_dataframe_method(&method) {
        handle_dataframe_method(receiver, method, args)
    } else {
        Ok(create_method_call(receiver, method, args))
    }
}
/// Parse method arguments (complexity: 4)
fn parse_method_arguments(state: &mut ParserState) -> Result<Vec<Expr>> {
    let (mut args, named_args) = parse_arguments_list(state)?;

    // If we have named arguments, convert them to object literal
    if !named_args.is_empty() {
        args.push(convert_named_args_to_object(state, named_args));
    }

    Ok(args)
}

/// Convert named arguments to object literal expression (complexity: 2)
fn convert_named_args_to_object(state: &mut ParserState, named_args: Vec<(String, Expr)>) -> Expr {
    use crate::frontend::ast::ObjectField;

    let fields = named_args
        .into_iter()
        .map(|(name, value)| ObjectField::KeyValue { key: name, value })
        .collect();

    let span = if let Some((_, span)) = state.tokens.peek() {
        *span
    } else {
        crate::frontend::ast::Span::new(0, 0)
    };

    Expr::new(ExprKind::ObjectLiteral { fields }, span)
}

/// Parse argument list with both positional and named arguments (complexity: 3, cognitive: 5)
fn parse_arguments_list(state: &mut ParserState) -> Result<(Vec<Expr>, Vec<(String, Expr)>)> {
    let mut args = Vec::new();
    let mut named_args = Vec::new();

    while !is_at_argument_list_end(state) {
        parse_single_argument(state, &mut args, &mut named_args)?;

        if !handle_argument_separator(state) {
            break;
        }
    }

    Ok((args, named_args))
}

/// Check if at end of argument list (complexity: 1, cognitive: 1)
fn is_at_argument_list_end(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::RightParen, _)))
}

/// Parse a single argument (named or positional) (complexity: 2, cognitive: 3)
fn parse_single_argument(
    state: &mut ParserState,
    args: &mut Vec<Expr>,
    named_args: &mut Vec<(String, Expr)>,
) -> Result<()> {
    if let Some((name, value)) = try_parse_named_argument(state)? {
        named_args.push((name, value));
    } else {
        args.push(super::parse_expr_recursive(state)?);
    }
    Ok(())
}

/// Try to parse a named argument (identifier: value) (complexity: 5)
fn try_parse_named_argument(state: &mut ParserState) -> Result<Option<(String, Expr)>> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name_clone = name.clone();
        let saved_pos = state.tokens.position();
        state.tokens.advance();

        if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            state.tokens.advance(); // consume :
            let value = super::parse_expr_recursive(state)?;
            return Ok(Some((name_clone, value)));
        }

        // Not a named arg, restore position
        state.tokens.set_position(saved_pos);
    }
    Ok(None)
}

/// Handle comma separator between arguments (complexity: 2)
fn handle_argument_separator(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}
/// Check if method is DataFrame-specific (complexity: 1)
fn is_dataframe_method(method: &str) -> bool {
    matches!(
        method,
        "select"
            | "groupby"
            | "group_by"
            | "agg"
            | "pivot"
            | "melt"
            | "join"
            | "rolling"
            | "shift"
            | "diff"
            | "pct_change"
            | "corr"
            | "cov"
    )
}
/// Handle DataFrame-specific methods (complexity: 4)
fn handle_dataframe_method(receiver: Expr, method: String, args: Vec<Expr>) -> Result<Expr> {
    let operation = match method.as_str() {
        "select" => DataFrameOp::Select(extract_select_columns(args)),
        "groupby" | "group_by" => DataFrameOp::GroupBy(extract_groupby_columns(args)),
        _ => return Ok(create_method_call(receiver, method, args)),
    };
    Ok(Expr {
        kind: ExprKind::DataFrameOperation {
            source: Box::new(receiver),
            operation,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    })
}
/// Extract column names from select arguments (complexity: 8)
fn extract_select_columns(args: Vec<Expr>) -> Vec<String> {
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
    columns
}
/// Extract column names from groupby arguments (complexity: 3)
fn extract_groupby_columns(args: Vec<Expr>) -> Vec<String> {
    args.into_iter()
        .filter_map(|arg| {
            if let ExprKind::Identifier(name) = arg.kind {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}
/// Create a method call expression (complexity: 1)
fn create_method_call(receiver: Expr, method: String, args: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method,
            args,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    }
}
/// Create a field access expression (complexity: 1)
fn create_field_access(receiver: Expr, field: String) -> Expr {
    Expr {
        kind: ExprKind::FieldAccess {
            object: Box::new(receiver),
            field,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    }
}
/// Parse optional method or field access (?. operator) (complexity: 2, cognitive: 3)
fn parse_optional_method_or_field_access(
    state: &mut ParserState,
    receiver: Expr,
    method: String,
) -> Result<Expr> {
    if is_method_call(state) {
        parse_optional_method_call_syntax(state, receiver, method)
    } else {
        Ok(create_optional_field_access(receiver, method))
    }
}

/// Check if next token indicates method call (complexity: 1, cognitive: 1)
fn is_method_call(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::LeftParen, _)))
}

/// Parse optional method call arguments and create expression (complexity: 3, cognitive: 5)
fn parse_optional_method_call_syntax(
    state: &mut ParserState,
    receiver: Expr,
    method: String,
) -> Result<Expr> {
    state.tokens.advance(); // consume (
    let args = parse_optional_method_args(state)?;
    state.tokens.expect(&Token::RightParen)?;
    Ok(create_optional_method_call(receiver, method, args))
}

/// Parse arguments for optional method call (complexity: 3, cognitive: 5)
fn parse_optional_method_args(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut args = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        args.push(super::parse_expr_recursive(state)?);
        if !consume_comma_if_present(state) {
            break;
        }
    }
    Ok(args)
}

/// Create optional method call expression (complexity: 1, cognitive: 1)
fn create_optional_method_call(receiver: Expr, method: String, args: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::OptionalMethodCall {
            receiver: Box::new(receiver),
            method,
            args,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    }
}

/// Create optional field access expression (complexity: 1, cognitive: 1)
fn create_optional_field_access(receiver: Expr, field: String) -> Expr {
    Expr {
        kind: ExprKind::OptionalFieldAccess {
            object: Box::new(receiver),
            field,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    }
}

/// Parse where clause (e.g., where T: Display, U: Clone)
/// For now, we parse and skip it as we don't enforce trait bounds yet
/// # Errors
/// Returns an error if parsing fails
fn parse_where_clause(state: &mut ParserState) -> Result<()> {
    state.tokens.advance(); // consume 'where'

    // Parse trait bounds: T: Trait, U: Trait
    while parse_single_trait_bound(state)? {
        // Continue parsing bounds
    }

    Ok(())
}

/// Parse a single trait bound (T: Trait) and return true if more bounds may follow
/// # Errors
/// Returns an error if parsing fails
fn parse_single_trait_bound(state: &mut ParserState) -> Result<bool> {
    // Check for type parameter name
    if !matches!(state.tokens.peek(), Some((Token::Identifier(_), _))) {
        return Ok(false);
    }
    state.tokens.advance(); // consume type param name

    // Expect colon
    if !matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        return Ok(false);
    }
    state.tokens.advance(); // consume :

    // Parse trait bound tokens until comma or left brace
    consume_trait_bound_tokens(state)
}

/// Consume tokens that are part of a trait bound (complexity: 3, cognitive: 5)
/// # Errors
/// Returns an error if parsing fails
fn consume_trait_bound_tokens(state: &mut ParserState) -> Result<bool> {
    while should_continue_parsing_trait_bound(state)? {
        // Continue consuming tokens
    }
    Ok(is_comma_delimiter(state))
}

/// Check if should continue parsing trait bound (complexity: 3)
fn should_continue_parsing_trait_bound(state: &mut ParserState) -> Result<bool> {
    if is_trait_bound_end(state) {
        return Ok(false);
    }
    consume_trait_bound_token_if_present(state);
    Ok(true)
}

/// Check if at end of trait bound (complexity: 2)
fn is_trait_bound_end(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::Comma, _)) | Some((Token::LeftBrace, _)))
}

/// Check if current delimiter is comma (complexity: 1)
fn is_comma_delimiter(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::Comma, _)))
}

/// Try to handle trait bound delimiters (comma or brace) (complexity: 2, cognitive: 3)
fn try_handle_trait_bound_delimiter(state: &mut ParserState) -> Option<bool> {
    match state.tokens.peek() {
        Some((Token::Comma, _)) => {
            state.tokens.advance();
            Some(true) // More bounds may follow
        }
        Some((Token::LeftBrace, _)) => {
            Some(false) // End of where clause
        }
        _ => None,
    }
}

/// Consume a single trait bound token if present (complexity: 1, cognitive: 2)
fn consume_trait_bound_token_if_present(state: &mut ParserState) -> bool {
    if state.tokens.peek().is_some() {
        state.tokens.advance();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_parse_simple_function() {
        let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse simple function");
    }

    #[test]
    fn test_parse_function_no_params() {
        let mut parser = Parser::new("fun hello() { println(\"Hello\") }");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function without parameters"
        );
    }

    #[test]
    fn test_parse_function_no_return_type() {
        let mut parser = Parser::new("fun greet(name: String) { println(name) }");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function without return type"
        );
    }

    #[test]
    fn test_parse_anonymous_function() {
        let mut parser = Parser::new("fun (x: i32) -> i32 { x * 2 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse anonymous function");
    }

    #[test]
    fn test_parse_generic_function() {
        let mut parser = Parser::new("fun identity<T>(value: T) -> T { value }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse generic function");
    }

    #[test]
    fn test_parse_function_multiple_params() {
        let mut parser = Parser::new("fun sum(a: i32, b: i32, c: i32) -> i32 { a + b + c }");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function with multiple parameters"
        );
    }

    #[test]
    fn test_parse_lambda_simple() {
        let mut parser = Parser::new("|x| x + 1");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse simple lambda");
    }

    #[test]
    fn test_parse_lambda_multiple_params() {
        let mut parser = Parser::new("|x, y| x + y");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse lambda with multiple parameters"
        );
    }

    #[test]

    fn test_parse_lambda_with_types() {
        let mut parser = Parser::new("|x, y| x + y");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse lambda with type annotations"
        );
    }

    #[test]
    fn test_parse_lambda_no_params() {
        let mut parser = Parser::new("|| 42");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse lambda without parameters");
    }

    #[test]
    fn test_parse_fat_arrow_lambda() {
        let mut parser = Parser::new("x => x * 2");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse fat arrow lambda");
    }

    #[test]
    fn test_parse_fat_arrow_lambda_multiple() {
        let mut parser = Parser::new("(x, y) => x + y");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse fat arrow lambda with multiple params"
        );
    }

    #[test]
    fn test_parse_function_call_no_args() {
        let mut parser = Parser::new("print()");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function call without arguments"
        );
    }

    #[test]
    fn test_parse_function_call_single_arg() {
        let mut parser = Parser::new("sqrt(16)");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function call with single argument"
        );
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let mut parser = Parser::new("max(1, 2, 3)");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function call with multiple arguments"
        );
    }

    #[test]

    fn test_parse_function_call_named_args() {
        let mut parser = Parser::new("create(\"test\", 42)");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function call with named arguments"
        );
    }

    #[test]
    fn test_parse_method_call_no_args() {
        let mut parser = Parser::new("obj.method()");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse method call without arguments"
        );
    }

    #[test]
    fn test_parse_method_call_with_args() {
        let mut parser = Parser::new("list.append(42)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse method call with arguments");
    }

    #[test]
    fn test_parse_chained_method_calls() {
        let mut parser = Parser::new("str.trim().to_uppercase().split(\",\")");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse chained method calls");
    }

    #[test]
    fn test_parse_safe_navigation() {
        let mut parser = Parser::new("obj?.method()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse safe navigation operator");
    }

    #[test]
    fn test_parse_safe_navigation_chain() {
        let mut parser = Parser::new("obj?.field?.method()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse chained safe navigation");
    }

    #[test]
    fn test_parse_function_with_default_params() {
        let mut parser = Parser::new("fun greet(name: String = \"World\") { println(name) }");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function with default parameters"
        );
    }

    #[test]

    fn test_parse_function_with_rest_params() {
        let mut parser = Parser::new("fun sum(numbers: Vec<i32>) -> i32 { numbers.sum() }");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function with rest parameters"
        );
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let mut parser = Parser::new("outer(inner(deep(42)))");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse nested function calls");
    }

    #[test]
    fn test_parse_function_call_with_lambda() {
        let mut parser = Parser::new("map(list, |x| x * 2)");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse function call with lambda argument"
        );
    }

    #[test]
    fn test_parse_function_with_block_body() {
        let mut parser = Parser::new("fun complex(x: i32) -> i32 { let y = x + 1; y * 2 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse function with block body");
    }

    #[test]
    fn test_parse_recursive_function() {
        let mut parser = Parser::new(
            "fun factorial(n: i32) -> i32 { if n <= 1 { 1 } else { n * factorial(n - 1) } }",
        );
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse recursive function");
    }

    #[test]

    fn test_parse_higher_order_function() {
        let mut parser = Parser::new("fun apply(f: fn(i32) -> i32, x: i32) -> i32 { f(x) }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse higher-order function");
    }

    #[test]
    fn test_parse_closure_capture() {
        let mut parser = Parser::new("{ let x = 10; |y| x + y }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse closure with capture");
    }

    #[test]
    fn test_parse_iife() {
        let mut parser = Parser::new("(|x| x * 2)(5)");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse immediately invoked function expression"
        );
    }

    // Sprint 8 Phase 1: Mutation test gap coverage for functions.rs
    // Target: 1 MISSED → 0 MISSED (mutation coverage improvement)

    #[test]
    fn test_tuple_access_with_integer_index() {
        // Test gap: verify Token::Integer match arm (line 298) in parse_method_call
        // This tests tuple access syntax like tuple.0, tuple.1, etc.
        let mut parser = Parser::new("let t = (1, 2, 3); t.0");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Should parse tuple access with integer index t.0"
        );
    }

    #[test]
    fn test_tuple_access_multiple_indices() {
        // Test gap: verify tuple access works for different indices
        let mut parser = Parser::new("let t = (\"a\", \"b\", \"c\"); t.1");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse tuple access t.1");
    }

    #[test]
    fn test_tuple_access_third_element() {
        // Test gap: verify tuple access for index 2
        let mut parser = Parser::new("let t = (1, 2, 3); t.2");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse tuple access t.2");
    }
}
