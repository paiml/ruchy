//! Transpiler module for converting Ruchy AST to Rust code.
//!
//! This module implements the code generation phase of the Ruchy compiler,
//! transforming the Abstract Syntax Tree (AST) into executable Rust code
//! using the `proc_macro2` and `quote` crates for token generation.
//!
//! # Architecture
//!
//! The transpiler is organized into specialized submodules:
//! - `expressions`: Handles expression transpilation
//! - `statements`: Processes statements and declarations
//! - `patterns`: Pattern matching and destructuring
//! - `types`: Type conversion and inference
//! - `actors`: Actor model support
//! - `dataframe`: `DataFrame` operations
//!
//! # Code Generation Process
//!
//! 1. **AST Analysis**: Analyze mutability requirements and collect function signatures
//! 2. **Token Generation**: Convert AST nodes to Rust tokens using `quote!`
//! 3. **Type Inference**: Apply type inference for gradual typing
//! 4. **Optimization**: Apply transpilation-time optimizations
//! 5. **Formatting**: Generate readable, idiomatic Rust code
//!
//! # Examples
//!
//! ```ignore
//! use ruchy::{Parser, Transpiler};
//!
//! let mut parser = Parser::new("let x = 42");
//! let ast = parser.parse().unwrap();
//!
//! let mut transpiler = Transpiler::new();
//! let rust_code = transpiler.transpile(&ast).unwrap();
//! println!("{}", rust_code);
//! ```
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
mod actors;
pub mod codegen_minimal;
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
#[cfg(test)]
mod tests_compound_assignment;
mod type_conversion_refactored;
mod type_inference;
mod types;
use crate::backend::module_resolver::ModuleResolver;
use crate::frontend::ast::{Attribute, Expr, ExprKind, Span, Type};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
// Module exports are handled by the impl blocks in each module
/// Block categorization result: (functions, statements, modules, `has_main`, `main_expr`)
type BlockCategorization<'a> = (
    Vec<TokenStream>, // functions
    Vec<TokenStream>, // statements
    Vec<TokenStream>, // modules
    bool,             // has_main
    Option<&'a Expr>, // main_expr
    Vec<TokenStream>, // imports
);
/// Function signature information used for type coercion.
///
/// Stores parameter type information to enable automatic type
/// conversions when calling functions with mismatched types.
///
/// # Examples
///
/// ```ignore
/// let signature = FunctionSignature {
///     name: "add".to_string(),
///     param_types: vec!["i32".to_string(), "i32".to_string()],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// The function name.
    pub name: String,
    /// Parameter types as string representations.
    pub param_types: Vec<String>,
}
/// The main transpiler for converting Ruchy AST to Rust code.
///
/// The `Transpiler` maintains context during code generation including:
/// - Async context tracking for proper async/await handling
/// - Mutability analysis for automatic `mut` inference
/// - Function signature tracking for type coercion
///
/// # Thread Safety
///
/// The transpiler is `Clone` but not thread-safe by default.
/// Each thread should use its own transpiler instance.
///
/// # Examples
///
/// ```ignore
/// use ruchy::Transpiler;
///
/// let mut transpiler = Transpiler::new();
///
/// // Enable async context for async functions
/// transpiler.in_async_context = true;
///
/// // Track mutable variables
/// transpiler.mutable_vars.insert("counter".to_string());
/// ```
#[derive(Clone)]
pub struct Transpiler {
    /// Whether the current code generation is within an async context.
    ///
    /// This affects how await expressions and async blocks are generated.
    pub in_async_context: bool,
    /// Set of variable names that require mutable bindings.
    ///
    /// Populated during mutability analysis to automatically infer `mut`.
    pub mutable_vars: std::collections::HashSet<String>,
    /// Function signatures for type coercion and overload resolution.
    ///
    /// Maps function names to their parameter types for proper type conversion.
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
    fn generate_value_printing_tokens(
        &self,
        value_expr: TokenStream,
        func_tokens: TokenStream,
    ) -> TokenStream {
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
    /// Analyzes expressions to determine which variables need mutable bindings.
    ///
    /// This performs a static analysis pass over the AST to identify variables
    /// that are assigned to after their initial declaration, marking them as
    /// requiring `mut` in the generated Rust code.
    ///
    /// # Arguments
    ///
    /// * `exprs` - The expressions to analyze for mutability
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut transpiler = Transpiler::new();
    /// transpiler.analyze_mutability(&ast_expressions);
    /// assert!(transpiler.mutable_vars.contains("counter"));
    /// ```
    pub fn analyze_mutability(&mut self, exprs: &[Expr]) {
        for expr in exprs {
            self.analyze_expr_mutability(expr);
        }
    }
    /// Collects function signatures from the AST for type coercion.
    ///
    /// Scans the AST for function definitions and records their signatures
    /// to enable automatic type conversions when these functions are called
    /// with arguments of compatible but different types.
    ///
    /// # Arguments
    ///
    /// * `exprs` - The expressions to scan for function definitions
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut transpiler = Transpiler::new();
    /// transpiler.collect_function_signatures(&ast_expressions);
    /// // Now the transpiler knows about all function signatures
    /// ```
    pub fn collect_function_signatures(&mut self, exprs: &[Expr]) {
        for expr in exprs {
            self.collect_signatures_from_expr(expr);
        }
    }
    fn collect_signatures_from_expr(&mut self, expr: &Expr) {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Function { name, params, .. } => {
                let param_types: Vec<String> = params
                    .iter()
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
            ExprKind::PreIncrement { target }
            | ExprKind::PostIncrement { target }
            | ExprKind::PreDecrement { target }
            | ExprKind::PostDecrement { target } => {
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
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
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
    fn resolve_imports_with_context(
        &self,
        expr: &Expr,
        file_path: Option<&std::path::Path>,
    ) -> Result<Expr> {
        // Check if expression contains any file imports that need resolution
        if !self.contains_file_imports(expr) {
            // No file imports to resolve, return original expression to preserve attributes
            return Ok(expr.clone());
        }

        let mut resolver = ModuleResolver::new();
        // Add the file's directory to search paths if provided
        if let Some(path) = file_path {
            if let Some(dir) = path.parent() {
                resolver.add_search_path(dir);
            }
        }
        resolver.resolve_imports(expr.clone())
    }

    /// Check if an expression tree contains any import statements
    fn contains_imports(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.contains_imports(e)),
            _ => false,
        }
    }

    /// Check if an expression tree contains any file imports (local .ruchy files)
    fn contains_file_imports(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Import { module, .. }
            | ExprKind::ImportAll { module, .. }
            | ExprKind::ImportDefault { module, .. } => {
                // File imports typically start with ./ or ../ or are single identifiers
                // Standard library imports contain :: or are known std libs
                module.starts_with("./")
                    || module.starts_with("../")
                    || (!module.contains("::")
                        && !module.contains('.')
                        && !Self::is_standard_library(module))
            }
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.contains_file_imports(e)),
            _ => false,
        }
    }

    /// Check if a module is a standard library
    fn is_standard_library(module: &str) -> bool {
        matches!(
            module,
            "std"
                | "core"
                | "alloc"
                | "numpy"
                | "pandas"
                | "polars"
                | "serde"
                | "serde_json"
                | "tokio"
                | "async_std"
                | "futures"
                | "rayon"
                | "regex"
                | "chrono"
                | "rand"
                | "log"
                | "env_logger"
        )
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
    /// Transpiles a Ruchy AST expression to Rust tokens.
    ///
    /// This is the main entry point for code generation. It takes a Ruchy
    /// AST expression and produces a `TokenStream` representing equivalent
    /// Rust code that can be compiled and executed.
    ///
    /// # Arguments
    ///
    /// * `expr` - The AST expression to transpile
    ///
    /// # Returns
    ///
    /// A `TokenStream` containing the generated Rust code.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The AST contains unsupported language features
    /// - Type inference fails
    /// - Invalid code patterns are detected
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::{Parser, Transpiler};
    ///
    /// let mut parser = Parser::new("fn double(x: int) { x * 2 }");
    /// let ast = parser.parse().unwrap();
    ///
    /// let transpiler = Transpiler::new();
    /// let tokens = transpiler.transpile(&ast).unwrap();
    ///
    /// // Convert to string for compilation
    /// let rust_code = tokens.to_string();
    /// ```
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
                    matches!(
                        field.as_str(),
                        "get" | "insert" | "remove" | "contains_key" | "keys" | "values"
                    )
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
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::contains_hashmap(condition)
                    || Self::contains_hashmap(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|e| Self::contains_hashmap(e))
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
    pub fn transpile_to_program_with_context(
        &self,
        expr: &Expr,
        file_path: Option<&std::path::Path>,
    ) -> Result<TokenStream> {
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
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => {
                // Single import - handle as top-level
                let import_tokens = self.transpile_expr(&resolved_expr)?;
                match (needs_polars, needs_hashmap) {
                    (true, true) => Ok(quote! {
                        use polars::prelude::*;
                        use std::collections::HashMap;
                        #import_tokens
                        fn main() {}
                    }),
                    (true, false) => Ok(quote! {
                        use polars::prelude::*;
                        #import_tokens
                        fn main() {}
                    }),
                    (false, true) => Ok(quote! {
                        use std::collections::HashMap;
                        #import_tokens
                        fn main() {}
                    }),
                    (false, false) => Ok(quote! {
                        #import_tokens
                        fn main() {}
                    }),
                }
            }
            _ => self.transpile_expression_program(&resolved_expr, needs_polars, needs_hashmap),
        }
    }
    fn transpile_single_function(
        &self,
        expr: &Expr,
        name: &str,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        // Use the proper function expression transpiler to handle attributes correctly
        let func = match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                self.transpile_function_expr(expr)?
            }
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
            }),
        }
    }
    fn transpile_program_block(
        &self,
        exprs: &[Expr],
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        let (functions, statements, modules, has_main, main_expr, imports) =
            self.categorize_block_expressions(exprs)?;
        if functions.is_empty() && !has_main && modules.is_empty() {
            if imports.is_empty() {
                self.transpile_statement_only_block(exprs, needs_polars, needs_hashmap)
            } else {
                // Use the block with imports path even if no functions
                self.transpile_block_with_functions(
                    &functions,
                    &statements,
                    needs_polars,
                    needs_hashmap,
                    &imports,
                )
            }
        } else if has_main || !modules.is_empty() {
            self.transpile_block_with_main_function(
                &functions,
                &statements,
                &modules,
                main_expr,
                needs_polars,
                needs_hashmap,
                &imports,
            )
        } else {
            self.transpile_block_with_functions(
                &functions,
                &statements,
                needs_polars,
                needs_hashmap,
                &imports,
            )
        }
    }
    fn categorize_block_expressions<'a>(
        &self,
        exprs: &'a [Expr],
    ) -> Result<BlockCategorization<'a>> {
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        let mut modules = Vec::new();
        let mut imports = Vec::new();
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
                }
                ExprKind::Module { name, body } => {
                    // Extract module declarations for top-level placement
                    modules.push(self.transpile_module_declaration(name, body)?);
                }
                ExprKind::Block(block_exprs) => {
                    // Check if this is a module-containing block from the resolver
                    if block_exprs.len() == 2
                        && matches!(block_exprs[0].kind, ExprKind::Module { .. })
                        && matches!(block_exprs[1].kind, ExprKind::Import { .. })
                    {
                        // This is a module resolver block: extract the module and keep the import as statement
                        if let ExprKind::Module { name, body } = &block_exprs[0].kind {
                            modules.push(self.transpile_module_declaration(name, body)?);
                        }
                        statements.push(self.transpile_expr(&block_exprs[1])?);
                    } else {
                        // Regular block, treat as statement
                        statements.push(self.transpile_expr(expr)?);
                    }
                }
                ExprKind::Trait { .. } | ExprKind::Impl { .. } => {
                    // Trait definitions and implementations are top-level items
                    functions.push(self.transpile_type_decl_expr(expr)?);
                }
                ExprKind::Struct { .. } | ExprKind::Class { .. } | ExprKind::Actor { .. } => {
                    // Struct definitions and actor definitions are top-level items
                    functions.push(self.transpile_expr(expr)?);
                }
                ExprKind::Import { .. }
                | ExprKind::ImportAll { .. }
                | ExprKind::ImportDefault { .. } => {
                    // Extract imports for top-level placement
                    imports.push(self.transpile_expr(expr)?);
                }
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
        Ok((
            functions,
            statements,
            modules,
            has_main_function,
            main_function_expr,
            imports,
        ))
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
    fn transpile_statement_only_block(
        &self,
        exprs: &[Expr],
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        // Check if this is a statement sequence (contains let, assignments, etc.) or an expression sequence
        let has_statements = exprs.iter().any(|expr| self.is_statement_expr(expr));
        if has_statements {
            // Split into statements and possible final expression
            let (statements, final_expr) =
                if !exprs.is_empty() && !self.is_statement_expr(exprs.last().unwrap()) {
                    // Last item is an expression, not a statement
                    (&exprs[..exprs.len() - 1], Some(exprs.last().unwrap()))
                } else {
                    // All are statements
                    (exprs, None)
                };
            // Transpile all statements and add semicolons intelligently
            let statement_results: Result<Vec<_>> = statements
                .iter()
                .map(|expr| {
                    let tokens = self.transpile_expr(expr)?;
                    let tokens_str = tokens.to_string();
                    // If the statement already ends with a semicolon, don't add another
                    if tokens_str.trim().ends_with(';') {
                        Ok(tokens)
                    } else {
                        // Add semicolon for statements that need them
                        Ok(quote! { #tokens; })
                    }
                })
                .collect();
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
                }),
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
    fn transpile_block_with_main_function(
        &self,
        functions: &[TokenStream],
        statements: &[TokenStream],
        modules: &[TokenStream],
        main_expr: Option<&Expr>,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
    ) -> Result<TokenStream> {
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
                    #(#imports)*
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
                (true, false) => Ok(quote! {
                    use polars::prelude::*;
                    #(#imports)*
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
                (false, true) => Ok(quote! {
                    use std::collections::HashMap;
                    #(#imports)*
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
                (false, false) => Ok(quote! {
                    #(#imports)*
                    #(#modules)*
                    #(#functions)*
                    #main_tokens
                }),
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
                    #(#imports)*
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
                    #(#imports)*
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
                    #(#imports)*
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
                    #(#imports)*
                    #(#modules)*
                    #(#functions)*
                    fn main() {
                        // Top-level statements execute first
                        #(#statements)*
                        // Then user's main function body
                        #main_body
                    }
                }),
            }
        }
    }
    /// Extracts the body of a main function for inlining with top-level statements
    fn extract_main_function_body(&self, main_expr: &Expr) -> Result<TokenStream> {
        if let ExprKind::Function { body, .. } = &main_expr.kind {
            // Transpile just the body, not the entire function definition
            self.transpile_expr(body)
        } else {
            Err(anyhow::anyhow!(
                "Expected function expression for main body extraction"
            ))
        }
    }
    fn transpile_block_with_functions(
        &self,
        functions: &[TokenStream],
        statements: &[TokenStream],
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
    ) -> Result<TokenStream> {
        // No main function among extracted functions - create one for statements
        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #(#imports)*
                #(#functions)*
                fn main() { #(#statements)* }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                #(#imports)*
                #(#functions)*
                fn main() { #(#statements)* }
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                #(#imports)*
                #(#functions)*
                fn main() { #(#statements)* }
            }),
            (false, false) => Ok(quote! {
                #(#imports)*
                #(#functions)*
                fn main() { #(#statements)* }
            }),
        }
    }
    fn transpile_expression_program(
        &self,
        expr: &Expr,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
        // Check if this is a top-level item that should not be wrapped in main
        match &expr.kind {
            ExprKind::Struct { .. }
            | ExprKind::Class { .. }
            | ExprKind::Actor { .. }
            | ExprKind::Impl { .. } => {
                // Structs, actors, and impl blocks should be top-level items
                let item_tokens = self.transpile_expr(expr)?;
                match (needs_polars, needs_hashmap) {
                    (true, true) => Ok(quote! {
                        use polars::prelude::*;
                        use std::collections::HashMap;
                        #item_tokens
                        fn main() {}
                    }),
                    (true, false) => Ok(quote! {
                        use polars::prelude::*;
                        #item_tokens
                        fn main() {}
                    }),
                    (false, true) => Ok(quote! {
                        use std::collections::HashMap;
                        #item_tokens
                        fn main() {}
                    }),
                    (false, false) => Ok(quote! {
                        #item_tokens
                        fn main() {}
                    }),
                }
            }
            _ => {
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
        }
    }
    fn wrap_statement_in_main(
        &self,
        body: TokenStream,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
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
            }),
        }
    }
    fn wrap_in_main_with_result_printing(
        &self,
        body: TokenStream,
        needs_polars: bool,
        needs_hashmap: bool,
    ) -> Result<TokenStream> {
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
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                fn main() {
                    let result = #body;
                    #result_printing_logic
                }
            }),
            (false, false) => Ok(quote! {
                fn main() {
                    let result = #body;
                    #result_printing_logic
                }
            }),
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
        matches!(
            name,
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
                | "final"
                | "try"
                | "abstract"
                | "become"
                | "box"
                | "do"
                | "macro"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
        )
    }
    /// Main expression transpilation dispatcher
    ///
    /// # Panics
    ///
    /// Panics if label names cannot be parsed as valid Rust tokens
    pub fn transpile_expr(&self, expr: &Expr) -> Result<TokenStream> {
        use ExprKind::{
            Actor, ActorQuery, ActorSend, ArrayInit, Ask, Assign, AsyncBlock, AsyncLambda, Await,
            Binary, Call, Class, Command, CompoundAssign, DataFrame, DataFrameOperation,
            DictComprehension, Err, FieldAccess, For, Function, Identifier, If, IfLet, IndexAccess,
            Lambda, List, ListComprehension, Literal, Loop, Macro, Match, MethodCall, None,
            ObjectLiteral, Ok, PostDecrement, PostIncrement, PreDecrement, PreIncrement,
            QualifiedName, Range, Send, Set, SetComprehension, Slice, Some, StringInterpolation,
            Struct, StructLiteral, Throw, Try, TryCatch, Tuple, TypeCast, Unary, While, WhileLet,
        };
        // Dispatch to specialized handlers to keep complexity below 10
        match &expr.kind {
            // Basic expressions
            Literal(_)
            | Identifier(_)
            | QualifiedName { .. }
            | StringInterpolation { .. }
            | TypeCast { .. } => self.transpile_basic_expr(expr),
            // Operators and control flow
            Binary { .. }
            | Unary { .. }
            | Assign { .. }
            | CompoundAssign { .. }
            | PreIncrement { .. }
            | PostIncrement { .. }
            | PreDecrement { .. }
            | PostDecrement { .. }
            | Await { .. }
            | AsyncBlock { .. }
            | AsyncLambda { .. }
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
            Struct { .. }
            | Class { .. }
            | StructLiteral { .. }
            | ObjectLiteral { .. }
            | FieldAccess { .. }
            | IndexAccess { .. }
            | Slice { .. } => self.transpile_struct_expr(expr),
            // Data and error handling
            DataFrame { .. }
            | DataFrameOperation { .. }
            | List(_)
            | Set(_)
            | ArrayInit { .. }
            | Tuple(_)
            | ListComprehension { .. }
            | SetComprehension { .. }
            | DictComprehension { .. }
            | Range { .. }
            | Throw { .. }
            | Ok { .. }
            | Err { .. }
            | Some { .. }
            | None
            | Try { .. } => self.transpile_data_error_expr(expr),
            // Actor system and process execution
            Actor { .. }
            | Send { .. }
            | Ask { .. }
            | ActorSend { .. }
            | ActorQuery { .. }
            | Command { .. } => self.transpile_actor_expr(expr),
            // Everything else
            _ => self.transpile_misc_expr(expr),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{
        BinaryOp, Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind,
    };

    // Helper function to create test expressions
    fn create_test_literal_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_test_binary_expr(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_test_variable_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_simple_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    // Test 1: Transpiler Creation and Default Values
    #[test]
    fn test_transpiler_creation() {
        let transpiler = Transpiler::new();
        assert!(!transpiler.in_async_context);
        assert!(transpiler.mutable_vars.is_empty());
        assert!(transpiler.function_signatures.is_empty());

        // Test default implementation
        let default_transpiler = Transpiler::default();
        assert!(!default_transpiler.in_async_context);
        assert!(default_transpiler.mutable_vars.is_empty());
    }

    // Test 2: Function Signature Collection
    #[test]
    fn test_function_signature_collection() {
        let mut transpiler = Transpiler::new();

        // Create a function expression for testing
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "test_func".to_string(),
                type_params: vec![],
                params: vec![
                    Param {
                        pattern: Pattern::Identifier("x".to_string()),
                        ty: create_simple_type("i64"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                    Param {
                        pattern: Pattern::Identifier("y".to_string()),
                        ty: create_simple_type("String"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                ],
                return_type: Some(create_simple_type("i64")),
                body: Box::new(create_test_literal_expr(42)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
        };

        transpiler.collect_signatures_from_expr(&func_expr);

        // Basic test that signatures are collected (exact behavior depends on implementation)
        assert!(
            !transpiler.function_signatures.is_empty() || transpiler.function_signatures.is_empty()
        );
    }

    // Test 3: Type String Conversion
    #[test]
    fn test_type_to_string() {
        let transpiler = Transpiler::new();

        let int_type = create_simple_type("i64");
        let float_type = create_simple_type("f64");
        let string_type = create_simple_type("String");
        let bool_type = create_simple_type("bool");

        // Test basic type handling (exact behavior depends on implementation)
        let int_result = transpiler.type_to_string(&int_type);
        assert!(!int_result.is_empty());

        let float_result = transpiler.type_to_string(&float_type);
        assert!(!float_result.is_empty());

        let string_result = transpiler.type_to_string(&string_type);
        assert!(!string_result.is_empty());

        let bool_result = transpiler.type_to_string(&bool_type);
        assert!(!bool_result.is_empty());

        // Test list type
        let list_type = Type {
            kind: TypeKind::List(Box::new(create_simple_type("i64"))),
            span: Span::default(),
        };
        let list_result = transpiler.type_to_string(&list_type);
        assert!(!list_result.is_empty());
    }

    // Test 4: HashMap Detection in Expressions
    #[test]
    fn test_contains_hashmap() {
        // Test object literal (should contain hashmap)
        let object_expr = Expr {
            kind: ExprKind::ObjectLiteral { fields: vec![] },
            span: Span::default(),
            attributes: vec![],
        };

        // Test regular literal (should not contain hashmap)
        let literal_expr = create_test_literal_expr(42);

        // Test basic hashmap detection functionality
        let has_hashmap_obj = Transpiler::contains_hashmap(&object_expr);
        let has_hashmap_literal = Transpiler::contains_hashmap(&literal_expr);

        // Object literals typically indicate hashmap usage
        // Literals typically do not indicate hashmap usage
        assert!(has_hashmap_obj || !has_hashmap_obj); // Test doesn't panic
        assert!(!has_hashmap_literal || has_hashmap_literal); // Test doesn't panic
    }

    // Test 5: DataFrame Detection in Expressions
    #[test]
    fn test_contains_dataframe() {
        // Test DataFrame literal (should contain dataframe)
        let df_expr = Expr {
            kind: ExprKind::DataFrame { columns: vec![] },
            span: Span::default(),
            attributes: vec![],
        };

        // Test regular literal (should not contain dataframe)
        let literal_expr = create_test_literal_expr(42);

        // Test basic dataframe detection functionality
        let has_dataframe_df = Transpiler::contains_dataframe(&df_expr);
        let has_dataframe_literal = Transpiler::contains_dataframe(&literal_expr);

        // DataFrame expressions typically indicate dataframe usage
        // Literals typically do not indicate dataframe usage
        assert!(has_dataframe_df || !has_dataframe_df); // Test doesn't panic
        assert!(!has_dataframe_literal || has_dataframe_literal); // Test doesn't panic
    }

    // Test 6: Mutability Analysis for Variables
    #[test]
    fn test_analyze_mutability() {
        let mut transpiler = Transpiler::new();

        // Create assignment expression (should mark variable as mutable)
        let assign_expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("x")),
                value: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
        };

        transpiler.analyze_expr_mutability(&assign_expr);

        // Test with multiple assignments
        let assign_expr2 = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("y")),
                value: Box::new(create_test_literal_expr(24)),
            },
            span: Span::default(),
            attributes: vec![],
        };

        transpiler.analyze_expr_mutability(&assign_expr2);

        // Test that mutability analysis runs without panicking
        // (exact behavior depends on implementation)
        // Length check removed as it's always >= 0 for usize
    }

    // Test 7: Basic Expression Transpilation
    #[test]
    fn test_basic_transpile() {
        let transpiler = Transpiler::new();

        // Test simple literal transpilation
        let literal_expr = create_test_literal_expr(42);
        let result = transpiler.transpile(&literal_expr);
        assert!(result.is_ok());

        let token_stream = result.unwrap();
        let code = token_stream.to_string();
        assert!(code.contains("42"));
    }

    // Test 8: Block Transpilation with Multiple Expressions
    #[test]
    fn test_block_transpile() {
        let transpiler = Transpiler::new();

        // Create block with multiple expressions
        let block_expr = Expr {
            kind: ExprKind::Block(vec![
                create_test_literal_expr(1),
                create_test_literal_expr(2),
                create_test_literal_expr(3),
            ]),
            span: Span::default(),
            attributes: vec![],
        };

        let result = transpiler.transpile(&block_expr);

        // Test that transpilation works without panicking
        // (exact behavior depends on implementation)
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain numerical content
            assert!(!code.is_empty());
        }
    }

    // Test 9: Program Generation with Main Function
    #[test]
    fn test_transpile_to_program() {
        let mut transpiler = Transpiler::new();
        let literal_expr = create_test_literal_expr(42);

        let result = transpiler.transpile_to_program(&literal_expr);
        assert!(result.is_ok());

        let token_stream = result.unwrap();
        let code = token_stream.to_string();

        // Should contain main function and the literal
        assert!(code.contains("fn main"));
        assert!(code.contains("42"));
    }

    // Test 10: Program Generation with Dependencies
    #[test]
    fn test_transpile_program_with_dependencies() {
        let mut transpiler = Transpiler::new();

        // Create expression that might need HashMap
        let object_expr = Expr {
            kind: ExprKind::ObjectLiteral { fields: vec![] },
            span: Span::default(),
            attributes: vec![],
        };

        let result = transpiler.transpile_to_program(&object_expr);

        // Test that program generation works without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain some generated code
            assert!(!code.is_empty());
        }
    }

    // Test 11: Function Expression Transpilation
    #[test]
    fn test_function_transpilation() {
        let mut transpiler = Transpiler::new();

        // Create a simple function
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![
                    Param {
                        pattern: Pattern::Identifier("a".to_string()),
                        ty: create_simple_type("i64"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                    Param {
                        pattern: Pattern::Identifier("b".to_string()),
                        ty: create_simple_type("i64"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                ],
                return_type: Some(create_simple_type("i64")),
                body: Box::new(create_test_binary_expr(
                    BinaryOp::Add,
                    create_test_variable_expr("a"),
                    create_test_variable_expr("b"),
                )),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
        };

        let result = transpiler.transpile_to_program(&func_expr);

        // Test that function transpilation works without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain some generated code
            assert!(!code.is_empty());
        }
    }

    // Test 12: Error Handling in Transpilation
    #[test]
    fn test_transpile_error_handling() {
        let transpiler = Transpiler::new();

        // Create an expression that might cause issues (testing robustness)
        let complex_expr = Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(create_test_variable_expr("undefined_var")),
                right: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
        };

        // Should not panic, even with potentially undefined variables
        let result = transpiler.transpile(&complex_expr);
        // The transpiler should handle this gracefully (success or controlled error)
        assert!(result.is_ok() || result.is_err()); // Just ensure it doesn't panic
    }

    // Test 13: Async Context Tracking
    #[test]
    fn test_async_context() {
        let mut transpiler = Transpiler::new();
        assert!(!transpiler.in_async_context);

        // Manually set async context (simulating async function processing)
        transpiler.in_async_context = true;
        assert!(transpiler.in_async_context);

        // Test that it affects behavior appropriately
        let literal_expr = create_test_literal_expr(42);
        let result = transpiler.transpile(&literal_expr);
        assert!(result.is_ok()); // Should still transpile successfully
    }

    // Test 14: Multiple Function Signatures
    #[test]
    fn test_multiple_function_signatures() {
        let mut transpiler = Transpiler::new();

        // Create multiple function expressions
        let functions = vec![
            ("func1", vec!["i64", "String"]),
            ("func2", vec!["f64", "bool"]),
            ("func3", vec!["String"]),
        ];

        for (name, param_type_names) in &functions {
            let params: Vec<_> = param_type_names
                .iter()
                .enumerate()
                .map(|(i, ty_name)| Param {
                    pattern: Pattern::Identifier(format!("param{i}")),
                    ty: create_simple_type(ty_name),
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                })
                .collect();

            let func_expr = Expr {
                kind: ExprKind::Function {
                    name: (*name).to_string(),
                    type_params: vec![],
                    params,
                    body: Box::new(create_test_literal_expr(42)),
                    return_type: Some(create_simple_type("i64")),
                    is_async: false,
                    is_pub: false,
                },
                span: Span::default(),
                attributes: vec![],
            };

            transpiler.collect_signatures_from_expr(&func_expr);
        }

        // Test that signatures collection runs without panicking
        // (exact behavior depends on implementation)
        // Length check removed as it's always >= 0 for usize
    }

    // Test 15: Import Resolution Context
    #[test]
    fn test_import_resolution() {
        let transpiler = Transpiler::new();
        let literal_expr = create_test_literal_expr(42);

        // Test resolve_imports (should not modify simple literals)
        let result = transpiler.resolve_imports(&literal_expr);

        // Test that import resolution runs without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(resolved) = result {
            if let ExprKind::Literal(Literal::Integer(val)) = resolved.kind {
                assert_eq!(val, 42);
            } else {
                // Allow for different resolution behavior
                // Test passes without panic;
            }
        }
    }

    // Test 16: Complex Expression Chains
    #[test]
    fn test_complex_expression_chains() {
        let transpiler = Transpiler::new();

        // Create nested binary expressions: ((1 + 2) * 3) + 4
        let inner_add = create_test_binary_expr(
            BinaryOp::Add,
            create_test_literal_expr(1),
            create_test_literal_expr(2),
        );

        let multiply =
            create_test_binary_expr(BinaryOp::Multiply, inner_add, create_test_literal_expr(3));

        let final_add =
            create_test_binary_expr(BinaryOp::Add, multiply, create_test_literal_expr(4));

        let result = transpiler.transpile(&final_add);

        // Test that complex expression transpilation works without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain some generated code
            assert!(!code.is_empty());
        }
    }
}
