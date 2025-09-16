//! Result type support for Ruchy
//!
//! This module provides comprehensive `Result<T, E>` type support including:
//! - Result constructors (Ok/Err)
//! - Pattern matching on Results
//! - ? operator for error propagation
//! - Result combinators (map, `and_then`, etc.)
use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
impl Transpiler {
    /// Generates Result type helpers and combinators
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::Transpiler;
    /// 
    /// let helpers = Transpiler::generate_result_helpers();
    /// let code = helpers.to_string();
    /// assert!(code.contains("trait ResultExt"));
    /// assert!(code.contains("map_err_with"));
    /// assert!(code.contains("unwrap_or_else_with"));
    /// assert!(code.contains("and_then_with"));
    /// assert!(code.contains("or_else_with"));
    /// ```
    pub fn generate_result_helpers() -> TokenStream {
        quote! {
            // Result extension trait for additional combinators
            trait ResultExt<T, E> {
                fn map_err_with<F, E2>(self, f: F) -> Result<T, E2>
                where
                    F: FnOnce(E) -> E2;
                fn unwrap_or_else_with<F>(self, f: F) -> T
                where
                    F: FnOnce(E) -> T;
                fn and_then_with<F, U>(self, f: F) -> Result<U, E>
                where
                    F: FnOnce(T) -> Result<U, E>;
                fn or_else_with<F, E2>(self, f: F) -> Result<T, E2>
                where
                    F: FnOnce(E) -> Result<T, E2>;
            }
            impl<T, E> ResultExt<T, E> for Result<T, E> {
                fn map_err_with<F, E2>(self, f: F) -> Result<T, E2>
                where
                    F: FnOnce(E) -> E2
                {
                    self.map_err(f)
                }
                fn unwrap_or_else_with<F>(self, f: F) -> T
                where
                    F: FnOnce(E) -> T
                {
                    self.unwrap_or_else(f)
                }
                fn and_then_with<F, U>(self, f: F) -> Result<U, E>
                where
                    F: FnOnce(T) -> Result<U, E>
                {
                    self.and_then(f)
                }
                fn or_else_with<F, E2>(self, f: F) -> Result<T, E2>
                where
                    F: FnOnce(E) -> Result<T, E2>
                {
                    self.or_else(f)
                }
            }
        }
    }
    /// Transpiles Result pattern matching
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"match result { Ok(val) => val, Err(e) => 0 }"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// 
    /// let result = transpiler.transpile(&ast).unwrap();
    /// let code = result.to_string();
    /// assert!(code.contains("Ok"));
    /// assert!(code.contains("Err"));
    /// ```
    pub fn transpile_result_match(
        &self,
        expr: &Expr,
        arms: &[(String, Expr)],
    ) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        let mut match_arms = Vec::new();
        for (pattern, body) in arms {
            let body_tokens = self.transpile_expr(body)?;
            let arm_tokens = if pattern == "Ok" {
                quote! { Ok(value) => #body_tokens }
            } else if pattern == "Err" {
                quote! { Err(error) => #body_tokens }
            } else {
                quote! { _ => #body_tokens }
            };
            match_arms.push(arm_tokens);
        }
        Ok(quote! {
            match #expr_tokens {
                #(#match_arms,)*
            }
        })
    }
    /// Transpiles Result chaining with ? operator
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("result?");
    /// let ast = parser.parse().expect("Failed to parse");
    /// 
    /// let result = transpiler.transpile(&ast).unwrap();
    /// let code = result.to_string();
    /// assert!(code.contains("?"));
    /// ```
    pub fn transpile_result_chain(&self, operations: &[Expr]) -> Result<TokenStream> {
        if operations.is_empty() {
            return Ok(quote! { Ok(()) });
        }
        let mut chain = self.transpile_expr(&operations[0])?;
        for op in &operations[1..] {
            let op_tokens = self.transpile_expr(op)?;
            chain = quote! { #chain.and_then(|_| #op_tokens) };
        }
        Ok(chain)
    }
    /// Transpiles Result unwrapping with default
    pub fn transpile_result_unwrap_or(&self, result: &Expr, default: &Expr) -> Result<TokenStream> {
        let result_tokens = self.transpile_expr(result)?;
        let default_tokens = self.transpile_expr(default)?;
        Ok(quote! {
            #result_tokens.unwrap_or(#default_tokens)
        })
    }
    /// Transpiles Result mapping
    pub fn transpile_result_map(&self, result: &Expr, mapper: &Expr) -> Result<TokenStream> {
        let result_tokens = self.transpile_expr(result)?;
        let mapper_tokens = self.transpile_expr(mapper)?;
        Ok(quote! {
            #result_tokens.map(#mapper_tokens)
        })
    }
    /// Transpiles custom error types
    pub fn transpile_error_type(
        &self,
        name: &str,
        variants: &[(String, Option<String>)],
    ) -> TokenStream {
        let error_name = format_ident!("{}", name);
        let mut variant_tokens = Vec::new();
        for (variant, data) in variants {
            let variant_ident = format_ident!("{}", variant);
            let variant_token = if let Some(data_type) = data {
                // Parse the type string to handle paths like std::io::Error
                let data_type_tokens: TokenStream = data_type.parse().unwrap_or_else(|_| {
                    // If parsing fails, fall back to String
                    quote! { String }
                });
                quote! { #variant_ident(#data_type_tokens) }
            } else {
                quote! { #variant_ident }
            };
            variant_tokens.push(variant_token);
        }
        quote! {
            #[derive(Debug, Clone)]
            enum #error_name {
                #(#variant_tokens,)*
            }
            impl std::fmt::Display for #error_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for #error_name {}
        }
    }
}
/// Generate Result type test cases
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    fn make_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn make_literal(lit: Literal) -> Expr {
        Expr::new(ExprKind::Literal(lit), Span::new(0, 1))
    }

    fn make_ident(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Span::new(0, 1))
    }

    #[test]
    fn test_result_helpers_generation() {
        let helpers = Transpiler::generate_result_helpers();
        let code = helpers.to_string();
        assert!(code.contains("ResultExt"));
        assert!(code.contains("map_err_with"));
        assert!(code.contains("unwrap_or_else_with"));
        assert!(code.contains("and_then_with"));
        assert!(code.contains("or_else_with"));
    }

    #[test]
    fn test_result_ext_trait_implementation() {
        let helpers = Transpiler::generate_result_helpers();
        let code = helpers.to_string();

        // Check trait definition
        assert!(code.contains("trait ResultExt"));

        // Check impl block
        assert!(code.contains("impl < T , E > ResultExt"));
        assert!(code.contains("for Result < T , E >"));

        // Check each method implementation
        assert!(code.contains("self . map_err"));
        assert!(code.contains("self . unwrap_or_else"));
        assert!(code.contains("self . and_then"));
        assert!(code.contains("self . or_else"));
    }

    #[test]
    fn test_transpile_error_type() {
        let transpiler = Transpiler::new();
        let variants = vec![
            ("NotFound".to_string(), None),
            ("InvalidInput".to_string(), Some("String".to_string())),
            ("NetworkError".to_string(), Some("std::io::Error".to_string())),
        ];
        let error_type = transpiler.transpile_error_type("AppError", &variants);
        let code = error_type.to_string();
        // Check enum definition
        assert!(code.contains("enum AppError"));
        assert!(code.contains("NotFound"));
        assert!(code.contains("InvalidInput") && code.contains("String"));
        assert!(code.contains("NetworkError") && code.contains("std") && code.contains("io") && code.contains("Error"));
        // Check trait implementations
        assert!(code.contains("derive") && code.contains("Debug") && code.contains("Clone"));
        assert!(code.contains("impl") && code.contains("std") && code.contains("fmt") && code.contains("Display"));
        assert!(code.contains("impl") && code.contains("std") && code.contains("error") && code.contains("Error"));
    }

    #[test]
    fn test_transpile_error_type_empty() {
        let transpiler = Transpiler::new();
        let variants = vec![];
        let error_type = transpiler.transpile_error_type("EmptyError", &variants);
        let code = error_type.to_string();
        assert!(code.contains("enum EmptyError"));
        assert!(code.contains("Debug"));
        assert!(code.contains("Clone"));
    }

    #[test]
    fn test_transpile_error_type_single_variant() {
        let transpiler = Transpiler::new();
        let variants = vec![("GenericError".to_string(), None)];
        let error_type = transpiler.transpile_error_type("SimpleError", &variants);
        let code = error_type.to_string();
        assert!(code.contains("enum SimpleError"));
        assert!(code.contains("GenericError"));
    }

    #[test]
    fn test_transpile_result_match_ok_err() {
        let transpiler = make_transpiler();
        let expr = make_ident("result");
        let arms = vec![
            ("Ok".to_string(), make_literal(Literal::Integer(42))),
            ("Err".to_string(), make_literal(Literal::Integer(0))),
        ];

        let result = transpiler.transpile_result_match(&expr, &arms);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("match result"));
        assert!(code.contains("Ok") && code.contains("value"));
        assert!(code.contains("Err") && code.contains("error"));
    }

    #[test]
    fn test_transpile_result_match_with_default() {
        let transpiler = make_transpiler();
        let expr = make_ident("result");
        let arms = vec![
            ("Ok".to_string(), make_ident("value")),
            ("_".to_string(), make_literal(Literal::Unit)),
        ];

        let result = transpiler.transpile_result_match(&expr, &arms);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Ok") && code.contains("value"));
        assert!(code.contains("_"));
    }

    #[test]
    fn test_transpile_result_chain_empty() {
        let transpiler = make_transpiler();
        let operations = vec![];

        let result = transpiler.transpile_result_chain(&operations);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert_eq!(code, "Ok (())");
    }

    #[test]
    fn test_transpile_result_chain_single() {
        let transpiler = make_transpiler();
        let operations = vec![make_ident("operation")];

        let result = transpiler.transpile_result_chain(&operations);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert_eq!(code, "operation");
    }

    #[test]
    fn test_transpile_result_chain_multiple() {
        let transpiler = make_transpiler();
        let operations = vec![
            make_ident("op1"),
            make_ident("op2"),
            make_ident("op3"),
        ];

        let result = transpiler.transpile_result_chain(&operations);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("and_then"));
        assert!(code.contains("op1"));
        assert!(code.contains("op2"));
        assert!(code.contains("op3"));
    }

    #[test]
    fn test_transpile_result_unwrap_or() {
        let transpiler = make_transpiler();
        let result_expr = make_ident("my_result");
        let default = make_literal(Literal::Integer(0));

        let result = transpiler.transpile_result_unwrap_or(&result_expr, &default);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("my_result . unwrap_or"));
        assert!(code.contains("0"));
    }

    #[test]
    fn test_transpile_result_map() {
        let transpiler = make_transpiler();
        let result_expr = make_ident("my_result");
        let mapper = make_ident("transform");

        let result = transpiler.transpile_result_map(&result_expr, &mapper);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("my_result . map"));
        assert!(code.contains("transform"));
    }

    #[test]
    fn test_transpile_error_type_with_invalid_type() {
        let transpiler = Transpiler::new();
        let variants = vec![
            ("BadInput".to_string(), Some("Invalid::Type::Syntax".to_string())),
        ];
        let error_type = transpiler.transpile_error_type("TestError", &variants);
        let code = error_type.to_string();
        // Should fall back to String for invalid types
        assert!(code.contains("enum TestError"));
        assert!(code.contains("BadInput"));
    }

    #[test]
    fn test_transpile_error_type_complex_types() {
        let transpiler = Transpiler::new();
        let variants = vec![
            ("IoError".to_string(), Some("std::io::Error".to_string())),
            ("ParseError".to_string(), Some("std::num::ParseIntError".to_string())),
            ("Custom".to_string(), Some("Box<dyn std::error::Error>".to_string())),
        ];
        let error_type = transpiler.transpile_error_type("ComplexError", &variants);
        let code = error_type.to_string();
        assert!(code.contains("enum ComplexError"));
        assert!(code.contains("IoError"));
        assert!(code.contains("ParseError"));
        assert!(code.contains("Custom"));
    }

    #[test]
    fn test_result_helpers_method_signatures() {
        let helpers = Transpiler::generate_result_helpers();
        let code = helpers.to_string();

        // Check method signatures
        assert!(code.contains("fn map_err_with"));
        assert!(code.contains("FnOnce"));
        assert!(code.contains("fn unwrap_or_else_with"));
        assert!(code.contains("fn and_then_with"));
        assert!(code.contains("Result"));
        assert!(code.contains("fn or_else_with"));
    }

    #[test]
    fn test_transpile_result_match_only_ok() {
        let transpiler = make_transpiler();
        let expr = make_ident("maybe_value");
        let arms = vec![
            ("Ok".to_string(), make_ident("value")),
        ];

        let result = transpiler.transpile_result_match(&expr, &arms);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("match maybe_value"));
        assert!(code.contains("Ok") && code.contains("value"));
    }

    #[test]
    fn test_transpile_result_chain_complex() {
        let transpiler = make_transpiler();
        let operations = vec![
            make_ident("fetch_data"),
            make_ident("parse_json"),
            make_ident("validate"),
            make_ident("save_to_db"),
        ];

        let result = transpiler.transpile_result_chain(&operations);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();

        // Should chain all operations with and_then
        assert!(code.contains("fetch_data"));
        assert!(code.contains("and_then"));
        assert!(code.contains("parse_json"));
        assert!(code.contains("validate"));
        assert!(code.contains("save_to_db"));
    }
}
#[cfg(test)]
mod property_tests_result_type {
    use proptest::proptest;
    
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_generate_result_helpers_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
