//! Field and index access transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    pub fn transpile_field_access(&self, object: &Expr, field: &str) -> Result<TokenStream> {
        use crate::frontend::ast::ExprKind;
        let obj_tokens = self.transpile_expr(object)?;
        // Check if the object is an ObjectLiteral (HashMap) or module path
        match &object.kind {
            ExprKind::ObjectLiteral { .. } => {
                // Direct object literal access - use get()
                Ok(quote! {
                    #obj_tokens.get(#field)
                        .cloned()
                        .unwrap_or_else(|| panic!("Field '{}' not found", #field))
                })
            }
            ExprKind::FieldAccess { .. } => {
                // Nested field access - check if numeric (tuple) or struct field
                if field.chars().all(|c| c.is_ascii_digit()) {
                    // Nested tuple access like (nested.0).1
                    let index: usize = field.parse().unwrap();
                    let index = syn::Index::from(index);
                    Ok(quote! { #obj_tokens.#index })
                } else {
                    // Nested struct field access like rect.top_left.x - use . syntax
                    let field_ident = format_ident!("{}", field);
                    Ok(quote! { #obj_tokens.#field_ident })
                }
            }
            ExprKind::Identifier(name) if name.contains("::") => {
                // Module path identifier - use :: syntax
                let field_ident = format_ident!("{}", field);
                Ok(quote! { #obj_tokens::#field_ident })
            }
            _ => {
                // Check if field is numeric (tuple field access)
                if field.chars().all(|c| c.is_ascii_digit()) {
                    // Tuple field access - use numeric index
                    let index: usize = field.parse().unwrap();
                    let index = syn::Index::from(index);
                    Ok(quote! { #obj_tokens.#index })
                } else {
                    // Regular struct field access
                    let field_ident = format_ident!("{}", field);
                    Ok(quote! { #obj_tokens.#field_ident })
                }
            }
        }
    }
    /// Transpiles index access `(array[index])`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_index_access;
    ///
    /// let result = transpile_index_access(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_index_access(&self, object: &Expr, index: &Expr) -> Result<TokenStream> {
        use crate::frontend::ast::{ExprKind, Literal};
        let obj_tokens = self.transpile_expr(object)?;
        let index_tokens = self.transpile_expr(index)?;
        // Smart index access: HashMap.get() for string keys, array indexing for numeric
        match &index.kind {
            // String literal keys use HashMap.get()
            ExprKind::Literal(Literal::String(_)) => Ok(quote! {
                #obj_tokens.get(#index_tokens)
                    .cloned()
                    .unwrap_or_else(|| panic!("Key not found"))
            }),
            // Numeric and other keys use array indexing
            _ => Ok(quote! { #obj_tokens[#index_tokens as usize] }),
        }
    }
    /// Transpiles slice access `(array[start:end])`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_slice;
    ///
    /// let result = transpile_slice(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_slice(
        &self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
    ) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        match (start, end) {
            (None, None) => {
                // Full slice [..]
                Ok(quote! { &#obj_tokens[..] })
            }
            (None, Some(end)) => {
                // Slice from beginning [..end]
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[..#end_tokens as usize] })
            }
            (Some(start), None) => {
                // Slice to end [start..]
                let start_tokens = self.transpile_expr(start)?;
                Ok(quote! { &#obj_tokens[#start_tokens as usize..] })
            }
            (Some(start), Some(end)) => {
                // Full range slice [start..end]
                let start_tokens = self.transpile_expr(start)?;
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[#start_tokens as usize..#end_tokens as usize] })
            }
        }
    }
}
