//! Transpiler from Ruchy AST to Rust code

use crate::frontend::ast::{
    Attribute, BinaryOp, Expr, ExprKind, Literal, MatchArm, Param, Pattern, PipelineStage,
    StringPart, Type, TypeKind, UnaryOp,
};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn;

/// Transpiler from Ruchy AST to Rust code
pub struct Transpiler {
    /// Whether to include type annotations
    pub include_types: bool,
}

impl Default for Transpiler {
    fn default() -> Self {
        Self {
            include_types: true,
        }
    }
}

impl Transpiler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Transpile a Ruchy expression to Rust `TokenStream`
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be transpiled to valid Rust code.
    pub fn transpile(&self, expr: &Expr) -> Result<TokenStream> {
        self.transpile_expr(expr)
    }

    /// Transpile to a formatted Rust string
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be transpiled or parsed as valid Rust.
    pub fn transpile_to_string(&self, expr: &Expr) -> Result<String> {
        let tokens = self.transpile(expr)?;
        let file = syn::parse2::<syn::File>(quote! {
            fn main() {
                #tokens
            }
        })?;
        Ok(prettyplease::unparse(&file))
    }

    /// Transpile an expression to Rust tokens
    ///
    /// # Errors
    ///
    /// Returns an error if the expression type is not supported or contains invalid constructs.
    #[allow(clippy::too_many_lines)]
    pub fn transpile_expr(&self, expr: &Expr) -> Result<TokenStream> {
        // Handle attributes first
        if let Some(property_attr) = expr.attributes.iter().find(|attr| attr.name == "property") {
            return self.transpile_property_test(expr, property_attr);
        }

        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Self::transpile_literal(lit)),
            ExprKind::Identifier(name) => {
                let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                Ok(quote! { #ident })
            }
            ExprKind::StringInterpolation { parts } => self.transpile_string_interpolation(parts),
            ExprKind::Binary { left, op, right } => self.transpile_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.transpile_unary(*op, operand),
            ExprKind::Try { expr } => self.transpile_try(expr),
            ExprKind::TryCatch {
                try_block,
                catch_var,
                catch_block,
            } => self.transpile_try_catch(try_block, catch_var, catch_block),
            ExprKind::Await { expr } => self.transpile_await(expr),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.transpile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Let { name, value, body } => self.transpile_let(name, value, body),
            ExprKind::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                is_async,
            } => self.transpile_function(
                name,
                type_params,
                params,
                return_type.as_ref(),
                body,
                *is_async,
            ),
            ExprKind::Lambda { params, body } => self.transpile_lambda(params, body),
            ExprKind::Call { func, args } => self.transpile_call(func, args),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.transpile_method_call(receiver, method, args),
            ExprKind::Block(exprs) => self.transpile_block(exprs),
            ExprKind::Pipeline { expr, stages } => self.transpile_pipeline(expr, stages),
            ExprKind::Match { expr, arms } => self.transpile_match(expr, arms),
            ExprKind::List(elements) => self.transpile_list(elements),
            ExprKind::ListComprehension {
                element,
                variable,
                iterable,
                condition,
            } => {
                self.transpile_list_comprehension(element, variable, iterable, condition.as_deref())
            }
            ExprKind::DataFrame { columns, rows } => self.transpile_dataframe(columns, rows),
            ExprKind::For { var, iter, body } => self.transpile_for(var, iter, body),
            ExprKind::While { condition, body } => self.transpile_while(condition, body),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.transpile_range(start, end, *inclusive),
            ExprKind::Import { path, items } => Ok(Self::transpile_import(path, items)),
            ExprKind::Struct {
                name,
                type_params,
                fields,
            } => self.transpile_struct(name, type_params, fields),
            ExprKind::StructLiteral { name, fields } => self.transpile_struct_literal(name, fields),
            ExprKind::FieldAccess { object, field } => self.transpile_field_access(object, field),
            ExprKind::Trait {
                name,
                type_params,
                methods,
            } => self.transpile_trait(name, type_params, methods),
            ExprKind::Impl {
                type_params,
                trait_name,
                for_type,
                methods,
            } => self.transpile_impl(type_params, trait_name.as_deref(), for_type, methods),
            ExprKind::Actor {
                name,
                state,
                handlers,
            } => self.transpile_actor(name, state, handlers),
            ExprKind::Send { actor, message } => self.transpile_send(actor, message),
            ExprKind::Ask {
                actor,
                message,
                timeout,
            } => self.transpile_ask(actor, message, timeout.as_deref()),
            ExprKind::Break { label } => {
                if let Some(_label) = label {
                    // For now, just emit simple break (labels need more complex handling)
                    Ok(quote! { break })
                } else {
                    Ok(quote! { break })
                }
            }
            ExprKind::Continue { label } => {
                if let Some(_label) = label {
                    // For now, just emit simple continue (labels need more complex handling)
                    Ok(quote! { continue })
                } else {
                    Ok(quote! { continue })
                }
            }
        }
    }

    fn transpile_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::Integer(n) => quote! { #n },
            Literal::Float(f) => quote! { #f },
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Unit => quote! { () },
        }
    }

    /// Transpile string interpolation to format! macro
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::{StringPart, Expr, ExprKind, Literal};
    /// let transpiler = Transpiler::new();
    /// // "Hello, {name}!" becomes format!("Hello, {}!", name)
    /// ```
    pub fn transpile_string_interpolation(&self, parts: &[StringPart]) -> Result<TokenStream> {
        let mut format_string = String::new();
        let mut args = Vec::new();

        for part in parts {
            match part {
                StringPart::Text(text) => {
                    // Escape braces in literal text for format! macro
                    let escaped = text.replace('{', "{{").replace('}', "}}");
                    format_string.push_str(&escaped);
                }
                StringPart::Expr(expr) => {
                    format_string.push_str("{}");
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
            }
        }

        if args.is_empty() {
            // No interpolation, just return the string
            Ok(quote! { #format_string })
        } else {
            // Use format! macro for interpolation
            Ok(quote! { format!(#format_string, #(#args),*) })
        }
    }

    fn transpile_binary(&self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;

        let op_tokens = match op {
            BinaryOp::Add => quote! { + },
            BinaryOp::Subtract => quote! { - },
            BinaryOp::Multiply => quote! { * },
            BinaryOp::Divide => quote! { / },
            BinaryOp::Modulo => quote! { % },
            BinaryOp::Power => {
                // Rust doesn't have a power operator, use method
                return Ok(quote! { (#left_tokens).pow(#right_tokens as u32) });
            }
            BinaryOp::Equal => quote! { == },
            BinaryOp::NotEqual => quote! { != },
            BinaryOp::Less => quote! { < },
            BinaryOp::LessEqual => quote! { <= },
            BinaryOp::Greater => quote! { > },
            BinaryOp::GreaterEqual => quote! { >= },
            BinaryOp::And => quote! { && },
            BinaryOp::Or => quote! { || },
            BinaryOp::BitwiseAnd => quote! { & },
            BinaryOp::BitwiseOr => quote! { | },
            BinaryOp::BitwiseXor => quote! { ^ },
            BinaryOp::LeftShift => quote! { << },
            BinaryOp::RightShift => quote! { >> },
        };

        Ok(quote! { (#left_tokens #op_tokens #right_tokens) })
    }

    fn transpile_unary(&self, op: UnaryOp, operand: &Expr) -> Result<TokenStream> {
        let operand_tokens = self.transpile_expr(operand)?;

        Ok(match op {
            UnaryOp::Not | UnaryOp::BitwiseNot => quote! { !(#operand_tokens) }, // Rust uses ! for both logical and bitwise not
            UnaryOp::Negate => quote! { -(#operand_tokens) },
        })
    }

    fn transpile_try(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        // In Rust, the ? operator is just appended
        Ok(quote! { #expr_tokens? })
    }

    fn transpile_try_catch(
        &self,
        try_block: &Expr,
        catch_var: &str,
        catch_block: &Expr,
    ) -> Result<TokenStream> {
        // Transpile try/catch to a Rust match expression on a Result
        // We wrap the try block in a closure that returns Result
        let try_tokens = self.transpile_expr(try_block)?;
        let catch_var_ident = syn::Ident::new(catch_var, proc_macro2::Span::call_site());
        let catch_tokens = self.transpile_expr(catch_block)?;

        // Generate: match (|| -> Result<_, _> { try_block })() { Ok(v) => v, Err(catch_var) => catch_block }
        Ok(quote! {
            match (|| -> Result<_, Box<dyn std::error::Error>> {
                Ok(#try_tokens)
            })() {
                Ok(__v) => __v,
                Err(#catch_var_ident) => #catch_tokens,
            }
        })
    }

    fn transpile_await(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        // In Rust, await is a postfix operator
        Ok(quote! { #expr_tokens.await })
    }

    fn transpile_if(
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

    fn transpile_let(&self, name: &str, value: &Expr, body: &Expr) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());

        // Check if body is just Unit (meaning simple let statement, not let-in expression)
        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            // Simple let statement
            Ok(quote! {
                let #name_ident = #value_tokens
            })
        } else {
            // Let-in expression
            let body_tokens = self.transpile_expr(body)?;
            Ok(quote! {
                {
                    let #name_ident = #value_tokens;
                    #body_tokens
                }
            })
        }
    }

    fn transpile_function(
        &self,
        name: &str,
        type_params: &[String],
        params: &[Param],
        return_type: Option<&Type>,
        body: &Expr,
        is_async: bool,
    ) -> Result<TokenStream> {
        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());

        // Build generic parameters if any
        let generics = if type_params.is_empty() {
            quote! {}
        } else {
            let type_param_idents: Vec<TokenStream> = type_params
                .iter()
                .map(|tp| {
                    let ident = syn::Ident::new(tp, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            quote! { <#(#type_param_idents),*> }
        };

        let body_tokens = self.transpile_expr(body)?;

        let param_tokens: Vec<TokenStream> = params
            .iter()
            .map(|p| {
                let param_name = syn::Ident::new(&p.name, proc_macro2::Span::call_site());
                let param_type = self
                    .transpile_type(&p.ty)
                    .unwrap_or_else(|_| quote! { impl std::fmt::Display });
                quote! { #param_name: #param_type }
            })
            .collect();

        let return_type_tokens = if let Some(ret_ty) = return_type {
            let ty = self.transpile_type(ret_ty)?;
            quote! { -> #ty }
        } else {
            // No explicit return type - default to unit type ()
            quote! { -> () }
        };

        // If no explicit return type, make sure body doesn't return a value
        let body_tokens = if return_type.is_none() {
            quote! { #body_tokens; }
        } else {
            body_tokens
        };

        if is_async {
            Ok(quote! {
                async fn #name_ident #generics (#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            })
        } else {
            Ok(quote! {
                fn #name_ident #generics (#(#param_tokens),*) #return_type_tokens {
                    #body_tokens
                }
            })
        }
    }

    fn transpile_lambda(&self, params: &[Param], body: &Expr) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;

        // Create parameter list for the closure
        let param_tokens: Vec<_> = params
            .iter()
            .map(|p| {
                let param_name = syn::Ident::new(&p.name, proc_macro2::Span::call_site());
                // For closures, we typically don't specify types and let Rust infer them
                quote! { #param_name }
            })
            .collect();

        // Generate closure syntax
        Ok(quote! {
            |#(#param_tokens),*| #body_tokens
        })
    }

    fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        // Special handling for DataFrame col() function
        if let ExprKind::Identifier(name) = &func.kind {
            if name == "col" {
                // col("column_name") -> polars::prelude::col("column_name")
                if args.len() != 1 {
                    bail!("col() expects exactly one argument");
                }
                let col_name = self.transpile_expr(&args[0])?;
                return Ok(quote! {
                    polars::prelude::col(#col_name)
                });
            }

            // Special handling for println with string interpolation
            if name == "println" && args.len() == 1 {
                if let ExprKind::Literal(Literal::String(s)) = &args[0].kind {
                    // Parse string interpolation: "Hello, {name}!" -> "Hello, {}!", name
                    if let Some((format_str, format_args)) = Self::parse_interpolation(s) {
                        let format_str_lit = format_str.as_str();
                        return Ok(quote! {
                            println!(#format_str_lit, #(#format_args),*)
                        });
                    }
                }
            }
        }

        let func_tokens = self.transpile_expr(func)?;
        let arg_tokens: Result<Vec<_>> = args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;

        Ok(quote! {
            #func_tokens(#(#arg_tokens),*)
        })
    }

    fn transpile_method_call(
        &self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let receiver_tokens = self.transpile_expr(receiver)?;

        // Special handling for DataFrame methods - only if receiver is actually a DataFrame
        if matches!(receiver.kind, ExprKind::DataFrame { .. })
            && (method == "filter"
                || method == "select"
                || method == "groupby"
                || method == "agg"
                || method == "mean"
                || method == "std"
                || method == "sum"
                || method == "count"
                || method == "col")
        {
            return self.transpile_dataframe_method(receiver, method, args);
        }

        // Special handling for Vec extension methods
        match method {
            "sorted" => {
                // vec.sorted() -> { let mut v = vec.clone(); v.sort(); v }
                Ok(quote! {
                    {
                        let mut __sorted = #receiver_tokens.clone();
                        __sorted.sort();
                        __sorted
                    }
                })
            }
            "sum" => {
                // vec.sum() -> vec.iter().sum()
                Ok(quote! {
                    #receiver_tokens.iter().sum()
                })
            }
            "reversed" => {
                // vec.reversed() -> { let mut v = vec.clone(); v.reverse(); v }
                Ok(quote! {
                    {
                        let mut __reversed = #receiver_tokens.clone();
                        __reversed.reverse();
                        __reversed
                    }
                })
            }
            "unique" => {
                // vec.unique() -> vec.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect()
                Ok(quote! {
                    {
                        let __set: std::collections::HashSet<_> = #receiver_tokens.iter().cloned().collect();
                        __set.into_iter().collect::<Vec<_>>()
                    }
                })
            }
            "min" => {
                // vec.min() -> vec.iter().min().cloned()
                Ok(quote! {
                    #receiver_tokens.iter().min().cloned()
                })
            }
            "max" => {
                // vec.max() -> vec.iter().max().cloned()
                Ok(quote! {
                    #receiver_tokens.iter().max().cloned()
                })
            }
            _ => {
                // Default method call
                let method_ident = syn::Ident::new(method, proc_macro2::Span::call_site());
                let arg_tokens: Result<Vec<_>> =
                    args.iter().map(|arg| self.transpile_expr(arg)).collect();
                let arg_tokens = arg_tokens?;

                Ok(quote! {
                    #receiver_tokens.#method_ident(#(#arg_tokens),*)
                })
            }
        }
    }

    fn parse_interpolation(s: &str) -> Option<(String, Vec<TokenStream>)> {
        // Simple interpolation parser for {variable} patterns
        let mut format_str = String::new();
        let mut args = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() != Some(&'{') {
                // Found interpolation start
                let mut var_name = String::new();
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                if var_name.is_empty() {
                    // Empty braces, keep as-is
                    format_str.push_str("{}");
                } else {
                    format_str.push_str("{}");
                    // Convert variable name to identifier token
                    let ident = syn::Ident::new(&var_name, proc_macro2::Span::call_site());
                    args.push(quote! { #ident });
                }
            } else if ch == '{' && chars.peek() == Some(&'{') {
                // Escaped brace
                format_str.push('{');
                chars.next(); // consume second '{'
            } else if ch == '}' && chars.peek() == Some(&'}') {
                // Escaped brace
                format_str.push('}');
                chars.next(); // consume second '}'
            } else {
                format_str.push(ch);
            }
        }

        if args.is_empty() {
            None
        } else {
            Some((format_str, args))
        }
    }

    fn transpile_block(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.is_empty() {
            return Ok(quote! { {} });
        }

        // Optimization: if block contains only one expression, don't add extra braces
        if exprs.len() == 1 {
            return self.transpile_expr(&exprs[0]);
        }

        let mut tokens = Vec::new();
        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            if i < exprs.len() - 1 {
                // Not the last expression, add semicolon
                tokens.push(quote! { #expr_tokens; });
            } else {
                // Last expression, no semicolon (it's the return value)
                tokens.push(expr_tokens);
            }
        }

        Ok(quote! {
            {
                #(#tokens)*
            }
        })
    }

    fn transpile_pipeline(&self, expr: &Expr, stages: &[PipelineStage]) -> Result<TokenStream> {
        // Desugar pipeline: expr |> f |> g becomes g(f(expr))
        let mut result = self.transpile_expr(expr)?;

        for stage in stages {
            // Each stage is a function call with the previous result as first argument
            match &stage.op.kind {
                ExprKind::Identifier(func_name) => {
                    let func = syn::Ident::new(func_name, proc_macro2::Span::call_site());
                    result = quote! { #func(#result) };
                }
                ExprKind::Call { func, args } => {
                    // If the stage is already a call, insert the result as first argument
                    let func_tokens = self.transpile_expr(func)?;
                    let arg_tokens: Result<Vec<_>> =
                        args.iter().map(|arg| self.transpile_expr(arg)).collect();
                    let arg_tokens = arg_tokens?;
                    result = quote! { #func_tokens(#result, #(#arg_tokens),*) };
                }
                _ => bail!("Invalid pipeline stage"),
            }
        }

        Ok(result)
    }

    fn transpile_match(&self, expr: &Expr, arms: &[MatchArm]) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;

        let arm_tokens: Result<Vec<_>> = arms
            .iter()
            .map(|arm| {
                let pattern_tokens = self.transpile_pattern(&arm.pattern)?;
                let body_tokens = self.transpile_expr(&arm.body)?;

                if let Some(guard) = &arm.guard {
                    let guard_tokens = self.transpile_expr(guard)?;
                    Ok(quote! {
                        #pattern_tokens if #guard_tokens => #body_tokens
                    })
                } else {
                    Ok(quote! {
                        #pattern_tokens => #body_tokens
                    })
                }
            })
            .collect();
        let arm_tokens = arm_tokens?;

        Ok(quote! {
            match #expr_tokens {
                #(#arm_tokens),*
            }
        })
    }

    #[allow(clippy::only_used_in_recursion)] // Self parameter needed for consistency
    fn transpile_pattern(&self, pattern: &Pattern) -> Result<TokenStream> {
        Ok(match pattern {
            Pattern::Wildcard => quote! { _ },
            Pattern::Literal(lit) => Self::transpile_literal(lit),
            Pattern::Identifier(name) => {
                let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                quote! { #ident }
            }
            Pattern::List(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                quote! { [#(#pattern_tokens),*] }
            }
        })
    }

    fn transpile_list(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;

        Ok(quote! {
            vec![#(#element_tokens),*]
        })
    }

    fn transpile_list_comprehension(
        &self,
        element: &Expr,
        variable: &str,
        iterable: &Expr,
        condition: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_ident = syn::Ident::new(variable, proc_macro2::Span::call_site());
        let iterable_tokens = self.transpile_expr(iterable)?;
        let element_tokens = self.transpile_expr(element)?;

        if let Some(cond) = condition {
            // List comprehension with filter: [expr for var in iterable if condition]
            // Transpiles to: iterable.into_iter().filter(|var| condition).map(|var| expr).collect()
            let condition_tokens = self.transpile_expr(cond)?;
            Ok(quote! {
                #iterable_tokens
                    .into_iter()
                    .filter(|#var_ident| #condition_tokens)
                    .map(|#var_ident| #element_tokens)
                    .collect::<Vec<_>>()
            })
        } else {
            // Simple list comprehension: [expr for var in iterable]
            // Transpiles to: iterable.into_iter().map(|var| expr).collect()
            Ok(quote! {
                #iterable_tokens
                    .into_iter()
                    .map(|#var_ident| #element_tokens)
                    .collect::<Vec<_>>()
            })
        }
    }

    fn transpile_for(&self, var: &str, iter: &Expr, body: &Expr) -> Result<TokenStream> {
        let var_ident = syn::Ident::new(var, proc_macro2::Span::call_site());
        let iter_tokens = self.transpile_expr(iter)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            for #var_ident in #iter_tokens {
                #body_tokens
            }
        })
    }

    fn transpile_while(&self, condition: &Expr, body: &Expr) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            while #cond_tokens {
                #body_tokens
            }
        })
    }

    fn transpile_range(&self, start: &Expr, end: &Expr, inclusive: bool) -> Result<TokenStream> {
        let start_tokens = self.transpile_expr(start)?;
        let end_tokens = self.transpile_expr(end)?;

        if inclusive {
            Ok(quote! { (#start_tokens..=#end_tokens) })
        } else {
            Ok(quote! { (#start_tokens..#end_tokens) })
        }
    }

    fn transpile_dataframe_method(
        &self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let receiver_tokens = self.transpile_expr(receiver)?;

        match method {
            "filter" => {
                // df.filter(condition)
                if args.len() != 1 {
                    bail!("filter expects exactly one argument");
                }
                let condition = self.transpile_expr(&args[0])?;
                Ok(quote! {
                    #receiver_tokens.lazy().filter(#condition).collect()?
                })
            }
            "select" => {
                // df.select([col1, col2, ...])
                let col_tokens: Result<Vec<_>> =
                    args.iter().map(|arg| self.transpile_expr(arg)).collect();
                let col_tokens = col_tokens?;
                Ok(quote! {
                    #receiver_tokens.select([#(#col_tokens),*])?
                })
            }
            "groupby" => {
                // df.groupby(columns)
                if args.is_empty() {
                    bail!("groupby expects at least one column");
                }
                let col_tokens: Result<Vec<_>> =
                    args.iter().map(|arg| self.transpile_expr(arg)).collect();
                let col_tokens = col_tokens?;
                Ok(quote! {
                    #receiver_tokens.groupby([#(#col_tokens),*])?
                })
            }
            "agg" => {
                // df.agg(aggregations)
                let agg_tokens: Result<Vec<_>> =
                    args.iter().map(|arg| self.transpile_expr(arg)).collect();
                let agg_tokens = agg_tokens?;
                Ok(quote! {
                    #receiver_tokens.agg([#(#agg_tokens),*])?
                })
            }
            "mean" => {
                // df.mean() or series.mean()
                if !args.is_empty() {
                    bail!("mean takes no arguments");
                }
                Ok(quote! {
                    #receiver_tokens.mean()
                })
            }
            "std" => {
                // df.std() or series.std()
                if !args.is_empty() {
                    bail!("std takes no arguments");
                }
                Ok(quote! {
                    #receiver_tokens.std()
                })
            }
            "sum" => {
                // df.sum() or series.sum()
                if !args.is_empty() {
                    bail!("sum takes no arguments");
                }
                Ok(quote! {
                    #receiver_tokens.sum()
                })
            }
            "count" => {
                // df.count() or series.count()
                if !args.is_empty() {
                    bail!("count takes no arguments");
                }
                Ok(quote! {
                    #receiver_tokens.shape().0
                })
            }
            "col" => {
                // df.col("column_name")
                if args.len() != 1 {
                    bail!("col expects exactly one argument");
                }
                let col_name = self.transpile_expr(&args[0])?;
                Ok(quote! {
                    polars::prelude::col(#col_name)
                })
            }
            _ => bail!("Unknown DataFrame method: {}", method),
        }
    }

    fn transpile_dataframe(&self, columns: &[String], rows: &[Vec<Expr>]) -> Result<TokenStream> {
        // Generate Polars DataFrame creation code
        // df! macro equivalent in Polars:
        // DataFrame::new(vec![
        //     Series::new("col1", vec![val1, val2, ...]),
        //     Series::new("col2", vec![val1, val2, ...]),
        // ])

        if columns.is_empty() {
            return Ok(quote! { polars::frame::DataFrame::empty() });
        }

        // Transpose rows to columns for Polars Series creation
        let mut series_data: Vec<Vec<TokenStream>> = vec![Vec::new(); columns.len()];

        for row in rows {
            for (i, value) in row.iter().enumerate() {
                let value_tokens = self.transpile_expr(value)?;
                series_data[i].push(value_tokens);
            }
        }

        // Create Series for each column
        let series_tokens: Vec<TokenStream> = columns
            .iter()
            .zip(series_data.iter())
            .map(|(col_name, col_data)| {
                quote! {
                    polars::series::Series::new(#col_name, vec![#(#col_data),*])
                }
            })
            .collect();

        Ok(quote! {
            polars::frame::DataFrame::new(vec![
                #(#series_tokens),*
            ]).expect("Failed to create DataFrame")
        })
    }

    fn transpile_import(path: &str, items: &[String]) -> TokenStream {
        // Convert path string to token stream
        // For now, we'll just generate a comment since use statements need special handling
        if items.is_empty() {
            // Simple import: import std::io
            let _comment = format!("// use {path};");
            quote! {
                // Import would be: use #path;
            }
        } else {
            // Import with items: import std::io::{Read, Write}
            let items_str = items.join(", ");
            let _comment = format!("// use {path}::{{{items_str}}};");
            quote! {
                // Import would be: use #path::{#items_str};
            }
        }
    }

    fn transpile_type(&self, ty: &Type) -> Result<TokenStream> {
        let _ = self; // Suppress unused self warning
        Ok(match &ty.kind {
            TypeKind::Named(name) => {
                // Map common Ruchy types to Rust types
                let rust_type = match name.as_str() {
                    "i32" => quote! { i32 },
                    "i64" => quote! { i64 },
                    "f32" => quote! { f32 },
                    "f64" => quote! { f64 },
                    "bool" => quote! { bool },
                    "String" => quote! { String },
                    "Any" => quote! { impl std::fmt::Display }, // Gradual typing - use trait bounds
                    _ => {
                        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                        quote! { #ident }
                    }
                };
                rust_type
            }
            TypeKind::Generic { base, params } => {
                let base_ident = syn::Ident::new(base, proc_macro2::Span::call_site());
                let param_tokens: Result<Vec<_>> =
                    params.iter().map(|p| self.transpile_type(p)).collect();
                let param_tokens = param_tokens?;
                quote! { #base_ident<#(#param_tokens),*> }
            }
            TypeKind::Optional(inner) => {
                let inner_tokens = self.transpile_type(inner)?;
                quote! { Option<#inner_tokens> }
            }
            TypeKind::List(inner) => {
                let inner_tokens = self.transpile_type(inner)?;
                quote! { Vec<#inner_tokens> }
            }
            TypeKind::Function { params, ret } => {
                let param_tokens: Result<Vec<_>> =
                    params.iter().map(|p| self.transpile_type(p)).collect();
                let param_tokens = param_tokens?;
                let ret_tokens = self.transpile_type(ret)?;
                quote! { fn(#(#param_tokens),*) -> #ret_tokens }
            }
        })
    }

    fn transpile_struct(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[crate::frontend::ast::StructField],
    ) -> Result<TokenStream> {
        let struct_name = syn::Ident::new(name, proc_macro2::Span::call_site());

        // Build generic parameters if any
        let generics = if type_params.is_empty() {
            quote! {}
        } else {
            let type_param_idents: Vec<TokenStream> = type_params
                .iter()
                .map(|tp| {
                    let ident = syn::Ident::new(tp, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            quote! { <#(#type_param_idents),*> }
        };

        let field_tokens: Result<Vec<_>> = fields
            .iter()
            .map(|field| {
                let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                let field_type = self.transpile_type(&field.ty)?;

                if field.is_pub {
                    Ok(quote! { pub #field_name: #field_type })
                } else {
                    Ok(quote! { #field_name: #field_type })
                }
            })
            .collect();
        let field_tokens = field_tokens?;

        Ok(quote! {
            struct #struct_name #generics {
                #(#field_tokens),*
            }
        })
    }

    fn transpile_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> Result<TokenStream> {
        let struct_name = syn::Ident::new(name, proc_macro2::Span::call_site());

        let field_tokens: Result<Vec<_>> = fields
            .iter()
            .map(|(field_name, value)| {
                let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());
                let value_tokens = self.transpile_expr(value)?;
                Ok(quote! { #field_ident: #value_tokens })
            })
            .collect();
        let field_tokens = field_tokens?;

        Ok(quote! {
            #struct_name {
                #(#field_tokens),*
            }
        })
    }

    fn transpile_field_access(&self, object: &Expr, field: &str) -> Result<TokenStream> {
        let object_tokens = self.transpile_expr(object)?;
        let field_ident = syn::Ident::new(field, proc_macro2::Span::call_site());

        Ok(quote! {
            #object_tokens.#field_ident
        })
    }

    fn transpile_trait(
        &self,
        name: &str,
        type_params: &[String],
        methods: &[crate::frontend::ast::TraitMethod],
    ) -> Result<TokenStream> {
        let trait_name = syn::Ident::new(name, proc_macro2::Span::call_site());

        // Build generic parameters if any
        let generics = if type_params.is_empty() {
            quote! {}
        } else {
            let type_param_idents: Vec<TokenStream> = type_params
                .iter()
                .map(|tp| {
                    let ident = syn::Ident::new(tp, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            quote! { <#(#type_param_idents),*> }
        };

        let method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = syn::Ident::new(&method.name, proc_macro2::Span::call_site());

                // Determine self type and other parameters (similar to impl block handling)
                let (self_tokens, other_params) = if method.params.is_empty() {
                    // No parameters at all
                    (quote! {}, &method.params[..])
                } else {
                    let first_param = &method.params[0];
                    if first_param.name == "self" || first_param.name == "mut self" {
                        // Method with self parameter
                        let self_tok = if first_param.name == "mut self" {
                            quote! { &mut self }
                        } else {
                            // Check if type annotation suggests mutability or ownership
                            match &first_param.ty.kind {
                                TypeKind::Named(name) if name == "&mut self" => {
                                    quote! { &mut self }
                                }
                                TypeKind::Named(name) if name == "self" => quote! { self },
                                _ => quote! { &self }, // Default to immutable reference
                            }
                        };
                        (self_tok, &method.params[1..])
                    } else {
                        // Associated function (no self parameter)
                        (quote! {}, &method.params[..])
                    }
                };

                // Parse remaining parameters
                let param_tokens: Result<Vec<_>> = other_params
                    .iter()
                    .map(|p| {
                        let param_name = syn::Ident::new(&p.name, proc_macro2::Span::call_site());
                        let param_type = self.transpile_type(&p.ty)?;
                        Ok(quote! { #param_name: #param_type })
                    })
                    .collect();
                let param_tokens = param_tokens?;

                // Return type
                let return_type_tokens = if let Some(ret_ty) = &method.return_type {
                    let ty = self.transpile_type(ret_ty)?;
                    quote! { -> #ty }
                } else {
                    quote! {}
                };

                // Method body (for default implementations)
                if let Some(body) = &method.body {
                    let body_tokens = self.transpile_expr(body)?;
                    // Generate method with appropriate self type
                    if self_tokens.is_empty() {
                        // Associated function
                        Ok(quote! {
                            fn #method_name(#(#param_tokens),*) #return_type_tokens {
                                #body_tokens
                            }
                        })
                    } else {
                        // Instance method
                        Ok(quote! {
                            fn #method_name(#self_tokens, #(#param_tokens),*) #return_type_tokens {
                                #body_tokens
                            }
                        })
                    }
                } else {
                    // Just the signature
                    if self_tokens.is_empty() {
                        // Associated function
                        Ok(quote! {
                            fn #method_name(#(#param_tokens),*) #return_type_tokens;
                        })
                    } else {
                        // Instance method
                        Ok(quote! {
                            fn #method_name(#self_tokens, #(#param_tokens),*) #return_type_tokens;
                        })
                    }
                }
            })
            .collect();
        let method_tokens = method_tokens?;

        Ok(quote! {
            trait #trait_name #generics {
                #(#method_tokens)*
            }
        })
    }

    fn transpile_impl(
        &self,
        type_params: &[String],
        trait_name: Option<&str>,
        for_type: &str,
        methods: &[crate::frontend::ast::ImplMethod],
    ) -> Result<TokenStream> {
        let type_name = syn::Ident::new(for_type, proc_macro2::Span::call_site());

        // Build generic parameters if any
        let generics = if type_params.is_empty() {
            quote! {}
        } else {
            let type_param_idents: Vec<TokenStream> = type_params
                .iter()
                .map(|tp| {
                    let ident = syn::Ident::new(tp, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            quote! { <#(#type_param_idents),*> }
        };

        let method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = syn::Ident::new(&method.name, proc_macro2::Span::call_site());

                // Determine self type and other parameters
                let (self_tokens, other_params) = if method.params.is_empty() {
                    // No parameters at all
                    (quote! {}, &method.params[..])
                } else {
                    let first_param = &method.params[0];
                    if first_param.name == "self" || first_param.name == "mut self" {
                        // Method with self parameter
                        let self_tok = if first_param.name == "mut self" {
                            quote! { &mut self }
                        } else {
                            // Check if type annotation suggests mutability or ownership
                            match &first_param.ty.kind {
                                TypeKind::Named(name) if name == "&mut self" => {
                                    quote! { &mut self }
                                }
                                TypeKind::Named(name) if name == "self" => quote! { self },
                                _ => quote! { &self }, // Default to immutable reference
                            }
                        };
                        (self_tok, &method.params[1..])
                    } else {
                        // Static method (no self parameter)
                        (quote! {}, &method.params[..])
                    }
                };

                // Parse remaining parameters
                let param_tokens: Result<Vec<_>> = other_params
                    .iter()
                    .map(|p| {
                        let param_name = syn::Ident::new(&p.name, proc_macro2::Span::call_site());
                        let param_type = self.transpile_type(&p.ty)?;
                        Ok(quote! { #param_name: #param_type })
                    })
                    .collect();
                let param_tokens = param_tokens?;

                // Return type
                let return_type_tokens = if let Some(ret_ty) = &method.return_type {
                    let ty = self.transpile_type(ret_ty)?;
                    quote! { -> #ty }
                } else {
                    quote! {}
                };

                // Method body
                let body_tokens = self.transpile_expr(&method.body)?;

                // Generate method with appropriate self type
                if self_tokens.is_empty() {
                    // Static method
                    Ok(quote! {
                        fn #method_name(#(#param_tokens),*) #return_type_tokens {
                            #body_tokens
                        }
                    })
                } else {
                    // Instance method
                    Ok(quote! {
                        fn #method_name(#self_tokens, #(#param_tokens),*) #return_type_tokens {
                            #body_tokens
                        }
                    })
                }
            })
            .collect();
        let method_tokens = method_tokens?;

        if let Some(trait_name) = trait_name {
            let trait_ident = syn::Ident::new(trait_name, proc_macro2::Span::call_site());
            Ok(quote! {
                impl #generics #trait_ident for #type_name {
                    #(#method_tokens)*
                }
            })
        } else {
            // Inherent impl
            Ok(quote! {
                impl #generics #type_name {
                    #(#method_tokens)*
                }
            })
        }
    }

    fn transpile_actor(
        &self,
        name: &str,
        state: &[crate::frontend::ast::StructField],
        handlers: &[crate::frontend::ast::ActorHandler],
    ) -> Result<TokenStream> {
        let actor_name = syn::Ident::new(name, proc_macro2::Span::call_site());

        // Generate state struct
        let field_tokens: Result<Vec<_>> = state
            .iter()
            .map(|f| {
                let field_name = syn::Ident::new(&f.name, proc_macro2::Span::call_site());
                let field_type = self.transpile_type(&f.ty)?;
                Ok(quote! { #field_name: #field_type })
            })
            .collect();
        let field_tokens = field_tokens?;

        // Generate message enum
        let message_variants: Vec<_> = handlers
            .iter()
            .map(|h| {
                let variant_name = syn::Ident::new(&h.message_type, proc_macro2::Span::call_site());
                // For simplicity, assuming messages have no parameters for now
                quote! { #variant_name }
            })
            .collect();

        let message_enum_name =
            syn::Ident::new(&format!("{name}Message"), proc_macro2::Span::call_site());

        // Generate handler match arms
        let handler_arms: Result<Vec<_>> = handlers
            .iter()
            .map(|h| {
                let variant_name = syn::Ident::new(&h.message_type, proc_macro2::Span::call_site());
                let body = self.transpile_expr(&h.body)?;
                Ok(quote! {
                    #message_enum_name::#variant_name => {
                        #body
                    }
                })
            })
            .collect();
        let handler_arms = handler_arms?;

        // Generate actor implementation using tokio
        Ok(quote! {
            struct #actor_name {
                #(#field_tokens,)*
            }

            #[derive(Debug, Clone)]
            enum #message_enum_name {
                #(#message_variants,)*
            }

            impl #actor_name {
                async fn handle_message(&mut self, msg: #message_enum_name) {
                    match msg {
                        #(#handler_arms)*
                    }
                }

                fn spawn(self) -> tokio::sync::mpsc::Sender<#message_enum_name> {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
                    let mut actor = self;
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            actor.handle_message(msg).await;
                        }
                    });
                    tx
                }
            }
        })
    }

    fn transpile_send(&self, actor: &Expr, message: &Expr) -> Result<TokenStream> {
        let actor_tokens = self.transpile_expr(actor)?;
        let message_tokens = self.transpile_expr(message)?;

        // Generate send operation
        Ok(quote! {
            #actor_tokens.send(#message_tokens).await
        })
    }

    fn transpile_ask(
        &self,
        actor: &Expr,
        message: &Expr,
        timeout: Option<&Expr>,
    ) -> Result<TokenStream> {
        let actor_tokens = self.transpile_expr(actor)?;
        let message_tokens = self.transpile_expr(message)?;

        if let Some(timeout_expr) = timeout {
            let timeout_tokens = self.transpile_expr(timeout_expr)?;
            // Generate ask with timeout
            Ok(quote! {
                tokio::time::timeout(
                    std::time::Duration::from_millis(#timeout_tokens),
                    #actor_tokens.ask(#message_tokens)
                ).await
            })
        } else {
            // Generate ask without timeout
            Ok(quote! {
                #actor_tokens.ask(#message_tokens).await
            })
        }
    }

    fn transpile_property_test(&self, expr: &Expr, _attr: &Attribute) -> Result<TokenStream> {
        // Property tests must be functions
        if let ExprKind::Function {
            name,
            type_params,
            params,
            body,
            ..
        } = &expr.kind
        {
            let fn_name = syn::Ident::new(name, proc_macro2::Span::call_site());
            let test_fn_name = syn::Ident::new(
                &format!("test_property_{name}"),
                proc_macro2::Span::call_site(),
            );

            // Generate parameter types for proptest
            let mut param_strategies = Vec::new();
            let mut param_names = Vec::new();

            for param in params {
                let param_name = syn::Ident::new(&param.name, proc_macro2::Span::call_site());
                param_names.push(param_name.clone());

                let strategy = match &param.ty.kind {
                    TypeKind::Named(name) => match name.as_str() {
                        "i32" | "i64" => quote! { any::<i32>() },
                        "f32" | "f64" => quote! { any::<f64>() },
                        "bool" => quote! { any::<bool>() },
                        "String" => quote! { any::<String>() },
                        _ => quote! { any::<i32>() }, // Default fallback
                    },
                    TypeKind::Generic { base, .. } => {
                        // For generic types, use the base type strategy
                        match base.as_str() {
                            "Vec" => quote! { prop::collection::vec(any::<i32>(), 0..100) },
                            "Option" => quote! { prop::option::of(any::<i32>()) },
                            _ => quote! { any::<i32>() },
                        }
                    }
                    TypeKind::List(elem_ty) => {
                        let elem_strategy = if let TypeKind::Named(name) = &elem_ty.kind {
                            match name.as_str() {
                                "i32" | "i64" => quote! { any::<i32>() },
                                "f32" | "f64" => quote! { any::<f64>() },
                                "bool" => quote! { any::<bool>() },
                                "String" => quote! { any::<String>() },
                                _ => quote! { any::<i32>() },
                            }
                        } else {
                            quote! { any::<i32>() }
                        };
                        quote! { prop::collection::vec(#elem_strategy, 0..100) }
                    }
                    _ => quote! { any::<i32>() }, // Default fallback
                };

                param_strategies.push(quote! { #param_name in #strategy });
            }

            // Transpile the function body
            let _body_tokens = self.transpile_expr(body)?;

            // Generate the original function
            let original_fn =
                self.transpile_function(name, type_params, params, None, body, false)?;

            // Generate the property test
            let test_tokens = quote! {
                #original_fn

                #[cfg(test)]
                mod property_tests {
                    use super::*;
                    use proptest::prelude::*;

                    proptest! {
                        #[test]
                        fn #test_fn_name(#(#param_strategies),*) {
                            // Call the function and verify it doesn't panic
                            let _result = #fn_name(#(#param_names),*);
                            // Property test passes if function completes without panic
                            prop_assert!(true);
                        }
                    }
                }
            };

            Ok(test_tokens)
        } else {
            bail!("#[property] attribute can only be applied to functions");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::Parser;

    fn transpile_str(input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        transpiler.transpile_to_string(&ast)
    }

    #[test]
    fn test_transpile_literals() {
        let result = transpile_str("42").unwrap();
        assert!(result.contains("42"));

        let result = transpile_str("3.14").unwrap();
        assert!(result.contains("3.14"));

        let result = transpile_str("\"hello\"").unwrap();
        assert!(result.contains("\"hello\""));

        let result = transpile_str("true").unwrap();
        assert!(result.contains("true"));
    }

    #[test]
    fn test_transpile_binary_ops() {
        let result = transpile_str("1 + 2 * 3").unwrap();
        // Check that the operations are present (formatting may vary)
        assert!(result.contains('1'));
        assert!(result.contains('2'));
        assert!(result.contains('3'));
        assert!(result.contains('+'));
        assert!(result.contains('*'));
    }

    #[test]
    fn test_transpile_if() {
        let result = transpile_str("if x > 0 { positive } else { negative }").unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_transpile_function() {
        let result = transpile_str("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        assert!(result.contains("fn add"));
        assert!(result.contains("x: i32"));
        assert!(result.contains("y: i32"));
        assert!(result.contains("-> i32"));
    }

    #[test]
    fn test_transpile_list() {
        let result = transpile_str("[1, 2, 3]").unwrap();
        assert!(result.contains("vec!"));
        assert!(result.contains('1'));
        assert!(result.contains('2'));
        assert!(result.contains('3'));
    }

    #[test]
    fn test_transpile_match() {
        let result = transpile_str(r#"match x { 1 => "one", _ => "other" }"#).unwrap();
        assert!(result.contains("match"));
        assert!(result.contains('1'));
        assert!(result.contains("\"one\""));
        assert!(result.contains("\"other\""));
    }

    #[test]
    fn test_transpile_let() {
        let result = transpile_str("let x = 42 in x + 1").unwrap();
        assert!(result.contains("let x"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_transpile_for() {
        let result = transpile_str("for i in 1..10 { print(i) }").unwrap();
        assert!(result.contains("for i"));
        assert!(result.contains("in"));
    }

    #[test]
    fn test_transpile_range() {
        let result = transpile_str("1..10").unwrap();
        assert!(result.contains(".."));
        assert!(!result.contains("..="));

        let result = transpile_str("1..=10").unwrap();
        assert!(result.contains("..="));
    }

    #[test]
    fn test_transpile_pipeline() {
        let result = transpile_str("x |> f |> g").unwrap();
        // Pipeline becomes nested function calls: g(f(x))
        assert!(result.contains('g'));
        assert!(result.contains('f'));
    }

    #[test]
    fn test_transpile_unary() {
        let result = transpile_str("!true").unwrap();
        assert!(result.contains('!'));
        assert!(result.contains("true"));

        let result = transpile_str("-42").unwrap();
        assert!(result.contains('-'));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_transpile_block() {
        // Blocks are part of function bodies or if expressions
        let result = transpile_str("if true { let x = 1; x + 1 } else { 0 }").unwrap();
        // Block should have braces
        assert!(result.contains('{'));
        assert!(result.contains('}'));
        assert!(result.contains("let x"));
    }

    #[test]
    fn test_transpile_lambda() {
        // Simple lambda
        let result = transpile_str("|x| x + 1").unwrap();
        assert!(result.contains("|x|"));
        assert!(result.contains("x + 1"));

        // Lambda with multiple parameters
        let result = transpile_str("|x, y| x * y").unwrap();
        assert!(result.contains("|x, y|"));
        assert!(result.contains("x * y"));

        // Lambda with no parameters
        let result = transpile_str("|| 42").unwrap();
        assert!(result.contains("||"));
        assert!(result.contains("42"));

        // Lambda in a function call context
        let result = transpile_str("map(|x| x * 2)").unwrap();
        assert!(result.contains("map"));
        assert!(result.contains("|x|"));
        assert!(result.contains("x * 2"));
    }

    #[test]
    #[cfg(feature = "dataframe")]
    fn test_transpile_dataframe() {
        // Simple DataFrame
        let result = transpile_str("df![name, age; \"Alice\", 30; \"Bob\", 25]").unwrap();
        assert!(result.contains("DataFrame::new"));
        assert!(result.contains("Series::new"));
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"age\""));

        // Empty DataFrame
        let result = transpile_str("df![]").unwrap();
        assert!(result.contains("DataFrame::empty"));

        // Single column DataFrame
        let result = transpile_str("df![values; 1; 2; 3]").unwrap();
        assert!(result.contains("Series::new"));
        assert!(result.contains("\"values\""));
    }

    #[test]
    fn test_transpile_dataframe_operations() {
        // Create a DataFrame and test operations
        let code = "df![name, age; \"Alice\", 30; \"Bob\", 25].mean()";
        let result = transpile_str(code).unwrap();
        assert!(result.contains(".mean()"));
    }

    #[test]
    fn test_transpile_try_operator() {
        // Simple try
        let result = transpile_str("foo()?").unwrap();
        assert!(result.contains("foo()?"));

        // Try with method call
        let result = transpile_str("x.method()?").unwrap();
        assert!(result.contains("x.method()?"));

        // Chained try operations (use parentheses to avoid SafeNav token)
        let result = transpile_str("(get_data()?).process()?").unwrap();
        assert!(result.contains("get_data()?"));
    }

    #[test]
    fn test_transpile_while() {
        // Simple while loop
        let result = transpile_str("while x < 10 { println(x) }").unwrap();
        assert!(result.contains("while"));
        assert!(result.contains("x < 10"));
        assert!(result.contains("println"));

        // While with boolean literal
        let result = transpile_str("while true { println(\"loop\") }").unwrap();
        assert!(result.contains("while true"));
        assert!(result.contains("println"));
    }

    #[test]
    fn test_transpile_struct() {
        // Simple struct definition
        let result = transpile_str("struct Point { x: i32, y: i32 }").unwrap();
        assert!(result.contains("struct Point"));
        assert!(result.contains("x: i32"));
        assert!(result.contains("y: i32"));

        // Struct with public fields
        let result = transpile_str("struct Person { pub name: String, pub age: i32 }").unwrap();
        assert!(result.contains("struct Person"));
        assert!(result.contains("pub name: String"));
        assert!(result.contains("pub age: i32"));
    }

    #[test]
    fn test_transpile_struct_literal() {
        // Simple struct instantiation
        let result = transpile_str("Point { x: 10, y: 20 }").unwrap();
        assert!(result.contains("Point"));
        assert!(result.contains("x: 10"));
        assert!(result.contains("y: 20"));

        // Struct literal with expressions
        let result = transpile_str("Person { name: \"Alice\", age: 25 + 5 }").unwrap();
        assert!(result.contains("Person"));
        assert!(result.contains("name: \"Alice\""));
        assert!(result.contains("age:"));
        assert!(result.contains("25"));
        assert!(result.contains('5'));
    }

    #[test]
    fn test_transpile_field_access() {
        // Simple field access
        let result = transpile_str("point.x").unwrap();
        assert!(result.contains("point.x"));

        // Chained field access
        let result = transpile_str("obj.field1.field2").unwrap();
        assert!(result.contains(".field1"));
        assert!(result.contains(".field2"));
    }

    #[test]
    fn test_transpile_trait() {
        // Simple trait with instance method
        let result = transpile_str("trait Display { fun show(self) -> String }").unwrap();
        assert!(result.contains("trait Display"));
        assert!(result.contains("fn show(&self) -> String"));

        // Trait with default implementation
        let result = transpile_str("trait Greet { fun hello(self) { println(\"Hi\") } }").unwrap();
        assert!(result.contains("trait Greet"));
        assert!(result.contains("fn hello(&self)"));
        assert!(result.contains("println"));

        // Trait with associated function (no self)
        let result = transpile_str("trait Factory { fun create(name: String) -> Self }").unwrap();
        assert!(result.contains("trait Factory"));
        assert!(result.contains("fn create(name: String) -> Self"));
        assert!(!result.contains("&self"));
    }

    #[test]
    fn test_transpile_vec_methods() {
        // sorted method
        let result = transpile_str("[3, 1, 2].sorted()").unwrap();
        assert!(result.contains("sort()"));
        assert!(result.contains("clone()"));

        // sum method
        let result = transpile_str("[1, 2, 3].sum()").unwrap();
        assert!(result.contains("iter().sum()"));

        // reversed method
        let result = transpile_str("[1, 2, 3].reversed()").unwrap();
        assert!(result.contains("reverse()"));

        // unique method
        let result = transpile_str("[1, 2, 2, 3].unique()").unwrap();
        assert!(result.contains("HashSet"));

        // min/max methods
        let result = transpile_str("[1, 2, 3].min()").unwrap();
        assert!(result.contains("iter().min()"));

        let result = transpile_str("[1, 2, 3].max()").unwrap();
        assert!(result.contains("iter().max()"));
    }

    #[test]
    fn test_transpile_try_catch() {
        let code = r"
            try {
                42
            } catch (e) {
                0
            }
        ";
        let result = transpile_str(code).unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("Ok"));
        assert!(result.contains("Err"));
        assert!(result.contains("Result"));
    }

    #[test]
    fn test_transpile_actor() {
        let code = r"
            actor Counter {
                count: i32,
                
                receive {
                    Increment => { 42 }
                    Get => { 100 }
                }
            }
        ";
        let result = transpile_str(code).unwrap();
        assert!(result.contains("struct Counter"));
        assert!(result.contains("enum CounterMessage"));
        assert!(result.contains("async fn handle_message"));
        assert!(result.contains("tokio::spawn"));
    }

    #[test]
    fn test_transpile_send() {
        let code = "counter ! Increment";
        let result = transpile_str(code).unwrap();
        assert!(result.contains(".send("));
        assert!(result.contains(".await"));
    }

    #[test]
    fn test_transpile_ask() {
        let code = "counter ? Get";
        let result = transpile_str(code).unwrap();
        assert!(result.contains(".ask("));
        assert!(result.contains(".await"));
    }

    #[test]
    fn test_transpile_break_continue() {
        // Test break
        let code = "while true { break }";
        let result = transpile_str(code).unwrap();
        assert!(result.contains("break"));

        // Test continue
        let code = "while true { continue }";
        let result = transpile_str(code).unwrap();
        assert!(result.contains("continue"));
    }

    #[test]
    fn test_transpile_async() {
        // Async function
        let result = transpile_str("async fun fetch() -> String { \"data\" }").unwrap();
        assert!(result.contains("async fn fetch"));
        assert!(result.contains("-> String"));
        assert!(result.contains("\"data\""));

        // Await expression
        let result = transpile_str("await fetch()").unwrap();
        assert!(result.contains("fetch().await"));
    }

    #[test]
    fn test_transpile_col_function() {
        // Test standalone col() function
        let result = transpile_str("col(\"name\")").unwrap();
        assert!(result.contains("polars::prelude::col"));
        assert!(result.contains("\"name\""));

        // Test col() in expression context (like col("score") > 90)
        let result = transpile_str("col(\"score\") > 90").unwrap();
        assert!(result.contains("polars::prelude::col"));
        assert!(result.contains("\"score\""));
        assert!(result.contains("> 90"));
    }

    #[test]
    fn test_transpile_impl() {
        // Inherent impl with &self
        let result = transpile_str("impl Point { fun distance(self) -> f64 { 0.0 } }").unwrap();
        assert!(result.contains("impl Point"));
        assert!(result.contains("fn distance(&self) -> f64"));
        assert!(result.contains("0f64")); // Rust formats 0.0 as 0f64

        // Trait impl
        let result =
            transpile_str("impl Display for Point { fun show(self) -> String { \"Point\" } }")
                .unwrap();
        assert!(result.contains("impl Display for Point"));
        assert!(result.contains("fn show(&self) -> String"));
        assert!(result.contains("\"Point\""));

        // Static method (no self parameter)
        let result =
            transpile_str("impl Point { fun new(x: f64, y: f64) -> Point { Point } }").unwrap();
        assert!(result.contains("fn new(x: f64, y: f64) -> Point"));
        assert!(!result.contains("self"));
    }

    #[test]
    fn test_transpile_string_interpolation() {
        // Test simple interpolation
        let result = transpile_str("\"Hello, {name}!\"").unwrap();
        assert!(result.contains("format!"));
        assert!(result.contains("Hello, {}!"));
        assert!(result.contains("name"));

        // Test complex interpolation
        let result = transpile_str("\"Result: {x + y}\"").unwrap();
        assert!(result.contains("format!"));
        assert!(result.contains("Result: {}"));
        assert!(result.contains("x + y"));

        // Test no interpolation
        let result = transpile_str("\"Simple string\"").unwrap();
        assert!(!result.contains("format!"));
        assert!(result.contains("Simple string"));
    }

    #[test]
    fn test_transpile_string_interpolation_escaped() {
        let result = transpile_str("\"Value: {{static}} and {dynamic}\"").unwrap();
        assert!(result.contains("format!"));
        assert!(result.contains("Value: {{static}} and {}"));
        assert!(result.contains("dynamic"));
    }

    #[test]
    fn test_transpile_property_test() {
        let result = transpile_str("#[property] fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        assert!(result.contains("fn add"));
        assert!(result.contains("mod property_tests"));
        assert!(result.contains("use proptest::prelude::*"));
        assert!(result.contains("proptest!"));
        assert!(result.contains("fn test_property_add"));
    }

    #[test]
    fn test_transpile_list_comprehension() {
        // Simple comprehension: [x * 2 for x in numbers]
        let result = transpile_str("[x * 2 for x in numbers]").unwrap();
        assert!(result.contains("numbers"));
        assert!(result.contains(".into_iter()"));
        assert!(result.contains(".map(|x|"));
        assert!(result.contains("x * 2"));
        assert!(result.contains(".collect::<Vec<_>>()"));

        // Comprehension with filter: [x for x in numbers if x > 0]
        let result = transpile_str("[x for x in numbers if x > 0]").unwrap();
        assert!(result.contains("numbers"));
        assert!(result.contains(".into_iter()"));
        assert!(result.contains(".filter(|x|"));
        assert!(result.contains("x > 0"));
        assert!(result.contains(".map(|x|"));
        assert!(result.contains(".collect::<Vec<_>>()"));

        // Complex comprehension: [name.len() for name in ["Alice", "Bob"] if name.len() > 3]
        let result =
            transpile_str("[name.len() for name in [\"Alice\", \"Bob\"] if name.len() > 3]")
                .unwrap();
        assert!(result.contains("vec![\"Alice\", \"Bob\"]"));
        assert!(result.contains(".filter(|name|"));
        assert!(result.contains("name.len() > 3"));
        assert!(result.contains(".map(|name|"));
        assert!(result.contains("name.len()"));
    }
}
