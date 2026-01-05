//! Utility Built-in Function Transpilation
//!
//! This module handles transpilation of utility built-in functions:
//! - Time: timestamp, get_time_ms, now_millis
//! - Assertions: assert, assert_eq, assert_ne
//! - Collections: HashMap, HashSet constructors
//! - Range: range(end) and range(start, end)
//!
//! **EXTREME TDD Round 60**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Handle time functions (timestamp, `get_time_ms`, `now_millis`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ruchy::backend::transpiler::Transpiler;
    /// let transpiler = Transpiler::new();
    /// // timestamp() -> milliseconds since Unix epoch
    /// // get_time_ms() -> milliseconds since Unix epoch
    /// // now_millis() -> milliseconds since Unix epoch
    /// ```
    /// Complexity: 3 (within Toyota Way limits)
    pub fn try_transpile_time_functions(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "timestamp" | "get_time_ms" | "now_millis" => {
                if !args.is_empty() {
                    bail!("{base_name}() expects no arguments");
                }
                // Get current time in milliseconds since Unix epoch
                Ok(Some(quote! {
                    {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .expect("System time before Unix epoch")
                            .as_millis() as i64
                    }
                }))
            }
            _ => Ok(None),
        }
    }

    /// Handle assert functions (assert, `assert_eq`, `assert_ne`)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("assert(true)");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("assert !"));
    /// ```
    /// Complexity: 8 (within Toyota Way limits)
    pub fn try_transpile_assert_function(
        &self,
        _func_tokens: &TokenStream,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "assert" => {
                if args.is_empty() || args.len() > 2 {
                    bail!("assert expects 1 or 2 arguments (condition, optional message)");
                }
                let condition = self.transpile_expr(&args[0])?;
                if args.len() == 1 {
                    Ok(Some(quote! { assert!(#condition) }))
                } else {
                    let message = self.transpile_expr(&args[1])?;
                    Ok(Some(quote! { assert!(#condition, "{}", #message) }))
                }
            }
            "assert_eq" => {
                if args.len() < 2 || args.len() > 3 {
                    bail!("assert_eq expects 2 or 3 arguments (left, right, optional message)");
                }
                let left = self.transpile_expr(&args[0])?;
                let right = self.transpile_expr(&args[1])?;
                if args.len() == 2 {
                    Ok(Some(quote! { assert_eq!(#left, #right) }))
                } else {
                    let message = self.transpile_expr(&args[2])?;
                    Ok(Some(quote! { assert_eq!(#left, #right, "{}", #message) }))
                }
            }
            "assert_ne" => {
                if args.len() < 2 || args.len() > 3 {
                    bail!("assert_ne expects 2 or 3 arguments (left, right, optional message)");
                }
                let left = self.transpile_expr(&args[0])?;
                let right = self.transpile_expr(&args[1])?;
                if args.len() == 2 {
                    Ok(Some(quote! { assert_ne!(#left, #right) }))
                } else {
                    let message = self.transpile_expr(&args[2])?;
                    Ok(Some(quote! { assert_ne!(#left, #right, "{}", #message) }))
                }
            }
            _ => Ok(None),
        }
    }

    /// Handle collection constructors (`HashMap`, `HashSet`)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("HashMap()");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("HashMap"));
    /// ```
    /// Complexity: 2 (within Toyota Way limits)
    pub fn try_transpile_collection_constructor(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match (base_name, args.len()) {
            ("HashMap", 0) => Ok(Some(quote! { std::collections::HashMap::new() })),
            ("HashSet", 0) => Ok(Some(quote! { std::collections::HashSet::new() })),
            _ => Ok(None),
        }
    }

    /// Handle `range()` function - transpile to Rust range syntax
    ///
    /// Converts:
    /// - `range(end)` to `(0..end)`
    /// - `range(start, end)` to `(start..end)`
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"range(0, 10)"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("(0 .. 10)"));
    /// ```
    /// Complexity: 4 (within Toyota Way limits)
    pub fn try_transpile_range_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        if base_name != "range" {
            return Ok(None);
        }

        match args.len() {
            // range(end) -> (0..end)
            1 => {
                let end = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (0 .. #end) }))
            }
            // range(start, end) -> (start..end)
            2 => {
                let start = self.transpile_expr(&args[0])?;
                let end = self.transpile_expr(&args[1])?;
                Ok(Some(quote! { (#start .. #end) }))
            }
            _ => Ok(None),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn bool_expr(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // try_transpile_time_functions tests
    // ========================================================================

    #[test]
    fn test_try_transpile_time_functions_timestamp() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_time_functions("timestamp", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("SystemTime"));
        assert!(tokens_str.contains("UNIX_EPOCH"));
        assert!(tokens_str.contains("as_millis"));
    }

    #[test]
    fn test_try_transpile_time_functions_get_time_ms() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_time_functions("get_time_ms", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("SystemTime"));
    }

    #[test]
    fn test_try_transpile_time_functions_now_millis() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_time_functions("now_millis", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("i64"));
    }

    #[test]
    fn test_try_transpile_time_functions_with_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(123)];
        let result = transpiler.try_transpile_time_functions("timestamp", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no arguments"));
    }

    #[test]
    fn test_try_transpile_time_functions_unknown() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_time_functions("unknown_time", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // try_transpile_assert_function tests
    // ========================================================================

    #[test]
    fn test_try_transpile_assert_function_simple() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert };
        let args = vec![bool_expr(true)];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("assert"));
    }

    #[test]
    fn test_try_transpile_assert_function_with_message() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert };
        let args = vec![bool_expr(true), string_expr("test message")];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("assert"));
        assert!(tokens_str.contains("test message"));
    }

    #[test]
    fn test_try_transpile_assert_function_no_args() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert };
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("1 or 2 arguments"));
    }

    #[test]
    fn test_try_transpile_assert_function_too_many_args() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert };
        let args = vec![bool_expr(true), string_expr("msg"), int_expr(42)];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_transpile_assert_eq_simple() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_eq };
        let args = vec![int_expr(1), int_expr(1)];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_eq", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("assert_eq"));
    }

    #[test]
    fn test_try_transpile_assert_eq_with_message() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_eq };
        let args = vec![int_expr(1), int_expr(2), string_expr("should be equal")];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_eq", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("assert_eq"));
        assert!(tokens_str.contains("should be equal"));
    }

    #[test]
    fn test_try_transpile_assert_eq_wrong_args() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_eq };
        let args = vec![int_expr(1)];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_eq", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("2 or 3 arguments"));
    }

    #[test]
    fn test_try_transpile_assert_ne_simple() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_ne };
        let args = vec![int_expr(1), int_expr(2)];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_ne", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("assert_ne"));
    }

    #[test]
    fn test_try_transpile_assert_ne_with_message() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_ne };
        let args = vec![int_expr(1), int_expr(2), string_expr("should differ")];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_ne", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("assert_ne"));
        assert!(tokens_str.contains("should differ"));
    }

    #[test]
    fn test_try_transpile_assert_ne_wrong_args() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_ne };
        let args = vec![int_expr(1)];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_ne", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_transpile_assert_function_unknown() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_unknown };
        let args = vec![bool_expr(true)];
        let result =
            transpiler.try_transpile_assert_function(&func_tokens, "assert_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // try_transpile_collection_constructor tests
    // ========================================================================

    #[test]
    fn test_try_transpile_collection_constructor_hashmap() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_collection_constructor("HashMap", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("HashMap"));
        assert!(tokens_str.contains("new"));
    }

    #[test]
    fn test_try_transpile_collection_constructor_hashset() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_collection_constructor("HashSet", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("HashSet"));
        assert!(tokens_str.contains("new"));
    }

    #[test]
    fn test_try_transpile_collection_constructor_hashmap_with_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(10)];
        let result = transpiler.try_transpile_collection_constructor("HashMap", &args);
        assert!(result.is_ok());
        // With args, returns None (not handled)
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_collection_constructor_hashset_with_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(10)];
        let result = transpiler.try_transpile_collection_constructor("HashSet", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_collection_constructor_unknown() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_collection_constructor("Vec", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // try_transpile_range_function tests
    // ========================================================================

    #[test]
    fn test_try_transpile_range_function_single_arg() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(10)];
        let result = transpiler.try_transpile_range_function("range", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("0"));
        assert!(tokens_str.contains("10"));
        assert!(tokens_str.contains(".."));
    }

    #[test]
    fn test_try_transpile_range_function_two_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(5), int_expr(15)];
        let result = transpiler.try_transpile_range_function("range", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("5"));
        assert!(tokens_str.contains("15"));
        assert!(tokens_str.contains(".."));
    }

    #[test]
    fn test_try_transpile_range_function_no_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_range_function("range", &args);
        assert!(result.is_ok());
        // range() with no args returns None (not handled)
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_range_function_three_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(0), int_expr(10), int_expr(2)];
        let result = transpiler.try_transpile_range_function("range", &args);
        assert!(result.is_ok());
        // range(start, end, step) not handled, returns None
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_range_function_not_range() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(10)];
        let result = transpiler.try_transpile_range_function("other", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_range_function_with_identifier() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("n")];
        let result = transpiler.try_transpile_range_function("range", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("n"));
    }

    #[test]
    fn test_try_transpile_range_function_with_two_identifiers() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("start"), ident_expr("end")];
        let result = transpiler.try_transpile_range_function("range", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("start"));
        assert!(tokens_str.contains("end"));
    }

    // ========================================================================
    // Integration tests with expressions
    // ========================================================================

    #[test]
    fn test_assert_with_identifier_condition() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert };
        let args = vec![ident_expr("valid")];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("valid"));
    }

    #[test]
    fn test_assert_eq_with_identifiers() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { assert_eq };
        let args = vec![ident_expr("actual"), ident_expr("expected")];
        let result = transpiler.try_transpile_assert_function(&func_tokens, "assert_eq", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("actual"));
        assert!(tokens_str.contains("expected"));
    }
}
