//! Call Transpilation Module
//!
//! This module handles transpilation of function calls and method calls
//! to Rust code, including special handling for built-in functions.
//!
//! **EXTREME TDD Round 72**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpile function calls
    /// Complexity: 10 (at Toyota Way limit)
    pub(crate) fn transpile_call_impl(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        // Handle main() calls → __ruchy_main()
        let func_tokens = self.transform_main_call(func)?;

        // Check for std::time::now_millis() path-based calls
        if let Some(tokens) = self.try_transpile_std_time_call(func, args)? {
            return Ok(tokens);
        }

        // Check if this is a built-in function with special handling
        if let ExprKind::Identifier(name) = &func.kind {
            if let Some(tokens) = self.try_transpile_builtin_call(&func_tokens, name, args)? {
                return Ok(tokens);
            }
        }

        // Default: regular function call with string conversion
        self.transpile_regular_function_call(&func_tokens, args)
    }

    /// Transform main() calls to __ruchy_main()
    fn transform_main_call(&self, func: &Expr) -> Result<TokenStream> {
        if let ExprKind::Identifier(name) = &func.kind {
            if name == "main" {
                let renamed_ident = format_ident!("__ruchy_main");
                return Ok(quote! { #renamed_ident });
            }
        }
        self.transpile_expr(func)
    }

    /// Try to transpile std::time::now_millis() calls
    fn try_transpile_std_time_call(
        &self,
        func: &Expr,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        if let ExprKind::FieldAccess { object, field } = &func.kind {
            if let ExprKind::FieldAccess {
                object: std_obj,
                field: module_name,
            } = &object.kind
            {
                if let ExprKind::Identifier(std_name) = &std_obj.kind {
                    if std_name == "std" && module_name == "time" && field == "now_millis" {
                        if !args.is_empty() {
                            bail!("std::time::now_millis() expects no arguments");
                        }
                        return Ok(Some(quote! {
                            {
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .expect("System time before Unix epoch")
                                    .as_millis() as i64
                            }
                        }));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Try to transpile built-in function calls
    fn try_transpile_builtin_call(
        &self,
        func_tokens: &TokenStream,
        name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        let base_name = if name.ends_with('!') {
            name.strip_suffix('!').unwrap_or(name)
        } else {
            name
        };

        // Try specialized handlers in order of precedence
        if let Some(result) = self.try_transpile_print_macro(func_tokens, base_name, args)? {
            return Ok(Some(result));
        }

        // len(x) → x.len()
        if base_name == "len" && args.len() == 1 {
            let arg_tokens = self.transpile_expr(&args[0])?;
            return Ok(Some(quote! { #arg_tokens.len() }));
        }

        // time_micros()
        if base_name == "time_micros" {
            if !args.is_empty() {
                bail!("time_micros() expects no arguments");
            }
            return Ok(Some(quote! {
                {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("System time before Unix epoch")
                        .as_micros() as u64
                }
            }));
        }

        // Try all the specialized builtin handlers
        let handlers: Vec<Box<dyn Fn(&str, &[Expr]) -> Result<Option<TokenStream>> + '_>> = vec![
            Box::new(|n, a| self.try_transpile_math_function(n, a)),
            Box::new(|n, a| self.try_transpile_input_function(n, a)),
            Box::new(|n, a| self.try_transpile_assert_function(func_tokens, n, a)),
            Box::new(|n, a| self.try_transpile_type_conversion(n, a)),
            Box::new(|n, a| self.try_transpile_math_functions(n, a)),
            Box::new(|n, a| self.try_transpile_time_functions(n, a)),
            Box::new(|n, a| self.try_transpile_collection_constructor(n, a)),
            Box::new(|n, a| self.try_transpile_range_function(n, a)),
            Box::new(|n, a| self.try_transpile_dataframe_function_impl(n, a)),
            Box::new(|n, a| self.try_transpile_environment_function(n, a)),
            Box::new(|n, a| self.try_transpile_fs_function(n, a)),
            Box::new(|n, a| self.try_transpile_path_function(n, a)),
            Box::new(|n, a| self.try_transpile_json_function(n, a)),
            Box::new(|n, a| self.try_transpile_http_function(n, a)),
            Box::new(|n, a| self.try_transpile_trueno_function(n, a)),
            Box::new(|n, a| self.try_transpile_result_call(n, a)),
        ];

        for handler in handlers {
            if let Some(result) = handler(base_name, args)? {
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    /// Try to transpile DataFrame function calls
    fn try_transpile_dataframe_function_impl(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        // Handle DataFrame static methods
        if base_name.starts_with("DataFrame::") {
            let method = base_name
                .strip_prefix("DataFrame::")
                .expect("Already checked starts_with");
            match method {
                "new" if args.is_empty() => {
                    return Ok(Some(quote! { polars::prelude::DataFrame::empty() }));
                }
                "from_csv" if args.len() == 1 => {
                    let path_tokens = self.transpile_expr(&args[0])?;
                    return Ok(Some(quote! {
                        polars::prelude::CsvReader::from_path(#path_tokens)
                            .expect("Failed to open CSV file")
                            .finish()
                            .expect("Failed to read CSV file")
                    }));
                }
                _ => {}
            }
        }

        // Handle col() function for column references
        if base_name == "col" && args.len() == 1 {
            if let ExprKind::Literal(Literal::String(col_name)) = &args[0].kind {
                return Ok(Some(quote! { polars::prelude::col(#col_name) }));
            }
        }

        Ok(None)
    }

    /// Transpile method calls - entry point
    /// Complexity: 5 (within Toyota Way limits)
    pub(crate) fn transpile_method_call_impl(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        // Check for DataFrame builder pattern
        if method == "column" || method == "build" {
            if let Some(builder_tokens) = self.try_transpile_builder_pattern(object, method, args)?
            {
                return Ok(builder_tokens);
            }
        }

        // Handle contains() with proper borrowing
        if method == "contains" && !args.is_empty() {
            if let Some(tokens) = self.try_transpile_contains_call(object, method, args)? {
                return Ok(tokens);
            }
        }

        // Use the standard implementation
        self.transpile_method_call_standard(object, method, args)
    }

    /// Try to transpile DataFrame builder pattern
    fn try_transpile_builder_pattern(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        let method_call_expr = Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(object.clone()),
                method: method.to_string(),
                args: args.to_vec(),
            },
            span: object.span,
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        };

        self.try_transpile_dataframe_builder_inline(&method_call_expr)
    }

    /// Try to transpile contains() call with proper borrowing
    fn try_transpile_contains_call(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match &args[0].kind {
            ExprKind::FieldAccess { .. } | ExprKind::Identifier(_) => {
                let obj_tokens = self.transpile_expr(object)?;
                let arg_tokens = self.transpile_expr(&args[0])?;
                let method_ident = format_ident!("{}", method);
                Ok(Some(quote! { #obj_tokens.#method_ident(&#arg_tokens) }))
            }
            _ => Ok(None),
        }
    }

    /// Standard method call transpilation
    /// Complexity: 10 (at Toyota Way limit)
    fn transpile_method_call_standard(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        // Check if this is a module function call
        if let ExprKind::Identifier(name) = &object.kind {
            if self.module_names.contains(name) {
                let module_ident = format_ident!("{}", name);
                let method_ident = format_ident!("{}", method);
                let arg_tokens: Result<Vec<_>> =
                    args.iter().map(|a| self.transpile_expr(a)).collect();
                let arg_tokens = arg_tokens?;
                return Ok(quote! { #module_ident::#method_ident(#(#arg_tokens),*) });
            }
        }

        let obj_tokens = self.transpile_expr(object)?;
        let method_ident = format_ident!("{}", method);
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;

        // Check DataFrame methods first
        if Transpiler::is_dataframe_expr(object) && Self::is_dataframe_method(method) {
            return self.transpile_dataframe_method(object, method, args);
        }

        // Dispatch by method category
        self.dispatch_method_by_category(&obj_tokens, method, &method_ident, &arg_tokens, object)
    }

    /// Check if a method is a DataFrame method
    fn is_dataframe_method(method: &str) -> bool {
        matches!(
            method,
            "get" | "rows"
                | "columns"
                | "select"
                | "filter"
                | "sort"
                | "head"
                | "tail"
                | "mean"
                | "std"
                | "min"
                | "max"
                | "sum"
                | "count"
        )
    }

    /// Dispatch method call by category
    fn dispatch_method_by_category(
        &self,
        obj_tokens: &TokenStream,
        method: &str,
        method_ident: &proc_macro2::Ident,
        arg_tokens: &[TokenStream],
        object: &Expr,
    ) -> Result<TokenStream> {
        match method {
            // Iterator operations
            "map" | "filter" | "reduce" => {
                if self.is_option_or_result_with_context(object) {
                    Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
                } else {
                    self.transpile_iterator_methods(obj_tokens, method, arg_tokens)
                }
            }
            // Map/Set methods
            "contains_key" | "keys" | "values" | "entry" | "items" | "update" => {
                self.transpile_map_set_methods(obj_tokens, method_ident, method, arg_tokens)
            }
            // Set operations
            "union" | "intersection" | "difference" | "symmetric_difference" => {
                self.transpile_set_operations(obj_tokens, method, arg_tokens)
            }
            // Common collection methods
            "insert" | "remove" | "clear" | "len" | "is_empty" | "iter" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            // DataFrame operations
            "select" | "groupby" | "group_by" | "agg" | "sort" | "mean" | "std" | "min" | "max"
            | "sum" | "count" | "drop_nulls" | "fill_null" | "pivot" | "melt" | "head" | "tail"
            | "sample" | "describe" | "rows" | "columns" | "column" | "build" => {
                if Transpiler::is_dataframe_expr(object) {
                    self.transpile_dataframe_method(object, method, &[])
                } else {
                    Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
                }
            }
            // String methods
            "to_s" | "to_string" | "to_upper" | "to_lower" | "upper" | "lower" | "length"
            | "substring" | "strip" | "lstrip" | "rstrip" | "startswith" | "endswith" | "split"
            | "replace" => self.transpile_string_methods(obj_tokens, method, arg_tokens),
            // List methods
            "append" => Ok(quote! { #obj_tokens.push(#(#arg_tokens),*) }),
            "extend" => Ok(quote! { #obj_tokens.extend(#(#arg_tokens),*) }),
            // Collection methods
            "push" | "pop" | "contains" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            // Advanced collection methods
            "slice" | "concat" | "flatten" | "unique" | "join" => {
                self.transpile_advanced_collection_methods(obj_tokens, method, arg_tokens)
            }
            // Collect
            "collect" => {
                let obj_str = obj_tokens.to_string();
                if obj_str.contains(". collect ::") || obj_str.contains(".collect::<") {
                    Ok(obj_tokens.clone())
                } else {
                    Ok(quote! { #obj_tokens.collect::<Vec<_>>() })
                }
            }
            // Default
            _ => Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) }),
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

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // transform_main_call tests
    // ========================================================================

    #[test]
    fn test_transform_main_call_renames() {
        let transpiler = make_transpiler();
        let func = ident_expr("main");
        let result = transpiler.transform_main_call(&func).unwrap();
        assert!(result.to_string().contains("__ruchy_main"));
    }

    #[test]
    fn test_transform_main_call_preserves_other() {
        let transpiler = make_transpiler();
        let func = ident_expr("other_func");
        let result = transpiler.transform_main_call(&func).unwrap();
        assert!(result.to_string().contains("other_func"));
        assert!(!result.to_string().contains("__ruchy"));
    }

    // ========================================================================
    // try_transpile_builtin_call tests
    // ========================================================================

    #[test]
    fn test_builtin_len_call() {
        let transpiler = make_transpiler();
        let func_tokens = quote! { len };
        let args = vec![ident_expr("arr")];
        let result = transpiler
            .try_transpile_builtin_call(&func_tokens, "len", &args)
            .unwrap();
        assert!(result.is_some());
        let tokens = result.unwrap();
        assert!(tokens.to_string().contains("arr"));
        assert!(tokens.to_string().contains("len"));
    }

    #[test]
    fn test_builtin_time_micros_call() {
        let transpiler = make_transpiler();
        let func_tokens = quote! { time_micros };
        let result = transpiler
            .try_transpile_builtin_call(&func_tokens, "time_micros", &[])
            .unwrap();
        assert!(result.is_some());
        let tokens = result.unwrap();
        assert!(tokens.to_string().contains("SystemTime"));
    }

    // ========================================================================
    // try_transpile_dataframe_function_impl tests
    // ========================================================================

    #[test]
    fn test_dataframe_new_call() {
        let transpiler = make_transpiler();
        let result = transpiler
            .try_transpile_dataframe_function_impl("DataFrame::new", &[])
            .unwrap();
        assert!(result.is_some());
        let tokens = result.unwrap();
        assert!(tokens.to_string().contains("DataFrame"));
        assert!(tokens.to_string().contains("empty"));
    }

    #[test]
    fn test_dataframe_col_call() {
        let transpiler = make_transpiler();
        let args = vec![string_expr("name")];
        let result = transpiler
            .try_transpile_dataframe_function_impl("col", &args)
            .unwrap();
        assert!(result.is_some());
        let tokens = result.unwrap();
        assert!(tokens.to_string().contains("col"));
        assert!(tokens.to_string().contains("name"));
    }

    #[test]
    fn test_non_dataframe_call() {
        let transpiler = make_transpiler();
        let result = transpiler
            .try_transpile_dataframe_function_impl("some_func", &[])
            .unwrap();
        assert!(result.is_none());
    }

    // ========================================================================
    // try_transpile_contains_call tests
    // ========================================================================

    #[test]
    fn test_contains_with_identifier() {
        let transpiler = make_transpiler();
        let object = ident_expr("haystack");
        let args = vec![ident_expr("needle")];
        let result = transpiler
            .try_transpile_contains_call(&object, "contains", &args)
            .unwrap();
        assert!(result.is_some());
        let tokens = result.unwrap();
        let token_str = tokens.to_string();
        assert!(token_str.contains("haystack"));
        assert!(token_str.contains("&")); // Should borrow
    }

    #[test]
    fn test_contains_with_literal() {
        let transpiler = make_transpiler();
        let object = ident_expr("haystack");
        let args = vec![string_expr("test")];
        let result = transpiler
            .try_transpile_contains_call(&object, "contains", &args)
            .unwrap();
        assert!(result.is_none()); // Literals don't need special handling
    }

    // ========================================================================
    // is_dataframe_method tests
    // ========================================================================

    #[test]
    fn test_is_dataframe_method_true() {
        assert!(Transpiler::is_dataframe_method("select"));
        assert!(Transpiler::is_dataframe_method("filter"));
        assert!(Transpiler::is_dataframe_method("head"));
        assert!(Transpiler::is_dataframe_method("tail"));
    }

    #[test]
    fn test_is_dataframe_method_false() {
        assert!(!Transpiler::is_dataframe_method("push"));
        assert!(!Transpiler::is_dataframe_method("pop"));
        assert!(!Transpiler::is_dataframe_method("custom_method"));
    }

    // ========================================================================
    // dispatch_method_by_category tests
    // ========================================================================

    #[test]
    fn test_dispatch_append_to_push() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { 42 }];
        let method_ident = format_ident!("append");
        let object = ident_expr("vec");
        let result = transpiler
            .dispatch_method_by_category(
                &obj_tokens,
                "append",
                &method_ident,
                &arg_tokens,
                &object,
            )
            .unwrap();
        assert!(result.to_string().contains("push"));
    }

    #[test]
    fn test_dispatch_extend() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { vec };
        let arg_tokens = vec![quote! { other }];
        let method_ident = format_ident!("extend");
        let object = ident_expr("vec");
        let result = transpiler
            .dispatch_method_by_category(
                &obj_tokens,
                "extend",
                &method_ident,
                &arg_tokens,
                &object,
            )
            .unwrap();
        assert!(result.to_string().contains("extend"));
    }

    #[test]
    fn test_dispatch_collect_no_duplicate() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { iter.collect::<Vec<_>>() };
        let method_ident = format_ident!("collect");
        let object = ident_expr("iter");
        let result = transpiler
            .dispatch_method_by_category(&obj_tokens, "collect", &method_ident, &[], &object)
            .unwrap();
        // Should not add another .collect() if already present
        let result_str = result.to_string();
        assert!(!result_str.contains("collect :: < Vec < _ > > () . collect"));
    }

    #[test]
    fn test_dispatch_default_method() {
        let transpiler = make_transpiler();
        let obj_tokens = quote! { obj };
        let arg_tokens = vec![quote! { 1 }, quote! { 2 }];
        let method_ident = format_ident!("custom_method");
        let object = ident_expr("obj");
        let result = transpiler
            .dispatch_method_by_category(
                &obj_tokens,
                "custom_method",
                &method_ident,
                &arg_tokens,
                &object,
            )
            .unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("obj"));
        assert!(result_str.contains("custom_method"));
    }
}
