//! Variable declaration parsing (let and var statements)
//!
//! Handles parsing of variable declarations with support for:
//! - Let bindings: `let x = value` or `let x = value in body`
//! - Mutable bindings: `let mut x = value`
//! - Var statements: `var x = value` (implicitly mutable)
//! - Type annotations: `let x: i32 = 42`
//! - Pattern matching: `let (x, y) = tuple`
//! - Let-else patterns: `let Some(x) = opt else { return }`
//!
//! # Examples
//! ```ruchy
//! // Simple let binding
//! let x = 42
//!
//! // Mutable binding
//! let mut count = 0
//!
//! // Type annotation
//! let name: String = "Alice"
//!
//! // Tuple destructuring
//! let (x, y) = (1, 2)
//!
//! // Let-else pattern
//! let Some(value) = optional else {
//!     return Err("Missing value")
//! }
//!
//! // Let-in expression
//! let x = 10 in x * 2
//!
//! // Var statement (mutable)
//! var counter = 0
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Literal, Pattern, Span, Type};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

// Import pattern parsing from patterns module
use super::patterns::{
    parse_list_pattern, parse_single_pattern, parse_struct_pattern, parse_struct_pattern_with_name,
    parse_tuple_pattern,
};

/// Parse let statement
///
/// Supports let bindings, let-in expressions, and let-else patterns.
pub(in crate::frontend::parser) fn parse_let_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Let)?;
    // Check for optional 'mut' keyword
    let is_mutable = parse_let_mutability(state);
    // Parse variable name or destructuring pattern
    let pattern = parse_let_pattern(state, is_mutable)?;
    // Parse optional type annotation
    let type_annotation = parse_let_type_annotation(state)?;
    // Parse '=' token
    state.tokens.expect(&Token::Equal)?;
    // Parse value expression, stopping at 'in' keyword for let-in expressions
    // Set context flag so nested expressions (like lambda body) also stop at 'in'
    let old_context = state.in_let_value_context;
    state.in_let_value_context = true;
    let value = Box::new(parse_expr_recursive(state)?);
    state.in_let_value_context = old_context;

    // Check for 'else' clause (let-else pattern)
    let else_block = parse_let_else_clause(state)?;

    // Parse optional 'in' clause for let expressions (not compatible with let-else)
    let body = if else_block.is_none() {
        parse_let_in_clause(state, value.span)?
    } else {
        // For let-else, body is unit (the else block handles divergence)
        Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span))
    };

    // Create the appropriate expression based on pattern type
    create_let_expression(
        pattern,
        type_annotation,
        value,
        body,
        is_mutable,
        else_block,
        start_span,
    )
}

/// Parse var statement (implicitly mutable)
///
/// Syntax: `var name [: type] = value`
pub(in crate::frontend::parser) fn parse_var_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Var)?;
    // var is always mutable

    let pattern = parse_var_pattern(state)?;
    let type_annotation = parse_optional_type_annotation(state)?;

    state.tokens.expect(&Token::Equal)?;
    let value = Box::new(parse_expr_recursive(state)?);

    create_var_expression(pattern, type_annotation, value, start_span)
}

/// Parse mutability keyword for let statement
fn parse_let_mutability(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse pattern for let statement
///
/// Supports identifiers, destructuring, and variant patterns.
fn parse_let_pattern(state: &mut ParserState, is_mutable: bool) -> Result<Pattern> {
    match state.tokens.peek() {
        // Handle Option::Some pattern
        Some((Token::Some, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_variant_pattern_with_name(state, "Some".to_string())
            } else {
                bail!("Some must be followed by parentheses in patterns: Some(value)")
            }
        }
        // Handle Result::Ok pattern
        Some((Token::Ok, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_variant_pattern_with_name(state, "Ok".to_string())
            } else {
                bail!("Ok must be followed by parentheses in patterns: Ok(value)")
            }
        }
        // Handle Result::Err pattern
        Some((Token::Err, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_variant_pattern_with_name(state, "Err".to_string())
            } else {
                bail!("Err must be followed by parentheses in patterns: Err(value)")
            }
        }
        // Handle Option::None pattern
        Some((Token::None, _)) => {
            state.tokens.advance();
            Ok(Pattern::None)
        }
        Some((Token::Identifier(_) | Token::Result | Token::Var, _)) => {
            // Handle identifier or reserved keywords that can be used as identifiers
            let name = match state.tokens.peek() {
                Some((Token::Identifier(n), _)) => n.clone(),
                Some((Token::Result, _)) => "Result".to_string(),
                Some((Token::Var, _)) => "var".to_string(),
                _ => bail!("Expected identifier in let pattern"),
            };
            state.tokens.advance();

            // Check if this is a variant pattern with custom variants
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                // Parse enum variant pattern with tuple destructuring
                parse_variant_pattern_with_name(state, name)
            }
            // Check if this is a struct pattern: Name { ... }
            else if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                parse_struct_pattern_with_name(state, name)
            } else {
                Ok(Pattern::Identifier(name))
            }
        }
        Some((Token::DataFrame, _)) => {
            // Allow 'df' as a variable name (common in data science)
            state.tokens.advance();
            Ok(Pattern::Identifier("df".to_string()))
        }
        Some((Token::Default, _)) => {
            // Allow 'default' as a variable name (common in configurations)
            state.tokens.advance();
            Ok(Pattern::Identifier("default".to_string()))
        }
        Some((Token::Final, _)) => {
            // Allow 'final' as a variable name (Rust keyword, needs r# prefix in transpiler)
            state.tokens.advance();
            Ok(Pattern::Identifier("final".to_string()))
        }
        Some((Token::Underscore, _)) => {
            // Allow wildcard pattern
            state.tokens.advance();
            Ok(Pattern::Identifier("_".to_string()))
        }
        Some((Token::LeftParen, _)) => {
            // Parse tuple destructuring: (x, y) = (1, 2)
            parse_tuple_pattern(state)
        }
        Some((Token::LeftBracket, _)) => {
            // Parse list destructuring: [a, b] = [1, 2]
            parse_list_pattern(state)
        }
        Some((Token::LeftBrace, _)) => {
            // Parse struct destructuring: {name, age} = obj
            parse_struct_pattern(state)
        }
        _ => bail!(
            "Expected identifier or pattern after 'let{}'",
            if is_mutable { " mut" } else { "" }
        ),
    }
}

/// Parse variant pattern with name
///
/// Examples: Some(x), Ok(val), Err(e), Color(r, g, b)
fn parse_variant_pattern_with_name(
    state: &mut ParserState,
    variant_name: String,
) -> Result<Pattern> {
    // At this point, we've consumed the variant name and peeked '('
    state.tokens.expect(&Token::LeftParen)?;

    // Parse patterns (could be single or multiple comma-separated)
    let mut patterns = vec![];

    // Parse first pattern
    if !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        patterns.push(parse_single_pattern(state)?);

        // Parse additional patterns separated by commas
        while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma

            // Check for trailing comma
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                break;
            }

            patterns.push(parse_single_pattern(state)?);
        }
    }

    state.tokens.expect(&Token::RightParen)?;

    // Try to create special pattern for common variants
    create_pattern_for_variant(variant_name, patterns)
}

/// Create pattern for variant
///
/// Special cases for Some/Ok/Err, otherwise `TupleVariant`.
fn create_pattern_for_variant(variant_name: String, patterns: Vec<Pattern>) -> Result<Pattern> {
    // Special case for common Option/Result variants (single element)
    if patterns.len() == 1 {
        match variant_name.as_str() {
            "Some" => {
                return Ok(Pattern::Some(Box::new(
                    patterns
                        .into_iter()
                        .next()
                        .expect("patterns.len() == 1 so next() must return Some"),
                )))
            }
            "Ok" => {
                return Ok(Pattern::Ok(Box::new(
                    patterns
                        .into_iter()
                        .next()
                        .expect("patterns.len() == 1 so next() must return Some"),
                )))
            }
            "Err" => {
                return Ok(Pattern::Err(Box::new(
                    patterns
                        .into_iter()
                        .next()
                        .expect("patterns.len() == 1 so next() must return Some"),
                )))
            }
            _ => {}
        }
    }

    // For other variants or multiple elements, use TupleVariant
    Ok(Pattern::TupleVariant {
        path: vec![variant_name],
        patterns,
    })
}

/// Parse optional type annotation
fn parse_let_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume ':'
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Parse optional 'else' clause for let-else patterns
fn parse_let_else_clause(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Else, _))) {
        state.tokens.advance(); // consume 'else'
                                // Must be followed by a block (diverging expression)
        if !matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
            bail!("let-else requires a block after 'else'");
        }
        let block = parse_expr_recursive(state)?;
        Ok(Some(Box::new(block)))
    } else {
        Ok(None)
    }
}

/// Parse optional 'in' clause for let expressions
fn parse_let_in_clause(state: &mut ParserState, value_span: Span) -> Result<Box<Expr>> {
    if matches!(state.tokens.peek(), Some((Token::In, _))) {
        state.tokens.advance(); // consume 'in'
        Ok(Box::new(parse_expr_recursive(state)?))
    } else {
        // For let statements (no 'in'), body is unit
        Ok(Box::new(Expr::new(
            ExprKind::Literal(Literal::Unit),
            value_span,
        )))
    }
}

/// Create let expression based on pattern type
fn create_let_expression(
    pattern: Pattern,
    type_annotation: Option<Type>,
    value: Box<Expr>,
    body: Box<Expr>,
    is_mutable: bool,
    else_block: Option<Box<Expr>>,
    start_span: Span,
) -> Result<Expr> {
    let end_span = body.span;
    match &pattern {
        Pattern::Identifier(name) => Ok(Expr::new(
            ExprKind::Let {
                name: name.clone(),
                type_annotation,
                value,
                body,
                is_mutable,
                else_block,
            },
            start_span.merge(end_span),
        )),
        Pattern::Tuple(_) | Pattern::List(_) => {
            // For destructuring patterns, use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                    else_block,
                },
                start_span.merge(end_span),
            ))
        }
        Pattern::Wildcard
        | Pattern::Literal(_)
        | Pattern::QualifiedName(_)
        | Pattern::Struct { .. }
        | Pattern::TupleVariant { .. }
        | Pattern::Range { .. }
        | Pattern::Or(_)
        | Pattern::Rest
        | Pattern::RestNamed(_)
        | Pattern::AtBinding { .. }
        | Pattern::WithDefault { .. }
        | Pattern::Ok(_)
        | Pattern::Err(_)
        | Pattern::Some(_)
        | Pattern::None
        | Pattern::Mut(_) => {
            // For other pattern types, use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                    else_block,
                },
                start_span.merge(end_span),
            ))
        }
    }
}

/// Parse variable pattern for var statement
fn parse_var_pattern(state: &mut ParserState) -> Result<Pattern> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        Some((Token::DataFrame, _)) => {
            // Allow 'df' as a variable name (common in data science)
            state.tokens.advance();
            Ok(Pattern::Identifier("df".to_string()))
        }
        Some((Token::Underscore, _)) => {
            // Allow wildcard pattern in var statements too
            state.tokens.advance();
            Ok(Pattern::Identifier("_".to_string()))
        }
        Some((Token::LeftParen, _)) => parse_tuple_pattern(state),
        Some((Token::LeftBracket, _)) => parse_list_pattern(state),
        _ => bail!("Expected identifier or pattern after 'var'"),
    }
}

/// Parse optional type annotation
fn parse_optional_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Create variable expression (var is always mutable)
fn create_var_expression(
    pattern: Pattern,
    type_annotation: Option<Type>,
    value: Box<Expr>,
    start_span: Span,
) -> Result<Expr> {
    let body = Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span));
    let end_span = value.span;
    let is_mutable = true;

    match &pattern {
        Pattern::Identifier(name) => Ok(Expr::new(
            ExprKind::Let {
                name: name.clone(),
                type_annotation,
                value,
                body,
                is_mutable,
                else_block: None,
            },
            start_span.merge(end_span),
        )),
        Pattern::Tuple(_) | Pattern::List(_) | Pattern::Wildcard => Ok(Expr::new(
            ExprKind::LetPattern {
                pattern,
                type_annotation,
                value,
                body,
                is_mutable,
                else_block: None,
            },
            start_span.merge(end_span),
        )),
        _ => bail!("var only supports simple patterns (identifier, tuple, list, wildcard)"),
    }
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_let_simple() {
        let code = "let x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple let binding should parse");
    }

    #[test]
    fn test_let_mut() {
        let code = "let mut x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Mutable let binding should parse");
    }

    #[test]
    fn test_let_with_type() {
        let code = "let x: i32 = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Let with type annotation should parse");
    }

    #[test]
    fn test_let_tuple_destructuring() {
        let code = "let (x, y) = (1, 2)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple destructuring should parse");
    }

    #[test]
    fn test_let_in_expression() {
        let code = "let x = 10 in x * 2";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Let-in expression should parse");
    }

    #[test]
    fn test_let_else_pattern() {
        let code = "let Some(x) = opt else { return }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Let-else pattern should parse");
    }

    #[test]
    fn test_var_statement() {
        let code = "var x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Var statement should parse");
    }

    #[test]
    fn test_var_with_type() {
        let code = "var count: i32 = 0";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Var with type should parse");
    }

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::{Expr, ExprKind, Pattern};
    use crate::frontend::parser::Result;

    fn parse(code: &str) -> Result<Expr> {
        Parser::new(code).parse()
    }

    fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
        match &expr.kind {
            ExprKind::Block(exprs) => Some(exprs),
            _ => None,
        }
    }

    // ============================================================
    // Let statement tests
    // ============================================================

    #[test]
    fn test_let_creates_let_expr_kind() {
        let expr = parse("let x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Let { .. }),
                "let should produce ExprKind::Let"
            );
        }
    }

    #[test]
    fn test_let_name_is_captured() {
        let expr = parse("let foo = 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { name, .. } = &exprs[0].kind {
                assert_eq!(name, "foo", "Let name should be 'foo'");
            }
        }
    }

    #[test]
    fn test_let_mut_sets_is_mutable_true() {
        let expr = parse("let mut x = 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(*is_mutable, "let mut should set is_mutable=true");
            }
        }
    }

    #[test]
    fn test_let_without_mut_is_immutable() {
        let expr = parse("let x = 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(!*is_mutable, "let without mut should set is_mutable=false");
            }
        }
    }

    #[test]
    fn test_let_with_string_value() {
        let code = r#"let name = "Alice""#;
        let result = parse(code);
        assert!(result.is_ok(), "Let with string value should parse");
    }

    #[test]
    fn test_let_with_boolean_value() {
        let result = parse("let flag = true");
        assert!(result.is_ok(), "Let with boolean value should parse");
    }

    #[test]
    fn test_let_with_float_value() {
        let result = parse("let pi = 3.14");
        assert!(result.is_ok(), "Let with float value should parse");
    }

    #[test]
    fn test_let_with_array_value() {
        let result = parse("let arr = [1, 2, 3]");
        assert!(result.is_ok(), "Let with array value should parse");
    }

    #[test]
    fn test_let_with_expression_value() {
        let result = parse("let sum = 1 + 2 + 3");
        assert!(result.is_ok(), "Let with expression value should parse");
    }

    #[test]
    fn test_let_with_function_call_value() {
        let result = parse("let result = foo(1, 2)");
        assert!(result.is_ok(), "Let with function call value should parse");
    }

    #[test]
    fn test_let_with_lambda_value() {
        let result = parse("let f = |x| x * 2");
        assert!(result.is_ok(), "Let with lambda value should parse");
    }

    // ============================================================
    // Type annotation tests
    // ============================================================

    #[test]
    fn test_let_type_annotation_i32() {
        let expr = parse("let x: i32 = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let {
                type_annotation, ..
            } = &exprs[0].kind
            {
                assert!(
                    type_annotation.is_some(),
                    "Type annotation should be present"
                );
            }
        }
    }

    #[test]
    fn test_let_type_annotation_string() {
        let result = parse(r#"let s: String = "hello""#);
        assert!(result.is_ok(), "Let with String type should parse");
    }

    #[test]
    fn test_let_type_annotation_bool() {
        let result = parse("let b: bool = false");
        assert!(result.is_ok(), "Let with bool type should parse");
    }

    #[test]
    fn test_let_type_annotation_f64() {
        let result = parse("let f: f64 = 3.14");
        assert!(result.is_ok(), "Let with f64 type should parse");
    }

    #[test]
    fn test_let_type_annotation_vec() {
        let result = parse("let v: Vec<i32> = [1, 2, 3]");
        assert!(result.is_ok(), "Let with Vec type should parse");
    }

    #[test]
    fn test_let_type_annotation_option() {
        let result = parse("let opt: Option<i32> = Some(42)");
        assert!(result.is_ok(), "Let with Option type should parse");
    }

    #[test]
    fn test_let_mut_with_type_annotation() {
        let result = parse("let mut count: i32 = 0");
        assert!(result.is_ok(), "Let mut with type should parse");
    }

    // ============================================================
    // Let-in expression tests
    // ============================================================

    #[test]
    fn test_let_in_simple() {
        let expr = parse("let x = 1 in x + 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { body, .. } = &exprs[0].kind {
                assert!(
                    !matches!(body.kind, ExprKind::Literal(_)),
                    "Body should not be unit literal"
                );
            }
        }
    }

    #[test]
    fn test_let_in_nested() {
        let result = parse("let x = 1 in let y = 2 in x + y");
        assert!(result.is_ok(), "Nested let-in should parse");
    }

    #[test]
    fn test_let_in_with_function_call() {
        let result = parse("let x = 1 in foo(x)");
        assert!(result.is_ok(), "Let-in with function call should parse");
    }

    #[test]
    fn test_let_in_with_lambda() {
        let result = parse("let f = |x| x in f(1)");
        assert!(result.is_ok(), "Let-in with lambda should parse");
    }

    #[test]
    fn test_let_in_with_block() {
        let result = parse("let x = 1 in { x + 1 }");
        assert!(result.is_ok(), "Let-in with block body should parse");
    }

    // ============================================================
    // Let-else pattern tests
    // ============================================================

    #[test]
    fn test_let_else_with_some() {
        let result = parse("let Some(x) = opt else { return }");
        assert!(result.is_ok(), "Let-else with Some should parse");
    }

    #[test]
    fn test_let_else_with_ok() {
        let result = parse("let Ok(val) = result else { return }");
        assert!(result.is_ok(), "Let-else with Ok should parse");
    }

    #[test]
    fn test_let_else_with_err() {
        let result = parse("let Err(e) = result else { return }");
        assert!(result.is_ok(), "Let-else with Err should parse");
    }

    #[test]
    fn test_let_else_produces_let_pattern() {
        let expr = parse("let Some(x) = opt else { return }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::LetPattern { .. }),
                "Let-else should produce LetPattern"
            );
        }
    }

    #[test]
    fn test_let_else_has_else_block() {
        let expr = parse("let Some(x) = opt else { panic() }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::LetPattern { else_block, .. } = &exprs[0].kind {
                assert!(else_block.is_some(), "else_block should be Some");
            }
        }
    }

    #[test]
    fn test_let_else_complex_block() {
        let result = parse(
            r#"let Some(x) = opt else {
            println("error")
            return
        }"#,
        );
        assert!(result.is_ok(), "Let-else with complex block should parse");
    }

    // ============================================================
    // Pattern variant tests (Some, Ok, Err, None)
    // ============================================================

    #[test]
    fn test_let_some_pattern() {
        let expr = parse("let Some(value) = opt else { return }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::LetPattern { pattern, .. } = &exprs[0].kind {
                assert!(
                    matches!(pattern, Pattern::Some(_)),
                    "Should produce Some pattern"
                );
            }
        }
    }

    #[test]
    fn test_let_ok_pattern() {
        let expr = parse("let Ok(value) = res else { return }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::LetPattern { pattern, .. } = &exprs[0].kind {
                assert!(
                    matches!(pattern, Pattern::Ok(_)),
                    "Should produce Ok pattern"
                );
            }
        }
    }

    #[test]
    fn test_let_err_pattern() {
        let expr = parse("let Err(e) = res else { return }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::LetPattern { pattern, .. } = &exprs[0].kind {
                assert!(
                    matches!(pattern, Pattern::Err(_)),
                    "Should produce Err pattern"
                );
            }
        }
    }

    #[test]
    fn test_let_none_pattern() {
        let result = parse("let None = opt else { return }");
        assert!(result.is_ok(), "Let with None pattern should parse");
    }

    // ============================================================
    // Tuple destructuring tests
    // ============================================================

    #[test]
    fn test_let_tuple_two_elements() {
        let result = parse("let (a, b) = (1, 2)");
        assert!(result.is_ok(), "Tuple with 2 elements should parse");
    }

    #[test]
    fn test_let_tuple_three_elements() {
        let result = parse("let (x, y, z) = (1, 2, 3)");
        assert!(result.is_ok(), "Tuple with 3 elements should parse");
    }

    #[test]
    fn test_let_tuple_nested() {
        let result = parse("let ((a, b), c) = ((1, 2), 3)");
        assert!(result.is_ok(), "Nested tuple should parse");
    }

    #[test]
    fn test_let_tuple_produces_let_pattern() {
        let expr = parse("let (x, y) = (1, 2)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::LetPattern { .. }),
                "Tuple destructuring should produce LetPattern"
            );
        }
    }

    #[test]
    fn test_let_tuple_with_type() {
        let result = parse("let (x, y): (i32, i32) = (1, 2)");
        assert!(result.is_ok(), "Tuple with type annotation should parse");
    }

    #[test]
    fn test_let_mut_tuple() {
        let result = parse("let mut (x, y) = (1, 2)");
        assert!(result.is_ok(), "Mutable tuple destructuring should parse");
    }

    // ============================================================
    // List destructuring tests
    // ============================================================

    #[test]
    fn test_let_list_destructure() {
        let result = parse("let [a, b] = [1, 2]");
        assert!(result.is_ok(), "List destructuring should parse");
    }

    #[test]
    fn test_let_list_three_elements() {
        let result = parse("let [x, y, z] = [1, 2, 3]");
        assert!(result.is_ok(), "List with 3 elements should parse");
    }

    #[test]
    fn test_let_list_produces_let_pattern() {
        let expr = parse("let [a, b] = arr").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::LetPattern { .. }),
                "List destructuring should produce LetPattern"
            );
        }
    }

    // ============================================================
    // Struct destructuring tests
    // ============================================================

    #[test]
    fn test_let_struct_destructure() {
        let result = parse("let Point { x, y } = point");
        assert!(result.is_ok(), "Struct destructuring should parse");
    }

    #[test]
    fn test_let_struct_destructure_brace_only() {
        let result = parse("let { name, age } = person");
        assert!(result.is_ok(), "Brace-only struct destructuring should parse");
    }

    #[test]
    fn test_let_struct_destructure_produces_let_pattern() {
        let expr = parse("let Point { x, y } = p").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::LetPattern { .. }),
                "Struct destructuring should produce LetPattern"
            );
        }
    }

    // ============================================================
    // Special identifier tests
    // ============================================================

    #[test]
    fn test_let_underscore_pattern() {
        let result = parse("let _ = something()");
        assert!(result.is_ok(), "Underscore pattern should parse");
    }

    #[test]
    fn test_let_df_identifier() {
        let result = parse("let df = load_data()");
        assert!(result.is_ok(), "'df' identifier should parse");
    }

    #[test]
    fn test_let_default_identifier() {
        let result = parse("let default = get_default()");
        assert!(result.is_ok(), "'default' identifier should parse");
    }

    #[test]
    fn test_let_final_identifier() {
        let result = parse("let final = compute_final()");
        assert!(result.is_ok(), "'final' identifier should parse");
    }

    #[test]
    fn test_let_result_identifier() {
        let result = parse("let Result = compute()");
        assert!(result.is_ok(), "'Result' identifier should parse");
    }

    // ============================================================
    // Var statement tests
    // ============================================================

    #[test]
    fn test_var_is_always_mutable() {
        let expr = parse("var x = 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(*is_mutable, "var should always be mutable");
            }
        }
    }

    #[test]
    fn test_var_with_string_value() {
        let result = parse(r#"var name = "Bob""#);
        assert!(result.is_ok(), "Var with string should parse");
    }

    #[test]
    fn test_var_with_array() {
        let result = parse("var arr = [1, 2, 3]");
        assert!(result.is_ok(), "Var with array should parse");
    }

    #[test]
    fn test_var_tuple_destructure() {
        let result = parse("var (x, y) = (1, 2)");
        assert!(result.is_ok(), "Var tuple destructuring should parse");
    }

    #[test]
    fn test_var_list_destructure() {
        let result = parse("var [a, b] = [1, 2]");
        assert!(result.is_ok(), "Var list destructuring should parse");
    }

    #[test]
    fn test_var_df_identifier() {
        let result = parse("var df = load_dataframe()");
        assert!(result.is_ok(), "Var with 'df' should parse");
    }

    #[test]
    fn test_var_underscore() {
        let result = parse("var _ = ignored()");
        assert!(result.is_ok(), "Var with underscore should parse");
    }

    #[test]
    fn test_var_type_i64() {
        let result = parse("var big: i64 = 1000000");
        assert!(result.is_ok(), "Var with i64 type should parse");
    }

    #[test]
    fn test_var_type_vec_string() {
        let result = parse(r#"var names: Vec<String> = ["a", "b"]"#);
        assert!(result.is_ok(), "Var with Vec<String> type should parse");
    }

    // ============================================================
    // Custom variant patterns (enum variants)
    // ============================================================

    #[test]
    fn test_let_custom_variant_single() {
        let result = parse("let Color(r) = c else { return }");
        assert!(result.is_ok(), "Custom variant with single element should parse");
    }

    #[test]
    fn test_let_custom_variant_multiple() {
        let result = parse("let Point(x, y) = p else { return }");
        assert!(result.is_ok(), "Custom variant with multiple elements should parse");
    }

    #[test]
    fn test_let_custom_variant_produces_tuple_variant() {
        let expr = parse("let Color(r, g, b) = c else { return }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::LetPattern { pattern, .. } = &exprs[0].kind {
                assert!(
                    matches!(pattern, Pattern::TupleVariant { .. }),
                    "Should produce TupleVariant pattern"
                );
            }
        }
    }

    // ============================================================
    // Edge cases and complex expressions
    // ============================================================

    #[test]
    fn test_let_with_if_value() {
        let result = parse("let x = if cond { 1 } else { 2 }");
        assert!(result.is_ok(), "Let with if expression value should parse");
    }

    #[test]
    fn test_let_with_match_value() {
        let result = parse("let x = match opt { Some(v) => v, None => 0 }");
        assert!(result.is_ok(), "Let with match expression value should parse");
    }

    #[test]
    fn test_let_with_block_value() {
        let result = parse("let x = { let a = 1; a + 1 }");
        assert!(result.is_ok(), "Let with block value should parse");
    }

    #[test]
    fn test_let_chain_multiple() {
        let result = parse("let a = 1; let b = 2; let c = a + b");
        assert!(result.is_ok(), "Multiple let statements should parse");
    }

    #[test]
    fn test_let_with_method_chain() {
        let result = parse("let result = data.filter().map().collect()");
        assert!(result.is_ok(), "Let with method chain should parse");
    }

    #[test]
    fn test_let_with_index_value() {
        let result = parse("let x = arr[0]");
        assert!(result.is_ok(), "Let with index value should parse");
    }

    #[test]
    fn test_let_with_field_access() {
        let result = parse("let x = obj.field");
        assert!(result.is_ok(), "Let with field access should parse");
    }

    #[test]
    fn test_let_in_with_complex_body() {
        let result = parse("let x = 1 in if x > 0 { x } else { 0 }");
        assert!(result.is_ok(), "Let-in with complex body should parse");
    }

    #[test]
    fn test_let_with_negative_number() {
        let result = parse("let x = -42");
        assert!(result.is_ok(), "Let with negative number should parse");
    }

    #[test]
    fn test_let_with_tuple_value() {
        let result = parse("let pair = (1, 2)");
        assert!(result.is_ok(), "Let with tuple value should parse");
    }

    #[test]
    fn test_let_with_range_value() {
        let result = parse("let r = 0..10");
        assert!(result.is_ok(), "Let with range value should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_let_with_identifiers(name in "[a-z]+", value in 0i32..100) {
                let code = format!("let {name} = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_mut_parses(name in "[a-z]+", value in 0i32..100) {
                let code = format!("let mut {name} = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_var_parses(name in "[a-z]+", value in 0i32..100) {
                let code = format!("var {name} = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_with_tuple(n1 in 0i32..100, n2 in 0i32..100) {
                let code = format!("let (x, y) = ({n1}, {n2})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_in_expr(name in "[a-z]+", val in 0i32..100, expr in 0i32..100) {
                let code = format!("let {name} = {val} in {expr}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_type_annotation(name in "[a-z]+", value in 0i32..100) {
                let code = format!("let {name}: i32 = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_var_tuple_destructuring(n1 in 0i32..100, n2 in 0i32..100) {
                let code = format!("var (a, b) = ({n1}, {n2})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
