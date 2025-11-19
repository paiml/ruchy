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
            UnaryOp::MutableReference => quote! { &mut #operand_tokens }, // PARSER-085: Issue #71
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    // Helper: Create test transpiler instance
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper: Create identifier expression
    fn ident_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create string literal expression
    fn string_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: transpile_unary - Not operator
    #[test]
    fn test_transpile_unary_not() {
        let transpiler = test_transpiler();
        let operand = ident_expr("x");
        let result = transpiler.transpile_unary(UnaryOp::Not, &operand).unwrap();
        assert_eq!(result.to_string(), "! x");
    }

    // Test 2: transpile_unary - BitwiseNot operator
    #[test]
    fn test_transpile_unary_bitwise_not() {
        let transpiler = test_transpiler();
        let operand = ident_expr("bits");
        let result = transpiler
            .transpile_unary(UnaryOp::BitwiseNot, &operand)
            .unwrap();
        assert_eq!(result.to_string(), "! bits");
    }

    // Test 3: transpile_unary - Negate operator
    #[test]
    fn test_transpile_unary_negate() {
        let transpiler = test_transpiler();
        let operand = ident_expr("num");
        let result = transpiler
            .transpile_unary(UnaryOp::Negate, &operand)
            .unwrap();
        assert_eq!(result.to_string(), "- num");
    }

    // Test 4: transpile_unary - Reference operator
    #[test]
    fn test_transpile_unary_reference() {
        let transpiler = test_transpiler();
        let operand = ident_expr("value");
        let result = transpiler
            .transpile_unary(UnaryOp::Reference, &operand)
            .unwrap();
        assert_eq!(result.to_string(), "& value");
    }

    // Test 5: transpile_unary - MutableReference operator (PARSER-085: Issue #71)
    #[test]
    fn test_transpile_unary_mutable_reference() {
        let transpiler = test_transpiler();
        let operand = ident_expr("data");
        let result = transpiler
            .transpile_unary(UnaryOp::MutableReference, &operand)
            .unwrap();
        assert_eq!(result.to_string(), "& mut data");
    }

    // Test 6: transpile_unary - Deref operator
    #[test]
    fn test_transpile_unary_deref() {
        let transpiler = test_transpiler();
        let operand = ident_expr("ptr");
        let result = transpiler
            .transpile_unary(UnaryOp::Deref, &operand)
            .unwrap();
        assert_eq!(result.to_string(), "* ptr");
    }

    // Test 7: transpile_await - basic await expression
    #[test]
    fn test_transpile_await_basic() {
        let transpiler = test_transpiler();
        let expr = ident_expr("future");
        let result = transpiler.transpile_await(&expr).unwrap();
        assert_eq!(result.to_string(), "future . await");
    }

    // Test 8: transpile_spawn - struct literal (actor pattern)
    #[test]
    fn test_transpile_spawn_struct_literal() {
        let transpiler = test_transpiler();
        let actor = Expr {
            kind: ExprKind::StructLiteral {
                name: "Worker".to_string(),
                fields: vec![("id".to_string(), ident_expr("worker_id"))],
                base: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_spawn(&actor).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("Arc"));
        assert!(result_str.contains("Mutex"));
        assert!(result_str.contains("Worker"));
        assert!(result_str.contains("id"));
    }

    // Test 9: transpile_spawn - non-struct expression (fallback)
    #[test]
    fn test_transpile_spawn_non_struct() {
        let transpiler = test_transpiler();
        let actor = ident_expr("actor_instance");
        let result = transpiler.transpile_spawn(&actor).unwrap();
        assert_eq!(result.to_string(), "actor_instance");
    }

    // Test 10: transpile_async_block - basic block (SPEC-001-E)
    #[test]
    fn test_transpile_async_block_basic() {
        let transpiler = test_transpiler();
        let body = ident_expr("value");
        let result = transpiler.transpile_async_block(&body).unwrap();
        assert_eq!(result.to_string(), "{ value }");
    }

    // Test 11: transpile_async_lambda - no params
    #[test]
    fn test_transpile_async_lambda_no_params() {
        let transpiler = test_transpiler();
        let params = vec![];
        let body = string_expr("result");
        let result = transpiler.transpile_async_lambda(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("async move"));
        assert!(result_str.contains("\"result\""));
    }

    // Test 12: transpile_async_lambda - single param
    #[test]
    fn test_transpile_async_lambda_single_param() {
        let transpiler = test_transpiler();
        let params = vec!["x".to_string()];
        let body = ident_expr("x");
        let result = transpiler.transpile_async_lambda(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("| x |"));
        assert!(result_str.contains("async move"));
    }

    // Test 13: transpile_async_lambda - multiple params
    #[test]
    fn test_transpile_async_lambda_multiple_params() {
        let transpiler = test_transpiler();
        let params = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let body = ident_expr("result");
        let result = transpiler.transpile_async_lambda(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains('a'));
        assert!(result_str.contains('b'));
        assert!(result_str.contains('c'));
        assert!(result_str.contains("async move"));
    }

    // Test 14: transpile_throw - string message
    #[test]
    fn test_transpile_throw_string() {
        let transpiler = test_transpiler();
        let expr = string_expr("Error occurred");
        let result = transpiler.transpile_throw(&expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("panic"));
        assert!(result_str.contains("\"Error occurred\""));
    }

    // Test 15: transpile_throw - identifier expression
    #[test]
    fn test_transpile_throw_identifier() {
        let transpiler = test_transpiler();
        let expr = ident_expr("error_msg");
        let result = transpiler.transpile_throw(&expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("panic"));
        assert!(result_str.contains("error_msg"));
    }
}
