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
                if let Some(val_expr) = value {
                    let val_tokens = self.transpile_expr(val_expr)?;
                    Ok(quote! { return #val_tokens; })
                } else {
                    Ok(quote! { return; })
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
