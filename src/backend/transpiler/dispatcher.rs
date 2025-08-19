//! Dispatcher functions to reduce complexity in transpiler
//!
//! This module contains delegated transpilation functions to keep
//! cyclomatic complexity below 10 for each function.

use super::Transpiler;
use crate::frontend::ast::{ExprKind, Expr};
use anyhow::{Result, bail};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpile basic expressions (literals, identifiers, strings)
    pub(super) fn transpile_basic_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Self::transpile_literal(lit)),
            ExprKind::Identifier(name) => Ok(Self::transpile_identifier(name)),
            ExprKind::QualifiedName { module, name } => Ok(Self::transpile_qualified_name(module, name)),
            ExprKind::StringInterpolation { parts } => self.transpile_string_interpolation(parts),
            _ => unreachable!("Non-basic expression in transpile_basic_expr"),
        }
    }

    fn transpile_identifier(name: &str) -> TokenStream {
        let ident = format_ident!("{}", name);
        quote! { #ident }
    }

    fn transpile_qualified_name(module: &str, name: &str) -> TokenStream {
        let module_ident = format_ident!("{}", module);
        let name_ident = format_ident!("{}", name);
        quote! { #module_ident::#name_ident }
    }

    /// Transpile operator and control flow expressions (split for complexity)
    pub(super) fn transpile_operator_control_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            // Operators
            ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::Try { .. } | ExprKind::Await { .. } =>
                self.transpile_operator_only_expr(expr),
            // Control flow
            ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::For { .. } | ExprKind::While { .. } =>
                self.transpile_control_flow_only_expr(expr),
            _ => unreachable!("Non-operator/control expression in transpile_operator_control_expr"),
        }
    }

    fn transpile_operator_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Binary { left, op, right } => self.transpile_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.transpile_unary(*op, operand),
            ExprKind::Try { expr } => self.transpile_try(expr),
            ExprKind::Await { expr } => self.transpile_await(expr),
            _ => unreachable!(),
        }
    }

    fn transpile_control_flow_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::If { condition, then_branch, else_branch } => 
                self.transpile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Match { expr, arms } => self.transpile_match(expr, arms),
            ExprKind::For { var, iter, body } => self.transpile_for(var, iter, body),
            ExprKind::While { condition, body } => self.transpile_while(condition, body),
            _ => unreachable!(),
        }
    }

    /// Transpile function-related expressions
    pub(super) fn transpile_function_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Function { name, type_params, params, body, is_async, return_type } =>
                self.transpile_function(name, type_params, params, body, *is_async, return_type.as_ref()),
            ExprKind::Lambda { params, body } => self.transpile_lambda(params, body),
            ExprKind::Call { func, args } => self.transpile_call(func, args),
            ExprKind::MethodCall { receiver, method, args } => 
                self.transpile_method_call(receiver, method, args),
            _ => unreachable!("Non-function expression in transpile_function_expr"),
        }
    }

    /// Transpile structure-related expressions
    pub(super) fn transpile_struct_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Struct { name, type_params, fields } => 
                self.transpile_struct(name, type_params, fields),
            ExprKind::StructLiteral { name, fields } => 
                self.transpile_struct_literal(name, fields),
            ExprKind::ObjectLiteral { fields } => 
                self.transpile_object_literal(fields),
            ExprKind::FieldAccess { object, field } => 
                self.transpile_field_access(object, field),
            _ => unreachable!("Non-struct expression in transpile_struct_expr"),
        }
    }

    /// Transpile data and error handling expressions (split for complexity)
    pub(super) fn transpile_data_error_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::DataFrame { .. } | ExprKind::DataFrameOperation { .. } | 
            ExprKind::List(_) | ExprKind::ListComprehension { .. } | ExprKind::Range { .. } => 
                self.transpile_data_only_expr(expr),
            ExprKind::TryCatch { .. } | ExprKind::Throw { .. } | ExprKind::Ok { .. } | ExprKind::Err { .. } => 
                self.transpile_error_only_expr(expr),
            _ => unreachable!("Non-data/error expression in transpile_data_error_expr"),
        }
    }

    fn transpile_data_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::DataFrame { columns } => self.transpile_dataframe(columns),
            ExprKind::DataFrameOperation { source, operation } => 
                self.transpile_dataframe_operation(source, operation),
            ExprKind::List(elements) => self.transpile_list(elements),
            ExprKind::ListComprehension { element, variable, iterable, condition } =>
                self.transpile_list_comprehension(element, variable, iterable, condition.as_deref()),
            ExprKind::Range { start, end, inclusive } => 
                self.transpile_range(start, end, *inclusive),
            _ => unreachable!(),
        }
    }

    fn transpile_error_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::TryCatch { try_block, catch_clauses, finally_block } => 
                self.transpile_try_catch(try_block, catch_clauses, finally_block.as_deref()),
            ExprKind::Throw { expr } => self.transpile_throw(expr),
            ExprKind::Ok { value } => self.transpile_result_ok(value),
            ExprKind::Err { error } => self.transpile_result_err(error),
            _ => unreachable!(),
        }
    }

    fn transpile_result_ok(&self, value: &Expr) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        Ok(quote! { Ok(#value_tokens) })
    }

    fn transpile_result_err(&self, error: &Expr) -> Result<TokenStream> {
        let error_tokens = self.transpile_expr(error)?;
        Ok(quote! { Err(#error_tokens) })
    }

    /// Transpile actor system expressions
    pub(super) fn transpile_actor_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Actor { name, state, handlers } => 
                self.transpile_actor(name, state, handlers),
            ExprKind::Send { actor, message } => 
                self.transpile_send(actor, message),
            ExprKind::Ask { actor, message, timeout } => 
                self.transpile_ask(actor, message, timeout.as_deref()),
            _ => unreachable!("Non-actor expression in transpile_actor_expr"),
        }
    }

    /// Transpile miscellaneous expressions
    pub(super) fn transpile_misc_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Let { name, value, body, is_mutable } => 
                self.transpile_let(name, value, body, *is_mutable),
            ExprKind::Block(exprs) => self.transpile_block(exprs),
            ExprKind::Pipeline { expr, stages } => self.transpile_pipeline(expr, stages),
            ExprKind::Import { path, items } => Ok(Self::transpile_import(path, items)),
            ExprKind::Trait { .. } | ExprKind::Impl { .. } => self.transpile_type_decl_expr(expr),
            ExprKind::Break { .. } | ExprKind::Continue { .. } | ExprKind::Export { .. } => 
                Self::transpile_control_misc_expr(expr),
            _ => bail!("Unsupported expression kind: {:?}", expr.kind),
        }
    }

    fn transpile_type_decl_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Trait { name, type_params, methods } => 
                self.transpile_trait(name, type_params, methods),
            ExprKind::Impl { type_params, trait_name, for_type, methods } => 
                self.transpile_impl(for_type, type_params, trait_name.as_deref(), methods),
            _ => unreachable!(),
        }
    }

    fn transpile_control_misc_expr(expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Break { label } => Ok(Self::make_break_continue(true, label.as_ref())),
            ExprKind::Continue { label } => Ok(Self::make_break_continue(false, label.as_ref())),
            ExprKind::Export { items } => {
                let item_idents: Vec<_> = items.iter().map(|item| format_ident!("{}", item)).collect();
                Ok(quote! { pub use { #(#item_idents),* }; })
            }
            _ => unreachable!(),
        }
    }

    fn make_break_continue(is_break: bool, label: Option<&String>) -> TokenStream {
        let keyword = if is_break { quote! { break } } else { quote! { continue } };
        match label {
            Some(l) => {
                let label_ident = format_ident!("{}", l);
                quote! { #keyword #label_ident }
            }
            None => keyword,
        }
    }
}