//! Lifetime Parameter Helpers
//!
//! This module handles generation of function parameters and return types
//! with lifetime annotations for borrowed references.
//!
//! **EXTREME TDD Round 74**: Extracted from statements.rs for modularization.

use super::return_type_helpers::returns_string_literal;
use super::Transpiler;
use crate::frontend::ast::{Expr, Param, Type, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Generate parameter tokens with lifetime annotations
    /// Complexity: 8 (within Toyota Way limits)
    pub(crate) fn generate_param_tokens_with_lifetime_impl(
        &self,
        params: &[Param],
        body: &Expr,
        func_name: &str,
    ) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|p| self.generate_single_param_with_lifetime(p, body, func_name))
            .collect()
    }

    /// Generate a single parameter with lifetime annotation
    fn generate_single_param_with_lifetime(
        &self,
        param: &Param,
        body: &Expr,
        func_name: &str,
    ) -> Result<TokenStream> {
        let param_name = format_ident!("{}", param.name());

        // Handle special Rust receiver syntax
        if param.name() == "self" {
            return self.generate_self_receiver(param);
        }

        // Regular parameter handling
        let type_tokens = self.get_param_type_with_lifetime(param, body, func_name)?;

        if param.is_mutable {
            Ok(quote! { mut #param_name: #type_tokens })
        } else {
            Ok(quote! { #param_name: #type_tokens })
        }
    }

    /// Generate self receiver syntax
    fn generate_self_receiver(&self, param: &Param) -> Result<TokenStream> {
        if let TypeKind::Reference { is_mut, .. } = &param.ty.kind {
            if *is_mut {
                Ok(quote! { &mut self })
            } else {
                Ok(quote! { &self })
            }
        } else {
            Ok(quote! { self })
        }
    }

    /// Get parameter type with lifetime, falling back to inference
    fn get_param_type_with_lifetime(
        &self,
        param: &Param,
        body: &Expr,
        func_name: &str,
    ) -> Result<TokenStream> {
        if let Ok(tokens) = self.transpile_type_with_lifetime_impl(&param.ty) {
            let token_str = tokens.to_string();
            if token_str == "_" {
                Ok(self.infer_param_type(param, body, func_name))
            } else {
                Ok(tokens)
            }
        } else {
            Ok(self.infer_param_type(param, body, func_name))
        }
    }

    /// Transpile type with lifetime annotation (&T becomes &'a T)
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn transpile_type_with_lifetime_impl(&self, ty: &Type) -> Result<TokenStream> {
        match &ty.kind {
            TypeKind::Reference {
                is_mut,
                inner,
                lifetime: _,
            } => self.generate_reference_type_with_lifetime(inner, *is_mut),
            _ => self.transpile_type(ty),
        }
    }

    /// Generate reference type with lifetime annotation
    fn generate_reference_type_with_lifetime(
        &self,
        inner: &Type,
        is_mut: bool,
    ) -> Result<TokenStream> {
        // Special case &str to avoid double reference
        let inner_tokens = if let TypeKind::Named(name) = &inner.kind {
            if name == "str" {
                quote! { str }
            } else {
                self.transpile_type(inner)?
            }
        } else {
            self.transpile_type(inner)?
        };

        let mut_token = if is_mut {
            quote! { mut }
        } else {
            quote! {}
        };

        Ok(quote! { &'a #mut_token #inner_tokens })
    }

    /// Generate return type tokens with lifetime annotation
    /// Complexity: 7 (within Toyota Way limits)
    pub(crate) fn generate_return_type_tokens_with_lifetime_impl(
        &self,
        name: &str,
        return_type: Option<&Type>,
        body: &Expr,
    ) -> Result<TokenStream> {
        if let Some(ty) = return_type {
            let ty_tokens = self.transpile_type_with_lifetime_impl(ty)?;
            return Ok(quote! { -> #ty_tokens });
        }

        if name == "main" {
            return Ok(quote! {});
        }

        self.infer_return_type_with_lifetime(name, body)
    }

    /// Infer return type with lifetime when not explicitly specified
    fn infer_return_type_with_lifetime(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        if super::function_analysis::returns_closure(body) {
            return Ok(quote! { -> impl Fn(i32) -> i32 });
        }

        if super::function_analysis::looks_like_numeric_function(name) {
            return Ok(quote! { -> i32 });
        }

        if returns_string_literal(body) {
            return Ok(quote! { -> &'a str });
        }

        if super::function_analysis::has_non_unit_expression(body) {
            return Ok(quote! { -> i32 });
        }

        Ok(quote! {})
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Pattern, Span};

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

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
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

    fn make_ref_param(name: &str, inner_type: &str, is_mut: bool) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Reference {
                    is_mut,
                    inner: Box::new(Type {
                        kind: TypeKind::Named(inner_type.to_string()),
                        span: Span::default(),
                    }),
                    lifetime: None,
                },
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    fn make_type(kind: TypeKind) -> Type {
        Type {
            kind,
            span: Span::default(),
        }
    }

    // ========================================================================
    // generate_param_tokens_with_lifetime_impl tests
    // ========================================================================

    #[test]
    fn test_param_tokens_empty() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler
            .generate_param_tokens_with_lifetime_impl(&[], &body, "test")
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_param_tokens_single_inferred() {
        let transpiler = make_transpiler();
        let params = vec![make_param("x")];
        let body = ident_expr("x");
        let result = transpiler
            .generate_param_tokens_with_lifetime_impl(&params, &body, "test")
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("x"));
    }

    #[test]
    fn test_param_tokens_reference_type() {
        let transpiler = make_transpiler();
        let params = vec![make_ref_param("data", "i32", false)];
        let body = ident_expr("data");
        let result = transpiler
            .generate_param_tokens_with_lifetime_impl(&params, &body, "test")
            .unwrap();
        assert_eq!(result.len(), 1);
        let token_str = result[0].to_string();
        assert!(token_str.contains("data"));
        assert!(token_str.contains("'a")); // Should have lifetime
    }

    // ========================================================================
    // generate_self_receiver tests
    // ========================================================================

    #[test]
    fn test_self_receiver_owned() {
        let transpiler = make_transpiler();
        let param = make_typed_param("self", "Self");
        let result = transpiler.generate_self_receiver(&param).unwrap();
        assert_eq!(result.to_string(), "self");
    }

    #[test]
    fn test_self_receiver_immutable_ref() {
        let transpiler = make_transpiler();
        let param = make_ref_param("self", "Self", false);
        let result = transpiler.generate_self_receiver(&param).unwrap();
        assert!(result.to_string().contains("& self"));
    }

    #[test]
    fn test_self_receiver_mutable_ref() {
        let transpiler = make_transpiler();
        let param = make_ref_param("self", "Self", true);
        let result = transpiler.generate_self_receiver(&param).unwrap();
        assert!(result.to_string().contains("& mut self"));
    }

    // ========================================================================
    // transpile_type_with_lifetime_impl tests
    // ========================================================================

    #[test]
    fn test_type_with_lifetime_simple() {
        let transpiler = make_transpiler();
        let ty = make_type(TypeKind::Named("i32".to_string()));
        let result = transpiler.transpile_type_with_lifetime_impl(&ty).unwrap();
        assert!(result.to_string().contains("i32"));
    }

    #[test]
    fn test_type_with_lifetime_reference() {
        let transpiler = make_transpiler();
        let ty = make_type(TypeKind::Reference {
            is_mut: false,
            inner: Box::new(make_type(TypeKind::Named("i32".to_string()))),
            lifetime: None,
        });
        let result = transpiler.transpile_type_with_lifetime_impl(&ty).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("'a"));
        assert!(token_str.contains("i32"));
    }

    #[test]
    fn test_type_with_lifetime_mutable_reference() {
        let transpiler = make_transpiler();
        let ty = make_type(TypeKind::Reference {
            is_mut: true,
            inner: Box::new(make_type(TypeKind::Named("i32".to_string()))),
            lifetime: None,
        });
        let result = transpiler.transpile_type_with_lifetime_impl(&ty).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("'a"));
        assert!(token_str.contains("mut"));
    }

    #[test]
    fn test_type_with_lifetime_str_no_double_ref() {
        let transpiler = make_transpiler();
        let ty = make_type(TypeKind::Reference {
            is_mut: false,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
            lifetime: None,
        });
        let result = transpiler.transpile_type_with_lifetime_impl(&ty).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("'a"));
        assert!(token_str.contains("str"));
        // Should not have double reference like & & str
        assert!(!token_str.contains("& & "));
    }

    // ========================================================================
    // generate_return_type_tokens_with_lifetime_impl tests
    // ========================================================================

    #[test]
    fn test_return_type_explicit() {
        let transpiler = make_transpiler();
        let ty = make_type(TypeKind::Named("i32".to_string()));
        let body = int_expr(42);
        let result = transpiler
            .generate_return_type_tokens_with_lifetime_impl("test", Some(&ty), &body)
            .unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("->"));
        assert!(token_str.contains("i32"));
    }

    #[test]
    fn test_return_type_main_no_type() {
        let transpiler = make_transpiler();
        let body = int_expr(0);
        let result = transpiler
            .generate_return_type_tokens_with_lifetime_impl("main", None, &body)
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_return_type_string_literal() {
        let transpiler = make_transpiler();
        let body = string_expr("hello");
        let result = transpiler
            .generate_return_type_tokens_with_lifetime_impl("greet", None, &body)
            .unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("'a"));
        assert!(token_str.contains("str"));
    }

    // ========================================================================
    // infer_return_type_with_lifetime tests
    // ========================================================================

    #[test]
    fn test_infer_return_numeric_name() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler
            .infer_return_type_with_lifetime("calculate_sum", &body)
            .unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("i32"));
    }

    #[test]
    fn test_infer_return_string_literal_body() {
        let transpiler = make_transpiler();
        let body = string_expr("result");
        let result = transpiler
            .infer_return_type_with_lifetime("get_name", &body)
            .unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("'a"));
        assert!(token_str.contains("str"));
    }
}
