//! Type Transpilation Helpers
//!
//! This module provides helper functions for transpiling type annotations
//! including named types, generics, optionals, lists, arrays, and tuples.
//!
//! **EXTREME TDD Round 75**: Extracted from types.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Type, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Lifetime;

impl Transpiler {
    /// Transpile named types with built-in type mapping
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn transpile_named_type_impl(&self, name: &str) -> Result<TokenStream> {
        match name {
            "int" => Ok(quote! { i64 }),
            "float" => Ok(quote! { f64 }),
            "bool" => Ok(quote! { bool }),
            "str" => Ok(quote! { &str }),
            "string" | "String" => Ok(quote! { String }),
            "char" => Ok(quote! { char }),
            "()" => Ok(quote! { () }),
            "_" | "Any" => Ok(quote! { _ }),
            "Object" => Ok(quote! { std::collections::BTreeMap<String, String> }),
            _ => self.transpile_complex_named_type(name),
        }
    }

    /// Transpile complex named types (namespaced or custom)
    fn transpile_complex_named_type(&self, name: &str) -> Result<TokenStream> {
        if name.contains("::") {
            self.transpile_namespaced_type(name)
        } else {
            let ident = format_ident!("{}", name);
            Ok(quote! { #ident })
        }
    }

    /// Transpile namespaced types (e.g., std::io::Error)
    fn transpile_namespaced_type(&self, name: &str) -> Result<TokenStream> {
        let segments: Vec<_> = name
            .split("::")
            .map(|seg| format_ident!("{}", seg))
            .collect();
        Ok(quote! { #(#segments)::* })
    }

    /// Transpile generic types (e.g., Vec<T>, HashMap<K, V>)
    /// Complexity: 4 (within Toyota Way limits)
    pub(crate) fn transpile_generic_type_impl(
        &self,
        base: &str,
        params: &[Type],
    ) -> Result<TokenStream> {
        let base_ident = format_ident!("{}", base);
        let param_tokens: Result<Vec<_>> = params.iter().map(|p| self.transpile_type(p)).collect();
        let param_tokens = param_tokens?;
        Ok(quote! { #base_ident<#(#param_tokens),*> })
    }

    /// Transpile optional types (T? -> Option<T>)
    /// Complexity: 2 (within Toyota Way limits)
    pub(crate) fn transpile_optional_type_impl(&self, inner: &Type) -> Result<TokenStream> {
        let inner_tokens = self.transpile_type(inner)?;
        Ok(quote! { Option<#inner_tokens> })
    }

    /// Transpile list types ([T] -> Vec<T>)
    /// Complexity: 2 (within Toyota Way limits)
    pub(crate) fn transpile_list_type_impl(&self, elem_type: &Type) -> Result<TokenStream> {
        let elem_tokens = self.transpile_type(elem_type)?;
        Ok(quote! { Vec<#elem_tokens> })
    }

    /// Transpile array types ([T; N])
    /// Complexity: 2 (within Toyota Way limits)
    pub(crate) fn transpile_array_type_impl(
        &self,
        elem_type: &Type,
        size: usize,
    ) -> Result<TokenStream> {
        let elem_tokens = self.transpile_type(elem_type)?;
        Ok(quote! { [#elem_tokens; #size] })
    }

    /// Transpile tuple types ((A, B, C))
    /// Complexity: 3 (within Toyota Way limits)
    pub(crate) fn transpile_tuple_type_impl(&self, types: &[Type]) -> Result<TokenStream> {
        let type_tokens: Result<Vec<_>> = types.iter().map(|t| self.transpile_type(t)).collect();
        let type_tokens = type_tokens?;
        Ok(quote! { (#(#type_tokens),*) })
    }

    /// Transpile function types (fn(A, B) -> C)
    /// Complexity: 4 (within Toyota Way limits)
    pub(crate) fn transpile_function_type_impl(
        &self,
        params: &[Type],
        ret: &Type,
    ) -> Result<TokenStream> {
        let param_tokens: Result<Vec<_>> = params.iter().map(|p| self.transpile_type(p)).collect();
        let param_tokens = param_tokens?;
        let ret_tokens = self.transpile_type(ret)?;
        Ok(quote! { fn(#(#param_tokens),*) -> #ret_tokens })
    }

    /// Transpile reference types (&T, &mut T, &'a T)
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn transpile_reference_type_impl(
        &self,
        is_mut: bool,
        lifetime: Option<&str>,
        inner: &Type,
    ) -> Result<TokenStream> {
        let inner_tokens = self.transpile_type(inner)?;

        let lifetime_token = lifetime.map(|lt| {
            let lifetime = Lifetime::new(lt, proc_macro2::Span::call_site());
            quote! { #lifetime }
        });

        match (is_mut, lifetime_token) {
            (true, Some(lt)) => Ok(quote! { &#lt mut #inner_tokens }),
            (true, None) => Ok(quote! { &mut #inner_tokens }),
            (false, Some(lt)) => Ok(quote! { &#lt #inner_tokens }),
            (false, None) => Ok(quote! { &#inner_tokens }),
        }
    }

    /// Parse type parameter string to TokenStream
    /// Handles simple params ("T") and params with bounds ("T: Clone + Debug")
    /// Complexity: 5 (within Toyota Way limits)
    pub(crate) fn parse_type_param_to_tokens_impl(p: &str) -> TokenStream {
        if p.starts_with('\'') {
            let lifetime = Lifetime::new(p, proc_macro2::Span::call_site());
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
    }

    /// Check if type params contain a lifetime
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn has_lifetime_params_impl(type_params: &[String]) -> bool {
        type_params.iter().any(|p| p.starts_with('\''))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;

    fn make_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn make_type(kind: TypeKind) -> Type {
        Type {
            kind,
            span: Span::default(),
        }
    }

    // ========================================================================
    // transpile_named_type_impl tests
    // ========================================================================

    #[test]
    fn test_named_type_int() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("int").unwrap();
        assert!(result.to_string().contains("i64"));
    }

    #[test]
    fn test_named_type_float() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("float").unwrap();
        assert!(result.to_string().contains("f64"));
    }

    #[test]
    fn test_named_type_bool() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("bool").unwrap();
        assert!(result.to_string().contains("bool"));
    }

    #[test]
    fn test_named_type_str() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("str").unwrap();
        assert!(result.to_string().contains("& str"));
    }

    #[test]
    fn test_named_type_string() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("String").unwrap();
        assert!(result.to_string().contains("String"));
    }

    #[test]
    fn test_named_type_custom() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("MyStruct").unwrap();
        assert!(result.to_string().contains("MyStruct"));
    }

    #[test]
    fn test_named_type_namespaced() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_named_type_impl("std::io::Error").unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("std"));
        assert!(token_str.contains("io"));
        assert!(token_str.contains("Error"));
    }

    // ========================================================================
    // transpile_generic_type_impl tests
    // ========================================================================

    #[test]
    fn test_generic_single_param() {
        let transpiler = make_transpiler();
        let params = vec![make_type(TypeKind::Named("i32".to_string()))];
        let result = transpiler.transpile_generic_type_impl("Vec", &params).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("Vec"));
        assert!(token_str.contains("i32"));
    }

    #[test]
    fn test_generic_multiple_params() {
        let transpiler = make_transpiler();
        let params = vec![
            make_type(TypeKind::Named("String".to_string())),
            make_type(TypeKind::Named("i32".to_string())),
        ];
        let result = transpiler.transpile_generic_type_impl("HashMap", &params).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("HashMap"));
        assert!(token_str.contains("String"));
        assert!(token_str.contains("i32"));
    }

    // ========================================================================
    // transpile_optional_type_impl tests
    // ========================================================================

    #[test]
    fn test_optional_type() {
        let transpiler = make_transpiler();
        let inner = make_type(TypeKind::Named("i32".to_string()));
        let result = transpiler.transpile_optional_type_impl(&inner).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("Option"));
        assert!(token_str.contains("i32"));
    }

    // ========================================================================
    // transpile_list_type_impl tests
    // ========================================================================

    #[test]
    fn test_list_type() {
        let transpiler = make_transpiler();
        let elem = make_type(TypeKind::Named("String".to_string()));
        let result = transpiler.transpile_list_type_impl(&elem).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("Vec"));
        assert!(token_str.contains("String"));
    }

    // ========================================================================
    // transpile_array_type_impl tests
    // ========================================================================

    #[test]
    fn test_array_type() {
        let transpiler = make_transpiler();
        let elem = make_type(TypeKind::Named("i32".to_string()));
        let result = transpiler.transpile_array_type_impl(&elem, 10).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("i32"));
        assert!(token_str.contains("10"));
    }

    // ========================================================================
    // transpile_tuple_type_impl tests
    // ========================================================================

    #[test]
    fn test_tuple_type() {
        let transpiler = make_transpiler();
        let types = vec![
            make_type(TypeKind::Named("i32".to_string())),
            make_type(TypeKind::Named("String".to_string())),
        ];
        let result = transpiler.transpile_tuple_type_impl(&types).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("i32"));
        assert!(token_str.contains("String"));
    }

    // ========================================================================
    // transpile_function_type_impl tests
    // ========================================================================

    #[test]
    fn test_function_type() {
        let transpiler = make_transpiler();
        let params = vec![make_type(TypeKind::Named("i32".to_string()))];
        let ret = make_type(TypeKind::Named("bool".to_string()));
        let result = transpiler.transpile_function_type_impl(&params, &ret).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("fn"));
        assert!(token_str.contains("i32"));
        assert!(token_str.contains("bool"));
    }

    // ========================================================================
    // transpile_reference_type_impl tests
    // ========================================================================

    #[test]
    fn test_reference_immutable() {
        let transpiler = make_transpiler();
        let inner = make_type(TypeKind::Named("i32".to_string()));
        let result = transpiler.transpile_reference_type_impl(false, None, &inner).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("&"));
        assert!(token_str.contains("i32"));
        assert!(!token_str.contains("mut"));
    }

    #[test]
    fn test_reference_mutable() {
        let transpiler = make_transpiler();
        let inner = make_type(TypeKind::Named("i32".to_string()));
        let result = transpiler.transpile_reference_type_impl(true, None, &inner).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("&"));
        assert!(token_str.contains("mut"));
    }

    #[test]
    fn test_reference_with_lifetime() {
        let transpiler = make_transpiler();
        let inner = make_type(TypeKind::Named("str".to_string()));
        let result = transpiler.transpile_reference_type_impl(false, Some("'a"), &inner).unwrap();
        let token_str = result.to_string();
        assert!(token_str.contains("'a"));
    }

    // ========================================================================
    // parse_type_param_to_tokens_impl tests
    // ========================================================================

    #[test]
    fn test_parse_simple_type_param() {
        let result = Transpiler::parse_type_param_to_tokens_impl("T");
        assert!(result.to_string().contains("T"));
    }

    #[test]
    fn test_parse_lifetime_param() {
        let result = Transpiler::parse_type_param_to_tokens_impl("'a");
        assert!(result.to_string().contains("'a"));
    }

    #[test]
    fn test_parse_bounded_type_param() {
        let result = Transpiler::parse_type_param_to_tokens_impl("T: Clone");
        let token_str = result.to_string();
        assert!(token_str.contains("T"));
        assert!(token_str.contains("Clone"));
    }

    // ========================================================================
    // has_lifetime_params_impl tests
    // ========================================================================

    #[test]
    fn test_has_lifetime_true() {
        let params = vec!["'a".to_string(), "T".to_string()];
        assert!(Transpiler::has_lifetime_params_impl(&params));
    }

    #[test]
    fn test_has_lifetime_false() {
        let params = vec!["T".to_string(), "U".to_string()];
        assert!(!Transpiler::has_lifetime_params_impl(&params));
    }
}
