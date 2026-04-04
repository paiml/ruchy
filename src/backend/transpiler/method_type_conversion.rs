//! Method-style type conversion transpilation (.to_int(), .to_float(), .to_bool())
//!
//! PDCA-20: These methods were previously falling through to default passthrough
//! in dispatch_method_by_category, emitting verbatim .to_int() which is not valid Rust.
//!
//! String receivers use .parse::<T>().unwrap(), numeric receivers use `as T`.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Transpile `.to_int()` method call to valid Rust
    /// Strings use `.parse::<i64>().unwrap()`, numerics use `as i64`
    pub(super) fn transpile_to_int_method(
        &self,
        obj_tokens: &TokenStream,
        object: &Expr,
    ) -> Result<TokenStream> {
        if self.is_string_typed(object) {
            Ok(quote! { #obj_tokens.parse::<i64>().unwrap() })
        } else {
            Ok(quote! { (#obj_tokens as i64) })
        }
    }

    /// Transpile `.to_float()` method call to valid Rust
    /// Strings use `.parse::<f64>().unwrap()`, numerics use `as f64`
    pub(super) fn transpile_to_float_method(
        &self,
        obj_tokens: &TokenStream,
        object: &Expr,
    ) -> Result<TokenStream> {
        if self.is_string_typed(object) {
            Ok(quote! { #obj_tokens.parse::<f64>().unwrap() })
        } else {
            Ok(quote! { (#obj_tokens as f64) })
        }
    }

    /// Check if an expression is string-typed using AST + tracked string variables
    pub(super) fn is_string_typed(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(Literal::String(_)) | ExprKind::StringInterpolation { .. } => true,
            ExprKind::Identifier(name) => self.string_vars.borrow().contains(name),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;
    use quote::{format_ident, quote};

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
            contracts: Vec::new(),
        }
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // PDCA Cycle 20: .to_int() / .to_float() method call transpilation
    // ========================================================================

    #[test]
    fn test_to_int_on_identifier_emits_cast() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { count };
        let object = ident_expr("count");
        let result = transpiler
            .transpile_to_int_method(&obj_tokens, &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            output.contains("as i64"),
            "Non-string identifier should use `as i64`, got: {output}"
        );
    }

    #[test]
    fn test_to_int_on_string_var_emits_parse() {
        let transpiler = make_transpiler();
        transpiler
            .string_vars
            .borrow_mut()
            .insert("num_str".to_string());
        let obj_tokens = quote! { num_str };
        let object = ident_expr("num_str");
        let result = transpiler
            .transpile_to_int_method(&obj_tokens, &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            output.contains("parse"),
            "String variable should use .parse(), got: {output}"
        );
        assert!(
            output.contains("i64"),
            "Should parse to i64, got: {output}"
        );
    }

    #[test]
    fn test_to_float_on_identifier_emits_cast() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { val };
        let object = ident_expr("val");
        let result = transpiler
            .transpile_to_float_method(&obj_tokens, &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            output.contains("as f64"),
            "Non-string identifier should use `as f64`, got: {output}"
        );
    }

    #[test]
    fn test_to_float_on_string_var_emits_parse() {
        let transpiler = make_transpiler();
        transpiler
            .string_vars
            .borrow_mut()
            .insert("text".to_string());
        let obj_tokens = quote! { text };
        let object = ident_expr("text");
        let result = transpiler
            .transpile_to_float_method(&obj_tokens, &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            output.contains("parse"),
            "String variable should use .parse(), got: {output}"
        );
        assert!(
            output.contains("f64"),
            "Should parse to f64, got: {output}"
        );
    }

    #[test]
    fn test_is_string_typed_literal() {
        let transpiler = make_transpiler();
        let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        assert!(transpiler.is_string_typed(&expr));
    }

    #[test]
    fn test_is_string_typed_identifier_not_registered() {
        let transpiler = make_transpiler();
        let expr = ident_expr("x");
        assert!(!transpiler.is_string_typed(&expr));
    }

    #[test]
    fn test_is_string_typed_identifier_registered() {
        let transpiler = make_transpiler();
        transpiler
            .string_vars
            .borrow_mut()
            .insert("s".to_string());
        let expr = ident_expr("s");
        assert!(transpiler.is_string_typed(&expr));
    }

    #[test]
    fn test_dispatch_to_int_not_verbatim() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { x };
        let method_ident = format_ident!("to_int");
        let object = ident_expr("x");
        let result = transpiler
            .dispatch_method_by_category(&obj_tokens, "to_int", &method_ident, &[], &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            !output.contains(". to_int"),
            "to_int should NOT fall through to default passthrough, got: {output}"
        );
    }

    #[test]
    fn test_dispatch_to_float_not_verbatim() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { x };
        let method_ident = format_ident!("to_float");
        let object = ident_expr("x");
        let result = transpiler
            .dispatch_method_by_category(&obj_tokens, "to_float", &method_ident, &[], &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            !output.contains(". to_float"),
            "to_float should NOT fall through to default passthrough, got: {output}"
        );
    }

    #[test]
    fn test_dispatch_to_bool_not_verbatim() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { val };
        let method_ident = format_ident!("to_bool");
        let object = ident_expr("val");
        let result = transpiler
            .dispatch_method_by_category(&obj_tokens, "to_bool", &method_ident, &[], &object)
            .unwrap();
        let output = result.to_string();
        assert!(
            !output.contains("to_bool"),
            "Should not emit .to_bool() verbatim, got: {output}"
        );
    }
}
