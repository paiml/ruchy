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
mod codegen_minimal;

use crate::frontend::ast::{Attribute, Expr, ExprKind, Span, Type};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

// Module exports are handled by the impl blocks in each module

/// Block categorization result: (functions, statements, `has_main`, `main_expr`)
type BlockCategorization<'a> = (Vec<TokenStream>, Vec<TokenStream>, bool, Option<&'a Expr>);

/// The main transpiler struct
pub struct Transpiler {
    /// Track whether we're in an async context
    pub in_async_context: bool,
}

impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Transpiler {
    /// Creates a new transpiler instance
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::Transpiler;
    /// 
    /// let transpiler = Transpiler::new();
    /// assert!(!transpiler.in_async_context);
    /// ```
    pub fn new() -> Self {
        Self {
            in_async_context: false,
        }
    }

    /// Transpiles an expression to a `TokenStream`
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let mut parser = Parser::new("42");
    /// let ast = parser.parse().unwrap();
    /// 
    /// let transpiler = Transpiler::new();
    /// let result = transpiler.transpile(&ast);
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the AST cannot be transpiled to valid Rust code.
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
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let mut parser = Parser::new("42");
    /// let ast = parser.parse().unwrap();
    /// 
    /// let transpiler = Transpiler::new();
    /// let result = transpiler.transpile_to_program(&ast);
    /// assert!(result.is_ok());
    /// 
    /// let code = result.unwrap().to_string();
    /// assert!(code.contains("fn main"));
    /// assert!(code.contains("42"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the AST cannot be transpiled to a valid Rust program.
    pub fn transpile_to_program(&self, expr: &Expr) -> Result<TokenStream> {
        let needs_polars = Self::contains_dataframe(expr);
        
        match &expr.kind {
            ExprKind::Function { name, .. } => {
                self.transpile_single_function(expr, name, needs_polars)
            }
            ExprKind::Block(exprs) => {
                self.transpile_program_block(exprs, needs_polars)
            }
            _ => {
                self.transpile_expression_program(expr, needs_polars)
            }
        }
    }
    
    fn transpile_single_function(&self, expr: &Expr, name: &str, needs_polars: bool) -> Result<TokenStream> {
        let func = self.transpile_expr(expr)?;
        let needs_main = name != "main";
        
        match (needs_polars, needs_main) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #func
                fn main() { /* Function defined but not called */ }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #func
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                #func
                fn main() { /* Function defined but not called */ }
            }),
            (false, false) => Ok(quote! { 
                use std::collections::HashMap;
                #func 
            })
        }
    }
    
    fn transpile_program_block(&self, exprs: &[Expr], needs_polars: bool) -> Result<TokenStream> {
        let (functions, statements, has_main, main_expr) = self.categorize_block_expressions(exprs)?;
        
        if functions.is_empty() && !has_main {
            self.transpile_statement_only_block(exprs, needs_polars)
        } else if has_main {
            self.transpile_block_with_main_function(&functions, &statements, main_expr, needs_polars)
        } else {
            self.transpile_block_with_functions(&functions, &statements, needs_polars)
        }
    }
    
    fn categorize_block_expressions<'a>(&self, exprs: &'a [Expr]) -> Result<BlockCategorization<'a>> {
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        let mut has_main_function = false;
        let mut main_function_expr = None;
        
        for expr in exprs {
            if let ExprKind::Function { name, .. } = &expr.kind {
                if name == "main" {
                    has_main_function = true;
                    main_function_expr = Some(expr);
                } else {
                    functions.push(self.transpile_expr(expr)?);
                }
            } else {
                statements.push(self.transpile_expr(expr)?);
            }
        }
        
        Ok((functions, statements, has_main_function, main_function_expr))
    }
    
    fn transpile_statement_only_block(&self, exprs: &[Expr], needs_polars: bool) -> Result<TokenStream> {
        let block_expr = Expr::new(ExprKind::Block(exprs.to_vec()), Span::new(0, 0));
        let body = self.transpile_expr(&block_expr)?;
        self.wrap_in_main_with_result_printing(body, needs_polars)
    }
    
    fn transpile_block_with_main_function(&self, functions: &[TokenStream], statements: &[TokenStream], main_expr: Option<&Expr>, needs_polars: bool) -> Result<TokenStream> {
        if statements.is_empty() {
            // Only functions, just emit them normally (includes user's main)
            let main_tokens = if let Some(main) = main_expr {
                self.transpile_expr(main)?
            } else {
                return Err(anyhow::anyhow!("Expected main function expression"));
            };
            
            if needs_polars {
                Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    #(#functions)*
                    #main_tokens
                })
            } else {
                Ok(quote! {
                    use std::collections::HashMap;
                    #(#functions)*
                    #main_tokens
                })
            }
        } else {
            // TOP-LEVEL STATEMENTS: Extract main body and combine with statements
            let main_body = if let Some(main) = main_expr {
                self.extract_main_function_body(main)?
            } else {
                return Err(anyhow::anyhow!("Expected main function expression"));
            };
            
            if needs_polars {
                Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements;)*
                        
                        // Then user's main function body  
                        #main_body
                    }
                })
            } else {
                Ok(quote! {
                    use std::collections::HashMap;
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements;)*
                        
                        // Then user's main function body
                        #main_body
                    }
                })
            }
        }
    }
    
    /// Extracts the body of a main function for inlining with top-level statements
    fn extract_main_function_body(&self, main_expr: &Expr) -> Result<TokenStream> {
        if let ExprKind::Function { body, .. } = &main_expr.kind {
            // Transpile just the body, not the entire function definition
            self.transpile_expr(body)
        } else {
            Err(anyhow::anyhow!("Expected function expression for main body extraction"))
        }
    }
    
    fn transpile_block_with_functions(&self, functions: &[TokenStream], statements: &[TokenStream], needs_polars: bool) -> Result<TokenStream> {
        // No main function among extracted functions - create one for statements
        if needs_polars {
            Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #(#functions)*
                fn main() { #(#statements;)* }
            })
        } else {
            Ok(quote! {
                use std::collections::HashMap;
                #(#functions)*
                fn main() { #(#statements;)* }
            })
        }
    }
    
    
    fn transpile_expression_program(&self, expr: &Expr, needs_polars: bool) -> Result<TokenStream> {
        let body = self.transpile_expr(expr)?;
        self.wrap_in_main_with_result_printing(body, needs_polars)
    }
    
    fn wrap_in_main_with_result_printing(&self, body: TokenStream, needs_polars: bool) -> Result<TokenStream> {
        if needs_polars {
            Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                fn main() {
                    let result = #body;
                    match &result {
                        s if std::any::type_name_of_val(&s).contains("String") || 
                             std::any::type_name_of_val(&s).contains("&str") => println!("{}", s),
                        _ => println!("{:?}", result)
                    }
                }
            })
        } else {
            Ok(quote! {
                use std::collections::HashMap;
                fn main() {
                    let result = #body;
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

    /// Generate minimal code for self-hosting (direct Rust mapping, no optimization)
    pub fn transpile_minimal(&self, expr: &Expr) -> Result<String> {
        codegen_minimal::MinimalCodeGen::gen_program(expr)
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
            List, ListComprehension, Literal, Loop, Macro, Match, MethodCall, None, ObjectLiteral, Ok, QualifiedName, 
            Range, Send, Slice, Some, StringInterpolation, Struct, StructLiteral, Throw, Try,
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
            | Err { .. }
            | Some { .. }
            | None
            | Try { .. } => self.transpile_data_error_expr(expr),

            // Actor system and process execution
            Actor { .. } | Send { .. } | Ask { .. } | ActorSend { .. } | ActorQuery { .. } | Command { .. } => self.transpile_actor_expr(expr),

            // Everything else
            _ => self.transpile_misc_expr(expr),
        }
    }
}
