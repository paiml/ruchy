//! Async, unary, and exception handling transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, UnaryOp};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    pub fn transpile_unary(&self, op: UnaryOp, operand: &Expr) -> Result<TokenStream> {
        let operand_tokens = self.transpile_expr(operand)?;
        Ok(match op {
            UnaryOp::Not | UnaryOp::BitwiseNot => quote! { !#operand_tokens },
            UnaryOp::Negate => quote! { -#operand_tokens },
            UnaryOp::Reference => quote! { &#operand_tokens },
            UnaryOp::MutableReference => quote! { &mut #operand_tokens },  // PARSER-085: Issue #71
            UnaryOp::Deref => quote! { *#operand_tokens },
        })
    }
    /// Transpiles await expressions
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_await;
    ///
    /// let result = transpile_await(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_await(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { #expr_tokens.await })
    }

    /// Transpiles spawn expressions for actor creation
    pub fn transpile_spawn(&self, actor: &Expr) -> Result<TokenStream> {
        // Check if it's a struct literal (actor instantiation)
        if let ExprKind::StructLiteral { name, fields, .. } = &actor.kind {
            // Actors transpile to structs with Arc<Mutex<>> for thread safety
            let actor_name = format_ident!("{}", name);
            let field_tokens = fields
                .iter()
                .map(|(name, value)| {
                    let field_name = format_ident!("{}", name);
                    let value_tokens = self.transpile_expr(value)?;
                    Ok(quote! { #field_name: #value_tokens })
                })
                .collect::<Result<Vec<_>>>()?;

            // Create the actor wrapped in Arc<Mutex<>> for thread-safe access
            Ok(quote! {
                std::sync::Arc::new(std::sync::Mutex::new(#actor_name {
                    #(#field_tokens),*
                }))
            })
        } else {
            // For other expressions (e.g., function calls), just evaluate them
            let actor_tokens = self.transpile_expr(actor)?;
            Ok(quote! { #actor_tokens })
        }
    }

    /// Transpiles async blocks
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_async_block;
    ///
    /// let result = transpile_async_block(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_async_block(&self, body: &Expr) -> Result<TokenStream> {
        // SPEC-001-E: Async block - simplified synchronous evaluation
        // For true async support, would need tokio runtime integration
        // Current: Evaluates block immediately (no Future/await)
        let body_tokens = self.transpile_expr(body)?;
        Ok(quote! { { #body_tokens } })
    }

    /// Transpiles async lambda expressions to Rust async closures
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_async_lambda;
    ///
    /// let result = transpile_async_lambda(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_async_lambda(&self, params: &[String], body: &Expr) -> Result<TokenStream> {
        let param_idents: Vec<proc_macro2::Ident> =
            params.iter().map(|p| format_ident!("{}", p)).collect();

        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! { |#(#param_idents),*| async move { #body_tokens } })
    }
    /// Transpiles throw expressions (panic in Rust)
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_throw;
    ///
    /// let result = transpile_throw(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_throw(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! {
            panic!(#expr_tokens)
        })
    }
}
