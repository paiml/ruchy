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
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume fun
                                                                                 // For regular functions, async is not supported in this path
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
            is_pub,
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
            pattern: Pattern::Identifier(name),
            ty,
            span: Span { start: 0, end: 0 },
            is_mutable: false,
            default_value: None,
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
    let mut args = Vec::new();
    let mut named_args = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        // Check if this is a named argument (identifier followed by colon)
        let maybe_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            Some(name.clone())
        } else {
            None
        };

        let is_named = if let Some(name) = maybe_name {
            let saved_pos = state.tokens.position();
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
                state.tokens.advance(); // consume :
                let value = super::parse_expr_recursive(state)?;
                named_args.push((name, value));
                true
            } else {
                // Not a named arg, restore position
                state.tokens.set_position(saved_pos);
                false
            }
        } else {
            false
        };

        if !is_named {
            // Regular positional argument
            args.push(super::parse_expr_recursive(state)?);
        }

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }
    state.tokens.expect(&Token::RightParen)?;

    // If we have named arguments, convert to a struct literal call
    if named_args.is_empty() {
        Ok(Expr {
            kind: ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        })
    } else {
        // This is actually a constructor call with named arguments
        // Convert func(name: value) to func { name: value }
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
            })
        } else {
            // For now, only support named args with simple identifiers
            Ok(Expr {
                kind: ExprKind::Call {
                    func: Box::new(func),
                    args,
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
            })
        }
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
            let index = *index;
            state.tokens.advance();
            Ok(Expr {
                kind: ExprKind::FieldAccess {
                    object: Box::new(receiver),
                    field: index.to_string(),
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
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
            let index = *index;
            state.tokens.advance();
            Ok(Expr {
                kind: ExprKind::OptionalFieldAccess {
                    object: Box::new(receiver),
                    field: index.to_string(),
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
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
/// Parse method arguments (complexity: 5)
fn parse_method_arguments(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut args = Vec::new();
    let mut named_args = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        // Check if this is a named argument (identifier followed by colon)
        let maybe_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            Some(name.clone())
        } else {
            None
        };

        let is_named = if let Some(name) = maybe_name {
            let saved_pos = state.tokens.position();
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
                state.tokens.advance(); // consume :
                let value = super::parse_expr_recursive(state)?;
                named_args.push((name, value));
                true
            } else {
                // Not a named arg, restore position
                state.tokens.set_position(saved_pos);
                false
            }
        } else {
            false
        };

        if !is_named {
            // Regular positional argument
            args.push(super::parse_expr_recursive(state)?);
        }

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }

    // If we have named arguments, convert them to object literal
    if !named_args.is_empty() {
        // Convert named args to object literal fields
        use crate::frontend::ast::ObjectField;
        let fields = named_args
            .into_iter()
            .map(|(name, value)| ObjectField::KeyValue { key: name, value })
            .collect();

        // Get the current position's span for the object literal
        let span = if let Some((_, span)) = state.tokens.peek() {
            *span
        } else {
            crate::frontend::ast::Span::new(0, 0)
        };

        args.push(Expr::new(ExprKind::ObjectLiteral { fields }, span));
    }

    Ok(args)
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
    }
}
fn parse_optional_method_or_field_access(
    state: &mut ParserState,
    receiver: Expr,
    method: String,
) -> Result<Expr> {
    // Check if it's a method call (with parentheses) or field access
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        // Optional method call - convert to OptionalMethodCall AST node
        // For now, we'll just parse as regular method call but with optional semantics
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
        // Create an OptionalMethodCall expression
        Ok(Expr {
            kind: ExprKind::OptionalMethodCall {
                receiver: Box::new(receiver),
                method,
                args,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        })
    } else {
        // Optional field access
        Ok(Expr {
            kind: ExprKind::OptionalFieldAccess {
                object: Box::new(receiver),
                field: method,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        })
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
}
