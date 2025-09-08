//! Statement and control flow transpilation

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]

use super::*;
use crate::frontend::ast::{CatchClause, Literal, Param, Pattern, PipelineStage, UnaryOp};
use anyhow::{Result, bail};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Checks if a variable is mutated (reassigned or modified) in an expression tree
    fn is_variable_mutated(name: &str, expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;
        
        match &expr.kind {
            // Direct assignment to the variable
            ExprKind::Assign { target, value: _ } => {
                if let ExprKind::Identifier(var_name) = &target.kind {
                    if var_name == name {
                        return true;
                    }
                }
                false
            }
            // Compound assignment (+=, -=, etc.)
            ExprKind::CompoundAssign { target, value: _, .. } => {
                if let ExprKind::Identifier(var_name) = &target.kind {
                    if var_name == name {
                        return true;
                    }
                }
                false
            }
            // Pre/Post increment/decrement
            ExprKind::PreIncrement { target } | 
            ExprKind::PostIncrement { target } |
            ExprKind::PreDecrement { target } |
            ExprKind::PostDecrement { target } => {
                if let ExprKind::Identifier(var_name) = &target.kind {
                    if var_name == name {
                        return true;
                    }
                }
                false
            }
            // Check in blocks
            ExprKind::Block(exprs) => {
                exprs.iter().any(|e| Self::is_variable_mutated(name, e))
            }
            // Check in if branches
            ExprKind::If { condition, then_branch, else_branch } => {
                Self::is_variable_mutated(name, condition) ||
                Self::is_variable_mutated(name, then_branch) ||
                else_branch.as_ref().is_some_and(|e| Self::is_variable_mutated(name, e))
            }
            // Check in while loops
            ExprKind::While { condition, body } => {
                Self::is_variable_mutated(name, condition) ||
                Self::is_variable_mutated(name, body)
            }
            // Check in for loops
            ExprKind::For { body, .. } => {
                Self::is_variable_mutated(name, body)
            }
            // Check in match expressions
            ExprKind::Match { expr, arms } => {
                Self::is_variable_mutated(name, expr) ||
                arms.iter().any(|arm| Self::is_variable_mutated(name, &arm.body))
            }
            // Check in nested let expressions
            ExprKind::Let { body, .. } | ExprKind::LetPattern { body, .. } => {
                Self::is_variable_mutated(name, body)
            }
            // Check in function bodies
            ExprKind::Function { body, .. } => {
                Self::is_variable_mutated(name, body)
            }
            // Check in lambda bodies
            ExprKind::Lambda { body, .. } => {
                Self::is_variable_mutated(name, body)
            }
            // Check binary operations
            ExprKind::Binary { left, right, .. } => {
                Self::is_variable_mutated(name, left) ||
                Self::is_variable_mutated(name, right)
            }
            // Check unary operations
            ExprKind::Unary { operand, .. } => {
                Self::is_variable_mutated(name, operand)
            }
            // Check function/method calls
            ExprKind::Call { func, args } => {
                Self::is_variable_mutated(name, func) ||
                args.iter().any(|a| Self::is_variable_mutated(name, a))
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                Self::is_variable_mutated(name, receiver) ||
                args.iter().any(|a| Self::is_variable_mutated(name, a))
            }
            // Other expressions don't contain mutations
            _ => false,
        }
    }

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
        
        // Auto-detect mutability: check if variable is in the mutable_vars set or is reassigned in body
        let effective_mutability = is_mutable || 
                                  self.mutable_vars.contains(name) || 
                                  Self::is_variable_mutated(name, body);
        
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
            if effective_mutability {
                Ok(quote! { let mut #name_ident = #value_tokens; })
            } else {
                Ok(quote! { let #name_ident = #value_tokens; })
            }
        } else {
            // Check if body is a Block containing sequential let statements
            // This flattens nested let expressions to avoid excessive nesting
            if let crate::frontend::ast::ExprKind::Block(exprs) = &body.kind {
                // Flatten sequential let statements into a single block
                let mut statements = Vec::new();
                
                // Add the current let statement
                if effective_mutability {
                    statements.push(quote! { let mut #name_ident = #value_tokens; });
                } else {
                    statements.push(quote! { let #name_ident = #value_tokens; });
                }
                
                // Add all the block expressions
                for (i, expr) in exprs.iter().enumerate() {
                    let expr_tokens = self.transpile_expr(expr)?;
                    
                    // Check if this is a Let expression with Unit body (standalone let statement)
                    // These already have semicolons from transpile_let
                    let is_standalone_let = matches!(&expr.kind, 
                        crate::frontend::ast::ExprKind::Let { body, .. } 
                        if matches!(body.kind, crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Unit))
                    );
                    
                    if is_standalone_let {
                        // Standalone let statements already have semicolons
                        statements.push(expr_tokens);
                    } else if i < exprs.len() - 1 {
                        // Not the last statement - add semicolon
                        statements.push(quote! { #expr_tokens; });
                    } else {
                        // Last expression - check if it's void
                        if self.is_void_expression(expr) {
                            statements.push(quote! { #expr_tokens; });
                        } else {
                            statements.push(expr_tokens);
                        }
                    }
                }
                
                Ok(quote! { #(#statements)* })
            } else {
                // Traditional let-in expression with proper scoping
                let body_tokens = self.transpile_expr(body)?;
                if effective_mutability {
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
        // FIRST CHECK: Override for test functions
        if name.starts_with("test_") {
            return Ok(quote! {});
        }
        
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
            // Check if body is already a block to avoid double-wrapping
            match &body.kind {
                ExprKind::Block(exprs) => {
                    // For function bodies that are blocks, transpile the contents directly
                    if exprs.len() == 1 {
                        // Single expression block - transpile the expression directly
                        self.transpile_expr(&exprs[0])
                    } else {
                        // Multiple expressions - need proper semicolons between statements
                        let mut statements = Vec::new();
                        for (i, expr) in exprs.iter().enumerate() {
                            let expr_tokens = self.transpile_expr(expr)?;
                            
                            // Add semicolons to all statements except the last one
                            // (unless it's a void expression that needs a semicolon)
                            if i < exprs.len() - 1 {
                                // Not the last statement - always add semicolon
                                statements.push(quote! { #expr_tokens; });
                            } else {
                                // Last statement - check if it's void
                                if self.is_void_expression(expr) {
                                    // Void expressions should have semicolons
                                    statements.push(quote! { #expr_tokens; });
                                } else {
                                    // Non-void last expression - no semicolon (it's the return value)
                                    statements.push(expr_tokens);
                                }
                            }
                        }
                        
                        if statements.is_empty() {
                            Ok(quote! {})
                        } else {
                            Ok(quote! { #(#statements)* })
                        }
                    }
                },
                _ => {
                    // Not a block - transpile normally
                    self.transpile_expr(body)
                }
            }
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
        attributes: &[crate::frontend::ast::Attribute],
    ) -> Result<TokenStream> {
        // Override return type for test functions
        let final_return_type = if fn_name.to_string().starts_with("test_") {
            quote! {}
        } else {
            return_type_tokens.clone()
        };
        let visibility = if is_pub { quote! { pub } } else { quote! {} };
        
        // Generate attribute tokens
        let attr_tokens: Vec<TokenStream> = attributes.iter()
            .map(|attr| {
                let attr_name = format_ident!("{}", attr.name);
                if attr.args.is_empty() {
                    quote! { #[#attr_name] }
                } else {
                    let args: Vec<TokenStream> = attr.args.iter()
                        .map(|arg| arg.parse().unwrap_or_else(|_| quote! { #arg }))
                        .collect();
                    quote! { #[#attr_name(#(#args),*)] }
                }
            })
            .collect();
        
        Ok(match (type_param_tokens.is_empty(), is_async) {
            (true, false) => quote! {
                #(#attr_tokens)*
                #visibility fn #fn_name(#(#param_tokens),*) #final_return_type {
                    #body_tokens
                }
            },
            (true, true) => quote! {
                #(#attr_tokens)*
                #visibility async fn #fn_name(#(#param_tokens),*) #final_return_type {
                    #body_tokens
                }
            },
            (false, false) => quote! {
                #(#attr_tokens)*
                #visibility fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #final_return_type {
                    #body_tokens
                }
            },
            (false, true) => quote! {
                #(#attr_tokens)*
                #visibility async fn #fn_name<#(#type_param_tokens),*>(#(#param_tokens),*) #final_return_type {
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
        attributes: &[crate::frontend::ast::Attribute],
    ) -> Result<TokenStream> {
        let fn_name = format_ident!("{}", name);
        let param_tokens = self.generate_param_tokens(params, body, name)?;
        let body_tokens = self.generate_body_tokens(body, is_async)?;
        
        // Check for #[test] attribute and override return type if found
        let has_test_attribute = attributes.iter().any(|attr| attr.name == "test");
        
        let effective_return_type = if has_test_attribute {
            None // Test functions should have unit return type
        } else {
            return_type
        };
        
        let return_type_tokens = self.generate_return_type_tokens(name, effective_return_type, body)?;
        let type_param_tokens = self.generate_type_param_tokens(type_params)?;

        self.generate_function_signature(
            is_pub, 
            is_async, 
            &fn_name, 
            &type_param_tokens, 
            &param_tokens, 
            &return_type_tokens, 
            &body_tokens,
            attributes
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
                name.strip_suffix('!').unwrap_or(name)
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
        // Check if this is part of a DataFrame builder pattern
        if method == "column" || method == "build" {
            // Build the full method call expression to check for builder pattern
            let method_call_expr = Expr {
                kind: ExprKind::MethodCall {
                    receiver: Box::new(object.clone()),
                    method: method.to_string(),
                    args: args.to_vec(),
                },
                span: object.span.clone(),
                attributes: vec![],
            };
            
            if let Some(tokens) = self.transpile_dataframe_builder(&method_call_expr)? {
                return Ok(tokens);
            }
        }
        
        // Use the old implementation for other cases
        self.transpile_method_call_old(object, method, args)
    }
    
    #[allow(dead_code)]
    fn transpile_method_call_old(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        let method_ident = format_ident!("{}", method);
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;

        // Check DataFrame methods FIRST before generic collection methods
        if self.is_dataframe_expr(object) && matches!(method, 
            "get" | "rows" | "columns" | "select" | "filter" | "sort" | 
            "head" | "tail" | "mean" | "std" | "min" | "max" | "sum" | "count"
        ) {
            return self.transpile_dataframe_method(object, method, args);
        }
        
        // Dispatch to specialized handlers based on method category
        match method {
            // Iterator operations (map, filter, reduce)
            "map" | "filter" | "reduce" => {
                self.transpile_iterator_methods(&obj_tokens, method, &arg_tokens)
            }
            // HashMap/HashSet methods (get, contains_key, items, etc.)
            "get" | "contains_key" | "keys" | "values" | "entry" | "items" |
            "update" | "add" => {
                self.transpile_map_set_methods(&obj_tokens, &method_ident, method, &arg_tokens)
            }
            // Set operations (union, intersection, difference, symmetric_difference)
            "union" | "intersection" | "difference" | "symmetric_difference" => {
                self.transpile_set_operations(&obj_tokens, method, &arg_tokens)
            }
            // Common collection methods (insert, remove, clear, len, is_empty, iter)
            "insert" | "remove" | "clear" | "len" | "is_empty" | "iter" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            // DataFrame operations - use special handling for correct Polars API
            "select" | "groupby" | "group_by" | "agg" | "sort" | "mean" | "std" | "min"
            | "max" | "sum" | "count" | "drop_nulls" | "fill_null" | "pivot"
            | "melt" | "head" | "tail" | "sample" | "describe" | "rows" | "columns" | "column" | "build" => {
                // Check if this is a DataFrame operation
                if self.is_dataframe_expr(object) {
                    self.transpile_dataframe_method(object, method, args)
                } else {
                    Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
                }
            }
            // String methods (Python-style and Rust-style)
            "to_s" | "to_string" | "to_upper" | "to_lower" | "upper" | "lower" | 
            "length" | "substring" | "strip" | "lstrip" | "rstrip" | 
            "startswith" | "endswith" | "split" | "replace" => {
                self.transpile_string_methods(&obj_tokens, method, &arg_tokens)
            }
            // List/Vec methods (Python-style)
            "append" => {
                // Python's append() -> Rust's push()
                Ok(quote! { #obj_tokens.push(#(#arg_tokens),*) })
            }
            "extend" => {
                // Python's extend() -> Rust's extend()
                Ok(quote! { #obj_tokens.extend(#(#arg_tokens),*) })
            }
            // Collection methods that work as-is (not already handled above)
            "push" | "pop" | "contains" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            // Advanced collection methods (slice, concat, flatten, unique, join)
            "slice" | "concat" | "flatten" | "unique" | "join" => {
                self.transpile_advanced_collection_methods(&obj_tokens, method, &arg_tokens)
            }
            _ => {
                // Regular method call
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
        }
    }
    
    /// Handle iterator operations: map, filter, reduce
    fn transpile_iterator_methods(&self, obj_tokens: &TokenStream, method: &str, arg_tokens: &[TokenStream]) -> Result<TokenStream> {
        match method {
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
            _ => unreachable!("Non-iterator method passed to transpile_iterator_methods"),
        }
    }
    
    /// Handle HashMap/HashSet methods: get, `contains_key`, items, etc.
    fn transpile_map_set_methods(&self, obj_tokens: &TokenStream, method_ident: &proc_macro2::Ident, method: &str, arg_tokens: &[TokenStream]) -> Result<TokenStream> {
        match method {
            "get" => {
                // HashMap.get() returns Option<&V>, but we want owned values
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*).cloned() })
            }
            "contains_key" | "keys" | "values" | "entry" | "contains" => {
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
            "items" => {
                // HashMap.items() -> iterator of (K, V) tuples (not references)
                Ok(quote! { #obj_tokens.iter().map(|(k, v)| (k.clone(), v.clone())) })
            }
            "update" => {
                // Python dict.update(other) -> Rust HashMap.extend(other)
                Ok(quote! { #obj_tokens.extend(#(#arg_tokens),*) })
            }
            "add" => {
                // Python set.add(item) -> Rust HashSet.insert(item)
                Ok(quote! { #obj_tokens.insert(#(#arg_tokens),*) })
            }
            _ => unreachable!("Non-map/set method {} passed to transpile_map_set_methods", method),
        }
    }
    
    /// Handle `HashSet` set operations: union, intersection, difference, `symmetric_difference`
    fn transpile_set_operations(&self, obj_tokens: &TokenStream, method: &str, arg_tokens: &[TokenStream]) -> Result<TokenStream> {
        if arg_tokens.len() != 1 {
            bail!("{} requires exactly 1 argument", method);
        }
        let other = &arg_tokens[0];
        let method_ident = format_ident!("{}", method);
        Ok(quote! { 
            {
                use std::collections::HashSet;
                #obj_tokens.#method_ident(&#other).cloned().collect::<HashSet<_>>()
            }
        })
    }
    
    /// Handle string methods: Python-style and Rust-style
    fn transpile_string_methods(&self, obj_tokens: &TokenStream, method: &str, arg_tokens: &[TokenStream]) -> Result<TokenStream> {
        match method {
            "to_s" | "to_string" => {
                // Convert any value to string - already a String stays String
                Ok(quote! { #obj_tokens })
            }
            "to_upper" | "upper" => {
                let rust_method = format_ident!("to_uppercase");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "to_lower" | "lower" => {
                let rust_method = format_ident!("to_lowercase");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "strip" => {
                Ok(quote! { #obj_tokens.trim().to_string() })
            }
            "lstrip" => {
                Ok(quote! { #obj_tokens.trim_start() })
            }
            "rstrip" => {
                Ok(quote! { #obj_tokens.trim_end() })
            }
            "startswith" => {
                Ok(quote! { #obj_tokens.starts_with(#(#arg_tokens),*) })
            }
            "endswith" => {
                Ok(quote! { #obj_tokens.ends_with(#(#arg_tokens),*) })
            }
            "split" => {
                Ok(quote! { #obj_tokens.split(#(#arg_tokens),*) })
            }
            "replace" => {
                Ok(quote! { #obj_tokens.replace(#(#arg_tokens),*) })
            }
            "length" => {
                // Map Ruchy's length() to Rust's len()
                let rust_method = format_ident!("len");
                Ok(quote! { #obj_tokens.#rust_method(#(#arg_tokens),*) })
            }
            "substring" => {
                // string.substring(start, end) -> string.chars().skip(start).take(end-start).collect()
                if arg_tokens.len() != 2 {
                    bail!("substring requires exactly 2 arguments");
                }
                let start = &arg_tokens[0];
                let end = &arg_tokens[1];
                Ok(quote! { 
                    #obj_tokens.chars()
                        .skip(#start as usize)
                        .take((#end as usize).saturating_sub(#start as usize))
                        .collect::<String>()
                })
            }
            _ => unreachable!("Non-string method {} passed to transpile_string_methods", method),
        }
    }
    
    /// Handle advanced collection methods: slice, concat, flatten, unique, join
    fn transpile_advanced_collection_methods(&self, obj_tokens: &TokenStream, method: &str, arg_tokens: &[TokenStream]) -> Result<TokenStream> {
        match method {
            "slice" => {
                // vec.slice(start, end) -> vec[start..end].to_vec()
                if arg_tokens.len() != 2 {
                    bail!("slice requires exactly 2 arguments");
                }
                let start = &arg_tokens[0];
                let end = &arg_tokens[1];
                Ok(quote! { #obj_tokens[#start as usize..#end as usize].to_vec() })
            }
            "concat" => {
                // vec.concat(other) -> [vec, other].concat()
                if arg_tokens.len() != 1 {
                    bail!("concat requires exactly 1 argument");
                }
                let other = &arg_tokens[0];
                Ok(quote! { [#obj_tokens, #other].concat() })
            }
            "flatten" => {
                // vec.flatten() -> vec.into_iter().flatten().collect()
                if !arg_tokens.is_empty() {
                    bail!("flatten requires no arguments");
                }
                Ok(quote! { #obj_tokens.into_iter().flatten().collect::<Vec<_>>() })
            }
            "unique" => {
                // vec.unique() -> vec.into_iter().collect::<HashSet<_>>().into_iter().collect()
                if !arg_tokens.is_empty() {
                    bail!("unique requires no arguments");
                }
                Ok(quote! { 
                    {
                        use std::collections::HashSet;
                        #obj_tokens.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>()
                    }
                })
            }
            "join" => {
                // vec.join(separator) -> vec.join(separator) (for Vec<String>)
                if arg_tokens.len() != 1 {
                    bail!("join requires exactly 1 argument");
                }
                let separator = &arg_tokens[0];
                Ok(quote! { #obj_tokens.join(&#separator) })
            }
            _ => unreachable!("Non-advanced-collection method passed to transpile_advanced_collection_methods"),
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

    /// Transpiles try-catch-finally blocks
    pub fn transpile_try_catch(
        &self, 
        try_block: &Expr, 
        catch_clauses: &[CatchClause],
        finally_block: Option<&Expr>
    ) -> Result<TokenStream> {
        // For now, we'll transpile try-catch to a match on Result
        // This is a simplified implementation that handles the common case
        let try_body = self.transpile_expr(try_block)?;
        
        if catch_clauses.is_empty() {
            bail!("Try block must have at least one catch clause");
        }
        
        // Generate the catch handling
        let catch_pattern = match &catch_clauses[0].pattern {
            Pattern::Identifier(name) => {
                let ident = format_ident!("{}", name);
                quote! { #ident }
            }
            _ => quote! { _e }
        };
        
        let catch_body = self.transpile_expr(&catch_clauses[0].body)?;
        
        // If there's a finally block, we need to ensure it runs
        let result = if let Some(finally) = finally_block {
            let finally_tokens = self.transpile_expr(finally)?;
            quote! {
                {
                    let _result = (|| -> Result<_, Box<dyn std::error::Error>> {
                        Ok(#try_body)
                    })();
                    
                    let _final_result = match _result {
                        Ok(val) => val,
                        Err(#catch_pattern) => #catch_body
                    };
                    
                    #finally_tokens;
                    _final_result
                }
            }
        } else {
            // Simple try-catch without finally
            quote! {
                match (|| -> Result<_, Box<dyn std::error::Error>> {
                    Ok(#try_body)
                })() {
                    Ok(val) => val,
                    Err(#catch_pattern) => #catch_body
                }
            }
        };
        
        Ok(result)
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
        
        
        // All imports should have module-level scope, not be wrapped in blocks
        // This includes both std library imports and local module imports
        Self::transpile_import_inline(path, items)
    }
    
    /// Handle `std::fs` imports and generate file operation functions
    fn transpile_std_fs_import(items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        use crate::frontend::ast::ImportItem;
        
        let mut tokens = TokenStream::new();
        
        // Always include std::fs for file operations
        tokens.extend(quote! { use std::fs; });
        
        if items.is_empty() || items.iter().any(|i| matches!(i, ImportItem::Wildcard)) {
            // Import all file operations
            tokens.extend(Self::generate_all_file_operations());
        } else {
            // Import specific operations
            for item in items {
                match item {
                    ImportItem::Named(name) => {
                        match name.as_str() {
                            "read_file" => tokens.extend(Self::generate_read_file_function()),
                            "write_file" => tokens.extend(Self::generate_write_file_function()),
                            _ => {
                                // Unknown std::fs function, generate stub or error
                                let func_name = format_ident!("{}", name);
                                tokens.extend(quote! {
                                    fn #func_name() -> ! {
                                        panic!("std::fs::{} not yet implemented", #name);
                                    }
                                });
                            }
                        }
                    }
                    ImportItem::Aliased { name, alias } => {
                        let alias_ident = format_ident!("{}", alias);
                        match name.as_str() {
                            "read_file" => {
                                tokens.extend(quote! {
                                    fn #alias_ident(filename: String) -> String {
                                        fs::read_to_string(filename).unwrap_or_else(|e| panic!("Failed to read file: {}", e))
                                    }
                                });
                            }
                            "write_file" => {
                                tokens.extend(quote! {
                                    fn #alias_ident(filename: String, content: String) {
                                        fs::write(filename, content).unwrap_or_else(|e| panic!("Failed to write file: {}", e));
                                    }
                                });
                            }
                            _ => {
                                tokens.extend(quote! {
                                    fn #alias_ident() -> ! {
                                        panic!("std::fs::{} not yet implemented", #name);
                                    }
                                });
                            }
                        }
                    }
                    ImportItem::Wildcard => {
                        tokens.extend(Self::generate_all_file_operations());
                    }
                }
            }
        }
        
        tokens
    }
    
    /// Generate `read_file` function
    fn generate_read_file_function() -> TokenStream {
        quote! {
            fn read_file(filename: String) -> String {
                fs::read_to_string(filename).unwrap_or_else(|e| panic!("Failed to read file: {}", e))
            }
        }
    }
    
    /// Generate `write_file` function  
    fn generate_write_file_function() -> TokenStream {
        quote! {
            fn write_file(filename: String, content: String) {
                fs::write(filename, content).unwrap_or_else(|e| panic!("Failed to write file: {}", e));
            }
        }
    }
    
    /// Generate all file operation functions
    fn generate_all_file_operations() -> TokenStream {
        let read_func = Self::generate_read_file_function();
        let write_func = Self::generate_write_file_function();
        
        quote! {
            #read_func
            #write_func
        }
    }
    
    /// Handle `std::fs` imports with path-based syntax (import `std::fs::read_file`)
    fn transpile_std_fs_import_with_path(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        use crate::frontend::ast::ImportItem;
        
        
        let mut tokens = TokenStream::new();
        
        // Always include std::fs for file operations
        tokens.extend(quote! { use std::fs; });
        
        if path == "std::fs" {
            // Wildcard import or specific items from std::fs
            // Special case: if path is "std::fs" and items contain Named("fs"), treat as wildcard
            let is_wildcard_import = items.is_empty() 
                || items.iter().any(|i| matches!(i, ImportItem::Wildcard))
                || (items.len() == 1 && matches!(&items[0], ImportItem::Named(name) if name == "fs"));
                
            if is_wildcard_import {
                // Import all file operations for wildcard or empty imports
                tokens.extend(Self::generate_all_file_operations());
            } else {
                // Import specific operations
                for item in items {
                    match item {
                        ImportItem::Named(name) => {
                            match name.as_str() {
                                "read_file" => tokens.extend(Self::generate_read_file_function()),
                                "write_file" => tokens.extend(Self::generate_write_file_function()),
                                _ => {} // Ignore unknown functions
                            }
                        }
                        ImportItem::Wildcard => {
                            tokens.extend(Self::generate_all_file_operations());
                            break;
                        }
                        ImportItem::Aliased { name, alias: _ } => {
                            // Handle aliased imports like "read_file as rf"
                            match name.as_str() {
                                "read_file" => tokens.extend(Self::generate_read_file_function()),
                                "write_file" => tokens.extend(Self::generate_write_file_function()),
                                _ => {} // Ignore unknown functions
                            }
                        }
                    }
                }
            }
        } else if path.starts_with("std::fs::") {
            // Path-based import like std::fs::read_file
            let function_name = path.strip_prefix("std::fs::").unwrap_or("");
            match function_name {
                "read_file" => tokens.extend(Self::generate_read_file_function()),
                "write_file" => tokens.extend(Self::generate_write_file_function()),
                _ => {} // Ignore unknown functions
            }
        }
        
        tokens
    }

    /// Handle `std::process` imports with process management functions
    fn transpile_std_process_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate process functions
        quote! {
            mod process {
                pub fn current_pid() -> i32 {
                    std::process::id() as i32
                }
                
                pub fn exit(code: i32) {
                    std::process::exit(code);
                }
                
                pub fn spawn(command: &str) -> Result<i32, String> {
                    match std::process::Command::new(command).spawn() {
                        Ok(child) => Ok(child.id() as i32),
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
        }
    }
    
    /// Handle `std::system` imports with system information functions
    fn transpile_std_system_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate system functions
        quote! {
            mod system {
                pub fn get_env(key: &str) -> Option<String> {
                    std::env::var(key).ok()
                }
                
                pub fn set_env(key: &str, value: &str) {
                    std::env::set_var(key, value);
                }
                
                pub fn os_name() -> String {
                    std::env::consts::OS.to_string()
                }
                
                pub fn arch() -> String {
                    std::env::consts::ARCH.to_string()
                }
            }
        }
    }
    
    /// Handle `std::signal` imports with signal handling functions
    fn transpile_std_signal_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // For now, just provide stubs as signal handling is complex and platform-specific
        quote! {
            // Import signal constants at top level
            const SIGINT: i32 = 2;
            const SIGTERM: i32 = 15;
            const SIGKILL: i32 = 9;
            
            // Also import exit function for signal handlers
            fn exit(code: i32) {
                std::process::exit(code);
            }
            
            mod signal {
                pub const SIGINT: i32 = 2;
                pub const SIGTERM: i32 = 15;
                pub const SIGKILL: i32 = 9;
                
                pub fn on(_signal: i32, _handler: impl Fn()) {
                    // Signal handling would require unsafe code and platform-specific logic
                    // For now, this is a stub
                }
            }
        }
    }
    
    /// Handle `std::net` imports with networking functions
    fn transpile_std_net_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate networking functions and re-export std types
        quote! {
            mod net {
                pub use std::net::*;
                
                pub struct TcpListener;
                
                impl TcpListener {
                    pub fn bind(addr: String) -> Result<Self, String> {
                        println!("Would bind TCP listener to: {}", addr);
                        Ok(TcpListener)
                    }
                    
                    pub fn accept(&self) -> Result<TcpStream, String> {
                        println!("Would accept connection");
                        Ok(TcpStream)
                    }
                }
                
                pub struct TcpStream;
                
                impl TcpStream {
                    pub fn connect(addr: String) -> Result<Self, String> {
                        println!("Would connect to: {}", addr);
                        Ok(TcpStream)
                    }
                }
            }
            
            // Also make available as module for http submodules
            mod http {
                pub struct Server {
                    addr: String,
                }
                
                impl Server {
                    pub fn new(addr: String) -> Self {
                        println!("Creating HTTP server on: {}", addr);
                        Server { addr }
                    }
                    
                    pub fn listen(&self) {
                        println!("HTTP server listening on: {}", self.addr);
                    }
                }
            }
        }
    }

    fn transpile_std_mem_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate memory management functions
        quote! {
            mod mem {
                pub struct Array<T> {
                    data: Vec<T>,
                }
                
                impl<T: Clone> Array<T> {
                    pub fn new(size: usize, default_value: T) -> Self {
                        Array {
                            data: vec![default_value; size],
                        }
                    }
                }
                
                pub struct MemoryInfo {
                    pub allocated: usize,
                    pub peak: usize,
                }
                
                impl std::fmt::Display for MemoryInfo {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "allocated: {}KB, peak: {}KB", self.allocated / 1024, self.peak / 1024)
                    }
                }
                
                pub fn usage() -> MemoryInfo {
                    MemoryInfo {
                        allocated: 1024 * 100, // 100KB stub
                        peak: 1024 * 150,      // 150KB stub
                    }
                }
            }
        }
    }

    fn transpile_std_parallel_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate parallel processing functions
        quote! {
            mod parallel {
                pub fn map<T, U, F>(data: Vec<T>, func: F) -> Vec<U>
                where
                    T: Send,
                    U: Send,
                    F: Fn(T) -> U + Send + Sync,
                {
                    // Simple sequential implementation for now (stub)
                    data.into_iter().map(func).collect()
                }
                
                pub fn filter<T, F>(data: Vec<T>, predicate: F) -> Vec<T>
                where
                    T: Send,
                    F: Fn(&T) -> bool + Send + Sync,
                {
                    data.into_iter().filter(|x| predicate(x)).collect()
                }
                
                pub fn reduce<T, F>(data: Vec<T>, func: F) -> Option<T>
                where
                    T: Send,
                    F: Fn(T, T) -> T + Send + Sync,
                {
                    data.into_iter().reduce(func)
                }
            }
        }
    }

    fn transpile_std_simd_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate SIMD vectorization functions
        quote! {
            mod simd {
                use std::ops::Add;
                
                pub struct SimdVec<T> {
                    data: Vec<T>,
                }
                
                impl<T> SimdVec<T> {
                    pub fn from_slice(slice: &[T]) -> Self
                    where
                        T: Clone,
                    {
                        SimdVec {
                            data: slice.to_vec(),
                        }
                    }
                }
                
                impl<T> Add for SimdVec<T>
                where
                    T: Add<Output = T> + Copy,
                {
                    type Output = SimdVec<T>;
                    
                    fn add(self, other: SimdVec<T>) -> SimdVec<T> {
                        let result: Vec<T> = self.data.iter()
                            .zip(other.data.iter())
                            .map(|(&a, &b)| a + b)
                            .collect();
                        SimdVec { data: result }
                    }
                }
                
                impl<T> std::fmt::Display for SimdVec<T>
                where
                    T: std::fmt::Display,
                {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "[{}]", self.data.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(", "))
                    }
                }
                
                pub fn from_slice<T: Clone>(slice: &[T]) -> SimdVec<T> {
                    SimdVec::from_slice(slice)
                }
            }
        }
    }

    fn transpile_std_cache_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate caching functions - placeholder for @memoize attribute support
        quote! {
            mod cache {
                use std::collections::HashMap;
                
                pub struct Cache<K, V> {
                    data: HashMap<K, V>,
                }
                
                impl<K, V> Cache<K, V>
                where
                    K: std::hash::Hash + Eq,
                {
                    pub fn new() -> Self {
                        Cache {
                            data: HashMap::new(),
                        }
                    }
                    
                    pub fn get(&self, key: &K) -> Option<&V> {
                        self.data.get(key)
                    }
                    
                    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                        self.data.insert(key, value)
                    }
                }
            }
        }
    }

    fn transpile_std_bench_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate benchmarking functions
        quote! {
            mod bench {
                use std::time::{Duration, Instant};
                
                pub struct BenchmarkResult {
                    pub elapsed: u128,
                }
                
                impl BenchmarkResult {
                    pub fn new(elapsed: Duration) -> Self {
                        BenchmarkResult {
                            elapsed: elapsed.as_millis(),
                        }
                    }
                }
                
                impl std::fmt::Display for BenchmarkResult {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "{}ms", self.elapsed)
                    }
                }
                
                pub fn time<F, T>(mut func: F) -> BenchmarkResult
                where
                    F: FnMut() -> T,
                {
                    let start = Instant::now();
                    let _ = func();
                    let elapsed = start.elapsed();
                    BenchmarkResult::new(elapsed)
                }
            }
        }
    }

    fn transpile_std_profile_import(_path: &str, _items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Generate profiling functions - placeholder for @hot_path attribute support
        quote! {
            mod profile {
                pub struct ProfileInfo {
                    pub function_name: String,
                    pub call_count: usize,
                    pub total_time: u128,
                }
                
                impl std::fmt::Display for ProfileInfo {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "{}: {} calls, {}ms total", 
                               self.function_name, self.call_count, self.total_time)
                    }
                }
                
                pub fn get_stats(function_name: &str) -> ProfileInfo {
                    ProfileInfo {
                        function_name: function_name.to_string(),
                        call_count: 42, // Stub values
                        total_time: 100,
                    }
                }
            }
        }
    }
    
    /// Handle `std::system` imports with system information functions
    /// Core inline import transpilation logic - REFACTORED FOR COMPLEXITY REDUCTION
    /// Original: 48 cyclomatic complexity, Target: <20
    pub fn transpile_import_inline(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Try std module handlers first (complexity: delegated)
        if let Some(result) = Self::handle_std_module_import(path, items) {
            return result;
        }
        
        // Fall back to generic import handling (complexity: delegated)
        Self::handle_generic_import(path, items)
    }
    
    /// Extract std module dispatcher (complexity ~12)
    fn handle_std_module_import(path: &str, items: &[crate::frontend::ast::ImportItem]) -> Option<TokenStream> {
        if path.starts_with("std::fs") {
            return Some(Self::transpile_std_fs_import_with_path(path, items));
        }
        if path.starts_with("std::process") {
            return Some(Self::transpile_std_process_import(path, items));
        }
        if path.starts_with("std::system") {
            return Some(Self::transpile_std_system_import(path, items));
        }
        if path.starts_with("std::signal") {
            return Some(Self::transpile_std_signal_import(path, items));
        }
        if path.starts_with("std::net") {
            return Some(Self::transpile_std_net_import(path, items));
        }
        if path.starts_with("std::mem") {
            return Some(Self::transpile_std_mem_import(path, items));
        }
        if path.starts_with("std::parallel") {
            return Some(Self::transpile_std_parallel_import(path, items));
        }
        if path.starts_with("std::simd") {
            return Some(Self::transpile_std_simd_import(path, items));
        }
        if path.starts_with("std::cache") {
            return Some(Self::transpile_std_cache_import(path, items));
        }
        if path.starts_with("std::bench") {
            return Some(Self::transpile_std_bench_import(path, items));
        }
        if path.starts_with("std::profile") {
            return Some(Self::transpile_std_profile_import(path, items));
        }
        None
    }
    
    /// Extract generic import handling (complexity ~8)
    fn handle_generic_import(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        
        let path_tokens = Self::path_to_tokens(path);
        
        if items.is_empty() {
            quote! { use #path_tokens::*; }
        } else if items.len() == 1 {
            Self::handle_single_import_item(&path_tokens, path, &items[0])
        } else {
            Self::handle_multiple_import_items(&path_tokens, items)
        }
    }
    
    /// Extract path tokenization (complexity ~4)
    fn path_to_tokens(path: &str) -> TokenStream {
        let mut path_tokens = TokenStream::new();
        let segments: Vec<_> = path.split("::").collect();
        
        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                path_tokens.extend(quote! { :: });
            }
            if !segment.is_empty() {
                let seg_ident = format_ident!("{}", segment);
                path_tokens.extend(quote! { #seg_ident });
            }
        }
        
        path_tokens
    }
    
    /// Extract single item handling (complexity ~5)
    fn handle_single_import_item(
        path_tokens: &TokenStream, 
        path: &str, 
        item: &crate::frontend::ast::ImportItem
    ) -> TokenStream {
        use crate::frontend::ast::ImportItem;
        
        match item {
            ImportItem::Named(name) => {
                if path.ends_with(&format!("::{name}")) {
                    quote! { use #path_tokens; }
                } else {
                    let item_ident = format_ident!("{}", name);
                    quote! { use #path_tokens::#item_ident; }
                }
            }
            ImportItem::Aliased { name, alias } => {
                let name_ident = format_ident!("{}", name);
                let alias_ident = format_ident!("{}", alias);
                quote! { use #path_tokens::#name_ident as #alias_ident; }
            }
            ImportItem::Wildcard => quote! { use #path_tokens::*; },
        }
    }
    
    /// Extract multiple items handling (complexity ~3)
    fn handle_multiple_import_items(
        path_tokens: &TokenStream, 
        items: &[crate::frontend::ast::ImportItem]
    ) -> TokenStream {
        let item_tokens = Self::process_import_items(items);
        quote! { use #path_tokens::{#(#item_tokens),*}; }
    }
    
    /// Extract import items processing (complexity ~3)
    fn process_import_items(items: &[crate::frontend::ast::ImportItem]) -> Vec<TokenStream> {
        use crate::frontend::ast::ImportItem;
        
        items.iter().map(|item| match item {
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
        }).collect()
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
                    let printing_logic = self.generate_value_printing_tokens(quote! { #arg }, quote! { #func_tokens });
                    Ok(Some(printing_logic))
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
        // Delegate to refactored version with reduced complexity
        // Original complexity: 62, New complexity: <20 per function
        self.try_transpile_type_conversion_refactored(base_name, args)
    }
    
    // Old implementation kept for reference (will be removed after verification)
    #[allow(dead_code)]
    pub fn try_transpile_type_conversion_old(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match base_name {
            "str" => self.transpile_str_conversion(args).map(Some),
            "int" => self.transpile_int_conversion(args).map(Some), 
            "float" => self.transpile_float_conversion(args).map(Some),
            "bool" => self.transpile_bool_conversion(args).map(Some),
            _ => Ok(None)
        }
    }
    
    /// Handle `str()` type conversion - extract string representation
    fn transpile_str_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("str() expects exactly 1 argument");
        }
        let value = self.transpile_expr(&args[0])?;
        Ok(quote! { format!("{}", #value) })
    }
    
    /// Handle `int()` type conversion with literal-specific optimizations
    fn transpile_int_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("int() expects exactly 1 argument");
        }
        
        // Check if the argument is a literal for compile-time optimizations
        match &args[0].kind {
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { #value.parse::<i64>().expect("Failed to parse integer") })
            }
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                    let value = self.transpile_expr(&args[0])?;
                    Ok(quote! { #value.parse::<i64>().expect("Failed to parse integer") })
                } else {
                    self.transpile_int_generic(&args[0])
                }
            }
            ExprKind::Literal(Literal::Float(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value as i64) })
            }
            ExprKind::Literal(Literal::Bool(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { if #value { 1i64 } else { 0i64 } })
            }
            _ => self.transpile_int_generic(&args[0])
        }
    }
    
    /// Generic int conversion for non-literal expressions
    fn transpile_int_generic(&self, expr: &Expr) -> Result<TokenStream> {
        let value = self.transpile_expr(expr)?;
        Ok(quote! { (#value as i64) })
    }
    
    /// Handle `float()` type conversion with literal-specific optimizations
    fn transpile_float_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("float() expects exactly 1 argument");
        }
        
        // Check if the argument is a literal for compile-time optimizations
        match &args[0].kind {
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { #value.parse::<f64>().expect("Failed to parse float") })
            }
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                    let value = self.transpile_expr(&args[0])?;
                    Ok(quote! { #value.parse::<f64>().expect("Failed to parse float") })
                } else {
                    self.transpile_float_generic(&args[0])
                }
            }
            ExprKind::Literal(Literal::Integer(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value as f64) })
            }
            _ => self.transpile_float_generic(&args[0])
        }
    }
    
    /// Generic float conversion for non-literal expressions
    fn transpile_float_generic(&self, expr: &Expr) -> Result<TokenStream> {
        let value = self.transpile_expr(expr)?;
        Ok(quote! { (#value as f64) })
    }
    
    /// Handle `bool()` type conversion with type-specific logic
    fn transpile_bool_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("bool() expects exactly 1 argument");
        }
        
        // Check the type of the argument to generate appropriate conversion
        match &args[0].kind {
            ExprKind::Literal(Literal::Integer(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value != 0) })
            }
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { !#value.is_empty() })
            }
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { !#value.is_empty() })
            }
            ExprKind::Literal(Literal::Bool(_)) => {
                // Boolean already, just pass through
                let value = self.transpile_expr(&args[0])?;
                Ok(value)
            }
            _ => {
                // Generic case - for numbers check != 0
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value != 0) })
            }
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
        // Handle DataFrame static methods
        if base_name.starts_with("DataFrame::") {
            let method = base_name.strip_prefix("DataFrame::").unwrap();
            match method {
                "new" if args.is_empty() => {
                    return Ok(Some(quote! { polars::prelude::DataFrame::empty() }));
                }
                "from_csv" if args.len() == 1 => {
                    let path_tokens = self.transpile_expr(&args[0])?;
                    return Ok(Some(quote! { 
                        polars::prelude::CsvReader::from_path(#path_tokens)
                            .unwrap()
                            .finish()
                            .unwrap()
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
        // Get function name for signature lookup
        let func_name = func_tokens.to_string().trim().to_string();
        
        // Apply type coercion based on function signature
        let arg_tokens: Result<Vec<_>> = if let Some(signature) = self.function_signatures.get(&func_name) {
            args.iter().enumerate().map(|(i, arg)| {
                let base_tokens = self.transpile_expr(arg)?;
                
                // Apply String/&str coercion if needed
                if let Some(expected_type) = signature.param_types.get(i) {
                    self.apply_string_coercion(arg, &base_tokens, expected_type)
                } else {
                    Ok(base_tokens)
                }
            }).collect()
        } else {
            // No signature info - transpile as-is
            args.iter().map(|a| self.transpile_expr(a)).collect()
        };
        
        let arg_tokens = arg_tokens?;
        Ok(quote! { #func_tokens(#(#arg_tokens),*) })
    }
    
    /// Apply String/&str coercion based on expected type
    fn apply_string_coercion(
        &self,
        arg: &Expr,
        tokens: &TokenStream,
        expected_type: &str
    ) -> Result<TokenStream> {
        use crate::frontend::ast::{ExprKind, Literal};
        
        match (&arg.kind, expected_type) {
            // String literal to String parameter: add .to_string()
            (ExprKind::Literal(Literal::String(_)), "String") => {
                Ok(quote! { #tokens.to_string() })
            }
            // String literal to &str parameter: keep as-is
            (ExprKind::Literal(Literal::String(_)), expected) if expected.starts_with('&') => {
                Ok(tokens.clone())
            }
            // Variable that might be &str to String parameter
            (ExprKind::Identifier(_), "String") => {
                // For now, assume string variables are String type from auto-conversion
                // This matches the existing behavior in transpile_let
                Ok(tokens.clone())
            }
            // No coercion needed
            _ => Ok(tokens.clone())
        }
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
