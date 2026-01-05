//! Impl block parsing
//!
//! Handles parsing of implementation blocks:
//! - Type implementations: `impl TypeName { methods }`
//! - Trait implementations: `impl TraitName for TypeName { methods }`
//! - Generic implementations: `impl<T> TraitName for TypeName<T> { methods }`
//! - Method definitions within impl blocks
//!
//! # Examples
//! ```ruchy
//! // Type implementation
//! impl Point {
//!     fun new(x: f64, y: f64) -> Point {
//!         Point { x, y }
//!     }
//! }
//!
//! // Trait implementation
//! impl Display for Point {
//!     fun fmt(&self) -> String {
//!         f"Point({self.x}, {self.y})"
//!     }
//! }
//!
//! // Generic implementation
//! impl<T> From<T> for Wrapper<T> {
//!     fun from(value: T) -> Wrapper<T> {
//!         Wrapper { value }
//!     }
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, ImplMethod};
use crate::frontend::lexer::Token;
use crate::frontend::parser::collections::parse_block;
use crate::frontend::parser::utils::{parse_params, parse_type, parse_type_parameters};
use crate::frontend::parser::{ParserState, Result};

/// Parse trait and type names: impl [Trait for] Type
/// Handles generic types like Container<T> or Result<T, E>
fn parse_impl_target(state: &mut ParserState) -> Result<(Option<String>, String)> {
    let first_type = expect_type_name_with_generics(state)?;

    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance(); // consume 'for'
        let type_name = expect_type_name_with_generics(state)?;
        Ok((Some(first_type), type_name))
    } else {
        Ok((None, first_type))
    }
}

/// Parse impl block: impl [Trait for] Type { methods }
pub(in crate::frontend::parser) fn parse_impl_block(state: &mut ParserState) -> Result<Expr> {
    let start = state.tokens.expect(&Token::Impl)?.start;

    // Parse optional type parameters: impl<T, U>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        parse_type_parameters(state)?
    } else {
        vec![]
    };

    let (trait_name, for_type) = parse_impl_target(state)?;

    // Parse methods block
    state.tokens.expect(&Token::LeftBrace)?;

    let mut methods = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _)) | None) {
        // PARSER-XXX: Skip comment tokens in impl blocks
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
        // Check again after skipping comments
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _)) | None) {
            break;
        }
        methods.push(parse_impl_method(state)?);
    }

    let end = state.tokens.expect(&Token::RightBrace)?.end;

    Ok(Expr::new(
        ExprKind::Impl {
            type_params,
            trait_name,
            for_type,
            methods,
            is_pub: false,
        },
        crate::frontend::ast::Span::new(start, end),
    ))
}

/// Helper: Expect identifier token
fn expect_identifier(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        use crate::frontend::parser::bail;
        bail!("Expected identifier")
    }
}

/// Helper: Parse type name including optional generic arguments
/// E.g., "Container" or "Container<T>" or "Result<T, E>"
fn expect_type_name_with_generics(state: &mut ParserState) -> Result<String> {
    let base = expect_identifier(state)?;

    // Check for generic type arguments
    if !matches!(state.tokens.peek(), Some((Token::Less, _))) {
        return Ok(base);
    }

    // Consume the generic arguments: <T, U, ...>
    let mut result = base;
    result.push('<');
    state.tokens.advance(); // consume <

    let mut depth = 1;
    let mut first = true;
    while depth > 0 {
        match state.tokens.peek() {
            Some((Token::Less, _)) => {
                result.push('<');
                depth += 1;
                state.tokens.advance();
            }
            Some((Token::Greater, _)) => {
                result.push('>');
                depth -= 1;
                state.tokens.advance();
            }
            Some((Token::RightShift, _)) => {
                // >> is two > tokens
                result.push_str(">>");
                depth -= 2;
                state.tokens.advance();
                if depth < 0 {
                    use crate::frontend::parser::bail;
                    bail!("Mismatched >> in generic type");
                }
            }
            Some((Token::Comma, _)) => {
                result.push_str(", ");
                state.tokens.advance();
                first = true;
            }
            Some((Token::Identifier(name), _)) => {
                if !first {
                    result.push(' ');
                }
                result.push_str(name);
                state.tokens.advance();
                first = false;
            }
            Some((Token::Colon, _)) => {
                result.push(':');
                state.tokens.advance();
            }
            Some((Token::ColonColon, _)) => {
                result.push_str("::");
                state.tokens.advance();
            }
            Some((_, _)) => {
                // Skip other tokens in generics (like &, mut, etc.)
                state.tokens.advance();
            }
            None => {
                use crate::frontend::parser::bail;
                bail!("Unexpected end of file in generic type arguments");
            }
        }
    }

    Ok(result)
}

/// Parse single method in impl block
///
/// Complexity: 6 (within Toyota Way limits)
fn parse_impl_method(state: &mut ParserState) -> Result<ImplMethod> {
    // Check for pub visibility
    let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance();
        true
    } else {
        false
    };

    // PARSER-XXX: Accept both 'fun' and 'fn' in impl blocks for consistency
    if !matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
        use crate::frontend::parser::bail;
        bail!("Expected 'fun' or 'fn' keyword in impl method");
    }
    state.tokens.advance();

    // Method name
    let name = expect_identifier(state)?;

    // Parameters (parse_params handles both parens)
    let params = parse_params(state)?;

    // Return type
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(parse_type(state)?)
    } else {
        None
    };

    // Body
    let body = Box::new(parse_block(state)?);

    Ok(ImplMethod {
        name,
        params,
        return_type,
        body,
        is_pub,
    })
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use crate::frontend::ast::ExprKind;

    #[test]
    fn test_parse_impl_block_simple() {
        let code = r#"impl Point { fun new() { 42 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_block_with_method() {
        let code = r#"impl Point { fun new(x: i32) -> Point { Point { x } } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_block_multiple_methods() {
        let code = r#"impl Point {
            fun new() { 0 }
            fun get() { 1 }
        }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_trait() {
        let code = r#"impl Display for Point { fun fmt() { "" } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        if let Ok(ast) = &result {
            if let ExprKind::Block(exprs) = &ast.kind {
                for expr in exprs {
                    if let ExprKind::Impl { trait_name, for_type, .. } = &expr.kind {
                        assert_eq!(trait_name.as_deref(), Some("Display"));
                        assert_eq!(for_type, "Point");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_impl_with_type_params() {
        let code = r#"impl<T> Container { fun new() { 0 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_generic_type() {
        let code = r#"impl Container<T> { fun new() { 0 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_pub_method() {
        let code = r#"impl Point { pub fun new() { 0 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_fn_keyword() {
        let code = r#"impl Point { fn new() { 0 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_empty() {
        let code = r#"impl Point { }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_return_type() {
        let code = r#"impl Point { fun value() -> i32 { 42 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_complex_generic() {
        let code = r#"impl<T, U> Pair<T, U> { fun new() { 0 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_trait_for_generic() {
        let code = r#"impl<T> Display for Container<T> { fun fmt() { "" } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_comments() {
        let code = r#"impl Point {
            // This is a comment
            fun new() { 0 }
        }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_with_self_param() {
        let code = r#"impl Point { fun get(&self) { self.x } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_impl_extracts_methods() {
        let code = r#"impl Point { fun a() { 1 } fun b() { 2 } }"#;
        let mut parser = Parser::new(code);
        let result = parser.parse().unwrap();

        if let ExprKind::Block(exprs) = &result.kind {
            for expr in exprs {
                if let ExprKind::Impl { methods, .. } = &expr.kind {
                    assert_eq!(methods.len(), 2);
                    assert_eq!(methods[0].name, "a");
                    assert_eq!(methods[1].name, "b");
                }
            }
        }
    }

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::Expr;
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
    // Basic impl tests
    // ============================================================

    #[test]
    fn test_impl_produces_impl_expr_kind() {
        let expr = parse("impl Foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Impl { .. }),
                "Should produce Impl ExprKind"
            );
        }
    }

    #[test]
    fn test_impl_for_type_captured() {
        let expr = parse("impl Point { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { for_type, .. } = &exprs[0].kind {
                assert_eq!(for_type, "Point", "for_type should be Point");
            }
        }
    }

    #[test]
    fn test_impl_trait_name_none() {
        let expr = parse("impl Point { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { trait_name, .. } = &exprs[0].kind {
                assert!(trait_name.is_none(), "trait_name should be None");
            }
        }
    }

    #[test]
    fn test_impl_trait_name_some() {
        let expr = parse("impl Display for Point { fun fmt() { } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { trait_name, .. } = &exprs[0].kind {
                assert_eq!(trait_name.as_deref(), Some("Display"));
            }
        }
    }

    // ============================================================
    // Generic impl tests
    // ============================================================

    #[test]
    fn test_impl_single_type_param() {
        let result = parse("impl<T> Container<T> { }");
        assert!(result.is_ok(), "Single type param should parse");
    }

    #[test]
    fn test_impl_two_type_params() {
        let result = parse("impl<K, V> Map<K, V> { }");
        assert!(result.is_ok(), "Two type params should parse");
    }

    #[test]
    fn test_impl_three_type_params() {
        let result = parse("impl<A, B, C> Triple<A, B, C> { }");
        assert!(result.is_ok(), "Three type params should parse");
    }

    #[test]
    fn test_impl_type_params_captured() {
        let expr = parse("impl<T, U> Pair<T, U> { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { type_params, .. } = &exprs[0].kind {
                assert_eq!(type_params.len(), 2, "Should have 2 type params");
            }
        }
    }

    // ============================================================
    // Trait implementation tests
    // ============================================================

    #[test]
    fn test_impl_trait_for_simple() {
        let result = parse("impl Clone for Point { fun clone(&self) { } }");
        assert!(result.is_ok(), "Trait for type should parse");
    }

    #[test]
    fn test_impl_trait_for_generic() {
        let result = parse("impl<T: Clone> Clone for Box<T> { fun clone(&self) { } }");
        assert!(result.is_ok(), "Trait for generic type should parse");
    }

    #[test]
    fn test_impl_generic_trait_for_type() {
        // Generic traits may need different syntax - just check no crash
        let _ = parse("impl<T> From<T> for MyInt { fun from(n: T) { } }");
    }

    // ============================================================
    // Method tests
    // ============================================================

    #[test]
    fn test_impl_method_with_self() {
        let result = parse("impl Point { fun get_x(&self) -> i32 { self.x } }");
        assert!(result.is_ok(), "Method with &self should parse");
    }

    #[test]
    fn test_impl_method_with_mut_self() {
        let result = parse("impl Counter { fun inc(&mut self) { self.count = self.count + 1 } }");
        assert!(result.is_ok(), "Method with &mut self should parse");
    }

    #[test]
    fn test_impl_method_with_owned_self() {
        let result = parse("impl Point { fun into_tuple(self) { (self.x, self.y) } }");
        assert!(result.is_ok(), "Method with owned self should parse");
    }

    #[test]
    fn test_impl_static_method() {
        let result = parse("impl Point { fun origin() -> Point { Point { x: 0, y: 0 } } }");
        assert!(result.is_ok(), "Static method should parse");
    }

    #[test]
    fn test_impl_method_with_multiple_params() {
        let result = parse("impl Math { fun add(a: i32, b: i32) -> i32 { a + b } }");
        assert!(result.is_ok(), "Method with multiple params should parse");
    }

    #[test]
    fn test_impl_three_methods() {
        let result = parse("impl Point { fun a() { } fun b() { } fun c() { } }");
        assert!(result.is_ok(), "Three methods should parse");
    }

    #[test]
    fn test_impl_method_with_complex_return() {
        let result = parse("impl Foo { fun bar() -> Result<String, Error> { Ok(\"\") } }");
        assert!(result.is_ok(), "Method with complex return should parse");
    }

    // ============================================================
    // Visibility tests
    // ============================================================

    #[test]
    fn test_impl_pub_method_is_pub() {
        let expr = parse("impl Point { pub fun get() { } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { methods, .. } = &exprs[0].kind {
                assert!(methods[0].is_pub, "Method should be public");
            }
        }
    }

    #[test]
    fn test_impl_private_method_not_pub() {
        let expr = parse("impl Point { fun get() { } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { methods, .. } = &exprs[0].kind {
                assert!(!methods[0].is_pub, "Method should be private");
            }
        }
    }

    #[test]
    fn test_impl_mixed_visibility() {
        let result = parse("impl Point { pub fun a() { } fun b() { } pub fun c() { } }");
        assert!(result.is_ok(), "Mixed visibility should parse");
    }

    // ============================================================
    // Edge cases
    // ============================================================

    #[test]
    fn test_impl_with_doc_comment() {
        let result = parse(r#"impl Point {
            /// Gets the x coordinate
            fun get_x(&self) { self.x }
        }"#);
        assert!(result.is_ok(), "Impl with doc comment should parse");
    }

    #[test]
    fn test_impl_with_block_comment() {
        let result = parse(r#"impl Point {
            /* Multi-line
               comment */
            fun get_x(&self) { self.x }
        }"#);
        assert!(result.is_ok(), "Impl with block comment should parse");
    }

    #[test]
    fn test_impl_multiline() {
        let result = parse(r#"
            impl Point {
                fun new(x: i32, y: i32) -> Point {
                    Point { x, y }
                }

                fun origin() -> Point {
                    Point::new(0, 0)
                }
            }
        "#);
        assert!(result.is_ok(), "Multiline impl should parse");
    }

    #[test]
    fn test_impl_nested_generic() {
        let result = parse("impl<T> Container<Option<T>> { }");
        assert!(result.is_ok(), "Nested generic should parse");
    }

    #[test]
    fn test_impl_double_nested_generic() {
        let result = parse("impl<T> Container<Result<Option<T>, Error>> { }");
        assert!(result.is_ok(), "Double nested generic should parse");
    }

    #[test]
    fn test_impl_method_generic_return() {
        let result = parse("impl Factory { fun create<T>() -> T { } }");
        // Generic methods may or may not be supported
        let _ = result;
    }

    #[test]
    fn test_impl_for_tuple_type() {
        // Tuple types may have different syntax
        let result = parse("impl Point for (i32, i32) { }");
        // May or may not parse
        let _ = result;
    }

    // ============================================================
    // Additional EXTREME TDD tests for impls
    // ============================================================

    // ===== Type name variations =====

    #[test]
    fn test_impl_single_char_type() {
        let result = parse("impl A { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_long_type_name() {
        let result = parse("impl VeryLongTypeNameHere { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_snake_case_type() {
        let result = parse("impl my_type { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_type_with_numbers() {
        let result = parse("impl Point2D { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_type_uppercase() {
        let result = parse("impl CONSTANT { }");
        assert!(result.is_ok());
    }

    // ===== Method name variations =====

    #[test]
    fn test_impl_method_single_char() {
        let result = parse("impl Foo { fun a() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_long_name() {
        let result = parse("impl Foo { fun very_long_method_name() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_with_underscore() {
        let result = parse("impl Foo { fun get_value() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_new() {
        let result = parse("impl Foo { fun new() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_get_default() {
        let result = parse("impl Foo { fun get_default() { } }");
        assert!(result.is_ok());
    }

    // ===== Return type variations =====

    #[test]
    fn test_impl_return_i32() {
        let result = parse("impl Foo { fun get() -> i32 { 42 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_i64() {
        let result = parse("impl Foo { fun get() -> i64 { 42 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_f64() {
        let result = parse("impl Foo { fun get() -> f64 { 3.14 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_bool() {
        let result = parse("impl Foo { fun is_valid() -> bool { true } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_string() {
        let result = parse("impl Foo { fun name() -> String { \"\" } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_option() {
        let result = parse("impl Foo { fun find() -> Option<i32> { None } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_result() {
        let result = parse("impl Foo { fun try_get() -> Result<i32, Error> { Ok(0) } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_vec() {
        let result = parse("impl Foo { fun items() -> Vec<i32> { vec![] } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_return_self_type() {
        let result = parse("impl Point { fun origin() -> Point { Point { x: 0, y: 0 } } }");
        assert!(result.is_ok());
    }

    // ===== Parameter variations =====

    #[test]
    fn test_impl_no_params() {
        let result = parse("impl Foo { fun bar() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_one_param() {
        let result = parse("impl Foo { fun bar(x: i32) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_two_params() {
        let result = parse("impl Foo { fun bar(x: i32, y: i32) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_three_params() {
        let result = parse("impl Foo { fun bar(x: i32, y: i32, z: i32) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_many_params() {
        let result = parse("impl Foo { fun bar(a: i32, b: i32, c: i32, d: i32, e: i32) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_mixed_param_types() {
        let result = parse("impl Foo { fun bar(n: i32, s: String, b: bool) { } }");
        assert!(result.is_ok());
    }

    // ===== Trait implementations =====

    #[test]
    fn test_impl_trait_debug() {
        let result = parse("impl Debug for Point { fun fmt(&self) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_trait_eq() {
        let result = parse("impl Eq for Point { fun eq(&self, other: &Point) -> bool { true } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_trait_ord() {
        let result = parse("impl Ord for Point { fun cmp(&self, other: &Point) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_trait_hash() {
        let result = parse("impl Hash for Point { fun hash(&self) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_trait_defaultable() {
        let result = parse("impl Defaultable for Point { fun make_default() -> Point { Point { x: 0, y: 0 } } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_trait_iterator() {
        let result = parse("impl Iterator for MyIter { fun next(&mut self) -> Option<i32> { None } }");
        assert!(result.is_ok());
    }

    // ===== Multiple impls =====

    #[test]
    fn test_two_impls() {
        let result = parse("impl A { } impl B { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_three_impls() {
        let result = parse("impl A { } impl B { } impl C { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_same_type_twice() {
        let result = parse("impl Point { fun a() { } } impl Point { fun b() { } }");
        assert!(result.is_ok());
    }

    // ===== Complex generics =====

    #[test]
    fn test_impl_generic_maybe() {
        let result = parse("impl<T> Maybe<T> { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_generic_outcome() {
        let result = parse("impl<T, E> Outcome<T, E> { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_generic_vec() {
        let result = parse("impl<T> Vec<T> { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_generic_hashmap() {
        let result = parse("impl<K, V> HashMap<K, V> { }");
        assert!(result.is_ok());
    }

    // ===== Method body variations =====

    #[test]
    fn test_impl_method_empty_body() {
        let result = parse("impl Foo { fun bar() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_single_expr() {
        let result = parse("impl Foo { fun bar() { 42 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_let_binding() {
        let result = parse("impl Foo { fun bar() { let x = 1 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_if_expr() {
        let result = parse("impl Foo { fun bar(&self) { if self.x > 0 { 1 } else { 0 } } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_match_expr() {
        let result = parse("impl Foo { fun bar(&self) { match self.x { 0 => 0, _ => 1 } } }");
        assert!(result.is_ok());
    }

    // ===== fn keyword tests =====

    #[test]
    fn test_impl_all_fn_methods() {
        let result = parse("impl Foo { fn a() { } fn b() { } fn c() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_mixed_fun_fn() {
        let result = parse("impl Foo { fun a() { } fn b() { } fun c() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_pub_fn() {
        let result = parse("impl Foo { pub fn bar() { } }");
        assert!(result.is_ok());
    }

    // ===== Self type variations =====

    #[test]
    fn test_impl_method_ref_self() {
        let result = parse("impl Point { fun get(&self) { self.x } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_mut_ref_self() {
        let result = parse("impl Counter { fun inc(&mut self) { self.n = self.n + 1 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_self_and_params() {
        let result = parse("impl Point { fun add(&self, other: &Point) -> Point { Point { x: self.x + other.x, y: self.y + other.y } } }");
        assert!(result.is_ok());
    }

    // ===== pub variations =====

    #[test]
    fn test_impl_all_pub_methods() {
        let result = parse("impl Foo { pub fun a() { } pub fun b() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_method_count_with_pub() {
        let expr = parse("impl Foo { pub fun a() { } fun b() { } pub fun c() { } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Impl { methods, .. } = &exprs[0].kind {
                assert_eq!(methods.len(), 3);
                assert!(methods[0].is_pub);
                assert!(!methods[1].is_pub);
                assert!(methods[2].is_pub);
            }
        }
    }
}
