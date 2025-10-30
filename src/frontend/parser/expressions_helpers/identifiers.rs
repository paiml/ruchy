//! Identifier and path resolution parsing
//!
//! Handles parsing of:
//! - Identifiers (variable names, function names, etc.)
//! - Qualified paths (`std::string::String`, `math::add`)
//! - Module path segments (`crate::module::submodule`)
//! - Turbofish generic arguments (`Vec::`<i32>, `HashMap::`<String, i32>)
//! - Special identifiers (self, super, _, default)
//! - Fat arrow lambdas (x => x + 1)
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

/// Parse identifier tokens into expressions
///
/// Handles regular identifiers, qualified paths, and fat arrow lambdas.
///
/// # Examples
/// ```ruchy
/// foo             // Simple identifier
/// math::add       // Qualified path
/// std::Vec::<i32> // Path with turbofish generics
/// x => x + 1      // Fat arrow lambda
/// _               // Underscore identifier
/// self            // Self identifier
/// super           // Super identifier
/// ```
pub(in crate::frontend::parser) fn parse_identifier_token(
    state: &mut ParserState,
    token: &Token,
    span: Span,
) -> Result<Expr> {
    match token {
        Token::Identifier(name) => {
            state.tokens.advance();
            // Check for fat arrow lambda: x => x * 2
            // PARSER-071: Don't treat `=>` as lambda in match guard context
            if !state.in_guard_context && matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                let ident_expr = Expr::new(ExprKind::Identifier(name.clone()), span);
                super::super::parse_lambda_from_expr(state, ident_expr, span)
            } else {
                // Don't consume ! here - let postfix handle macro calls
                // Don't consume :: here - let postfix handle field access (enum variants, module paths)
                Ok(Expr::new(ExprKind::Identifier(name.clone()), span))
            }
        }
        Token::Underscore => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("_".to_string()), span))
        }
        Token::Self_ => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("self".to_string()), span))
        }
        Token::Super => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("super".to_string()), span))
        }
        _ => bail!("Expected identifier token, got: {token:?}"),
    }
}

/// Parse module path segments separated by :: (complexity: 3)
///
/// # Examples
/// ```ruchy
/// std::string::String
/// crate::module::function
/// super::sibling::Type
/// ```
pub(in crate::frontend::parser) fn parse_module_path_segments(
    state: &mut ParserState,
    initial: String,
) -> Result<String> {
    let mut path = vec![initial];
    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        path.push(parse_path_segment(state)?);
    }
    Ok(path.join("::"))
}

/// Parse a single path segment after ::
///
/// Accepts identifiers, keywords (as module names), wildcards, and turbofish generics.
/// DEFECT-PARSER-016 FIX: Accept any keyword as path segment (keywords can be module names).
///
/// # Examples
/// ```ruchy
/// foo          // Identifier
/// *            // Wildcard (for use statements)
/// ::<i32>      // Turbofish generics
/// if           // Keyword as module name (pub(in crate::if))
/// ```
pub(in crate::frontend::parser) fn parse_path_segment(state: &mut ParserState) -> Result<String> {
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        // Parse turbofish generic arguments
        parse_turbofish_generics(state)
    } else if let Some((Token::Identifier(segment), _)) = state.tokens.peek() {
        let segment = segment.clone();
        state.tokens.advance();
        Ok(segment)
    } else if matches!(state.tokens.peek(), Some((Token::Star, _))) {
        // Handle wildcard in qualified names (for use statements)
        state.tokens.advance();
        Ok("*".to_string())
    } else if let Some((token, _)) = state.tokens.peek() {
        // Accept any keyword as a path segment (keywords can be module names)
        // This handles: as, for, if, match, etc. in paths like pub(in crate::as::match)
        let name = token_to_keyword_string(token);
        if name.is_empty() {
            bail!("Expected identifier or '*' after '::'")
        }
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected identifier or '*' after '::'")
    }
}

/// Parse turbofish generic arguments: ::<i32> or ::<String, i32>
///
/// Returns a string representation of the turbofish for path construction.
///
/// # Examples
/// ```ruchy
/// Vec::<i32>
/// HashMap::<String, i32>
/// Option::<Vec::<i32>>  // Nested generics
/// ```
pub(in crate::frontend::parser) fn parse_turbofish_generics(state: &mut ParserState) -> Result<String> {
    // Consume the < token
    state.tokens.advance();

    let mut type_args = Vec::new();

    // Parse comma-separated type list
    loop {
        // Parse single type argument
        let type_str = parse_turbofish_type(state)?;
        type_args.push(type_str);

        // Check for comma (more types) or > (end of list)
        match state.tokens.peek() {
            Some((Token::Comma, _)) => {
                state.tokens.advance(); // consume comma
            }
            Some((Token::Greater, _)) => {
                state.tokens.advance(); // consume >
                break;
            }
            Some((Token::GreaterEqual, _)) => {
                // Handle >> as two > tokens (for nested generics like Vec<Vec<i32>>)
                state.tokens.advance();
                break;
            }
            _ => bail!("Expected ',' or '>' in turbofish generics"),
        }
    }

    // Build string representation
    Ok(format!("<{}>", type_args.join(", ")))
}

/// Parse a single type in turbofish context
///
/// Returns a string representation of the type.
///
/// # Examples
/// ```ruchy
/// i32
/// String
/// std::Vec
/// Vec<i32>  // Nested generics
/// ```
fn parse_turbofish_type(state: &mut ParserState) -> Result<String> {
    let mut type_str = String::new();

    // Parse type name (could be qualified path like std::string::String)
    loop {
        match state.tokens.peek() {
            Some((Token::Identifier(name), _)) => {
                type_str.push_str(name);
                state.tokens.advance();

                // Check for :: (qualified path)
                if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                    type_str.push_str("::");
                    state.tokens.advance();
                    continue;
                }
                break;
            }
            Some((Token::Integer(n), _)) => {
                // For array sizes like [i32; 10]
                type_str.push_str(&n.clone());
                state.tokens.advance();
                break;
            }
            _ => break,
        }
    }

    // Check for nested generics: Vec<i32>
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        let nested = parse_turbofish_generics(state)?;
        type_str.push_str(&nested);
    }

    Ok(type_str)
}

/// Convert token to lowercase keyword string if it's a keyword, empty string otherwise
///
/// Used for accepting keywords as path segments in qualified names.
/// This enables paths like `pub(in crate::if::match)` where keywords are module names.
pub(in crate::frontend::parser) fn token_to_keyword_string(token: &Token) -> String {
    match token {
        Token::As => "as".to_string(),
        Token::Async => "async".to_string(),
        Token::Await => "await".to_string(),
        Token::Break => "break".to_string(),
        Token::Const => "const".to_string(),
        Token::Continue => "continue".to_string(),
        Token::Crate => "crate".to_string(),
        Token::Default => "default".to_string(),
        Token::Else => "else".to_string(),
        Token::Enum => "enum".to_string(),
        Token::Err => "Err".to_string(),
        Token::Fn => "fn".to_string(),
        Token::For => "for".to_string(),
        Token::From => "from".to_string(),
        Token::Fun => "fun".to_string(),
        Token::If => "if".to_string(),
        Token::Impl => "impl".to_string(),
        Token::In => "in".to_string(),
        Token::Let => "let".to_string(),
        Token::Loop => "loop".to_string(),
        Token::Match => "match".to_string(),
        Token::Mod => "mod".to_string(),
        Token::Module => "module".to_string(),
        Token::Mut => "mut".to_string(),
        Token::None => "None".to_string(),
        Token::Ok => "Ok".to_string(),
        Token::Private => "private".to_string(),
        Token::Pub => "pub".to_string(),
        Token::Return => "return".to_string(),
        Token::Self_ => "self".to_string(),
        Token::Some => "Some".to_string(),
        Token::Static => "static".to_string(),
        Token::Struct => "struct".to_string(),
        Token::Super => "super".to_string(),
        Token::Trait => "trait".to_string(),
        Token::Type => "type".to_string(),
        Token::Unsafe => "unsafe".to_string(),
        Token::Use => "use".to_string(),
        Token::Where => "where".to_string(),
        Token::While => "while".to_string(),
        // Standard library types that are keywords
        Token::Option => "Option".to_string(),
        Token::Result => "Result".to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_simple_identifier() {
        let code = "foo";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple identifier should parse");
    }

    #[test]
    fn test_qualified_path() {
        let code = "math::add";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Qualified path should parse");
    }

    #[test]
    fn test_nested_qualified_path() {
        let code = "std::collections::HashMap";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested qualified path should parse");
    }

    #[test]
    fn test_underscore_identifier() {
        let code = "_";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Underscore identifier should parse");
    }

    #[test]
    fn test_self_identifier() {
        let code = "self";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Self identifier should parse");
    }

    #[test]
    fn test_super_identifier() {
        let code = "super";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Super identifier should parse");
    }

    #[test]
    fn test_fat_arrow_lambda() {
        let code = "x => x + 1";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Fat arrow lambda should parse");
    }

    // Property tests for identifiers
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Generate valid identifier strings (alphanumeric + underscore, not starting with digit)
        fn valid_identifier() -> impl Strategy<Value = String> {
            prop::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
        }

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_valid_identifiers_always_parse(ident in valid_identifier()) {
                let code = ident.clone();
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Valid identifier {} should parse", ident);
            }

            #[test]
            #[ignore]
            fn prop_qualified_paths_parse(
                mod1 in valid_identifier(),
                mod2 in valid_identifier()
            ) {
                let code = format!("{mod1}::{mod2}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Qualified path {}::{} should parse", mod1, mod2);
            }

            #[test]
            #[ignore]
            fn prop_triple_qualified_paths_parse(
                mod1 in valid_identifier(),
                mod2 in valid_identifier(),
                mod3 in valid_identifier()
            ) {
                let code = format!("{mod1}::{mod2}::{mod3}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Triple path {}::{}::{} should parse", mod1, mod2, mod3);
            }

            #[test]
            #[ignore]
            fn prop_keywords_as_path_segments(
                keyword in prop::sample::select(vec![
                    "as", "for", "if", "match", "while", "let", "fn", "mod"
                ])
            ) {
                let code = format!("crate::{keyword}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Keyword {} as path segment should parse", keyword);
            }

            #[test]
            #[ignore]
            fn prop_special_identifiers_always_parse(
                special in prop::sample::select(vec!["_", "self", "super", "default"])
            ) {
                let code = special.to_string();
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Special identifier {} should parse", special);
            }

            #[test]
            #[ignore]
            fn prop_fat_arrow_lambdas_parse(param in valid_identifier()) {
                let code = format!("{param} => {param} + 1");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Fat arrow lambda with {} should parse", param);
            }
        }
    }
}
