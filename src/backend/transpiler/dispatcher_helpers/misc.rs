//! Miscellaneous helper functions (type cast, break/continue/return)

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    pub(in crate::backend::transpiler) fn transpile_type_cast(&self, expr: &Expr, target_type: &str) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        // Map Ruchy types to Rust types
        let rust_type = match target_type {
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            "f32" => quote! { f32 },
            "f64" => quote! { f64 },
            "usize" => quote! { usize },
            "u8" => quote! { u8 },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "i8" => quote! { i8 },
            "i16" => quote! { i16 },
            _ => bail!("Unsupported cast target type: {target_type}"),
        };
        Ok(quote! { (#expr_tokens as #rust_type) })
    }

    pub(in crate::backend::transpiler) fn transpile_control_misc_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Break { label, value } => {
                if let Some(val_expr) = value {
                    let val_tokens = self.transpile_expr(val_expr)?;
                    Ok(Self::make_break_continue_with_value(true, label.as_ref(), Some(val_tokens)))
                } else {
                    Ok(Self::make_break_continue(true, label.as_ref()))
                }
            }
            ExprKind::Continue { label } => {
                Ok(Self::make_break_continue(false, label.as_ref()))
            }
            ExprKind::Return { value } => {
                // ISSUE-103: Don't add semicolon - will be added by statement context
                // In match arms, return is an expression and shouldn't have trailing semicolon
                if let Some(val_expr) = value {
                    let val_tokens = self.transpile_expr(val_expr)?;
                    Ok(quote! { return #val_tokens })
                } else {
                    Ok(quote! { return })
                }
            }
            _ => unreachable!(),
        }
    }

    pub(in crate::backend::transpiler) fn make_break_continue(is_break: bool, label: Option<&String>) -> TokenStream {
        let keyword = if is_break {
            quote! { break }
        } else {
            quote! { continue }
        };

        match label {
            Some(l) if !l.is_empty() => {
                let label_name = l.strip_prefix('\'').unwrap_or(l);
                let label_ident = format_ident!("{}", label_name);
                quote! { #keyword #label_ident }
            }
            _ => keyword,
        }
    }

    pub(in crate::backend::transpiler) fn make_break_continue_with_value(
        is_break: bool,
        label: Option<&String>,
        value: Option<TokenStream>,
    ) -> TokenStream {
        let keyword = if is_break {
            quote! { break }
        } else {
            quote! { continue }
        };

        match (label, value) {
            (Some(l), Some(v)) if !l.is_empty() => {
                let label_name = l.strip_prefix('\'').unwrap_or(l);
                let label_ident = format_ident!("{}", label_name);
                quote! { #keyword #label_ident #v }
            }
            (Some(l), None) if !l.is_empty() => {
                let label_name = l.strip_prefix('\'').unwrap_or(l);
                let label_ident = format_ident!("{}", label_name);
                quote! { #keyword #label_ident }
            }
            (_, Some(v)) => quote! { #keyword #v },
            _ => keyword,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
    use quote::quote;

    // Helper: Create test transpiler
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper: Create integer literal expression
    fn int_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: transpile_type_cast - i32
    #[test]
    fn test_transpile_type_cast_i32() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "i32").unwrap();
        assert!(result.to_string().contains("as i32"));
    }

    // Test 2: transpile_type_cast - i64
    #[test]
    fn test_transpile_type_cast_i64() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "i64").unwrap();
        assert!(result.to_string().contains("as i64"));
    }

    // Test 3: transpile_type_cast - f32
    #[test]
    fn test_transpile_type_cast_f32() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "f32").unwrap();
        assert!(result.to_string().contains("as f32"));
    }

    // Test 4: transpile_type_cast - f64
    #[test]
    fn test_transpile_type_cast_f64() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "f64").unwrap();
        assert!(result.to_string().contains("as f64"));
    }

    // Test 5: transpile_type_cast - usize
    #[test]
    fn test_transpile_type_cast_usize() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "usize").unwrap();
        assert!(result.to_string().contains("as usize"));
    }

    // Test 6: transpile_type_cast - u8
    #[test]
    fn test_transpile_type_cast_u8() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "u8").unwrap();
        assert!(result.to_string().contains("as u8"));
    }

    // Test 7: transpile_type_cast - unsupported type (error path)
    #[test]
    fn test_transpile_type_cast_unsupported() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);
        let result = transpiler.transpile_type_cast(&expr, "CustomType");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported cast target type"));
    }

    // Test 8: transpile_control_misc_expr - break without value or label
    #[test]
    fn test_transpile_control_misc_expr_break_simple() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Break { label: None, value: None },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_misc_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "break");
    }

    // Test 9: transpile_control_misc_expr - break with value
    #[test]
    fn test_transpile_control_misc_expr_break_with_value() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Break {
                label: None,
                value: Some(Box::new(int_expr(42))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_misc_expr(&expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("break"));
        assert!(result_str.contains("42"));
    }

    // Test 10: transpile_control_misc_expr - continue without label
    #[test]
    fn test_transpile_control_misc_expr_continue_simple() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Continue { label: None },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_misc_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "continue");
    }

    // Test 11: transpile_control_misc_expr - return without value
    #[test]
    fn test_transpile_control_misc_expr_return_void() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Return { value: None },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_misc_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "return");
    }

    // Test 12: transpile_control_misc_expr - return with value (ISSUE-103)
    #[test]
    fn test_transpile_control_misc_expr_return_with_value() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Return {
                value: Some(Box::new(int_expr(42))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_misc_expr(&expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("return"));
        assert!(result_str.contains("42"));
        assert!(!result_str.ends_with(';')); // ISSUE-103: No trailing semicolon
    }

    // Test 13: make_break_continue - break without label
    #[test]
    fn test_make_break_continue_break() {
        let result = Transpiler::make_break_continue(true, None);
        assert_eq!(result.to_string(), "break");
    }

    // Test 14: make_break_continue - continue without label
    #[test]
    fn test_make_break_continue_continue() {
        let result = Transpiler::make_break_continue(false, None);
        assert_eq!(result.to_string(), "continue");
    }

    // Test 15: make_break_continue - break with label
    #[test]
    fn test_make_break_continue_break_with_label() {
        let label = String::from("'outer");
        let result = Transpiler::make_break_continue(true, Some(&label));
        assert!(result.to_string().contains("break"));
        assert!(result.to_string().contains("outer"));
    }

    // Test 16: make_break_continue_with_value - break with label and value
    #[test]
    fn test_make_break_continue_with_value_all() {
        let label = String::from("'loop1");
        let value = quote! { 42 };
        let result = Transpiler::make_break_continue_with_value(true, Some(&label), Some(value));
        let result_str = result.to_string();
        assert!(result_str.contains("break"));
        assert!(result_str.contains("loop1"));
        assert!(result_str.contains("42"));
    }

    // Test 17: make_break_continue_with_value - break with value but no label
    #[test]
    fn test_make_break_continue_with_value_no_label() {
        let value = quote! { result };
        let result = Transpiler::make_break_continue_with_value(true, None, Some(value));
        let result_str = result.to_string();
        assert!(result_str.contains("break"));
        assert!(result_str.contains("result"));
    }

    // Test 18: make_break_continue_with_value - continue with label (no value)
    #[test]
    fn test_make_break_continue_with_value_continue_with_label() {
        let label = String::from("'outer");
        let result = Transpiler::make_break_continue_with_value(false, Some(&label), None);
        let result_str = result.to_string();
        assert!(result_str.contains("continue"));
        assert!(result_str.contains("outer"));
    }

    // Test 19: make_break_continue_with_value - empty label treated as no label
    #[test]
    fn test_make_break_continue_with_value_empty_label() {
        let label = String::from("");
        let result = Transpiler::make_break_continue_with_value(true, Some(&label), None);
        assert_eq!(result.to_string(), "break");
    }

    // Test 20: transpile_type_cast - all unsigned types
    #[test]
    fn test_transpile_type_cast_unsigned_types() {
        let transpiler = test_transpiler();
        let expr = int_expr(42);

        let u16_result = transpiler.transpile_type_cast(&expr, "u16").unwrap();
        assert!(u16_result.to_string().contains("as u16"));

        let u32_result = transpiler.transpile_type_cast(&expr, "u32").unwrap();
        assert!(u32_result.to_string().contains("as u32"));

        let u64_result = transpiler.transpile_type_cast(&expr, "u64").unwrap();
        assert!(u64_result.to_string().contains("as u64"));
    }
}
