//! Error and option handling transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    fn transpile_error_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Throw { expr } => self.transpile_throw(expr),
            ExprKind::Ok { value } => self.transpile_result_ok(value),
            ExprKind::Err { error } => self.transpile_result_err(error),
            ExprKind::Some { value } => self.transpile_option_some(value),
            ExprKind::None => Ok(quote! { None }),
            ExprKind::Try { expr } => self.transpile_try_operator(expr),
            _ => unreachable!(),
        }
    }
    fn transpile_result_ok(&self, value: &Expr) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        // DEFECT-STRING-RESULT FIX: Convert string literals to String
        let final_tokens = match &value.kind {
            ExprKind::Literal(crate::frontend::ast::Literal::String(_)) => {
                quote! { #value_tokens.to_string() }
            }
            _ => value_tokens,
        };
        Ok(quote! { Ok(#final_tokens) })
    }
    fn transpile_result_err(&self, error: &Expr) -> Result<TokenStream> {
        let error_tokens = self.transpile_expr(error)?;
        // If error is a string literal, add .to_string() for String error types
        let final_tokens = match &error.kind {
            ExprKind::Literal(crate::frontend::ast::Literal::String(_)) => {
                quote! { #error_tokens.to_string() }
            }
            _ => error_tokens,
        };
        Ok(quote! { Err(#final_tokens) })
    }
    fn transpile_option_some(&self, value: &Expr) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        // DEFECT-STRING-RESULT FIX: Convert string literals to String
        let final_tokens = match &value.kind {
            ExprKind::Literal(crate::frontend::ast::Literal::String(_)) => {
                quote! { #value_tokens.to_string() }
            }
            _ => value_tokens,
        };
        Ok(quote! { Some(#final_tokens) })
    }
    fn transpile_try_operator(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { #expr_tokens? })
    }
    /// Transpile actor system expressions
    pub(super) fn transpile_actor_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Actor {
                name,
                state,
                handlers,
            } => self.transpile_actor(name, state, handlers),
            ExprKind::Effect { name, operations } => self.transpile_effect(name, operations),
            ExprKind::Send { actor, message } | ExprKind::ActorSend { actor, message } => {
                self.transpile_send(actor, message)
            }
            ExprKind::Ask {
                actor,
                message,
                timeout,
            } => self.transpile_ask(actor, message, timeout.as_deref()),
            ExprKind::ActorQuery { actor, message } => {
                // Actor query is like Ask without timeout
                self.transpile_ask(actor, message, None)
            }
            ExprKind::Command {
                program,
                args,
                env,
                working_dir,
            } => self.transpile_command(program, args, env, working_dir),
            _ => unreachable!("Non-actor expression in transpile_actor_expr"),
        }
    }
    /// Transpile miscellaneous expressions
    pub(super) fn transpile_misc_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Let {
                name,
                type_annotation,
                value,
                body,
                is_mutable,
                else_block,
            } => {
                if let Some(else_expr) = else_block {
                    // Transpile let-else: let PAT = EXPR else { BLOCK }
                    self.transpile_let_else(name, value, body, else_expr)
                } else {
                    // PARSER-073: Check for const attribute
                    let is_const = expr.attributes.iter().any(|attr| attr.name == "const");
                    self.transpile_let_with_type(
                        name,
                        type_annotation.as_ref(),
                        value,
                        body,
                        *is_mutable,
                        is_const,
                    )
                }
            }
            ExprKind::LetPattern {
                pattern,
                type_annotation,
                value,
                body,
                is_mutable: _,
                else_block,
            } => {
                if let Some(else_expr) = else_block {
                    // Transpile let-else pattern: let PAT = EXPR else { BLOCK }
                    self.transpile_let_pattern_else(pattern, value, body, else_expr)
                } else {
                    self.transpile_let_pattern_with_type(pattern, type_annotation.as_ref(), value, body)
                }
            }
            ExprKind::Block(exprs) => self.transpile_block(exprs),
            ExprKind::Pipeline { expr, stages } => self.transpile_pipeline(expr, stages),
            ExprKind::Import { module, items } => {
                // Check if this import has a "pub" attribute
                let has_pub = expr.attributes.iter().any(|attr| attr.name == "pub");
                let import_tokens = Self::transpile_import(module, items.as_deref());
                if has_pub {
                    // Add pub prefix to the use statement
                    Ok(quote! { pub #import_tokens })
                } else {
                    Ok(import_tokens)
                }
            }
            ExprKind::ImportAll { module, alias } => {
                // Check if this import has a "pub" attribute
                let has_pub = expr.attributes.iter().any(|attr| attr.name == "pub");
                let import_tokens = Self::transpile_import_all(module, alias);
                if has_pub {
                    Ok(quote! { pub #import_tokens })
                } else {
                    Ok(import_tokens)
                }
            }
            ExprKind::ImportDefault { module, name } => {
                Ok(Self::transpile_import_default(module, name))
            }
            ExprKind::ReExport { items, module } => Ok(Self::transpile_reexport(items, module)),
            ExprKind::Module { name, body } => self.transpile_module(name, body),
            ExprKind::Trait { .. }
            | ExprKind::Impl { .. }
            | ExprKind::Extension { .. }
            | ExprKind::Enum { .. }
            | ExprKind::TypeAlias { .. } => self.transpile_type_decl_expr(expr),
            ExprKind::Break { .. } | ExprKind::Continue { .. } | ExprKind::Return { .. } => {
                self.transpile_control_misc_expr(expr)
            }
            ExprKind::Export { expr, is_default } => Ok(Self::transpile_export(expr, *is_default)),
            ExprKind::ExportList { names } => Ok(Self::transpile_export_list(names)),
            ExprKind::ExportDefault { expr } => Ok(Self::transpile_export_default(expr)),
            _ => bail!("Unsupported expression kind: {:?}", expr.kind),
        }
    }
    pub(crate) fn transpile_type_decl_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Trait {
                name,
                type_params,
                associated_types,
                methods,
                is_pub,
            } => self.transpile_trait(name, type_params, associated_types, methods, *is_pub),
            ExprKind::Impl {
                type_params,
                trait_name,
                for_type,
                methods,
                is_pub,
            } => self.transpile_impl(
                for_type,
                type_params,
                trait_name.as_deref(),
                methods,
                *is_pub,
            ),
            ExprKind::Extension {
                target_type,
                methods,
            } => self.transpile_extend(target_type, methods),
            ExprKind::Enum {
                name,
                type_params,
                variants,
                is_pub,
            } => self.transpile_enum(name, type_params, variants, *is_pub),
            ExprKind::TypeAlias { name, target_type } => {
                let name_ident = format_ident!("{}", name);
                let type_tokens = self.transpile_type(target_type)?;
                Ok(quote! { type #name_ident = #type_tokens; })
            }
            _ => unreachable!(),
        }
    }

}
