//! AST Analysis and Collection Functions
//!
//! This module handles AST analysis tasks for the transpiler:
//! - Mutability analysis: detecting which variables need `mut`
//! - Const collection: tracking const declarations
//! - Function signature collection: building signature map for type coercion
//! - Module name collection: tracking module declarations
//! - Import detection and resolution
//! - AST content detection (`HashMap`, `DataFrame`, etc.)
//!
//! **EXTREME TDD Round 66**: Extracted from mod.rs for modularization.

#![allow(clippy::doc_markdown)]

use super::{FunctionSignature, Transpiler};
use crate::backend::module_resolver::ModuleResolver;
use crate::frontend::ast::{Expr, ExprKind, Type, TypeKind};
use anyhow::Result;

impl Transpiler {
    // ========================================================================
    // Mutability Analysis
    // ========================================================================

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

    /// Analyze mutability for a single expression (complexity: 9)
    pub fn analyze_expr_mutability(&mut self, expr: &Expr) {
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
    pub fn mark_target_mutable(&mut self, target: &Expr) {
        if let ExprKind::Identifier(name) = &target.kind {
            self.mutable_vars.insert(name.clone());
        }
    }

    /// Analyze mutability for block expressions (complexity: 1)
    pub fn analyze_block_mutability(&mut self, exprs: &[Expr]) {
        for e in exprs {
            self.analyze_expr_mutability(e);
        }
    }

    /// Analyze mutability for if expressions (complexity: 2)
    pub fn analyze_if_mutability(
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
    pub fn analyze_two_expr_mutability(&mut self, expr1: &Expr, expr2: &Expr) {
        self.analyze_expr_mutability(expr1);
        self.analyze_expr_mutability(expr2);
    }

    /// Analyze mutability for match expressions (complexity: 1)
    pub fn analyze_match_mutability(&mut self, expr: &Expr, arms: &[crate::frontend::ast::MatchArm]) {
        self.analyze_expr_mutability(expr);
        for arm in arms {
            self.analyze_expr_mutability(&arm.body);
        }
    }

    /// Analyze mutability for call expressions (complexity: 1)
    pub fn analyze_call_mutability(&mut self, func: &Expr, args: &[Expr]) {
        self.analyze_expr_mutability(func);
        for arg in args {
            self.analyze_expr_mutability(arg);
        }
    }

    // ========================================================================
    // Const Declaration Collection
    // ========================================================================

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

    // ========================================================================
    // Function Signature Collection
    // ========================================================================

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

    /// Helper to recursively collect function signatures from an expression
    pub fn collect_signatures_from_expr(&mut self, expr: &Expr) {
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

    /// Convert a Type AST node to a string representation
    pub fn type_to_string(ty: &Type) -> String {
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

    // ========================================================================
    // Module Name Collection
    // ========================================================================

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
    pub fn collect_module_names_from_expr(&mut self, expr: &Expr) {
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

    // ========================================================================
    // Import Resolution
    // ========================================================================

    /// Resolves file imports in the AST using `ModuleResolver`
    #[allow(dead_code)]
    pub fn resolve_imports(&self, expr: &Expr) -> Result<Expr> {
        // For now, just use default search paths since we don't have file context here
        let mut resolver = ModuleResolver::new();
        resolver.resolve_imports(expr.clone())
    }

    /// Resolves file imports with a specific file context for search paths
    pub fn resolve_imports_with_context(
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
    pub fn contains_imports(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Import { .. }
            | ExprKind::ImportAll { .. }
            | ExprKind::ImportDefault { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_imports),
            _ => false,
        }
    }

    /// Check if an expression tree contains any file imports (local .ruchy files)
    pub fn contains_file_imports(expr: &Expr) -> bool {
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
    pub fn is_standard_library(module: &str) -> bool {
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

    // ========================================================================
    // AST Content Detection
    // ========================================================================

    /// Check if AST contains `HashMap` operations requiring `std::collections::HashMap` import
    pub fn contains_hashmap(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::ObjectLiteral { .. } => true,
            ExprKind::Call { func, .. } => {
                // Check for HashMap methods like .get(), .insert(), etc.
                if let ExprKind::Identifier(name) = &func.kind {
                    if name.starts_with("HashMap") {
                        return true;
                    }
                }
                false
            }
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_hashmap),
            ExprKind::Let { value, body, .. } => {
                Self::contains_hashmap(value) || Self::contains_hashmap(body)
            }
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
            _ => false,
        }
    }

    /// Check if AST contains `DataFrame` operations requiring polars import
    pub fn contains_dataframe(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Call { func, .. } => {
                // Check for DataFrame constructors and methods
                if let ExprKind::Identifier(name) = &func.kind {
                    if name.starts_with("DataFrame") || name == "col" || name == "lit" {
                        return true;
                    }
                }
                // Check for qualified names like DataFrame::new
                if let ExprKind::QualifiedName { module, .. } = &func.kind {
                    if module == "DataFrame" {
                        return true;
                    }
                }
                false
            }
            ExprKind::MethodCall { receiver, method, .. } => {
                // DataFrame-specific methods (don't exist on iterators)
                let is_df_only_method = matches!(
                    method.as_str(),
                    "select"
                        | "groupby"
                        | "group_by"
                        | "agg"
                        | "column"
                        | "build"
                        | "rows"
                        | "columns"
                        | "lazy"
                        | "collect"
                        | "with_column"
                        | "drop"
                        | "join"
                        | "vstack"
                        | "hstack"
                );
                if is_df_only_method {
                    return true;
                }
                // Common methods (filter, sum, min, max, etc.) - only count as DataFrame
                // if the receiver is already detected as a DataFrame
                let is_common_method = matches!(
                    method.as_str(),
                    "filter" | "sort" | "head" | "tail" | "mean" | "std" | "min" | "max" | "sum" | "count"
                );
                if is_common_method {
                    // Only return true if receiver is a DataFrame
                    return Self::contains_dataframe(receiver);
                }
                false
            }
            ExprKind::Block(exprs) => exprs.iter().any(Self::contains_dataframe),
            ExprKind::Let { value, body, .. } => {
                Self::contains_dataframe(value) || Self::contains_dataframe(body)
            }
            ExprKind::Function { body, .. } => Self::contains_dataframe(body),
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
    pub fn has_standalone_functions(expr: &Expr) -> bool {
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
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn block_expr(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn assign_expr(target: Expr, value: Expr) -> Expr {
        make_expr(ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    // ========================================================================
    // Mutability Analysis Tests
    // ========================================================================

    #[test]
    fn test_analyze_mutability_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.analyze_mutability(&[]);
        assert!(transpiler.mutable_vars.is_empty());
    }

    #[test]
    fn test_analyze_mutability_simple_assign() {
        let mut transpiler = Transpiler::new();
        let exprs = vec![assign_expr(ident_expr("x"), int_expr(5))];
        transpiler.analyze_mutability(&exprs);
        assert!(transpiler.mutable_vars.contains("x"));
    }

    #[test]
    fn test_analyze_mutability_block() {
        let mut transpiler = Transpiler::new();
        let exprs = vec![block_expr(vec![assign_expr(ident_expr("y"), int_expr(10))])];
        transpiler.analyze_mutability(&exprs);
        assert!(transpiler.mutable_vars.contains("y"));
    }

    #[test]
    fn test_mark_target_mutable() {
        let mut transpiler = Transpiler::new();
        transpiler.mark_target_mutable(&ident_expr("counter"));
        assert!(transpiler.mutable_vars.contains("counter"));
    }

    #[test]
    fn test_mark_target_mutable_non_ident() {
        let mut transpiler = Transpiler::new();
        transpiler.mark_target_mutable(&int_expr(42));
        assert!(transpiler.mutable_vars.is_empty());
    }

    #[test]
    fn test_analyze_block_mutability_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.analyze_block_mutability(&[]);
        assert!(transpiler.mutable_vars.is_empty());
    }

    #[test]
    fn test_analyze_if_mutability() {
        let mut transpiler = Transpiler::new();
        let condition = int_expr(1);
        let then_branch = assign_expr(ident_expr("a"), int_expr(1));
        let else_branch = assign_expr(ident_expr("b"), int_expr(2));
        transpiler.analyze_if_mutability(&condition, &then_branch, Some(&else_branch));
        assert!(transpiler.mutable_vars.contains("a"));
        assert!(transpiler.mutable_vars.contains("b"));
    }

    #[test]
    fn test_analyze_two_expr_mutability() {
        let mut transpiler = Transpiler::new();
        let expr1 = assign_expr(ident_expr("x"), int_expr(1));
        let expr2 = assign_expr(ident_expr("y"), int_expr(2));
        transpiler.analyze_two_expr_mutability(&expr1, &expr2);
        assert!(transpiler.mutable_vars.contains("x"));
        assert!(transpiler.mutable_vars.contains("y"));
    }

    #[test]
    fn test_analyze_call_mutability() {
        let mut transpiler = Transpiler::new();
        let func = ident_expr("foo");
        let args = vec![assign_expr(ident_expr("arg"), int_expr(1))];
        transpiler.analyze_call_mutability(&func, &args);
        assert!(transpiler.mutable_vars.contains("arg"));
    }

    // ========================================================================
    // Collection Tests
    // ========================================================================

    #[test]
    fn test_collect_const_declarations_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.collect_const_declarations(&[]);
        assert!(transpiler
            .const_vars
            .read()
            .expect("rwlock")
            .is_empty());
    }

    #[test]
    fn test_collect_function_signatures_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.collect_function_signatures(&[]);
        assert!(transpiler.function_signatures.is_empty());
    }

    #[test]
    fn test_collect_module_names_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.collect_module_names(&[]);
        assert!(transpiler.module_names.is_empty());
    }

    #[test]
    fn test_type_to_string_named() {
        let ty = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        assert_eq!(Transpiler::type_to_string(&ty), "i32");
    }

    #[test]
    fn test_type_to_string_generic() {
        let inner = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let ty = Type {
            kind: TypeKind::Generic {
                base: "Option".to_string(),
                params: vec![inner],
            },
            span: Span::default(),
        };
        assert_eq!(Transpiler::type_to_string(&ty), "Option<i32>");
    }

    #[test]
    fn test_type_to_string_reference() {
        let inner = Type {
            kind: TypeKind::Named("str".to_string()),
            span: Span::default(),
        };
        let ty = Type {
            kind: TypeKind::Reference {
                inner: Box::new(inner),
                is_mut: false,
                lifetime: None,
            },
            span: Span::default(),
        };
        assert_eq!(Transpiler::type_to_string(&ty), "&str");
    }

    // ========================================================================
    // Import Detection Tests
    // ========================================================================

    #[test]
    fn test_contains_imports_false() {
        let expr = int_expr(42);
        assert!(!Transpiler::contains_imports(&expr));
    }

    #[test]
    fn test_contains_imports_true() {
        let expr = make_expr(ExprKind::Import {
            module: "std::fs".to_string(),
            items: None,
        });
        assert!(Transpiler::contains_imports(&expr));
    }

    #[test]
    fn test_contains_file_imports_relative() {
        let expr = make_expr(ExprKind::Import {
            module: "./helper".to_string(),
            items: None,
        });
        assert!(Transpiler::contains_file_imports(&expr));
    }

    #[test]
    fn test_contains_file_imports_parent() {
        let expr = make_expr(ExprKind::Import {
            module: "../utils".to_string(),
            items: None,
        });
        assert!(Transpiler::contains_file_imports(&expr));
    }

    #[test]
    fn test_contains_file_imports_std_false() {
        let expr = make_expr(ExprKind::Import {
            module: "std".to_string(),
            items: None,
        });
        assert!(!Transpiler::contains_file_imports(&expr));
    }

    #[test]
    fn test_is_standard_library_std() {
        assert!(Transpiler::is_standard_library("std"));
        assert!(Transpiler::is_standard_library("core"));
        assert!(Transpiler::is_standard_library("polars"));
    }

    #[test]
    fn test_is_standard_library_false() {
        assert!(!Transpiler::is_standard_library("my_module"));
        assert!(!Transpiler::is_standard_library("helper"));
    }

    // ========================================================================
    // AST Content Detection Tests
    // ========================================================================

    #[test]
    fn test_contains_hashmap_false() {
        let expr = int_expr(42);
        assert!(!Transpiler::contains_hashmap(&expr));
    }

    #[test]
    fn test_contains_hashmap_object_literal() {
        let expr = make_expr(ExprKind::ObjectLiteral { fields: vec![] });
        assert!(Transpiler::contains_hashmap(&expr));
    }

    #[test]
    fn test_contains_dataframe_false() {
        let expr = int_expr(42);
        assert!(!Transpiler::contains_dataframe(&expr));
    }

    #[test]
    fn test_contains_dataframe_col() {
        let expr = make_expr(ExprKind::Call {
            func: Box::new(ident_expr("col")),
            args: vec![],
        });
        assert!(Transpiler::contains_dataframe(&expr));
    }

    #[test]
    fn test_has_standalone_functions_false() {
        let expr = int_expr(42);
        assert!(!Transpiler::has_standalone_functions(&expr));
    }

    #[test]
    fn test_has_standalone_functions_single() {
        let func = make_expr(ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![],
            body: Box::new(int_expr(1)),
            return_type: None,
            is_async: false,
            is_pub: false,
        });
        assert!(Transpiler::has_standalone_functions(&func));
    }

    #[test]
    fn test_has_standalone_functions_block() {
        let func = make_expr(ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![],
            body: Box::new(int_expr(1)),
            return_type: None,
            is_async: false,
            is_pub: false,
        });
        let block = block_expr(vec![func]);
        assert!(Transpiler::has_standalone_functions(&block));
    }
}
