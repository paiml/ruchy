//! `DataFrame` builder pattern transpilation for correct Polars API
//! 
//! Transforms Ruchy's builder pattern into valid Polars code

use super::Transpiler;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use crate::frontend::ast::{Expr, ExprKind};

impl Transpiler {
    /// Transpile `DataFrame` builder pattern chains
    /// Transforms: `DataFrame::new().column("a", [1,2]).column("b", [3,4]).build()`
    /// Into: `DataFrame::new(vec![Series::new("a", &[1,2]), Series::new("b", &[3,4])])`
    pub fn transpile_dataframe_builder(&self, expr: &Expr) -> Result<Option<TokenStream>> {
        // Check if this is a DataFrame builder pattern
        if let Some((columns, _base)) = self.extract_dataframe_builder_chain(expr) {
            // Generate Series for each column
            let mut series_tokens = Vec::new();
            for (name, data) in columns {
                let name_tokens = self.transpile_expr(&name)?;
                let data_tokens = self.transpile_expr(&data)?;
                series_tokens.push(quote! {
                    polars::prelude::Series::new(#name_tokens, &#data_tokens)
                });
            }
            
            // Generate the DataFrame constructor
            if series_tokens.is_empty() {
                Ok(Some(quote! { polars::prelude::DataFrame::empty() }))
            } else {
                Ok(Some(quote! {
                    polars::prelude::DataFrame::new(vec![#(#series_tokens),*]).unwrap()
                }))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Extract `DataFrame` builder chain pattern
    /// Returns columns and base expression if it's a builder pattern
    fn extract_dataframe_builder_chain(&self, expr: &Expr) -> Option<(Vec<(Expr, Expr)>, Expr)> {
        match &expr.kind {
            // .build() at the end
            ExprKind::MethodCall { receiver, method, args } if method == "build" && args.is_empty() => {
                self.extract_column_chain(receiver)
            }
            // Just column chains without .build()
            ExprKind::MethodCall { receiver, method, args } if method == "column" && args.len() == 2 => {
                if let Some((mut cols, base)) = self.extract_column_chain(receiver) {
                    cols.push((args[0].clone(), args[1].clone()));
                    Some((cols, base))
                } else {
                    Some((vec![(args[0].clone(), args[1].clone())], receiver.as_ref().clone()))
                }
            }
            _ => None
        }
    }
    
    /// Extract column method calls recursively
    fn extract_column_chain(&self, expr: &Expr) -> Option<(Vec<(Expr, Expr)>, Expr)> {
        match &expr.kind {
            ExprKind::MethodCall { receiver, method, args } if method == "column" && args.len() == 2 => {
                if let Some((mut cols, base)) = self.extract_column_chain(receiver) {
                    cols.push((args[0].clone(), args[1].clone()));
                    Some((cols, base))
                } else {
                    // Base case: reached the DataFrame::new()
                    Some((vec![(args[0].clone(), args[1].clone())], receiver.as_ref().clone()))
                }
            }
            ExprKind::Call { func, args } => {
                // Check if it's DataFrame::new()
                if let ExprKind::QualifiedName { module, name } = &func.kind {
                    if module == "DataFrame" && name == "new" && args.is_empty() {
                        return Some((Vec::new(), expr.clone()));
                    }
                }
                None
            }
            _ => None
        }
    }
    
    /// Check if expression is a `DataFrame` builder pattern
    pub fn is_dataframe_builder(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::MethodCall { method, .. } => {
                matches!(method.as_str(), "column" | "build")
            }
            ExprKind::Call { func, .. } => {
                if let ExprKind::QualifiedName { module, name } = &func.kind {
                    module == "DataFrame" && name == "new"
                } else {
                    false
                }
            }
            _ => false
        }
    }
}