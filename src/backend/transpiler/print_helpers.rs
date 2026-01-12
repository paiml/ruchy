//! Print Macro Helper Transpilation
//!
//! This module handles transpilation of print-related macros:
//! - println/print with string interpolation
//! - dbg and panic macros
//! - Multiple argument formatting
//! - Value printing with type-aware formatting
//!
//! **EXTREME TDD Round 64**: Extracted from statements.rs and mod.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Transpiles println/print with string interpolation directly
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"println("Hello {name}")"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("println"));
    /// ```
    /// Complexity: 6 (within Toyota Way limits)
    pub fn transpile_print_with_interpolation(
        &self,
        func_name: &str,
        parts: &[crate::frontend::ast::StringPart],
    ) -> Result<TokenStream> {
        if parts.is_empty() {
            let func_tokens = proc_macro2::Ident::new(func_name, proc_macro2::Span::call_site());
            return Ok(quote! { #func_tokens!("") });
        }
        let mut format_string = String::new();
        let mut args = Vec::new();
        for part in parts {
            match part {
                crate::frontend::ast::StringPart::Text(s) => {
                    // Escape any format specifiers in literal parts
                    format_string.push_str(&s.replace('{', "{{").replace('}', "}}"));
                }
                crate::frontend::ast::StringPart::Expr(expr) => {
                    format_string.push_str("{}");
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
                crate::frontend::ast::StringPart::ExprWithFormat { expr, format_spec } => {
                    // Include the format specifier in the format string
                    format_string.push('{');
                    format_string.push_str(format_spec);
                    format_string.push('}');
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
            }
        }
        let func_tokens = proc_macro2::Ident::new(func_name, proc_macro2::Span::call_site());
        Ok(quote! {
            #func_tokens!(#format_string #(, #args)*)
        })
    }

    /// Try to transpile print/println/dbg/panic macros
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("println(42)");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("println"));
    /// ```
    /// Complexity: 7 (within Toyota Way limits)
    pub fn try_transpile_print_macro(
        &self,
        func_tokens: &TokenStream,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        if !(base_name == "println"
            || base_name == "print"
            || base_name == "dbg"
            || base_name == "panic")
        {
            return Ok(None);
        }
        // Handle single argument with string interpolation
        if (base_name == "println" || base_name == "print") && args.len() == 1 {
            if let ExprKind::StringInterpolation { parts } = &args[0].kind {
                return Ok(Some(
                    self.transpile_print_with_interpolation(base_name, parts)?,
                ));
            }
            // For single non-string arguments, add smart format string
            if !matches!(&args[0].kind, ExprKind::Literal(Literal::String(_))) {
                let arg_tokens = self.transpile_expr(&args[0])?;
                // DEFECT-DICT-DETERMINISM FIX: Use Debug format with BTreeMap (deterministic)
                // BTreeMap Debug format is sorted, so {:?} is safe and deterministic
                let format_str = "{:?}";
                return Ok(Some(quote! { #func_tokens!(#format_str, #arg_tokens) }));
            }
        }
        // Handle multiple arguments
        if args.len() > 1 {
            return self.transpile_print_multiple_args(func_tokens, args);
        }
        // Single string literal or simple case
        // RUCHYRUCHY-001: Escape braces in string literals used as format strings
        if args.len() == 1 {
            if let ExprKind::Literal(Literal::String(s)) = &args[0].kind {
                // For println!/print!, string literals are format strings, so escape braces
                let escaped = s.replace('{', "{{").replace('}', "}}");
                return Ok(Some(quote! { #func_tokens!(#escaped) }));
            }
        }
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;
        Ok(Some(quote! { #func_tokens!(#(#arg_tokens),*) }))
    }

    /// Handle multiple arguments for print macros
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"println("Hello", "World")"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("println"));
    /// ```
    /// Complexity: 9 (within Toyota Way limits)
    pub fn transpile_print_multiple_args(
        &self,
        func_tokens: &TokenStream,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        // FIXED: Don't treat first string argument as format string
        // Instead, treat all arguments as values to print with spaces
        if args.is_empty() {
            return Ok(Some(quote! { #func_tokens!() }));
        }
        let all_args: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let all_args = all_args?;
        if args.len() == 1 {
            // Single argument - check if it's a string-like expression
            match &args[0].kind {
                ExprKind::Literal(Literal::String(_)) | ExprKind::StringInterpolation { .. } => {
                    // String literal or interpolation - use Display format
                    Ok(Some(quote! { #func_tokens!("{}", #(#all_args)*) }))
                }
                ExprKind::Identifier(_) => {
                    // For identifiers, we can't know the type at compile time
                    // Use a runtime check to decide format
                    let arg = &all_args[0];
                    let printing_logic = self
                        .generate_value_printing_tokens(quote! { #arg }, quote! { #func_tokens });
                    Ok(Some(printing_logic))
                }
                _ => {
                    // DEFECT-DICT-DETERMINISM FIX: Debug format is OK with BTreeMap (sorted)
                    Ok(Some(quote! { #func_tokens!("{:?}", #(#all_args)*) }))
                }
            }
        } else {
            // Multiple arguments - check if first is format string
            if let ExprKind::Literal(Literal::String(format_str)) = &args[0].kind {
                if format_str.contains("{}") {
                    // First argument is a format string, rest are values
                    let format_arg = &all_args[0];
                    let value_args = &all_args[1..];
                    Ok(Some(
                        quote! { #func_tokens!(#format_arg, #(#value_args),*) },
                    ))
                } else {
                    // First argument is regular string, treat all as separate values
                    let format_parts: Vec<_> = args
                        .iter()
                        .map(|arg| match &arg.kind {
                            ExprKind::Literal(Literal::String(_)) => "{}",
                            _ => "{:?}",
                        })
                        .collect();
                    let format_str = format_parts.join(" ");
                    Ok(Some(quote! { #func_tokens!(#format_str, #(#all_args),*) }))
                }
            } else {
                // No format string, treat all as separate values
                let format_parts: Vec<_> = args
                    .iter()
                    .map(|arg| match &arg.kind {
                        ExprKind::Literal(Literal::String(_)) => "{}",
                        _ => "{:?}",
                    })
                    .collect();
                let format_str = format_parts.join(" ");
                Ok(Some(quote! { #func_tokens!(#format_str, #(#all_args),*) }))
            }
        }
    }

    /// Centralized value printing logic for functions like println
    ///
    /// Generates code that checks type at runtime to avoid quotes around strings.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let tokens = transpiler.generate_value_printing_tokens(
    ///     quote! { my_value },
    ///     quote! { println }
    /// );
    /// ```
    /// Complexity: 3 (within Toyota Way limits)
    pub fn generate_value_printing_tokens(
        &self,
        value_expr: TokenStream,
        func_tokens: TokenStream,
    ) -> TokenStream {
        quote! {
            {
                use std::any::Any;
                let value = #value_expr;
                // Special handling for String and &str types to avoid quotes
                if let Some(s) = (&value as &dyn Any).downcast_ref::<String>() {
                    #func_tokens!("{}", s)
                } else if let Some(s) = (&value as &dyn Any).downcast_ref::<&str>() {
                    #func_tokens!("{}", s)
                } else {
                    #func_tokens!("{:?}", value)
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span, StringPart};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn interpolation_expr(parts: Vec<StringPart>) -> Expr {
        make_expr(ExprKind::StringInterpolation { parts })
    }

    // ========================================================================
    // transpile_print_with_interpolation tests
    // ========================================================================

    #[test]
    fn test_print_interpolation_empty() {
        let transpiler = Transpiler::new();
        let parts: Vec<StringPart> = vec![];
        let result = transpiler.transpile_print_with_interpolation("println", &parts);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("println"));
    }

    #[test]
    fn test_print_interpolation_text_only() {
        let transpiler = Transpiler::new();
        let parts = vec![StringPart::Text("Hello, World!".to_string())];
        let result = transpiler.transpile_print_with_interpolation("println", &parts);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("println"));
        assert!(tokens_str.contains("Hello, World!"));
    }

    #[test]
    fn test_print_interpolation_expr() {
        let transpiler = Transpiler::new();
        let parts = vec![
            StringPart::Text("Value: ".to_string()),
            StringPart::Expr(Box::new(int_expr(42))),
        ];
        let result = transpiler.transpile_print_with_interpolation("println", &parts);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("println"));
        assert!(tokens_str.contains("Value:"));
        assert!(tokens_str.contains("42"));
    }

    #[test]
    fn test_print_interpolation_with_format_spec() {
        let transpiler = Transpiler::new();
        let parts = vec![StringPart::ExprWithFormat {
            expr: Box::new(int_expr(42)),
            format_spec: ":>10".to_string(),
        }];
        let result = transpiler.transpile_print_with_interpolation("print", &parts);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("print"));
    }

    #[test]
    fn test_print_interpolation_escapes_braces() {
        let transpiler = Transpiler::new();
        let parts = vec![StringPart::Text("Use {braces}".to_string())];
        let result = transpiler.transpile_print_with_interpolation("println", &parts);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        // Braces should be escaped
        assert!(tokens_str.contains("{{braces}}"));
    }

    #[test]
    fn test_print_interpolation_multiple_parts() {
        let transpiler = Transpiler::new();
        let parts = vec![
            StringPart::Text("Hello, ".to_string()),
            StringPart::Expr(Box::new(ident_expr("name"))),
            StringPart::Text("! You have ".to_string()),
            StringPart::Expr(Box::new(int_expr(5))),
            StringPart::Text(" messages.".to_string()),
        ];
        let result = transpiler.transpile_print_with_interpolation("println", &parts);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("println"));
        assert!(tokens_str.contains("name"));
    }

    // ========================================================================
    // try_transpile_print_macro tests
    // ========================================================================

    #[test]
    fn test_print_macro_println() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![string_expr("Hello")];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "println", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("println"));
    }

    #[test]
    fn test_print_macro_println_escapes_braces() {
        // RUCHYRUCHY-001: Test brace escaping for ruchyruchy bootstrap compatibility
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![string_expr("Delimiters: {, }")];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "println", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        // Braces should be escaped to {{ and }}
        assert!(
            tokens_str.contains("{{") && tokens_str.contains("}}"),
            "Expected escaped braces, got: {}",
            tokens_str
        );
    }

    #[test]
    fn test_print_macro_print() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { print };
        let args = vec![string_expr("Hello")];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "print", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_macro_dbg() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { dbg };
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "dbg", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_macro_panic() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { panic };
        let args = vec![string_expr("error!")];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "panic", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_macro_unknown() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { foo };
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "foo", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_print_macro_with_interpolation() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![interpolation_expr(vec![
            StringPart::Text("Hello, ".to_string()),
            StringPart::Expr(Box::new(ident_expr("name"))),
        ])];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "println", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_macro_single_int() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_print_macro(&func_tokens, "println", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        // Should use {:?} format for non-string
        assert!(tokens_str.contains("{:?}"));
    }

    // ========================================================================
    // transpile_print_multiple_args tests
    // ========================================================================

    #[test]
    fn test_print_multiple_args_empty() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args: Vec<Expr> = vec![];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_multiple_args_single_string() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![string_expr("Hello")];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("{}"));
    }

    #[test]
    fn test_print_multiple_args_single_ident() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![ident_expr("value")];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_multiple_args_single_int() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![int_expr(42)];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("{:?}"));
    }

    #[test]
    fn test_print_multiple_args_format_string() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![string_expr("Value: {}"), int_expr(42)];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_multiple_args_no_format_string() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![string_expr("Hello"), string_expr("World")];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_multiple_args_mixed_types() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![string_expr("Count:"), int_expr(42), ident_expr("name")];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    #[test]
    fn test_print_multiple_args_no_format_all_ints() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { println };
        let args = vec![int_expr(1), int_expr(2), int_expr(3)];
        let result = transpiler.transpile_print_multiple_args(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        // Should have {:?} format parts
        assert!(tokens_str.contains("{:?}"));
    }

    // ========================================================================
    // generate_value_printing_tokens tests
    // ========================================================================

    #[test]
    fn test_generate_value_printing_tokens() {
        let transpiler = Transpiler::new();
        let value_expr = quote! { my_value };
        let func_tokens = quote! { println };
        let result = transpiler.generate_value_printing_tokens(value_expr, func_tokens);
        let result_str = result.to_string();
        assert!(result_str.contains("Any"));
        assert!(result_str.contains("downcast_ref"));
        assert!(result_str.contains("String"));
    }

    #[test]
    fn test_generate_value_printing_tokens_different_func() {
        let transpiler = Transpiler::new();
        let value_expr = quote! { x };
        let func_tokens = quote! { print };
        let result = transpiler.generate_value_printing_tokens(value_expr, func_tokens);
        let result_str = result.to_string();
        assert!(result_str.contains("print"));
    }

    #[test]
    fn test_generate_value_printing_tokens_complex_expr() {
        let transpiler = Transpiler::new();
        let value_expr = quote! { some_struct.field };
        let func_tokens = quote! { println };
        let result = transpiler.generate_value_printing_tokens(value_expr, func_tokens);
        let result_str = result.to_string();
        assert!(result_str.contains("some_struct"));
    }
}
