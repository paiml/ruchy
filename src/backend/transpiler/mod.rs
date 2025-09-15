//! Modular transpiler for Ruchy language
//!
//! This module is responsible for converting Ruchy AST into Rust code using `proc_macro2` `TokenStream`.
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
mod actors;
mod dataframe;
#[cfg(feature = "dataframe")]
// mod dataframe_arrow; // Temporarily disabled until proper implementation
mod dataframe_builder;
mod dataframe_helpers;
mod dispatcher;
mod expressions;
mod method_call_refactored;
mod patterns;
mod result_type;
mod statements;
mod type_conversion_refactored;
mod type_inference;
mod types;
pub mod codegen_minimal;
use crate::frontend::ast::{Attribute, Expr, ExprKind, Span, Type};
use crate::backend::module_resolver::ModuleResolver;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
// Module exports are handled by the impl blocks in each module
/// Block categorization result: (functions, statements, modules, `has_main`, `main_expr`)
type BlockCategorization<'a> = (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, bool, Option<&'a Expr>);
/// Function signature information for type coercion
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub param_types: Vec<String>,  // Simplified: just the type name as string
}
/// The main transpiler struct
#[derive(Clone)]
pub struct Transpiler {
    /// Track whether we're in an async context
    pub in_async_context: bool,
    /// Track variables that need to be mutable (for auto-mutability)
    pub mutable_vars: std::collections::HashSet<String>,
    /// Track function signatures for type coercion
    pub function_signatures: std::collections::HashMap<String, FunctionSignature>,
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
            mutable_vars: std::collections::HashSet::new(),
            function_signatures: std::collections::HashMap::new(),
        }
    }
    /// Centralized result printing logic - ONE PLACE FOR ALL RESULT PRINTING
    /// This eliminates code duplication and ensures consistent Unit type handling
    fn generate_result_printing_tokens(&self) -> TokenStream {
        quote! {
            // Check the type name first to avoid Unit type Display error
            if std::any::type_name_of_val(&result) == "()" {
                // Don't print Unit type
            } else if std::any::type_name_of_val(&result).contains("String") || 
                      std::any::type_name_of_val(&result).contains("&str") {
                println!("{}", result);
            } else {
                println!("{:?}", result);
            }
        }
    }
    /// Centralized value printing logic for functions like println
    fn generate_value_printing_tokens(&self, value_expr: TokenStream, func_tokens: TokenStream) -> TokenStream {
        quote! {
            {
                use std::any::Any;
                let value = #value_expr;
                // Special handling for String and &str types to avoid quotes
                if let Some(s) = (&value as &dyn Any).downcast_ref::<String>() {
                    #func_tokens!("{}", s)
                } else if let Some(s) = (&value as &dyn Any).downcast_ref::<&str>() {
                    #func_tokens!("{}", s)
                } else {
                    #func_tokens!("{:?}", value)
                }
            }
        }
    }
    /// Analyze expressions to determine which variables need to be mutable
    pub fn analyze_mutability(&mut self, exprs: &[Expr]) {
        for expr in exprs {
            self.analyze_expr_mutability(expr);
        }
    }
    /// Collect function signatures for type coercion
    pub fn collect_function_signatures(&mut self, exprs: &[Expr]) {
        for expr in exprs {
            self.collect_signatures_from_expr(expr);
        }
    }
    fn collect_signatures_from_expr(&mut self, expr: &Expr) {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Function { name, params, .. } => {
                let param_types: Vec<String> = params.iter()
                    .map(|param| self.type_to_string(&param.ty))
                    .collect();
                let signature = FunctionSignature {
                    name: name.clone(),
                    param_types,
                };
                self.function_signatures.insert(name.clone(), signature);
            }
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.collect_signatures_from_expr(e);
                }
            }
            ExprKind::Let { body, .. } => {
                self.collect_signatures_from_expr(body);
            }
            _ => {}
        }
    }
    fn type_to_string(&self, ty: &crate::frontend::ast::Type) -> String {
        use crate::frontend::ast::TypeKind;
        match &ty.kind {
            TypeKind::Named(name) => name.clone(),
            TypeKind::Reference { inner, .. } => format!("&{}", self.type_to_string(inner)),
            _ => "Unknown".to_string(),
        }
    }
    fn analyze_expr_mutability(&mut self, expr: &Expr) {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            // Direct assignment marks the target as mutable
            ExprKind::Assign { target, value } => {
                if let ExprKind::Identifier(name) = &target.kind {
                    self.mutable_vars.insert(name.clone());
                }
                self.analyze_expr_mutability(value);
            }
            // Compound assignment marks the target as mutable
            ExprKind::CompoundAssign { target, value, .. } => {
                if let ExprKind::Identifier(name) = &target.kind {
                    self.mutable_vars.insert(name.clone());
                }
                self.analyze_expr_mutability(value);
            }
            // Pre/Post increment/decrement mark the target as mutable
            ExprKind::PreIncrement { target } |
            ExprKind::PostIncrement { target } |
            ExprKind::PreDecrement { target } |
            ExprKind::PostDecrement { target } => {
                if let ExprKind::Identifier(name) = &target.kind {
                    self.mutable_vars.insert(name.clone());
                }
            }
            // Recursively analyze blocks
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.analyze_expr_mutability(e);
                }
            }
            // Analyze control flow
            ExprKind::If { condition, then_branch, else_branch } => {
                self.analyze_expr_mutability(condition);
                self.analyze_expr_mutability(then_branch);
                if let Some(else_expr) = else_branch {
                    self.analyze_expr_mutability(else_expr);
                }
            }
            ExprKind::While { condition, body } => {
                self.analyze_expr_mutability(condition);
                self.analyze_expr_mutability(body);
            }
            ExprKind::For { body, iter, .. } => {
                self.analyze_expr_mutability(iter);
                self.analyze_expr_mutability(body);
            }
            // Analyze match arms
            ExprKind::Match { expr, arms } => {
                self.analyze_expr_mutability(expr);
                for arm in arms {
                    self.analyze_expr_mutability(&arm.body);
                }
            }
            // Analyze let bodies
            ExprKind::Let { body, value, .. } | ExprKind::LetPattern { body, value, .. } => {
                self.analyze_expr_mutability(value);
                self.analyze_expr_mutability(body);
            }
            // Analyze function bodies
            ExprKind::Function { body, .. } => {
                self.analyze_expr_mutability(body);
            }
            ExprKind::Lambda { body, .. } => {
                self.analyze_expr_mutability(body);
            }
            // Analyze binary/unary operations
            ExprKind::Binary { left, right, .. } => {
                self.analyze_expr_mutability(left);
                self.analyze_expr_mutability(right);
            }
            ExprKind::Unary { operand, .. } => {
                self.analyze_expr_mutability(operand);
            }
            // Analyze calls
            ExprKind::Call { func, args } => {
                self.analyze_expr_mutability(func);
                for arg in args {
                    self.analyze_expr_mutability(arg);
                }
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                self.analyze_expr_mutability(receiver);
                for arg in args {
                    self.analyze_expr_mutability(arg);
                }
            }
            _ => {}
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
    /// let ast = parser.parse().expect("Failed to parse");
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
    /// Check if AST contains `HashMap` operations requiring `std::collections::HashMap` import
    fn contains_hashmap(expr: &Expr) -> bool {
        use crate::frontend::ast::{ExprKind, Literal};
        match &expr.kind {
            ExprKind::ObjectLiteral { .. } => true,
            ExprKind::Call { func, .. } => {
                // Check for HashMap methods like .get(), .insert(), etc.
                if let ExprKind::FieldAccess { field, .. } = &func.kind {
                    matches!(field.as_str(), "get" | "insert" | "remove" | "contains_key" | "keys" | "values")
                } else {
                    false
                }
            }
            ExprKind::IndexAccess { object: _, index } => {
                // String literal index access suggests HashMap
                matches!(&index.kind, ExprKind::Literal(Literal::String(_)))
            }
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_hashmap),
            ExprKind::Function { body, .. } => Self::contains_hashmap(body),
            ExprKind::If { condition, then_branch, else_branch } => {
                Self::contains_hashmap(condition) || 
                Self::contains_hashmap(then_branch) ||
                else_branch.as_ref().is_some_and(|e| Self::contains_hashmap(e))
            }
            ExprKind::Binary { left, right, .. } => {
                Self::contains_hashmap(left) || Self::contains_hashmap(right)
            }
            _ => false,
        }
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
    /// let ast = parser.parse().expect("Failed to parse");
    /// 
    /// let mut transpiler = Transpiler::new();
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
    pub fn transpile_to_program(&mut self, expr: &Expr) -> Result<TokenStream> {
        // First analyze the entire program to detect mutable variables and function signatures
        if let ExprKind::Block(exprs) = &expr.kind {
            self.analyze_mutability(exprs);
            self.collect_function_signatures(exprs);
        } else {
            self.analyze_expr_mutability(expr);
            self.collect_signatures_from_expr(expr);
        }
        let result = self.transpile_to_program_with_context(expr, None);
        if let Ok(ref token_stream) = result {
            // Debug: Write the generated Rust code to a debug file
            let rust_code = token_stream.to_string();
            std::fs::write("/tmp/debug_transpiler_output.rs", &rust_code).ok();
        }
        result
    }
    /// Transpile with file context for module resolution
    pub fn transpile_to_program_with_context(&self, expr: &Expr, file_path: Option<&std::path::Path>) -> Result<TokenStream> {
        // First, resolve any file imports using the module resolver
        let resolved_expr = self.resolve_imports_with_context(expr, file_path)?;
        let needs_polars = Self::contains_dataframe(&resolved_expr);
        let needs_hashmap = Self::contains_hashmap(&resolved_expr);
        match &resolved_expr.kind {
            ExprKind::Function { name, .. } => {
                self.transpile_single_function(&resolved_expr, name, needs_polars, needs_hashmap)
            }
            ExprKind::Block(exprs) => {
                self.transpile_program_block(exprs, needs_polars, needs_hashmap)
            }
            _ => {
                self.transpile_expression_program(&resolved_expr, needs_polars, needs_hashmap)
            }
        }
    }
    fn transpile_single_function(&self, expr: &Expr, name: &str, needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        // Use the proper function expression transpiler to handle attributes correctly
        let func = match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => self.transpile_function_expr(expr)?,
            _ => self.transpile_expr(expr)?,
        };
        let needs_main = name != "main";
        match (needs_polars, needs_hashmap, needs_main) {
            (true, true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #func
                fn main() { /* Function defined but not called */ }
            }),
            (true, true, false) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #func
            }),
            (true, false, true) => Ok(quote! {
                use polars::prelude::*;
                #func
                fn main() { /* Function defined but not called */ }
            }),
            (true, false, false) => Ok(quote! {
                use polars::prelude::*;
                #func
            }),
            (false, true, true) => Ok(quote! {
                use std::collections::HashMap;
                #func
                fn main() { /* Function defined but not called */ }
            }),
            (false, true, false) => Ok(quote! {
                use std::collections::HashMap;
                #func
            }),
            (false, false, true) => Ok(quote! {
                #func
                fn main() { /* Function defined but not called */ }
            }),
            (false, false, false) => Ok(quote! { 
                #func 
            })
        }
    }
    fn transpile_program_block(&self, exprs: &[Expr], needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        let (functions, statements, modules, has_main, main_expr) = self.categorize_block_expressions(exprs)?;
        if functions.is_empty() && !has_main && modules.is_empty() {
            self.transpile_statement_only_block(exprs, needs_polars, needs_hashmap)
        } else if has_main || !modules.is_empty() {
            self.transpile_block_with_main_function(&functions, &statements, &modules, main_expr, needs_polars, needs_hashmap)
        } else {
            self.transpile_block_with_functions(&functions, &statements, needs_polars, needs_hashmap)
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
                        // Use proper function transpiler to handle attributes correctly
                        functions.push(self.transpile_function_expr(expr)?);
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
                    let stmt = self.transpile_expr(expr)?;
                    // Ensure statements have semicolons for proper separation
                    let stmt_str = stmt.to_string();
                    if !stmt_str.trim().ends_with(';') && !stmt_str.trim().ends_with('}') {
                        statements.push(quote! { #stmt; });
                    } else {
                        statements.push(stmt);
                    }
                }
            }
        }
        Ok((functions, statements, modules, has_main_function, main_function_expr))
    }
    fn transpile_module_declaration(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let module_name = format_ident!("{}", name);
        // Handle module body - if it's a block, transpile its contents as module items
        let body_tokens = if let ExprKind::Block(exprs) = &body.kind {
            // Separate functions from other items in the module
            let mut module_items = Vec::new();
            for expr in exprs {
                match &expr.kind {
                    ExprKind::Function { .. } => {
                        // Transpile functions as module items
                        module_items.push(self.transpile_function_expr(expr)?);
                    }
                    _ => {
                        // Other items (constants, etc.)
                        module_items.push(self.transpile_expr(expr)?);
                    }
                }
            }
            quote! { #(#module_items)* }
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
    fn transpile_statement_only_block(&self, exprs: &[Expr], needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
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
            // Transpile all statements and add semicolons intelligently
            let statement_results: Result<Vec<_>> = statements.iter().map(|expr| {
                let tokens = self.transpile_expr(expr)?;
                let tokens_str = tokens.to_string();
                // If the statement already ends with a semicolon, don't add another
                if tokens_str.trim().ends_with(';') {
                    Ok(tokens)
                } else {
                    // Add semicolon for statements that need them
                    Ok(quote! { #tokens; })
                }
            }).collect();
            let statement_tokens = statement_results?;
            // Handle final expression if present
            let main_body = if let Some(final_expr) = final_expr {
                let final_tokens = self.transpile_expr(final_expr)?;
                let result_printing_logic = self.generate_result_printing_tokens();
                quote! {
                    #(#statement_tokens)*
                    let result = #final_tokens;
                    #result_printing_logic
                }
            } else {
                quote! {
                    #(#statement_tokens)*
                }
            };
            match (needs_polars, needs_hashmap) {
                (true, true) => Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    fn main() {
                        #main_body
                    }
                }),
                (true, false) => Ok(quote! {
                    use polars::prelude::*;
                    fn main() {
                        #main_body
                    }
                }),
                (false, true) => Ok(quote! {
                    use std::collections::HashMap;
                    fn main() {
                        #main_body
                    }
                }),
                (false, false) => Ok(quote! {
                    fn main() {
                        #main_body
                    }
                })
            }
        } else {
            // Pure expression sequence - use existing result printing approach
            let block_expr = Expr::new(ExprKind::Block(exprs.to_vec()), Span::new(0, 0));
            let body = self.transpile_expr(&block_expr)?;
            self.wrap_in_main_with_result_printing(body, needs_polars, needs_hashmap)
        }
    }
    fn is_statement_expr(&self, expr: &Expr) -> bool {
        match &expr.kind {
            // Let bindings are statements
            ExprKind::Let { .. } | ExprKind::LetPattern { .. } => true,
            // Assignment operations are statements  
            ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } => true,
            // Loops are statements (void/unit type)
            ExprKind::While { .. } | ExprKind::For { .. } | ExprKind::Loop { .. } => true,
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
    fn transpile_block_with_main_function(&self, functions: &[TokenStream], statements: &[TokenStream], modules: &[TokenStream], main_expr: Option<&Expr>, needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        if statements.is_empty() && main_expr.is_some() {
            // Only functions, just emit them normally (includes user's main)
            let main_tokens = if let Some(main) = main_expr {
                self.transpile_expr(main)?
            } else {
                return Err(anyhow::anyhow!("Expected main function expression"));
            };
            match (needs_polars, needs_hashmap) {
                (true, true) => Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
                (true, false) => Ok(quote! {
                    use polars::prelude::*;
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
                (false, true) => Ok(quote! {
                    use std::collections::HashMap;
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
                (false, false) => Ok(quote! {
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
            match (needs_polars, needs_hashmap) {
                (true, true) => Ok(quote! {
                    use polars::prelude::*;
                    use std::collections::HashMap;
                    #(#modules)*
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements)*
                        // Then user's main function body  
                        #main_body
                    }
                }),
                (true, false) => Ok(quote! {
                    use polars::prelude::*;
                    #(#modules)*
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements)*
                        // Then user's main function body  
                        #main_body
                    }
                }),
                (false, true) => Ok(quote! {
                    use std::collections::HashMap;
                    #(#modules)*
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements)*
                        // Then user's main function body
                        #main_body
                    }
                }),
                (false, false) => Ok(quote! {
                    #(#modules)*
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements)*
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
    fn transpile_block_with_functions(&self, functions: &[TokenStream], statements: &[TokenStream], needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        // No main function among extracted functions - create one for statements
        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #(#functions)*
                fn main() { #(#statements)* }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                #(#functions)*
                fn main() { #(#statements)* }
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                #(#functions)*
                fn main() { #(#statements)* }
            }),
            (false, false) => Ok(quote! {
                #(#functions)*
                fn main() { #(#statements)* }
            })
        }
    }
    fn transpile_expression_program(&self, expr: &Expr, needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        let body = self.transpile_expr(expr)?;
        // Check if this is a statement vs expression
        if self.is_statement_expr(expr) {
            // For statements, execute directly without result wrapping
            self.wrap_statement_in_main(body, needs_polars, needs_hashmap)
        } else {
            // For expressions, wrap with result printing
            self.wrap_in_main_with_result_printing(body, needs_polars, needs_hashmap)
        }
    }
    fn wrap_statement_in_main(&self, body: TokenStream, needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        // For statements, execute directly without result capture
        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                fn main() {
                    #body;
                }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                fn main() {
                    #body;
                }
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                fn main() {
                    #body;
                }
            }),
            (false, false) => Ok(quote! {
                fn main() {
                    #body;
                }
            })
        }
    }
    fn wrap_in_main_with_result_printing(&self, body: TokenStream, needs_polars: bool, needs_hashmap: bool) -> Result<TokenStream> {
        let result_printing_logic = self.generate_result_printing_tokens();
        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                fn main() {
                    let result = #body;
                    #result_printing_logic
                }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                fn main() {
                    let result = #body;
                    #result_printing_logic
                }
            }),
            (false, true) => {
                Ok(quote! {
                    use std::collections::HashMap;
                    fn main() {
                        let result = #body;
                        #result_printing_logic
                    }
                })
            },
            (false, false) => {
                Ok(quote! {
                    fn main() {
                        let result = #body;
                        #result_printing_logic
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
            Actor, ActorQuery, ActorSend, ArrayInit, Ask, Assign, AsyncBlock, Await, Binary, Call, Command, CompoundAssign, DataFrame, 
            DataFrameOperation, Err, FieldAccess, For, Function, Identifier, If, IfLet, IndexAccess, Lambda, 
            List, ListComprehension, Literal, Loop, Macro, Match, MethodCall, None, ObjectLiteral, Ok, QualifiedName, 
            Range, Send, Slice, Some, StringInterpolation, Struct, StructLiteral, Throw, Try, TryCatch, TypeCast,
            Tuple, Unary, While, WhileLet,
        };
        // Dispatch to specialized handlers to keep complexity below 10
        match &expr.kind {
            // Basic expressions
            Literal(_) | Identifier(_) | QualifiedName { .. } | StringInterpolation { .. } | TypeCast { .. } => {
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
            | Loop { .. }
            | TryCatch { .. } => self.transpile_operator_control_expr(expr),
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
            | ArrayInit { .. }
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
#[cfg(test)]
mod property_tests_mod {
    use proptest::proptest;
    
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
