//! Module declaration parsing
//!
//! Handles parsing of module declarations and organization:
//! - Module declarations: `mod name { ... }`
//! - Module bodies with visibility modifiers
//! - Module items (functions, use statements, nested modules)
//!
//! # Examples
//! ```ruchy
//! // Basic module
//! mod utils {
//!     fun helper() { 42 }
//! }
//!
//! // Module with public items
//! mod api {
//!     pub fun handler() { "response" }
//!     pub mod v2 {
//!         pub fun new_handler() { "v2 response" }
//!     }
//! }
//!
//! // Module with use statements
//! mod tools {
//!     pub use std::collections::HashMap
//!     fun process() { HashMap::new() }
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

/// Parse module declaration: mod name { body }
///
/// Syntax: `mod name { body }` or `module name { body }`
pub(in crate::frontend::parser) fn parse_module_declaration(state: &mut ParserState) -> Result<Expr> {
    // Accept both 'mod' and 'module' keywords
    let start_span = if matches!(state.tokens.peek(), Some((Token::Mod, _))) {
        state.tokens.expect(&Token::Mod)?
    } else {
        state.tokens.expect(&Token::Module)?
    };
    // Parse module name (accept keywords as module names)
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let n = n.clone();
            state.tokens.advance();
            n
        }
        // DEFECT-PARSER-015 FIX: Allow keyword module names (mod private, mod utils, etc.)
        Some((Token::Private, _)) => {
            state.tokens.advance();
            "private".to_string()
        }
        _ => bail!("Expected module name after 'mod' or 'module'"),
    };
    // Parse module body with visibility support
    state.tokens.expect(&Token::LeftBrace)?;
    let body = Box::new(parse_module_body(state)?);
    Ok(Expr::new(ExprKind::Module { name, body }, start_span))
}

/// Parse module body with support for visibility modifiers (pub)
fn parse_module_body(state: &mut ParserState) -> Result<Expr> {
    let start_span = state
        .tokens
        .peek()
        .map_or(Span { start: 0, end: 0 }, |t| t.1);
    let mut exprs = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let is_pub = parse_visibility_modifier(state);
        exprs.push(parse_module_item(state, is_pub)?);
        skip_optional_semicolon(state);
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Block(exprs), start_span))
}

/// Parse visibility modifier (pub)
fn parse_visibility_modifier(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse module item (function, use statement, nested module, or expression)
fn parse_module_item(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    match state.tokens.peek() {
        // DEFECT-PARSER-015 FIX: Accept both 'fun' and 'fn' for functions
        Some((Token::Fun, _)) | Some((Token::Fn, _)) => {
            crate::frontend::parser::functions::parse_function_with_visibility(state, is_pub)
        }
        Some((Token::Use, _)) if is_pub => {
            state.tokens.advance();
            crate::frontend::parser::parse_use_statement_with_visibility(state, true)
        }
        // DEFECT-PARSER-015 FIX: Allow pub mod
        Some((Token::Mod, _)) | Some((Token::Module, _)) if is_pub => {
            parse_module_declaration(state)
        }
        _ if is_pub => {
            bail!("'pub' can only be used with function declarations, use statements, or module declarations")
        }
        _ => crate::frontend::parser::parse_expr_recursive(state),
    }
}

/// Skip optional semicolon
fn skip_optional_semicolon(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
        state.tokens.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_basic_module() {
        let code = "mod utils { fun helper() { 42 } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Basic module should parse");
    }

    #[test]
    fn test_module_with_pub_function() {
        let code = "mod api { pub fun handler() { \"response\" } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Module with pub function should parse");
    }

    #[test]
    fn test_nested_modules() {
        let code = "mod outer { mod inner { fun nested() { 1 } } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested modules should parse");
    }

    #[test]
    fn test_module_keyword() {
        let code = "module utils { fun helper() { 42 } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Module keyword should parse");
    }

    #[test]
    fn test_pub_module() {
        let code = "mod outer { pub mod inner { fun test() { 1 } } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Public nested module should parse");
    }

    #[test]
    fn test_module_with_use() {
        let code = "mod tools { pub use std::collections::HashMap }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Module with use statement should parse");
    }

    #[test]
    fn test_empty_module() {
        let code = "mod empty { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty module should parse");
    }

    #[test]
    fn test_keyword_module_name() {
        let code = "mod private { fun test() { 1 } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Keyword as module name should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_basic_modules_parse(name in "[a-z][a-z0-9_]*") {
                let code = format!("mod {} {{ 42 }}", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_pub_functions_parse(mod_name in "[a-z]+", fn_name in "[a-z]+") {
                let code = format!("mod {} {{ pub fun {}() {{ 1 }} }}", mod_name, fn_name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_nested_modules_parse(outer in "[a-z]+", inner in "[a-z]+") {
                let code = format!("mod {} {{ mod {} {{ 42 }} }}", outer, inner);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_empty_modules_parse(name in "[a-z]+") {
                let code = format!("mod {} {{}}", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_module_keyword_parses(name in "[a-z]+") {
                let code = format!("module {} {{ 42 }}", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_pub_nested_modules_parse(outer in "[a-z]+", inner in "[a-z]+") {
                let code = format!("mod {} {{ pub mod {} {{ 1 }} }}", outer, inner);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_module_with_multiple_items(name in "[a-z]+", n in 1usize..5) {
                let mut code = format!("mod {} {{", name);
                for i in 0..n {
                    code.push_str(&format!(" fun f{}() {{ {} }}", i, i));
                }
                code.push_str(" }");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
