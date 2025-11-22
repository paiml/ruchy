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
                    self.transpile_let_pattern_with_type(
                        pattern,
                        type_annotation.as_ref(),
                        value,
                        body,
                    )
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        assert_eq!(
            result.expect("result should be Ok in test").to_string(),
            "None"
        );
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
        let output = result.expect("result should be Ok in test").to_string();
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
        let output = result.expect("result should be Ok in test").to_string();
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
        let output = result.expect("result should be Ok in test").to_string();
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
        let output = result.expect("result should be Ok in test").to_string();
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
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("type"));
        assert!(output.contains("MyType"));
    }

    // Test 15: transpile_result_ok - identifier (no string conversion)
    #[test]
    fn test_transpile_result_ok_identifier() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_ok(&ident_expr("value"));
        assert!(result.is_ok());
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let tokens = result.expect("result should be Ok in test");
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
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("panic"));
    }

    // Test 19: transpile_result_ok - empty string
    #[test]
    fn test_transpile_result_ok_empty_string() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_result_ok(&string_expr(""));
        assert!(result.is_ok());
        let tokens = result.expect("result should be Ok in test");
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
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("?"));
    }

    // Test 21: transpile_misc_expr - Let with type annotation
    #[test]
    fn test_transpile_misc_expr_let_with_type() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Let {
                name: "x".to_string(),
                type_annotation: Some("i32".to_string()),
                value: Box::new(int_expr(10)),
                body: None,
                is_mutable: false,
                else_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("let"));
        assert!(output.contains("x"));
    }

    // Test 22: transpile_misc_expr - Let mutable
    #[test]
    fn test_transpile_misc_expr_let_mut() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Let {
                name: "counter".to_string(),
                type_annotation: None,
                value: Box::new(int_expr(0)),
                body: None,
                is_mutable: true,
                else_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("let"));
        assert!(output.contains("mut"));
    }

    // Test 23: transpile_misc_expr - Import
    #[test]
    fn test_transpile_misc_expr_import() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Import {
                module: "std::collections".to_string(),
                items: Some(vec!["HashMap".to_string(), "Vec".to_string()]),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("use"));
    }

    // Test 24: transpile_misc_expr - ImportAll
    #[test]
    fn test_transpile_misc_expr_import_all() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ImportAll {
                module: "std::prelude".to_string(),
                alias: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("use"));
        assert!(output.contains("*"));
    }

    // Test 25: transpile_misc_expr - Break (via transpile_control_misc_expr)
    #[test]
    fn test_transpile_misc_expr_break() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Break {
                label: None,
                value: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("result should be Ok in test").to_string(),
            "break"
        );
    }

    // Test 26: transpile_misc_expr - Continue (via transpile_control_misc_expr)
    #[test]
    fn test_transpile_misc_expr_continue() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Continue { label: None },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("result should be Ok in test").to_string(),
            "continue"
        );
    }

    // Test 27: transpile_misc_expr - Return (via transpile_control_misc_expr)
    #[test]
    fn test_transpile_misc_expr_return() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Return {
                value: Some(Box::new(int_expr(42))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("return"));
        assert!(output.contains("42"));
    }

    // Test 28: transpile_misc_expr - ImportDefault
    #[test]
    fn test_transpile_misc_expr_import_default() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ImportDefault {
                module: "utils".to_string(),
                name: "helper".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("use"));
    }

    // Test 29: transpile_error_only_expr - Throw with integer (panic)
    #[test]
    fn test_transpile_error_only_expr_throw_integer() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Throw {
                expr: Box::new(int_expr(500)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("panic"));
    }

    // Test 30: transpile_result_ok - nested result
    #[test]
    fn test_transpile_result_ok_nested() {
        let transpiler = test_transpiler();
        let nested = Expr {
            kind: ExprKind::Ok {
                value: Box::new(int_expr(1)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_result_ok(&nested);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("Ok"));
    }

    // Test 31: transpile_result_err - nested error
    #[test]
    fn test_transpile_result_err_nested() {
        let transpiler = test_transpiler();
        let nested = Expr {
            kind: ExprKind::Err {
                error: Box::new(int_expr(404)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_result_err(&nested);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("Err"));
    }

    // Test 32: transpile_option_some - nested option
    #[test]
    fn test_transpile_option_some_nested() {
        let transpiler = test_transpiler();
        let nested = Expr {
            kind: ExprKind::Some {
                value: Box::new(string_expr("nested")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_option_some(&nested);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("Some"));
    }

    // Test 33: transpile_misc_expr - Let with else block (let-else)
    #[test]
    fn test_transpile_misc_expr_let_else() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Let {
                name: "val".to_string(),
                type_annotation: None,
                value: Box::new(ident_expr("opt")),
                body: None,
                is_mutable: false,
                else_block: Some(Box::new(Expr {
                    kind: ExprKind::Return { value: None },
                    span: Span::default(),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                })),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("let"));
    }

    // Test 34: transpile_error_only_expr - Try with method call
    #[test]
    fn test_transpile_error_only_expr_try_method_call() {
        let transpiler = test_transpiler();
        let method_call = Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(ident_expr("file")),
                method: "read".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let expr = Expr {
            kind: ExprKind::Try {
                expr: Box::new(method_call),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("?"));
        assert!(output.contains("read"));
    }

    // Test 35: transpile_misc_expr - Block with multiple expressions
    #[test]
    fn test_transpile_misc_expr_block_multi() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Block(vec![
                int_expr(1),
                int_expr(2),
                int_expr(3),
                ident_expr("result"),
            ]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 36: transpile_misc_expr - Import with pub attribute
    #[test]
    fn test_transpile_misc_expr_import_pub() {
        use crate::frontend::ast::Attribute;
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Import {
                module: "std::io".to_string(),
                items: Some(vec!["Read".to_string()]),
            },
            span: Span::default(),
            attributes: vec![Attribute {
                name: "pub".to_string(),
                args: vec![],
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("pub"));
        assert!(output.contains("use"));
    }

    // Test 37: transpile_misc_expr - ImportAll with pub attribute
    #[test]
    fn test_transpile_misc_expr_import_all_pub() {
        use crate::frontend::ast::Attribute;
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ImportAll {
                module: "std::collections".to_string(),
                alias: None,
            },
            span: Span::default(),
            attributes: vec![Attribute {
                name: "pub".to_string(),
                args: vec![],
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("pub"));
        assert!(output.contains("*"));
    }

    // Test 38: transpile_misc_expr - ImportAll with alias
    #[test]
    fn test_transpile_misc_expr_import_all_alias() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ImportAll {
                module: "std::fs".to_string(),
                alias: Some("filesystem".to_string()),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("use"));
    }

    // Test 39: transpile_misc_expr - ReExport
    #[test]
    fn test_transpile_misc_expr_reexport() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ReExport {
                items: vec!["Foo".to_string(), "Bar".to_string()],
                module: Some("internal".to_string()),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("pub"));
        assert!(output.contains("use"));
    }

    // Test 40: transpile_misc_expr - Module
    #[test]
    fn test_transpile_misc_expr_module() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Module {
                name: "utils".to_string(),
                body: vec![int_expr(1)],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("mod"));
        assert!(output.contains("utils"));
    }

    // Test 41: transpile_misc_expr - Trait (via transpile_type_decl_expr)
    #[test]
    fn test_transpile_misc_expr_trait() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Trait {
                name: "MyTrait".to_string(),
                type_params: vec![],
                associated_types: vec![],
                methods: vec![],
                is_pub: true,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("trait"));
    }

    // Test 42: transpile_misc_expr - Impl (via transpile_type_decl_expr)
    #[test]
    fn test_transpile_misc_expr_impl() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Impl {
                type_params: vec![],
                trait_name: None,
                for_type: "MyType".to_string(),
                methods: vec![],
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("impl"));
    }

    // Test 43: transpile_misc_expr - Extension (via transpile_type_decl_expr)
    #[test]
    fn test_transpile_misc_expr_extension() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Extension {
                target_type: "String".to_string(),
                methods: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("impl"));
    }

    // Test 44: transpile_misc_expr - Enum (via transpile_type_decl_expr)
    #[test]
    fn test_transpile_misc_expr_enum() {
        use crate::frontend::ast::EnumVariant;
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Enum {
                name: "Color".to_string(),
                type_params: vec![],
                variants: vec![
                    EnumVariant {
                        name: "Red".to_string(),
                        fields: None,
                    },
                    EnumVariant {
                        name: "Blue".to_string(),
                        fields: None,
                    },
                ],
                is_pub: true,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("enum"));
        assert!(output.contains("Color"));
    }

    // Test 45: transpile_misc_expr - Export
    #[test]
    fn test_transpile_misc_expr_export() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Export {
                expr: Box::new(ident_expr("my_func")),
                is_default: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("pub"));
    }

    // Test 46: transpile_misc_expr - ExportList
    #[test]
    fn test_transpile_misc_expr_export_list() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ExportList {
                names: vec!["foo".to_string(), "bar".to_string()],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("pub"));
    }

    // Test 47: transpile_misc_expr - ExportDefault
    #[test]
    fn test_transpile_misc_expr_export_default() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ExportDefault {
                expr: Box::new(ident_expr("main_func")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_misc_expr(&expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("pub"));
    }

    // Test 48: transpile_result_ok - boolean literal
    #[test]
    fn test_transpile_result_ok_boolean() {
        let transpiler = test_transpiler();
        let bool_expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_result_ok(&bool_expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("Ok"));
        assert!(output.contains("true"));
    }

    // Test 49: transpile_result_err - boolean literal
    #[test]
    fn test_transpile_result_err_boolean() {
        let transpiler = test_transpiler();
        let bool_expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(false)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_result_err(&bool_expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("Err"));
        assert!(output.contains("false"));
    }

    // Test 50: transpile_option_some - boolean literal
    #[test]
    fn test_transpile_option_some_boolean() {
        let transpiler = test_transpiler();
        let bool_expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_option_some(&bool_expr);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("Some"));
        assert!(output.contains("true"));
    }
}
