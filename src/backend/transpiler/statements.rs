//! Statement and control flow transpilation
//! EXTREME TDD Round 82: Cleaned up unused imports
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]
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
    pub(crate) fn infer_param_type(&self, param: &Param, body: &Expr, func_name: &str) -> TokenStream {
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
    pub(crate) fn generate_type_param_tokens(&self, type_params: &[String]) -> Result<Vec<TokenStream>> {
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

