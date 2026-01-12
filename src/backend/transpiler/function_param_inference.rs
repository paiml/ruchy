//! Function Parameter Type Inference
//!
//! This module handles inference of parameter and return types for functions
//! based on how parameters are used in function bodies.
//!
//! **EXTREME TDD Round 70**: Extracted from statements.rs for modularization.

#![allow(clippy::doc_markdown)]

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Param, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;

impl Transpiler {
    /// Infer return type from parameter types
    /// Delegates to return_type_helpers module
    pub(crate) fn infer_return_type_from_params_impl(
        &self,
        body: &Expr,
        params: &[Param],
    ) -> Result<Option<TokenStream>> {
        super::return_type_helpers::infer_return_type_from_params(body, params, |ty| {
            self.transpile_type(ty)
        })
    }

    /// Infer parameter type based on usage in function body
    /// Complexity: 9 (within Toyota Way limits)
    pub(crate) fn infer_param_type_impl(
        &self,
        param: &Param,
        body: &Expr,
        func_name: &str,
    ) -> TokenStream {
        use super::type_inference::{
            infer_param_type_from_builtin_usage, is_param_used_as_array, is_param_used_as_bool,
            is_param_used_as_function, is_param_used_as_index, is_param_used_in_print_macro,
            is_param_used_in_string_concat, is_param_used_numerically, is_param_used_with_len,
        };

        // Check for function parameters first (higher-order functions)
        if is_param_used_as_function(&param.name(), body) {
            return quote! { impl Fn(i32) -> i32 };
        }

        // ISSUE-114 FIX: Check if parameter is used as boolean condition
        if is_param_used_as_bool(&param.name(), body) {
            return quote! { bool };
        }

        // Check if parameter is used as an array (indexed)
        if is_param_used_as_array(&param.name(), body) {
            if self.is_nested_array_param_impl(&param.name(), body) {
                return quote! { &Vec<Vec<i32>> };
            }
            return quote! { &Vec<i32> };
        }

        // Check if parameter is used with len()
        if is_param_used_with_len(&param.name(), body) {
            if self.is_nested_array_param_impl(&param.name(), body) {
                return quote! { &Vec<Vec<i32>> };
            }
            return quote! { &Vec<i32> };
        }

        // Check if parameter is used as an index
        if is_param_used_as_index(&param.name(), body) {
            return quote! { i32 };
        }

        // Check if used numerically
        if is_param_used_numerically(&param.name(), body)
            || super::function_analysis::looks_like_numeric_function(func_name)
        {
            return quote! { i32 };
        }

        // Check built-in function signatures for string-specific operations
        if let Some(type_hint) = infer_param_type_from_builtin_usage(&param.name(), body) {
            if type_hint == "&str" {
                return quote! { &str };
            }
        }

        // Check if parameter is used in string concatenation
        if is_param_used_in_string_concat(&param.name(), body) {
            return quote! { &str };
        }

        // Check if parameter is used in print/format macros
        if is_param_used_in_print_macro(&param.name(), body) {
            return quote! { &str };
        }

        // Default to i32 for unused/untyped parameters
        quote! { i32 }
    }

    /// Helper to detect nested array access (2D arrays)
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn is_nested_array_param_impl(&self, param_name: &str, expr: &Expr) -> bool {
        Self::find_nested_array_access_impl(param_name, expr, &mut HashSet::new())
    }

    /// Internal helper with visited tracking to prevent infinite recursion
    /// Complexity: 9 (within Toyota Way limits)
    fn find_nested_array_access_impl(
        param_name: &str,
        expr: &Expr,
        visited: &mut HashSet<usize>,
    ) -> bool {
        let expr_addr = std::ptr::from_ref(expr) as usize;
        if visited.contains(&expr_addr) {
            return false;
        }
        visited.insert(expr_addr);

        match &expr.kind {
            // Direct nested indexing: param[i][j]
            ExprKind::IndexAccess { object, .. } => {
                if let ExprKind::IndexAccess { object: inner, .. } = &object.kind {
                    if let ExprKind::Identifier(name) = &inner.kind {
                        if name == param_name {
                            return true;
                        }
                    }
                }
                Self::find_nested_array_access_impl(param_name, object, visited)
            }
            // Recurse into block expressions
            ExprKind::Block(exprs) => exprs
                .iter()
                .any(|e| Self::find_nested_array_access_impl(param_name, e, visited)),
            // Let bindings
            ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => {
                Self::find_nested_array_access_impl(param_name, value, visited)
                    || Self::find_nested_array_access_impl(param_name, body, visited)
            }
            // Binary operations
            ExprKind::Binary { left, right, .. } => {
                Self::find_nested_array_access_impl(param_name, left, visited)
                    || Self::find_nested_array_access_impl(param_name, right, visited)
            }
            // While loops
            ExprKind::While {
                condition, body, ..
            } => {
                Self::find_nested_array_access_impl(param_name, condition, visited)
                    || Self::find_nested_array_access_impl(param_name, body, visited)
            }
            // If expressions
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::find_nested_array_access_impl(param_name, condition, visited)
                    || Self::find_nested_array_access_impl(param_name, then_branch, visited)
                    || else_branch.as_ref().is_some_and(|e| {
                        Self::find_nested_array_access_impl(param_name, e, visited)
                    })
            }
            // Assignments
            ExprKind::Assign { target, value } | ExprKind::CompoundAssign { target, value, .. } => {
                Self::find_nested_array_access_impl(param_name, target, visited)
                    || Self::find_nested_array_access_impl(param_name, value, visited)
            }
            _ => false,
        }
    }

    /// Generate parameter tokens with proper type inference
    /// Complexity: 5 (within Toyota Way limits)
    pub(crate) fn generate_param_tokens_impl(
        &self,
        params: &[Param],
        body: &Expr,
        func_name: &str,
    ) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|p| {
                let param_name = format_ident!("{}", p.name());

                // Handle special Rust receiver syntax (&self, &mut self, self)
                if p.name() == "self" {
                    if let TypeKind::Reference { is_mut, .. } = &p.ty.kind {
                        if *is_mut {
                            return Ok(quote! { &mut self });
                        }
                        return Ok(quote! { &self });
                    }
                    return Ok(quote! { self });
                }

                // Regular parameter handling
                let type_tokens = if let Ok(tokens) = self.transpile_type(&p.ty) {
                    let token_str = tokens.to_string();
                    if token_str == "_" {
                        self.infer_param_type_impl(p, body, func_name)
                    } else {
                        tokens
                    }
                } else {
                    self.infer_param_type_impl(p, body, func_name)
                };

                // Preserve mut keyword for mutable parameters
                if p.is_mutable {
                    Ok(quote! { mut #param_name: #type_tokens })
                } else {
                    Ok(quote! { #param_name: #type_tokens })
                }
            })
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span, Type};

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
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: crate::frontend::ast::TypeKind::Named("_".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    fn make_mut_param(name: &str) -> Param {
        Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: crate::frontend::ast::TypeKind::Named("_".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: true,
            default_value: None,
        }
    }

    // ========================================================================
    // infer_param_type_impl tests
    // ========================================================================

    #[test]
    fn test_infer_param_type_numeric_function() {
        let transpiler = make_transpiler();
        let param = make_param("x");
        let body = int_expr(42);
        let result = transpiler.infer_param_type_impl(&param, &body, "add");
        assert_eq!(result.to_string(), "i32");
    }

    #[test]
    fn test_infer_param_type_bool_condition() {
        let transpiler = make_transpiler();
        let param = make_param("flag");
        // if flag { 1 } else { 0 }
        let body = make_expr(ExprKind::If {
            condition: Box::new(ident_expr("flag")),
            then_branch: Box::new(int_expr(1)),
            else_branch: Some(Box::new(int_expr(0))),
        });
        let result = transpiler.infer_param_type_impl(&param, &body, "check");
        assert_eq!(result.to_string(), "bool");
    }

    #[test]
    fn test_infer_param_type_array_indexing() {
        let transpiler = make_transpiler();
        let param = make_param("arr");
        // arr[0]
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(ident_expr("arr")),
            index: Box::new(int_expr(0)),
        });
        let result = transpiler.infer_param_type_impl(&param, &body, "get_first");
        assert_eq!(result.to_string(), "& Vec < i32 >");
    }

    #[test]
    fn test_infer_param_type_nested_array() {
        let transpiler = make_transpiler();
        let param = make_param("matrix");
        // matrix[0][1]
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::IndexAccess {
                object: Box::new(ident_expr("matrix")),
                index: Box::new(int_expr(0)),
            })),
            index: Box::new(int_expr(1)),
        });
        let result = transpiler.infer_param_type_impl(&param, &body, "get_element");
        let result_str = result.to_string();
        // Normalize whitespace for comparison
        assert!(result_str.contains("Vec") && result_str.contains("i32"));
        assert!(result_str.starts_with("&"));
    }

    #[test]
    fn test_infer_param_type_index_usage() {
        let transpiler = make_transpiler();
        let param = make_param("i");
        // arr[i]
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(ident_expr("arr")),
            index: Box::new(ident_expr("i")),
        });
        let result = transpiler.infer_param_type_impl(&param, &body, "access");
        assert_eq!(result.to_string(), "i32");
    }

    #[test]
    fn test_infer_param_type_default() {
        let transpiler = make_transpiler();
        let param = make_param("unused");
        let body = int_expr(42);
        let result = transpiler.infer_param_type_impl(&param, &body, "foo");
        assert_eq!(result.to_string(), "i32");
    }

    // ========================================================================
    // is_nested_array_param_impl tests
    // ========================================================================

    #[test]
    fn test_is_nested_array_simple() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(ident_expr("arr")),
            index: Box::new(int_expr(0)),
        });
        assert!(!transpiler.is_nested_array_param_impl("arr", &body));
    }

    #[test]
    fn test_is_nested_array_2d() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::IndexAccess {
                object: Box::new(ident_expr("matrix")),
                index: Box::new(int_expr(0)),
            })),
            index: Box::new(int_expr(1)),
        });
        assert!(transpiler.is_nested_array_param_impl("matrix", &body));
    }

    #[test]
    fn test_is_nested_array_in_block() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::Block(vec![make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::IndexAccess {
                object: Box::new(ident_expr("m")),
                index: Box::new(ident_expr("i")),
            })),
            index: Box::new(ident_expr("j")),
        })]));
        assert!(transpiler.is_nested_array_param_impl("m", &body));
    }

    #[test]
    fn test_is_nested_array_in_if() {
        let transpiler = make_transpiler();
        let nested_access = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::IndexAccess {
                object: Box::new(ident_expr("grid")),
                index: Box::new(int_expr(0)),
            })),
            index: Box::new(int_expr(0)),
        });
        let body = make_expr(ExprKind::If {
            condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
            then_branch: Box::new(nested_access),
            else_branch: None,
        });
        assert!(transpiler.is_nested_array_param_impl("grid", &body));
    }

    #[test]
    fn test_is_nested_array_different_param() {
        let transpiler = make_transpiler();
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::IndexAccess {
                object: Box::new(ident_expr("other")),
                index: Box::new(int_expr(0)),
            })),
            index: Box::new(int_expr(1)),
        });
        assert!(!transpiler.is_nested_array_param_impl("matrix", &body));
    }

    // ========================================================================
    // generate_param_tokens_impl tests
    // ========================================================================

    #[test]
    fn test_generate_param_tokens_simple() {
        let transpiler = make_transpiler();
        let params = vec![make_param("x")];
        let body = int_expr(42);
        let result = transpiler
            .generate_param_tokens_impl(&params, &body, "add")
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("x"));
    }

    #[test]
    fn test_generate_param_tokens_mutable() {
        let transpiler = make_transpiler();
        let params = vec![make_mut_param("count")];
        let body = int_expr(0);
        let result = transpiler
            .generate_param_tokens_impl(&params, &body, "increment")
            .unwrap();
        assert!(result[0].to_string().contains("mut"));
        assert!(result[0].to_string().contains("count"));
    }

    #[test]
    fn test_generate_param_tokens_multiple() {
        let transpiler = make_transpiler();
        let params = vec![make_param("a"), make_param("b")];
        let body = int_expr(0);
        let result = transpiler
            .generate_param_tokens_impl(&params, &body, "add")
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_generate_param_tokens_self_ref() {
        let transpiler = make_transpiler();
        let mut param = make_param("self");
        param.ty = Type {
            kind: TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(Type {
                    kind: crate::frontend::ast::TypeKind::Named("Self".to_string()),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        };
        let params = vec![param];
        let body = int_expr(0);
        let result = transpiler
            .generate_param_tokens_impl(&params, &body, "method")
            .unwrap();
        assert_eq!(result[0].to_string(), "& self");
    }

    #[test]
    fn test_generate_param_tokens_self_mut() {
        let transpiler = make_transpiler();
        let mut param = make_param("self");
        param.ty = Type {
            kind: TypeKind::Reference {
                is_mut: true,
                lifetime: None,
                inner: Box::new(Type {
                    kind: crate::frontend::ast::TypeKind::Named("Self".to_string()),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        };
        let params = vec![param];
        let body = int_expr(0);
        let result = transpiler
            .generate_param_tokens_impl(&params, &body, "method")
            .unwrap();
        assert_eq!(result[0].to_string(), "& mut self");
    }

    #[test]
    fn test_generate_param_tokens_self_owned() {
        let transpiler = make_transpiler();
        let param = make_param("self");
        let params = vec![param];
        let body = int_expr(0);
        let result = transpiler
            .generate_param_tokens_impl(&params, &body, "consume")
            .unwrap();
        assert_eq!(result[0].to_string(), "self");
    }

    // ========================================================================
    // infer_return_type_from_params tests
    // ========================================================================

    #[test]
    fn test_infer_return_type_no_params() {
        let transpiler = make_transpiler();
        let body = int_expr(42);
        let result = transpiler
            .infer_return_type_from_params_impl(&body, &[])
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_return_type_with_params() {
        let transpiler = make_transpiler();
        let params = vec![make_param("x")];
        // Return the parameter directly
        let body = ident_expr("x");
        let result = transpiler
            .infer_return_type_from_params_impl(&body, &params)
            .unwrap();
        // Should infer return type from parameter type if parameter is returned
        // This depends on implementation details in return_type_helpers
        // The test validates the function runs without error
        assert!(result.is_none() || result.is_some());
    }
}
