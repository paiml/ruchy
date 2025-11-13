//! Macro transpilation helper functions
//!
//! This module contains helper functions for transpiling various Ruchy macros
//! to their Rust equivalents (println!, print!, panic!, vec!, assert!, etc.)

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpile println! macro with string formatting support
    ///
    /// Handles string literals, string interpolation, and format strings correctly.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles arguments and wraps them in Rust's `println!` macro.
    /// Empty args produce `println!()`, otherwise `println!(arg1, arg2, ...)`
    pub(in crate::backend::transpiler) fn transpile_println_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens = self.transpile_print_args(args)?;
        if arg_tokens.is_empty() {
            Ok(quote! { println!() })
        } else {
            Ok(quote! { println!(#(#arg_tokens),*) })
        }
    }

    /// Transpile print! macro with string formatting support
    ///
    /// Handles string literals, string interpolation, and format strings correctly.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles arguments and wraps them in Rust's `print!` macro.
    /// Empty args produce `print!()`, otherwise `print!(arg1, arg2, ...)`
    pub(in crate::backend::transpiler) fn transpile_print_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens = self.transpile_print_args(args)?;
        if arg_tokens.is_empty() {
            Ok(quote! { print!() })
        } else {
            Ok(quote! { print!(#(#arg_tokens),*) })
        }
    }

    /// Transpile panic! macro with string formatting support
    ///
    /// Handles string literals, string interpolation, and format strings correctly.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles arguments and wraps them in Rust's `panic!` macro.
    /// Empty args produce `panic!()`, otherwise `panic!(arg1, arg2, ...)`
    pub(in crate::backend::transpiler) fn transpile_panic_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens = self.transpile_print_args(args)?;
        if arg_tokens.is_empty() {
            Ok(quote! { panic!() })
        } else {
            Ok(quote! { panic!(#(#arg_tokens),*) })
        }
    }

    /// Common helper for transpiling print-style macro arguments
    ///
    /// Handles string literals, string interpolation, and format strings.
    /// This eliminates code duplication between println!, print!, and panic!.
    /// Complexity: <10 per Toyota Way requirement.
    fn transpile_print_args(&self, args: &[Expr]) -> Result<Vec<TokenStream>> {
        if args.is_empty() {
            return Ok(vec![]);
        }

        // Check if first argument is a format string (contains {})
        // TRANSPILER-DEFECT-007 FIX: Also recognize {:?}, {:#?}, {:x}, etc. as format strings
        let first_is_format_string = match &args[0].kind {
            ExprKind::Literal(Literal::String(s)) => s.contains('{') && s.contains('}'),
            _ => false,
        };

        if first_is_format_string && args.len() > 1 {
            // First argument is format string, rest are values
            let format_str = match &args[0].kind {
                ExprKind::Literal(Literal::String(s)) => s,
                _ => unreachable!(),
            };

            let mut tokens = vec![quote! { #format_str }];

            // Add remaining arguments as values (without extra format strings)
            for arg in &args[1..] {
                let expr_tokens = self.transpile_expr(arg)?;
                tokens.push(expr_tokens);
            }

            Ok(tokens)
        } else {
            // Original behavior for non-format cases
            args.iter()
                .map(|arg| {
                    match &arg.kind {
                        ExprKind::Literal(Literal::String(s)) => Ok(quote! { #s }),
                        ExprKind::StringInterpolation { parts } => {
                            self.transpile_string_interpolation_for_print(parts)
                        }
                        _ => {
                            // DEFECT-DICT-DETERMINISM FIX: Use Debug format with BTreeMap (deterministic)
                            let expr_tokens = self.transpile_expr(arg)?;
                            Ok(quote! { "{:?}", #expr_tokens })
                        }
                    }
                })
                .collect()
        }
    }

    /// Handle string interpolation for print-style macros
    ///
    /// Detects if string interpolation has expressions or is just format text.
    /// Complexity: <10 per Toyota Way requirement.
    fn transpile_string_interpolation_for_print(
        &self,
        parts: &[crate::frontend::ast::StringPart],
    ) -> Result<TokenStream> {
        let has_expressions = parts.iter().any(|part| {
            matches!(
                part,
                crate::frontend::ast::StringPart::Expr(_)
                    | crate::frontend::ast::StringPart::ExprWithFormat { .. }
            )
        });

        if has_expressions {
            // This has actual interpolation - transpile normally
            self.transpile_string_interpolation(parts)
        } else {
            // This is a format string like "Hello {}" - treat as literal
            let format_string = parts
                .iter()
                .map(|part| match part {
                    crate::frontend::ast::StringPart::Text(s) => s.as_str(),
                    crate::frontend::ast::StringPart::Expr(_)
                    | crate::frontend::ast::StringPart::ExprWithFormat { .. } => unreachable!(),
                })
                .collect::<String>();
            Ok(quote! { #format_string })
        }
    }

    /// Transpile vec! macro
    ///
    /// Simple element-by-element transpilation for collection creation.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles list elements and wraps them in Rust's `vec!` macro.
    /// Produces `vec![elem1, elem2, ...]`
    pub(in crate::backend::transpiler) fn transpile_vec_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { vec![#(#arg_tokens),*] })
    }

    /// Transpile assert! macro
    ///
    /// Simple argument transpilation for basic assertions.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles assertion condition and wraps it in Rust's `assert!` macro.
    /// Produces `assert!(condition, optional_message)`
    pub(in crate::backend::transpiler) fn transpile_assert_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        if arg_tokens.is_empty() {
            Ok(quote! { assert!() })
        } else {
            Ok(quote! { assert!(#(#arg_tokens),*) })
        }
    }

    /// Transpile `assert_eq`! macro with validation
    ///
    /// Validates argument count and transpiles for equality assertions.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Validates at least 2 arguments and transpiles to Rust's `assert_eq!` macro.
    /// Produces `assert_eq!(left, right, optional_message)`
    pub(in crate::backend::transpiler) fn transpile_assert_eq_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() < 2 {
            bail!("assert_eq! requires at least 2 arguments")
        }
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { assert_eq!(#(#arg_tokens),*) })
    }

    /// Transpile `assert_ne`! macro with validation
    ///
    /// Validates argument count and transpiles for inequality assertions.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Validates at least 2 arguments and transpiles to Rust's `assert_ne!` macro.
    /// Produces `assert_ne!(left, right, optional_message)`
    pub(in crate::backend::transpiler) fn transpile_assert_ne_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() < 2 {
            bail!("assert_ne! requires at least 2 arguments")
        }
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { assert_ne!(#(#arg_tokens),*) })
    }

    /// Pass through external macros without modification
    pub(in crate::backend::transpiler) fn transpile_passthrough_macro(&self, name: &str, args: &[Expr]) -> Result<TokenStream> {
        let macro_ident = format_ident!("{}", name);

        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { #macro_ident!(#(#arg_tokens),*) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    // Test 1: transpile_println_macro with empty args
    #[test]
    fn test_println_empty_args() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_println_macro(&[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("println"));
    }

    // Test 2: transpile_println_macro with single string literal
    #[test]
    fn test_println_string_literal() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::String("Hello".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_println_macro(&[arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("println"));
        assert!(tokens.contains("Hello"));
    }

    // Test 3: transpile_println_macro with format string and args
    #[test]
    fn test_println_format_string() {
        let transpiler = Transpiler::new();
        let format_arg = Expr::new(
            ExprKind::Literal(Literal::String("Value: {}".to_string())),
            Span::default(),
        );
        let value_arg = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.transpile_println_macro(&[format_arg, value_arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("println"));
        assert!(tokens.contains("Value: {}"));
        assert!(tokens.contains("42"));
    }

    // Test 4: transpile_print_macro with empty args
    #[test]
    fn test_print_empty_args() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_print_macro(&[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("print"));
    }

    // Test 5: transpile_print_macro with single string
    #[test]
    fn test_print_string_literal() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::String("test".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_print_macro(&[arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("print"));
        assert!(tokens.contains("test"));
    }

    // Test 6: transpile_panic_macro with empty args
    #[test]
    fn test_panic_empty_args() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_panic_macro(&[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("panic"));
    }

    // Test 7: transpile_panic_macro with message
    #[test]
    fn test_panic_with_message() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::String("Error occurred".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_panic_macro(&[arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("panic"));
        assert!(tokens.contains("Error occurred"));
    }

    // Test 8: transpile_vec_macro with empty args
    #[test]
    fn test_vec_empty_args() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_vec_macro(&[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("vec"));
    }

    // Test 9: transpile_vec_macro with multiple elements
    #[test]
    fn test_vec_with_elements() {
        let transpiler = Transpiler::new();
        let elem1 = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let elem2 = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let result = transpiler.transpile_vec_macro(&[elem1, elem2]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("vec"));
        assert!(tokens.contains("1"));
        assert!(tokens.contains("2"));
    }

    // Test 10: transpile_assert_macro with empty args
    #[test]
    fn test_assert_empty_args() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_assert_macro(&[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert"));
    }

    // Test 11: transpile_assert_macro with condition
    #[test]
    fn test_assert_with_condition() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::default(),
        );
        let result = transpiler.transpile_assert_macro(&[arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert"));
        assert!(tokens.contains("true"));
    }

    // Test 12: transpile_assert_eq_macro with valid args
    #[test]
    fn test_assert_eq_valid() {
        let transpiler = Transpiler::new();
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let result = transpiler.transpile_assert_eq_macro(&[left, right]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert_eq"));
    }

    // Test 13: transpile_assert_eq_macro with <2 args (ERROR PATH)
    #[test]
    fn test_assert_eq_insufficient_args_error() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let result = transpiler.transpile_assert_eq_macro(&[arg]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires at least 2 arguments"));
    }

    // Test 14: transpile_assert_ne_macro with valid args
    #[test]
    fn test_assert_ne_valid() {
        let transpiler = Transpiler::new();
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let result = transpiler.transpile_assert_ne_macro(&[left, right]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert_ne"));
    }

    // Test 15: transpile_assert_ne_macro with <2 args (ERROR PATH)
    #[test]
    fn test_assert_ne_insufficient_args_error() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let result = transpiler.transpile_assert_ne_macro(&[arg]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires at least 2 arguments"));
    }

    // Test 16: transpile_passthrough_macro with custom macro name
    #[test]
    fn test_passthrough_macro() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.transpile_passthrough_macro("custom_macro", &[arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("custom_macro"));
        assert!(tokens.contains("42"));
    }

    // Test 17: transpile_print_args with non-format-string expression (Debug format)
    #[test]
    fn test_print_args_debug_format() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Identifier("variable".to_string()),
            Span::default(),
        );
        let result = transpiler.transpile_print_args(&[arg]);
        assert!(result.is_ok());
        let tokens_vec = result.unwrap();
        assert!(!tokens_vec.is_empty());
        let tokens_str = tokens_vec[0].to_string();
        // Should include Debug format {:?}
        assert!(tokens_str.contains("{:?}"));
        assert!(tokens_str.contains("variable"));
    }

    // Test 18: transpile_panic_macro with format string and args
    #[test]
    fn test_panic_format_string() {
        let transpiler = Transpiler::new();
        let format_arg = Expr::new(
            ExprKind::Literal(Literal::String("Error: {}".to_string())),
            Span::default(),
        );
        let value_arg = Expr::new(
            ExprKind::Identifier("error_code".to_string()),
            Span::default(),
        );
        let result = transpiler.transpile_panic_macro(&[format_arg, value_arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("panic"));
        assert!(tokens.contains("Error: {}"));
        assert!(tokens.contains("error_code"));
    }

    // Test 19: transpile_passthrough_macro with multiple args
    #[test]
    fn test_passthrough_macro_multiple_args() {
        let transpiler = Transpiler::new();
        let arg1 = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let arg2 = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let result = transpiler.transpile_passthrough_macro("my_macro", &[arg1, arg2]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("my_macro"));
        assert!(tokens.contains("1"));
        assert!(tokens.contains("2"));
    }

    // Test 20: transpile_print_macro with format string and multiple args
    #[test]
    fn test_print_format_string_multiple_args() {
        let transpiler = Transpiler::new();
        let format_arg = Expr::new(
            ExprKind::Literal(Literal::String("x: {}, y: {}".to_string())),
            Span::default(),
        );
        let arg1 = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let arg2 = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::default(),
        );
        let result = transpiler.transpile_print_macro(&[format_arg, arg1, arg2]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("print"));
        assert!(tokens.contains("x: {}, y: {}"));
        assert!(tokens.contains("10"));
        assert!(tokens.contains("20"));
    }

    // Test 21: transpile_assert_eq_macro with 3 args (condition + message)
    #[test]
    fn test_assert_eq_with_message() {
        let transpiler = Transpiler::new();
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            Span::default(),
        );
        let message = Expr::new(
            ExprKind::Literal(Literal::String("Values should be equal".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_assert_eq_macro(&[left, right, message]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert_eq"));
        assert!(tokens.contains("5"));
        assert!(tokens.contains("Values should be equal"));
    }

    // Test 22: transpile_assert_ne_macro with 3 args (condition + message)
    #[test]
    fn test_assert_ne_with_message() {
        let transpiler = Transpiler::new();
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let message = Expr::new(
            ExprKind::Literal(Literal::String("Values should differ".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_assert_ne_macro(&[left, right, message]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert_ne"));
        assert!(tokens.contains("5"));
        assert!(tokens.contains("10"));
        assert!(tokens.contains("Values should differ"));
    }

    // Test 23: transpile_assert_macro with condition and message
    #[test]
    fn test_assert_with_condition_and_message() {
        let transpiler = Transpiler::new();
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::default(),
        );
        let message = Expr::new(
            ExprKind::Literal(Literal::String("Assertion failed".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_assert_macro(&[condition, message]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert"));
        assert!(tokens.contains("true"));
    }

    // Test 24: transpile_println_macro with non-string literal
    #[test]
    fn test_println_non_string_literal() {
        let transpiler = Transpiler::new();
        let arg = Expr::new(
            ExprKind::Literal(Literal::Integer(123, None)),
            Span::default(),
        );
        let result = transpiler.transpile_println_macro(&[arg]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("println"));
        assert!(tokens.contains("{:?}"));
        assert!(tokens.contains("123"));
    }

    // Test 25: transpile_passthrough_macro with empty args
    #[test]
    fn test_passthrough_macro_empty_args() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_passthrough_macro("empty_macro", &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("empty_macro"));
    }

    // Test 26: transpile_vec_macro with single element
    #[test]
    fn test_vec_single_element() {
        let transpiler = Transpiler::new();
        let elem = Expr::new(
            ExprKind::Literal(Literal::String("single".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_vec_macro(&[elem]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("vec"));
        assert!(tokens.contains("single"));
    }

    // Test 27: transpile_assert_eq_macro with 0 args (ERROR PATH)
    #[test]
    fn test_assert_eq_zero_args_error() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_assert_eq_macro(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires at least 2 arguments"));
    }
}
