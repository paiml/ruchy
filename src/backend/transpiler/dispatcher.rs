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
            | ExprKind::Await { .. }
            | ExprKind::AsyncBlock { .. } => self.transpile_operator_only_expr(expr),
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
            ExprKind::CompoundAssign { target, op, value } => self.transpile_compound_assign(target, *op, value),
            ExprKind::Await { expr } => self.transpile_await(expr),
            ExprKind::AsyncBlock { body } => self.transpile_async_block(body),
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
            ExprKind::For { var, pattern, iter, body } => self.transpile_for(var, pattern.as_ref(), iter, body),
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
            ExprKind::TryCatch { try_block, catch_clauses, finally_block } => {
                self.transpile_try_catch(try_block, catch_clauses, finally_block.as_deref())
            }
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
                is_pub,
            } => self.transpile_struct(name, type_params, fields, *is_pub),
            ExprKind::StructLiteral { name, fields } => self.transpile_struct_literal(name, fields),
            ExprKind::ObjectLiteral { fields } => self.transpile_object_literal(fields),
            ExprKind::FieldAccess { object, field } => self.transpile_field_access(object, field),
            ExprKind::IndexAccess { object, index } => self.transpile_index_access(object, index),
            ExprKind::Slice { object, start, end } => self.transpile_slice(object, start.as_deref(), end.as_deref()),
            _ => unreachable!("Non-struct expression in transpile_struct_expr"),
        }
    }

    /// Transpile data and error handling expressions (split for complexity)
    pub(super) fn transpile_data_error_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::DataFrame { .. }
            | ExprKind::DataFrameOperation { .. }
            | ExprKind::List(_)
            | ExprKind::Tuple(_)
            | ExprKind::ListComprehension { .. }
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
            ExprKind::Tuple(elements) => self.transpile_tuple(elements),
            ExprKind::ListComprehension {
                element,
                variable,
                iterable,
                condition,
            } => {
                self.transpile_list_comprehension(element, variable, iterable, condition.as_deref())
            }
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
        Ok(quote! { Err(#error_tokens) })
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
            ExprKind::Command { program, args, env, working_dir } => {
                self.transpile_command(program, args, env, working_dir)
            }
            _ => unreachable!("Non-actor expression in transpile_actor_expr"),
        }
    }

    /// Transpile miscellaneous expressions
    pub(super) fn transpile_misc_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Let {
                name,
                type_annotation: _,
                value,
                body,
                is_mutable,
            } => self.transpile_let(name, value, body, *is_mutable),
            ExprKind::LetPattern {
                pattern,
                type_annotation: _,
                value,
                body,
                is_mutable: _,
            } => self.transpile_let_pattern(pattern, value, body),
            ExprKind::Block(exprs) => self.transpile_block(exprs),
            ExprKind::Pipeline { expr, stages } => self.transpile_pipeline(expr, stages),
            ExprKind::Import { path, items } => Ok(Self::transpile_import(path, items)),
            ExprKind::Module { name, body } => self.transpile_module(name, body),
            ExprKind::Trait { .. }
            | ExprKind::Impl { .. }
            | ExprKind::Extension { .. }
            | ExprKind::Enum { .. } => self.transpile_type_decl_expr(expr),
            ExprKind::Break { .. } | ExprKind::Continue { .. } | ExprKind::Return { .. } | ExprKind::Export { .. } => {
                Self::transpile_control_misc_expr(expr)
            }
            _ => bail!("Unsupported expression kind: {:?}", expr.kind),
        }
    }

    fn transpile_type_decl_expr(&self, expr: &Expr) -> Result<TokenStream> {
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
            } => self.transpile_impl(for_type, type_params, trait_name.as_deref(), methods, *is_pub),
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
        args.iter()
            .map(|arg| {
                match &arg.kind {
                    ExprKind::Literal(Literal::String(s)) => {
                        Ok(quote! { #s })
                    }
                    ExprKind::StringInterpolation { parts } => {
                        self.transpile_string_interpolation_for_print(parts)
                    }
                    _ => {
                        // Use Debug formatting for all non-string expressions to be safe
                        // This prevents Display trait errors and works with all types
                        let expr_tokens = self.transpile_expr(arg)?;
                        Ok(quote! { "{:?}", #expr_tokens })
                    }
                }
            })
            .collect()
    }


    /// Handle string interpolation for print-style macros
    /// 
    /// Detects if string interpolation has expressions or is just format text.
    /// Complexity: <10 per Toyota Way requirement.
    fn transpile_string_interpolation_for_print(&self, parts: &[crate::frontend::ast::StringPart]) -> Result<TokenStream> {
        let has_expressions = parts.iter().any(|part| matches!(part, 
            crate::frontend::ast::StringPart::Expr(_) | 
            crate::frontend::ast::StringPart::ExprWithFormat { .. }));
        
        if has_expressions {
            // This has actual interpolation - transpile normally
            self.transpile_string_interpolation(parts)
        } else {
            // This is a format string like "Hello {}" - treat as literal
            let format_string = parts.iter()
                .map(|part| match part {
                    crate::frontend::ast::StringPart::Text(s) => s.as_str(),
                    crate::frontend::ast::StringPart::Expr(_) | 
                    crate::frontend::ast::StringPart::ExprWithFormat { .. } => unreachable!()
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
        let arg_tokens: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| self.transpile_expr(arg))
            .collect();
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
        let arg_tokens: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| self.transpile_expr(arg))
            .collect();
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
        
        let arg_tokens: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| self.transpile_expr(arg))
            .collect();
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
        
        let arg_tokens: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| self.transpile_expr(arg))
            .collect();
        let arg_tokens = arg_tokens?;

        Ok(quote! { assert_ne!(#(#arg_tokens),*) })
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
            ExprKind::Export { items } => {
                let item_idents: Vec<_> =
                    items.iter().map(|item| format_ident!("{}", item)).collect();
                Ok(quote! { pub use { #(#item_idents),* }; })
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
            Some(l) => {
                let label_ident = format_ident!("{}", l);
                quote! { #keyword #label_ident }
            }
            None => keyword,
        }
    }

}
