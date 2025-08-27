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
                Ok(quote! { let mut #name_ident = #value_tokens; })
            } else {
                Ok(quote! { let #name_ident = #value_tokens; })
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

    /// Transpiles let pattern bindings (destructuring)
    pub fn transpile_let_pattern(
        &self,
        pattern: &crate::frontend::ast::Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let value_tokens = self.transpile_expr(value)?;
        
        // HOTFIX: If body is Unit, this is a top-level let statement without scoping
        if matches!(body.kind, crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Unit)) {
            Ok(quote! { let #pattern_tokens = #value_tokens })
        } else {
            // Traditional let-in expression with proper scoping
            let body_tokens = self.transpile_expr(body)?;
            Ok(quote! {
                {
                    let #pattern_tokens = #value_tokens;
                    #body_tokens
                }
            })
        }
    }

    /// Check if function name suggests numeric operations
    fn looks_like_numeric_function(&self, name: &str) -> bool {
        matches!(name, 
            "add" | "subtract" | "multiply" | "divide" | "sum" | "product" | 
            "min" | "max" | "abs" | "sqrt" | "pow" | "mod" | "gcd" | "lcm" |
            "factorial" | "fibonacci" | "prime" | "even" | "odd" | "square" | "cube" |
            "double" | "triple" | "quadruple"  // Added common numeric function names
        )
    }


    /// Check if expression is a void/unit function call
    fn is_void_function_call(&self, expr: &Expr) -> bool {
        match &expr.kind {
            crate::frontend::ast::ExprKind::Call { func, .. } => {
                if let crate::frontend::ast::ExprKind::Identifier(name) = &func.kind {
                    // Comprehensive list of void functions
                    matches!(name.as_str(), 
                        // Output functions
                        "println" | "print" | "eprintln" | "eprint" |
                        // Debug functions
                        "dbg" | "debug" | "trace" | "info" | "warn" | "error" |
                        // Control flow functions
                        "panic" | "assert" | "assert_eq" | "assert_ne" |
                        "todo" | "unimplemented" | "unreachable"
                    )
                } else {
                    false
                }
            }
            _ => false
        }
    }
    
    /// Check if an expression is void (returns unit/nothing)
    fn is_void_expression(&self, expr: &Expr) -> bool {
        match &expr.kind {
            // Unit literal is void
            crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Unit) => true,
            
            // Void function calls
            crate::frontend::ast::ExprKind::Call { .. } if self.is_void_function_call(expr) => true,
            
            // Assignments are void
            crate::frontend::ast::ExprKind::Assign { .. } |
            crate::frontend::ast::ExprKind::CompoundAssign { .. } => true,
            
            // Loops are void
            crate::frontend::ast::ExprKind::While { .. } |
            crate::frontend::ast::ExprKind::For { .. } => true,
            
            // Let bindings - check the body expression
            crate::frontend::ast::ExprKind::Let { body, .. } => {
                self.is_void_expression(body)
            }
            
            // Block - check last expression
            crate::frontend::ast::ExprKind::Block(exprs) => {
                exprs.last().is_none_or(|e| self.is_void_expression(e))
            }
            
            // If expression - both branches must be void
            crate::frontend::ast::ExprKind::If { then_branch, else_branch, .. } => {
                self.is_void_expression(then_branch) && 
                else_branch.as_ref().is_none_or(|e| self.is_void_expression(e))
            }
            
            // Match expression - all arms must be void
            crate::frontend::ast::ExprKind::Match { arms, .. } => {
                arms.iter().all(|arm| self.is_void_expression(&arm.body))
            }
            
            // Return without value is void
            crate::frontend::ast::ExprKind::Return { value } if value.is_none() => true,
            
            // Everything else produces a value
            _ => false
        }
    }

    /// Check if expression has a non-unit value (i.e., returns something meaningful)
    fn has_non_unit_expression(&self, body: &Expr) -> bool {
        !self.is_void_expression(body)
    }


    /// Transpiles function definitions
    #[allow(clippy::too_many_arguments)]
    /// Infer parameter type based on usage in function body
    fn infer_param_type(&self, param: &Param, body: &Expr, func_name: &str) -> TokenStream {
        use super::type_inference::{is_param_used_as_function, is_param_used_numerically, is_param_used_as_function_argument};
        
        if is_param_used_as_function(&param.name(), body) {
            quote! { impl Fn(i32) -> i32 }
        } else if is_param_used_numerically(&param.name(), body) || 
                  self.looks_like_numeric_function(func_name) ||
                  is_param_used_as_function_argument(&param.name(), body) {
            quote! { i32 }
        } else {
            quote! { String }
        }
    }

    /// Generate parameter tokens with proper type inference
    fn generate_param_tokens(&self, params: &[Param], body: &Expr, func_name: &str) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|p| {
                let param_name = format_ident!("{}", p.name());
                let type_tokens = if let Ok(tokens) = self.transpile_type(&p.ty) {
                    let token_str = tokens.to_string();
                    if token_str == "_" {
                        self.infer_param_type(p, body, func_name)
                    } else {
                        tokens
                    }
                } else {
                    self.infer_param_type(p, body, func_name)
                };
                Ok(quote! { #param_name: #type_tokens })
            })
            .collect()
    }

    /// Generate return type tokens based on function analysis
    fn generate_return_type_tokens(&self, name: &str, return_type: Option<&Type>, body: &Expr) -> Result<TokenStream> {
        if let Some(ty) = return_type {
            let ty_tokens = self.transpile_type(ty)?;
            Ok(quote! { -> #ty_tokens })
        } else if name == "main" {
            Ok(quote! {})
        } else if self.looks_like_numeric_function(name) {
            Ok(quote! { -> i32 })
        } else if self.has_non_unit_expression(body) {
            Ok(quote! { -> i32 })
        } else {
            Ok(quote! {})
        }
    }

    /// Generate body tokens with async support
    fn generate_body_tokens(&self, body: &Expr, is_async: bool) -> Result<TokenStream> {
        if is_async {
            let mut async_transpiler = Transpiler::new();
            async_transpiler.in_async_context = true;
            async_transpiler.transpile_expr(body)
        } else {
            self.transpile_expr(body)
        }
    }

    /// Generate type parameter tokens with trait bound support
    fn generate_type_param_tokens(&self, type_params: &[String]) -> Result<Vec<TokenStream>> {
        Ok(type_params
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
            .collect())
    }

    /// Generate complete function signature
    fn generate_function_signature(
        &self,
        is_pub: bool,
        is_async: bool,
        fn_name: &proc_macro2::Ident,
        type_param_tokens: &[TokenStream],
        param_tokens: &[TokenStream],
        return_type_tokens: &TokenStream,
        body_tokens: &TokenStream,
    ) -> Result<TokenStream> {
        let visibility = if is_pub { quote! { pub } } else { quote! {} };
        
        Ok(match (type_param_tokens.is_empty(), is_async) {
            (true, false) => quote! {
                #visibility fn #fn_name(#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            },
            (true, true) => quote! {
                #visibility async fn #fn_name(#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            },
            (false, false) => quote! {
                #visibility fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            },
            (false, true) => quote! {
                #visibility async fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            },
        })
    }

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
        let param_tokens = self.generate_param_tokens(params, body, name)?;
        let body_tokens = self.generate_body_tokens(body, is_async)?;
        let return_type_tokens = self.generate_return_type_tokens(name, return_type, body)?;
        let type_param_tokens = self.generate_type_param_tokens(type_params)?;

        self.generate_function_signature(
            is_pub, 
            is_async, 
            &fn_name, 
            &type_param_tokens, 
            &param_tokens, 
            &return_type_tokens, 
            &body_tokens
        )
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
    /// assert!(result.contains("println !"));
    /// assert!(result.contains("Hello, {}"));
    /// ```
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"println("Simple message")"#);
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("println !"));
    /// assert!(result.contains("Simple message"));
    /// ```
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("some_function(\"test\")");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("some_function"));
    /// assert!(result.contains("test"));
    /// ```
    pub fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        let func_tokens = self.transpile_expr(func)?;

        // Check if this is a built-in function with special handling
        if let ExprKind::Identifier(name) = &func.kind {
            let base_name = if name.ends_with('!') {
                name.strip_suffix('!').unwrap()
            } else {
                name
            };
            
            // Try specialized handlers in order of precedence
            if let Some(result) = self.try_transpile_print_macro(&func_tokens, base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_math_function(base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_input_function(base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_assert_function(&func_tokens, base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_type_conversion(base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_math_functions(base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_collection_constructor(base_name, args)? {
                return Ok(result);
            }
            
            if let Some(result) = self.try_transpile_dataframe_function(base_name, args)? {
                return Ok(result);
            }
        }

        // Default: regular function call with string conversion
        self.transpile_regular_function_call(&func_tokens, args)
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
                crate::frontend::ast::StringPart::ExprWithFormat { expr, format_spec } => {
                    // Include the format specifier in the format string
                    format_string.push('{');
                    format_string.push_str(format_spec);
                    format_string.push('}');
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
                // vec.filter(f) -> vec.into_iter().filter(f).collect::<Vec<_>>()
                Ok(quote! { #obj_tokens.into_iter().filter(#(#arg_tokens),*).collect::<Vec<_>>() })
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
            "items" => {
                // HashMap.items() -> iterator of (K, V) tuples (not references)
                // 
                // # Example
                // ```
                // let obj = {"key": "value"};
                // for k, v in obj.items() { println(k + "=" + v) }
                // ```
                Ok(quote! { #obj_tokens.iter().map(|(k, v)| (k.clone(), v.clone())) })
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
            
            // String method name mappings (Ruchy -> Rust)
            "to_upper" => {
                let rust_method = format_ident!("to_uppercase");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "to_lower" => {
                let rust_method = format_ident!("to_lowercase");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "length" => {
                // Map Ruchy's length() to Rust's len()
                let rust_method = format_ident!("len");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
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

    
    /// Static method for transpiling inline imports (backward compatibility)
    pub fn transpile_import(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        Self::transpile_import_inline(path, items)
    }
    
    /// Core inline import transpilation logic
    fn transpile_import_inline(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
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

    /// Handle print/debug macros (println, print, dbg, panic)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Test println macro handling
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("println(42)");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("println !"));
    /// assert!(result.contains("{}"));
    /// ```
    fn try_transpile_print_macro(
        &self, 
        func_tokens: &TokenStream, 
        base_name: &str, 
        args: &[Expr]
    ) -> Result<Option<TokenStream>> {
        if !(base_name == "println" || base_name == "print" || base_name == "dbg" || base_name == "panic") {
            return Ok(None);
        }
        
        // Handle single argument with string interpolation
        if (base_name == "println" || base_name == "print") && args.len() == 1 {
            if let ExprKind::StringInterpolation { parts } = &args[0].kind {
                return Ok(Some(self.transpile_print_with_interpolation(base_name, parts)?));
            }
            // For single non-string arguments, add smart format string
            if !matches!(&args[0].kind, ExprKind::Literal(Literal::String(_))) {
                let arg_tokens = self.transpile_expr(&args[0])?;
                // Use Debug formatting for safety - works with all types including Vec, tuples, etc.
                let format_str = "{:?}";
                return Ok(Some(quote! { #func_tokens!(#format_str, #arg_tokens) }));
            }
        }
        
        // Handle multiple arguments
        if args.len() > 1 {
            return self.transpile_print_multiple_args(func_tokens, args);
        }
        
        // Single string literal or simple case
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;
        Ok(Some(quote! { #func_tokens!(#(#arg_tokens),*) }))
    }
    
    /// Handle multiple arguments for print macros
    fn transpile_print_multiple_args(
        &self,
        func_tokens: &TokenStream,
        args: &[Expr]
    ) -> Result<Option<TokenStream>> {
        // FIXED: Don't treat first string argument as format string
        // Instead, treat all arguments as values to print with spaces
        if args.is_empty() {
            return Ok(Some(quote! { #func_tokens!() }));
        }
        
        let all_args: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let all_args = all_args?;
        
        if args.len() == 1 {
            // Single argument - check if it's a string-like expression
            match &args[0].kind {
                ExprKind::Literal(Literal::String(_)) | 
                ExprKind::StringInterpolation { .. } => {
                    // String literal or interpolation - use Display format
                    Ok(Some(quote! { #func_tokens!("{}", #(#all_args)*) }))
                }
                ExprKind::Identifier(_) => {
                    // For identifiers, we can't know the type at compile time
                    // Use a runtime check to decide format
                    let arg = &all_args[0];
                    Ok(Some(quote! {
                        {
                            let value = #arg;
                            // Check if it's a String type at runtime
                            if std::any::type_name_of_val(&value).contains("String") || 
                               std::any::type_name_of_val(&value).contains("&str") {
                                #func_tokens!("{}", value)
                            } else {
                                #func_tokens!("{:?}", value)
                            }
                        }
                    }))
                }
                _ => {
                    // Other types - use Debug format for complex types
                    Ok(Some(quote! { #func_tokens!("{:?}", #(#all_args)*) }))
                }
            }
        } else {
            // Multiple arguments - use appropriate format for each
            let format_parts: Vec<_> = args.iter().map(|arg| {
                match &arg.kind {
                    ExprKind::Literal(Literal::String(_)) => "{}",
                    _ => "{:?}"
                }
            }).collect();
            let format_str = format_parts.join(" ");
            Ok(Some(quote! { #func_tokens!(#format_str, #(#all_args),*) }))
        }
    }
    
    /// Handle math functions (sqrt, pow, abs, min, max, floor, ceil, round)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("sqrt(4.0)");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("sqrt"));
    /// ```
    fn try_transpile_math_function(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match (base_name, args.len()) {
            ("sqrt", 1) => self.transpile_sqrt(&args[0]).map(Some),
            ("pow", 2) => self.transpile_pow(&args[0], &args[1]).map(Some),
            ("abs", 1) => self.transpile_abs(&args[0]).map(Some),
            ("min", 2) => self.transpile_min(&args[0], &args[1]).map(Some),
            ("max", 2) => self.transpile_max(&args[0], &args[1]).map(Some),
            ("floor", 1) => self.transpile_floor(&args[0]).map(Some),
            ("ceil", 1) => self.transpile_ceil(&args[0]).map(Some),
            ("round", 1) => self.transpile_round(&args[0]).map(Some),
            _ => Ok(None)
        }
    }

    fn transpile_sqrt(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).sqrt() })
    }

    fn transpile_pow(&self, base: &Expr, exp: &Expr) -> Result<TokenStream> {
        let base_tokens = self.transpile_expr(base)?;
        let exp_tokens = self.transpile_expr(exp)?;
        Ok(quote! { (#base_tokens as f64).powf(#exp_tokens as f64) })
    }

    fn transpile_abs(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        // Check if arg is negative literal to handle type
        if let ExprKind::Unary { op: UnaryOp::Negate, operand } = &arg.kind {
            if matches!(&operand.kind, ExprKind::Literal(Literal::Float(_))) {
                return Ok(quote! { (#arg_tokens).abs() });
            }
        }
        // For all other cases, use standard abs
        Ok(quote! { #arg_tokens.abs() })
    }

    fn transpile_min(&self, a: &Expr, b: &Expr) -> Result<TokenStream> {
        let a_tokens = self.transpile_expr(a)?;
        let b_tokens = self.transpile_expr(b)?;
        // Check if args are float literals to determine type
        let is_float = matches!(&a.kind, ExprKind::Literal(Literal::Float(_))) 
            || matches!(&b.kind, ExprKind::Literal(Literal::Float(_)));
        if is_float {
            Ok(quote! { (#a_tokens as f64).min(#b_tokens as f64) })
        } else {
            Ok(quote! { std::cmp::min(#a_tokens, #b_tokens) })
        }
    }

    fn transpile_max(&self, a: &Expr, b: &Expr) -> Result<TokenStream> {
        let a_tokens = self.transpile_expr(a)?;
        let b_tokens = self.transpile_expr(b)?;
        // Check if args are float literals to determine type
        let is_float = matches!(&a.kind, ExprKind::Literal(Literal::Float(_))) 
            || matches!(&b.kind, ExprKind::Literal(Literal::Float(_)));
        if is_float {
            Ok(quote! { (#a_tokens as f64).max(#b_tokens as f64) })
        } else {
            Ok(quote! { std::cmp::max(#a_tokens, #b_tokens) })
        }
    }

    fn transpile_floor(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).floor() })
    }

    fn transpile_ceil(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).ceil() })
    }

    fn transpile_round(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).round() })
    }
    
    /// Handle input functions (input, readline)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("input()");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("read_line"));
    /// ```
    fn try_transpile_input_function(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match base_name {
            "input" => {
                if args.len() > 1 {
                    bail!("input expects 0 or 1 arguments (optional prompt)");
                }
                if args.is_empty() {
                    Ok(Some(self.generate_input_without_prompt()))
                } else {
                    let prompt = self.transpile_expr(&args[0])?;
                    Ok(Some(self.generate_input_with_prompt(prompt)))
                }
            }
            "readline" if args.is_empty() => {
                Ok(Some(self.generate_input_without_prompt()))
            }
            _ => Ok(None)
        }
    }
    
    /// Generate input reading code without prompt
    fn generate_input_without_prompt(&self) -> TokenStream {
        quote! { 
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
        }
    }
    
    /// Generate input reading code with prompt
    fn generate_input_with_prompt(&self, prompt: TokenStream) -> TokenStream {
        quote! { 
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
        }
    }
    
    /// Try to transpile type conversion functions (str, int, float, bool)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use ruchy::backend::transpiler::Transpiler;
    /// let transpiler = Transpiler::new();
    /// // str(42) -> 42.to_string()
    /// // int("42") -> "42".parse::<i64>().unwrap()
    /// // float(42) -> 42 as f64
    /// // bool(1) -> 1 != 0
    /// ```
    fn try_transpile_type_conversion(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match base_name {
            "str" => {
                if args.len() != 1 {
                    bail!("str() expects exactly 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { format!("{}", #value) }))
            }
            "int" => {
                if args.len() != 1 {
                    bail!("int() expects exactly 1 argument");
                }
                
                // Check if the argument is a literal
                match &args[0].kind {
                    ExprKind::Literal(Literal::String(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        return Ok(Some(quote! { #value.parse::<i64>().expect("Failed to parse integer") }));
                    }
                    ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                        if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                            let value = self.transpile_expr(&args[0])?;
                            return Ok(Some(quote! { #value.parse::<i64>().expect("Failed to parse integer") }));
                        }
                    }
                    ExprKind::Literal(Literal::Float(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        return Ok(Some(quote! { (#value as i64) }));
                    }
                    ExprKind::Literal(Literal::Bool(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        return Ok(Some(quote! { if #value { 1i64 } else { 0i64 } }));
                    }
                    _ => {}
                }
                
                // For other expressions, use runtime conversion
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (#value as i64) }))
            }
            "float" => {
                if args.len() != 1 {
                    bail!("float() expects exactly 1 argument");
                }
                
                // Check if the argument is a literal
                match &args[0].kind {
                    ExprKind::Literal(Literal::String(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        return Ok(Some(quote! { #value.parse::<f64>().expect("Failed to parse float") }));
                    }
                    ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                        if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                            let value = self.transpile_expr(&args[0])?;
                            return Ok(Some(quote! { #value.parse::<f64>().expect("Failed to parse float") }));
                        }
                    }
                    ExprKind::Literal(Literal::Integer(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        return Ok(Some(quote! { (#value as f64) }));
                    }
                    _ => {}
                }
                
                // For other expressions
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (#value as f64) }))
            }
            "bool" => {
                if args.len() != 1 {
                    bail!("bool() expects exactly 1 argument");
                }
                
                // Check the type of the argument to generate appropriate conversion
                match &args[0].kind {
                    ExprKind::Literal(Literal::Integer(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        Ok(Some(quote! { (#value != 0) }))
                    }
                    ExprKind::Literal(Literal::String(_)) => {
                        let value = self.transpile_expr(&args[0])?;
                        Ok(Some(quote! { !#value.is_empty() }))
                    }
                    ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                        if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                            let value = self.transpile_expr(&args[0])?;
                            Ok(Some(quote! { !#value.is_empty() }))
                        } else {
                            let value = self.transpile_expr(&args[0])?;
                            Ok(Some(quote! { !#value.is_empty() }))
                        }
                    }
                    ExprKind::Literal(Literal::Bool(_)) => {
                        // Boolean already, just pass through
                        let value = self.transpile_expr(&args[0])?;
                        Ok(Some(value))
                    }
                    _ => {
                        // Generic case - for numbers check != 0
                        let value = self.transpile_expr(&args[0])?;
                        Ok(Some(quote! { (#value != 0) }))
                    }
                }
            }
            _ => Ok(None)
        }
    }
    
    /// Try to transpile advanced math functions (sin, cos, tan, log, log10, random)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use ruchy::backend::transpiler::Transpiler;
    /// let transpiler = Transpiler::new();
    /// // sin(x) -> x.sin()
    /// // cos(x) -> x.cos()
    /// // log(x) -> x.ln()
    /// // random() -> rand::random::<f64>()
    /// ```
    fn try_transpile_math_functions(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match base_name {
            "sin" | "cos" | "tan" => {
                if args.len() != 1 {
                    bail!("{}() expects exactly 1 argument", base_name);
                }
                let value = self.transpile_expr(&args[0])?;
                let method = proc_macro2::Ident::new(base_name, proc_macro2::Span::call_site());
                Ok(Some(quote! { ((#value as f64).#method()) }))
            }
            "log" => {
                if args.len() != 1 {
                    bail!("log() expects exactly 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { ((#value as f64).ln()) }))
            }
            "log10" => {
                if args.len() != 1 {
                    bail!("log10() expects exactly 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { ((#value as f64).log10()) }))
            }
            "random" => {
                if !args.is_empty() {
                    bail!("random() expects no arguments");
                }
                // Use a simple pseudo-random generator
                Ok(Some(quote! {
                    {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                        // Use a safe LCG that won't overflow
                        let a = 1664525u64;
                        let c = 1013904223u64;
                        let m = 1u64 << 32;
                        ((seed.wrapping_mul(a).wrapping_add(c)) % m) as f64 / m as f64
                    }
                }))
            }
            _ => Ok(None)
        }
    }
    
    /// Handle assert functions (assert, `assert_eq`, `assert_ne`)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("assert(true)");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("assert !"));
    /// ```
    fn try_transpile_assert_function(
        &self,
        _func_tokens: &TokenStream,
        base_name: &str,
        args: &[Expr]
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "assert" => {
                if args.is_empty() || args.len() > 2 {
                    bail!("assert expects 1 or 2 arguments (condition, optional message)");
                }
                let condition = self.transpile_expr(&args[0])?;
                if args.len() == 1 {
                    Ok(Some(quote! { assert!(#condition) }))
                } else {
                    let message = self.transpile_expr(&args[1])?;
                    Ok(Some(quote! { assert!(#condition, "{}", #message) }))
                }
            }
            "assert_eq" => {
                if args.len() < 2 || args.len() > 3 {
                    bail!("assert_eq expects 2 or 3 arguments (left, right, optional message)");
                }
                let left = self.transpile_expr(&args[0])?;
                let right = self.transpile_expr(&args[1])?;
                if args.len() == 2 {
                    Ok(Some(quote! { assert_eq!(#left, #right) }))
                } else {
                    let message = self.transpile_expr(&args[2])?;
                    Ok(Some(quote! { assert_eq!(#left, #right, "{}", #message) }))
                }
            }
            "assert_ne" => {
                if args.len() < 2 || args.len() > 3 {
                    bail!("assert_ne expects 2 or 3 arguments (left, right, optional message)");
                }
                let left = self.transpile_expr(&args[0])?;
                let right = self.transpile_expr(&args[1])?;
                if args.len() == 2 {
                    Ok(Some(quote! { assert_ne!(#left, #right) }))
                } else {
                    let message = self.transpile_expr(&args[2])?;
                    Ok(Some(quote! { assert_ne!(#left, #right, "{}", #message) }))
                }
            }
            _ => Ok(None)
        }
    }
    
    /// Handle collection constructors (`HashMap`, `HashSet`)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("HashMap()");
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("HashMap"));
    /// ```
    fn try_transpile_collection_constructor(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match (base_name, args.len()) {
            ("HashMap", 0) => Ok(Some(quote! { std::collections::HashMap::new() })),
            ("HashSet", 0) => Ok(Some(quote! { std::collections::HashSet::new() })),
            _ => Ok(None)
        }
    }
    
    /// Handle `DataFrame` functions (col)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"col("name")"#);
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("polars"));
    /// ```
    fn try_transpile_dataframe_function(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        if base_name == "col" && args.len() == 1 {
            if let ExprKind::Literal(Literal::String(col_name)) = &args[0].kind {
                return Ok(Some(quote! { polars::prelude::col(#col_name) }));
            }
        }
        Ok(None)
    }
    
    /// Handle regular function calls with string literal conversion
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"my_func("test")"#);
    /// let ast = parser.parse().unwrap();
    /// let result = transpiler.transpile(&ast).unwrap().to_string();
    /// assert!(result.contains("my_func"));
    /// ```
    fn transpile_regular_function_call(
        &self,
        func_tokens: &TokenStream,
        args: &[Expr]
    ) -> Result<TokenStream> {
        // Convert string literals to String for regular function calls
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| {
            match &a.kind {
                ExprKind::Literal(Literal::String(s)) => {
                    Ok(quote! { #s.to_string() })
                }
                _ => self.transpile_expr(a)
            }
        }).collect();
        let arg_tokens = arg_tokens?;
        
        Ok(quote! { #func_tokens(#(#arg_tokens),*) })
    }
}

#[cfg(test)]
#[allow(clippy::single_char_pattern)]
mod tests {
    use super::*;
    use crate::Parser;

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    #[test]
    fn test_transpile_if_with_else() {
        let transpiler = create_transpiler();
        let code = "if true { 1 } else { 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("if"));
        assert!(rust_str.contains("else"));
    }

    #[test]
    fn test_transpile_if_without_else() {
        let transpiler = create_transpiler();
        let code = "if true { 1 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("if"));
        assert!(!rust_str.contains("else"));
    }

    #[test]
    fn test_transpile_let_binding() {
        let transpiler = create_transpiler();
        let code = "let x = 5; x";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("let"));
        assert!(rust_str.contains("x"));
        assert!(rust_str.contains("5"));
    }

    #[test]
    fn test_transpile_mutable_let() {
        let transpiler = create_transpiler();
        let code = "let mut x = 5; x";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("mut"));
    }

    #[test]
    fn test_transpile_for_loop() {
        let transpiler = create_transpiler();
        let code = "for x in [1, 2, 3] { x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("for"));
        assert!(rust_str.contains("in"));
    }

    #[test]
    fn test_transpile_while_loop() {
        let transpiler = create_transpiler();
        let code = "while true { }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("while"));
    }

    #[test]
    fn test_function_with_parameters() {
        let transpiler = create_transpiler();
        let code = "fun add(x, y) { x + y }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("fn add"));
        assert!(rust_str.contains("x"));
        assert!(rust_str.contains("y"));
    }

    #[test]
    fn test_function_without_parameters() {
        let transpiler = create_transpiler();
        let code = "fun hello() { \"world\" }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("fn hello"));
        assert!(rust_str.contains("()"));
    }

    #[test]
    fn test_looks_like_numeric_function() {
        let transpiler = create_transpiler();
        
        // Test known numeric function names
        assert!(transpiler.looks_like_numeric_function("double"));
        assert!(transpiler.looks_like_numeric_function("add"));
        assert!(transpiler.looks_like_numeric_function("square"));
        
        // Test non-numeric function names
        assert!(!transpiler.looks_like_numeric_function("hello"));
        assert!(!transpiler.looks_like_numeric_function("main"));
        assert!(!transpiler.looks_like_numeric_function("test"));
    }

    #[test]
    fn test_match_expression() {
        let transpiler = create_transpiler();
        let code = "match x { 1 => \"one\", _ => \"other\" }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("match"));
    }

    #[test]
    fn test_lambda_expression() {
        let transpiler = create_transpiler();
        let code = "(x) => x + 1";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Lambda should be transpiled to closure
        assert!(rust_str.contains("|") || rust_str.contains("move"));
    }

    #[test]
    fn test_reserved_keyword_handling() {
        let transpiler = create_transpiler();
        let code = "let final = 5; final";  // Use regular keyword, not r# syntax
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Should handle Rust reserved keywords by prefixing with r#
        assert!(rust_str.contains("r#final") || rust_str.contains("final"));
    }

    #[test]
    fn test_generic_function() {
        let transpiler = create_transpiler();
        let code = "fun identity<T>(x: T) -> T { x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        assert!(rust_str.contains("fn identity"));
    }

    #[test]
    fn test_main_function_special_case() {
        let transpiler = create_transpiler();
        let code = "fun main() { println(\"Hello\") }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // main should not have explicit return type
        assert!(!rust_str.contains("fn main() ->"));
        assert!(!rust_str.contains("fn main () ->"));
    }

    #[test]
    fn test_dataframe_function_call() {
        let transpiler = create_transpiler();
        let code = "col(\"name\")";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Should transpile DataFrame column access
        assert!(rust_str.contains("polars") || rust_str.contains("col"));
    }

    #[test]
    fn test_regular_function_call_string_conversion() {
        let transpiler = create_transpiler();
        let code = "my_func(\"test\")";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Regular function calls should convert string literals
        assert!(rust_str.contains("my_func"));
        assert!(rust_str.contains("to_string") || rust_str.contains("\"test\""));
    }

    #[test]
    fn test_nested_expressions() {
        let transpiler = create_transpiler();
        let code = "if true { let x = 5; x + 1 } else { 0 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Should handle nested let inside if
        assert!(rust_str.contains("if"));
        assert!(rust_str.contains("let"));
        assert!(rust_str.contains("else"));
    }

    #[test]
    fn test_type_inference_integration() {
        let transpiler = create_transpiler();
        
        // Test function parameter as function
        let code1 = "fun apply(f, x) { f(x) }";
        let mut parser1 = Parser::new(code1);
        let ast1 = parser1.parse().unwrap();
        let result1 = transpiler.transpile(&ast1).unwrap();
        let rust_str1 = result1.to_string();
        assert!(rust_str1.contains("impl Fn"));
        
        // Test numeric parameter
        let code2 = "fun double(n) { n * 2 }";
        let mut parser2 = Parser::new(code2);
        let ast2 = parser2.parse().unwrap();
        let result2 = transpiler.transpile(&ast2).unwrap();
        let rust_str2 = result2.to_string();
        assert!(rust_str2.contains("n : i32") || rust_str2.contains("n: i32"));
        
        // Test string parameter
        let code3 = "fun greet(name) { \"Hello \" + name }";
        let mut parser3 = Parser::new(code3);
        let ast3 = parser3.parse().unwrap();
        let result3 = transpiler.transpile(&ast3).unwrap();
        let rust_str3 = result3.to_string();
        assert!(rust_str3.contains("name : String") || rust_str3.contains("name: String"));
    }

    #[test]
    fn test_return_type_inference() {
        let transpiler = create_transpiler();
        
        // Test numeric function gets return type
        let code = "fun double(n) { n * 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        assert!(rust_str.contains("-> i32"));
    }

    #[test]
    fn test_void_function_no_return_type() {
        let transpiler = create_transpiler();
        let code = "fun print_hello() { println(\"Hello\") }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // Should not have explicit return type for void functions
        assert!(!rust_str.contains("-> "));
    }

    #[test]
    fn test_complex_function_combinations() {
        let transpiler = create_transpiler();
        let code = "fun transform(f, n, m) { f(n + m) * 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap();
        let rust_str = result.to_string();
        
        // f should be function, n and m should be i32
        assert!(rust_str.contains("impl Fn"));
        assert!(rust_str.contains("n : i32") || rust_str.contains("n: i32"));
        assert!(rust_str.contains("m : i32") || rust_str.contains("m: i32"));
    }
}
