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
            "factorial" | "fibonacci" | "prime" | "even" | "odd" | "square" | "cube"
        )
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
                // For inferred types, use appropriate concrete types instead of generics
                let type_tokens = if let Ok(tokens) = self.transpile_type(&p.ty) {
                    let token_str = tokens.to_string();
                    if token_str == "_" {
                        // Smart type inference based on function name
                        if self.looks_like_numeric_function(name) {
                            quote! { i32 }
                        } else {
                            quote! { String }
                        }
                    } else {
                        tokens
                    }
                } else { 
                    // Smart default based on function name
                    if self.looks_like_numeric_function(name) {
                        quote! { i32 }
                    } else {
                        quote! { String }
                    }
                };
                quote! { #param_name: #type_tokens }
            })
            .collect();

        // Use provided type parameters only - no automatic generic inference
        let all_type_params = type_params.to_vec();

        let body_tokens = if is_async {
            let mut async_transpiler = Transpiler::new();
            async_transpiler.in_async_context = true;
            async_transpiler.transpile_expr(body)?
        } else {
            self.transpile_expr(body)?
        };

        // Infer return type based on function body content
        let return_type_tokens = if let Some(ty) = return_type {
            let ty_tokens = self.transpile_type(ty)?;
            quote! { -> #ty_tokens }
        } else if self.looks_like_numeric_function(name) {
            // Numeric functions likely return numeric values
            quote! { -> i32 }
        } else {
            // Don't automatically assume generic return type
            // Let Rust's type inference handle it
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

    /// Build printf-style format string for macros (preserves {} as format specifiers)
    fn build_printf_format_string(&self, parts: &[crate::frontend::ast::StringPart]) -> Result<TokenStream> {
        let mut format_string = String::new();
        
        for part in parts {
            match part {
                crate::frontend::ast::StringPart::Text(s) => {
                    // Don't escape {} in printf context - they are format specifiers
                    format_string.push_str(s);
                }
                crate::frontend::ast::StringPart::Expr(_) | 
                crate::frontend::ast::StringPart::ExprWithFormat { .. } => {
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
                // HashMap.items() -> HashMap.iter() for iterating key-value pairs
                Ok(quote! { #obj_tokens.iter() })
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
        match &args[0].kind {
            ExprKind::Literal(Literal::String(_)) => {
                // First argument is format string literal, remaining are format arguments  
                let format_arg = self.transpile_expr(&args[0])?;
                let format_args: Result<Vec<_>> = args[1..].iter().map(|a| self.transpile_expr(a)).collect();
                let format_args = format_args?;
                Ok(Some(quote! { #func_tokens!(#format_arg, #(#format_args),*) }))
            }
            ExprKind::StringInterpolation { parts } => {
                // Handle format string with printf-style formatting
                let format_str = self.build_printf_format_string(parts)?;
                let format_args: Result<Vec<_>> = args[1..].iter().map(|a| self.transpile_expr(a)).collect();
                let format_args = format_args?;
                Ok(Some(quote! { #func_tokens!(#format_str, #(#format_args),*) }))
            }
            _ => {
                // Generate format string with placeholders for all arguments
                let format_str = (0..args.len()).map(|_| "{}").collect::<Vec<_>>().join(" ");
                let all_args: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
                let all_args = all_args?;
                Ok(Some(quote! { #func_tokens!(#format_str, #(#all_args),*) }))
            }
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
            ("sqrt", 1) => {
                let arg = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (#arg as f64).sqrt() }))
            }
            ("pow", 2) => {
                let base = self.transpile_expr(&args[0])?;
                let exp = self.transpile_expr(&args[1])?;
                Ok(Some(quote! { (#base as f64).powf(#exp as f64) }))
            }
            ("abs", 1) => {
                let arg = self.transpile_expr(&args[0])?;
                // Check if arg is negative literal to handle type
                if let ExprKind::Unary { op: UnaryOp::Negate, operand } = &args[0].kind {
                    if matches!(&operand.kind, ExprKind::Literal(Literal::Float(_))) {
                        return Ok(Some(quote! { (#arg).abs() }));
                    }
                }
                // For all other cases, use standard abs
                Ok(Some(quote! { #arg.abs() }))
            }
            ("min", 2) => {
                let a = self.transpile_expr(&args[0])?;
                let b = self.transpile_expr(&args[1])?;
                // Check if args are float literals to determine type
                let is_float = matches!(&args[0].kind, ExprKind::Literal(Literal::Float(_))) 
                    || matches!(&args[1].kind, ExprKind::Literal(Literal::Float(_)));
                if is_float {
                    Ok(Some(quote! { (#a as f64).min(#b as f64) }))
                } else {
                    Ok(Some(quote! { std::cmp::min(#a, #b) }))
                }
            }
            ("max", 2) => {
                let a = self.transpile_expr(&args[0])?;
                let b = self.transpile_expr(&args[1])?;
                // Check if args are float literals to determine type
                let is_float = matches!(&args[0].kind, ExprKind::Literal(Literal::Float(_))) 
                    || matches!(&args[1].kind, ExprKind::Literal(Literal::Float(_)));
                if is_float {
                    Ok(Some(quote! { (#a as f64).max(#b as f64) }))
                } else {
                    Ok(Some(quote! { std::cmp::max(#a, #b) }))
                }
            }
            ("floor", 1) => {
                let arg = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (#arg as f64).floor() }))
            }
            ("ceil", 1) => {
                let arg = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (#arg as f64).ceil() }))
            }
            ("round", 1) => {
                let arg = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { (#arg as f64).round() }))
            }
            _ => Ok(None)
        }
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
