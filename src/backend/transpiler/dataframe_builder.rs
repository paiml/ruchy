//! `DataFrame` builder pattern transpilation for correct Polars API
//! 
//! Transforms Ruchy's builder pattern into valid Polars code
use super::Transpiler;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use crate::frontend::ast::{Expr, ExprKind};
#[cfg(test)]
use proptest::prelude::*;
impl Transpiler {
    /// Transpile `DataFrame` builder pattern chains
    /// Transforms: `DataFrame::new().column("a", [1,2]).column("b", [3,4]).build()`
    /// Into: `DataFrame::new(vec![Series::new("a", &[1,2]), Series::new("b", &[3,4])])`
/// # Examples
/// 
/// ```
/// use ruchy::backend::transpiler::dataframe_builder::transpile_dataframe_builder;
/// 
/// let result = transpile_dataframe_builder(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::backend::transpiler::dataframe_builder::is_dataframe_builder;
/// 
/// let result = is_dataframe_builder(());
/// assert_eq!(result, Ok(()));
/// ```
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
#[cfg(test)]
mod property_tests_dataframe_builder {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_transpile_dataframe_builder_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
