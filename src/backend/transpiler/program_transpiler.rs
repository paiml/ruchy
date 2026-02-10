//! Program-Level Transpilation
//!
//! This module handles transpilation of complete Ruchy programs:
//! - Entry point transpilation (`transpile_to_program`)
//! - Module and import resolution
//! - Main function generation and wrapping
//! - Use statement generation
//! - Program structure handling
//!
//! **EXTREME TDD Round 68**: Extracted from mod.rs for modularization.

#![allow(clippy::doc_markdown)]

use super::{codegen_minimal, constant_folder, inline_expander, Transpiler};
use crate::frontend::ast::{Expr, ExprKind, Param};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Centralized result printing logic - ONE PLACE FOR ALL RESULT PRINTING
    /// This eliminates code duplication and ensures consistent Unit type handling
    /// FIX-001: Use {:?} for all types to avoid Display trait requirement on ()
    /// Complexity: 2 (within Toyota Way limits)
    pub fn generate_result_printing_tokens(&self) -> TokenStream {
        quote! {
            // Check the type name first to avoid printing Unit type
            // Use {:?} for all types since () implements Debug but not Display
            if std::any::type_name_of_val(&result) == "()" {
                // Don't print Unit type
            } else {
                // Use Debug formatting for all types to handle ()
                // This works for String, &str, and all other types
                println!("{:?}", result);
            }
        }
    }

    /// Wraps transpiled code in a complete Rust program with necessary imports
    /// Complexity: 4 (within Toyota Way limits)
    pub fn transpile_to_program(&mut self, expr: &Expr) -> Result<TokenStream> {
        // First analyze the entire program to detect mutable variables, const declarations, function signatures, and modules
        // SPEC-001-B: Must collect const names BEFORE optimization to preserve attributes
        if let ExprKind::Block(exprs) = &expr.kind {
            self.analyze_mutability(exprs);
            self.collect_const_declarations(exprs);
            self.collect_function_signatures(exprs);
            self.collect_module_names(exprs);
            // BOOK-COMPAT-017: Collect call-site argument types for parameter inference
            self.collect_call_site_types(exprs);
        } else {
            self.analyze_expr_mutability(expr);
            self.collect_const_declarations_from_expr(expr);
            self.collect_signatures_from_expr(expr);
            self.collect_module_names_from_expr(expr);
            // BOOK-COMPAT-017: Collect call-site types for single expressions too
            self.collect_call_site_types(std::slice::from_ref(expr));
        }
        let result = self.transpile_to_program_with_context(expr, None);
        if let Ok(ref token_stream) = result {
            // Debug: Write the generated Rust code to a debug file
            let rust_code = token_stream.to_string();
            std::fs::write("/tmp/debug_transpiler_output.rs", &rust_code).ok();
        }
        result
    }

    /// Transpile with file context for module resolution
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_to_program_with_context(
        &mut self,
        expr: &Expr,
        file_path: Option<&std::path::Path>,
    ) -> Result<TokenStream> {
        // First, resolve any file imports using the module resolver
        let resolved_expr = self.resolve_imports_with_context(expr, file_path)?;

        // TRANSPILER-009 FIX: Skip aggressive optimizations for top-level programs with standalone functions
        let has_standalone_functions = Self::has_standalone_functions(&resolved_expr);

        let optimized_expr = if has_standalone_functions {
            // Skip inlining and DCE for programs with standalone functions
            constant_folder::propagate_constants(resolved_expr)
        } else {
            // PERF-002-A/B: Apply constant folding + constant propagation
            let after_propagation = constant_folder::propagate_constants(resolved_expr);

            // OPT-CODEGEN-004: Inline small, non-recursive functions
            let (after_inlining, inlined_functions) =
                inline_expander::inline_small_functions(after_propagation);

            // PERF-002-C: Dead code elimination
            constant_folder::eliminate_dead_code(after_inlining, inlined_functions)
        };

        // CRITICAL: Analyze mutability, signatures, modules, and call-site types BEFORE transpiling
        if let ExprKind::Block(exprs) = &optimized_expr.kind {
            self.analyze_mutability(exprs);
            self.collect_function_signatures(exprs);
            self.collect_module_names(exprs);
            // BOOK-COMPAT-017: Collect call-site types from optimized expression
            self.collect_call_site_types(exprs);
        } else {
            self.analyze_expr_mutability(&optimized_expr);
            self.collect_signatures_from_expr(&optimized_expr);
            self.collect_module_names_from_expr(&optimized_expr);
            // BOOK-COMPAT-017: Collect call-site types from single expression
            self.collect_call_site_types(std::slice::from_ref(&optimized_expr));
        }

        let needs_polars = Self::contains_dataframe(&optimized_expr);
        let needs_hashmap = Self::contains_hashmap(&optimized_expr);

        match &optimized_expr.kind {
            ExprKind::Function { name, .. } => {
                self.transpile_single_function(&optimized_expr, name, needs_polars, needs_hashmap)
            }
            ExprKind::Block(exprs) => {
                self.transpile_program_block(exprs, needs_polars, needs_hashmap)
            }
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => {
                self.transpile_import_program(&optimized_expr, needs_polars, needs_hashmap)
            }
            _ => self.transpile_expression_program(&optimized_expr, needs_polars, needs_hashmap),
        }
    }

    /// Transpile a single import as a program
    /// Complexity: 4 (within Toyota Way limits)
    fn transpile_import_program(
        &self,
        expr: &Expr,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let import_tokens = self.transpile_expr(expr)?;
        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #import_tokens
                fn main() {}
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                #import_tokens
                fn main() {}
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                #import_tokens
                fn main() {}
            }),
            (false, false) => Ok(quote! {
                #import_tokens
                fn main() {}
            }),
        }
    }

    /// Transpile a single function definition
    /// Complexity: 4 (within Toyota Way limits)
    fn transpile_single_function(
        &self,
        expr: &Expr,
        name: &str,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let func_tokens = self.transpile_function_expr(expr)?;
        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);

        if name == "main" {
            Ok(quote! {
                #use_statements
                #func_tokens
            })
        } else {
            Ok(quote! {
                #use_statements
                #func_tokens
                fn main() { /* Function defined but not called */ }
            })
        }
    }

    /// Transpile a block of expressions as a program
    /// Complexity: 5 (within Toyota Way limits)
    fn transpile_program_block(
        &self,
        exprs: &[Expr],
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let (functions, statements, modules, has_main, main_expr, imports, globals) =
            self.categorize_block_expressions(exprs)?;

        // Check for statement-only blocks (all expressions are statements like let, while, for)
        let all_statements = exprs.iter().all(Self::is_statement_expr);
        if !has_main && functions.is_empty() && modules.is_empty() && all_statements {
            return self.transpile_statement_only_block(
                exprs,
                needs_polars,
                needs_hashmap,
                &imports,
            );
        }

        if has_main || !functions.is_empty() || !modules.is_empty() {
            self.transpile_block_with_main_function(
                &functions,
                &statements,
                &modules,
                main_expr,
                needs_polars,
                needs_hashmap,
                &imports,
                &globals,
            )
        } else {
            self.transpile_expression_program(
                &Expr::new(
                    ExprKind::Block(exprs.to_vec()),
                    crate::frontend::ast::Span::new(0, 0),
                ),
                needs_polars,
                needs_hashmap,
            )
        }
    }

    /// Transpile module declaration
    /// Complexity: 5 (within Toyota Way limits)
    pub fn transpile_module_declaration(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let module_name = format_ident!("{}", name);
        let body_tokens = if let ExprKind::Block(exprs) = &body.kind {
            let mut module_items = Vec::new();
            for expr in exprs {
                match &expr.kind {
                    ExprKind::Function { .. } => {
                        module_items.push(self.transpile_function_expr(expr)?);
                    }
                    _ => {
                        module_items.push(self.transpile_expr(expr)?);
                    }
                }
            }
            quote! { #(#module_items)* }
        } else {
            self.transpile_expr(body)?
        };

        Ok(quote! {
            mod #module_name {
                #body_tokens
            }
        })
    }

    /// Transpile statement-only block
    /// Complexity: 6 (within Toyota Way limits)
    fn transpile_statement_only_block(
        &self,
        exprs: &[Expr],
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
    ) -> Result<TokenStream> {
        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);

        let mut stmt_tokens = Vec::new();
        for expr in exprs {
            let tokens = self.transpile_expr(expr)?;
            let tokens_str = tokens.to_string();
            if !tokens_str.trim().ends_with(';') && !tokens_str.trim().ends_with('}') {
                stmt_tokens.push(quote! { #tokens; });
            } else {
                stmt_tokens.push(tokens);
            }
        }

        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #(#imports)*
                fn main() {
                    #(#stmt_tokens)*
                }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                #(#imports)*
                fn main() {
                    #(#stmt_tokens)*
                }
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                #(#imports)*
                fn main() {
                    #(#stmt_tokens)*
                }
            }),
            (false, false) => Ok(quote! {
                #(#imports)*
                #use_statements
                fn main() {
                    #(#stmt_tokens)*
                }
            }),
        }
    }

    /// Transpile block with main function wrapper
    /// Complexity: 3 (within Toyota Way limits)
    fn transpile_block_with_main_function(
        &self,
        functions: &[TokenStream],
        statements: &[TokenStream],
        modules: &[TokenStream],
        main_expr: Option<&Expr>,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        if let (true, Some(main)) = (statements.is_empty(), main_expr) {
            self.transpile_functions_only_mode(
                functions,
                modules,
                main,
                needs_polars,
                needs_hashmap,
                imports,
                globals,
            )
        } else {
            self.transpile_with_top_level_statements(
                functions,
                statements,
                modules,
                main_expr,
                needs_polars,
                needs_hashmap,
                imports,
                globals,
            )
        }
    }

    /// Transpile functions-only mode (no top-level statements)
    /// Complexity: 4 (within Toyota Way limits)
    fn transpile_functions_only_mode(
        &self,
        functions: &[TokenStream],
        modules: &[TokenStream],
        main_expr: &Expr,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);
        let main_tokens = self.transpile_function_expr(main_expr)?;

        Ok(quote! {
            #use_statements
            #(#imports)*
            #(#globals)*
            #(#modules)*
            #(#functions)*
            #main_tokens
        })
    }

    /// Transpile with top-level statements
    /// Complexity: 6 (within Toyota Way limits)
    fn transpile_with_top_level_statements(
        &self,
        functions: &[TokenStream],
        statements: &[TokenStream],
        modules: &[TokenStream],
        main_expr: Option<&Expr>,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);

        let main_tokens = if let Some(main_fn) = main_expr {
            let renamed_main = self.transpile_main_as_renamed_function(main_fn)?;
            quote! {
                #renamed_main
                fn main() {
                    #(#statements)*
                    __ruchy_main();
                }
            }
        } else {
            quote! {
                fn main() {
                    #(#statements)*
                }
            }
        };

        Ok(quote! {
            #use_statements
            #(#imports)*
            #(#globals)*
            #(#modules)*
            #(#functions)*
            #main_tokens
        })
    }

    /// DEFECT-COMPILE-MAIN-CALL: Renames `fun main()` to `fn __ruchy_main()`
    /// Complexity: 6 (within Toyota Way limits)
    fn transpile_main_as_renamed_function(&self, main_expr: &Expr) -> Result<TokenStream> {
        if let ExprKind::Function {
            params,
            body,
            return_type,
            is_async,
            ..
        } = &main_expr.kind
        {
            let renamed_ident = format_ident!("__ruchy_main");
            let param_tokens: Vec<TokenStream> = params
                .iter()
                .map(|p| self.transpile_param(p))
                .collect::<Result<Vec<_>>>()?;
            let body_tokens = self.transpile_expr(body)?;

            let return_type_tokens = if let Some(rt) = return_type {
                let rt_tokens = self.transpile_type(rt)?;
                quote! { -> #rt_tokens }
            } else {
                quote! {}
            };

            let async_token = if *is_async {
                quote! { async }
            } else {
                quote! {}
            };

            Ok(quote! {
                #async_token fn #renamed_ident(#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            })
        } else {
            anyhow::bail!("Expected Function expression for main")
        }
    }

    /// Generate use statements based on detected features
    /// Complexity: 2 (within Toyota Way limits)
    pub fn generate_use_statements(&self, needs_polars: bool, needs_hashmap: bool) -> TokenStream {
        match (needs_polars, needs_hashmap) {
            (true, true) => quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
            },
            (true, false) => quote! {
                use polars::prelude::*;
            },
            (false, true) => quote! {
                use std::collections::HashMap;
            },
            (false, false) => quote! {},
        }
    }

    /// Extract main function body for inlining
    /// Complexity: 2 (within Toyota Way limits)
    fn extract_main_function_body(&self, main_expr: &Expr) -> Result<TokenStream> {
        if let ExprKind::Function { body, .. } = &main_expr.kind {
            self.transpile_expr(body)
        } else {
            anyhow::bail!("Expected Function expression")
        }
    }

    /// Transpile block with functions
    /// Complexity: 5 (within Toyota Way limits)
    fn transpile_block_with_functions(
        &self,
        exprs: &[Expr],
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let _use_statements = self.generate_use_statements(needs_polars, needs_hashmap);
        let (functions, _statements, modules, has_main, main_expr, imports, globals) =
            self.categorize_block_expressions(exprs)?;

        if has_main {
            let main_body = self.extract_main_function_body(main_expr.expect("main exists"))?;
            match (needs_polars, needs_hashmap) {
                (true, true) => Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    #(#imports)*
                    #(#globals)*
                    #(#modules)*
                    #(#functions)*
                    fn main() { #main_body }
                }),
                (true, false) => Ok(quote! {
                    use polars::prelude::*;
                    #(#imports)*
                    #(#globals)*
                    #(#modules)*
                    #(#functions)*
                    fn main() { #main_body }
                }),
                (false, true) => Ok(quote! {
                    use std::collections::HashMap;
                    #(#imports)*
                    #(#globals)*
                    #(#modules)*
                    #(#functions)*
                    fn main() { #main_body }
                }),
                (false, false) => Ok(quote! {
                    #(#imports)*
                    #(#globals)*
                    #(#modules)*
                    #(#functions)*
                    fn main() { #main_body }
                }),
            }
        } else {
            // Use existing result printing approach
            let block_expr = Expr::new(
                ExprKind::Block(exprs.to_vec()),
                crate::frontend::ast::Span::new(0, 0),
            );
            let body = self.transpile_expr(&block_expr)?;
            self.wrap_in_main_with_result_printing(body, needs_polars, needs_hashmap)
        }
    }

    /// Transpile expression program (single expression, not a block)
    /// Complexity: 5 (within Toyota Way limits)
    fn transpile_expression_program(
        &self,
        expr: &Expr,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        // Check if all expressions are statements
        if let ExprKind::Block(exprs) = &expr.kind {
            let all_statements = exprs.iter().all(Self::is_statement_expr);
            if all_statements {
                return self.wrap_statement_in_main(exprs, needs_polars, needs_hashmap);
            }
        }

        let body = self.transpile_expr(expr)?;
        self.wrap_in_main_with_result_printing(body, needs_polars, needs_hashmap)
    }

    /// Wrap statements in main without result printing
    /// Complexity: 4 (within Toyota Way limits)
    fn wrap_statement_in_main(
        &self,
        exprs: &[Expr],
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);

        let mut stmt_tokens = Vec::new();
        for expr in exprs {
            let tokens = self.transpile_expr(expr)?;
            let tokens_str = tokens.to_string();
            if !tokens_str.trim().ends_with(';') && !tokens_str.trim().ends_with('}') {
                stmt_tokens.push(quote! { #tokens; });
            } else {
                stmt_tokens.push(tokens);
            }
        }

        Ok(quote! {
            #use_statements
            fn main() {
                #(#stmt_tokens)*
            }
        })
    }

    /// Wrap expression in main with result printing
    /// Complexity: 3 (within Toyota Way limits)
    pub fn wrap_in_main_with_result_printing(
        &self,
        body: TokenStream,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);
        let result_printing = self.generate_result_printing_tokens();

        Ok(quote! {
            #use_statements
            fn main() {
                let result = { #body };
                #result_printing
            }
        })
    }

    /// Transpiles an expression to a String
    /// Complexity: 3 (within Toyota Way limits)
    pub fn transpile_to_string(&mut self, expr: &Expr) -> Result<String> {
        let tokens = self.transpile(expr)?;
        let mut result = String::new();
        let token_str = tokens.to_string();
        for ch in token_str.chars() {
            result.push(ch);
            if ch == ';' || ch == '{' {
                result.push('\n');
            }
        }
        Ok(result)
    }

    /// Generate minimal code for self-hosting (direct Rust mapping, no optimization)
    /// Complexity: 1 (within Toyota Way limits)
    pub fn transpile_minimal(&self, expr: &Expr) -> Result<String> {
        codegen_minimal::MinimalCodeGen::gen_program(expr)
    }

    /// Transpile a function parameter
    /// Complexity: 3 (within Toyota Way limits)
    fn transpile_param(&self, param: &Param) -> Result<TokenStream> {
        let name = format_ident!("{}", param.name());
        let type_tokens = self.transpile_type(&param.ty)?;
        Ok(quote! { #name: #type_tokens })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span};

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

    fn block_expr(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn func_expr(name: &str, body: Expr) -> Expr {
        make_expr(ExprKind::Function {
            name: name.to_string(),
            type_params: vec![],
            params: vec![],
            return_type: None,
            body: Box::new(body),
            is_async: false,
            is_pub: false,
        })
    }

    // ========================================================================
    // generate_result_printing_tokens tests
    // ========================================================================

    #[test]
    fn test_generate_result_printing_tokens() {
        let transpiler = Transpiler::new();
        let tokens = transpiler.generate_result_printing_tokens();
        let code = tokens.to_string();
        assert!(code.contains("type_name_of_val"));
        assert!(code.contains("println"));
    }

    // ========================================================================
    // generate_use_statements tests
    // ========================================================================

    #[test]
    fn test_generate_use_statements_both() {
        let transpiler = Transpiler::new();
        let tokens = transpiler.generate_use_statements(true, true);
        let code = tokens.to_string();
        assert!(code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    #[test]
    fn test_generate_use_statements_polars_only() {
        let transpiler = Transpiler::new();
        let tokens = transpiler.generate_use_statements(true, false);
        let code = tokens.to_string();
        assert!(code.contains("polars"));
        assert!(!code.contains("HashMap"));
    }

    #[test]
    fn test_generate_use_statements_hashmap_only() {
        let transpiler = Transpiler::new();
        let tokens = transpiler.generate_use_statements(false, true);
        let code = tokens.to_string();
        assert!(!code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    #[test]
    fn test_generate_use_statements_none() {
        let transpiler = Transpiler::new();
        let tokens = transpiler.generate_use_statements(false, false);
        let code = tokens.to_string();
        assert!(code.is_empty());
    }

    // ========================================================================
    // transpile_to_program tests
    // ========================================================================

    #[test]
    fn test_transpile_to_program_simple_int() {
        let mut transpiler = Transpiler::new();
        let expr = int_expr(42);
        let result = transpiler.transpile_to_program(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn main"));
        assert!(code.contains("42"));
    }

    #[test]
    fn test_transpile_to_program_with_main() {
        let mut transpiler = Transpiler::new();
        let main_func = func_expr("main", int_expr(0));
        let result = transpiler.transpile_to_program(&main_func);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn main"));
    }

    #[test]
    fn test_transpile_to_program_block() {
        let mut transpiler = Transpiler::new();
        let block = block_expr(vec![int_expr(1), int_expr(2)]);
        let result = transpiler.transpile_to_program(&block);
        assert!(result.is_ok());
    }

    // ========================================================================
    // transpile_to_string tests
    // ========================================================================

    #[test]
    fn test_transpile_to_string_simple() {
        let mut transpiler = Transpiler::new();
        let expr = int_expr(42);
        let result = transpiler.transpile_to_string(&expr);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("42"));
    }

    #[test]
    fn test_transpile_to_string_formatting() {
        let mut transpiler = Transpiler::new();
        let expr = int_expr(42);
        let result = transpiler.transpile_to_string(&expr);
        assert!(result.is_ok());
        // Should have newlines after semicolons/braces
        let code = result.unwrap();
        assert!(code.contains('\n'));
    }

    // ========================================================================
    // wrap_in_main_with_result_printing tests
    // ========================================================================

    #[test]
    fn test_wrap_in_main_with_result_printing() {
        let transpiler = Transpiler::new();
        let body = quote! { 42 };
        let result = transpiler.wrap_in_main_with_result_printing(body, false, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn main"));
        assert!(code.contains("let result"));
    }

    #[test]
    fn test_wrap_in_main_with_polars() {
        let transpiler = Transpiler::new();
        let body = quote! { 42 };
        let result = transpiler.wrap_in_main_with_result_printing(body, true, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("polars"));
    }

    // ========================================================================
    // transpile_module_declaration tests
    // ========================================================================

    #[test]
    fn test_transpile_module_declaration_simple() {
        let transpiler = Transpiler::new();
        let body = int_expr(42);
        let result = transpiler.transpile_module_declaration("test_mod", &body);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("mod test_mod"));
    }

    #[test]
    fn test_transpile_module_declaration_with_function() {
        let transpiler = Transpiler::new();
        let func = func_expr("helper", int_expr(1));
        let body = block_expr(vec![func]);
        let result = transpiler.transpile_module_declaration("my_mod", &body);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("mod my_mod"));
        assert!(code.contains("fn helper"));
    }

    // ========================================================================
    // transpile_single_function tests
    // ========================================================================

    #[test]
    fn test_transpile_single_function_main() {
        let transpiler = Transpiler::new();
        let func = func_expr("main", int_expr(0));
        let result = transpiler.transpile_single_function(&func, "main", false, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn main"));
        // Should not have extra main wrapper
        assert_eq!(code.matches("fn main").count(), 1);
    }

    #[test]
    fn test_transpile_single_function_non_main() {
        let transpiler = Transpiler::new();
        let func = func_expr("helper", int_expr(1));
        let result = transpiler.transpile_single_function(&func, "helper", false, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn helper"));
        assert!(code.contains("fn main")); // Should have empty main
    }

    // ========================================================================
    // transpile_import_program tests
    // ========================================================================

    #[test]
    fn test_transpile_import_program() {
        let transpiler = Transpiler::new();
        let import = make_expr(ExprKind::Import {
            module: "std::io".to_string(),
            items: None,
        });
        let result = transpiler.transpile_import_program(&import, false, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn main"));
    }

    // ========================================================================
    // extract_main_function_body tests
    // ========================================================================

    #[test]
    fn test_extract_main_function_body() {
        let transpiler = Transpiler::new();
        let func = func_expr("main", int_expr(42));
        let result = transpiler.extract_main_function_body(&func);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("42"));
    }

    #[test]
    fn test_extract_main_function_body_not_function() {
        let transpiler = Transpiler::new();
        let not_func = int_expr(42);
        let result = transpiler.extract_main_function_body(&not_func);
        assert!(result.is_err());
    }

    // ========================================================================
    // EXTREME TDD Round 141 - Additional edge case tests
    // ========================================================================

    // Test: transpile_import_program with polars and hashmap
    #[test]
    fn test_transpile_import_program_with_polars_and_hashmap() {
        let transpiler = Transpiler::new();
        let import = make_expr(ExprKind::Import {
            module: "std::collections".to_string(),
            items: None,
        });
        let result = transpiler.transpile_import_program(&import, true, true);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    // Test: transpile_import_program with polars only
    #[test]
    fn test_transpile_import_program_with_polars_only() {
        let transpiler = Transpiler::new();
        let import = make_expr(ExprKind::Import {
            module: "data".to_string(),
            items: None,
        });
        let result = transpiler.transpile_import_program(&import, true, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("polars"));
        assert!(!code.contains("HashMap"));
    }

    // Test: transpile_import_program with hashmap only
    #[test]
    fn test_transpile_import_program_with_hashmap_only() {
        let transpiler = Transpiler::new();
        let import = make_expr(ExprKind::Import {
            module: "utils".to_string(),
            items: None,
        });
        let result = transpiler.transpile_import_program(&import, false, true);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(!code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    // Test: transpile_single_function with polars
    #[test]
    fn test_transpile_single_function_with_polars() {
        let transpiler = Transpiler::new();
        let func = func_expr("process_data", int_expr(0));
        let result = transpiler.transpile_single_function(&func, "process_data", true, false);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("polars"));
    }

    // Test: transpile_single_function with hashmap
    #[test]
    fn test_transpile_single_function_with_hashmap() {
        let transpiler = Transpiler::new();
        let func = func_expr("build_map", int_expr(0));
        let result = transpiler.transpile_single_function(&func, "build_map", false, true);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashMap"));
    }

    // Test: wrap_in_main_with_result_printing with both flags
    #[test]
    fn test_wrap_in_main_with_both_flags() {
        let transpiler = Transpiler::new();
        let body = quote! { "test" };
        let result = transpiler.wrap_in_main_with_result_printing(body, true, true);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    // Test: transpile_to_program with string
    #[test]
    fn test_transpile_to_program_string() {
        let mut transpiler = Transpiler::new();
        let expr = string_expr("hello");
        let result = transpiler.transpile_to_program(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("hello"));
    }

    // Test: transpile_to_program with identifier
    #[test]
    fn test_transpile_to_program_identifier() {
        let mut transpiler = Transpiler::new();
        let expr = ident_expr("my_var");
        let result = transpiler.transpile_to_program(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("my_var"));
    }

    // Test: transpile_module_declaration with multiple items
    #[test]
    fn test_transpile_module_declaration_multiple_items() {
        let transpiler = Transpiler::new();
        let func1 = func_expr("func_a", int_expr(1));
        let func2 = func_expr("func_b", int_expr(2));
        let body = block_expr(vec![func1, func2]);
        let result = transpiler.transpile_module_declaration("utils", &body);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("mod utils"));
        assert!(code.contains("fn func_a"));
        assert!(code.contains("fn func_b"));
    }

    // Test: transpile_module_declaration with expression
    #[test]
    fn test_transpile_module_declaration_with_expression() {
        let transpiler = Transpiler::new();
        let body = block_expr(vec![int_expr(42), string_expr("test")]);
        let result = transpiler.transpile_module_declaration("data", &body);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("mod data"));
    }

    // Test: transpile_to_string with block
    #[test]
    fn test_transpile_to_string_block() {
        let mut transpiler = Transpiler::new();
        let expr = block_expr(vec![int_expr(1), int_expr(2)]);
        let result = transpiler.transpile_to_string(&expr);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("1"));
        assert!(code.contains("2"));
    }

    // Test: transpile_to_program with function in block
    #[test]
    fn test_transpile_to_program_function_in_block() {
        let mut transpiler = Transpiler::new();
        let func = func_expr("helper", int_expr(10));
        let block = block_expr(vec![func]);
        let result = transpiler.transpile_to_program(&block);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn helper"));
    }

    // Test: transpile_to_program_with_context
    #[test]
    fn test_transpile_to_program_with_context_simple() {
        let mut transpiler = Transpiler::new();
        let expr = int_expr(100);
        let result = transpiler.transpile_to_program_with_context(&expr, None);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("100"));
    }

    // Test: wrap_in_main with hashmap
    #[test]
    fn test_wrap_in_main_with_hashmap() {
        let transpiler = Transpiler::new();
        let body = quote! { 42 };
        let result = transpiler.wrap_in_main_with_result_printing(body, false, true);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashMap"));
    }

    // Test: generate_result_printing_tokens contains expected patterns
    #[test]
    fn test_generate_result_printing_tokens_patterns() {
        let transpiler = Transpiler::new();
        let tokens = transpiler.generate_result_printing_tokens();
        let code = tokens.to_string();
        assert!(code.contains("type_name_of_val"));
        assert!(code.contains("\"()\""));
    }

    // Test: transpile_to_program with multiple functions
    #[test]
    fn test_transpile_to_program_multiple_functions() {
        let mut transpiler = Transpiler::new();
        let func1 = func_expr("helper", int_expr(1));
        let func2 = func_expr("main", int_expr(0));
        let block = block_expr(vec![func1, func2]);
        let result = transpiler.transpile_to_program(&block);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("fn helper"));
        assert!(code.contains("fn main"));
    }

    // ========================================================================
    // transpile_main_as_renamed_function tests
    // ========================================================================

    #[test]
    fn test_main_renamed_basic() {
        let transpiler = Transpiler::new();
        let main_expr = func_expr("main", int_expr(0));
        let result = transpiler
            .transpile_main_as_renamed_function(&main_expr)
            .unwrap();
        let code = result.to_string();
        assert!(
            code.contains("__ruchy_main"),
            "Should rename to __ruchy_main: {code}"
        );
        assert!(
            !code.contains("fn main"),
            "Should NOT contain fn main: {code}"
        );
    }

    #[test]
    fn test_main_renamed_with_return_type() {
        let transpiler = Transpiler::new();
        let main_expr = make_expr(ExprKind::Function {
            name: "main".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: Some(crate::frontend::ast::Type {
                kind: crate::frontend::ast::TypeKind::Named("i32".to_string()),
                span: crate::frontend::ast::Span::default(),
            }),
            body: Box::new(int_expr(0)),
            is_async: false,
            is_pub: false,
        });
        let result = transpiler
            .transpile_main_as_renamed_function(&main_expr)
            .unwrap();
        let code = result.to_string();
        assert!(code.contains("__ruchy_main"), "Should rename");
        assert!(code.contains("i32"), "Should have return type");
    }

    #[test]
    fn test_main_renamed_async() {
        let transpiler = Transpiler::new();
        let main_expr = make_expr(ExprKind::Function {
            name: "main".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: None,
            body: Box::new(int_expr(0)),
            is_async: true,
            is_pub: false,
        });
        let result = transpiler
            .transpile_main_as_renamed_function(&main_expr)
            .unwrap();
        let code = result.to_string();
        assert!(code.contains("async"), "Should have async keyword");
        assert!(code.contains("__ruchy_main"), "Should rename");
    }

    #[test]
    fn test_main_renamed_with_params() {
        let transpiler = Transpiler::new();
        let param = crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier("args".to_string()),
            ty: crate::frontend::ast::Type {
                kind: crate::frontend::ast::TypeKind::Named("Vec".to_string()),
                span: crate::frontend::ast::Span::default(),
            },
            span: crate::frontend::ast::Span::default(),
            is_mutable: false,
            default_value: None,
        };
        let main_expr = make_expr(ExprKind::Function {
            name: "main".to_string(),
            type_params: vec![],
            params: vec![param],
            return_type: None,
            body: Box::new(int_expr(0)),
            is_async: false,
            is_pub: false,
        });
        let result = transpiler
            .transpile_main_as_renamed_function(&main_expr)
            .unwrap();
        let code = result.to_string();
        assert!(code.contains("__ruchy_main"), "Should rename");
        assert!(code.contains("args"), "Should have parameter");
    }

    #[test]
    fn test_main_renamed_non_function_fails() {
        let transpiler = Transpiler::new();
        let non_func = int_expr(42);
        let result = transpiler.transpile_main_as_renamed_function(&non_func);
        assert!(result.is_err(), "Non-function should fail");
    }

    #[test]
    fn test_main_renamed_with_body_content() {
        let transpiler = Transpiler::new();
        let body = block_expr(vec![
            make_expr(ExprKind::Call {
                func: Box::new(ident_expr("println")),
                args: vec![string_expr("hello")],
            }),
        ]);
        let main_expr = func_expr("main", body);
        let result = transpiler
            .transpile_main_as_renamed_function(&main_expr)
            .unwrap();
        let code = result.to_string();
        assert!(code.contains("__ruchy_main"));
        assert!(code.contains("println"), "Body should be transpiled");
    }
}
