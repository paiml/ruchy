//! Function-related transpilation (definitions, lambdas, calls)
//!
//! Each function maintains complexity ≤10 through decomposition

use super::super::Transpiler;
use crate::frontend::ast::{Expr, Param, Type, Attribute};
use anyhow::{Result, bail};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

impl Transpiler {
    /// Transpile function definition (complexity: 9)
    pub fn transpile_function(
        &self,
        name: &str,
        type_params: &[String],
        params: &[Param],
        body: &Expr,
        is_async: bool,
        return_type: Option<&Type>,
        is_pub: bool,
        attributes: &[Attribute],
    ) -> Result<TokenStream> {
        let fn_name = format_ident!("{}", name);
        let param_tokens = self.generate_param_tokens(params, body, name)?;
        let body_tokens = self.generate_body_tokens(body, is_async)?;
        
        let effective_return_type = self.determine_return_type(attributes, return_type);
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

    /// Transpile lambda expression (complexity: 8)
    pub fn transpile_lambda(&self, params: &[Param], body: &Expr) -> Result<TokenStream> {
        let param_tokens = self.generate_lambda_params(params)?;
        let body_tokens = self.transpile_expr(body)?;
        
        if params.is_empty() {
            Ok(quote! { || #body_tokens })
        } else {
            Ok(quote! { |#(#param_tokens),*| #body_tokens })
        }
    }

    /// Transpile function call (complexity: 9)
    pub fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        use crate::frontend::ast::ExprKind;
        
        match &func.kind {
            ExprKind::Identifier(name) => {
                self.transpile_identifier_call(name, args)
            }
            _ => {
                // General function call
                let func_tokens = self.transpile_expr(func)?;
                let arg_tokens = self.transpile_call_arguments(args)?;
                Ok(quote! { #func_tokens(#(#arg_tokens),*) })
            }
        }
    }

    /// Helper: Generate parameter tokens (complexity: 8)
    pub fn generate_param_tokens(
        &self,
        params: &[Param],
        body: &Expr,
        fn_name: &str,
    ) -> Result<Vec<TokenStream>> {
        params.iter().map(|param| {
            let name_ident = format_ident!("{}", param.name);
            
            if let Some(type_) = &param.type_ {
                let type_tokens = self.transpile_type(type_)?;
                Ok(quote! { #name_ident: #type_tokens })
            } else {
                let inferred = self.infer_param_type(&param.name, body, fn_name);
                Ok(quote! { #name_ident: #inferred })
            }
        }).collect()
    }

    /// Helper: Generate body tokens with async handling (complexity: 5)
    pub fn generate_body_tokens(&self, body: &Expr, is_async: bool) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;
        
        if is_async {
            Ok(quote! { async #body_tokens })
        } else {
            Ok(body_tokens)
        }
    }

    /// Helper: Determine effective return type (complexity: 4)
    fn determine_return_type<'a>(
        &self,
        attributes: &[Attribute],
        return_type: Option<&'a Type>,
    ) -> Option<&'a Type> {
        let has_test_attribute = attributes.iter().any(|attr| attr.name == "test");
        
        if has_test_attribute {
            None // Test functions have unit return type
        } else {
            return_type
        }
    }

    /// Helper: Generate return type tokens (complexity: 8)
    pub fn generate_return_type_tokens(
        &self,
        fn_name: &str,
        return_type: Option<&Type>,
        body: &Expr,
    ) -> Result<TokenStream> {
        if let Some(type_) = return_type {
            let type_tokens = self.transpile_type(type_)?;
            Ok(quote! { -> #type_tokens })
        } else if self.needs_result_type(fn_name, body) {
            Ok(quote! { -> Result<(), Box<dyn std::error::Error>> })
        } else {
            Ok(quote! {})
        }
    }

    /// Helper: Generate type parameter tokens (complexity: 4)
    pub fn generate_type_param_tokens(&self, type_params: &[String]) -> Result<TokenStream> {
        if type_params.is_empty() {
            Ok(quote! {})
        } else {
            let params: Vec<_> = type_params.iter()
                .map(|p| format_ident!("{}", p))
                .collect();
            Ok(quote! { <#(#params),*> })
        }
    }

    /// Helper: Generate complete function signature (complexity: 9)
    pub fn generate_function_signature(
        &self,
        is_pub: bool,
        is_async: bool,
        fn_name: &proc_macro2::Ident,
        type_params: &TokenStream,
        params: &[TokenStream],
        return_type: &TokenStream,
        body: &TokenStream,
        attributes: &[Attribute],
    ) -> Result<TokenStream> {
        let attr_tokens = self.generate_attribute_tokens(attributes);
        let visibility = if is_pub { quote! { pub } } else { quote! {} };
        let async_token = if is_async { quote! { async } } else { quote! {} };
        
        Ok(quote! {
            #(#attr_tokens)*
            #visibility #async_token fn #fn_name #type_params(#(#params),*) #return_type {
                #body
            }
        })
    }

    /// Helper: Generate attribute tokens (complexity: 3)
    fn generate_attribute_tokens(&self, attributes: &[Attribute]) -> Vec<TokenStream> {
        attributes.iter().map(|attr| {
            let name = format_ident!("{}", attr.name);
            quote! { #[#name] }
        }).collect()
    }

    /// Helper: Transpile identifier call with special cases (complexity: 8)
    fn transpile_identifier_call(&self, name: &str, args: &[Expr]) -> Result<TokenStream> {
        // Check for type conversion functions
        if self.is_type_conversion(name) {
            return self.transpile_type_conversion(name, args);
        }
        
        // Check for DataFrame operations
        if name == "DataFrame" || name == "df" {
            return self.transpile_dataframe_call(args);
        }
        
        // Regular function call
        let name_ident = format_ident!("{}", name);
        let arg_tokens = self.transpile_call_arguments(args)?;
        Ok(quote! { #name_ident(#(#arg_tokens),*) })
    }

    /// Helper: Generate lambda parameters (complexity: 5)
    fn generate_lambda_params(&self, params: &[Param]) -> Result<Vec<TokenStream>> {
        params.iter().map(|param| {
            let name_ident = format_ident!("{}", param.name);
            if param.type_.is_some() {
                // Typed parameter
                Ok(quote! { #name_ident })
            } else {
                // Untyped parameter
                Ok(quote! { #name_ident })
            }
        }).collect()
    }

    /// Helper: Transpile call arguments (complexity: 3)
    fn transpile_call_arguments(&self, args: &[Expr]) -> Result<Vec<TokenStream>> {
        args.iter().map(|arg| self.transpile_expr(arg)).collect()
    }

    /// Helper: Check if function is type conversion (complexity: 2)
    fn is_type_conversion(&self, name: &str) -> bool {
        matches!(name, "str" | "int" | "float" | "bool" | "list" | "set" | "dict")
    }

    /// Helper: Infer parameter type (complexity: 6)
    fn infer_param_type(&self, param_name: &str, body: &Expr, fn_name: &str) -> TokenStream {
        // Simple heuristic for now
        if fn_name == "main" {
            quote! { String }
        } else if self.looks_like_numeric_param(param_name, body) {
            quote! { impl Into<f64> }
        } else {
            quote! { impl std::fmt::Display }
        }
    }

    /// Helper: Check if parameter looks numeric (complexity: 5)
    fn looks_like_numeric_param(&self, param_name: &str, body: &Expr) -> bool {
        // Simple heuristic - could be enhanced
        use crate::frontend::ast::ExprKind;
        matches!(&body.kind, 
            ExprKind::Binary { op, .. } if Self::is_numeric_op(op)
        )
    }

    /// Helper: Check if needs Result return type (complexity: 4)
    fn needs_result_type(&self, fn_name: &str, body: &Expr) -> bool {
        fn_name == "main" && Self::contains_io_operations(body)
    }

    /// Helper: Check for I/O operations (complexity: 5)
    fn contains_io_operations(expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Call { func, .. } => {
                matches!(&func.kind, ExprKind::Identifier(name) if name.contains("print"))
            }
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_io_operations),
            _ => false
        }
    }
}