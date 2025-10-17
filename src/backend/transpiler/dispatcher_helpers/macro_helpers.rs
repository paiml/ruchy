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
        let first_is_format_string = match &args[0].kind {
            ExprKind::Literal(Literal::String(s)) => s.contains("{}"),
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

    /// Transpile assert_eq! macro with validation
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

    /// Transpile assert_ne! macro with validation
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
