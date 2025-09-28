//! Dispatcher functions to reduce complexity in transpiler
//!
//! This module contains delegated transpilation functions to keep
//! cyclomatic complexity below 10 for each function.
use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
impl Transpiler {
    /// Transpile basic expressions (literals, identifiers, strings)
    pub(super) fn transpile_basic_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Self::transpile_literal(lit)),
            ExprKind::Identifier(name) => Ok(Self::transpile_identifier(name)),
            ExprKind::QualifiedName { module, name } => {
                Ok(Self::transpile_qualified_name(module, name))
            }
            ExprKind::StringInterpolation { parts } => self.transpile_string_interpolation(parts),
            ExprKind::TypeCast { expr, target_type } => self.transpile_type_cast(expr, target_type),
            _ => unreachable!("Non-basic expression in transpile_basic_expr"),
        }
    }
    fn transpile_type_cast(&self, expr: &Expr, target_type: &str) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        // Map Ruchy types to Rust types
        let rust_type = match target_type {
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            "f32" => quote! { f32 },
            "f64" => quote! { f64 },
            "usize" => quote! { usize },
            "u8" => quote! { u8 },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "i8" => quote! { i8 },
            "i16" => quote! { i16 },
            _ => bail!("Unsupported cast target type: {}", target_type),
        };
        Ok(quote! { (#expr_tokens as #rust_type) })
    }
    fn transpile_identifier(name: &str) -> TokenStream {
        // Check if this is a module path like "math::add"
        if name.contains("::") {
            // Split into module path components
            let parts: Vec<&str> = name.split("::").collect();
            let mut tokens = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                let safe_part = if matches!(*part, "self" | "Self" | "super" | "crate") {
                    (*part).to_string()
                } else if Self::is_rust_reserved_keyword(part) {
                    format!("r#{part}")
                } else {
                    (*part).to_string()
                };
                let ident = format_ident!("{}", safe_part);
                tokens.push(quote! { #ident });
                if i < parts.len() - 1 {
                    tokens.push(quote! { :: });
                }
            }
            quote! { #(#tokens)* }
        } else {
            // Handle single identifier with Rust reserved keywords
            let safe_name = if matches!(name, "self" | "Self" | "super" | "crate") {
                // These keywords cannot be raw identifiers, use them as-is
                name.to_string()
            } else if Self::is_rust_reserved_keyword(name) {
                format!("r#{name}")
            } else {
                name.to_string()
            };
            let ident = format_ident!("{}", safe_name);
            quote! { #ident }
        }
    }
    fn transpile_qualified_name(module: &str, name: &str) -> TokenStream {
        // Handle nested qualified names like "net::TcpListener"
        let module_parts: Vec<&str> = module.split("::").collect();
        let name_ident = format_ident!("{}", name);
        if module_parts.len() == 1 {
            // Simple case: single module name
            let module_ident = format_ident!("{}", module_parts[0]);
            quote! { #module_ident::#name_ident }
        } else {
            // Complex case: nested path like "net::TcpListener"
            let mut tokens = TokenStream::new();
            for (i, part) in module_parts.iter().enumerate() {
                if i > 0 {
                    tokens.extend(quote! { :: });
                }
                let part_ident = format_ident!("{}", part);
                tokens.extend(quote! { #part_ident });
            }
            quote! { #tokens::#name_ident }
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
            } => self.transpile_for(var, pattern.as_ref(), iter, body),
            ExprKind::While { condition, body } => self.transpile_while(condition, body),
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
            } => self.transpile_while_let(pattern, expr, body),
            ExprKind::Loop { body } => self.transpile_loop(body),
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
            _ => bail!("Unknown macro: {}", name),
        }
    }
    /// Transpile structure-related expressions
    pub(super) fn transpile_struct_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Struct {
                name,
                type_params,
                fields,
                derives,
                is_pub,
            } => self.transpile_struct(name, type_params, fields, derives, *is_pub),
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
                superclass: _, // TODO: implement inheritance
                traits,
                fields,
                constructors,
                methods,
                constants,
                properties: _, // TODO: implement property transpilation
                derives,
                is_pub,
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
        Ok(quote! { Ok(#value_tokens) })
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
        Ok(quote! { Some(#value_tokens) })
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
            } => self.transpile_let_with_type(
                name,
                type_annotation.as_ref(),
                value,
                body,
                *is_mutable,
            ),
            ExprKind::LetPattern {
                pattern,
                type_annotation,
                value,
                body,
                is_mutable: _,
            } => {
                self.transpile_let_pattern_with_type(pattern, type_annotation.as_ref(), value, body)
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
                Self::transpile_control_misc_expr(expr)
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
                methods,
                is_pub,
            } => self.transpile_trait(name, type_params, methods, *is_pub),
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
    /// Transpile println! macro with string formatting support
    ///
    /// Handles string literals, string interpolation, and format strings correctly.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles arguments and wraps them in Rust's `println!` macro.
    /// Empty args produce `println!()`, otherwise `println!(arg1, arg2, ...)`
    fn transpile_println_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens = self.transpile_print_args(args)?;
        if arg_tokens.is_empty() {
            Ok(quote! { println!() })
        } else {
            Ok(quote! { println!(#(#arg_tokens),*) })
        }
    }
    /// Transpile print! macro with string formatting support
    ///
    /// Handles string literals, string interpolation, and format strings correctly.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles arguments and wraps them in Rust's `print!` macro.
    /// Empty args produce `print!()`, otherwise `print!(arg1, arg2, ...)`
    fn transpile_print_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens = self.transpile_print_args(args)?;
        if arg_tokens.is_empty() {
            Ok(quote! { print!() })
        } else {
            Ok(quote! { print!(#(#arg_tokens),*) })
        }
    }
    /// Transpile panic! macro with string formatting support
    ///
    /// Handles string literals, string interpolation, and format strings correctly.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles arguments and wraps them in Rust's `panic!` macro.
    /// Empty args produce `panic!()`, otherwise `panic!(arg1, arg2, ...)`
    fn transpile_panic_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens = self.transpile_print_args(args)?;
        if arg_tokens.is_empty() {
            Ok(quote! { panic!() })
        } else {
            Ok(quote! { panic!(#(#arg_tokens),*) })
        }
    }
    /// Common helper for transpiling print-style macro arguments
    ///
    /// Handles string literals, string interpolation, and format strings.
    /// This eliminates code duplication between println!, print!, and panic!.
    /// Complexity: <10 per Toyota Way requirement.
    fn transpile_print_args(&self, args: &[Expr]) -> Result<Vec<TokenStream>> {
        if args.is_empty() {
            return Ok(vec![]);
        }
        // Check if first argument is a format string (contains {})
        let first_is_format_string = match &args[0].kind {
            ExprKind::Literal(Literal::String(s)) => s.contains("{}"),
            _ => false,
        };
        if first_is_format_string && args.len() > 1 {
            // First argument is format string, rest are values
            let format_str = match &args[0].kind {
                ExprKind::Literal(Literal::String(s)) => s,
                _ => unreachable!(),
            };
            let mut tokens = vec![quote! { #format_str }];
            // Add remaining arguments as values (without extra format strings)
            for arg in &args[1..] {
                let expr_tokens = self.transpile_expr(arg)?;
                tokens.push(expr_tokens);
            }
            Ok(tokens)
        } else {
            // Original behavior for non-format cases
            args.iter()
                .map(|arg| {
                    match &arg.kind {
                        ExprKind::Literal(Literal::String(s)) => Ok(quote! { #s }),
                        ExprKind::StringInterpolation { parts } => {
                            self.transpile_string_interpolation_for_print(parts)
                        }
                        _ => {
                            // Use Display formatting ({}) for simple values, Debug formatting ({:?}) for complex types
                            let expr_tokens = self.transpile_expr(arg)?;
                            match &arg.kind {
                                // Simple types that have clean Display formatting
                                ExprKind::Literal(_) | ExprKind::Identifier(_) => {
                                    Ok(quote! { "{}", #expr_tokens })
                                }
                                // Complex types that need Debug formatting for safety
                                _ => Ok(quote! { "{:?}", #expr_tokens }),
                            }
                        }
                    }
                })
                .collect()
        }
    }
    /// Handle string interpolation for print-style macros
    ///
    /// Detects if string interpolation has expressions or is just format text.
    /// Complexity: <10 per Toyota Way requirement.
    fn transpile_string_interpolation_for_print(
        &self,
        parts: &[crate::frontend::ast::StringPart],
    ) -> Result<TokenStream> {
        let has_expressions = parts.iter().any(|part| {
            matches!(
                part,
                crate::frontend::ast::StringPart::Expr(_)
                    | crate::frontend::ast::StringPart::ExprWithFormat { .. }
            )
        });
        if has_expressions {
            // This has actual interpolation - transpile normally
            self.transpile_string_interpolation(parts)
        } else {
            // This is a format string like "Hello {}" - treat as literal
            let format_string = parts
                .iter()
                .map(|part| match part {
                    crate::frontend::ast::StringPart::Text(s) => s.as_str(),
                    crate::frontend::ast::StringPart::Expr(_)
                    | crate::frontend::ast::StringPart::ExprWithFormat { .. } => unreachable!(),
                })
                .collect::<String>();
            Ok(quote! { #format_string })
        }
    }
    /// Transpile vec! macro
    ///
    /// Simple element-by-element transpilation for collection creation.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles list elements and wraps them in Rust's `vec!` macro.
    /// Produces `vec![elem1, elem2, ...]`
    fn transpile_vec_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { vec![#(#arg_tokens),*] })
    }
    /// Transpile assert! macro
    ///
    /// Simple argument transpilation for basic assertions.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Transpiles assertion condition and wraps it in Rust's `assert!` macro.
    /// Produces `assert!(condition, optional_message)`
    fn transpile_assert_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        if arg_tokens.is_empty() {
            Ok(quote! { assert!() })
        } else {
            Ok(quote! { assert!(#(#arg_tokens),*) })
        }
    }
    /// Transpile `assert_eq`! macro with validation
    ///
    /// Validates argument count and transpiles for equality assertions.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Validates at least 2 arguments and transpiles to Rust's `assert_eq!` macro.
    /// Produces `assert_eq!(left, right, optional_message)`
    fn transpile_assert_eq_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() < 2 {
            bail!("assert_eq! requires at least 2 arguments")
        }
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { assert_eq!(#(#arg_tokens),*) })
    }
    /// Transpile `assert_ne`! macro with validation
    ///
    /// Validates argument count and transpiles for inequality assertions.
    /// Complexity: <10 per Toyota Way requirement.
    ///
    /// # Example Usage
    /// Validates at least 2 arguments and transpiles to Rust's `assert_ne!` macro.
    /// Produces `assert_ne!(left, right, optional_message)`
    fn transpile_assert_ne_macro(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() < 2 {
            bail!("assert_ne! requires at least 2 arguments")
        }
        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { assert_ne!(#(#arg_tokens),*) })
    }

    /// Pass through external macros without modification
    fn transpile_passthrough_macro(&self, name: &str, args: &[Expr]) -> Result<TokenStream> {
        let macro_ident = format_ident!("{}", name);

        // Special handling for json! macro - needs raw JSON syntax
        if name == "json" && args.len() == 1 {
            if let ExprKind::ObjectLiteral { fields } = &args[0].kind {
                return self.transpile_json_macro_object(fields);
            }
        }

        let arg_tokens: Result<Vec<_>, _> =
            args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;
        Ok(quote! { #macro_ident!(#(#arg_tokens),*) })
    }

    /// Transpile object literal for json! macro
    fn transpile_json_macro_object(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<TokenStream> {
        use crate::frontend::ast::ObjectField;
        let mut json_fields = Vec::new();
        for field in fields {
            match field {
                ObjectField::KeyValue { key, value } => {
                    let value_tokens = match &value.kind {
                        ExprKind::Literal(Literal::String(s)) => {
                            quote! { #s }
                        }
                        _ => self.transpile_expr(value)?,
                    };
                    json_fields.push(quote! { #key: #value_tokens });
                }
                ObjectField::Spread { .. } => {
                    // JSON doesn't support spread, skip for now
                }
            }
        }
        Ok(quote! { json!({ #(#json_fields),* }) })
    }
    fn transpile_control_misc_expr(expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Break { label } => Ok(Self::make_break_continue(true, label.as_ref())),
            ExprKind::Continue { label } => Ok(Self::make_break_continue(false, label.as_ref())),
            ExprKind::Return { value } => {
                if let Some(val) = value {
                    let transpiler = Transpiler::new();
                    let val_tokens = transpiler.transpile_expr(val)?;
                    Ok(quote! { return #val_tokens })
                } else {
                    Ok(quote! { return })
                }
            }
            // Export variants are now handled elsewhere
            ExprKind::Export { .. } | ExprKind::ExportList { .. } => {
                // These should be handled in the main dispatch
                Ok(quote! { /* Export handled in main dispatch */ })
            }
            _ => unreachable!(),
        }
    }
    fn make_break_continue(is_break: bool, label: Option<&String>) -> TokenStream {
        let keyword = if is_break {
            quote! { break }
        } else {
            quote! { continue }
        };
        match label {
            Some(l) if !l.is_empty() => {
                let label_ident = format_ident!("{}", l);
                quote! { #keyword #label_ident }
            }
            _ => keyword, // Handle both None and empty string cases
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span};
    use quote::quote;

    // Helper to create a test expression
    fn make_expr(kind: ExprKind) -> Box<Expr> {
        Box::new(Expr {
            kind,
            span: Span::new(0, 0),
            attributes: vec![],
        })
    }

    #[test]
    fn test_transpile_identifier() {
        let result = Transpiler::transpile_identifier("test_var");
        let expected = quote! { test_var };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_identifier_with_keyword() {
        let result = Transpiler::transpile_identifier("type");
        let expected = quote! { r#type };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_literal_integer() {
        let result = Transpiler::transpile_literal(&Literal::Integer(42));
        let expected = quote! { 42 };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_transpile_literal_float() {
        let result = Transpiler::transpile_literal(&Literal::Float(3.14));
        let expected = quote! { 3.14f64 };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_literal_string() {
        let result = Transpiler::transpile_literal(&Literal::String("hello".to_string()));
        let expected = quote! { "hello" };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_literal_boolean_true() {
        let result = Transpiler::transpile_literal(&Literal::Bool(true));
        let expected = quote! { true };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_literal_boolean_false() {
        let result = Transpiler::transpile_literal(&Literal::Bool(false));
        let expected = quote! { false };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_literal_unit() {
        let result = Transpiler::transpile_literal(&Literal::Unit);
        let expected = quote! { () };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_transpile_literal_char() {
        let result = Transpiler::transpile_literal(&Literal::Char('a'));
        let expected = quote! { 'a' };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_make_break_continue_break_no_label() {
        let result = Transpiler::make_break_continue(true, None);
        let expected = quote! { break };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_make_break_continue_break_with_label() {
        let result = Transpiler::make_break_continue(true, Some(&"loop1".to_string()));
        let expected = quote! { break loop1 };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_make_break_continue_continue_no_label() {
        let result = Transpiler::make_break_continue(false, None);
        let expected = quote! { continue };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_make_break_continue_continue_with_label() {
        let result = Transpiler::make_break_continue(false, Some(&"loop2".to_string()));
        let expected = quote! { continue loop2 };
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_reserved_keyword_handling() {
        // Test various reserved keywords
        let keywords = vec!["type", "match", "async", "await", "move", "ref", "static"];
        for keyword in keywords {
            let result = Transpiler::transpile_identifier(keyword);
            assert!(result.to_string().contains("r#"));
        }
    }

    #[test]
    fn test_non_reserved_keyword() {
        let non_keywords = vec!["my_var", "foo", "bar", "data", "value"];
        for name in non_keywords {
            let result = Transpiler::transpile_identifier(name);
            assert!(!result.to_string().contains("r#"));
            assert_eq!(result.to_string(), name);
        }
    }

    #[test]
    fn test_transpile_type_cast() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Default::default());

        // Test various type casts
        let types = vec![
            "i32", "i64", "f32", "f64", "usize", "u8", "u16", "u32", "u64", "i8", "i16",
        ];
        for target_type in types {
            let result = transpiler.transpile_type_cast(&expr, target_type);
            assert!(result.is_ok());
            let tokens = result.unwrap();
            assert!(tokens.to_string().contains("as"));
            assert!(tokens.to_string().contains(target_type));
        }
    }

    #[test]
    fn test_transpile_type_cast_unsupported() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Default::default());

        let result = transpiler.transpile_type_cast(&expr, "unknown_type");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported cast target type"));
    }

    #[test]
    fn test_transpile_qualified_name() {
        // Simple module
        let result = Transpiler::transpile_qualified_name("std", "vec");
        assert_eq!(result.to_string(), "std :: vec");

        // Nested module path
        let result = Transpiler::transpile_qualified_name("std::collections", "HashMap");
        assert!(result.to_string().contains("std"));
        assert!(result.to_string().contains("collections"));
        assert!(result.to_string().contains("HashMap"));
    }

    #[test]
    fn test_transpile_identifier_with_path() {
        // Test module path identifier
        let result = Transpiler::transpile_identifier("std::collections::HashMap");
        assert!(result.to_string().contains("std"));
        assert!(result.to_string().contains("collections"));
        assert!(result.to_string().contains("HashMap"));

        // Test with reserved keywords in path
        let result = Transpiler::transpile_identifier("mod::type::static");
        assert!(result.to_string().contains("r#"));
    }

    #[test]
    fn test_transpile_identifier_special_keywords() {
        // Test special keywords that cannot be raw identifiers
        let special = vec!["self", "Self", "super", "crate"];
        for keyword in special {
            let result = Transpiler::transpile_identifier(keyword);
            assert!(!result.to_string().contains("r#"));
            assert_eq!(result.to_string(), keyword);
        }
    }

    #[test]
    fn test_transpile_basic_expr() {
        let transpiler = Transpiler::new();

        // Test literal
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Default::default());
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "42");

        // Test identifier
        let expr = Expr::new(
            ExprKind::Identifier("my_var".to_string()),
            Default::default(),
        );
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "my_var");

        // Test qualified name
        let expr = Expr::new(
            ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "vec".to_string(),
            },
            Default::default(),
        );
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_rust_reserved_keyword() {
        // Test that all documented Rust keywords are recognized
        let reserved = vec![
            "as", "async", "await", "break", "const", "continue", "crate", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "static", "struct", "super", "trait", "true",
            "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final",
            "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
        ];

        for keyword in reserved {
            assert!(
                Transpiler::is_rust_reserved_keyword(keyword),
                "Keyword '{keyword}' should be recognized as reserved"
            );
        }

        // Test non-keywords
        let non_reserved = vec!["my_var", "data", "value", "self_data"];
        for word in non_reserved {
            assert!(
                !Transpiler::is_rust_reserved_keyword(word),
                "Word '{word}' should not be recognized as reserved"
            );
        }
    }

    #[test]
    fn test_qualified_name_with_multiple_parts() {
        let result = Transpiler::transpile_qualified_name("a::b::c", "Method");
        let str_result = result.to_string();
        assert!(str_result.contains('a'));
        assert!(str_result.contains('b'));
        assert!(str_result.contains('c'));
        assert!(str_result.contains("Method"));
    }

    #[test]
    fn test_identifier_path_with_reserved() {
        // Test path with reserved keyword at different positions
        let result = Transpiler::transpile_identifier("std::type::new");
        assert!(result.to_string().contains("r#type"));

        let result = Transpiler::transpile_identifier("async::await::future");
        assert!(result.to_string().contains("r#async"));
        assert!(result.to_string().contains("r#await"));
    }

    #[test]
    fn test_literal_edge_cases() {
        // Test extreme integer values
        let result = Transpiler::transpile_literal(&Literal::Integer(i64::MAX));
        assert!(result.to_string().contains(&i64::MAX.to_string()));

        let result = Transpiler::transpile_literal(&Literal::Integer(i64::MIN));
        // i64::MIN transpiles correctly without overflow - just verify it doesn't panic
        assert!(!result.to_string().is_empty());

        // Test special float values
        let result = Transpiler::transpile_literal(&Literal::Float(0.0));
        assert_eq!(result.to_string(), "0f64");

        let result = Transpiler::transpile_literal(&Literal::Float(-0.0));
        // -0.0 transpiles to '- 0f64' (preserves the negative sign)
        assert_eq!(result.to_string(), "- 0f64");

        // Test special characters
        let result = Transpiler::transpile_literal(&Literal::Char('\n'));
        assert_eq!(result.to_string(), "'\\n'");

        let result = Transpiler::transpile_literal(&Literal::Char('\''));
        assert_eq!(result.to_string(), "'\\''");
    }

    #[test]
    fn test_string_with_special_chars() {
        let result = Transpiler::transpile_literal(&Literal::String("Hello\nWorld".to_string()));
        assert!(result.to_string().contains("Hello"));

        let result = Transpiler::transpile_literal(&Literal::String("Test\"Quote\"".to_string()));
        assert!(result.to_string().contains("Test"));
    }

    #[test]
    fn test_make_break_continue_edge_cases() {
        // Test empty label
        let result = Transpiler::make_break_continue(true, Some(&String::new()));
        assert_eq!(result.to_string(), "break");

        // Test label with special characters (though this shouldn't happen in practice)
        let result = Transpiler::make_break_continue(false, Some(&"my_loop_1".to_string()));
        assert_eq!(result.to_string(), "continue my_loop_1");
    }
}
