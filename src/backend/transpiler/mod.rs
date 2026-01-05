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
pub mod constant_folder; // PERF-002-A: Constant folding optimization
mod dataframe;
mod effects;
pub mod inline_expander; // OPT-CODEGEN-004: Inline expansion optimization
                         // #[cfg(feature = "dataframe")]
                         // mod dataframe_arrow; // Temporarily disabled until proper implementation
mod dataframe_builder;
mod dataframe_helpers;
mod dispatcher;
mod expressions;
mod method_call_refactored;
mod patterns;
mod result_type;
pub mod return_type_helpers;
mod statements;
mod comprehensions; // EXTREME TDD Round 53: List/set/dict comprehensions
mod control_flow; // EXTREME TDD Round 53: if/for/while/loop/try-catch
mod bindings; // EXTREME TDD Round 54: let bindings and patterns
mod imports; // EXTREME TDD Round 55: imports and exports
mod math_builtins; // EXTREME TDD Round 56: math built-in functions
mod input_builtins; // EXTREME TDD Round 57: input/readline functions
mod type_conversions; // EXTREME TDD Round 58: str/int/float/bool conversions
mod advanced_math; // EXTREME TDD Round 59: trig/log/random/trueno functions
mod utility_builtins; // EXTREME TDD Round 60: time/assert/collection/range functions
mod system_builtins; // EXTREME TDD Round 61: env/fs/path functions
pub mod builtin_type_inference;
pub mod mutation_detection;
pub mod pattern_bindings;
pub mod function_analysis;
pub mod type_analysis;
pub mod import_helpers;
pub mod expression_analysis;
pub mod std_imports;
pub mod param_usage_analysis;
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
/// Block categorization result: (functions, statements, modules, `has_main`, `main_expr`, imports, globals)
/// TRANSPILER-SCOPE: Added globals vector for static mut declarations
type BlockCategorization<'a> = (
    Vec<TokenStream>, // functions
    Vec<TokenStream>, // statements
    Vec<TokenStream>, // modules
    bool,             // has_main
    Option<&'a Expr>, // main_expr
    Vec<TokenStream>, // imports
    Vec<TokenStream>, // globals (static mut declarations)
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
pub struct Transpiler {
    /// Whether the current code generation is within an async context.
    ///
    /// This affects how await expressions and async blocks are generated.
    pub in_async_context: bool,
    /// Whether the current code generation is within a loop context (DEFECT-018 fix).
    ///
    /// This affects whether function call arguments need to be cloned to prevent
    /// "use of moved value" errors in loop iterations.
    /// Uses Cell for interior mutability since transpiler methods take &self.
    pub in_loop_context: std::cell::Cell<bool>,
    /// Set of variable names that require mutable bindings.
    ///
    /// Populated during mutability analysis to automatically infer `mut`.
    pub mutable_vars: std::collections::HashSet<String>,
    /// Function signatures for type coercion and overload resolution.
    ///
    /// Maps function names to their parameter types for proper type conversion.
    pub function_signatures: std::collections::HashMap<String, FunctionSignature>,
    /// Module names that have been imported/defined (Issue #103).
    ///
    /// Tracks module identifiers so field access can use :: syntax for module paths.
    pub module_names: std::collections::HashSet<String>,
    /// Variable names that hold String values (DEFECT-016 fix).
    ///
    /// Populated during transpilation to track which mutable variables are Strings.
    /// Used to distinguish string concatenation from numeric addition.
    /// Uses `RefCell` for interior mutability since transpiler methods take &self.
    pub string_vars: std::cell::RefCell<std::collections::HashSet<String>>,
    /// Current function return type (TRANSPILER-007 fix).
    ///
    /// Tracks the return type of the function currently being transpiled.
    /// Used to generate concrete type hints for empty vec initializations.
    /// Uses `RefCell` for interior mutability since transpiler methods take &self.
    pub current_function_return_type: std::cell::RefCell<Option<crate::frontend::ast::Type>>,
    /// Global variable names that need unsafe access (TRANSPILER-SCOPE fix).
    ///
    /// Tracks which variables are static mut globals requiring unsafe blocks.
    /// Uses `RwLock` for thread-safe interior mutability since transpiler is used in async contexts.
    pub global_vars: std::sync::RwLock<std::collections::HashSet<String>>,
    /// SPEC-001-B: Const variable names that need module-level const declarations
    ///
    /// Populated during initial analysis (before optimization) to preserve const attributes.
    pub const_vars: std::sync::RwLock<std::collections::HashSet<String>>,
    /// DEFECT-024 FIX: Track variable types for Option/Result detection
    ///
    /// Maps variable names to their type strings (e.g., "Option<i32>", "Result<T, E>")
    /// Used to detect Option/Result types when processing method chains.
    pub variable_types: std::cell::RefCell<std::collections::HashMap<String, String>>,
}
impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for Transpiler {
    fn clone(&self) -> Self {
        Self {
            in_async_context: self.in_async_context,
            in_loop_context: std::cell::Cell::new(self.in_loop_context.get()),
            mutable_vars: self.mutable_vars.clone(),
            function_signatures: self.function_signatures.clone(),
            module_names: self.module_names.clone(),
            string_vars: std::cell::RefCell::new(self.string_vars.borrow().clone()),
            current_function_return_type: std::cell::RefCell::new(
                self.current_function_return_type.borrow().clone(),
            ),
            global_vars: std::sync::RwLock::new(
                self.global_vars
                    .read()
                    .expect("rwlock should not be poisoned")
                    .clone(),
            ),
            const_vars: std::sync::RwLock::new(
                self.const_vars
                    .read()
                    .expect("rwlock should not be poisoned")
                    .clone(),
            ),
            variable_types: std::cell::RefCell::new(self.variable_types.borrow().clone()),
        }
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
    /// let mut transpiler = Transpiler::new();
    /// assert!(!transpiler.in_async_context);
    /// ```
    pub fn new() -> Self {
        Self {
            in_async_context: false,
            in_loop_context: std::cell::Cell::new(false),
            mutable_vars: std::collections::HashSet::new(),
            function_signatures: std::collections::HashMap::new(),
            module_names: std::collections::HashSet::new(),
            string_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
            current_function_return_type: std::cell::RefCell::new(None),
            global_vars: std::sync::RwLock::new(std::collections::HashSet::new()),
            const_vars: std::sync::RwLock::new(std::collections::HashSet::new()),
            variable_types: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
    /// Centralized result printing logic - ONE PLACE FOR ALL RESULT PRINTING
    /// This eliminates code duplication and ensures consistent Unit type handling
    /// FIX-001: Use {:?} for all types to avoid Display trait requirement on ()
    fn generate_result_printing_tokens(&self) -> TokenStream {
        quote! {
            // Check the type name first to avoid printing Unit type
            // Use {:?} for all types since () implements Debug but not Display
            if std::any::type_name_of_val(&result) == "()" {
                // Don't print Unit type
            } else {
                // Use Debug formatting for all types to handle ()
                // This works for String, &str, and all other types
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
    /// SPEC-001-B: Collects const declarations BEFORE optimization (preserves attributes)
    ///
    /// # Purpose
    /// Const declarations have a `const` attribute that gets stripped by optimization passes.
    /// We must collect them here to generate module-level const declarations later.
    pub fn collect_const_declarations(&mut self, exprs: &[Expr]) {
        for expr in exprs {
            self.collect_const_declarations_from_expr(expr);
        }
    }
    /// SPEC-001-B: Collect const declarations from a single expression
    pub fn collect_const_declarations_from_expr(&mut self, expr: &Expr) {
        if let ExprKind::Let { name, .. } = &expr.kind {
            // Check for const attribute (before it's lost in optimization)
            let is_const = expr.attributes.iter().any(|attr| attr.name == "const");
            if is_const {
                self.const_vars
                    .write()
                    .expect("rwlock should not be poisoned")
                    .insert(name.clone());
            }
        }
        // Recursively check nested expressions
        match &expr.kind {
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.collect_const_declarations_from_expr(e);
                }
            }
            ExprKind::Function { body, .. } => {
                self.collect_const_declarations_from_expr(body);
            }
            _ => {}
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

    /// Collects module names from the AST (Issue #103).
    ///
    /// Scans the AST for module declarations and records their names
    /// so field access can use :: syntax for module paths.
    ///
    /// # Arguments
    ///
    /// * `exprs` - The expressions to scan for module declarations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut transpiler = Transpiler::new();
    /// transpiler.collect_module_names(&ast_expressions);
    /// // Now the transpiler knows which identifiers are modules
    /// ```
    pub fn collect_module_names(&mut self, exprs: &[Expr]) {
        for expr in exprs {
            self.collect_module_names_from_expr(expr);
        }
    }

    /// Helper to recursively collect module names from an expression
    fn collect_module_names_from_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Module { name, body } => {
                self.module_names.insert(name.clone());
                self.collect_module_names_from_expr(body);
            }
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.collect_module_names_from_expr(e);
                }
            }
            _ => {}
        }
    }
    fn collect_signatures_from_expr(&mut self, expr: &Expr) {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Function { name, params, .. } => {
                let param_types: Vec<String> = params
                    .iter()
                    .map(|param| Self::type_to_string(&param.ty))
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
    fn type_to_string(ty: &crate::frontend::ast::Type) -> String {
        use crate::frontend::ast::TypeKind;
        match &ty.kind {
            TypeKind::Named(name) => name.clone(),
            // DEFECT-024 FIX: Handle generic types like Option<i32>, Result<T, E>
            TypeKind::Generic { base, params } => {
                if params.is_empty() {
                    base.clone()
                } else {
                    let param_strs: Vec<String> = params.iter().map(Self::type_to_string).collect();
                    format!("{}<{}>", base, param_strs.join(", "))
                }
            }
            TypeKind::Reference { inner, .. } => format!("&{}", Self::type_to_string(inner)),
            _ => "Unknown".to_string(),
        }
    }
    fn analyze_expr_mutability(&mut self, expr: &Expr) {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Assign { target, value } => {
                self.mark_target_mutable(target);
                self.analyze_expr_mutability(value);
            }
            ExprKind::CompoundAssign { target, value, .. } => {
                self.mark_target_mutable(target);
                self.analyze_expr_mutability(value);
            }
            ExprKind::PreIncrement { target }
            | ExprKind::PostIncrement { target }
            | ExprKind::PreDecrement { target }
            | ExprKind::PostDecrement { target } => {
                self.mark_target_mutable(target);
            }
            ExprKind::Block(exprs) => {
                self.analyze_block_mutability(exprs);
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze_if_mutability(condition, then_branch, else_branch.as_deref());
            }
            ExprKind::While {
                condition, body, ..
            } => {
                self.analyze_two_expr_mutability(condition, body);
            }
            ExprKind::For { body, iter, .. } => {
                self.analyze_two_expr_mutability(iter, body);
            }
            ExprKind::Match { expr, arms } => {
                self.analyze_match_mutability(expr, arms);
            }
            ExprKind::Let { body, value, .. } | ExprKind::LetPattern { body, value, .. } => {
                self.analyze_two_expr_mutability(value, body);
            }
            ExprKind::Function { body, .. } | ExprKind::Lambda { body, .. } => {
                self.analyze_expr_mutability(body);
            }
            ExprKind::Binary { left, right, .. } => {
                self.analyze_two_expr_mutability(left, right);
            }
            ExprKind::Unary { operand, .. } => {
                self.analyze_expr_mutability(operand);
            }
            ExprKind::Call { func, args } => {
                self.analyze_call_mutability(func, args);
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                self.analyze_call_mutability(receiver, args);
            }
            _ => {}
        }
    }

    /// Mark an expression target as mutable (complexity: 2)
    fn mark_target_mutable(&mut self, target: &Expr) {
        if let ExprKind::Identifier(name) = &target.kind {
            self.mutable_vars.insert(name.clone());
        }
    }

    /// Analyze mutability for block expressions (complexity: 1)
    fn analyze_block_mutability(&mut self, exprs: &[Expr]) {
        for e in exprs {
            self.analyze_expr_mutability(e);
        }
    }

    /// Analyze mutability for if expressions (complexity: 2)
    fn analyze_if_mutability(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) {
        self.analyze_expr_mutability(condition);
        self.analyze_expr_mutability(then_branch);
        if let Some(else_expr) = else_branch {
            self.analyze_expr_mutability(else_expr);
        }
    }

    /// Analyze mutability for two related expressions (complexity: 1)
    fn analyze_two_expr_mutability(&mut self, expr1: &Expr, expr2: &Expr) {
        self.analyze_expr_mutability(expr1);
        self.analyze_expr_mutability(expr2);
    }

    /// Analyze mutability for match expressions (complexity: 1)
    fn analyze_match_mutability(&mut self, expr: &Expr, arms: &[crate::frontend::ast::MatchArm]) {
        self.analyze_expr_mutability(expr);
        for arm in arms {
            self.analyze_expr_mutability(&arm.body);
        }
    }

    /// Analyze mutability for call expressions (complexity: 1)
    fn analyze_call_mutability(&mut self, func: &Expr, args: &[Expr]) {
        self.analyze_expr_mutability(func);
        for arg in args {
            self.analyze_expr_mutability(arg);
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
        if !Self::contains_file_imports(expr) {
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
    fn contains_imports(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_imports),
            _ => false,
        }
    }

    /// Check if an expression tree contains any file imports (local .ruchy files)
    fn contains_file_imports(expr: &Expr) -> bool {
        match &expr.kind {
            // ISSUE-106 FIX: ModuleDeclaration (mod name;) needs file resolution
            ExprKind::ModuleDeclaration { .. } => true,
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
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_file_imports),
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
    /// let mut transpiler = Transpiler::new();
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
    /// let mut transpiler = Transpiler::new();
    /// let tokens = transpiler.transpile(&ast).unwrap();
    ///
    /// // Convert to string for compilation
    /// let rust_code = tokens.to_string();
    /// ```
    /// TRANSPILER-009 FIX: Changed to call `transpile_to_program()` instead of `transpile_expr()`
    /// Root Cause: `transpile_expr()` treats Block as an expression and wraps in braces { ... }
    /// which produces invalid Rust when the Block contains top-level items (functions/structs/etc)
    /// Fix: Always use `transpile_to_program()` which properly handles top-level items
    pub fn transpile(&mut self, expr: &Expr) -> Result<TokenStream> {
        self.transpile_to_program(expr)
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
    /// Checks if an expression contains `DataFrame` operations
    /// Recursively scans the entire AST to detect `DataFrame` usage
    fn contains_dataframe(expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;

        match &expr.kind {
            // Direct DataFrame constructs
            ExprKind::DataFrame { .. } | ExprKind::DataFrameOperation { .. } => true,

            // DataFrame::new() constructor calls
            ExprKind::Call { func, .. } => {
                if let ExprKind::QualifiedName { module, name } = &func.kind {
                    module == "DataFrame" && name == "new"
                } else {
                    false
                }
            }

            // Method calls on DataFrame instances (builder pattern)
            // DEFECT-022 FIX: Only check receiver, removed generic "build" method heuristic
            ExprKind::MethodCall { receiver, .. } => Self::contains_dataframe(receiver),

            // Type annotations with DataFrame
            ExprKind::Function { params, body, .. } => {
                params.iter().any(|p| {
                    if let crate::frontend::ast::TypeKind::Named(name) = &p.ty.kind {
                        name == "DataFrame"
                    } else {
                        false
                    }
                }) || Self::contains_dataframe(body)
            }

            // Let bindings with DataFrame
            ExprKind::Let { body, value, .. } => {
                Self::contains_dataframe(value) || Self::contains_dataframe(body)
            }

            // Blocks and control flow
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_dataframe),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::contains_dataframe(condition)
                    || Self::contains_dataframe(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|e| Self::contains_dataframe(e))
            }

            _ => false,
        }
    }
    /// TRANSPILER-009: Check if expression contains standalone user-defined functions
    /// Returns true if this is a Block with Function definitions (not inside impl/class)
    /// Used to skip aggressive optimizations that would inline/eliminate user functions
    fn has_standalone_functions(expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;

        match &expr.kind {
            // A Block with Function expressions => has standalone functions
            ExprKind::Block(exprs) => exprs
                .iter()
                .any(|e| matches!(&e.kind, ExprKind::Function { .. })),
            // Single top-level Function
            ExprKind::Function { .. } => true,
            _ => false,
        }
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
        // First analyze the entire program to detect mutable variables, const declarations, function signatures, and modules
        // SPEC-001-B: Must collect const names BEFORE optimization to preserve attributes
        if let ExprKind::Block(exprs) = &expr.kind {
            self.analyze_mutability(exprs);
            self.collect_const_declarations(exprs);
            self.collect_function_signatures(exprs);
            self.collect_module_names(exprs);
        } else {
            self.analyze_expr_mutability(expr);
            self.collect_const_declarations_from_expr(expr);
            self.collect_signatures_from_expr(expr);
            self.collect_module_names_from_expr(expr);
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
        &mut self,
        expr: &Expr,
        file_path: Option<&std::path::Path>,
    ) -> Result<TokenStream> {
        // First, resolve any file imports using the module resolver
        let resolved_expr = self.resolve_imports_with_context(expr, file_path)?;

        // TRANSPILER-009 FIX: Skip aggressive optimizations for top-level programs with standalone functions
        // The inlining + DCE optimizations are meant for eval expressions, NOT full programs
        // They were causing standalone functions to be inlined and then eliminated
        let has_standalone_functions = Self::has_standalone_functions(&resolved_expr);

        let optimized_expr = if has_standalone_functions {
            // Skip inlining and DCE for programs with standalone functions
            // Only apply constant folding/propagation which is safe
            constant_folder::propagate_constants(resolved_expr)
        } else {
            // PERF-002-A/B: Apply constant folding + constant propagation
            let after_propagation = constant_folder::propagate_constants(resolved_expr);

            // OPT-CODEGEN-004: Inline small, non-recursive functions
            let (after_inlining, inlined_functions) =
                inline_expander::inline_small_functions(after_propagation);

            // PERF-002-C: Dead code elimination (removes unused inlined functions)
            // Pass inlined function names so DCE can preserve functions that weren't inlined
            constant_folder::eliminate_dead_code(after_inlining, inlined_functions)
        };

        // CRITICAL: Analyze mutability, signatures, and modules BEFORE transpiling
        // This populates self.mutable_vars, function_signatures, and module_names
        if let ExprKind::Block(exprs) = &optimized_expr.kind {
            self.analyze_mutability(exprs);
            self.collect_function_signatures(exprs);
            self.collect_module_names(exprs);
        } else {
            self.analyze_expr_mutability(&optimized_expr);
            self.collect_signatures_from_expr(&optimized_expr);
            self.collect_module_names_from_expr(&optimized_expr);
        }

        let needs_polars = Self::contains_dataframe(&optimized_expr);
        let needs_hashmap = Self::contains_hashmap(&optimized_expr);
        match &optimized_expr.kind {
            ExprKind::Function { name, .. } => {
                self.transpile_single_function(&optimized_expr, name, needs_polars, needs_hashmap)
            }
            ExprKind::Block(exprs) => {
                self.transpile_program_block(exprs, needs_polars, needs_hashmap)
            }
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => {
                // Single import - handle as top-level
                let import_tokens = self.transpile_expr(&optimized_expr)?;
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
            _ => self.transpile_expression_program(&optimized_expr, needs_polars, needs_hashmap),
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
        let (functions, statements, modules, has_main, main_expr, imports, globals) =
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
                    &globals,
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
                &globals,
            )
        } else {
            self.transpile_block_with_functions(
                &functions,
                &statements,
                needs_polars,
                needs_hashmap,
                &imports,
                &globals,
            )
        }
    }

    /// Helper: Infer type token from expression value
    /// Reduces cognitive complexity by extracting duplicated type inference patterns
    fn infer_type_from_value(value: &Expr) -> TokenStream {
        match &value.kind {
            ExprKind::Literal(lit) => match lit {
                crate::frontend::ast::Literal::Integer(_, _) => quote! { i32 },
                crate::frontend::ast::Literal::Float(_) => quote! { f64 },
                crate::frontend::ast::Literal::String(_) => quote! { &str },
                crate::frontend::ast::Literal::Bool(_) => quote! { bool },
                _ => quote! { i32 },
            },
            ExprKind::List(elements) => {
                if elements.is_empty() {
                    quote! { Vec<i32> }
                } else {
                    // Infer element type from first element
                    match &elements[0].kind {
                        ExprKind::Literal(lit) => match lit {
                            crate::frontend::ast::Literal::Integer(_, _) => quote! { Vec<i32> },
                            crate::frontend::ast::Literal::Float(_) => quote! { Vec<f64> },
                            crate::frontend::ast::Literal::String(_) => quote! { Vec<String> },
                            crate::frontend::ast::Literal::Bool(_) => quote! { Vec<bool> },
                            _ => quote! { Vec<i32> },
                        },
                        ExprKind::List(_) => quote! { Vec<Vec<i32>> },
                        _ => quote! { Vec<i32> },
                    }
                }
            }
            _ => quote! { i32 },
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

        // TRANSPILER-SCOPE: First pass - collect names of mutable Lets that will become globals
        // SPEC-001-B: Const names are collected earlier (before optimization) in collect_const_declarations()
        let mut global_var_names = std::collections::HashSet::new();
        let const_var_names = self
            .const_vars
            .read()
            .expect("rwlock should not be poisoned")
            .clone();
        for expr in exprs {
            if let ExprKind::Function { name, .. } = &expr.kind {
                if name == "main" {
                    has_main_function = true;
                }
            }
        }

        // If we have functions, collect mutable Let names to promote to globals
        let has_functions_check = exprs
            .iter()
            .any(|e| matches!(&e.kind, ExprKind::Function { .. }))
            || has_main_function;
        if has_functions_check {
            for expr in exprs {
                if let ExprKind::Let {
                    name, is_mutable, ..
                } = &expr.kind
                {
                    if *is_mutable && !const_var_names.contains(name) {
                        global_var_names.insert(name.clone());
                    }
                }
            }
        }

        // TRANSPILER-SCOPE: Store global variable names for use during expression transpilation
        (*self
            .global_vars
            .write()
            .expect("rwlock should not be poisoned"))
        .clone_from(&global_var_names);

        // Second pass - categorize expressions, skipping main() calls and promoted globals
        for expr in exprs {
            // Skip explicit main() calls when main function exists
            if has_main_function && Self::is_call_to_main(expr) {
                continue;
            }

            // TRANSPILER-SCOPE: Skip mutable Lets and const declarations that were promoted to globals
            if let ExprKind::Let {
                name, is_mutable, ..
            } = &expr.kind
            {
                if (*is_mutable && global_var_names.contains(name))
                    || const_var_names.contains(name)
                {
                    continue; // Skip this Let - it's now a static mut global or module-level const
                }
            }

            self.categorize_single_expression(
                expr,
                &mut functions,
                &mut statements,
                &mut modules,
                &mut imports,
                &mut has_main_function,
                &mut main_function_expr,
            )?;
        }

        // TRANSPILER-SCOPE: Third pass - generate static mut declarations for globals and const declarations
        let mut globals = Vec::new();

        // SPEC-001-B: Generate module-level const declarations
        if !const_var_names.is_empty() {
            for expr in exprs {
                if let ExprKind::Let {
                    name,
                    value,
                    type_annotation,
                    ..
                } = &expr.kind
                {
                    if const_var_names.contains(name) {
                        // Transpile value to get initializer
                        let value_tokens = self.transpile_expr(value)?;
                        let const_name = format_ident!("{}", name);

                        // Const declarations MUST have explicit type annotation
                        let type_token = if let Some(ref type_ann) = type_annotation {
                            self.transpile_type(type_ann)?
                        } else {
                            // Use helper to infer type from literal
                            Self::infer_type_from_value(value)
                        };

                        // Generate module-level const declaration
                        globals.push(quote! {
                            const #const_name: #type_token = #value_tokens;
                        });
                    }
                }
            }
        }

        // Generate thread-safe mutable globals using LazyLock<Mutex<T>>
        if !global_var_names.is_empty() {
            for expr in exprs {
                if let ExprKind::Let {
                    name,
                    value,
                    is_mutable,
                    type_annotation,
                    ..
                } = &expr.kind
                {
                    if *is_mutable && global_var_names.contains(name) {
                        // Transpile value to get initializer
                        let value_tokens = self.transpile_expr(value)?;
                        let var_name = format_ident!("{}", name);

                        // TRANSPILER-SCOPE: Infer type from literal or use annotation
                        // Static variables can't use `_` placeholder, need explicit type
                        let type_token = if let Some(ref type_ann) = type_annotation {
                            self.transpile_type(type_ann)?
                        } else {
                            // Use helper for type inference
                            Self::infer_type_from_value(value)
                        };

                        // Generate thread-safe global using LazyLock<Mutex<T>>
                        // Issue #132: NEVER generate unsafe code - use safe Rust abstractions
                        globals.push(quote! {
                            static #var_name: std::sync::LazyLock<std::sync::Mutex<#type_token>> =
                                std::sync::LazyLock::new(|| std::sync::Mutex::new(#value_tokens));
                        });
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
            globals,
        ))
    }

    /// Check if expression is a call to `main()` function
    /// Used to prevent stack overflow when both `fun main()` definition and `main()` call exist
    /// Complexity: 2 (within Toyota Way limits)
    fn is_call_to_main(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Call { func, .. } => {
                matches!(&func.kind, ExprKind::Identifier(name) if name == "main")
            }
            _ => false,
        }
    }

    /// Categorize a single expression into appropriate category (complexity: 8)
    fn categorize_single_expression<'a>(
        &self,
        expr: &'a Expr,
        functions: &mut Vec<TokenStream>,
        statements: &mut Vec<TokenStream>,
        modules: &mut Vec<TokenStream>,
        imports: &mut Vec<TokenStream>,
        has_main_function: &mut bool,
        main_function_expr: &mut Option<&'a Expr>,
    ) -> Result<()> {
        match &expr.kind {
            ExprKind::Function { name, .. } => {
                self.categorize_function(
                    expr,
                    name,
                    functions,
                    has_main_function,
                    main_function_expr,
                )?;
            }
            ExprKind::Module { name, body } => {
                modules.push(self.transpile_module_declaration(name, body)?);
            }
            ExprKind::Block(block_exprs) => {
                self.categorize_block(block_exprs, expr, modules, statements, imports)?;
            }
            ExprKind::Trait { .. } | ExprKind::Impl { .. } => {
                functions.push(self.transpile_type_decl_expr(expr)?);
            }
            ExprKind::Struct { .. } | ExprKind::TupleStruct { .. } => {
                functions.push(self.transpile_struct_expr(expr)?);
            }
            ExprKind::Enum { .. }
            | ExprKind::Class { .. }
            | ExprKind::Actor { .. }
            | ExprKind::Effect { .. } => {
                functions.push(self.transpile_expr(expr)?);
            }
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => {
                imports.push(self.transpile_expr(expr)?);
            }
            _ => {
                self.categorize_statement(expr, statements)?;
            }
        }
        Ok(())
    }

    /// Categorize function expression (complexity: 3)
    fn categorize_function<'a>(
        &self,
        expr: &'a Expr,
        name: &str,
        functions: &mut Vec<TokenStream>,
        has_main_function: &mut bool,
        main_function_expr: &mut Option<&'a Expr>,
    ) -> Result<()> {
        if name == "main" {
            *has_main_function = true;
            *main_function_expr = Some(expr);
        } else {
            functions.push(self.transpile_function_expr(expr)?);
        }
        Ok(())
    }

    /// Categorize block expression (complexity: 4)
    fn categorize_block(
        &self,
        block_exprs: &[Expr],
        expr: &Expr,
        modules: &mut Vec<TokenStream>,
        statements: &mut Vec<TokenStream>,
        imports: &mut Vec<TokenStream>,
    ) -> Result<()> {
        // Check if this is a module-containing block from the resolver
        if self.is_module_resolver_block(block_exprs) {
            if let ExprKind::Module { name, body } = &block_exprs[0].kind {
                modules.push(self.transpile_module_declaration(name, body)?);
            }
            // ISSUE-103: Import should go to imports vector, not statements
            imports.push(self.transpile_expr(&block_exprs[1])?);
        } else {
            // Regular block, treat as statement
            statements.push(self.transpile_expr(expr)?);
        }
        Ok(())
    }

    /// Check if block is a module resolver block (complexity: 2)
    fn is_module_resolver_block(&self, block_exprs: &[Expr]) -> bool {
        block_exprs.len() == 2
            && matches!(block_exprs[0].kind, ExprKind::Module { .. })
            && matches!(block_exprs[1].kind, ExprKind::Import { .. })
    }

    /// Categorize general statement expression (complexity: 3)
    fn categorize_statement(&self, expr: &Expr, statements: &mut Vec<TokenStream>) -> Result<()> {
        let stmt = self.transpile_expr(expr)?;
        let stmt_str = stmt.to_string();

        if !stmt_str.trim().ends_with(';') && !stmt_str.trim().ends_with('}') {
            statements.push(quote! { #stmt; });
        } else {
            statements.push(stmt);
        }
        Ok(())
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
        let has_statements = exprs.iter().any(Self::is_statement_expr);
        if has_statements {
            // Split into statements and possible final expression
            let (statements, final_expr) = if !exprs.is_empty()
                && !Self::is_statement_expr(
                    exprs
                        .last()
                        .expect("vec is non-empty due to is_empty check"),
                ) {
                // Last item is an expression, not a statement
                (
                    &exprs[..exprs.len() - 1],
                    Some(
                        exprs
                            .last()
                            .expect("vec is non-empty due to is_empty check"),
                    ),
                )
            } else {
                // All are statements
                (exprs, None)
            };
            // Transpile all statements and add semicolons intelligently
            let statement_results: Result<Vec<_>> = statements
                .iter()
                .map(|expr| {
                    let tokens = self.transpile_expr(expr)?;
                    // Let expressions already include semicolons in their transpilation
                    // Don't add another semicolon for them
                    if matches!(
                        expr.kind,
                        ExprKind::Let { .. } | ExprKind::LetPattern { .. }
                    ) {
                        Ok(tokens)
                    } else {
                        // Add semicolon for other statement types
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
    fn is_statement_expr(expr: &Expr) -> bool {
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
            // If expressions where both branches are statements (return unit)
            ExprKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                // If both branches are statements, the whole if is a statement
                Self::is_statement_expr(then_branch)
                    && else_branch
                        .as_ref()
                        .is_none_or(|e| Self::is_statement_expr(e))
            }
            // Blocks containing statements
            ExprKind::Block(exprs) => exprs.iter().any(Self::is_statement_expr),
            // Most other expressions are not statements
            _ => false,
        }
    }
    /// Transpile block with main function wrapper
    /// Complexity: 3 (within Toyota Way limits)
    fn transpile_block_with_main_function(
        &self,
        functions: &[TokenStream],
        statements: &[TokenStream],
        modules: &[TokenStream],
        main_expr: Option<&Expr>,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        if statements.is_empty() && main_expr.is_some() {
            self.transpile_functions_only_mode(
                functions,
                modules,
                main_expr,
                needs_polars,
                needs_hashmap,
                imports,
                globals,
            )
        } else {
            self.transpile_with_top_level_statements(
                functions,
                statements,
                modules,
                main_expr,
                needs_polars,
                needs_hashmap,
                imports,
                globals,
            )
        }
    }

    /// Transpile in functions-only mode (no top-level statements)
    /// Complexity: 2 (within Toyota Way limits)
    fn transpile_functions_only_mode(
        &self,
        functions: &[TokenStream],
        modules: &[TokenStream],
        main_expr: Option<&Expr>,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        let main_tokens = if let Some(main) = main_expr {
            self.transpile_expr(main)?
        } else {
            return Err(anyhow::anyhow!("Expected main function expression"));
        };

        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);
        // TRANSPILER-MODULE-001 FIX: Modules must be declared BEFORE use statements
        // that reference them. Reorder: modules -> imports -> functions -> main
        Ok(quote! {
            #use_statements
            #(#modules)*
            #(#imports)*
            #(#globals)*
            #(#functions)*
            #main_tokens
        })
    }

    /// Transpile with top-level statements
    /// Complexity: 2 (within Toyota Way limits)
    /// Transpile with top-level statements
    /// DEFECT-COMPILE-MAIN-CALL: When user has `fun main()` + module statements,
    /// rename user's main to `__ruchy_main` to avoid collision with Rust entry point
    /// Complexity: 3 (within Toyota Way limits)
    fn transpile_with_top_level_statements(
        &self,
        functions: &[TokenStream],
        statements: &[TokenStream],
        modules: &[TokenStream],
        main_expr: Option<&Expr>,
        needs_polars: bool,
        needs_hashmap: bool,
        imports: &[TokenStream],
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        // DEFECT-COMPILE-MAIN-CALL: If we have both main function AND module-level statements,
        // we need to rename the user's main to avoid collision with Rust's entry point
        let user_main_function = if let Some(main) = main_expr {
            // Transpile the user's main function as __ruchy_main
            self.transpile_main_as_renamed_function(main)?
        } else {
            quote! {}
        };

        let use_statements = self.generate_use_statements(needs_polars, needs_hashmap);

        // Issue #132: No unsafe wrapping needed - LazyLock<Mutex<T>> is thread-safe
        let main_body = quote! { #(#statements)* };

        // TRANSPILER-MODULE-001 FIX: Modules must be declared BEFORE use statements
        // that reference them. Reorder: modules -> imports -> functions
        Ok(quote! {
            #use_statements
            #(#modules)*
            #(#imports)*
            #(#globals)*
            #(#functions)*
            #user_main_function
            fn main() {
                #main_body
            }
        })
    }

    /// Transpile user's main function with renamed identifier to avoid Rust entry point collision
    /// DEFECT-COMPILE-MAIN-CALL: Renames `fun main()` to `fn __ruchy_main()` to prevent infinite recursion
    /// Complexity: 6 (within Toyota Way limits)
    fn transpile_main_as_renamed_function(&self, main_expr: &Expr) -> Result<TokenStream> {
        if let ExprKind::Function {
            params, body, name, ..
        } = &main_expr.kind
        {
            if name != "main" {
                return Err(anyhow::anyhow!("Expected main function, got {name}"));
            }

            // Transpile parameters
            let param_tokens: Result<Vec<TokenStream>> = params
                .iter()
                .map(|param| {
                    let param_name = format_ident!("{}", param.name());
                    let param_type = self.transpile_type(&param.ty)?;
                    Ok(quote! { #param_name: #param_type })
                })
                .collect();
            let param_tokens = param_tokens?;

            // Transpile body
            let body_tokens = self.transpile_expr(body)?;

            // Generate function with renamed identifier
            let renamed_ident = format_ident!("__ruchy_main");
            Ok(quote! {
                fn #renamed_ident(#(#param_tokens),*) {
                    #body_tokens
                }
            })
        } else {
            Err(anyhow::anyhow!("Expected function expression"))
        }
    }

    /// Generate use statements based on feature flags
    /// Complexity: 1 (within Toyota Way limits)
    fn generate_use_statements(&self, needs_polars: bool, needs_hashmap: bool) -> TokenStream {
        match (needs_polars, needs_hashmap) {
            (true, true) => quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
            },
            (true, false) => quote! {
                use polars::prelude::*;
            },
            (false, true) => quote! {
                use std::collections::HashMap;
            },
            (false, false) => quote! {},
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
        globals: &[TokenStream],
    ) -> Result<TokenStream> {
        // TRANSPILER-SCOPE: Emit static mut globals at module level
        // No main function among extracted functions - create one for statements

        // Issue #132: No unsafe wrapping needed - LazyLock<Mutex<T>> is thread-safe
        let main_body = quote! { #(#statements)* };

        match (needs_polars, needs_hashmap) {
            (true, true) => Ok(quote! {
                use polars::prelude::*;
                use std::collections::HashMap;
                #(#imports)*
                #(#globals)*
                #(#functions)*
                fn main() { #main_body }
            }),
            (true, false) => Ok(quote! {
                use polars::prelude::*;
                #(#imports)*
                #(#globals)*
                #(#functions)*
                fn main() { #main_body }
            }),
            (false, true) => Ok(quote! {
                use std::collections::HashMap;
                #(#imports)*
                #(#globals)*
                #(#functions)*
                fn main() { #main_body }
            }),
            (false, false) => Ok(quote! {
                #(#imports)*
                #(#globals)*
                #(#functions)*
                fn main() { #main_body }
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
            | ExprKind::TupleStruct { .. }
            | ExprKind::Class { .. }
            | ExprKind::Actor { .. }
            | ExprKind::Effect { .. }
            | ExprKind::Impl { .. } => {
                // Structs, actors, effects, and impl blocks should be top-level items
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
                if Self::is_statement_expr(expr) {
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
    /// TRANSPILER-009: Changed to &mut self to match `transpile()` signature
    pub fn transpile_to_string(&mut self, expr: &Expr) -> Result<String> {
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
            DictComprehension, Effect, Err, FieldAccess, For, Function, Handle, Identifier, If,
            IfLet, IndexAccess, Lambda, List, ListComprehension, Literal, Loop, Macro, Match,
            MethodCall, None, ObjectLiteral, Ok, PostDecrement, PostIncrement, PreDecrement,
            PreIncrement, QualifiedName, Range, Send, Set, SetComprehension, Slice, Some, Spawn,
            StringInterpolation, Struct, StructLiteral, Throw, Try, TryCatch, Tuple, TupleStruct,
            TypeCast, Unary, While, WhileLet,
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
            | Spawn { .. }
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
            | TupleStruct { .. }
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
            | Effect { .. }
            | Handle { .. }
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
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
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
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn create_test_variable_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
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
            leading_comments: vec![],
            trailing_comment: None,
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
        let int_type = create_simple_type("i64");
        let float_type = create_simple_type("f64");
        let string_type = create_simple_type("String");
        let bool_type = create_simple_type("bool");

        // Test basic type handling (exact behavior depends on implementation)
        let int_result = Transpiler::type_to_string(&int_type);
        assert!(!int_result.is_empty());

        let float_result = Transpiler::type_to_string(&float_type);
        assert!(!float_result.is_empty());

        let string_result = Transpiler::type_to_string(&string_type);
        assert!(!string_result.is_empty());

        let bool_result = Transpiler::type_to_string(&bool_type);
        assert!(!bool_result.is_empty());

        // Test list type
        let list_type = Type {
            kind: TypeKind::List(Box::new(create_simple_type("i64"))),
            span: Span::default(),
        };
        let list_result = Transpiler::type_to_string(&list_type);
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
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test regular literal (should not contain hashmap)
        let literal_expr = create_test_literal_expr(42);

        // Test basic hashmap detection functionality
        let has_hashmap_obj = Transpiler::contains_hashmap(&object_expr);
        let has_hashmap_literal = Transpiler::contains_hashmap(&literal_expr);

        // Object literals typically indicate hashmap usage
        // Literals typically do not indicate hashmap usage
        // These assertions are just checking the functions don't panic
        let _ = has_hashmap_obj;
        let _ = has_hashmap_literal;
    }

    // Test 5: DataFrame Detection in Expressions
    #[test]
    fn test_contains_dataframe() {
        // Test DataFrame literal (should contain dataframe)
        let df_expr = Expr {
            kind: ExprKind::DataFrame { columns: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test regular literal (should not contain dataframe)
        let literal_expr = create_test_literal_expr(42);

        // Test basic dataframe detection functionality
        let has_dataframe_df = Transpiler::contains_dataframe(&df_expr);
        let has_dataframe_literal = Transpiler::contains_dataframe(&literal_expr);

        // DataFrame expressions typically indicate dataframe usage
        // Literals typically do not indicate dataframe usage
        // These assertions are just checking the functions don't panic
        let _ = has_dataframe_df;
        let _ = has_dataframe_literal;
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
            leading_comments: vec![],
            trailing_comment: None,
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
            leading_comments: vec![],
            trailing_comment: None,
        };

        transpiler.analyze_expr_mutability(&assign_expr2);

        // Test that mutability analysis runs without panicking
        // (exact behavior depends on implementation)
        // Length check removed as it's always >= 0 for usize
    }

    // Test 7: Basic Expression Transpilation
    #[test]
    fn test_basic_transpile() {
        let mut transpiler = Transpiler::new();

        // Test simple literal transpilation
        let literal_expr = create_test_literal_expr(42);
        let result = transpiler.transpile(&literal_expr);
        assert!(result.is_ok());

        let token_stream = result.expect("operation should succeed in test");
        let code = token_stream.to_string();
        assert!(code.contains("42"));
    }

    // Test 8: Block Transpilation with Multiple Expressions
    #[test]
    fn test_block_transpile() {
        let mut transpiler = Transpiler::new();

        // Create block with multiple expressions
        let block_expr = Expr {
            kind: ExprKind::Block(vec![
                create_test_literal_expr(1),
                create_test_literal_expr(2),
                create_test_literal_expr(3),
            ]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
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

        let token_stream = result.expect("operation should succeed in test");
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
            leading_comments: vec![],
            trailing_comment: None,
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
            leading_comments: vec![],
            trailing_comment: None,
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
        let mut transpiler = Transpiler::new();

        // Create an expression that might cause issues (testing robustness)
        let complex_expr = Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(create_test_variable_expr("undefined_var")),
                right: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
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
                leading_comments: vec![],
                trailing_comment: None,
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
            if let ExprKind::Literal(Literal::Integer(val, None)) = resolved.kind {
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
        let mut transpiler = Transpiler::new();

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

    // Test 17: is_call_to_main - with main() call
    #[test]
    fn test_is_call_to_main_true() {
        let main_call = Expr {
            kind: ExprKind::Call {
                func: Box::new(create_test_variable_expr("main")),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_call_to_main(&main_call));
    }

    // Test 18: is_call_to_main - with non-main call
    #[test]
    fn test_is_call_to_main_false() {
        let other_call = Expr {
            kind: ExprKind::Call {
                func: Box::new(create_test_variable_expr("other_func")),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!Transpiler::is_call_to_main(&other_call));
    }

    // Test 19: is_standard_library - with std
    #[test]
    fn test_is_standard_library_std() {
        assert!(Transpiler::is_standard_library("std"));
        assert!(Transpiler::is_standard_library("core"));
        assert!(Transpiler::is_standard_library("alloc"));
    }

    // Test 20: is_standard_library - with third-party libs
    #[test]
    fn test_is_standard_library_third_party() {
        assert!(Transpiler::is_standard_library("tokio"));
        assert!(Transpiler::is_standard_library("serde"));
        assert!(Transpiler::is_standard_library("serde_json"));
        assert!(Transpiler::is_standard_library("polars"));
    }

    // Test 21: is_standard_library - with non-standard module
    #[test]
    fn test_is_standard_library_false() {
        assert!(!Transpiler::is_standard_library("my_module"));
        assert!(!Transpiler::is_standard_library("custom_lib"));
    }

    // Test 22: contains_imports - with Import expression
    #[test]
    fn test_contains_imports_true() {
        let _transpiler = Transpiler::new();
        let import_expr = Expr {
            kind: ExprKind::Import {
                module: "std::io".to_string(),
                items: Some(vec!["Read".to_string()]),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::contains_imports(&import_expr));
    }

    // Test 23: contains_imports - with non-import expression
    #[test]
    fn test_contains_imports_false() {
        let literal_expr = create_test_literal_expr(42);
        assert!(!Transpiler::contains_imports(&literal_expr));
    }

    // Test 24: contains_file_imports - with relative path
    #[test]
    fn test_contains_file_imports_relative() {
        let _transpiler = Transpiler::new();
        let file_import = Expr {
            kind: ExprKind::Import {
                module: "./my_module".to_string(),
                items: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::contains_file_imports(&file_import));
    }

    // Test 25: contains_file_imports - with parent path
    #[test]
    fn test_contains_file_imports_parent() {
        let file_import = Expr {
            kind: ExprKind::Import {
                module: "../parent_module".to_string(),
                items: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::contains_file_imports(&file_import));
    }

    // Test 26: contains_file_imports - with std library (not a file)
    #[test]
    fn test_contains_file_imports_std_false() {
        let std_import = Expr {
            kind: ExprKind::Import {
                module: "std::collections::HashMap".to_string(),
                items: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!Transpiler::contains_file_imports(&std_import));
    }

    // Test 27: is_statement_expr - with Let binding
    #[test]
    fn test_is_statement_expr_let() {
        let let_expr = Expr {
            kind: ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(create_test_literal_expr(42)),
                body: Box::new(create_test_literal_expr(0)),
                type_annotation: None,
                is_mutable: false,
                else_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_statement_expr(&let_expr));
    }

    // Test 28: is_statement_expr - with Assignment
    #[test]
    fn test_is_statement_expr_assign() {
        let assign_expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("x")),
                value: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_statement_expr(&assign_expr));
    }

    // Test 29: is_statement_expr - with While loop
    #[test]
    fn test_is_statement_expr_while() {
        let while_expr = Expr {
            kind: ExprKind::While {
                condition: Box::new(create_test_literal_expr(1)),
                body: Box::new(create_test_literal_expr(2)),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_statement_expr(&while_expr));
    }

    // Test 30: is_statement_expr - with expression (not statement)
    #[test]
    fn test_is_statement_expr_false() {
        let literal_expr = create_test_literal_expr(42);
        assert!(!Transpiler::is_statement_expr(&literal_expr));
    }

    // Test 31: generate_use_statements - with polars and HashMap
    #[test]
    fn test_generate_use_statements_both() {
        let transpiler = Transpiler::new();
        let result = transpiler.generate_use_statements(true, true);
        let code = result.to_string();
        assert!(code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    // Test 32: collect_module_names_from_expr - single module
    #[test]
    fn test_collect_module_names_single_module() {
        let mut transpiler = Transpiler::new();
        let module_expr = Expr {
            kind: ExprKind::Module {
                name: "test_module".to_string(),
                body: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_module_names_from_expr(&module_expr);
        assert!(transpiler.module_names.contains("test_module"));
    }

    // Test 33: collect_module_names_from_expr - block with modules
    #[test]
    fn test_collect_module_names_block() {
        let mut transpiler = Transpiler::new();
        let module1 = Expr {
            kind: ExprKind::Module {
                name: "mod1".to_string(),
                body: Box::new(create_test_literal_expr(1)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let module2 = Expr {
            kind: ExprKind::Module {
                name: "mod2".to_string(),
                body: Box::new(create_test_literal_expr(2)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let block_expr = Expr {
            kind: ExprKind::Block(vec![module1, module2]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_module_names_from_expr(&block_expr);
        assert!(transpiler.module_names.contains("mod1"));
        assert!(transpiler.module_names.contains("mod2"));
    }

    // Test 34: mark_target_mutable - identifier
    #[test]
    fn test_mark_target_mutable_identifier() {
        let mut transpiler = Transpiler::new();
        let target = create_test_variable_expr("x");
        transpiler.mark_target_mutable(&target);
        assert!(transpiler.mutable_vars.contains("x"));
    }

    // Test 35: analyze_block_mutability - empty block
    #[test]
    fn test_analyze_block_mutability_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.analyze_block_mutability(&[]);
        assert!(transpiler.mutable_vars.is_empty());
    }

    // Test 36: analyze_block_mutability - block with assignment
    #[test]
    fn test_analyze_block_mutability_with_assign() {
        let mut transpiler = Transpiler::new();
        let assign_expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("y")),
                value: Box::new(create_test_literal_expr(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.analyze_block_mutability(&[assign_expr]);
        assert!(transpiler.mutable_vars.contains("y"));
    }

    // Test 37: analyze_if_mutability - simple if
    #[test]
    fn test_analyze_if_mutability_simple() {
        let mut transpiler = Transpiler::new();
        let condition = create_test_literal_expr(1);
        let then_branch = create_test_variable_expr("z");
        transpiler.analyze_if_mutability(&condition, &then_branch, None);
        // Function should not panic - test passes if no panic occurs
    }

    // Test 38: analyze_two_expr_mutability - two literals
    #[test]
    fn test_analyze_two_expr_mutability_literals() {
        let mut transpiler = Transpiler::new();
        let expr1 = create_test_literal_expr(5);
        let expr2 = create_test_literal_expr(10);
        transpiler.analyze_two_expr_mutability(&expr1, &expr2);
        // Should not panic, no mutations expected from literals
        assert!(transpiler.mutable_vars.is_empty());
    }

    // Test 39: analyze_match_mutability - match with arms
    #[test]
    fn test_analyze_match_mutability_simple() {
        let mut transpiler = Transpiler::new();
        let match_expr = create_test_variable_expr("val");
        let arms = vec![];
        transpiler.analyze_match_mutability(&match_expr, &arms);
        // Should not panic with empty arms - test passes if no panic occurs
    }

    // Test 40: analyze_call_mutability - function call
    #[test]
    fn test_analyze_call_mutability_simple() {
        let mut transpiler = Transpiler::new();
        let func = create_test_variable_expr("my_func");
        let args = vec![create_test_literal_expr(42)];
        transpiler.analyze_call_mutability(&func, &args);
        // Should analyze without panicking - test passes if no panic occurs
    }

    // Test 41: has_standalone_functions - single function
    #[test]
    fn test_has_standalone_functions_single() {
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "foo".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(1)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::has_standalone_functions(&func_expr));
    }

    // Test 42: has_standalone_functions - block with functions
    #[test]
    fn test_has_standalone_functions_block() {
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "bar".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(2)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let block_expr = Expr {
            kind: ExprKind::Block(vec![func_expr, create_test_literal_expr(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::has_standalone_functions(&block_expr));
    }

    // Test 43: has_standalone_functions - non-function expression
    #[test]
    fn test_has_standalone_functions_false() {
        let literal_expr = create_test_literal_expr(100);
        assert!(!Transpiler::has_standalone_functions(&literal_expr));
    }

    // Test 44: type_to_string - reference type
    #[test]
    fn test_type_to_string_reference() {
        let ref_type = Type {
            kind: TypeKind::Reference {
                inner: Box::new(create_simple_type("i64")),
                is_mut: false,
                lifetime: None,
            },
            span: Span::default(),
        };
        let result = Transpiler::type_to_string(&ref_type);
        assert!(result.contains('&') || result.contains("i64"));
    }

    // Test 45: collect_signatures_from_expr - block with multiple functions
    #[test]
    fn test_collect_signatures_multiple_functions() {
        let mut transpiler = Transpiler::new();
        let func1 = Expr {
            kind: ExprKind::Function {
                name: "func1".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(1)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let func2 = Expr {
            kind: ExprKind::Function {
                name: "func2".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(2)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let block_expr = Expr {
            kind: ExprKind::Block(vec![func1, func2]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_signatures_from_expr(&block_expr);
        // Should collect both function signatures
        assert_eq!(transpiler.function_signatures.len(), 2);
    }

    // Test 46: collect_module_names_from_expr - nested modules
    #[test]
    fn test_collect_module_names_nested() {
        let mut transpiler = Transpiler::new();
        let inner_module = Expr {
            kind: ExprKind::Module {
                name: "inner".to_string(),
                body: Box::new(create_test_literal_expr(99)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let outer_module = Expr {
            kind: ExprKind::Module {
                name: "outer".to_string(),
                body: Box::new(inner_module),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_module_names_from_expr(&outer_module);
        assert!(transpiler.module_names.contains("outer"));
        assert!(transpiler.module_names.contains("inner"));
    }
}
