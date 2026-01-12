//! Function Body Token Generation
//!
//! This module handles the generation of function body tokens,
//! including proper handling of async context, Set/Block bodies,
//! and statement semicolon placement.
//!
//! **EXTREME TDD Round 70**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Generate body tokens with async support
    /// Complexity: 10 (at Toyota Way limit)
    pub(crate) fn generate_body_tokens_impl(
        &self,
        body: &Expr,
        is_async: bool,
    ) -> Result<TokenStream> {
        if is_async {
            return self.generate_async_body(body);
        }

        self.generate_sync_body(body)
    }

    /// Generate async function body
    fn generate_async_body(&self, body: &Expr) -> Result<TokenStream> {
        let mut async_transpiler = Transpiler::new();
        async_transpiler.in_async_context = true;
        async_transpiler.transpile_expr(body)
    }

    /// Generate synchronous function body
    fn generate_sync_body(&self, body: &Expr) -> Result<TokenStream> {
        match &body.kind {
            ExprKind::Set(elements) => self.transpile_set_as_body(elements),
            ExprKind::Block(exprs) => self.transpile_block_as_body(exprs),
            _ => self.transpile_expr(body),
        }
    }

    /// Transpile a Set expression as a function body
    /// (Emergency fix for parser regression)
    fn transpile_set_as_body(&self, elements: &[Expr]) -> Result<TokenStream> {
        if elements.len() == 1 {
            return self.transpile_expr(&elements[0]);
        }

        let mut statements = Vec::new();
        for (i, expr) in elements.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            if i < elements.len() - 1 {
                statements.push(quote! { #expr_tokens; });
            } else {
                statements.push(expr_tokens);
            }
        }
        Ok(quote! { { #(#statements)* } })
    }

    /// Transpile a Block expression as a function body
    fn transpile_block_as_body(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.len() == 1 {
            return self.transpile_expr(&exprs[0]);
        }

        let mut statements = Vec::new();
        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            let is_let = Self::is_let_expression(expr);

            if i < exprs.len() - 1 {
                self.push_non_final_statement(&mut statements, expr_tokens, is_let);
            } else {
                self.push_final_statement(&mut statements, expr_tokens, is_let, expr);
            }
        }

        if statements.is_empty() {
            Ok(quote! {})
        } else {
            Ok(quote! { #(#statements)* })
        }
    }

    /// Check if expression is a let binding
    fn is_let_expression(expr: &Expr) -> bool {
        matches!(
            &expr.kind,
            ExprKind::Let { .. } | ExprKind::LetPattern { .. }
        )
    }

    /// Push a non-final statement with proper semicolon handling
    fn push_non_final_statement(
        &self,
        statements: &mut Vec<TokenStream>,
        expr_tokens: TokenStream,
        is_let: bool,
    ) {
        if is_let {
            statements.push(expr_tokens);
        } else {
            statements.push(quote! { #expr_tokens; });
        }
    }

    /// Push a final statement with proper semicolon handling
    fn push_final_statement(
        &self,
        statements: &mut Vec<TokenStream>,
        expr_tokens: TokenStream,
        is_let: bool,
        expr: &Expr,
    ) {
        if super::function_analysis::is_void_expression(expr) {
            if is_let {
                statements.push(expr_tokens);
            } else {
                statements.push(quote! { #expr_tokens; });
            }
        } else {
            statements.push(expr_tokens);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span};

    fn make_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // generate_body_tokens_impl tests
    // ========================================================================

    #[test]
    fn test_body_simple_expression() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler.generate_body_tokens_impl(&body, false).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_body_single_block() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Block(vec![int_expr(42)]));
        let result = transpiler.generate_body_tokens_impl(&body, false).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_body_multi_statement_block() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Block(vec![ident_expr("a"), int_expr(42)]));
        let result = transpiler.generate_body_tokens_impl(&body, false).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("a"));
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_body_async_context() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler.generate_body_tokens_impl(&body, true).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_body_set_single() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Set(vec![int_expr(1)]));
        let result = transpiler.generate_body_tokens_impl(&body, false).unwrap();
        assert!(result.to_string().contains("1"));
    }

    #[test]
    fn test_body_set_multiple() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Set(vec![ident_expr("x"), int_expr(2)]));
        let result = transpiler.generate_body_tokens_impl(&body, false).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("x"));
        assert!(result_str.contains("2"));
    }

    // ========================================================================
    // is_let_expression tests
    // ========================================================================

    #[test]
    fn test_is_let_expression_let() {
        let expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_expr(1)),
            body: Box::new(int_expr(0)),
            is_mutable: false,
            else_block: None,
        });
        assert!(Transpiler::is_let_expression(&expr));
    }

    #[test]
    fn test_is_let_expression_let_pattern() {
        let expr = make_expr(ExprKind::LetPattern {
            pattern: crate::frontend::ast::Pattern::Wildcard,
            type_annotation: None,
            value: Box::new(int_expr(1)),
            body: Box::new(int_expr(0)),
            is_mutable: false,
            else_block: None,
        });
        assert!(Transpiler::is_let_expression(&expr));
    }

    #[test]
    fn test_is_let_expression_not_let() {
        let expr = int_expr(42);
        assert!(!Transpiler::is_let_expression(&expr));
    }

    // ========================================================================
    // transpile_set_as_body tests
    // ========================================================================

    #[test]
    fn test_transpile_set_as_body_empty() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_set_as_body(&[]).unwrap();
        assert!(result.to_string().contains("{"));
    }

    #[test]
    fn test_transpile_set_as_body_single() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_set_as_body(&[int_expr(99)]).unwrap();
        assert!(result.to_string().contains("99"));
    }

    #[test]
    fn test_transpile_set_as_body_multiple() {
        let transpiler = make_transpiler();
        let elements = vec![ident_expr("a"), ident_expr("b"), int_expr(3)];
        let result = transpiler.transpile_set_as_body(&elements).unwrap();
        let result_str = result.to_string();
        // Intermediate statements should have semicolons
        assert!(result_str.contains(";"));
        assert!(result_str.contains("3"));
    }

    // ========================================================================
    // transpile_block_as_body tests
    // ========================================================================

    #[test]
    fn test_transpile_block_as_body_empty() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_block_as_body(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_transpile_block_as_body_single() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_block_as_body(&[int_expr(42)]).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_transpile_block_as_body_with_let() {
        let transpiler = make_transpiler();
        let let_expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_expr(1)),
            body: Box::new(ident_expr("x")),
            is_mutable: false,
            else_block: None,
        });
        let exprs = vec![let_expr, int_expr(42)];
        let result = transpiler.transpile_block_as_body(&exprs).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
    }
}
