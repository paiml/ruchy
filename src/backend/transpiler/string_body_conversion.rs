//! String Body Conversion Module
//!
//! This module handles transpilation of function bodies that need to return String,
//! including proper .to_string() wrapping and string literal handling.
//!
//! **EXTREME TDD Round 73**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal, MatchArm, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Generate body tokens with .to_string() wrapper on last expression
    /// Complexity: 10 (at Toyota Way limit)
    pub(crate) fn generate_body_tokens_with_string_conversion_impl(
        &self,
        body: &Expr,
        is_async: bool,
    ) -> Result<TokenStream> {
        if is_async {
            return self.generate_async_string_body(body);
        }

        match &body.kind {
            ExprKind::Block(exprs) if exprs.len() > 1 => {
                self.convert_multi_expr_block_to_string(exprs)
            }
            ExprKind::Block(exprs) if exprs.len() == 1 => {
                self.convert_single_expr_block_to_string(&exprs[0])
            }
            ExprKind::Match { expr, arms } => self.transpile_match_with_string_arms_impl(expr, arms),
            _ => {
                let body_tokens = self.transpile_expr(body)?;
                Ok(quote! { (#body_tokens).to_string() })
            }
        }
    }

    /// Generate async body with string conversion
    fn generate_async_string_body(&self, body: &Expr) -> Result<TokenStream> {
        let mut async_transpiler = Transpiler::new();
        async_transpiler.in_async_context = true;
        let body_tokens = async_transpiler.transpile_expr(body)?;
        Ok(quote! { (#body_tokens).to_string() })
    }

    /// Convert multi-expression block to String
    fn convert_multi_expr_block_to_string(&self, exprs: &[Expr]) -> Result<TokenStream> {
        let mut statements = Vec::new();
        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            let is_let = matches!(
                &expr.kind,
                ExprKind::Let { .. } | ExprKind::LetPattern { .. }
            );

            if i < exprs.len() - 1 {
                if is_let {
                    statements.push(expr_tokens);
                } else {
                    statements.push(quote! { #expr_tokens; });
                }
            } else {
                statements.push(quote! { (#expr_tokens).to_string() });
            }
        }
        Ok(quote! { { #(#statements)* } })
    }

    /// Convert single-expression block to String
    fn convert_single_expr_block_to_string(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Let {
                name,
                type_annotation,
                value,
                body: let_body,
                is_mutable,
                ..
            } => self.convert_let_body_to_string(name, type_annotation, value, let_body, *is_mutable),
            ExprKind::Match { expr, arms } => self.transpile_match_with_string_arms_impl(expr, arms),
            _ => {
                let expr_tokens = self.transpile_expr(expr)?;
                Ok(quote! { (#expr_tokens).to_string() })
            }
        }
    }

    /// Convert let binding body to String
    fn convert_let_body_to_string(
        &self,
        name: &str,
        type_annotation: &Option<crate::frontend::ast::Type>,
        value: &Expr,
        let_body: &Expr,
        is_mutable: bool,
    ) -> Result<TokenStream> {
        let name_ident = format_ident!("{}", name);
        let is_mutable_var = self.check_mutability(name, is_mutable, let_body);
        let value_tokens = self.convert_value_for_string_context(name, value, type_annotation, is_mutable_var)?;
        let let_body_tokens = self.transpile_expr(let_body)?;

        let mutability = if is_mutable_var {
            quote! { mut }
        } else {
            quote! {}
        };

        let type_annotation_tokens = if let Some(ty) = type_annotation {
            let ty_tokens = self.transpile_type(ty)?;
            quote! { : #ty_tokens }
        } else {
            quote! {}
        };

        Ok(quote! {
            {
                let #mutability #name_ident #type_annotation_tokens = #value_tokens;
                (#let_body_tokens).to_string()
            }
        })
    }

    /// Check if variable needs mutability
    fn check_mutability(&self, name: &str, is_mutable: bool, body: &Expr) -> bool {
        is_mutable
            || self.mutable_vars.contains(name)
            || super::mutation_detection::is_variable_mutated(name, body)
    }

    /// Convert value expression for string context
    fn convert_value_for_string_context(
        &self,
        name: &str,
        value: &Expr,
        type_annotation: &Option<crate::frontend::ast::Type>,
        is_mutable_var: bool,
    ) -> Result<TokenStream> {
        match (&value.kind, type_annotation) {
            // String literal with String type annotation
            (ExprKind::Literal(Literal::String(s)), Some(type_ann))
                if matches!(&type_ann.kind, TypeKind::Named(n) if n == "String") =>
            {
                self.string_vars.borrow_mut().insert(name.to_string());
                Ok(quote! { #s.to_string() })
            }
            // Mutable string literal without type annotation
            (ExprKind::Literal(Literal::String(s)), None) if is_mutable_var => {
                self.string_vars.borrow_mut().insert(name.to_string());
                Ok(quote! { String::from(#s) })
            }
            // Array with List type annotation
            (ExprKind::List(_), Some(type_ann)) if matches!(&type_ann.kind, TypeKind::List(_)) => {
                let list_tokens = self.transpile_expr(value)?;
                Ok(quote! { #list_tokens.to_vec() })
            }
            // Function call - track in string_vars
            (ExprKind::Call { .. }, _) => {
                self.string_vars.borrow_mut().insert(name.to_string());
                self.transpile_expr(value)
            }
            _ => self.transpile_expr(value),
        }
    }

    /// Transpile match expression with string literal arm conversion
    pub(crate) fn transpile_match_with_string_arms_impl(
        &self,
        expr: &Expr,
        arms: &[MatchArm],
    ) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        let arm_tokens = self.convert_match_arms_to_string(arms)?;

        Ok(quote! {
            match #expr_tokens {
                #(#arm_tokens,)*
            }
        })
    }

    /// Convert match arms to string
    fn convert_match_arms_to_string(&self, arms: &[MatchArm]) -> Result<Vec<TokenStream>> {
        let mut arm_tokens = Vec::new();

        for arm in arms {
            let pattern_tokens = self.transpile_pattern(&arm.pattern)?;
            let body_tokens = self.convert_arm_body_to_string(&arm.body)?;

            if let Some(guard_expr) = &arm.guard {
                let guard_tokens = self.transpile_expr(guard_expr)?;
                arm_tokens.push(quote! {
                    #pattern_tokens if #guard_tokens => #body_tokens
                });
            } else {
                arm_tokens.push(quote! {
                    #pattern_tokens => #body_tokens
                });
            }
        }

        Ok(arm_tokens)
    }

    /// Convert arm body to string (adding .to_string() for literals)
    fn convert_arm_body_to_string(&self, body: &Expr) -> Result<TokenStream> {
        match &body.kind {
            ExprKind::Literal(Literal::String(s)) => Ok(quote! { #s.to_string() }),
            _ => self.transpile_expr(body),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Pattern, Span, Type};

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

    fn make_match_arm(pattern: Pattern, body: Expr) -> MatchArm {
        MatchArm {
            pattern,
            guard: None,
            body: Box::new(body),
            span: Span::default(),
        }
    }

    fn make_type(type_name: &str) -> Type {
        Type {
            kind: TypeKind::Named(type_name.to_string()),
            span: Span::default(),
        }
    }

    // ========================================================================
    // generate_body_tokens_with_string_conversion_impl tests
    // ========================================================================

    #[test]
    fn test_simple_expression_to_string() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler
            .generate_body_tokens_with_string_conversion_impl(&body, false)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_async_body_to_string() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler
            .generate_body_tokens_with_string_conversion_impl(&body, true)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_block_multiple_exprs_to_string() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Block(vec![ident_expr("a"), int_expr(42)]));
        let result = transpiler
            .generate_body_tokens_with_string_conversion_impl(&body, false)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("a"));
        assert!(result_str.contains(";"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_block_single_expr_to_string() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Block(vec![int_expr(99)]));
        let result = transpiler
            .generate_body_tokens_with_string_conversion_impl(&body, false)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("99"));
        assert!(result_str.contains("to_string"));
    }

    // ========================================================================
    // convert_multi_expr_block_to_string tests
    // ========================================================================

    #[test]
    fn test_multi_expr_block_semicolons() {
        let transpiler = make_transpiler();
        let exprs = vec![ident_expr("x"), ident_expr("y"), int_expr(10)];
        let result = transpiler.convert_multi_expr_block_to_string(&exprs).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("x"));
        assert!(result_str.contains("y"));
        assert!(result_str.contains(";"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_multi_expr_block_with_let() {
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
        let result = transpiler.convert_multi_expr_block_to_string(&exprs).unwrap();
        assert!(result.to_string().contains("to_string"));
    }

    // ========================================================================
    // transpile_match_with_string_arms_impl tests
    // ========================================================================

    #[test]
    fn test_match_with_string_literal_arms() {
        let transpiler = make_transpiler();
        let expr = ident_expr("x");
        let arms = vec![
            make_match_arm(Pattern::Literal(Literal::Integer(1, None)), string_expr("one")),
            make_match_arm(Pattern::Wildcard, string_expr("other")),
        ];
        let result = transpiler
            .transpile_match_with_string_arms_impl(&expr, &arms)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("match"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_match_with_non_string_arms() {
        let transpiler = make_transpiler();
        let expr = ident_expr("x");
        let arms = vec![
            make_match_arm(Pattern::Literal(Literal::Integer(1, None)), int_expr(10)),
            make_match_arm(Pattern::Wildcard, int_expr(0)),
        ];
        let result = transpiler
            .transpile_match_with_string_arms_impl(&expr, &arms)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("match"));
        assert!(result_str.contains("10"));
    }

    // ========================================================================
    // convert_arm_body_to_string tests
    // ========================================================================

    #[test]
    fn test_convert_string_literal_arm() {
        let transpiler = make_transpiler();
        let body = string_expr("hello");
        let result = transpiler.convert_arm_body_to_string(&body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("hello"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_convert_non_string_arm() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler.convert_arm_body_to_string(&body).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
        assert!(!result_str.contains("to_string"));
    }

    // ========================================================================
    // convert_value_for_string_context tests
    // ========================================================================

    #[test]
    fn test_convert_string_literal_with_string_type() {
        let transpiler = make_transpiler();
        let value = string_expr("hello");
        let type_ann = Some(make_type("String"));
        let result = transpiler
            .convert_value_for_string_context("x", &value, &type_ann, false)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("hello"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_convert_mutable_string_literal() {
        let transpiler = make_transpiler();
        let value = string_expr("hello");
        let result = transpiler
            .convert_value_for_string_context("x", &value, &None, true)
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("String :: from"));
    }

    #[test]
    fn test_convert_integer_value() {
        let transpiler = make_transpiler();
        let value = int_expr(42);
        let result = transpiler
            .convert_value_for_string_context("x", &value, &None, false)
            .unwrap();
        assert!(result.to_string().contains("42"));
    }

    // ========================================================================
    // check_mutability tests
    // ========================================================================

    #[test]
    fn test_check_mutability_explicit() {
        let transpiler = make_transpiler();
        let body = int_expr(0);
        assert!(transpiler.check_mutability("x", true, &body));
    }

    #[test]
    fn test_check_mutability_not_mutable() {
        let transpiler = make_transpiler();
        let body = int_expr(0);
        assert!(!transpiler.check_mutability("x", false, &body));
    }
}
