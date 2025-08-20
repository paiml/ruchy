//! Statement and control flow transpilation

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]

use super::*;
use crate::frontend::ast::{CatchClause, Param, PipelineStage};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles if expressions
    pub fn transpile_if(
        &self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;
        let then_tokens = self.transpile_expr(then_branch)?;

        if let Some(else_expr) = else_branch {
            let else_tokens = self.transpile_expr(else_expr)?;
            Ok(quote! {
                if #cond_tokens {
                    #then_tokens
                } else {
                    #else_tokens
                }
            })
        } else {
            Ok(quote! {
                if #cond_tokens {
                    #then_tokens
                }
            })
        }
    }

    /// Transpiles let bindings
    pub fn transpile_let(
        &self,
        name: &str,
        value: &Expr,
        body: &Expr,
        is_mutable: bool,
    ) -> Result<TokenStream> {
        let name_ident = format_ident!("{}", name);
        let value_tokens = self.transpile_expr(value)?;
        let body_tokens = self.transpile_expr(body)?;

        if is_mutable {
            Ok(quote! {
                {
                    let mut #name_ident = #value_tokens;
                    #body_tokens
                }
            })
        } else {
            Ok(quote! {
                {
                    let #name_ident = #value_tokens;
                    #body_tokens
                }
            })
        }
    }

    /// Transpiles function definitions
    pub fn transpile_function(
        &self,
        name: &str,
        type_params: &[String],
        params: &[Param],
        body: &Expr,
        is_async: bool,
        return_type: Option<&Type>,
    ) -> Result<TokenStream> {
        let fn_name = format_ident!("{}", name);

        let param_tokens: Vec<TokenStream> = params
            .iter()
            .map(|p| {
                let param_name = format_ident!("{}", p.name);
                let type_tokens = self.transpile_type(&p.ty).unwrap_or_else(|_| quote! { _ });
                quote! { #param_name: #type_tokens }
            })
            .collect();

        let body_tokens = if is_async {
            let mut async_transpiler = Transpiler::new();
            async_transpiler.in_async_context = true;
            async_transpiler.transpile_expr(body)?
        } else {
            self.transpile_expr(body)?
        };

        let return_type_tokens = if let Some(ty) = return_type {
            let ty_tokens = self.transpile_type(ty)?;
            quote! { -> #ty_tokens }
        } else {
            quote! {}
        };

        let type_param_tokens: Vec<_> =
            type_params.iter().map(|p| format_ident!("{}", p)).collect();

        if type_params.is_empty() {
            if is_async {
                Ok(quote! {
                    async fn #fn_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            } else {
                Ok(quote! {
                    fn #fn_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            }
        } else {
            if is_async {
                Ok(quote! {
                    async fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            } else {
                Ok(quote! {
                    fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            }
        }
    }

    /// Transpiles lambda expressions
    pub fn transpile_lambda(&self, params: &[Param], body: &Expr) -> Result<TokenStream> {
        let param_names: Vec<_> = params.iter().map(|p| format_ident!("{}", p.name)).collect();
        let body_tokens = self.transpile_expr(body)?;

        // Generate closure with proper formatting (no spaces around commas)
        if param_names.is_empty() {
            Ok(quote! { || #body_tokens })
        } else {
            // Use a more controlled approach to avoid extra spaces
            let param_list = param_names.iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let closure_str = format!("|{param_list}| {body_tokens}");
            closure_str.parse().map_err(|e| anyhow::anyhow!("Failed to parse closure: {}", e))
        }
    }

    /// Transpiles function calls
    pub fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        let func_tokens = self.transpile_expr(func)?;

        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;

        // Check if this is a DataFrame constructor, column function, or macro
        if let ExprKind::Identifier(name) = &func.kind {
            if name == "col" && args.len() == 1 {
                // Special handling for col() function in DataFrame context
                if let ExprKind::Literal(Literal::String(col_name)) = &args[0].kind {
                    return Ok(quote! { polars::prelude::col(#col_name) });
                }
            } else if name == "println" || name == "print" || name == "dbg" || name == "panic" {
                // These are macros in Rust, not functions
                return Ok(quote! { #func_tokens!(#(#arg_tokens),*) });
            }
        }

        Ok(quote! { #func_tokens(#(#arg_tokens),*) })
    }

    /// Transpiles method calls
    pub fn transpile_method_call(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        let method_ident = format_ident!("{}", method);

        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;

        // Special handling for DataFrame methods
        match method {
            "select" | "filter" | "groupby" | "agg" | "sort" | "join" | "mean" | "std" | "min"
            | "max" | "sum" | "count" | "unique" | "drop_nulls" | "fill_null" | "pivot"
            | "melt" | "head" | "tail" | "sample" | "describe" => {
                // These are DataFrame operations that should be chained
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            _ => {
                // Regular method call
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
        }
    }

    /// Transpiles blocks
    pub fn transpile_block(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.is_empty() {
            return Ok(quote! { {} });
        }

        let mut statements = Vec::new();

        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;

            // Add semicolon to all but the last expression (unless it's a control flow construct)
            if i < exprs.len() - 1 || Self::needs_semicolon(&expr.kind) {
                statements.push(quote! { #expr_tokens; });
            } else {
                statements.push(expr_tokens);
            }
        }

        Ok(quote! {
            {
                #(#statements)*
            }
        })
    }

    /// Determines if an expression needs a semicolon when used as a statement
    fn needs_semicolon(kind: &ExprKind) -> bool {
        !matches!(
            kind,
            ExprKind::If { .. }
                | ExprKind::Match { .. }
                | ExprKind::For { .. }
                | ExprKind::While { .. }
                | ExprKind::Block(_)
        )
    }

    /// Transpiles pipeline expressions
    pub fn transpile_pipeline(&self, expr: &Expr, stages: &[PipelineStage]) -> Result<TokenStream> {
        let mut result = self.transpile_expr(expr)?;

        for stage in stages {
            // Each stage contains an expression to apply
            let stage_expr = &stage.op;

            // Apply the stage - check what kind of expression it is
            match &stage_expr.kind {
                ExprKind::Call { func, args } => {
                    let func_tokens = self.transpile_expr(func)?;
                    let arg_tokens: Result<Vec<_>> =
                        args.iter().map(|a| self.transpile_expr(a)).collect();
                    let arg_tokens = arg_tokens?;

                    // Pipeline passes the previous result as the first argument
                    result = quote! { #func_tokens(#result #(, #arg_tokens)*) };
                }
                ExprKind::MethodCall { method, args, .. } => {
                    let method_ident = format_ident!("{}", method);
                    let arg_tokens: Result<Vec<_>> =
                        args.iter().map(|a| self.transpile_expr(a)).collect();
                    let arg_tokens = arg_tokens?;

                    result = quote! { #result.#method_ident(#(#arg_tokens),*) };
                }
                _ => {
                    // For other expressions, apply them directly
                    let stage_tokens = self.transpile_expr(stage_expr)?;
                    result = quote! { #stage_tokens(#result) };
                }
            }
        }

        Ok(result)
    }

    /// Transpiles for loops
    pub fn transpile_for(&self, var: &str, iter: &Expr, body: &Expr) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var);
        let iter_tokens = self.transpile_expr(iter)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            for #var_ident in #iter_tokens {
                #body_tokens
            }
        })
    }

    /// Transpiles while loops
    pub fn transpile_while(&self, condition: &Expr, body: &Expr) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            while #cond_tokens {
                #body_tokens
            }
        })
    }

    /// Transpiles list comprehensions
    pub fn transpile_list_comprehension(
        &self,
        expr: &Expr,
        var: &str,
        iter: &Expr,
        filter: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var);
        let iter_tokens = self.transpile_expr(iter)?;
        let expr_tokens = self.transpile_expr(expr)?;

        if let Some(filter_expr) = filter {
            let filter_tokens = self.transpile_expr(filter_expr)?;
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter(|#var_ident| #filter_tokens)
                    .map(|#var_ident| #expr_tokens)
                    .collect::<Vec<_>>()
            })
        } else {
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .map(|#var_ident| #expr_tokens)
                    .collect::<Vec<_>>()
            })
        }
    }

    /// Transpiles try-catch blocks
    pub fn transpile_try_catch(
        &self,
        try_block: &Expr,
        catch_clauses: &[CatchClause],
        finally_block: Option<&Expr>,
    ) -> Result<TokenStream> {
        // Rust doesn't have traditional try-catch, so we need to be creative
        // We'll use a combination of Result types and match expressions

        let try_tokens = self.transpile_expr(try_block)?;

        if catch_clauses.is_empty() && finally_block.is_none() {
            // Just a try block with no handlers
            return Ok(try_tokens);
        }

        // Build the catch handling logic
        let mut catch_arms = Vec::new();

        for clause in catch_clauses {
            let var_ident = format_ident!("{}", clause.variable);
            let body_tokens = self.transpile_expr(&clause.body)?;

            if let Some(ref ty) = clause.exception_type {
                // Typed catch clause - convert string to type
                let type_ident = format_ident!("{}", ty);
                catch_arms.push(quote! {
                    Err(#var_ident) if #var_ident.is::<#type_ident>() => {
                        #body_tokens
                    }
                });
            } else {
                // Generic catch clause
                catch_arms.push(quote! {
                    Err(#var_ident) => {
                        #body_tokens
                    }
                });
            }
        }

        // Add Ok arm
        catch_arms.insert(
            0,
            quote! {
                Ok(value) => value
            },
        );

        let match_expr = quote! {
            match (|| -> Result<_, Box<dyn std::error::Error>> {
                Ok(#try_tokens)
            })() {
                #(#catch_arms,)*
            }
        };

        if let Some(finally) = finally_block {
            let finally_tokens = self.transpile_expr(finally)?;
            Ok(quote! {
                {
                    let result = #match_expr;
                    #finally_tokens;
                    result
                }
            })
        } else {
            Ok(match_expr)
        }
    }

    /// Transpiles module declarations
    pub fn transpile_module(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let module_name = format_ident!("{}", name);
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            mod #module_name {
                #body_tokens
            }
        })
    }

    /// Transpiles import statements
    pub fn transpile_import(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        use crate::frontend::ast::ImportItem;

        // Build the path as a TokenStream
        let mut path_tokens = TokenStream::new();
        let segments: Vec<_> = path.split("::").collect();
        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                path_tokens.extend(quote! { :: });
            }
            let seg_ident = format_ident!("{}", segment);
            path_tokens.extend(quote! { #seg_ident });
        }

        if items.is_empty() {
            // Simple import without specific items
            quote! { use #path_tokens::*; }
        } else if items.len() == 1 {
            match &items[0] {
                ImportItem::Named(name) => {
                    let item_ident = format_ident!("{}", name);
                    quote! { use #path_tokens::#item_ident; }
                }
                ImportItem::Aliased { name, alias } => {
                    let name_ident = format_ident!("{}", name);
                    let alias_ident = format_ident!("{}", alias);
                    quote! { use #path_tokens::#name_ident as #alias_ident; }
                }
                ImportItem::Wildcard => {
                    quote! { use #path_tokens::*; }
                }
            }
        } else {
            // Multiple items
            let item_tokens: Vec<TokenStream> = items
                .iter()
                .map(|item| match item {
                    ImportItem::Named(name) => {
                        let name_ident = format_ident!("{}", name);
                        quote! { #name_ident }
                    }
                    ImportItem::Aliased { name, alias } => {
                        let name_ident = format_ident!("{}", name);
                        let alias_ident = format_ident!("{}", alias);
                        quote! { #name_ident as #alias_ident }
                    }
                    ImportItem::Wildcard => quote! { * },
                })
                .collect();

            quote! { use #path_tokens::{#(#item_tokens),*}; }
        }
    }

    /// Transpiles export statements
    pub fn transpile_export(items: &[String]) -> TokenStream {
        let item_idents: Vec<_> = items.iter().map(|s| format_ident!("{}", s)).collect();

        if items.len() == 1 {
            let item = &item_idents[0];
            quote! { pub use #item; }
        } else {
            quote! { pub use {#(#item_idents),*}; }
        }
    }
}
