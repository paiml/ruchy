//! Modular transpiler for Ruchy language
//!
//! This module is responsible for converting Ruchy AST into Rust code using `proc_macro2` `TokenStream`.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

mod actors;
mod dataframe;
mod dataframe_helpers;
mod dispatcher;
mod expressions;
mod patterns;
mod result_type;
mod statements;
mod types;

use crate::frontend::ast::{Attribute, Expr, ExprKind, Type};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

// Module exports are handled by the impl blocks in each module

/// The main transpiler struct
pub struct Transpiler {
    /// Track whether we're in an async context
    in_async_context: bool,
}

impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
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

    /// Checks if an expression contains `DataFrame` operations (simplified for complexity)
    fn contains_dataframe(expr: &Expr) -> bool {
        matches!(
            expr.kind,
            ExprKind::DataFrame { .. } | ExprKind::DataFrameOperation { .. }
        )
    }

    /// Wraps transpiled code in a complete Rust program with necessary imports
    pub fn transpile_to_program(&self, expr: &Expr) -> Result<TokenStream> {
        let body = self.transpile_expr(expr)?;
        let needs_polars = Self::contains_dataframe(expr);

        if needs_polars {
            Ok(quote! {
                use polars::prelude::*;

                fn main() {
                    let result = #body;
                    println!("{:?}", result);
                }
            })
        } else {
            Ok(quote! {
                fn main() {
                    let result = #body;
                    println!("{:?}", result);
                }
            })
        }
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
        use ExprKind::{
            Actor, Ask, AsyncBlock, Await, Binary, Call, Command, DataFrame, DataFrameOperation, Err,
            FieldAccess, For, Function, Identifier, If, IndexAccess, Lambda, List, ListComprehension, 
            Literal, Match, MethodCall, ObjectLiteral, Ok, QualifiedName, Range, Send, Slice,
            StringInterpolation, Struct, StructLiteral, Throw, Try, TryCatch, Tuple, Unary, While,
        };

        // Dispatch to specialized handlers to keep complexity below 10
        match &expr.kind {
            // Basic expressions
            Literal(_) | Identifier(_) | QualifiedName { .. } | StringInterpolation { .. } => {
                self.transpile_basic_expr(expr)
            }

            // Operators and control flow
            Binary { .. }
            | Unary { .. }
            | Try { .. }
            | Await { .. }
            | AsyncBlock { .. }
            | If { .. }
            | Match { .. }
            | For { .. }
            | While { .. } => self.transpile_operator_control_expr(expr),

            // Functions
            Function { .. } | Lambda { .. } | Call { .. } | MethodCall { .. } => {
                self.transpile_function_expr(expr)
            }

            // Structures
            Struct { .. } | StructLiteral { .. } | ObjectLiteral { .. } | FieldAccess { .. } 
            | IndexAccess { .. } | Slice { .. } => {
                self.transpile_struct_expr(expr)
            }

            // Data and error handling
            DataFrame { .. }
            | DataFrameOperation { .. }
            | List(_)
            | Tuple(_)
            | ListComprehension { .. }
            | Range { .. }
            | TryCatch { .. }
            | Throw { .. }
            | Ok { .. }
            | Err { .. } => self.transpile_data_error_expr(expr),

            // Actor system and process execution
            Actor { .. } | Send { .. } | Ask { .. } | Command { .. } => self.transpile_actor_expr(expr),

            // Everything else
            _ => self.transpile_misc_expr(expr),
        }
    }
}
