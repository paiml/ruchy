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
        let needs_polars = Self::contains_dataframe(expr);
        
        // Check if this is a function definition
        if let ExprKind::Function { name, .. } = &expr.kind {
            // For function definitions, transpile as a top-level item
            let func = self.transpile_expr(expr)?;
            
            // Only add empty main if the function is not already named "main"
            let needs_main = name != "main";
            
            if needs_polars {
                if needs_main {
                    Ok(quote! {
                        use polars::prelude::*;
                        
                        #func
                        
                        fn main() {
                            // Function defined but not called
                        }
                    })
                } else {
                    Ok(quote! {
                        use polars::prelude::*;
                        
                        #func
                    })
                }
            } else if needs_main {
                Ok(quote! {
                    #func
                    
                    fn main() {
                        // Function defined but not called
                    }
                })
            } else {
                Ok(quote! {
                    #func
                })
            }
        } else if let ExprKind::Block(exprs) = &expr.kind {
            // Handle blocks that contain function definitions and statements
            // Check if this block contains functions - if so, extract them as top-level items
            let mut functions = Vec::new();
            let mut statements = Vec::new();
            
            for expr in exprs {
                if let ExprKind::Function { .. } = &expr.kind {
                    functions.push(self.transpile_expr(expr)?);
                } else {
                    statements.push(self.transpile_expr(expr)?);
                }
            }
            
            if !functions.is_empty() {
                // We have function definitions - put them at top level
                if needs_polars {
                    Ok(quote! {
                        use polars::prelude::*;
                        
                        #(#functions)*
                        
                        fn main() {
                            #(#statements;)*
                        }
                    })
                } else {
                    Ok(quote! {
                        #(#functions)*
                        
                        fn main() {
                            #(#statements;)*
                        }
                    })
                }
            } else {
                // No functions, treat as normal expression block
                let body = self.transpile_expr(expr)?;
                if needs_polars {
                    Ok(quote! {
                        use polars::prelude::*;

                        fn main() {
                            let result = #body;
                            // Use Display trait for strings, Debug for other types
                            match &result {
                                s if std::any::type_name_of_val(&s).contains("String") || 
                                     std::any::type_name_of_val(&s).contains("&str") => println!("{}", s),
                                _ => println!("{:?}", result)
                            }
                        }
                    })
                } else {
                    Ok(quote! {
                        fn main() {
                            let result = #body;
                            // For strings, print without quotes
                            if let Some(s) = (&result as &dyn std::any::Any).downcast_ref::<String>() {
                                println!("{}", s);
                            } else if let Some(s) = (&result as &dyn std::any::Any).downcast_ref::<&str>() {
                                println!("{}", s);
                            } else {
                                println!("{:?}", result);
                            }
                        }
                    })
                }
            }
        } else {
            // For expressions, wrap in main and execute
            let body = self.transpile_expr(expr)?;
            if needs_polars {
                Ok(quote! {
                    use polars::prelude::*;

                    fn main() {
                        let result = #body;
                        // Use Display trait for strings, Debug for other types
                        match &result {
                            s if std::any::type_name_of_val(&s).contains("String") || 
                                 std::any::type_name_of_val(&s).contains("&str") => println!("{}", s),
                            _ => println!("{:?}", result)
                        }
                    }
                })
            } else {
                Ok(quote! {
                    fn main() {
                        let result = #body;
                        // For strings, print without quotes
                        if let Some(s) = (&result as &dyn std::any::Any).downcast_ref::<String>() {
                            println!("{}", s);
                        } else if let Some(s) = (&result as &dyn std::any::Any).downcast_ref::<&str>() {
                            println!("{}", s);
                        } else {
                            println!("{:?}", result);
                        }
                    }
                })
            }
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
            Actor, ActorQuery, ActorSend, Ask, Assign, AsyncBlock, Await, Binary, Call, Command, CompoundAssign, DataFrame, 
            DataFrameOperation, Err, FieldAccess, For, Function, Identifier, If, IfLet, IndexAccess, Lambda, 
            List, ListComprehension, Literal, Loop, Macro, Match, MethodCall, ObjectLiteral, Ok, QualifiedName, 
            Range, Send, Slice, StringInterpolation, Struct, StructLiteral, Throw, 
            Tuple, Unary, While, WhileLet,
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
            | Assign { .. }
            | CompoundAssign { .. }
            | Await { .. }
            | AsyncBlock { .. }
            | If { .. }
            | IfLet { .. }
            | Match { .. }
            | For { .. }
            | While { .. }
            | WhileLet { .. }
            | Loop { .. } => self.transpile_operator_control_expr(expr),

            // Functions
            Function { .. } | Lambda { .. } | Call { .. } | MethodCall { .. } | Macro { .. } => {
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
            | Throw { .. }
            | Ok { .. }
            | Err { .. } => self.transpile_data_error_expr(expr),

            // Actor system and process execution
            Actor { .. } | Send { .. } | Ask { .. } | ActorSend { .. } | ActorQuery { .. } | Command { .. } => self.transpile_actor_expr(expr),

            // Everything else
            _ => self.transpile_misc_expr(expr),
        }
    }
}
