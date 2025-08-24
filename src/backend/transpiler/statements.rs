//! Statement and control flow transpilation

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]

use super::*;
use crate::frontend::ast::{Literal, Param, Pattern, PipelineStage, UnaryOp};
use anyhow::{Result, bail};
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
        // Handle Rust reserved keywords by prefixing with r#
        let safe_name = if Self::is_rust_reserved_keyword(name) {
            format!("r#{name}")
        } else {
            name.to_string()
        };
        let name_ident = format_ident!("{}", safe_name);
        
        // Convert string literals to String type at variable declaration time
        // This ensures string variables are String, not &str, making function calls work
        let value_tokens = match &value.kind {
            crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::String(s)) => {
                quote! { #s.to_string() }
            }
            _ => self.transpile_expr(value)?
        };
        
        // HOTFIX: If body is Unit, this is a top-level let statement without scoping
        if matches!(body.kind, crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Unit)) {
            if is_mutable {
                Ok(quote! { let mut #name_ident = #value_tokens })
            } else {
                Ok(quote! { let #name_ident = #value_tokens })
            }
        } else {
            // Traditional let-in expression with proper scoping
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
    }

    /// Transpiles function definitions
    #[allow(clippy::too_many_arguments)]
    pub fn transpile_function(
        &self,
        name: &str,
        type_params: &[String],
        params: &[Param],
        body: &Expr,
        is_async: bool,
        return_type: Option<&Type>,
        is_pub: bool,
    ) -> Result<TokenStream> {
        let fn_name = format_ident!("{}", name);

        let param_tokens: Vec<TokenStream> = params
            .iter()
            .map(|p| {
                let param_name = format_ident!("{}", p.name());
                // HOTFIX: For function signatures, use single generic type T for inferred types
                let type_tokens = if let Ok(tokens) = self.transpile_type(&p.ty) {
                    let token_str = tokens.to_string();
                    if token_str == "_" {
                        // Use single generic type T for all inferred parameters
                        quote! { T }
                    } else {
                        tokens
                    }
                } else { quote! { T } };
                quote! { #param_name: #type_tokens }
            })
            .collect();

        // HOTFIX: Add inferred generic type parameters with appropriate trait bounds
        let mut all_type_params = type_params.to_vec();
        let mut has_inferred_types = false;
        for p in params {
            let type_tokens = self.transpile_type(&p.ty).unwrap_or_else(|_| quote! { _ });
            let token_str = type_tokens.to_string();
            if token_str == "_" {
                // Use a single generic type T for all inferred parameters (for operations like addition)
                if !has_inferred_types {
                    all_type_params.push("T: std::ops::Add<Output=T> + std::ops::Mul<Output=T> + std::fmt::Display + std::fmt::Debug + Clone".to_string());
                    has_inferred_types = true;
                }
            }
        }

        let body_tokens = if is_async {
            let mut async_transpiler = Transpiler::new();
            async_transpiler.in_async_context = true;
            async_transpiler.transpile_expr(body)?
        } else {
            self.transpile_expr(body)?
        };

        // HOTFIX: Infer return type for functions with inferred parameters
        let return_type_tokens = if let Some(ty) = return_type {
            let ty_tokens = self.transpile_type(ty)?;
            quote! { -> #ty_tokens }
        } else if has_inferred_types {
            // If we have inferred parameters, likely returning the same type
            quote! { -> T }
        } else {
            quote! {}
        };

        // HOTFIX: Handle complex trait bounds that can't use format_ident
        let type_param_tokens: Vec<TokenStream> = all_type_params
            .iter()
            .map(|p| {
                if p.contains(':') {
                    // Complex trait bound - parse as TokenStream
                    p.parse().unwrap_or_else(|_| quote! { T })
                } else {
                    // Simple type parameter
                    let ident = format_ident!("{}", p);
                    quote! { #ident }
                }
            })
            .collect();

        let visibility = if is_pub { quote! { pub } } else { quote! {} };

        if all_type_params.is_empty() {
            if is_async {
                Ok(quote! {
                    #visibility async fn #fn_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            } else {
                Ok(quote! {
                    #visibility fn #fn_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            }
        } else {
            if is_async {
                Ok(quote! {
                    #visibility async fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            } else {
                Ok(quote! {
                    #visibility fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            }
        }
    }

    /// Transpiles lambda expressions
    pub fn transpile_lambda(&self, params: &[Param], body: &Expr) -> Result<TokenStream> {
        let param_names: Vec<_> = params
            .iter()
            .map(|p| format_ident!("{}", p.name()))
            .collect();
        let body_tokens = self.transpile_expr(body)?;

        // Generate closure with proper formatting (no spaces around commas)
        if param_names.is_empty() {
            Ok(quote! { || #body_tokens })
        } else {
            // Use a more controlled approach to avoid extra spaces
            let param_list = param_names
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let closure_str = format!("|{param_list}| {body_tokens}");
            closure_str
                .parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse closure: {}", e))
        }
    }

    /// Transpiles function calls
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"println("Hello, {}", name)"#);
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains(r#"println ! ( "Hello, {}" , name )"#));
    /// ```
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"println("Simple message")"#);
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains(r#"println ! ( "Simple message" )"#));
    /// ```
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("some_function(\"test\")");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains(r#"some_function ( "test" . to_string ( ) )"#));
    /// ```
    pub fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        let func_tokens = self.transpile_expr(func)?;

        // Check if this is a macro first (before string conversion)
        if let ExprKind::Identifier(name) = &func.kind {
            // Handle Rust-style macro syntax (println!, assert!, etc.)
            let base_name = if name.ends_with('!') {
                name.strip_suffix('!').unwrap()
            } else {
                name
            };
            
            if base_name == "println" || base_name == "print" || base_name == "dbg" || base_name == "panic" {
                // These are macros in Rust, not functions
                // Special handling for string interpolation in println/print
                if (base_name == "println" || base_name == "print") && args.len() == 1 {
                    if let ExprKind::StringInterpolation { parts } = &args[0].kind {
                        // Generate println!/print! with format string directly
                        return self.transpile_print_with_interpolation(base_name, parts);
                    }
                    // For single non-string arguments, add "{}" format string
                    if !matches!(&args[0].kind, ExprKind::Literal(Literal::String(_))) {
                        let arg_tokens = self.transpile_expr(&args[0])?;
                        let format_str = "{}";
                        return Ok(quote! { #func_tokens!(#format_str, #arg_tokens) });
                    }
                }
                // For multiple arguments with first being a string literal OR string interpolation, treat as format string + args
                if args.len() > 1 {
                    match &args[0].kind {
                        ExprKind::Literal(Literal::String(_)) => {
                            // First argument is format string literal, remaining are format arguments  
                            let format_arg = self.transpile_expr(&args[0])?;
                            let format_args: Result<Vec<_>> = args[1..].iter().map(|a| self.transpile_expr(a)).collect();
                            let format_args = format_args?;
                            return Ok(quote! { #func_tokens!(#format_arg, #(#format_args),*) });
                        }
                        ExprKind::StringInterpolation { parts } => {
                            // Handle format string with printf-style formatting
                            let format_str = self.build_printf_format_string(parts)?;
                            let format_args: Result<Vec<_>> = args[1..].iter().map(|a| self.transpile_expr(a)).collect();
                            let format_args = format_args?;
                            return Ok(quote! { #func_tokens!(#format_str, #(#format_args),*) });
                        }
                        _ => {
                            // Generate format string with placeholders for all arguments
                            let format_str = (0..args.len()).map(|_| "{}").collect::<Vec<_>>().join(" ");
                            let all_args: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
                            let all_args = all_args?;
                            return Ok(quote! { #func_tokens!(#format_str, #(#all_args),*) });
                        }
                    }
                }
                // Single string literal - use as-is for macros
                let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
                let arg_tokens = arg_tokens?;
                return Ok(quote! { #func_tokens!(#(#arg_tokens),*) });
            }
            
            // Math functions - generate method calls or use std functions
            if base_name == "sqrt" && args.len() == 1 {
                let arg = self.transpile_expr(&args[0])?;
                return Ok(quote! { (#arg as f64).sqrt() });
            }
            if base_name == "pow" && args.len() == 2 {
                let base = self.transpile_expr(&args[0])?;
                let exp = self.transpile_expr(&args[1])?;
                return Ok(quote! { (#base as f64).powf(#exp as f64) });
            }
            if base_name == "abs" && args.len() == 1 {
                let arg = self.transpile_expr(&args[0])?;
                // Check if arg is negative literal to handle type
                if let ExprKind::Unary { op: UnaryOp::Negate, operand } = &args[0].kind {
                    if matches!(&operand.kind, ExprKind::Literal(Literal::Float(_))) {
                        return Ok(quote! { (#arg).abs() });
                    }
                }
                // For all other cases, try both int and float abs
                return Ok(quote! { #arg.abs() });
            }
            if base_name == "min" && args.len() == 2 {
                let a = self.transpile_expr(&args[0])?;
                let b = self.transpile_expr(&args[1])?;
                // Check if args are float literals to determine type
                let is_float = matches!(&args[0].kind, ExprKind::Literal(Literal::Float(_))) 
                    || matches!(&args[1].kind, ExprKind::Literal(Literal::Float(_)));
                if is_float {
                    return Ok(quote! { (#a as f64).min(#b as f64) });
                }
                return Ok(quote! { std::cmp::min(#a, #b) });
            }
            if base_name == "max" && args.len() == 2 {
                let a = self.transpile_expr(&args[0])?;
                let b = self.transpile_expr(&args[1])?;
                // Check if args are float literals to determine type
                let is_float = matches!(&args[0].kind, ExprKind::Literal(Literal::Float(_))) 
                    || matches!(&args[1].kind, ExprKind::Literal(Literal::Float(_)));
                if is_float {
                    return Ok(quote! { (#a as f64).max(#b as f64) });
                }
                return Ok(quote! { std::cmp::max(#a, #b) });
            }
            if base_name == "floor" && args.len() == 1 {
                let arg = self.transpile_expr(&args[0])?;
                return Ok(quote! { (#arg as f64).floor() });
            }
            if base_name == "ceil" && args.len() == 1 {
                let arg = self.transpile_expr(&args[0])?;
                return Ok(quote! { (#arg as f64).ceil() });
            }
            if base_name == "round" && args.len() == 1 {
                let arg = self.transpile_expr(&args[0])?;
                return Ok(quote! { (#arg as f64).round() });
            }
            
            // Input functions
            if base_name == "input" {
                if args.len() > 1 {
                    bail!("input expects 0 or 1 arguments (optional prompt)");
                }
                if args.is_empty() {
                    return Ok(quote! { 
                        {
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).expect("Failed to read input");
                            if input.ends_with('\n') {
                                input.pop();
                                if input.ends_with('\r') {
                                    input.pop();
                                }
                            }
                            input
                        }
                    });
                } else {
                    let prompt = self.transpile_expr(&args[0])?;
                    return Ok(quote! { 
                        {
                            print!("{}", #prompt);
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).expect("Failed to read input");
                            if input.ends_with('\n') {
                                input.pop();
                                if input.ends_with('\r') {
                                    input.pop();
                                }
                            }
                            input
                        }
                    });
                }
            }
            
            if base_name == "readline" && args.is_empty() {
                return Ok(quote! { 
                    {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).expect("Failed to read input");
                        if input.ends_with('\n') {
                            input.pop();
                            if input.ends_with('\r') {
                                input.pop();
                            }
                        }
                        input
                    }
                });
            }
            
            // Assert functions - generate Rust assert macros
            if base_name == "assert" {
                if args.len() < 1 || args.len() > 2 {
                    bail!("assert expects 1 or 2 arguments (condition, optional message)");
                }
                let condition = self.transpile_expr(&args[0])?;
                if args.len() == 1 {
                    return Ok(quote! { assert!(#condition) });
                } else {
                    let message = self.transpile_expr(&args[1])?;
                    return Ok(quote! { assert!(#condition, "{}", #message) });
                }
            }
            
            if base_name == "assert_eq" {
                if args.len() < 2 || args.len() > 3 {
                    bail!("assert_eq expects 2 or 3 arguments (left, right, optional message)");
                }
                let left = self.transpile_expr(&args[0])?;
                let right = self.transpile_expr(&args[1])?;
                if args.len() == 2 {
                    return Ok(quote! { assert_eq!(#left, #right) });
                } else {
                    let message = self.transpile_expr(&args[2])?;
                    return Ok(quote! { assert_eq!(#left, #right, "{}", #message) });
                }
            }
            
            if base_name == "assert_ne" {
                if args.len() < 2 || args.len() > 3 {
                    bail!("assert_ne expects 2 or 3 arguments (left, right, optional message)");
                }
                let left = self.transpile_expr(&args[0])?;
                let right = self.transpile_expr(&args[1])?;
                if args.len() == 2 {
                    return Ok(quote! { assert_ne!(#left, #right) });
                } else {
                    let message = self.transpile_expr(&args[2])?;
                    return Ok(quote! { assert_ne!(#left, #right, "{}", #message) });
                }
            }
            
            // Collection constructors
            if base_name == "HashMap" && args.is_empty() {
                return Ok(quote! { std::collections::HashMap::new() });
            }
            if base_name == "HashSet" && args.is_empty() {
                return Ok(quote! { std::collections::HashSet::new() });
            }
        }

        // For regular function calls, convert string literals to String
        // Variable conversion is more complex and handled via Rust's type inference
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| {
            match &a.kind {
                ExprKind::Literal(Literal::String(s)) => {
                    Ok(quote! { #s.to_string() })
                }
                _ => self.transpile_expr(a)
            }
        }).collect();
        let arg_tokens = arg_tokens?;

        // Check if this is a DataFrame constructor or column function
        if let ExprKind::Identifier(name) = &func.kind {
            let base_name = if name.ends_with('!') {
                name.strip_suffix('!').unwrap()
            } else {
                name
            };
            if base_name == "col" && args.len() == 1 {
                // Special handling for col() function in DataFrame context
                if let ExprKind::Literal(Literal::String(col_name)) = &args[0].kind {
                    return Ok(quote! { polars::prelude::col(#col_name) });
                }
            }
        }

        Ok(quote! { #func_tokens(#(#arg_tokens),*) })
    }

    /// Build printf-style format string for macros (preserves {} as format specifiers)
    fn build_printf_format_string(&self, parts: &[crate::frontend::ast::StringPart]) -> Result<TokenStream> {
        let mut format_string = String::new();
        
        for part in parts {
            match part {
                crate::frontend::ast::StringPart::Text(s) => {
                    // Don't escape {} in printf context - they are format specifiers
                    format_string.push_str(s);
                }
                crate::frontend::ast::StringPart::Expr(_) => {
                    // String interpolation expressions become {} placeholders
                    format_string.push_str("{}");
                }
            }
        }
        
        Ok(quote! { #format_string })
    }

    /// Transpiles println/print with string interpolation directly
    fn transpile_print_with_interpolation(
        &self,
        func_name: &str,
        parts: &[crate::frontend::ast::StringPart],
    ) -> Result<TokenStream> {
        if parts.is_empty() {
            let func_tokens = proc_macro2::Ident::new(func_name, proc_macro2::Span::call_site());
            return Ok(quote! { #func_tokens!("") });
        }

        let mut format_string = String::new();
        let mut args = Vec::new();

        for part in parts {
            match part {
                crate::frontend::ast::StringPart::Text(s) => {
                    // Escape any format specifiers in literal parts
                    format_string.push_str(&s.replace('{', "{{").replace('}', "}}"));
                }
                crate::frontend::ast::StringPart::Expr(expr) => {
                    format_string.push_str("{}");
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
            }
        }

        let func_tokens = proc_macro2::Ident::new(func_name, proc_macro2::Span::call_site());

        Ok(quote! {
            #func_tokens!(#format_string #(, #args)*)
        })
    }

    /// Transpiles method calls
    #[allow(clippy::cognitive_complexity)]
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

        // Special handling for collection and DataFrame methods
        match method {
            // Vec collection methods - transform to iterator operations
            "map" => {
                // vec.map(f) -> vec.iter().map(f).collect::<Vec<_>>()
                Ok(quote! { #obj_tokens.iter().map(#(#arg_tokens),*).collect::<Vec<_>>() })
            }
            "filter" => {
                // vec.filter(f) -> vec.into_iter().filter(f).collect()
                Ok(quote! { #obj_tokens.into_iter().filter(#(#arg_tokens),*).collect() })
            }
            "reduce" => {
                // vec.reduce(f) -> vec.into_iter().reduce(f)
                Ok(quote! { #obj_tokens.into_iter().reduce(#(#arg_tokens),*) })
            }
            
            // HashMap/HashSet specific methods
            "get" => {
                // HashMap.get() returns Option<&V>, but we want owned values
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*).cloned() })
            }
            "contains_key" | "keys" | "values" | "entry" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            "contains" => {
                // HashSet contains method
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            "union" | "intersection" | "difference" | "symmetric_difference" => {
                // HashSet set operations
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            
            // Common collection methods (Vec, HashMap, HashSet)
            "insert" | "remove" | "clear" | "len" | "is_empty" | "iter" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            
            // DataFrame operations that should be chained
            "select" | "groupby" | "agg" | "sort" | "join" | "mean" | "std" | "min"
            | "max" | "sum" | "count" | "unique" | "drop_nulls" | "fill_null" | "pivot"
            | "melt" | "head" | "tail" | "sample" | "describe" => {
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

            // HOTFIX: Never add semicolon to the last expression in a block (it should be the return value)  
            if i < exprs.len() - 1 {
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
    pub fn transpile_for(&self, var: &str, pattern: Option<&Pattern>, iter: &Expr, body: &Expr) -> Result<TokenStream> {
        let iter_tokens = self.transpile_expr(iter)?;
        let body_tokens = self.transpile_expr(body)?;

        // If we have a pattern, use it for destructuring
        if let Some(pat) = pattern {
            let pattern_tokens = self.transpile_pattern(pat)?;
            Ok(quote! {
                for #pattern_tokens in #iter_tokens {
                    #body_tokens
                }
            })
        } else {
            // Fall back to simple variable
            let var_ident = format_ident!("{}", var);
            Ok(quote! {
                for #var_ident in #iter_tokens {
                    #body_tokens
                }
            })
        }
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

    /// Transpile if-let expression (complexity: 5)
    pub fn transpile_if_let(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let then_tokens = self.transpile_expr(then_branch)?;

        if let Some(else_expr) = else_branch {
            let else_tokens = self.transpile_expr(else_expr)?;
            Ok(quote! {
                if let #pattern_tokens = #expr_tokens {
                    #then_tokens
                } else {
                    #else_tokens
                }
            })
        } else {
            Ok(quote! {
                if let #pattern_tokens = #expr_tokens {
                    #then_tokens
                }
            })
        }
    }

    /// Transpile while-let expression (complexity: 4)
    pub fn transpile_while_let(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            while let #pattern_tokens = #expr_tokens {
                #body_tokens
            }
        })
    }

    pub fn transpile_loop(&self, body: &Expr) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            loop {
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
                    // Check if the path already ends with the item name
                    // This happens when parsing "use math::add"
                    if path.ends_with(&format!("::{name}")) {
                        // Path already includes the item, just use it directly
                        quote! { use #path_tokens; }
                    } else {
                        // Path doesn't include item, append it
                        let item_ident = format_ident!("{}", name);
                        quote! { use #path_tokens::#item_ident; }
                    }
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
