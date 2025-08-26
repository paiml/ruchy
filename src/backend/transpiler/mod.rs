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
mod type_inference;
mod types;
mod codegen_minimal;

use crate::frontend::ast::{Attribute, Expr, ExprKind, Span, Type};
use crate::backend::module_resolver::ModuleResolver;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

// Module exports are handled by the impl blocks in each module

/// Block categorization result: (functions, statements, modules, `has_main`, `main_expr`)
type BlockCategorization<'a> = (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, bool, Option<&'a Expr>);

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
    /// Creates a new transpiler instance without module loader
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

    /// Resolves file imports in the AST using `ModuleResolver`
    #[allow(dead_code)]
    fn resolve_imports(&self, expr: &Expr) -> Result<Expr> {
        // For now, just use default search paths since we don't have file context here
        let mut resolver = ModuleResolver::new();
        resolver.resolve_imports(expr.clone())
    }

    /// Resolves file imports with a specific file context for search paths
    fn resolve_imports_with_context(&self, expr: &Expr, file_path: Option<&std::path::Path>) -> Result<Expr> {
        let mut resolver = ModuleResolver::new();
        
        // Add the file's directory to search paths if provided
        if let Some(path) = file_path {
            if let Some(dir) = path.parent() {
                resolver.add_search_path(dir);
            }
        }
        
        resolver.resolve_imports(expr.clone())
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
        self.transpile_to_program_with_context(expr, None)
    }

    /// Transpile with file context for module resolution
    pub fn transpile_to_program_with_context(&self, expr: &Expr, file_path: Option<&std::path::Path>) -> Result<TokenStream> {
        // First, resolve any file imports using the module resolver
        let resolved_expr = self.resolve_imports_with_context(expr, file_path)?;
        let needs_polars = Self::contains_dataframe(&resolved_expr);
        
        match &resolved_expr.kind {
            ExprKind::Function { name, .. } => {
                self.transpile_single_function(&resolved_expr, name, needs_polars)
            }
            ExprKind::Block(exprs) => {
                self.transpile_program_block(exprs, needs_polars)
            }
            _ => {
                self.transpile_expression_program(&resolved_expr, needs_polars)
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
        let (functions, statements, modules, has_main, main_expr) = self.categorize_block_expressions(exprs)?;
        
        if functions.is_empty() && !has_main && modules.is_empty() {
            self.transpile_statement_only_block(exprs, needs_polars)
        } else if has_main || !modules.is_empty() {
            self.transpile_block_with_main_function(&functions, &statements, &modules, main_expr, needs_polars)
        } else {
            self.transpile_block_with_functions(&functions, &statements, needs_polars)
        }
    }
    
    fn categorize_block_expressions<'a>(&self, exprs: &'a [Expr]) -> Result<BlockCategorization<'a>> {
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        let mut modules = Vec::new();
        let mut has_main_function = false;
        let mut main_function_expr = None;
        
        for expr in exprs {
            match &expr.kind {
                ExprKind::Function { name, .. } => {
                    if name == "main" {
                        has_main_function = true;
                        main_function_expr = Some(expr);
                    } else {
                        functions.push(self.transpile_expr(expr)?);
                    }
                },
                ExprKind::Module { name, body } => {
                    // Extract module declarations for top-level placement
                    modules.push(self.transpile_module_declaration(name, body)?);
                },
                ExprKind::Block(block_exprs) => {
                    // Check if this is a module-containing block from the resolver
                    if block_exprs.len() == 2 
                        && matches!(block_exprs[0].kind, ExprKind::Module { .. })
                        && matches!(block_exprs[1].kind, ExprKind::Import { .. }) {
                        // This is a module resolver block: extract the module and keep the import as statement
                        if let ExprKind::Module { name, body } = &block_exprs[0].kind {
                            modules.push(self.transpile_module_declaration(name, body)?);
                        }
                        statements.push(self.transpile_expr(&block_exprs[1])?);
                    } else {
                        // Regular block, treat as statement
                        statements.push(self.transpile_expr(expr)?);
                    }
                },
                _ => {
                    statements.push(self.transpile_expr(expr)?);
                }
            }
        }
        
        Ok((functions, statements, modules, has_main_function, main_function_expr))
    }

    fn transpile_module_declaration(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let module_name = format_ident!("{}", name);
        
        // Handle module body - if it's a block, transpile its contents directly
        let body_tokens = if let ExprKind::Block(exprs) = &body.kind {
            // Transpile each expression in the block as individual items
            let items: Result<Vec<_>> = exprs.iter().map(|expr| self.transpile_expr(expr)).collect();
            let items = items?;
            quote! { #(#items)* }
        } else {
            // Single expression - transpile normally
            self.transpile_expr(body)?
        };

        Ok(quote! {
            mod #module_name {
                #body_tokens
            }
        })
    }
    
    fn transpile_statement_only_block(&self, exprs: &[Expr], needs_polars: bool) -> Result<TokenStream> {
        // Check if this is a statement sequence (contains let, assignments, etc.) or an expression sequence
        let has_statements = exprs.iter().any(|expr| self.is_statement_expr(expr));
        
        if has_statements {
            // Split into statements and possible final expression
            let (statements, final_expr) = if !exprs.is_empty() && !self.is_statement_expr(exprs.last().unwrap()) {
                // Last item is an expression, not a statement
                (&exprs[..exprs.len() - 1], Some(exprs.last().unwrap()))
            } else {
                // All are statements
                (exprs, None)
            };
            
            // Transpile all statements
            let statement_tokens: Result<Vec<_>> = statements.iter().map(|expr| self.transpile_expr(expr)).collect();
            let statement_tokens = statement_tokens?;
            
            // Handle final expression if present
            let main_body = if let Some(final_expr) = final_expr {
                let final_tokens = self.transpile_expr(final_expr)?;
                quote! {
                    #(#statement_tokens;)*
                    let result = #final_tokens;
                    // Use a match on type name to handle strings properly
                    match std::any::type_name_of_val(&result) {
                        name if name.contains("String") || name.contains("&str") => println!("{}", result),
                        _ => println!("{:?}", result)
                    }
                }
            } else {
                quote! {
                    #(#statement_tokens;)*
                }
            };
            
            if needs_polars {
                Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    fn main() {
                        #main_body
                    }
                })
            } else {
                Ok(quote! {
                    use std::collections::HashMap;
                    fn main() {
                        #main_body
                    }
                })
            }
        } else {
            // Pure expression sequence - use existing result printing approach
            let block_expr = Expr::new(ExprKind::Block(exprs.to_vec()), Span::new(0, 0));
            let body = self.transpile_expr(&block_expr)?;
            self.wrap_in_main_with_result_printing(body, needs_polars)
        }
    }
    
    fn is_statement_expr(&self, expr: &Expr) -> bool {
        match &expr.kind {
            // Let bindings are statements
            ExprKind::Let { .. } | ExprKind::LetPattern { .. } => true,
            // Assignment operations are statements  
            ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } => true,
            // Function calls that don't return meaningful values (like println)
            ExprKind::Call { func, .. } => {
                if let ExprKind::Identifier(name) = &func.kind {
                    matches!(name.as_str(), "println" | "print" | "dbg")
                } else {
                    false
                }
            }
            // Blocks containing statements
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.is_statement_expr(e)),
            // Most other expressions are not statements
            _ => false,
        }
    }
    
    fn transpile_block_with_main_function(&self, functions: &[TokenStream], statements: &[TokenStream], modules: &[TokenStream], main_expr: Option<&Expr>, needs_polars: bool) -> Result<TokenStream> {
        if statements.is_empty() && main_expr.is_some() {
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
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                })
            } else {
                Ok(quote! {
                    use std::collections::HashMap;
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                })
            }
        } else {
            // TOP-LEVEL STATEMENTS: Extract main body and combine with statements
            let main_body = if let Some(main) = main_expr {
                self.extract_main_function_body(main)?
            } else {
                // No user main function, just use empty body
                quote! {}
            };
            
            if needs_polars {
                Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    #(#modules)*
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
                    #(#modules)*
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

    /// Check if a name is a Rust reserved keyword
    pub(crate) fn is_rust_reserved_keyword(name: &str) -> bool {
        // List of Rust reserved keywords that would conflict
        matches!(name, 
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" |
            "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" |
            "mod" | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" |
            "static" | "struct" | "super" | "trait" | "true" | "type" | "unsafe" |
            "use" | "where" | "while" | "async" | "await" | "dyn" | "final" | "try" |
            "abstract" | "become" | "box" | "do" | "override" | "priv" | "typeof" |
            "unsized" | "virtual" | "yield"
        )
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
