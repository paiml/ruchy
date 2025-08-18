//! Modular transpiler for Ruchy language
//!
//! This module is responsible for converting Ruchy AST into Rust code using `proc_macro2` `TokenStream`.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

mod actors;
mod dataframe;
mod expressions;
mod patterns;
mod statements;
mod types;

use crate::frontend::ast::{Attribute, Expr, ExprKind, Literal, Type};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

// Module exports are handled by the impl blocks in each module

/// The main transpiler struct
pub struct Transpiler {
    /// Track whether we're in an async context
    in_async_context: bool,
}

impl Transpiler {
    /// Creates a new transpiler instance
    pub fn new() -> Self {
        Self {
            in_async_context: false,
        }
    }

    /// Transpiles an expression to a `TokenStream`
    pub fn transpile(&self, expr: &Expr) -> Result<TokenStream> {
        self.transpile_expr(expr)
    }

    /// Transpiles an expression to a String
    pub fn transpile_to_string(&self, expr: &Expr) -> Result<String> {
        let tokens = self.transpile(expr)?;

        // Format the tokens with rustfmt-like style
        let mut result = String::new();
        let token_str = tokens.to_string();

        // Basic formatting: add newlines after semicolons and braces
        for ch in token_str.chars() {
            result.push(ch);
            if ch == ';' || ch == '{' {
                result.push('\n');
            }
        }

        Ok(result)
    }

    /// Main expression transpilation dispatcher
    ///
    /// # Panics
    ///
    /// Panics if label names cannot be parsed as valid Rust tokens
    pub fn transpile_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Self::transpile_literal(lit)),
            ExprKind::Identifier(name) => {
                let ident = format_ident!("{}", name);
                Ok(quote! { #ident })
            }
            ExprKind::QualifiedName { module, name } => {
                let module_ident = format_ident!("{}", module);
                let name_ident = format_ident!("{}", name);
                Ok(quote! { #module_ident::#name_ident })
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
            ExprKind::Let {
                name,
                value,
                body,
                is_mutable,
            } => self.transpile_let(name, value, body, *is_mutable),
            ExprKind::Function {
                name,
                type_params,
                params,
                body,
                is_async,
                return_type,
            } => self.transpile_function(
                name,
                type_params,
                params,
                body,
                *is_async,
                return_type.as_ref(),
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
            ExprKind::For { var, iter, body } => self.transpile_for(var, iter, body),
            ExprKind::While { condition, body } => self.transpile_while(condition, body),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.transpile_range(start, end, *inclusive),
            ExprKind::DataFrame { columns } => self.transpile_dataframe(columns),
            ExprKind::DataFrameOperation { source, operation } => {
                self.transpile_dataframe_operation(source, operation)
            }
            ExprKind::Import { path, items } => Ok(Self::transpile_import(path, items)),
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
            } => self.transpile_impl(for_type, type_params, trait_name.as_deref(), methods),
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
            ExprKind::Assign { target, value } => self.transpile_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.transpile_compound_assign(target, *op, value)
            }
            ExprKind::PreIncrement { target } => self.transpile_pre_increment(target),
            ExprKind::PostIncrement { target } => self.transpile_post_increment(target),
            ExprKind::PreDecrement { target } => self.transpile_pre_decrement(target),
            ExprKind::PostDecrement { target } => self.transpile_post_decrement(target),
            ExprKind::Module { name, body } => self.transpile_module(name, body),
            ExprKind::Export { items } => Ok(Self::transpile_export(items)),
            ExprKind::Break { label } => {
                if let Some(lbl) = label {
                    let label_name = format!("'{lbl}");
                    let tokens: proc_macro2::TokenStream = label_name
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Failed to parse label token: {}", e))?;
                    Ok(quote! { break #tokens })
                } else {
                    Ok(quote! { break })
                }
            }
            ExprKind::Continue { label } => {
                if let Some(lbl) = label {
                    let label_name = format!("'{lbl}");
                    let tokens: proc_macro2::TokenStream = label_name
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Failed to parse label token: {}", e))?;
                    Ok(quote! { continue #tokens })
                } else {
                    Ok(quote! { continue })
                }
            }
        }
    }

    /// Transpiles a literal value
    fn transpile_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::Integer(n) => {
                // Create a literal token with i64 suffix
                let lit_token = proc_macro2::Literal::i64_suffixed(*n);
                quote! { #lit_token }
            }
            Literal::Float(f) => {
                let f_str = f.to_string();
                if !f_str.contains('.') && !f_str.contains('e') && !f_str.contains('E') {
                    // Add .0 if it's a whole number without scientific notation
                    let f_with_decimal = format!("{f_str}.0");
                    quote! { #f_with_decimal }
                } else {
                    quote! { #f }
                }
            }
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Unit => quote! { () },
        }
    }
}

impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    fn transpile_str(input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        transpiler.transpile_to_string(&ast)
    }

    #[test]
    fn test_basic_transpilation() {
        let result = transpile_str("42").unwrap_or_else(|_| String::from("error"));
        assert!(result.contains("42"));
    }
}
