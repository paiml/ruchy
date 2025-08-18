//! Transpiler from Ruchy AST to Rust code

use crate::frontend::ast::{
    Attribute, BinaryOp, CatchClause, DataFrameColumn, DataFrameOp, Expr, ExprKind, Literal, MatchArm, Param,
    Pattern, PipelineStage, StringPart, Type, TypeKind, UnaryOp,
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
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn transpile(&self, expr: &Expr) -> Result<TokenStream> {
        self.transpile_expr(expr)
    }

    /// Transpile to a formatted Rust string
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be transpiled or parsed as valid Rust.
    /// # Errors
    ///
    /// Returns an error if the operation fails
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
    /// # Errors
    ///
    /// Returns an error if the operation fails
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
            ExprKind::QualifiedName { module, name } => {
                let module_tokens: TokenStream = module.parse().unwrap_or_else(|_| quote! { module });
                let name_tokens: TokenStream = name.parse().unwrap_or_else(|_| quote! { item });
                Ok(quote! { #module_tokens::#name_tokens })
            }
            ExprKind::StringInterpolation { parts } => self.transpile_string_interpolation(parts),
            ExprKind::Binary { left, op, right } => self.transpile_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.transpile_unary(*op, operand),
            ExprKind::Try { expr } => self.transpile_try(expr),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => self.transpile_try_catch(try_block, catch_clauses, finally_block.as_deref()),
            ExprKind::Throw { expr } => self.transpile_throw(expr),
            ExprKind::Ok { value } => {
                let value_tokens = self.transpile_expr(value)?;
                Ok(quote! { Ok(#value_tokens) })
            }
            ExprKind::Err { error } => {
                let error_tokens = self.transpile_expr(error)?;
                Ok(quote! { Err(#error_tokens) })
            }
            ExprKind::Await { expr } => self.transpile_await(expr),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.transpile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Let { name, value, body, is_mutable } => self.transpile_let(name, value, body, *is_mutable),
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
            ExprKind::DataFrame { columns } => self.transpile_dataframe(columns),
            ExprKind::DataFrameOperation { source, operation } => {
                self.transpile_dataframe_operation(source, operation)
            }
            ExprKind::For { var, iter, body } => self.transpile_for(var, iter, body),
            ExprKind::While { condition, body } => self.transpile_while(condition, body),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.transpile_range(start, end, *inclusive),
            ExprKind::Import { path, items } => Ok(Self::transpile_import(path, items)),
            ExprKind::Module { name, body } => self.transpile_module(name, body),
            ExprKind::Export { items } => Ok(Self::transpile_export(items)),
            ExprKind::Struct {
                name,
                type_params,
                fields,
            } => self.transpile_struct(name, type_params, fields),
            ExprKind::StructLiteral { name, fields } => self.transpile_struct_literal(name, fields),
            ExprKind::ObjectLiteral { fields } => self.transpile_object_literal(fields),
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
            ExprKind::Assign { target, value } => self.transpile_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.transpile_compound_assign(target, *op, value)
            }
            ExprKind::PreIncrement { target } => self.transpile_pre_increment(target),
            ExprKind::PostIncrement { target } => self.transpile_post_increment(target),
            ExprKind::PreDecrement { target } => self.transpile_pre_decrement(target),
            ExprKind::PostDecrement { target } => self.transpile_post_decrement(target),
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
    /// Transpile string interpolation to Rust format strings
    ///
    /// # Errors
    ///
    /// Returns an error if any expression inside the interpolation fails to transpile
    /// # Errors
    ///
    /// Returns an error if the operation fails
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
        catch_clauses: &[CatchClause],
        finally_block: Option<&Expr>,
    ) -> Result<TokenStream> {
        
        let try_tokens = self.transpile_expr(try_block)?;

        // Generate catch arms
        let mut catch_arms = Vec::new();
        for clause in catch_clauses {
            let var_ident = syn::Ident::new(&clause.variable, proc_macro2::Span::call_site());
            let body_tokens = self.transpile_expr(&clause.body)?;
            
            let pattern = if let Some(ref exc_type) = clause.exception_type {
                // Typed catch: match specific error types
                let type_ident = syn::Ident::new(exc_type, proc_macro2::Span::call_site());
                if let Some(ref condition) = clause.condition {
                    let cond_tokens = self.transpile_expr(condition)?;
                    quote! { Err(#var_ident) if #var_ident.is::<#type_ident>() && (#cond_tokens) => { #body_tokens } }
                } else {
                    quote! { Err(#var_ident) if #var_ident.is::<#type_ident>() => { #body_tokens } }
                }
            } else {
                // Catch-all
                if let Some(ref condition) = clause.condition {
                    let cond_tokens = self.transpile_expr(condition)?;
                    quote! { Err(#var_ident) if (#cond_tokens) => { #body_tokens } }
                } else {
                    quote! { Err(#var_ident) => { #body_tokens } }
                }
            };
            catch_arms.push(pattern);
        }

        // Add a default success arm
        let match_expr = if catch_arms.is_empty() {
            // No catch clauses, just try block
            try_tokens
        } else {
            quote! {
                match (|| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(#try_tokens)
                })() {
                    Ok(val) => val,
                    #(#catch_arms),*
                }
            }
        };

        // Wrap with finally block if present
        if let Some(finally) = finally_block {
            let finally_tokens = self.transpile_expr(finally)?;
            Ok(quote! {
                {
                    let _result = #match_expr;
                    #finally_tokens;
                    _result
                }
            })
        } else {
            Ok(match_expr)
        }
    }

    fn transpile_throw(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! {
            return Err(Box::new(#expr_tokens) as Box<dyn std::error::Error + Send + Sync>)
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

    fn transpile_let(&self, name: &str, value: &Expr, body: &Expr, is_mutable: bool) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        
        let let_keyword = if is_mutable {
            quote! { let mut }
        } else {
            quote! { let }
        };

        // Check if body is just Unit (meaning simple let statement, not let-in expression)
        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            // Simple let statement
            Ok(quote! {
                #let_keyword #name_ident = #value_tokens
            })
        } else {
            // Let-in expression
            let body_tokens = self.transpile_expr(body)?;
            Ok(quote! {
                {
                    #let_keyword #name_ident = #value_tokens;
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
                
                if p.is_mutable {
                    quote! { mut #param_name: #param_type }
                } else {
                    quote! { #param_name: #param_type }
                }
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
            // Empty block evaluates to unit
            return Ok(quote! { () });
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
                _ => bail!("Invalid pipeline stage: {:?}", stage.op.kind),
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
            Pattern::Ok(inner) => {
                let inner_pattern = self.transpile_pattern(inner)?;
                quote! { Ok(#inner_pattern) }
            }
            Pattern::Err(inner) => {
                let inner_pattern = self.transpile_pattern(inner)?;
                quote! { Err(#inner_pattern) }
            }
            Pattern::Tuple(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                quote! { (#(#pattern_tokens),*) }
            }
            Pattern::List(patterns) => {
                let pattern_tokens: Result<Vec<_>> = patterns.iter().map(|p| {
                    if matches!(p, Pattern::Rest) {
                        Ok(quote! { .. })
                    } else {
                        self.transpile_pattern(p)
                    }
                }).collect();
                let pattern_tokens = pattern_tokens?;
                quote! { [#(#pattern_tokens),*] }
            }
            Pattern::Struct { name, fields } => {
                let struct_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                let field_patterns: Result<Vec<_>> = fields.iter().map(|field| {
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    if let Some(pattern) = &field.pattern {
                        let pattern_tokens = self.transpile_pattern(pattern)?;
                        Ok(quote! { #field_name: #pattern_tokens })
                    } else {
                        // Shorthand syntax { x } instead of { x: x }
                        Ok(quote! { #field_name })
                    }
                }).collect();
                let field_patterns = field_patterns?;
                quote! { #struct_name { #(#field_patterns),* } }
            }
            Pattern::Range { start, end, inclusive } => {
                let start_pattern = self.transpile_pattern(start)?;
                let end_pattern = self.transpile_pattern(end)?;
                if *inclusive {
                    quote! { #start_pattern..=#end_pattern }
                } else {
                    quote! { #start_pattern..#end_pattern }
                }
            }
            Pattern::Or(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                quote! { #(#pattern_tokens)|* }
            }
            Pattern::Rest => {
                // Rest pattern should be handled by the parent context
                quote! { .. }
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

    fn transpile_dataframe(&self, columns: &[DataFrameColumn]) -> Result<TokenStream> {
        // Generate Polars DataFrame creation code
        // df! macro equivalent in Polars:
        // DataFrame::new(vec![
        //     Series::new("col1", vec![val1, val2, ...]),
        //     Series::new("col2", vec![val1, val2, ...]),
        // ])

        if columns.is_empty() {
            return Ok(quote! { DataFrame::empty() });
        }

        // Create Series for each column using new column structure
        let series_tokens: Result<Vec<TokenStream>> = columns
            .iter()
            .map(|col| {
                let col_name = &col.name;
                let values: Result<Vec<TokenStream>> =
                    col.values.iter().map(|v| self.transpile_expr(v)).collect();
                let values = values?;
                Ok(quote! {
                    Series::new(#col_name, vec![#(#values),*])
                })
            })
            .collect();
        let series_tokens = series_tokens?;

        Ok(quote! {
            DataFrame::new(vec![
                #(#series_tokens),*
            ]).expect("Failed to create DataFrame")
        })
    }

    fn transpile_dataframe_operation(
        &self,
        source: &Expr,
        operation: &DataFrameOp,
    ) -> Result<TokenStream> {
        let source_tokens = self.transpile_expr(source)?;

        match operation {
            DataFrameOp::Filter(condition) => {
                let cond_tokens = self.transpile_expr(condition)?;
                Ok(quote! {
                    #source_tokens.lazy().filter(#cond_tokens).collect()?
                })
            }
            DataFrameOp::Select(columns) => {
                let cols: Vec<_> = columns.iter().map(|c| quote! { #c }).collect();
                Ok(quote! {
                    #source_tokens.select(&[#(#cols),*])?
                })
            }
            DataFrameOp::GroupBy(columns) => {
                let cols: Vec<_> = columns.iter().map(|c| quote! { #c }).collect();
                Ok(quote! {
                    #source_tokens.groupby(&[#(#cols),*])?
                })
            }
            DataFrameOp::Sort(columns) => {
                let cols: Vec<_> = columns.iter().map(|c| quote! { #c }).collect();
                Ok(quote! {
                    #source_tokens.sort(&[#(#cols),*], false)?
                })
            }
            DataFrameOp::Limit(n) | DataFrameOp::Head(n) => Ok(quote! {
                #source_tokens.head(Some(#n))
            }),
            DataFrameOp::Tail(n) => Ok(quote! {
                #source_tokens.tail(Some(#n))
            }),
            _ => bail!("Unsupported DataFrame operation"),
        }
    }

    fn transpile_import(path: &str, items: &[crate::frontend::ast::ImportItem]) -> TokenStream {
        // Convert Ruchy import paths to Rust use statements
        // Replace dots with double colons for Rust-style paths
        let rust_path = path.replace('.', "::");
        
        // Build the path as a series of identifiers
        let path_segments: Vec<_> = rust_path
            .split("::")
            .filter(|s| !s.is_empty())
            .map(|segment| syn::Ident::new(segment, proc_macro2::Span::call_site()))
            .collect();

        // Check if this is a simple import where the last segment is the imported item
        let is_simple_import = items.len() == 1 
            && matches!(&items[0], crate::frontend::ast::ImportItem::Named(name) 
                if path_segments.last().map(std::string::ToString::to_string) == Some(name.clone()));

        if is_simple_import || items.is_empty() {
            // Simple import: import std::collections::HashMap -> use std::collections::HashMap;
            if path_segments.is_empty() {
                quote! {}
            } else {
                // Build path manually with :: separators
                let first = &path_segments[0];
                let rest = &path_segments[1..];
                quote! {
                    use #first #(::#rest)*;
                }
            }
        } else if items.len() == 1 && matches!(items[0], crate::frontend::ast::ImportItem::Wildcard) {
            // Wildcard import: import std::collections::* -> use std::collections::*;
            if path_segments.is_empty() {
                quote! { use *; }
            } else {
                let first = &path_segments[0];
                let rest = &path_segments[1..];
                quote! {
                    use #first #(::#rest)*::*;
                }
            }
        } else {
            // Import with items: import std::io::{Read, Write} -> use std::io::{Read, Write};
            let items_tokens: Vec<TokenStream> = items
                .iter()
                .map(|item| match item {
                    crate::frontend::ast::ImportItem::Named(name) => {
                        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                        quote! { #ident }
                    }
                    crate::frontend::ast::ImportItem::Aliased { name, alias } => {
                        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                        let alias_ident = syn::Ident::new(alias, proc_macro2::Span::call_site());
                        quote! { #name_ident as #alias_ident }
                    }
                    crate::frontend::ast::ImportItem::Wildcard => {
                        quote! { * }
                    }
                })
                .collect();
            
            if path_segments.is_empty() {
                quote! {
                    use {#(#items_tokens),*};
                }
            } else {
                let first = &path_segments[0];
                let rest = &path_segments[1..];
                quote! {
                    use #first #(::#rest)*::{#(#items_tokens),*};
                }
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

    fn transpile_object_literal(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<TokenStream> {
        use crate::frontend::ast::ObjectField;

        // Object literals in Ruchy translate to anonymous structs or HashMap in Rust
        // For now, we'll use HashMap for dynamic object literals
        let mut field_insertions = Vec::new();

        for field in fields {
            match field {
                ObjectField::KeyValue { key, value } => {
                    let value_tokens = self.transpile_expr(value)?;
                    field_insertions.push(quote! {
                        __object.insert(#key.to_string(), Box::new(#value_tokens) as Box<dyn std::any::Any>);
                    });
                }
                ObjectField::Spread { expr } => {
                    let expr_tokens = self.transpile_expr(expr)?;
                    // Spread operator merges another object into this one
                    field_insertions.push(quote! {
                        for (k, v) in #expr_tokens {
                            __object.insert(k, v);
                        }
                    });
                }
            }
        }

        Ok(quote! {
            {
                let mut __object = std::collections::HashMap::<String, Box<dyn std::any::Any>>::new();
                #(#field_insertions)*
                __object
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
                    if matches!(
                        first_param.name.as_str(),
                        "self" | "mut self" | "&self" | "&mut self"
                    ) {
                        // Method with self parameter
                        let self_tok = match first_param.name.as_str() {
                            "mut self" | "&mut self" => quote! { &mut self },
                            "self" | "&self" => quote! { &self }, // In traits, self defaults to &self
                            _ => {
                                // Fallback to type annotation
                                match &first_param.ty.kind {
                                    TypeKind::Named(name) if name == "&mut self" => {
                                        quote! { &mut self }
                                    }
                                    TypeKind::Named(name) if name == "self" => quote! { &self }, // In traits, self defaults to &self
                                    _ => quote! { &self }, // Default to immutable reference
                                }
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
                    if matches!(
                        first_param.name.as_str(),
                        "self" | "mut self" | "&self" | "&mut self"
                    ) {
                        // Method with self parameter
                        let self_tok = match first_param.name.as_str() {
                            "mut self" | "&mut self" => quote! { &mut self },
                            "self" | "&self" => quote! { &self }, // In impl blocks, self defaults to &self
                            _ => {
                                // Fallback to type annotation
                                match &first_param.ty.kind {
                                    TypeKind::Named(name) if name == "&mut self" => {
                                        quote! { &mut self }
                                    }
                                    TypeKind::Named(name) if name == "self" => quote! { &self }, // In impl blocks, self defaults to &self
                                    _ => quote! { &self }, // Default to immutable reference
                                }
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
        let message_enum_name =
            syn::Ident::new(&format!("{name}Message"), proc_macro2::Span::call_site());

        // Generate state struct fields
        let field_tokens: Result<Vec<_>> = state
            .iter()
            .map(|f| {
                let field_name = syn::Ident::new(&f.name, proc_macro2::Span::call_site());
                let field_type = self.transpile_type(&f.ty)?;
                Ok(quote! { pub #field_name: #field_type })
            })
            .collect();
        let field_tokens = field_tokens?;

        // Generate message enum variants
        let message_variants: Vec<_> = handlers
            .iter()
            .map(|h| {
                let variant = syn::Ident::new(&h.message_type, proc_macro2::Span::call_site());
                quote! { #variant }
            })
            .collect();

        // Generate handler match arms for message routing
        let handler_arms: Result<Vec<_>> = handlers
            .iter()
            .map(|h| {
                let _variant = syn::Ident::new(&h.message_type, proc_macro2::Span::call_site());
                let body_tokens = self.transpile_expr(&h.body)?;
                let enum_prefix = format!("{name}Message");
                let variant_path: TokenStream = format!("{}::{}", enum_prefix, h.message_type)
                    .parse()
                    .unwrap_or_else(|_| quote! { Message });

                Ok(quote! {
                    #variant_path => { #body_tokens; }
                })
            })
            .collect();
        let handler_arms = handler_arms?;

        // Generate field initializers
        let field_inits: Vec<_> = state
            .iter()
            .map(|f| {
                let field_name = syn::Ident::new(&f.name, proc_macro2::Span::call_site());
                quote! { #field_name: crate::frontend::ast::Span::default() }
            })
            .collect();

        // Generate the complete actor implementation
        Ok(quote! {
            use tokio::sync::mpsc;
            use anyhow::Result;

            // Message enum for this actor
            #[derive(Debug, Clone)]
            pub enum #message_enum_name {
                #(#message_variants),*
            }

            // Actor state struct
            #[derive(Debug, Clone, Default)]
            pub struct #actor_name {
                #(#field_tokens),*
            }

            impl #actor_name {
                #[must_use]
                pub fn new() -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }

                pub async fn handle_message(&mut self, msg: #message_enum_name) -> Result<()> {
                    match msg {
                        #(#handler_arms)*
                    }
                    Ok(())
                }

                pub fn spawn(mut self) -> mpsc::Sender<#message_enum_name> {
                    let (tx, mut rx) = mpsc::channel(100);
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if let Err(e) = self.handle_message(msg).await {
                                // Actor error: {e}
                            }
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

        // Generate send operation using the actor's message type directly
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
            // Generate ask with timeout using the actor's message type directly
            Ok(quote! {
                #actor_tokens.ask(
                    #message_tokens,
                    std::time::Duration::from_millis(#timeout_tokens as u64)
                ).await
            })
        } else {
            // Generate ask without timeout (default 5 seconds)
            Ok(quote! {
                #actor_tokens.ask(
                    #message_tokens,
                    std::time::Duration::from_secs(5)
                ).await
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
                        "f32" | "f64" => quote! { any::<f64>() },
                        "bool" => quote! { any::<bool>() },
                        "String" => quote! { any::<String>() },
                        _ => quote! { any::<i32>() }, // Default fallback for i32, i64, and unknown types
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
                                "f32" | "f64" => quote! { any::<f64>() },
                                "bool" => quote! { any::<bool>() },
                                "String" => quote! { any::<String>() },
                                _ => quote! { any::<i32>() }, // Default for i32, i64, and unknown types
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

    fn transpile_assign(&self, target: &Expr, value: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        
        Ok(quote! {
            #target_tokens = #value_tokens
        })
    }

    fn transpile_compound_assign(
        &self,
        target: &Expr,
        op: BinaryOp,
        value: &Expr,
    ) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        
        let op_tokens = match op {
            BinaryOp::Add => quote! { += },
            BinaryOp::Subtract => quote! { -= },
            BinaryOp::Multiply => quote! { *= },
            BinaryOp::Divide => quote! { /= },
            BinaryOp::Modulo => quote! { %= },
            BinaryOp::Power => {
                // **= doesn't exist in Rust, expand to target = target.pow(value)
                return Ok(quote! {
                    #target_tokens = (#target_tokens).pow(#value_tokens as u32)
                });
            }
            BinaryOp::BitwiseAnd => quote! { &= },
            BinaryOp::BitwiseOr => quote! { |= },
            BinaryOp::BitwiseXor => quote! { ^= },
            BinaryOp::LeftShift => quote! { <<= },
            BinaryOp::RightShift => quote! { >>= },
            _ => bail!("Unsupported compound assignment operator: {:?}", op),
        };
        
        Ok(quote! {
            #target_tokens #op_tokens #value_tokens
        })
    }

    fn transpile_pre_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            { #target_tokens += 1; #target_tokens }
        })
    }

    fn transpile_post_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            {
                let temp = #target_tokens;
                #target_tokens += 1;
                temp
            }
        })
    }

    fn transpile_pre_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            { #target_tokens -= 1; #target_tokens }
        })
    }

    fn transpile_post_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            {
                let temp = #target_tokens;
                #target_tokens -= 1;
                temp
            }
        })
    }

    fn transpile_module(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let module_name = syn::Ident::new(name, proc_macro2::Span::call_site());
        
        // Handle module body - if it's a block, expand the statements
        let body_tokens = match &body.kind {
            ExprKind::Block(exprs) => {
                let stmts: Result<Vec<_>> = exprs.iter().map(|e| self.transpile_expr(e)).collect();
                let stmts = stmts?;
                quote! { #(#stmts)* }
            }
            _ => self.transpile_expr(body)?
        };
        
        Ok(quote! {
            pub mod #module_name {
                #body_tokens
            }
        })
    }

    fn transpile_export(items: &[String]) -> TokenStream {
        let export_items: Vec<TokenStream> = items
            .iter()
            .map(|item| {
                let ident = syn::Ident::new(item, proc_macro2::Span::call_site());
                quote! { pub use #ident; }
            })
            .collect();
        
        quote! {
            #(#export_items)*
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::frontend::Parser;

    fn transpile_str(input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        let tokens = transpiler.transpile(&ast)?;
        Ok(tokens.to_string())
    }
    
    // Helper to normalize whitespace for test assertions
    fn normalize_ws(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }
    
    // Better helper: check if transpiled code contains expected tokens
    // regardless of spacing
    fn assert_transpiled_contains(input: &str, expected_tokens: &[&str]) {
        let result = transpile_str(input).expect("Transpilation failed");
        let normalized = normalize_ws(&result);
        
        for token in expected_tokens {
            let normalized_token = normalize_ws(token);
            assert!(
                normalized.contains(&normalized_token),
                "Transpiled output '{result}' does not contain expected token '{token}'"
            );
        }
    }
    
    // Helper to verify AST round-trip
    #[allow(dead_code)]
    fn assert_ast_roundtrip(input: &str) {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();
        let transpiler = Transpiler::new();
        let _tokens = transpiler.transpile(&ast).unwrap();
        // If we get here without panicking, the round-trip succeeded
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
        assert!(normalize_ws(&result).contains("x:i32"));
        assert!(normalize_ws(&result).contains("y:i32"));
        assert!(normalize_ws(&result).contains("->i32"));
    }

    #[test]
    fn test_transpile_list() {
        let result = transpile_str("[1, 2, 3]").unwrap();
        assert!(result.contains("vec") && result.contains('!'));
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
        assert_transpiled_contains("|x| x + 1", &["|x|", "x+1"]);

        // Lambda with multiple parameters
        assert_transpiled_contains("|x, y| x * y", &["|x,y|", "x*y"]);

        // Lambda with no parameters
        assert_transpiled_contains("|| 42", &["||", "42"]);

        // Lambda in a function call context
        assert_transpiled_contains("map(|x| x * 2)", &["map", "|x|", "x*2"]);
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
        assert!(normalize_ws(&result).contains(".mean("));
    }

    #[test]
    fn test_transpile_try_operator() {
        // Simple try
        let result = transpile_str("foo()?").unwrap();
        assert!(normalize_ws(&result).contains("foo()?"));

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
        assert!(normalize_ws(&result).contains("x:i32"));
        assert!(normalize_ws(&result).contains("y:i32"));

        // Struct with public fields
        let result = transpile_str("struct Person { pub name: String, pub age: i32 }").unwrap();
        assert!(result.contains("struct Person"));
        assert!(normalize_ws(&result).contains("pubname:String"));
        assert!(normalize_ws(&result).contains("pubage:i32"));
    }

    #[test]
    fn test_transpile_struct_literal() {
        // Simple struct instantiation
        let result = transpile_str("Point { x: 10, y: 20 }").unwrap();
        assert!(result.contains("Point"));
        assert!(normalize_ws(&result).contains("x:10"));
        assert!(normalize_ws(&result).contains("y:20"));

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
        assert!(normalize_ws(&result).contains("point.x"));

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
        assert!(normalize_ws(&result).contains("fncreate(name:String)->Self"));
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
        assert!(normalize_ws(&result).contains(".sum("));

        // reversed method
        let result = transpile_str("[1, 2, 3].reversed()").unwrap();
        assert!(result.contains("reverse()"));

        // unique method
        let result = transpile_str("[1, 2, 2, 3].unique()").unwrap();
        assert!(result.contains("HashSet"));

        // min/max methods
        let result = transpile_str("[1, 2, 3].min()").unwrap();
        assert!(normalize_ws(&result).contains(".min("));

        let result = transpile_str("[1, 2, 3].max()").unwrap();
        assert!(normalize_ws(&result).contains(".max("));
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
        assert!(normalize_ws(&result).contains(".send("));
        assert!(result.contains(".await"));
    }

    #[test]
    fn test_transpile_ask() {
        let code = "counter ? Get";
        let result = transpile_str(code).unwrap();
        assert!(normalize_ws(&result).contains(".ask("));
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
        assert!(result.contains("fetch") && result.contains(".await"));
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
        assert_transpiled_contains(
            "impl Point { fun distance(self) -> f64 { 0.0 } }",
            &["impl Point", "fn distance(&self) -> f64", "0"]
        );

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
        assert!(normalize_ws(&result).contains("fnnew(x:f64,y:f64)->Point"));
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
        assert!(normalize_ws(&result).contains("Result:{}"));
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
        assert!(normalize_ws(&result).contains("Value:{{static}}and{}"));
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
    fn test_transpile_assignment() {
        // Simple assignment
        let result = transpile_str("x = 42").unwrap();
        assert!(result.contains("x = 42"));

        // Assignment to field
        let result = transpile_str("obj.field = value").unwrap();
        assert!(result.contains("obj.field = value"));
    }

    #[test]
    fn test_transpile_compound_assignment() {
        // Addition assignment
        let result = transpile_str("x += 5").unwrap();
        assert!(result.contains("x += 5"));

        // Subtraction assignment
        let result = transpile_str("x -= 3").unwrap();
        assert!(result.contains("x -= 3"));

        // Multiplication assignment
        let result = transpile_str("x *= 2").unwrap();
        assert!(result.contains("x *= 2"));

        // Division assignment
        let result = transpile_str("x /= 4").unwrap();
        assert!(result.contains("x /= 4"));

        // Modulo assignment
        let result = transpile_str("x %= 3").unwrap();
        assert!(result.contains("x %= 3"));

        // Power assignment (should expand to x = x.pow(2))
        let result = transpile_str("x **= 2").unwrap();
        assert!(result.contains("x = (x).pow(2"));

        // Bitwise assignments
        let result = transpile_str("x &= mask").unwrap();
        assert!(result.contains("x &= mask"));

        let result = transpile_str("x |= flags").unwrap();
        assert!(result.contains("x |= flags"));

        let result = transpile_str("x ^= toggle").unwrap();
        assert!(result.contains("x ^= toggle"));

        let result = transpile_str("x <<= 2").unwrap();
        assert!(result.contains("x <<= 2"));

        let result = transpile_str("x >>= 1").unwrap();
        assert!(result.contains("x >>= 1"));
    }

    #[test]
    fn test_transpile_increment_decrement() {
        // Pre-increment
        let result = transpile_str("++x").unwrap();
        assert!(result.contains("x += 1"));

        // Post-increment
        let result = transpile_str("x++").unwrap();
        assert!(result.contains("let temp = x"));
        assert!(result.contains("x += 1"));
        assert!(result.contains("temp"));

        // Pre-decrement
        let result = transpile_str("--x").unwrap();
        assert!(result.contains("x -= 1"));

        // Post-decrement
        let result = transpile_str("x--").unwrap();
        assert!(result.contains("let temp = x"));
        assert!(result.contains("x -= 1"));
        assert!(result.contains("temp"));
    }

    #[test]
    fn test_transpile_mutable_let() {
        // Mutable variable
        let result = transpile_str("let mut x = 42 in x + 1").unwrap();
        assert!(result.contains("let mut x"));
        assert!(result.contains("42"));

        // Immutable variable (default)
        let result = transpile_str("let x = 42 in x + 1").unwrap();
        assert!(result.contains("let x"));
        assert!(!result.contains("let mut x"));
    }

    #[test]
    fn test_transpile_module_system() {
        // Simple module declaration
        let result = transpile_str("module utils { fun add(x: i32, y: i32) -> i32 { x + y } }").unwrap();
        assert!(result.contains("pub mod utils"));
        assert!(result.contains("fn add"));

        // Module with multiple items
        let result = transpile_str("module math { fun add(x: i32, y: i32) -> i32 { x + y }; fun sub(x: i32, y: i32) -> i32 { x - y } }").unwrap();
        assert!(result.contains("pub mod math"));
        assert!(result.contains("fn add"));
        assert!(result.contains("fn sub"));

        // Export statement
        let result = transpile_str("export { add, subtract }").unwrap();
        assert!(result.contains("pub use add"));
        assert!(result.contains("pub use subtract"));

        // Single export
        let result = transpile_str("export multiply").unwrap();
        assert!(result.contains("pub use multiply"));
    }

    #[test]
    fn test_transpile_import_system() {
        // Helper to normalize whitespace for comparison
        let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        
        // Simple import
        let result = transpile_str("import std::collections::HashMap").unwrap();
        assert!(normalize(&result).contains(&normalize("use std::collections::HashMap")));

        // Import with items
        let result = transpile_str("import std::collections::{HashMap, HashSet}").unwrap();
        assert!(normalize(&result).contains(&normalize("use std::collections::{HashMap, HashSet}")));

        // Import with alias
        let result = transpile_str("import std::collections::{HashMap as Map}").unwrap();
        let normalized_result = normalize(&result);
        let normalized_expected = normalize("use std::collections::{HashMap as Map}");
        assert!(normalized_result.contains(&normalized_expected), 
                "Failed: '{normalized_result}' does not contain '{normalized_expected}'");

        // Wildcard import
        let result = transpile_str("import std::collections::*").unwrap();
        assert!(normalize(&result).contains(&normalize("use std::collections::*")));

        // Mixed import with aliases
        let result = transpile_str("import std::io::{Read, Write as Writer}").unwrap();
        assert!(normalize(&result).contains(&normalize("use std::io::{Read, Write as Writer}")));
    }

    #[test]
    fn test_transpile_qualified_names() {
        // Qualified function call
        let result = transpile_str("math::add(1, 2)").unwrap();
        assert!(normalize_ws(&result).contains("math::add"));

        // Qualified constant access
        let result = transpile_str("constants::PI").unwrap();
        assert!(normalize_ws(&result).contains("constants::PI"));

        // Qualified struct instantiation
        let result = transpile_str("types::Point { x: 1, y: 2 }").unwrap();
        assert!(normalize_ws(&result).contains("types::Point"));
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

    #[test]
    fn test_transpile_advanced_patterns() {
        use crate::frontend::ast::StructPatternField;
        let transpiler = Transpiler { include_types: false };

        // Test tuple pattern
        let tuple_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
            Pattern::Literal(Literal::Integer(42)),
        ]);
        let result = transpiler.transpile_pattern(&tuple_pattern).unwrap();
        let result_str = format!("{result}");
        assert!(result_str.contains("(x , y"));
        assert!(result_str.contains("42"));

        // Test struct pattern
        let struct_pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                StructPatternField {
                    name: "x".to_string(),
                    pattern: Some(Pattern::Identifier("px".to_string())),
                },
                StructPatternField {
                    name: "y".to_string(),
                    pattern: None, // Shorthand
                },
            ],
        };
        let result = transpiler.transpile_pattern(&struct_pattern).unwrap();
        let expected = quote! { Point { x: px, y } };
        assert_eq!(format!("{result}"), format!("{expected}"));

        // Test range pattern
        let range_pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1))),
            end: Box::new(Pattern::Literal(Literal::Integer(10))),
            inclusive: true,
        };
        let result = transpiler.transpile_pattern(&range_pattern).unwrap();
        let result_str = format!("{result}");
        assert!(result_str.contains("..="));
        assert!(result_str.contains('1'));
        assert!(result_str.contains("10"));

        // Test OR pattern
        let or_pattern = Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Identifier("x".to_string()),
        ]);
        let result = transpiler.transpile_pattern(&or_pattern).unwrap();
        let result_str = format!("{result}");
        assert!(result_str.contains('|'));
        assert!(result_str.contains('1'));
        assert!(result_str.contains('2'));
        assert!(result_str.contains('x'));

        // Test list with rest pattern
        let list_with_rest = Pattern::List(vec![
            Pattern::Identifier("head".to_string()),
            Pattern::Rest,
            Pattern::Identifier("tail".to_string()),
        ]);
        let result = transpiler.transpile_pattern(&list_with_rest).unwrap();
        let result_str = format!("{result}");
        assert!(result_str.contains('['));
        assert!(result_str.contains("head"));
        assert!(result_str.contains(".."));
        assert!(result_str.contains("tail"));
        assert!(result_str.contains(']'));
    }

    #[test]
    fn test_transpile_complex_match() {
        let transpiler = Transpiler { include_types: false };

        let match_expr = Expr::new(
            ExprKind::Match {
                expr: Box::new(Expr::new(
                    ExprKind::Identifier("value".to_string()),
                    crate::frontend::ast::Span::default(),
                )),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Tuple(vec![
                            Pattern::Identifier("x".to_string()),
                            Pattern::Identifier("y".to_string()),
                        ]),
                        guard: None,
                        body: Box::new(Expr::new(
                            ExprKind::Binary {
                                left: Box::new(Expr::new(
                                    ExprKind::Identifier("x".to_string()),
                                    crate::frontend::ast::Span::default(),
                                )),
                                op: BinaryOp::Add,
                                right: Box::new(Expr::new(
                                    ExprKind::Identifier("y".to_string()),
                                    crate::frontend::ast::Span::default(),
                                )),
                            },
                            crate::frontend::ast::Span::default(),
                        )),
                        span: crate::frontend::ast::Span::default(),
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        guard: None,
                        body: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Integer(0)),
                            crate::frontend::ast::Span::default(),
                        )),
                        span: crate::frontend::ast::Span::default(),
                    },
                ],
            },
            crate::frontend::ast::Span::default(),
        );

        let result = transpiler.transpile_expr(&match_expr).unwrap();
        // The exact formatting might vary, so we just check that it compiles and contains expected parts
        let result_str = format!("{result}");
        assert!(result_str.contains("match"));
        assert!(result_str.contains("value"));
        assert!(result_str.contains("(x , y)"));
        assert!(result_str.contains("x + y"));
        assert!(result_str.contains('_'));
    }

    #[test]
    fn test_pattern_parsing_integration() {
        // Test that we can parse and transpile advanced pattern syntax
        
        // Basic pattern matching should work
        let result = transpile_str("match point { x => x }").unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("point"));
        
        // Wildcard pattern should work
        let result = transpile_str("match x { _ => 0 }").unwrap();
        assert!(result.contains('_'));
        
        // OR pattern (simple version)
        let result = transpile_str("match x { 1 => \"one\", 2 => \"two\" }").unwrap();
        assert!(result.contains("match"));
        assert!(result.contains('1'));
        assert!(result.contains('2'));
    }

    #[test]
    fn test_transpile_error_handling() {
        // Test basic try/catch
        let result = transpile_str("try { risky_operation() } catch e { handle_error(e) }").unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("risky_operation"));
        assert!(result.contains("Err(e)"));
        assert!(result.contains("handle_error"));

        // Test throw expression
        let result = transpile_str("throw RuntimeError(\"Something went wrong\")").unwrap();
        assert!(result.contains("return Err"));
        assert!(result.contains("RuntimeError"));
        assert!(result.contains("Something went wrong"));

        // Test try with finally
        let result = transpile_str("try { operation() } finally { cleanup() }").unwrap();
        assert!(result.contains("operation"));
        assert!(result.contains("cleanup"));
    }

    #[test]
    fn test_enhanced_error_messages() {
        use crate::frontend::error_recovery::ParseError;
        use crate::frontend::ast::Span;
        use crate::frontend::lexer::Token;

        // Test error creation with context
        let error = ParseError::unexpected_token(
            vec![Token::RightParen],
            Token::Semicolon,
            Span::new(10, 15)
        ).with_context("function parameter list".to_string())
         .with_hint("Try adding a closing parenthesis".to_string());

        let error_msg = format!("{error}");
        assert!(error_msg.contains("UnexpectedToken"));
        assert!(error_msg.contains("function parameter list"));
        assert!(error_msg.contains("closing parenthesis"));
        assert!(error_msg.contains("line 11")); // 1-based indexing

        // Test missing token error
        let error = ParseError::missing_token(Token::RightBrace, Span::new(20, 20))
            .with_context("struct definition".to_string());
        
        let error_msg = format!("{error}");
        assert!(error_msg.contains("MissingToken"));
        assert!(error_msg.contains("struct definition"));
        assert!(error_msg.contains("RightBrace"));
    }

    #[test]
    fn test_transpile_complex_try_catch() {
        let transpiler = Transpiler { include_types: false };

        // Create a complex try/catch AST
        let try_catch = Expr::new(
            ExprKind::TryCatch {
                try_block: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("risky_operation".to_string()),
                            crate::frontend::ast::Span::default(),
                        )),
                        args: vec![],
                    },
                    crate::frontend::ast::Span::default(),
                )),
                catch_clauses: vec![
                    CatchClause {
                        exception_type: Some("IOException".to_string()),
                        variable: "io_err".to_string(),
                        condition: None,
                        body: Box::new(Expr::new(
                            ExprKind::Literal(Literal::String("IO Error".to_string())),
                            crate::frontend::ast::Span::default(),
                        )),
                        span: crate::frontend::ast::Span::default(),
                    },
                    CatchClause {
                        exception_type: None,
                        variable: "general_err".to_string(),
                        condition: None,
                        body: Box::new(Expr::new(
                            ExprKind::Literal(Literal::String("General Error".to_string())),
                            crate::frontend::ast::Span::default(),
                        )),
                        span: crate::frontend::ast::Span::default(),
                    },
                ],
                finally_block: Some(Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("cleanup".to_string()),
                            crate::frontend::ast::Span::default(),
                        )),
                        args: vec![],
                    },
                    crate::frontend::ast::Span::default(),
                ))),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = transpiler.transpile_expr(&try_catch).unwrap();
        let result_str = format!("{result}");
        
        // Verify the transpiled code contains expected elements
        assert!(result_str.contains("risky_operation"));
        assert!(result_str.contains("cleanup"));
        assert!(result_str.contains("match"));
        assert!(result_str.contains("Result"));
    }
}
