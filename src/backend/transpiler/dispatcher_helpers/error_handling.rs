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
            // SPEC-001-J: Effect handler expression
            ExprKind::Handle { expr, handlers } => self.transpile_handler(expr, handlers),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    // Helper: Create test transpiler instance
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper: Create integer literal expression
    fn int_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create string literal expression
    fn string_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create identifier expression
    fn ident_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: transpile_result_ok - integer value
    #[test]
    fn test_transpile_result_ok_integer() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_ok(&int_expr(42));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Ok"));
        assert!(output.contains("42"));
    }

    // Test 2: transpile_result_ok - string literal (DEFECT-STRING-RESULT FIX)
    #[test]
    fn test_transpile_result_ok_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_ok(&string_expr("success"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Ok"));
        assert!(output.contains("to_string"));
    }

    // Test 3: transpile_result_err - integer value
    #[test]
    fn test_transpile_result_err_integer() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_err(&int_expr(404));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Err"));
        assert!(output.contains("404"));
    }

    // Test 4: transpile_result_err - string literal (auto-conversion)
    #[test]
    fn test_transpile_result_err_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_err(&string_expr("error"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Err"));
        assert!(output.contains("to_string"));
    }

    // Test 5: transpile_option_some - integer value
    #[test]
    fn test_transpile_option_some_integer() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_option_some(&int_expr(99));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Some"));
        assert!(output.contains("99"));
    }

    // Test 6: transpile_option_some - string literal (DEFECT-STRING-RESULT FIX)
    #[test]
    fn test_transpile_option_some_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_option_some(&string_expr("value"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Some"));
        assert!(output.contains("to_string"));
    }

    // Test 7: transpile_try_operator - basic try
    #[test]
    fn test_transpile_try_operator() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_try_operator(&ident_expr("fallible_func"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.to_string(), "fallible_func ?");
    }

    // Test 8: transpile_error_only_expr - None
    #[test]
    fn test_transpile_error_only_expr_none() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::None,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "None");
    }

    // Test 9: transpile_error_only_expr - Ok
    #[test]
    fn test_transpile_error_only_expr_ok() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Ok {
                value: Box::new(int_expr(1)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("Ok"));
    }

    // Test 10: transpile_error_only_expr - Err
    #[test]
    fn test_transpile_error_only_expr_err() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Err {
                error: Box::new(string_expr("fail")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("Err"));
    }

    // Test 11: transpile_error_only_expr - Some
    #[test]
    fn test_transpile_error_only_expr_some() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Some {
                value: Box::new(int_expr(5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("Some"));
    }

    // Test 12: transpile_error_only_expr - Try
    #[test]
    fn test_transpile_error_only_expr_try() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Try {
                expr: Box::new(ident_expr("operation")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("?"));
    }

    // Test 13: transpile_misc_expr - Block
    #[test]
    fn test_transpile_misc_expr_block() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Block(vec![int_expr(1), int_expr(2)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 14: transpile_type_decl_expr - TypeAlias
    #[test]
    fn test_transpile_type_decl_expr_type_alias() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::TypeAlias {
                name: "MyType".to_string(),
                target_type: "i32".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_type_decl_expr(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("type"));
        assert!(output.contains("MyType"));
    }

    // Test 15: transpile_result_ok - identifier (no string conversion)
    #[test]
    fn test_transpile_result_ok_identifier() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_ok(&ident_expr("value"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Ok"));
        assert!(output.contains("value"));
        assert!(!output.contains("to_string")); // No conversion for non-string-literals
    }

    // Test 16: transpile_result_err - identifier (no string conversion)
    #[test]
    fn test_transpile_result_err_identifier() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_err(&ident_expr("error"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Err"));
        assert!(output.contains("error"));
        assert!(!output.contains("to_string")); // No conversion for non-string-literals
    }

    // Test 17: transpile_option_some - identifier (no string conversion)
    #[test]
    fn test_transpile_option_some_identifier() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_option_some(&ident_expr("opt"));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Some"));
        assert!(output.contains("opt"));
        assert!(!output.contains("to_string")); // No conversion for non-string-literals
    }

    // Test 18: transpile_error_only_expr - Throw
    #[test]
    fn test_transpile_error_only_expr_throw() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Throw {
                expr: Box::new(string_expr("exception")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("panic"));
    }

    // Test 19: transpile_result_ok - empty string
    #[test]
    fn test_transpile_result_ok_empty_string() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_ok(&string_expr(""));
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let output = tokens.to_string();
        assert!(output.contains("Ok"));
        assert!(output.contains("to_string"));
    }

    // Test 20: transpile_try_operator - complex expression
    #[test]
    fn test_transpile_try_operator_complex() {
        let transpiler = test_transpiler();
        let complex_expr = Expr {
            kind: ExprKind::Call {
                func: Box::new(ident_expr("parse")),
                args: vec![string_expr("42")],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_try_operator(&complex_expr);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("?"));
    }
}
