//! Function Signature Generation
//!
//! This module handles generation of function signatures including:
//! - Return type inference and generation
//! - Visibility tokens
//! - Type parameter handling
//! - Attribute processing
//!
//! **EXTREME TDD Round 70**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Attribute, Expr, ExprKind, Type};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;

impl Transpiler {
    /// Generate return type tokens based on function analysis
    /// Complexity: 10 (at Toyota Way limit)
    pub(crate) fn generate_return_type_tokens_impl(
        &self,
        name: &str,
        return_type: Option<&Type>,
        body: &Expr,
        params: &[crate::frontend::ast::Param],
    ) -> Result<TokenStream> {
        use super::return_type_helpers::{
            returns_boolean, returns_object_literal, returns_string, returns_string_literal,
            returns_vec,
        };
        use super::type_inference::infer_return_type_from_builtin_call;

        if let Some(ty) = return_type {
            // BOOK-COMPAT-017: Handle &str return type without input borrows
            // When return type is &str and there are no reference params, use 'static
            use crate::frontend::ast::TypeKind;
            if let TypeKind::Reference { inner, is_mut, .. } = &ty.kind {
                if let TypeKind::Named(inner_name) = &inner.kind {
                    if inner_name == "str" {
                        // Check if any params are references
                        let has_ref_params = params
                            .iter()
                            .any(|p| matches!(&p.ty.kind, TypeKind::Reference { .. }));
                        if !has_ref_params && !is_mut {
                            // No input refs, use 'static for string literals
                            return Ok(quote! { -> &'static str });
                        }
                    }
                }
            }
            let ty_tokens = self.transpile_type(ty)?;
            return Ok(quote! { -> #ty_tokens });
        }

        if name == "main" {
            return Ok(quote! {});
        }

        if super::function_analysis::returns_closure(body) {
            return Ok(quote! { -> impl Fn(i32) -> i32 });
        }

        if let Some(return_ty) = infer_return_type_from_builtin_call(body) {
            return self.generate_builtin_return_type(return_ty);
        }

        if returns_boolean(body) {
            return Ok(quote! { -> bool });
        }

        if returns_vec(body) {
            return Ok(quote! { -> Vec<i32> });
        }

        if returns_string(body) {
            return Ok(quote! { -> String });
        }

        // BOOK-COMPAT-017: Check call-site types for numeric return type
        // Check if function has float arguments - if so, infer f64 return type
        let has_float_args = self.call_site_arg_types.borrow().get(name).is_some_and(|call_types| {
            !call_types.is_empty() && call_types.iter().all(|t| t == "f64")
        });
        if has_float_args && super::function_analysis::has_non_unit_expression(body) {
            return Ok(quote! { -> f64 });
        }
        if super::function_analysis::looks_like_numeric_function(name) {
            if has_float_args {
                return Ok(quote! { -> f64 });
            }
            return Ok(quote! { -> i32 });
        }

        if returns_string_literal(body) {
            return Ok(quote! { -> &'static str });
        }

        if returns_object_literal(body) {
            return Ok(quote! { -> std::collections::BTreeMap<String, String> });
        }

        // BOOK-COMPAT-017: If body returns a parameter and we have call-site types, use those
        // This MUST come before infer_return_type_from_params_impl because untyped params return `_`
        if let Some(return_ty) = self.infer_return_from_call_site(name, body, params) {
            return Ok(return_ty);
        }

        if let Some(return_ty) = self.infer_return_type_from_params_impl(body, params)? {
            return Ok(return_ty);
        }

        if super::function_analysis::has_non_unit_expression(body) {
            return Ok(quote! { -> i32 });
        }

        Ok(quote! {})
    }

    /// Generate return type tokens for builtin function calls
    fn generate_builtin_return_type(&self, return_ty: &str) -> Result<TokenStream> {
        match return_ty {
            "String" => {
                let string_ident = format_ident!("String");
                Ok(quote! { -> #string_ident })
            }
            "Vec<String>" => {
                let vec_ident = format_ident!("Vec");
                let string_ident = format_ident!("String");
                Ok(quote! { -> #vec_ident<#string_ident> })
            }
            "bool" => Ok(quote! { -> bool }),
            "()" => Ok(quote! {}),
            // BOOK-COMPAT-011: DataFrame returns HashMap<String, Vec<String>>
            "std::collections::HashMap<String, Vec<String>>" => {
                Ok(quote! { -> std::collections::HashMap<String, Vec<String>> })
            }
            _ => Ok(quote! { -> i32 }),
        }
    }

    /// BOOK-COMPAT-017: Infer return type from call-site types when body returns a parameter
    fn infer_return_from_call_site(
        &self,
        func_name: &str,
        body: &Expr,
        params: &[crate::frontend::ast::Param],
    ) -> Option<TokenStream> {
        // Check if body returns a parameter identifier
        let returned_param = Self::get_returned_param_name(body)?;

        // Find the parameter index
        let param_index = params.iter().position(|p| p.name() == returned_param)?;

        // Get call-site type for this parameter
        let call_types = self.call_site_arg_types.borrow();
        let arg_types = call_types.get(func_name)?;
        let arg_type = arg_types.get(param_index)?;

        // Generate the return type based on call-site type
        match arg_type.as_str() {
            t if t.starts_with("Vec<") => {
                // Parse Vec<T> and generate proper type
                let inner = &t[4..t.len() - 1];
                let inner_ident = format_ident!("{}", inner);
                Some(quote! { -> Vec<#inner_ident> })
            }
            "String" => Some(quote! { -> String }),
            "f64" => Some(quote! { -> f64 }),
            "i32" => Some(quote! { -> i32 }),
            "bool" => Some(quote! { -> bool }),
            _ => None,
        }
    }

    /// Helper to get the name of the parameter being returned from the function body
    fn get_returned_param_name(body: &Expr) -> Option<String> {
        match &body.kind {
            ExprKind::Identifier(name) => Some(name.clone()),
            ExprKind::Block(exprs) => {
                // Get the last expression in the block
                exprs.last().and_then(Self::get_returned_param_name)
            }
            _ => None,
        }
    }

    /// Check if an expression references any global variables
    pub(crate) fn references_globals_impl(&self, expr: &Expr) -> bool {
        let globals = self
            .global_vars
            .read()
            .expect("RwLock poisoned: global_vars lock is corrupted");
        if globals.is_empty() {
            return false;
        }
        Self::expr_references_any_impl(expr, &globals)
    }

    /// Recursively check if expression references any of the given names
    pub(crate) fn expr_references_any_impl(expr: &Expr, names: &HashSet<String>) -> bool {
        match &expr.kind {
            ExprKind::Identifier(name) => names.contains(name),
            ExprKind::Assign { target, value } => {
                Self::expr_references_any_impl(target, names)
                    || Self::expr_references_any_impl(value, names)
            }
            ExprKind::Binary { left, right, .. } => {
                Self::expr_references_any_impl(left, names)
                    || Self::expr_references_any_impl(right, names)
            }
            ExprKind::Block(exprs) => exprs
                .iter()
                .any(|e| Self::expr_references_any_impl(e, names)),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::expr_references_any_impl(condition, names)
                    || Self::expr_references_any_impl(then_branch, names)
                    || else_branch
                        .as_ref()
                        .is_some_and(|e| Self::expr_references_any_impl(e, names))
            }
            ExprKind::Call { func, args } => {
                Self::expr_references_any_impl(func, names)
                    || args
                        .iter()
                        .any(|a| Self::expr_references_any_impl(a, names))
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                Self::expr_references_any_impl(receiver, names)
                    || args
                        .iter()
                        .any(|a| Self::expr_references_any_impl(a, names))
            }
            ExprKind::List(elements) | ExprKind::Tuple(elements) => elements
                .iter()
                .any(|e| Self::expr_references_any_impl(e, names)),
            ExprKind::Set(elements) => elements
                .iter()
                .any(|e| Self::expr_references_any_impl(e, names)),
            _ => false,
        }
    }

    /// Generate type parameter tokens with trait bound support
    /// Complexity: 5 (within Toyota Way limits)
    pub(crate) fn generate_type_param_tokens_impl(
        &self,
        type_params: &[String],
    ) -> Result<Vec<TokenStream>> {
        use proc_macro2::Span;
        use syn::Lifetime;
        Ok(type_params
            .iter()
            .map(|p| {
                if p.starts_with('\'') {
                    let lifetime = Lifetime::new(p, Span::call_site());
                    quote! { #lifetime }
                } else if p.contains(':') {
                    syn::parse_str::<syn::TypeParam>(p).map_or_else(
                        |_| {
                            let name = p.split(':').next().unwrap_or(p).trim();
                            let ident = format_ident!("{}", name);
                            quote! { #ident }
                        },
                        |tp| quote! { #tp },
                    )
                } else {
                    let ident = format_ident!("{}", p);
                    quote! { #ident }
                }
            })
            .collect())
    }

    /// Generate visibility token
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn generate_visibility_token_impl(&self, is_pub: bool) -> TokenStream {
        if is_pub {
            quote! { pub }
        } else {
            quote! {}
        }
    }

    /// Process attributes into regular attributes and modifiers
    /// Complexity: 4 (within Toyota Way limits)
    pub(crate) fn process_attributes_impl(
        &self,
        attributes: &[Attribute],
    ) -> (Vec<TokenStream>, TokenStream) {
        let mut regular_attrs = Vec::new();
        let mut modifiers = Vec::new();

        for attr in attributes {
            match attr.name.as_str() {
                "unsafe" => modifiers.push(quote! { unsafe }),
                "const" => modifiers.push(quote! { const }),
                _ => {
                    regular_attrs.push(self.format_regular_attribute_impl(attr));
                }
            }
        }

        let modifiers_tokens = if modifiers.is_empty() {
            quote! {}
        } else {
            quote! { #(#modifiers)* }
        };

        (regular_attrs, modifiers_tokens)
    }

    /// Format a regular attribute (non-modifier)
    /// Complexity: 4 (within Toyota Way limits)
    pub(crate) fn format_regular_attribute_impl(&self, attr: &Attribute) -> TokenStream {
        let attr_name = format_ident!("{}", attr.name);
        if attr.args.is_empty() {
            quote! { #[#attr_name] }
        } else {
            let args_str = attr.args.join(", ");
            if let Ok(args_tokens) = args_str.parse::<TokenStream>() {
                quote! { #[#attr_name(#args_tokens)] }
            } else {
                quote! { #[#attr_name] }
            }
        }
    }

    /// Compute final return type (handles special cases)
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn compute_final_return_type_impl(
        &self,
        _fn_name: &proc_macro2::Ident,
        return_type_tokens: &TokenStream,
    ) -> TokenStream {
        return_type_tokens.clone()
    }

    /// Generate complete function signature
    /// Complexity: 2 (within Toyota Way limits)
    pub(crate) fn generate_function_signature_impl(
        &self,
        is_pub: bool,
        is_async: bool,
        fn_name: &proc_macro2::Ident,
        type_param_tokens: &[TokenStream],
        param_tokens: &[TokenStream],
        return_type_tokens: &TokenStream,
        body_tokens: &TokenStream,
        attributes: &[Attribute],
    ) -> Result<TokenStream> {
        let final_return_type = self.compute_final_return_type_impl(fn_name, return_type_tokens);
        let visibility = self.generate_visibility_token_impl(is_pub);
        let (regular_attrs, modifiers_tokens) = self.process_attributes_impl(attributes);

        self.generate_function_declaration_impl(
            is_async,
            type_param_tokens,
            &regular_attrs,
            &visibility,
            &modifiers_tokens,
            fn_name,
            param_tokens,
            &final_return_type,
            body_tokens,
        )
    }

    /// Generate function declaration
    /// Complexity: 3 (within Toyota Way limits)
    pub(crate) fn generate_function_declaration_impl(
        &self,
        is_async: bool,
        type_param_tokens: &[TokenStream],
        regular_attrs: &[TokenStream],
        visibility: &TokenStream,
        modifiers_tokens: &TokenStream,
        fn_name: &proc_macro2::Ident,
        param_tokens: &[TokenStream],
        final_return_type: &TokenStream,
        body_tokens: &TokenStream,
    ) -> Result<TokenStream> {
        let async_keyword = if is_async {
            quote! { async }
        } else {
            quote! {}
        };
        let type_params = if type_param_tokens.is_empty() {
            quote! {}
        } else {
            quote! { <#(#type_param_tokens),*> }
        };

        Ok(quote! {
            #(#regular_attrs)*
            #visibility #modifiers_tokens #async_keyword fn #fn_name #type_params(#(#param_tokens),*) #final_return_type {
                #body_tokens
            }
        })
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

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // generate_return_type_tokens_impl tests
    // ========================================================================

    #[test]
    fn test_return_type_explicit() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let ty = Type {
            kind: crate::frontend::ast::TypeKind::Named("String".to_string()),
            span: Span::default(),
        };
        let result = transpiler
            .generate_return_type_tokens_impl("foo", Some(&ty), &body, &[])
            .unwrap();
        assert!(result.to_string().contains("String"));
    }

    #[test]
    fn test_return_type_main_no_type() {
        let transpiler = make_transpiler();
        let body = int_expr(0);
        let result = transpiler
            .generate_return_type_tokens_impl("main", None, &body, &[])
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_return_type_numeric_function() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler
            .generate_return_type_tokens_impl("calculate_sum", None, &body, &[])
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("i32") || result_str.is_empty());
    }

    #[test]
    fn test_return_type_string_literal() {
        let transpiler = make_transpiler();
        let body = string_expr("hello");
        let result = transpiler
            .generate_return_type_tokens_impl("get_greeting", None, &body, &[])
            .unwrap();
        assert!(result.to_string().contains("str"));
    }

    // ========================================================================
    // references_globals_impl tests
    // ========================================================================

    #[test]
    fn test_references_globals_empty() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        assert!(!transpiler.references_globals_impl(&body));
    }

    #[test]
    fn test_references_globals_with_global() {
        let transpiler = make_transpiler();
        {
            let mut globals = transpiler.global_vars.write().unwrap();
            globals.insert("GLOBAL".to_string());
        }
        let body = ident_expr("GLOBAL");
        assert!(transpiler.references_globals_impl(&body));
    }

    #[test]
    fn test_references_globals_no_match() {
        let transpiler = make_transpiler();
        {
            let mut globals = transpiler.global_vars.write().unwrap();
            globals.insert("GLOBAL".to_string());
        }
        let body = ident_expr("local");
        assert!(!transpiler.references_globals_impl(&body));
    }

    // ========================================================================
    // expr_references_any_impl tests
    // ========================================================================

    #[test]
    fn test_expr_references_any_identifier() {
        let names: HashSet<String> = ["x", "y"].iter().map(|s| s.to_string()).collect();
        let expr = ident_expr("x");
        assert!(Transpiler::expr_references_any_impl(&expr, &names));
    }

    #[test]
    fn test_expr_references_any_not_found() {
        let names: HashSet<String> = ["x", "y"].iter().map(|s| s.to_string()).collect();
        let expr = ident_expr("z");
        assert!(!Transpiler::expr_references_any_impl(&expr, &names));
    }

    #[test]
    fn test_expr_references_any_in_binary() {
        let names: HashSet<String> = ["x"].iter().map(|s| s.to_string()).collect();
        let expr = make_expr(ExprKind::Binary {
            left: Box::new(ident_expr("x")),
            op: crate::frontend::ast::BinaryOp::Add,
            right: Box::new(int_expr(1)),
        });
        assert!(Transpiler::expr_references_any_impl(&expr, &names));
    }

    #[test]
    fn test_expr_references_any_in_block() {
        let names: HashSet<String> = ["target"].iter().map(|s| s.to_string()).collect();
        let expr = make_expr(ExprKind::Block(vec![int_expr(1), ident_expr("target")]));
        assert!(Transpiler::expr_references_any_impl(&expr, &names));
    }

    // ========================================================================
    // generate_type_param_tokens_impl tests
    // ========================================================================

    #[test]
    fn test_type_param_simple() {
        let transpiler = make_transpiler();
        let result = transpiler
            .generate_type_param_tokens_impl(&["T".to_string()])
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "T");
    }

    #[test]
    fn test_type_param_lifetime() {
        let transpiler = make_transpiler();
        let result = transpiler
            .generate_type_param_tokens_impl(&["'a".to_string()])
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("'a"));
    }

    #[test]
    fn test_type_param_with_bound() {
        let transpiler = make_transpiler();
        let result = transpiler
            .generate_type_param_tokens_impl(&["T: Clone".to_string()])
            .unwrap();
        assert_eq!(result.len(), 1);
        let result_str = result[0].to_string();
        assert!(result_str.contains("T") || result_str.contains("Clone"));
    }

    #[test]
    fn test_type_param_multiple() {
        let transpiler = make_transpiler();
        let result = transpiler
            .generate_type_param_tokens_impl(&["T".to_string(), "U".to_string()])
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    // ========================================================================
    // generate_visibility_token_impl tests
    // ========================================================================

    #[test]
    fn test_visibility_public() {
        let transpiler = make_transpiler();
        let result = transpiler.generate_visibility_token_impl(true);
        assert_eq!(result.to_string(), "pub");
    }

    #[test]
    fn test_visibility_private() {
        let transpiler = make_transpiler();
        let result = transpiler.generate_visibility_token_impl(false);
        assert!(result.is_empty());
    }

    // ========================================================================
    // process_attributes_impl tests
    // ========================================================================

    #[test]
    fn test_process_attrs_empty() {
        let transpiler = make_transpiler();
        let (attrs, modifiers) = transpiler.process_attributes_impl(&[]);
        assert!(attrs.is_empty());
        assert!(modifiers.is_empty());
    }

    #[test]
    fn test_process_attrs_unsafe() {
        let transpiler = make_transpiler();
        let attr = Attribute {
            name: "unsafe".to_string(),
            args: vec![],
            span: Span::default(),
        };
        let (attrs, modifiers) = transpiler.process_attributes_impl(&[attr]);
        assert!(attrs.is_empty());
        assert!(modifiers.to_string().contains("unsafe"));
    }

    #[test]
    fn test_process_attrs_const() {
        let transpiler = make_transpiler();
        let attr = Attribute {
            name: "const".to_string(),
            args: vec![],
            span: Span::default(),
        };
        let (attrs, modifiers) = transpiler.process_attributes_impl(&[attr]);
        assert!(attrs.is_empty());
        assert!(modifiers.to_string().contains("const"));
    }

    #[test]
    fn test_process_attrs_regular() {
        let transpiler = make_transpiler();
        let attr = Attribute {
            name: "test".to_string(),
            args: vec![],
            span: Span::default(),
        };
        let (attrs, modifiers) = transpiler.process_attributes_impl(&[attr]);
        assert_eq!(attrs.len(), 1);
        assert!(attrs[0].to_string().contains("test"));
        assert!(modifiers.is_empty());
    }

    #[test]
    fn test_process_attrs_with_args() {
        let transpiler = make_transpiler();
        let attr = Attribute {
            name: "derive".to_string(),
            args: vec!["Clone".to_string(), "Debug".to_string()],
            span: Span::default(),
        };
        let (attrs, _) = transpiler.process_attributes_impl(&[attr]);
        assert_eq!(attrs.len(), 1);
        let attr_str = attrs[0].to_string();
        assert!(attr_str.contains("derive"));
    }

    // ========================================================================
    // generate_function_declaration_impl tests
    // ========================================================================

    #[test]
    fn test_generate_declaration_simple() {
        let transpiler = make_transpiler();
        let fn_name = format_ident!("foo");
        let result = transpiler
            .generate_function_declaration_impl(
                false,
                &[],
                &[],
                &quote! {},
                &quote! {},
                &fn_name,
                &[],
                &quote! { -> i32 },
                &quote! { 42 },
            )
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("fn foo"));
        assert!(result_str.contains("-> i32"));
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_generate_declaration_async() {
        let transpiler = make_transpiler();
        let fn_name = format_ident!("async_fn");
        let result = transpiler
            .generate_function_declaration_impl(
                true,
                &[],
                &[],
                &quote! {},
                &quote! {},
                &fn_name,
                &[],
                &quote! {},
                &quote! {},
            )
            .unwrap();
        assert!(result.to_string().contains("async fn"));
    }

    #[test]
    fn test_generate_declaration_with_type_params() {
        let transpiler = make_transpiler();
        let fn_name = format_ident!("generic");
        let type_params = vec![quote! { T }];
        let result = transpiler
            .generate_function_declaration_impl(
                false,
                &type_params,
                &[],
                &quote! {},
                &quote! {},
                &fn_name,
                &[],
                &quote! {},
                &quote! {},
            )
            .unwrap();
        assert!(result.to_string().contains("< T >"));
    }

    #[test]
    fn test_generate_declaration_public() {
        let transpiler = make_transpiler();
        let fn_name = format_ident!("public_fn");
        let result = transpiler
            .generate_function_declaration_impl(
                false,
                &[],
                &[],
                &quote! { pub },
                &quote! {},
                &fn_name,
                &[],
                &quote! {},
                &quote! {},
            )
            .unwrap();
        assert!(result.to_string().contains("pub"));
    }

    #[test]
    fn test_generate_declaration_with_attrs() {
        let transpiler = make_transpiler();
        let fn_name = format_ident!("test_fn");
        let attrs = vec![quote! { #[test] }];
        let result = transpiler
            .generate_function_declaration_impl(
                false,
                &[],
                &attrs,
                &quote! {},
                &quote! {},
                &fn_name,
                &[],
                &quote! {},
                &quote! {},
            )
            .unwrap();
        assert!(result.to_string().contains("[test]"));
    }
}
