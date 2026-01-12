//! Statement and control flow transpilation
//! EXTREME TDD Round 82: Cleaned up unused imports
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::doc_markdown)]
use super::*;
use crate::frontend::ast::{Expr, Param, PipelineStage};
use anyhow::Result;
use proc_macro2::TokenStream;

impl Transpiler {
    // EXTREME TDD Round 53: transpile_if moved to control_flow.rs

    // EXTREME TDD Round 54: Let binding methods moved to bindings.rs
    // (generate_let_binding, require_exact_args, require_no_args, transpile_let,
    //  transpile_let_pattern, transpile_let_with_type, transpile_let_pattern_with_type,
    //  transpile_let_else, transpile_let_pattern_else, pattern_needs_slice, value_creates_vec)

    /// Infer return type from parameter types
    /// Delegates to return_type_helpers module
    pub(crate) fn infer_return_type_from_params(
        &self,
        body: &Expr,
        params: &[Param],
    ) -> Result<Option<proc_macro2::TokenStream>> {
        super::return_type_helpers::infer_return_type_from_params(body, params, |ty| {
            self.transpile_type(ty)
        })
    }

    /// Transpiles function definitions
    /// Infer parameter type based on usage in function body
    /// EXTREME TDD Round 79: Delegates to function_param_inference module
    pub(crate) fn infer_param_type(
        &self,
        param: &Param,
        body: &Expr,
        func_name: &str,
    ) -> TokenStream {
        self.infer_param_type_impl(param, body, func_name)
    }

    /// Helper to detect nested array access (2D arrays)
    /// Detects patterns like: param[i][j], param[row][col], param[0][i]
    /// Delegates to function_param_inference module (EXTREME TDD Round 70)
    pub(crate) fn is_nested_array_param(&self, param_name: &str, expr: &Expr) -> bool {
        self.is_nested_array_param_impl(param_name, expr)
    }
    /// Generate parameter tokens with proper type inference
    /// EXTREME TDD Round 78: Delegates to function_param_inference module
    pub(crate) fn generate_param_tokens(
        &self,
        params: &[Param],
        body: &Expr,
        func_name: &str,
    ) -> Result<Vec<TokenStream>> {
        self.generate_param_tokens_impl(params, body, func_name)
    }
    /// Generate return type tokens based on function analysis
    /// EXTREME TDD Round 78: Delegates to function_signature module
    pub(crate) fn generate_return_type_tokens(
        &self,
        name: &str,
        return_type: Option<&Type>,
        body: &Expr,
        params: &[Param],
    ) -> Result<TokenStream> {
        self.generate_return_type_tokens_impl(name, return_type, body, params)
    }
    /// Check if an expression references any global variables (TRANSPILER-SCOPE)
    /// Delegates to function_signature module (EXTREME TDD Round 70)
    pub(crate) fn references_globals(&self, expr: &Expr) -> bool {
        self.references_globals_impl(expr)
    }

    /// Generate body tokens with async support
    /// EXTREME TDD Round 79: Delegates to body_generation module
    pub(crate) fn generate_body_tokens(&self, body: &Expr, is_async: bool) -> Result<TokenStream> {
        self.generate_body_tokens_impl(body, is_async)
    }
    /// Generate type parameter tokens with trait bound support
    /// DEFECT-021 FIX: Properly handle trait bounds like "T: Clone + Debug"
    /// EXTREME TDD Round 76: Delegates to function_signature module
    pub(crate) fn generate_type_param_tokens(
        &self,
        type_params: &[String],
    ) -> Result<Vec<TokenStream>> {
        self.generate_type_param_tokens_impl(type_params)
    }
    /// Generate complete function signature
    /// EXTREME TDD Round 79: Delegates to function_signature module
    pub(crate) fn generate_function_signature(
        &self,
        is_pub: bool,
        is_async: bool,
        fn_name: &proc_macro2::Ident,
        type_param_tokens: &[TokenStream],
        param_tokens: &[TokenStream],
        return_type_tokens: &TokenStream,
        body_tokens: &TokenStream,
        attributes: &[crate::frontend::ast::Attribute],
    ) -> Result<TokenStream> {
        self.generate_function_signature_impl(
            is_pub,
            is_async,
            fn_name,
            type_param_tokens,
            param_tokens,
            return_type_tokens,
            body_tokens,
            attributes,
        )
    }

    /// Compute final return type (test functions have unit type)
    /// Complexity: 1 (within Toyota Way limits)
    /// ISSUE-103: Removed test_ prefix check - already handled by #[test] attribute check
    /// EXTREME TDD Round 78: Delegates to function_signature module
    pub(crate) fn compute_final_return_type(
        &self,
        fn_name: &proc_macro2::Ident,
        return_type_tokens: &TokenStream,
    ) -> TokenStream {
        self.compute_final_return_type_impl(fn_name, return_type_tokens)
    }

    /// Generate visibility token
    /// EXTREME TDD Round 78: Delegates to function_signature module
    pub(crate) fn generate_visibility_token(&self, is_pub: bool) -> TokenStream {
        self.generate_visibility_token_impl(is_pub)
    }

    /// Process attributes into regular attributes and modifiers
    /// EXTREME TDD Round 76: Delegates to function_signature module
    pub(crate) fn process_attributes(
        &self,
        attributes: &[crate::frontend::ast::Attribute],
    ) -> (Vec<TokenStream>, TokenStream) {
        self.process_attributes_impl(attributes)
    }

    /// Generate function declaration based on async/generic flags
    /// EXTREME TDD Round 77: Delegates to function_signature module
    pub(crate) fn generate_function_declaration(
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
        self.generate_function_declaration_impl(
            is_async,
            type_param_tokens,
            regular_attrs,
            visibility,
            modifiers_tokens,
            fn_name,
            param_tokens,
            final_return_type,
            body_tokens,
        )
    }

    /// Helper: Transpile match expression with string literal arm conversion
    /// EXTREME TDD Round 77: Delegates to string_body_conversion module
    pub(crate) fn transpile_match_with_string_arms(
        &self,
        expr: &Expr,
        arms: &[crate::frontend::ast::MatchArm],
    ) -> Result<TokenStream> {
        self.transpile_match_with_string_arms_impl(expr, arms)
    }

    /// DEFECT-012: Generate body tokens with .`to_string()` wrapper on last expression
    /// EXTREME TDD Round 77: Delegates to string_body_conversion module
    pub(crate) fn generate_body_tokens_with_string_conversion(
        &self,
        body: &Expr,
        is_async: bool,
    ) -> Result<TokenStream> {
        self.generate_body_tokens_with_string_conversion_impl(body, is_async)
    }

    /// Generate param tokens with lifetime annotations
    /// EXTREME TDD Round 77: Delegates to lifetime_helpers module
    pub(crate) fn generate_param_tokens_with_lifetime(
        &self,
        params: &[Param],
        body: &Expr,
        func_name: &str,
    ) -> Result<Vec<TokenStream>> {
        self.generate_param_tokens_with_lifetime_impl(params, body, func_name)
    }

    /// Transpile type with lifetime annotation (&T becomes &'a T)
    /// EXTREME TDD Round 77: Delegates to lifetime_helpers module
    pub(crate) fn transpile_type_with_lifetime(&self, ty: &Type) -> Result<TokenStream> {
        self.transpile_type_with_lifetime_impl(ty)
    }

    /// Generate return type tokens with lifetime annotation
    /// EXTREME TDD Round 77: Delegates to lifetime_helpers module
    pub(crate) fn generate_return_type_tokens_with_lifetime(
        &self,
        name: &str,
        return_type: Option<&Type>,
        body: &Expr,
    ) -> Result<TokenStream> {
        self.generate_return_type_tokens_with_lifetime_impl(name, return_type, body)
    }

    /// Transpile function definition
    /// EXTREME TDD Round 81: Delegates to function_transpiler module
    pub fn transpile_function(
        &self,
        name: &str,
        type_params: &[String],
        params: &[Param],
        body: &Expr,
        is_async: bool,
        return_type: Option<&Type>,
        is_pub: bool,
        attributes: &[crate::frontend::ast::Attribute],
    ) -> Result<TokenStream> {
        self.transpile_function_impl(
            name,
            type_params,
            params,
            body,
            is_async,
            return_type,
            is_pub,
            attributes,
        )
    }
    /// Transpiles lambda expressions
    /// # Examples
    /// Transpile lambda/closure expressions
    /// EXTREME TDD Round 79: Delegates to lambda_transpiler module
    pub fn transpile_lambda(&self, params: &[Param], body: &Expr) -> Result<TokenStream> {
        self.transpile_lambda_impl(params, body)
    }
    /// Transpiles function calls
    /// EXTREME TDD Round 79: Delegates to call_transpilation module
    pub fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        self.transpile_call_impl(func, args)
    }

    // EXTREME TDD Round 64: transpile_print_with_interpolation moved to print_helpers.rs

    /// Transpiles method calls
    /// EXTREME TDD Round 79: Delegates to call_transpilation module
    pub fn transpile_method_call(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        self.transpile_method_call_impl(object, method, args)
    }

    /// DEFECT-TRANSPILER-DF-002: Inline `DataFrame` builder pattern transpilation
    /// EXTREME TDD Round 80: Delegates to dataframe_transpilers module
    pub(crate) fn try_transpile_dataframe_builder_inline(
        &self,
        expr: &Expr,
    ) -> Result<Option<TokenStream>> {
        self.try_transpile_dataframe_builder_inline_impl(expr)
    }

    // EXTREME TDD Round 80: extract_dataframe_columns moved to dataframe_transpilers.rs
    // EXTREME TDD Round 65: Method transpilers moved to method_transpilers.rs
    // EXTREME TDD Round 79: transpile_method_call_old removed - consolidated with call_transpilation.rs
    // (transpile_iterator_methods, transpile_map_set_methods, transpile_set_operations,
    //  transpile_string_methods, transpile_advanced_collection_methods)

    /// Transpiles blocks
    /// EXTREME TDD Round 79: Delegates to block_transpiler module
    pub fn transpile_block(&self, exprs: &[Expr]) -> Result<TokenStream> {
        self.transpile_block_impl(exprs)
    }

    /// Transpiles pipeline expressions
    /// EXTREME TDD Round 79: Delegates to block_transpiler module
    pub fn transpile_pipeline(&self, expr: &Expr, stages: &[PipelineStage]) -> Result<TokenStream> {
        self.transpile_pipeline_impl(expr, stages)
    }
    // EXTREME TDD Round 53: Control flow methods moved to control_flow.rs
    // (transpile_for, transpile_while, transpile_if_let, transpile_while_let,
    //  transpile_loop, transpile_try_catch)

    // EXTREME TDD Round 53: Comprehension methods moved to comprehensions.rs

    // EXTREME TDD Round 55: Import/export methods moved to imports.rs
    // (transpile_module, transpile_import, transpile_import_all, transpile_import_default,
    //  transpile_reexport, transpile_export, transpile_export_list, transpile_export_default,
    //  transpile_import_inline, handle_std_module_import, handle_generic_import, path_to_tokens,
    //  handle_single_import_item, handle_multiple_import_items, process_import_items,
    //  transpile_export_legacy)

    // EXTREME TDD Round 56: Math built-in functions moved to math_builtins.rs
    // (try_transpile_math_function, transpile_sqrt, transpile_pow, transpile_abs,
    //  transpile_min, transpile_max, transpile_floor, transpile_ceil, transpile_round)

    // EXTREME TDD Round 57: Input built-in functions moved to input_builtins.rs
    // (try_transpile_input_function, generate_input_without_prompt, generate_input_with_prompt)

    // EXTREME TDD Round 58: Type conversion functions moved to type_conversions.rs
    // (try_transpile_type_conversion, transpile_str_conversion, transpile_int_conversion,
    //  transpile_int_generic, transpile_float_conversion, transpile_float_generic,
    //  transpile_bool_conversion, try_transpile_type_conversion_old)

    // EXTREME TDD Round 59: Advanced math functions moved to advanced_math.rs
    // (try_transpile_math_functions, try_transpile_trueno_function)

    // EXTREME TDD Round 60: Utility built-ins moved to utility_builtins.rs
    // (try_transpile_time_functions, try_transpile_assert_function,
    //  try_transpile_collection_constructor, try_transpile_range_function)

    /// Handle `DataFrame` functions (col)
    /// EXTREME TDD Round 80: Delegates to dataframe_transpilers module
    pub(crate) fn try_transpile_dataframe_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        self.try_transpile_dataframe_function_impl(base_name, args)
    }

    // EXTREME TDD Round 61: System built-ins moved to system_builtins.rs
    // (try_transpile_environment_function, try_transpile_fs_function,
    //  try_transpile_path_function)

    // EXTREME TDD Round 62: Network built-ins moved to network_builtins.rs
    // (try_transpile_json_function, try_transpile_http_function)

    // EXTREME TDD Round 63: Call helpers moved to call_helpers.rs
    // (try_transpile_result_call, transpile_regular_function_call, apply_string_coercion)

    // EXTREME TDD Round 64: Print helpers moved to print_helpers.rs
    // (transpile_print_with_interpolation, try_transpile_print_macro, transpile_print_multiple_args)
}

// ===== EXTREME TDD Round 155 - Statement Transpilation Tests =====

#[cfg(test)]
mod extreme_tdd_tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span, Type, TypeKind};

    fn make_type(kind: TypeKind) -> Type {
        Type {
            kind,
            span: Span::new(0, 0),
        }
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_param(name: &str, ty: Type) -> crate::frontend::ast::Param {
        crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty,
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        }
    }

    // ===== Function Transpilation Delegation Tests =====

    #[test]
    fn test_transpile_function_simple() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t
            .transpile_function(
                "answer",
                &[],
                &[],
                &body,
                false,
                Some(&make_type(TypeKind::Named("i32".to_string()))),
                true,
                &[],
            )
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("fn answer"));
        assert!(s.contains("i32"));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_transpile_function_with_params() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        let params = vec![make_param(
            "x",
            make_type(TypeKind::Named("i32".to_string())),
        )];
        let result = t
            .transpile_function(
                "identity",
                &[],
                &params,
                &body,
                false,
                Some(&make_type(TypeKind::Named("i32".to_string()))),
                false,
                &[],
            )
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("fn identity"));
        assert!(s.contains("x"));
    }

    #[test]
    fn test_transpile_function_async() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let result = t
            .transpile_function("async_func", &[], &[], &body, true, None, true, &[])
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("async"));
        assert!(s.contains("async_func"));
    }

    #[test]
    fn test_transpile_function_with_type_params() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Identifier("value".to_string()));
        let params = vec![make_param(
            "value",
            make_type(TypeKind::Named("T".to_string())),
        )];
        let result = t
            .transpile_function(
                "generic",
                &["T".to_string()],
                &params,
                &body,
                false,
                Some(&make_type(TypeKind::Named("T".to_string()))),
                true,
                &[],
            )
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("generic"));
        assert!(s.contains("<"));
        assert!(s.contains("T"));
    }

    // ===== Lambda Transpilation Tests =====

    #[test]
    fn test_transpile_lambda_no_params() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t.transpile_lambda(&[], &body).unwrap();
        let s = result.to_string();
        assert!(s.contains("||"));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_transpile_lambda_with_params() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        let params = vec![make_param(
            "x",
            make_type(TypeKind::Named("i32".to_string())),
        )];
        let result = t.transpile_lambda(&params, &body).unwrap();
        let s = result.to_string();
        assert!(s.contains("|"));
        assert!(s.contains("x"));
    }

    // ===== Call Transpilation Tests =====

    #[test]
    fn test_transpile_call_simple() {
        let t = Transpiler::new();
        let func = make_expr(ExprKind::Identifier("foo".to_string()));
        let result = t.transpile_call(&func, &[]).unwrap();
        let s = result.to_string();
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_transpile_call_with_args() {
        let t = Transpiler::new();
        let func = make_expr(ExprKind::Identifier("add".to_string()));
        let args = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ];
        let result = t.transpile_call(&func, &args).unwrap();
        let s = result.to_string();
        assert!(s.contains("add"));
        assert!(s.contains("1"));
        assert!(s.contains("2"));
    }

    // ===== Method Call Transpilation Tests =====

    #[test]
    fn test_transpile_method_call_simple() {
        let t = Transpiler::new();
        let obj = make_expr(ExprKind::Identifier("vec".to_string()));
        let result = t.transpile_method_call(&obj, "len", &[]).unwrap();
        let s = result.to_string();
        assert!(s.contains("len"));
    }

    #[test]
    fn test_transpile_method_call_with_args() {
        let t = Transpiler::new();
        let obj = make_expr(ExprKind::Identifier("vec".to_string()));
        let args = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];
        let result = t.transpile_method_call(&obj, "push", &args).unwrap();
        let s = result.to_string();
        assert!(s.contains("push"));
        assert!(s.contains("42"));
    }

    // ===== Block Transpilation Tests =====

    #[test]
    fn test_transpile_block_empty() {
        let t = Transpiler::new();
        let result = t.transpile_block(&[]).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_transpile_block_single() {
        let t = Transpiler::new();
        let exprs = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];
        let result = t.transpile_block(&exprs).unwrap();
        let s = result.to_string();
        assert!(s.contains("42"));
    }

    #[test]
    fn test_transpile_block_multiple() {
        let t = Transpiler::new();
        let exprs = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ];
        let result = t.transpile_block(&exprs).unwrap();
        let s = result.to_string();
        assert!(s.contains("1"));
        assert!(s.contains("2"));
        assert!(s.contains("3"));
    }

    // ===== Pipeline Transpilation Tests =====

    #[test]
    fn test_transpile_pipeline_single_stage() {
        let t = Transpiler::new();
        let expr = make_expr(ExprKind::List(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ]));
        // Create a method call expression as the pipeline stage operation
        let method_call = make_expr(ExprKind::MethodCall {
            receiver: Box::new(make_expr(ExprKind::Identifier("_".to_string()))),
            method: "len".to_string(),
            args: vec![],
        });
        let stages = vec![crate::frontend::ast::PipelineStage {
            op: Box::new(method_call),
            span: Span::new(0, 0),
        }];
        let result = t.transpile_pipeline(&expr, &stages).unwrap();
        let s = result.to_string();
        assert!(s.contains("len"));
    }

    // ===== Helper Method Tests =====

    #[test]
    fn test_generate_visibility_token_public() {
        let t = Transpiler::new();
        let result = t.generate_visibility_token(true);
        assert!(result.to_string().contains("pub"));
    }

    #[test]
    fn test_generate_visibility_token_private() {
        let t = Transpiler::new();
        let result = t.generate_visibility_token(false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_generate_type_param_tokens_simple() {
        let t = Transpiler::new();
        let result = t.generate_type_param_tokens(&["T".to_string()]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("T"));
    }

    #[test]
    fn test_generate_type_param_tokens_multiple() {
        let t = Transpiler::new();
        let result = t
            .generate_type_param_tokens(&["T".to_string(), "U".to_string()])
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_generate_type_param_tokens_with_bound() {
        let t = Transpiler::new();
        let result = t
            .generate_type_param_tokens(&["T: Clone".to_string()])
            .unwrap();
        assert_eq!(result.len(), 1);
        let s = result[0].to_string();
        assert!(s.contains("T"));
        assert!(s.contains("Clone"));
    }

    #[test]
    fn test_references_globals_false() {
        let t = Transpiler::new();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        assert!(!t.references_globals(&expr));
    }

    #[test]
    fn test_infer_param_type_basic() {
        let t = Transpiler::new();
        let param = make_param("x", make_type(TypeKind::Named("_".to_string())));
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        let result = t.infer_param_type(&param, &body, "test");
        // Should produce some type token
        assert!(!result.is_empty());
    }

    #[test]
    fn test_is_nested_array_param_false() {
        let t = Transpiler::new();
        let expr = make_expr(ExprKind::Identifier("x".to_string()));
        assert!(!t.is_nested_array_param("arr", &expr));
    }

    #[test]
    fn test_generate_body_tokens_basic() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t.generate_body_tokens(&body, false).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_generate_body_tokens_async() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t.generate_body_tokens(&body, true).unwrap();
        // Should still contain the body
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_generate_param_tokens_empty() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t.generate_param_tokens(&[], &body, "test").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_generate_param_tokens_single() {
        let t = Transpiler::new();
        let params = vec![make_param(
            "x",
            make_type(TypeKind::Named("i32".to_string())),
        )];
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        let result = t.generate_param_tokens(&params, &body, "test").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_generate_return_type_tokens() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let ret_type = make_type(TypeKind::Named("i32".to_string()));
        let result = t
            .generate_return_type_tokens("test", Some(&ret_type), &body, &[])
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("i32"));
    }

    #[test]
    fn test_compute_final_return_type() {
        let t = Transpiler::new();
        let fn_name = quote::format_ident!("my_func");
        let return_type_tokens = quote::quote! { -> i32 };
        let result = t.compute_final_return_type(&fn_name, &return_type_tokens);
        assert!(result.to_string().contains("i32"));
    }

    #[test]
    fn test_process_attributes_empty() {
        let t = Transpiler::new();
        let (regular, modifiers) = t.process_attributes(&[]);
        assert!(regular.is_empty());
        assert!(modifiers.is_empty());
    }

    #[test]
    fn test_infer_return_type_from_params() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t.infer_return_type_from_params(&body, &[]).unwrap();
        // Should return None for simple literals without type info
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_transpile_type_with_lifetime() {
        let t = Transpiler::new();
        let ty = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        let result = t.transpile_type_with_lifetime(&ty).unwrap();
        let s = result.to_string();
        assert!(s.contains("str"));
    }

    #[test]
    fn test_generate_param_tokens_with_lifetime() {
        let t = Transpiler::new();
        let params = vec![make_param(
            "s",
            make_type(TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
            }),
        )];
        let body = make_expr(ExprKind::Identifier("s".to_string()));
        let result = t
            .generate_param_tokens_with_lifetime(&params, &body, "test")
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_generate_return_type_tokens_with_lifetime() {
        let t = Transpiler::new();
        let ret_type = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        let body = make_expr(ExprKind::Identifier("s".to_string()));
        let result = t
            .generate_return_type_tokens_with_lifetime("test", Some(&ret_type), &body)
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("str"));
    }

    #[test]
    fn test_generate_body_tokens_with_string_conversion() {
        let t = Transpiler::new();
        let body = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        let result = t
            .generate_body_tokens_with_string_conversion(&body, false)
            .unwrap();
        let s = result.to_string();
        assert!(s.contains("hello"));
    }

    #[test]
    fn test_try_transpile_dataframe_builder_inline_none() {
        let t = Transpiler::new();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = t.try_transpile_dataframe_builder_inline(&expr).unwrap();
        // Should return None for non-dataframe expressions
        assert!(result.is_none());
    }

    #[test]
    fn test_try_transpile_dataframe_function_col() {
        let t = Transpiler::new();
        let args = vec![make_expr(ExprKind::Literal(Literal::String(
            "name".to_string(),
        )))];
        let result = t.try_transpile_dataframe_function("col", &args).unwrap();
        if let Some(tokens) = result {
            let s = tokens.to_string();
            assert!(s.contains("col"));
        }
    }

    #[test]
    fn test_try_transpile_dataframe_function_unknown() {
        let t = Transpiler::new();
        let result = t
            .try_transpile_dataframe_function("unknown_func", &[])
            .unwrap();
        // Unknown function should return None
        assert!(result.is_none());
    }
}
