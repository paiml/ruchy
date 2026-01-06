//! Lambda/Closure Transpilation
//!
//! This module handles transpilation of lambda expressions and closures
//! to Rust closure syntax with proper type annotations.
//!
//! **EXTREME TDD Round 71**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, Param};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Transpile lambda expressions to Rust closures
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn transpile_lambda_impl(
        &self,
        params: &[Param],
        body: &Expr,
    ) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;

        if params.is_empty() {
            return Ok(quote! { move || #body_tokens });
        }

        let param_list = self.build_lambda_param_list(params)?;
        let closure_str = format!("move |{param_list}| {body_tokens}");
        closure_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse closure: {e}"))
    }

    /// Build parameter list string with type annotations
    fn build_lambda_param_list(&self, params: &[Param]) -> Result<String> {
        let param_strs: Vec<String> = params
            .iter()
            .map(|p| {
                let name = p.name();
                let ty_str = self
                    .transpile_type(&p.ty)
                    .map_or_else(|_| "_".to_string(), |t| t.to_string());
                if ty_str == "_" {
                    name
                } else {
                    format!("{name}: {ty_str}")
                }
            })
            .collect();
        Ok(param_strs.join(", "))
    }

    /// Transpile async lambda expressions
    pub(crate) fn transpile_async_lambda_impl(
        &self,
        params: &[Param],
        body: &Expr,
    ) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;

        if params.is_empty() {
            return Ok(quote! { move || async move { #body_tokens } });
        }

        let param_list = self.build_lambda_param_list(params)?;
        let closure_str = format!("move |{param_list}| async move {{ {body_tokens} }}");
        closure_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse async closure: {e}"))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, ExprKind, Literal, Pattern, Span, Type, TypeKind};

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

    fn make_param(name: &str) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    fn make_typed_param(name: &str, type_name: &str) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Named(type_name.to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    // ========================================================================
    // transpile_lambda_impl tests
    // ========================================================================

    #[test]
    fn test_lambda_no_params() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler.transpile_lambda_impl(&[], &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("move"));
        assert!(result_str.contains("||"));
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_lambda_single_param_inferred() {
        let transpiler = make_transpiler();
        let params = vec![make_param("x")];
        let body = ident_expr("x");
        let result = transpiler.transpile_lambda_impl(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("move"));
        assert!(result_str.contains("x"));
    }

    #[test]
    fn test_lambda_single_param_typed() {
        let transpiler = make_transpiler();
        let params = vec![make_typed_param("x", "i32")];
        let body = ident_expr("x");
        let result = transpiler.transpile_lambda_impl(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("move"));
        assert!(result_str.contains("i32"));
    }

    #[test]
    fn test_lambda_multiple_params() {
        let transpiler = make_transpiler();
        let params = vec![make_typed_param("a", "i32"), make_typed_param("b", "i32")];
        let body = make_expr(ExprKind::Binary {
            left: Box::new(ident_expr("a")),
            op: BinaryOp::Add,
            right: Box::new(ident_expr("b")),
        });
        let result = transpiler.transpile_lambda_impl(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("move"));
        assert!(result_str.contains("a"));
        assert!(result_str.contains("b"));
    }

    #[test]
    fn test_lambda_complex_body() {
        let transpiler = make_transpiler();
        let params = vec![make_param("n")];
        let body = make_expr(ExprKind::Binary {
            left: Box::new(ident_expr("n")),
            op: BinaryOp::Multiply,
            right: Box::new(int_expr(2)),
        });
        let result = transpiler.transpile_lambda_impl(&params, &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("n"));
        assert!(result_str.contains("2"));
    }

    // ========================================================================
    // build_lambda_param_list tests
    // ========================================================================

    #[test]
    fn test_build_param_list_empty() {
        let transpiler = make_transpiler();
        let result = transpiler.build_lambda_param_list(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_build_param_list_single_inferred() {
        let transpiler = make_transpiler();
        let params = vec![make_param("x")];
        let result = transpiler.build_lambda_param_list(&params).unwrap();
        assert_eq!(result, "x");
    }

    #[test]
    fn test_build_param_list_single_typed() {
        let transpiler = make_transpiler();
        let params = vec![make_typed_param("x", "i32")];
        let result = transpiler.build_lambda_param_list(&params).unwrap();
        assert!(result.contains("x"));
        assert!(result.contains("i32"));
    }

    #[test]
    fn test_build_param_list_multiple() {
        let transpiler = make_transpiler();
        let params = vec![make_typed_param("a", "i32"), make_typed_param("b", "String")];
        let result = transpiler.build_lambda_param_list(&params).unwrap();
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains(","));
    }

    // ========================================================================
    // transpile_async_lambda_impl tests
    // ========================================================================

    #[test]
    fn test_async_lambda_no_params() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler.transpile_async_lambda_impl(&[], &body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("async"));
        assert!(result_str.contains("move"));
    }

    #[test]
    fn test_async_lambda_with_params() {
        let transpiler = make_transpiler();
        let params = vec![make_typed_param("x", "i32")];
        let body = ident_expr("x");
        let result = transpiler
            .transpile_async_lambda_impl(&params, &body)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("async"));
        assert!(result_str.contains("move"));
    }
}
