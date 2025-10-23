//! Visibility and modifier expression parsing
//!
//! Handles parsing of visibility markers and language modifiers:
//! - `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` - visibility
//! - `const` - compile-time evaluation modifier
//! - `sealed` - sealed class modifier (inheritance control)
//! - `final` - final class/method modifier (override prevention)
//! - `abstract` - abstract class/method modifier
//! - `unsafe` - unsafe function modifier
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Attribute, Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

// Import identifiers module for parse_module_path_segments
use super::identifiers;

/// Parse pub token with optional visibility scope
///
/// Syntax: `pub`, `pub(crate)`, `pub(super)`, `pub(in path::to::module)`
///
/// # Examples
/// ```ruchy
/// pub fn public_function() {}
/// pub(crate) fn crate_visible() {}
/// pub(super) fn parent_visible() {}
/// pub(in crate::module) fn module_visible() {}
/// ```
pub(in crate::frontend::parser) fn parse_pub_token(
    state: &mut ParserState,
    _span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume 'pub'
    skip_visibility_scope(state)?;

    let mut expr = parse_pub_target_expression(state)?;
    mark_expression_as_public(&mut expr);
    Ok(expr)
}

/// Parse visibility scope: pub(crate), pub(super), or pub(in path)
///
/// Complexity: 4 (within target ≤10)
/// Fixed in DEFECT-PARSER-016: Added support for pub(in path) syntax
fn skip_visibility_scope(state: &mut ParserState) -> Result<()> {
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        return Ok(());
    }

    state.tokens.advance(); // consume '('
    match state.tokens.peek() {
        Some((Token::Crate | Token::Super, _)) => {
            state.tokens.advance();
            state.tokens.expect(&Token::RightParen)?;
            Ok(())
        }
        Some((Token::In, _)) => {
            state.tokens.advance(); // consume 'in'
            parse_visibility_path(state)?;
            state.tokens.expect(&Token::RightParen)?;
            Ok(())
        }
        _ => bail!("Expected 'crate', 'super', or 'in' after 'pub('"),
    }
}

/// Parse path after pub(in ...)
///
/// Complexity: 5 (within target ≤10)
/// Extracted from `skip_visibility_scope` to reduce complexity
fn parse_visibility_path(state: &mut ParserState) -> Result<()> {
    // Parse path: can start with :: (absolute), crate, super, or self
    if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume leading ::
    }

    // Parse first segment
    match state.tokens.peek() {
        Some((Token::Crate, _)) => {
            state.tokens.advance();
            let _ = identifiers::parse_module_path_segments(state, "crate".to_string())?;
        }
        Some((Token::Super, _)) => {
            state.tokens.advance();
            let _ = identifiers::parse_module_path_segments(state, "super".to_string())?;
        }
        Some((Token::Self_, _)) => {
            state.tokens.advance();
            let _ = identifiers::parse_module_path_segments(state, "self".to_string())?;
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            let _ = identifiers::parse_module_path_segments(state, name)?;
        }
        _ => bail!("Expected path after 'pub(in'"),
    }
    Ok(())
}

/// Determine what expression follows pub keyword
fn parse_pub_target_expression(state: &mut ParserState) -> Result<Expr> {
    match state.tokens.peek() {
        Some((Token::Use, _)) => parse_pub_use_statement(state),
        Some((Token::Const, _)) => parse_pub_const_function(state),
        Some((Token::Unsafe, _)) => parse_pub_unsafe_function(state),
        _ => super::super::parse_prefix(state),
    }
}

/// Parse public use statement
fn parse_pub_use_statement(state: &mut ParserState) -> Result<Expr> {
    let mut expr = super::super::parse_use_statement(state)?;
    expr.attributes.push(Attribute {
        name: "pub".to_string(),
        args: vec![],
        span: expr.span,
    });
    Ok(expr)
}

/// Parse pub const function
fn parse_pub_const_function(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'const'
    if !matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
        bail!("Expected 'fun' or 'fn' after 'pub const'");
    }
    let mut expr = super::super::parse_prefix(state)?;
    if let ExprKind::Function { .. } = &expr.kind {
        expr.attributes.push(Attribute {
            name: "const".to_string(),
            args: vec![],
            span: expr.span,
        });
    }
    Ok(expr)
}

/// Parse pub unsafe function
fn parse_pub_unsafe_function(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'unsafe'
    if !matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
        bail!("Expected 'fun' or 'fn' after 'pub unsafe'");
    }
    let mut expr = super::super::parse_prefix(state)?;
    if let ExprKind::Function { .. } = &expr.kind {
        expr.attributes.push(Attribute {
            name: "unsafe".to_string(),
            args: vec![],
            span: expr.span,
        });
    }
    Ok(expr)
}

/// Mark expression as public (set `is_pub` flag)
fn mark_expression_as_public(expr: &mut Expr) {
    match &mut expr.kind {
        ExprKind::Function { is_pub, .. }
        | ExprKind::Struct { is_pub, .. }
        | ExprKind::TupleStruct { is_pub, .. }
        | ExprKind::Class { is_pub, .. }
        | ExprKind::Trait { is_pub, .. }
        | ExprKind::Impl { is_pub, .. } => *is_pub = true,
        _ => {}
    }
}

/// Parse const token - handles const declarations for functions and variables
///
/// Similar to `parse_pub_token` but for const modifier
///
/// # Examples
/// ```ruchy
/// const fn compile_time_fn() -> i32 { 42 }
/// const PI = 3.14159
/// const MAX_SIZE: i32 = 100
/// ```
pub(in crate::frontend::parser) fn parse_const_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume 'const'

    // Check what comes after 'const'
    match state.tokens.peek() {
        Some((Token::Fun | Token::Fn, _)) => {
            // const function
            let mut expr = super::super::parse_prefix(state)?;
            // Mark the function as const by adding an attribute
            if let ExprKind::Function { .. } = &expr.kind {
                expr.attributes.push(Attribute {
                    name: "const".to_string(),
                    args: vec![],
                    span: expr.span,
                });
            }
            Ok(expr)
        }
        Some((Token::Identifier(_), _)) => {
            // const variable declaration
            parse_const_variable(state, span)
        }
        _ => bail!("Expected identifier, 'fun', or 'fn' after 'const'"),
    }
}

/// Parse const variable declaration
///
/// Syntax: `const NAME [: Type] = value`
/// Similar to let but always immutable (`is_mutable = false`)
///
/// # Examples
/// ```ruchy
/// const PI = 3.14159
/// const MAX_SIZE: i32 = 100
/// ```
fn parse_const_variable(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    use crate::frontend::ast::Literal;
    use crate::frontend::parser::{parse_expr_recursive, utils};

    // Parse variable name
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        _ => bail!("Expected identifier after 'const'"),
    };

    // Parse optional type annotation: `: Type`
    let type_annotation = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume ':'
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Expect '=' token
    state.tokens.expect(&Token::Equal)?;

    // Parse value expression
    let value = Box::new(parse_expr_recursive(state)?);

    // Const variables don't have 'in' clause - body is always unit
    let body = Box::new(Expr::new(
        ExprKind::Literal(Literal::Unit),
        value.span,
    ));

    // Create Let expression with is_mutable = false
    let end_span = value.span;
    let mut expr = Expr::new(
        ExprKind::Let {
            name,
            type_annotation,
            value,
            body,
            is_mutable: false,  // const is never mutable
            else_block: None,    // const doesn't support else
        },
        start_span.merge(end_span),
    );

    // Add "const" attribute to distinguish from regular let
    expr.attributes.push(Attribute {
        name: "const".to_string(),
        args: vec![],
        span: start_span,
    });

    Ok(expr)
}

/// Parse sealed token - handles sealed modifier for classes
///
/// Sealed classes cannot be inherited from outside their module
///
/// # Examples
/// ```ruchy
/// sealed class InternalImplementation {}
/// ```
pub(in crate::frontend::parser) fn parse_sealed_token(
    state: &mut ParserState,
    _span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume 'sealed'

    // Check if next token is 'class'
    match state.tokens.peek() {
        Some((Token::Class, _)) => {
            let mut expr = super::super::parse_prefix(state)?;
            // Mark the class as sealed
            if let ExprKind::Class { is_sealed, .. } = &mut expr.kind {
                *is_sealed = true;
            }
            Ok(expr)
        }
        _ => bail!("Expected 'class' after 'sealed'"),
    }
}

/// Parse final token - used for final methods and classes, or as identifier
///
/// Final classes cannot be inherited, final methods cannot be overridden
///
/// # Examples
/// ```ruchy
/// final class CannotInherit {}
/// final fn cannot_override() {}
/// let final = 42;  // also valid as identifier
/// ```
pub(in crate::frontend::parser) fn parse_final_token(
    state: &mut ParserState,
    _span: Span,
) -> Result<Expr> {
    let start = state.tokens.current_position();
    state.tokens.advance(); // consume 'final'

    // Could be final class or final method
    match state.tokens.peek() {
        Some((Token::Class, _)) => {
            let mut expr = super::super::parse_prefix(state)?;
            // Mark the class as final (no inheritance)
            if let ExprKind::Class { .. } = &expr.kind {
                expr.attributes.push(Attribute {
                    name: "final".to_string(),
                    args: vec![],
                    span: expr.span,
                });
            }
            Ok(expr)
        }
        Some((Token::Fun | Token::Fn, _)) => {
            let mut expr = super::super::parse_prefix(state)?;
            // Mark the method as final
            if let ExprKind::Function { .. } = &expr.kind {
                expr.attributes.push(Attribute {
                    name: "final".to_string(),
                    args: vec![],
                    span: expr.span,
                });
            }
            Ok(expr)
        }
        _ => {
            // Not followed by class/fn - treat 'final' as a regular identifier
            // This allows using 'final' as a variable name (Rust keyword, needs r# in transpiler)
            Ok(Expr::new(
                ExprKind::Identifier("final".to_string()),
                Span::new(start.0, start.1 + 5), // 'final' is 5 characters
            ))
        }
    }
}

/// Parse abstract token - used for abstract classes and methods
///
/// Abstract classes cannot be instantiated, abstract methods must be implemented
///
/// # Examples
/// ```ruchy
/// abstract class MustImplement {}
/// abstract fn must_override();
/// ```
pub(in crate::frontend::parser) fn parse_abstract_token(
    state: &mut ParserState,
    _span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume 'abstract'

    // Could be abstract class or abstract method
    match state.tokens.peek() {
        Some((Token::Class, _)) => {
            let mut expr = super::super::parse_prefix(state)?;
            // Mark the class as abstract
            if let ExprKind::Class { is_abstract, .. } = &mut expr.kind {
                *is_abstract = true;
            }
            Ok(expr)
        }
        Some((Token::Fun | Token::Fn, _)) => {
            // Abstract method
            let mut expr = super::super::parse_prefix(state)?;
            if let ExprKind::Function { .. } = &expr.kind {
                expr.attributes.push(Attribute {
                    name: "abstract".to_string(),
                    args: vec![],
                    span: expr.span,
                });
            }
            Ok(expr)
        }
        _ => bail!("Expected 'class' or 'fn' after 'abstract'"),
    }
}

/// Parse unsafe token - handles unsafe declarations for functions
///
/// Unsafe functions can perform operations that bypass safety guarantees
///
/// # Examples
/// ```ruchy
/// unsafe fn direct_memory_access() {}
/// ```
pub(in crate::frontend::parser) fn parse_unsafe_token(
    state: &mut ParserState,
    _span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume 'unsafe'

    // Check if next token is 'fun' or 'fn'
    match state.tokens.peek() {
        Some((Token::Fun | Token::Fn, _)) => {
            let mut expr = super::super::parse_prefix(state)?;
            // Mark the function as unsafe by adding an attribute
            if let ExprKind::Function { .. } = &expr.kind {
                expr.attributes.push(Attribute {
                    name: "unsafe".to_string(),
                    args: vec![],
                    span: expr.span,
                });
            }
            Ok(expr)
        }
        _ => bail!("Expected 'fun' or 'fn' after 'unsafe'"),
    }
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    // ===== UNIT TESTS (from original expressions.rs) =====

    #[test]
    fn test_pub_module() {
        let code = "pub mod test_module { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Public module should parse successfully");
    }

    #[test]
    fn test_pub_const_fn() {
        let code = "pub const fn test() { }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Public const function should parse successfully"
        );
    }

    #[test]
    fn test_pub_unsafe_fn() {
        let code = "pub unsafe fn test() { }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Public unsafe function should parse successfully"
        );
    }

    #[test]
    fn test_mark_public() {
        let code = "pub fn test() { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Public function should parse successfully");
    }

    #[test]
    fn test_pub_crate() {
        let code = "pub(crate) fn test() { }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "pub(crate) function should parse successfully"
        );
    }

    #[test]
    fn test_pub_super() {
        let code = "pub(super) fn test() { }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "pub(super) function should parse successfully"
        );
    }

    #[test]
    fn test_final_as_identifier() {
        let code = "let final = 42";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "'final' should be usable as identifier: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_sealed_class() {
        let code = "sealed class Test {}";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Sealed class should parse successfully");
    }

    #[test]
    fn test_abstract_class() {
        let code = "abstract class Test {}";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Abstract class should parse successfully");
    }

    #[test]
    fn test_final_class() {
        let code = "final class Test {}";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Final class should parse successfully");
    }

    // ===== PROPERTY TESTS (EXTREME TDD) =====

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Property: All valid visibility modifiers should parse without panic
        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_visibility_modifiers_never_panic(modifier in prop::sample::select(vec![
                "pub", "pub(crate)", "pub(super)", "const", "sealed",
                "final", "abstract", "unsafe"
            ])) {
                let code = format!("{} fn test() {{}}", modifier);
                let _ = Parser::new(&code).parse(); // Should not panic
            }
        }

        // Property: pub(crate) and pub(super) are semantically equivalent for parsing
        proptest! {
            #[test]
            #[ignore]
            fn prop_pub_crate_super_equivalent_parsing(
                fn_name in "[a-z][a-z0-9_]{0,10}"
            ) {
                let code_crate = format!("pub(crate) fn {}() {{}}", fn_name);
                let code_super = format!("pub(super) fn {}() {{}}", fn_name);

                let result_crate = Parser::new(&code_crate).parse();
                let result_super = Parser::new(&code_super).parse();

                // Both should succeed or both should fail
                prop_assert_eq!(result_crate.is_ok(), result_super.is_ok());
            }
        }

        // Property: final can be used as identifier or modifier
        proptest! {
            #[test]
            #[ignore]
            fn prop_final_dual_usage(use_as_modifier in prop::bool::ANY) {
                let code = if use_as_modifier {
                    "final class Test {}"
                } else {
                    "let final = 42"
                };

                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "final should work in both contexts");
            }
        }

        // Property: Combining pub with other modifiers should work
        proptest! {
            #[test]
            #[ignore]
            fn prop_pub_combinations(
                modifier in prop::sample::select(vec!["const", "unsafe"])
            ) {
                let code = format!("pub {} fn test() {{}}", modifier);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "pub + {} should parse", modifier);
            }
        }

        // Property: Invalid modifier combinations should fail gracefully
        proptest! {
            #[test]
            #[ignore]
            fn prop_invalid_modifiers_fail_gracefully(
                invalid in prop::sample::select(vec![
                    "sealed fn", "abstract struct", "final trait"
                ])
            ) {
                let code = format!("{} Test {{}}", invalid);
                let result = Parser::new(&code).parse();
                // Should either fail or succeed, but never panic
                let _ = result;
            }
        }

        // Property: Class modifiers only work with classes
        proptest! {
            #[test]
            #[ignore]
            fn prop_class_modifiers_require_class(
                modifier in prop::sample::select(vec!["sealed", "abstract", "final"])
            ) {
                // Valid: modifier + class
                let valid_code = format!("{} class Test {{}}", modifier);
                let valid_result = Parser::new(&valid_code).parse();

                // Invalid: modifier + fn (except final which is dual-purpose)
                let invalid_code = format!("{} fn test() {{}}", modifier);
                let invalid_result = Parser::new(&invalid_code).parse();

                if modifier == "final" {
                    // final works with both
                    prop_assert!(valid_result.is_ok() && invalid_result.is_ok());
                } else {
                    // sealed/abstract require class
                    prop_assert!(valid_result.is_ok());
                    prop_assert!(invalid_result.is_err(),
                        "{} should not work with fn", modifier);
                }
            }
        }
    }
}
