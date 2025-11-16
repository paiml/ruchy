#![allow(clippy::approx_constant)]
//! Dispatcher functions to reduce complexity in transpiler
//!
//! This module contains delegated transpilation functions to keep
//! cyclomatic complexity below 10 for each function.

#[path = "dispatcher_helpers/macro_helpers.rs"]
mod macro_helpers;

#[path = "dispatcher_helpers/identifiers.rs"]
mod identifiers;

#[path = "dispatcher_helpers/misc.rs"]
mod misc;

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
impl Transpiler {
    /// Transpile basic expressions (literals, identifiers, strings)
    pub(super) fn transpile_basic_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Self::transpile_literal(lit)),
            ExprKind::Identifier(name) => Ok(self.transpile_identifier(name)),
            ExprKind::QualifiedName { module, name } => {
                Ok(Self::transpile_qualified_name(module, name))
            }
            ExprKind::StringInterpolation { parts } => self.transpile_string_interpolation(parts),
            ExprKind::TypeCast { expr, target_type } => self.transpile_type_cast(expr, target_type),
            _ => unreachable!("Non-basic expression in transpile_basic_expr"),
        }
    }

    /// Transpile operator and control flow expressions (split for complexity)
    pub(super) fn transpile_operator_control_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            // Operators
            ExprKind::Binary { .. }
            | ExprKind::Unary { .. }
            | ExprKind::Assign { .. }
            | ExprKind::CompoundAssign { .. }
            | ExprKind::PreIncrement { .. }
            | ExprKind::PostIncrement { .. }
            | ExprKind::PreDecrement { .. }
            | ExprKind::PostDecrement { .. }
            | ExprKind::Await { .. }
            | ExprKind::Spawn { .. }
            | ExprKind::AsyncBlock { .. }
            | ExprKind::AsyncLambda { .. } => self.transpile_operator_only_expr(expr),
            // Control flow
            ExprKind::If { .. }
            | ExprKind::IfLet { .. }
            | ExprKind::WhileLet { .. }
            | ExprKind::Match { .. }
            | ExprKind::For { .. }
            | ExprKind::While { .. }
            | ExprKind::Loop { .. }
            | ExprKind::TryCatch { .. } => self.transpile_control_flow_only_expr(expr),
            _ => unreachable!("Non-operator/control expression in transpile_operator_control_expr"),
        }
    }
    fn transpile_operator_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Binary { left, op, right } => self.transpile_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.transpile_unary(*op, operand),
            ExprKind::Assign { target, value } => self.transpile_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.transpile_compound_assign(target, *op, value)
            }
            ExprKind::PreIncrement { target } => self.transpile_pre_increment(target),
            ExprKind::PostIncrement { target } => self.transpile_post_increment(target),
            ExprKind::PreDecrement { target } => self.transpile_pre_decrement(target),
            ExprKind::PostDecrement { target } => self.transpile_post_decrement(target),
            ExprKind::Await { expr } => self.transpile_await(expr),
            ExprKind::Spawn { actor } => self.transpile_spawn(actor),
            ExprKind::AsyncBlock { body } => self.transpile_async_block(body),
            ExprKind::AsyncLambda { params, body } => self.transpile_async_lambda(params, body),
            _ => unreachable!(),
        }
    }
    fn transpile_control_flow_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.transpile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Match { expr, arms } => self.transpile_match(expr, arms),
            ExprKind::For {
                var,
                pattern,
                iter,
                body,
                ..
            } => self.transpile_for(var, pattern.as_ref(), iter, body),
            ExprKind::While {
                condition, body, ..
            } => self.transpile_while(condition, body),
            ExprKind::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
            } => self.transpile_if_let(pattern, expr, then_branch, else_branch.as_deref()),
            ExprKind::WhileLet {
                pattern,
                expr,
                body,
                ..
            } => self.transpile_while_let(pattern, expr, body),
            ExprKind::Loop { body, .. } => self.transpile_loop(body),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => self.transpile_try_catch(try_block, catch_clauses, finally_block.as_deref()),
            _ => unreachable!(),
        }
    }
    /// Transpile function-related expressions
    pub(super) fn transpile_function_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Function {
                name,
                type_params,
                params,
                body,
                is_async,
                return_type,
                is_pub,
            } => self.transpile_function(
                name,
                type_params,
                params,
                body,
                *is_async,
                return_type.as_ref(),
                *is_pub,
                &expr.attributes,
            ),
            ExprKind::Lambda { params, body } => self.transpile_lambda(params, body),
            ExprKind::Call { func, args } => self.transpile_call(func, args),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.transpile_method_call(receiver, method, args),
            ExprKind::Macro { name, args } => self.transpile_macro(name, args),
            _ => unreachable!("Non-function expression in transpile_function_expr"),
        }
    }
    /// Transpile macro expressions with clean dispatch pattern
    ///
    /// This function uses specialized handlers for different macro categories:
    /// - Print macros: `println!`, `print!`, `panic!` (string formatting)
    /// - Collection macros: `vec!` (simple element transpilation)
    /// - Assertion macros: `assert!`, `assert_eq!`, `assert_ne!` (validation + transpilation)
    ///
    /// # Example Usage
    /// This method dispatches to specific macro handlers based on the macro name.
    /// For example, `println` calls `transpile_println_macro`, `vec` calls `transpile_vec_macro`, etc.
    pub(super) fn transpile_macro(&self, name: &str, args: &[Expr]) -> Result<TokenStream> {
        match name {
            // Print macros (string formatting)
            "println" => self.transpile_println_macro(args),
            "print" => self.transpile_print_macro(args),
            "panic" => self.transpile_panic_macro(args),
            // Collection macros (simple transpilation)
            "vec" => self.transpile_vec_macro(args),
            // Assertion macros (validation + transpilation)
            "assert" => self.transpile_assert_macro(args),
            "assert_eq" => self.transpile_assert_eq_macro(args),
            "assert_ne" => self.transpile_assert_ne_macro(args),
            // External macros (pass through)
            "json" | "sql" | "format" | "dbg" | "include_str" | "include_bytes" | "todo"
            | "unimplemented" | "unreachable" | "compile_error" | "concat" | "env"
            | "option_env" | "cfg" | "column" | "file" | "line" | "module_path" | "stringify"
            | "write" | "writeln" | "eprintln" | "eprint" => {
                self.transpile_passthrough_macro(name, args)
            }
            _ => bail!("Unknown macro: {name}"),
        }
    }
    /// Transpile structure-related expressions
    pub(super) fn transpile_struct_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Struct {
                name,
                type_params,
                fields,
                methods,
                derives,
                is_pub,
            } => self.transpile_struct_with_methods(name, type_params, fields, methods, derives, *is_pub),
            ExprKind::TupleStruct {
                name,
                type_params,
                fields,
                derives,
                is_pub,
            } => self.transpile_tuple_struct(name, type_params, fields, derives, *is_pub),
            ExprKind::Class {
                name,
                type_params,
                superclass: _, // Inheritance not yet transpiled
                traits,
                fields,
                constructors,
                methods,
                constants,
                properties: _, // Properties not yet transpiled
                derives,
                is_pub,
                is_sealed: _,   // Sealed classes not yet transpiled
                is_abstract: _, // Abstract classes not yet transpiled
                decorators: _,  // Decorators not yet transpiled
            } => self.transpile_class(
                name,
                type_params,
                traits,
                fields,
                constructors,
                methods,
                constants,
                derives,
                *is_pub,
            ),
            ExprKind::StructLiteral { name, fields, base } => {
                self.transpile_struct_literal(name, fields, base.as_deref())
            }
            ExprKind::ObjectLiteral { fields } => self.transpile_object_literal(fields),
            ExprKind::FieldAccess { object, field } => self.transpile_field_access(object, field),
            ExprKind::IndexAccess { object, index } => self.transpile_index_access(object, index),
            ExprKind::Slice { object, start, end } => {
                self.transpile_slice(object, start.as_deref(), end.as_deref())
            }
            _ => unreachable!("Non-struct expression in transpile_struct_expr"),
        }
    }
    /// Transpile data and error handling expressions (split for complexity)
    pub(super) fn transpile_data_error_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::DataFrame { .. }
            | ExprKind::DataFrameOperation { .. }
            | ExprKind::List(_)
            | ExprKind::Set(_)
            | ExprKind::ArrayInit { .. }
            | ExprKind::Tuple(_)
            | ExprKind::ListComprehension { .. }
            | ExprKind::SetComprehension { .. }
            | ExprKind::DictComprehension { .. }
            | ExprKind::Range { .. } => self.transpile_data_only_expr(expr),
            ExprKind::Throw { .. }
            | ExprKind::Ok { .. }
            | ExprKind::Err { .. }
            | ExprKind::Some { .. }
            | ExprKind::None
            | ExprKind::Try { .. } => self.transpile_error_only_expr(expr),
            _ => unreachable!("Non-data/error expression in transpile_data_error_expr"),
        }
    }
    fn transpile_data_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::DataFrame { columns } => self.transpile_dataframe(columns),
            ExprKind::DataFrameOperation { source, operation } => {
                self.transpile_dataframe_operation(source, operation)
            }
            ExprKind::List(elements) => self.transpile_list(elements),
            ExprKind::Set(elements) => {
                // EMERGENCY FIX: Check if this Set is actually a misparsed Block
                if elements.len() == 1 && !self.looks_like_real_set(&elements[0]) {
                    eprintln!("DEBUG: Set detected as misparsed Block, transpiling as expression");
                    // Single expression that doesn't look like a real set element - treat as block expression
                    self.transpile_expr(&elements[0])
                } else {
                    self.transpile_set(elements)
                }
            }
            ExprKind::ArrayInit { value, size } => self.transpile_array_init(value, size),
            ExprKind::Tuple(elements) => self.transpile_tuple(elements),
            ExprKind::ListComprehension { element, clauses } => {
                self.transpile_list_comprehension_new(element, clauses)
            }
            ExprKind::SetComprehension { element, clauses } => {
                self.transpile_set_comprehension_new(element, clauses)
            }
            ExprKind::DictComprehension {
                key,
                value,
                clauses,
            } => self.transpile_dict_comprehension_new(key, value, clauses),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.transpile_range(start, end, *inclusive),
            _ => unreachable!(),
        }
    }
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
            // SPEC-001-I: Effect declaration transpiles to Rust trait
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
                    // Becomes: let name = if let PAT = EXPR { name } else { BLOCK };
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
            ExprKind::Lazy { expr } => {
                // SPEC-001-D: Lazy evaluation transpiles to immediate evaluation
                // For simplicity, just evaluate the expression immediately
                // Future: Use std::sync::LazyLock or once_cell::Lazy for true lazy semantics
                self.transpile_expr(expr)
            }
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
            ExprKind::ModuleDeclaration { name } => Ok(self.transpile_external_mod_declaration(name, expr)),
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
            // ISSUE-103: Handle MacroInvocation for compilation support
            ExprKind::MacroInvocation { name, args } => self.transpile_macro(name, args),
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

    // Test 1: transpile_basic_expr with Literal routes correctly
    #[test]
    fn test_transpile_basic_expr_literal() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("42"));
    }

    // Test 2: transpile_basic_expr with Identifier routes correctly
    #[test]
    fn test_transpile_basic_expr_identifier() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Identifier("my_var".to_string()),
            Span::default(),
        );
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("my_var"));
    }

    // Test 3: transpile_basic_expr with QualifiedName routes correctly
    #[test]
    fn test_transpile_basic_expr_qualified_name() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::QualifiedName {
                module: "std::collections".to_string(),
                name: "HashMap".to_string(),
            },
            Span::default(),
        );
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("HashMap"));
    }

    // Test 4: transpile_basic_expr with StringInterpolation routes correctly
    #[test]
    fn test_transpile_basic_expr_string_interpolation() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::StringInterpolation { parts: vec![] },
            Span::default(),
        );
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 5: transpile_macro with unknown macro (ERROR PATH)
    #[test]
    fn test_transpile_macro_unknown_error() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_macro("unknown_macro", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown macro"));
    }

    // Test 6: transpile_macro with println routes correctly
    #[test]
    fn test_transpile_macro_println() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_macro("println", &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("println"));
    }

    // Test 7: transpile_macro with vec routes correctly
    #[test]
    fn test_transpile_macro_vec() {
        let transpiler = Transpiler::new();
        let elem = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let result = transpiler.transpile_macro("vec", &[elem]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("vec"));
    }

    // Test 8: transpile_macro with assert routes correctly
    #[test]
    fn test_transpile_macro_assert() {
        let transpiler = Transpiler::new();
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::default(),
        );
        let result = transpiler.transpile_macro("assert", &[condition]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("assert"));
    }

    // Test 9: transpile_macro with passthrough macro (json) routes correctly
    #[test]
    fn test_transpile_macro_passthrough_json() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_macro("json", &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("json"));
    }

    // Test 10: transpile_result_ok with integer literal
    #[test]
    fn test_transpile_result_ok_integer() {
        let transpiler = Transpiler::new();
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.transpile_result_ok(&value);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Ok"));
        assert!(tokens.contains("42"));
    }

    // Test 11: transpile_result_ok with string literal (auto-conversion to String)
    #[test]
    fn test_transpile_result_ok_string_conversion() {
        let transpiler = Transpiler::new();
        let value = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_result_ok(&value);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Ok"));
        assert!(tokens.contains("to_string"));
    }

    // Test 12: transpile_result_err with integer literal
    #[test]
    fn test_transpile_result_err_integer() {
        let transpiler = Transpiler::new();
        let error = Expr::new(
            ExprKind::Literal(Literal::Integer(404, None)),
            Span::default(),
        );
        let result = transpiler.transpile_result_err(&error);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Err"));
        assert!(tokens.contains("404"));
    }

    // Test 13: transpile_result_err with string literal (auto-conversion to String)
    #[test]
    fn test_transpile_result_err_string_conversion() {
        let transpiler = Transpiler::new();
        let error = Expr::new(
            ExprKind::Literal(Literal::String("error message".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_result_err(&error);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Err"));
        assert!(tokens.contains("to_string"));
    }

    // Test 14: transpile_option_some with integer literal
    #[test]
    fn test_transpile_option_some_integer() {
        let transpiler = Transpiler::new();
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(123, None)),
            Span::default(),
        );
        let result = transpiler.transpile_option_some(&value);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Some"));
        assert!(tokens.contains("123"));
    }

    // Test 15: transpile_option_some with string literal (auto-conversion to String)
    #[test]
    fn test_transpile_option_some_string_conversion() {
        let transpiler = Transpiler::new();
        let value = Expr::new(
            ExprKind::Literal(Literal::String("value".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_option_some(&value);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Some"));
        assert!(tokens.contains("to_string"));
    }

    // Test 16: transpile_try_operator
    #[test]
    fn test_transpile_try_operator() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Identifier("result_value".to_string()),
            Span::default(),
        );
        let result = transpiler.transpile_try_operator(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("result_value"));
        assert!(tokens.contains("?"));
    }

    // Test 17: transpile_error_only_expr with None
    #[test]
    fn test_transpile_error_only_expr_none() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::None, Span::default());
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("None"));
    }

    // Test 18: transpile_error_only_expr with Ok routes correctly
    #[test]
    fn test_transpile_error_only_expr_ok() {
        let transpiler = Transpiler::new();
        let value = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        ));
        let expr = Expr::new(ExprKind::Ok { value }, Span::default());
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Ok"));
    }

    // Test 19: transpile_error_only_expr with Err routes correctly
    #[test]
    fn test_transpile_error_only_expr_err() {
        let transpiler = Transpiler::new();
        let error = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        ));
        let expr = Expr::new(ExprKind::Err { error }, Span::default());
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Err"));
    }

    // Test 20: transpile_error_only_expr with Some routes correctly
    #[test]
    fn test_transpile_error_only_expr_some() {
        let transpiler = Transpiler::new();
        let value = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        ));
        let expr = Expr::new(ExprKind::Some { value }, Span::default());
        let result = transpiler.transpile_error_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("Some"));
    }

    // Test 21: transpile_operator_control_expr routes Binary to operator handler
    #[test]
    fn test_transpile_operator_control_expr_binary() {
        use crate::frontend::ast::BinaryOp;
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::default())),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(2, None)), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_control_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 22: transpile_operator_control_expr routes If to control flow handler
    #[test]
    fn test_transpile_operator_control_expr_if() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default())),
                then_branch: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::default())),
                else_branch: None,
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_control_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 23: transpile_operator_control_expr routes Await to operator handler
    #[test]
    fn test_transpile_operator_control_expr_await() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Await {
                expr: Box::new(Expr::new(ExprKind::Identifier("future".to_string()), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_control_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 24: transpile_operator_only_expr handles Binary
    #[test]
    fn test_transpile_operator_only_expr_binary() {
        use crate::frontend::ast::BinaryOp;
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(5, None)), Span::default())),
                op: BinaryOp::Multiply,
                right: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(3, None)), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("5") && tokens.contains("3"));
    }

    // Test 25: transpile_operator_only_expr handles Unary
    #[test]
    fn test_transpile_operator_only_expr_unary() {
        use crate::frontend::ast::UnaryOp;
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(5, None)), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("-"));
    }

    // Test 26: transpile_operator_only_expr handles Assign
    #[test]
    fn test_transpile_operator_only_expr_assign() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(Expr::new(ExprKind::Identifier("x".to_string()), Span::default())),
                value: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(10, None)), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("x"));
    }

    // Test 27: transpile_operator_only_expr handles CompoundAssign
    #[test]
    fn test_transpile_operator_only_expr_compound_assign() {
        use crate::frontend::ast::BinaryOp;
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(Expr::new(ExprKind::Identifier("count".to_string()), Span::default())),
                op: BinaryOp::Add,
                value: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("count"));
    }

    // Test 28: transpile_operator_only_expr handles PreIncrement
    #[test]
    fn test_transpile_operator_only_expr_pre_increment() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::PreIncrement {
                target: Box::new(Expr::new(ExprKind::Identifier("i".to_string()), Span::default())),
            },
            Span::default(),
        );
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 29: transpile_control_flow_only_expr handles If
    #[test]
    fn test_transpile_control_flow_only_expr_if() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default())),
                then_branch: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::default())),
                else_branch: Some(Box::new(Expr::new(ExprKind::Literal(Literal::Integer(0, None)), Span::default()))),
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("if"));
    }

    // Test 30: transpile_control_flow_only_expr handles Match
    #[test]
    fn test_transpile_control_flow_only_expr_match() {
        use crate::frontend::ast::{MatchArm, Pattern};
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Match {
                expr: Box::new(Expr::new(ExprKind::Identifier("x".to_string()), Span::default())),
                arms: vec![MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::default())),
                    span: Span::default(),
                }],
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("match"));
    }

    // Test 31: transpile_control_flow_only_expr handles For
    #[test]
    fn test_transpile_control_flow_only_expr_for() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::For {
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(Expr::new(ExprKind::Identifier("items".to_string()), Span::default())),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), Span::default())),
                label: None,
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 32: transpile_control_flow_only_expr handles While
    #[test]
    fn test_transpile_control_flow_only_expr_while() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::While {
                condition: Box::new(Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default())),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), Span::default())),
                label: None,
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("while"));
    }

    // Test 33: transpile_control_flow_only_expr handles Loop
    #[test]
    fn test_transpile_control_flow_only_expr_loop() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Loop {
                body: Box::new(Expr::new(ExprKind::Block(vec![]), Span::default())),
                label: None,
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("loop"));
    }

    // Test 34: transpile_control_flow_only_expr handles IfLet
    #[test]
    fn test_transpile_control_flow_only_expr_if_let() {
        use crate::frontend::ast::Pattern;
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::IfLet {
                pattern: Pattern::Wildcard,
                expr: Box::new(Expr::new(ExprKind::Identifier("opt".to_string()), Span::default())),
                then_branch: Box::new(Expr::new(ExprKind::Block(vec![]), Span::default())),
                else_branch: None,
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("if") && tokens.contains("let"));
    }

    // Test 35: transpile_control_flow_only_expr handles WhileLet
    #[test]
    fn test_transpile_control_flow_only_expr_while_let() {
        use crate::frontend::ast::Pattern;
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::WhileLet {
                pattern: Pattern::Wildcard,
                expr: Box::new(Expr::new(ExprKind::Identifier("iter".to_string()), Span::default())),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), Span::default())),
                label: None,
            },
            Span::default(),
        );
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("while") && tokens.contains("let"));
    }
}
