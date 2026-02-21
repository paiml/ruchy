//! Block Categorization for Transpilation
//!
//! This module handles categorization of expressions in blocks:
//! - Block categorization into functions, statements, modules, imports, globals
//! - Statement vs expression detection
//! - Function categorization
//! - Module resolver block detection
//! - Type inference from values
//!
//! **EXTREME TDD Round 67**: Extracted from mod.rs for modularization.

#![allow(clippy::doc_markdown)]

use super::{BlockCategorization, Transpiler};
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Categorize block expressions into functions, statements, modules, imports, globals
    ///
    /// This function performs a multi-pass categorization:
    /// 1. First pass: Collect names of mutable Lets that will become globals
    /// 2. Second pass: Categorize expressions, skipping main() calls and promoted globals
    /// 3. Third pass: Generate static mut declarations for globals and const declarations
    ///
    /// Complexity: 8 (within Toyota Way limits)
    pub fn categorize_block_expressions<'a>(
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
        let globals =
            self.generate_global_declarations(exprs, &global_var_names, &const_var_names)?;

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

    /// Generate global declarations (const and mutable)
    /// Complexity: 6 (within Toyota Way limits)
    fn generate_global_declarations(
        &self,
        exprs: &[Expr],
        global_var_names: &std::collections::HashSet<String>,
        const_var_names: &std::collections::HashSet<String>,
    ) -> Result<Vec<TokenStream>> {
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

        Ok(globals)
    }

    /// Check if expression is a call to `main()` function
    /// Used to prevent stack overflow when both `fun main()` definition and `main()` call exist
    /// Complexity: 2 (within Toyota Way limits)
    pub fn is_call_to_main(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Call { func, .. } => {
                matches!(&func.kind, ExprKind::Identifier(name) if name == "main")
            }
            _ => false,
        }
    }

    /// Categorize a single expression into appropriate category (complexity: 8)
    pub fn categorize_single_expression<'a>(
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
    pub fn categorize_function<'a>(
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
    pub fn categorize_block(
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
    pub fn is_module_resolver_block(&self, block_exprs: &[Expr]) -> bool {
        block_exprs.len() == 2
            && matches!(block_exprs[0].kind, ExprKind::Module { .. })
            && matches!(block_exprs[1].kind, ExprKind::Import { .. })
    }

    /// Categorize general statement expression (complexity: 3)
    pub fn categorize_statement(
        &self,
        expr: &Expr,
        statements: &mut Vec<TokenStream>,
    ) -> Result<()> {
        let stmt = self.transpile_expr(expr)?;
        let stmt_str = stmt.to_string();

        if !stmt_str.trim().ends_with(';') && !stmt_str.trim().ends_with('}') {
            statements.push(quote! { #stmt; });
        } else {
            statements.push(stmt);
        }
        Ok(())
    }

    /// Helper: Infer type token from expression value
    /// Reduces cognitive complexity by extracting duplicated type inference patterns
    /// Complexity: 5 (within Toyota Way limits)
    pub fn infer_type_from_value(value: &Expr) -> TokenStream {
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
                    Self::infer_vec_element_type(&elements[0])
                }
            }
            _ => quote! { i32 },
        }
    }

    /// Infer Vec element type from first element
    /// Complexity: 3 (within Toyota Way limits)
    fn infer_vec_element_type(element: &Expr) -> TokenStream {
        match &element.kind {
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

    /// Check if an expression is a statement (not an expression)
    /// Complexity: 6 (within Toyota Way limits)
    pub fn is_statement_expr(expr: &Expr) -> bool {
        match &expr.kind {
            // Let bindings are statements
            ExprKind::Let { .. } | ExprKind::LetPattern { .. } => true,
            // Assignment operations are statements
            ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } => true,
            // Loops are statements (void/unit type)
            ExprKind::While { .. } | ExprKind::For { .. } | ExprKind::Loop { .. } => true,
            // Function calls that don't return meaningful values (like println)
            ExprKind::Call { func, .. } => Self::is_statement_call(func),
            // If expressions where both branches are statements (return unit)
            ExprKind::If {
                then_branch,
                else_branch,
                ..
            } => Self::is_statement_if(then_branch, else_branch.as_deref()),
            // Blocks containing statements
            ExprKind::Block(exprs) => exprs.iter().any(Self::is_statement_expr),
            // Most other expressions are not statements
            _ => false,
        }
    }

    /// Check if a call expression is a statement call (e.g., println)
    /// Complexity: 2 (within Toyota Way limits)
    fn is_statement_call(func: &Expr) -> bool {
        if let ExprKind::Identifier(name) = &func.kind {
            matches!(name.as_str(), "println" | "print" | "dbg")
        } else {
            false
        }
    }

    /// Check if an if expression is a statement
    /// Complexity: 2 (within Toyota Way limits)
    fn is_statement_if(then_branch: &Expr, else_branch: Option<&Expr>) -> bool {
        // If both branches are statements, the whole if is a statement
        Self::is_statement_expr(then_branch) && else_branch.is_none_or(Self::is_statement_expr)
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

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn float_expr(f: f64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Float(f)))
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn bool_expr(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn block_expr(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn let_expr(name: &str, value: Expr, is_mutable: bool) -> Expr {
        make_expr(ExprKind::Let {
            name: name.to_string(),
            value: Box::new(value),
            body: Box::new(int_expr(0)),
            type_annotation: None,
            is_mutable,
            else_block: None,
        })
    }

    fn assign_expr(target: Expr, value: Expr) -> Expr {
        make_expr(ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    fn call_expr(func_name: &str, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Call {
            func: Box::new(ident_expr(func_name)),
            args,
        })
    }

    fn while_expr(condition: Expr, body: Expr) -> Expr {
        make_expr(ExprKind::While {
            condition: Box::new(condition),
            body: Box::new(body),
            label: None,
        })
    }

    fn for_expr(var: &str, iter: Expr, body: Expr) -> Expr {
        make_expr(ExprKind::For {
            label: None,
            var: var.to_string(),
            pattern: None,
            iter: Box::new(iter),
            body: Box::new(body),
        })
    }

    fn loop_expr(body: Expr) -> Expr {
        make_expr(ExprKind::Loop {
            body: Box::new(body),
            label: None,
        })
    }

    fn if_expr(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        make_expr(ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn func_expr(name: &str, body: Expr) -> Expr {
        make_expr(ExprKind::Function {
            name: name.to_string(),
            type_params: vec![],
            params: vec![],
            return_type: None,
            body: Box::new(body),
            is_async: false,
            is_pub: false,
        })
    }

    fn module_expr(name: &str, body: Expr) -> Expr {
        make_expr(ExprKind::Module {
            name: name.to_string(),
            body: Box::new(body),
        })
    }

    fn import_expr(module: &str) -> Expr {
        make_expr(ExprKind::Import {
            module: module.to_string(),
            items: None,
        })
    }

    // ========================================================================
    // is_call_to_main tests
    // ========================================================================

    #[test]
    fn test_is_call_to_main_true() {
        let main_call = call_expr("main", vec![]);
        assert!(Transpiler::is_call_to_main(&main_call));
    }

    #[test]
    fn test_is_call_to_main_false_other_func() {
        let other_call = call_expr("other_func", vec![]);
        assert!(!Transpiler::is_call_to_main(&other_call));
    }

    #[test]
    fn test_is_call_to_main_false_not_call() {
        let literal = int_expr(42);
        assert!(!Transpiler::is_call_to_main(&literal));
    }

    #[test]
    fn test_is_call_to_main_false_identifier() {
        let ident = ident_expr("main");
        assert!(!Transpiler::is_call_to_main(&ident));
    }

    // ========================================================================
    // is_statement_expr tests
    // ========================================================================

    #[test]
    fn test_is_statement_expr_let() {
        let expr = let_expr("x", int_expr(42), false);
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_let_mutable() {
        let expr = let_expr("x", int_expr(42), true);
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_assign() {
        let expr = assign_expr(ident_expr("x"), int_expr(42));
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_while() {
        let expr = while_expr(bool_expr(true), int_expr(1));
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_for() {
        let iter = make_expr(ExprKind::Range {
            start: Box::new(int_expr(0)),
            end: Box::new(int_expr(10)),
            inclusive: false,
        });
        let expr = for_expr("i", iter, int_expr(1));
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_loop() {
        let expr = loop_expr(int_expr(1));
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_println_call() {
        let expr = call_expr("println", vec![string_expr("hello")]);
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_print_call() {
        let expr = call_expr("print", vec![string_expr("hello")]);
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_dbg_call() {
        let expr = call_expr("dbg", vec![int_expr(42)]);
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_regular_call_false() {
        let expr = call_expr("my_func", vec![]);
        assert!(!Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_if_both_branches_statements() {
        let then_branch = assign_expr(ident_expr("x"), int_expr(1));
        let else_branch = assign_expr(ident_expr("x"), int_expr(2));
        let expr = if_expr(bool_expr(true), then_branch, Some(else_branch));
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_if_then_statement_no_else() {
        let then_branch = assign_expr(ident_expr("x"), int_expr(1));
        let expr = if_expr(bool_expr(true), then_branch, None);
        assert!(Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_if_else_is_expression() {
        let then_branch = assign_expr(ident_expr("x"), int_expr(1));
        let else_branch = int_expr(2); // This is an expression, not a statement
        let expr = if_expr(bool_expr(true), then_branch, Some(else_branch));
        assert!(!Transpiler::is_statement_expr(&expr));
    }

    #[test]
    fn test_is_statement_expr_block_with_statement() {
        let block = block_expr(vec![let_expr("x", int_expr(1), false), int_expr(42)]);
        assert!(Transpiler::is_statement_expr(&block));
    }

    #[test]
    fn test_is_statement_expr_block_all_expressions() {
        let block = block_expr(vec![int_expr(1), int_expr(2)]);
        assert!(!Transpiler::is_statement_expr(&block));
    }

    #[test]
    fn test_is_statement_expr_literal_false() {
        assert!(!Transpiler::is_statement_expr(&int_expr(42)));
        assert!(!Transpiler::is_statement_expr(&float_expr(3.14)));
        assert!(!Transpiler::is_statement_expr(&string_expr("hello")));
        assert!(!Transpiler::is_statement_expr(&bool_expr(true)));
    }

    #[test]
    fn test_is_statement_expr_identifier_false() {
        assert!(!Transpiler::is_statement_expr(&ident_expr("x")));
    }

    // ========================================================================
    // infer_type_from_value tests
    // ========================================================================

    #[test]
    fn test_infer_type_integer() {
        let result = Transpiler::infer_type_from_value(&int_expr(42));
        assert_eq!(result.to_string(), "i32");
    }

    #[test]
    fn test_infer_type_float() {
        let result = Transpiler::infer_type_from_value(&float_expr(3.14));
        assert_eq!(result.to_string(), "f64");
    }

    #[test]
    fn test_infer_type_string() {
        let result = Transpiler::infer_type_from_value(&string_expr("hello"));
        assert_eq!(result.to_string(), "& str");
    }

    #[test]
    fn test_infer_type_bool() {
        let result = Transpiler::infer_type_from_value(&bool_expr(true));
        assert_eq!(result.to_string(), "bool");
    }

    #[test]
    fn test_infer_type_empty_list() {
        let list = make_expr(ExprKind::List(vec![]));
        let result = Transpiler::infer_type_from_value(&list);
        assert_eq!(result.to_string(), "Vec < i32 >");
    }

    #[test]
    fn test_infer_type_int_list() {
        let list = make_expr(ExprKind::List(vec![int_expr(1), int_expr(2)]));
        let result = Transpiler::infer_type_from_value(&list);
        assert_eq!(result.to_string(), "Vec < i32 >");
    }

    #[test]
    fn test_infer_type_float_list() {
        let list = make_expr(ExprKind::List(vec![float_expr(1.0), float_expr(2.0)]));
        let result = Transpiler::infer_type_from_value(&list);
        assert_eq!(result.to_string(), "Vec < f64 >");
    }

    #[test]
    fn test_infer_type_string_list() {
        let list = make_expr(ExprKind::List(vec![string_expr("a"), string_expr("b")]));
        let result = Transpiler::infer_type_from_value(&list);
        assert_eq!(result.to_string(), "Vec < String >");
    }

    #[test]
    fn test_infer_type_bool_list() {
        let list = make_expr(ExprKind::List(vec![bool_expr(true), bool_expr(false)]));
        let result = Transpiler::infer_type_from_value(&list);
        assert_eq!(result.to_string(), "Vec < bool >");
    }

    #[test]
    fn test_infer_type_nested_list() {
        let inner = make_expr(ExprKind::List(vec![int_expr(1)]));
        let list = make_expr(ExprKind::List(vec![inner]));
        let result = Transpiler::infer_type_from_value(&list);
        assert_eq!(result.to_string(), "Vec < Vec < i32 >>");
    }

    #[test]
    fn test_infer_type_identifier_default() {
        let result = Transpiler::infer_type_from_value(&ident_expr("x"));
        assert_eq!(result.to_string(), "i32");
    }

    // ========================================================================
    // is_module_resolver_block tests
    // ========================================================================

    #[test]
    fn test_is_module_resolver_block_true() {
        let transpiler = Transpiler::new();
        let module = module_expr("test_mod", int_expr(1));
        let import = import_expr("test_mod");
        let exprs = [module, import];
        assert!(transpiler.is_module_resolver_block(&exprs));
    }

    #[test]
    fn test_is_module_resolver_block_wrong_length() {
        let transpiler = Transpiler::new();
        let module = module_expr("test_mod", int_expr(1));
        let exprs = [module];
        assert!(!transpiler.is_module_resolver_block(&exprs));
    }

    #[test]
    fn test_is_module_resolver_block_wrong_order() {
        let transpiler = Transpiler::new();
        let module = module_expr("test_mod", int_expr(1));
        let import = import_expr("test_mod");
        let exprs = [import, module];
        assert!(!transpiler.is_module_resolver_block(&exprs));
    }

    #[test]
    fn test_is_module_resolver_block_not_module() {
        let transpiler = Transpiler::new();
        let not_module = int_expr(1);
        let import = import_expr("test_mod");
        let exprs = [not_module, import];
        assert!(!transpiler.is_module_resolver_block(&exprs));
    }

    #[test]
    fn test_is_module_resolver_block_not_import() {
        let transpiler = Transpiler::new();
        let module = module_expr("test_mod", int_expr(1));
        let not_import = int_expr(1);
        let exprs = [module, not_import];
        assert!(!transpiler.is_module_resolver_block(&exprs));
    }

    // ========================================================================
    // categorize_block_expressions tests
    // ========================================================================

    #[test]
    fn test_categorize_empty_block() {
        let transpiler = Transpiler::new();
        let result = transpiler.categorize_block_expressions(&[]);
        assert!(result.is_ok());
        let (functions, statements, modules, has_main, main_expr, imports, globals) =
            result.unwrap();
        assert!(functions.is_empty());
        assert!(statements.is_empty());
        assert!(modules.is_empty());
        assert!(!has_main);
        assert!(main_expr.is_none());
        assert!(imports.is_empty());
        assert!(globals.is_empty());
    }

    #[test]
    fn test_categorize_with_function() {
        let transpiler = Transpiler::new();
        let func = func_expr("my_func", int_expr(42));
        let exprs = vec![func];
        let result = transpiler.categorize_block_expressions(&exprs);
        assert!(result.is_ok());
        let (functions, _, _, has_main, main_expr, _, _) = result.unwrap();
        assert_eq!(functions.len(), 1);
        assert!(!has_main);
        assert!(main_expr.is_none());
    }

    #[test]
    fn test_categorize_with_main_function() {
        let transpiler = Transpiler::new();
        let func = func_expr("main", int_expr(42));
        let exprs = vec![func];
        let result = transpiler.categorize_block_expressions(&exprs);
        assert!(result.is_ok());
        let (functions, _, _, has_main, main_expr, _, _) = result.unwrap();
        assert!(functions.is_empty()); // main is not in functions
        assert!(has_main);
        assert!(main_expr.is_some());
    }

    #[test]
    fn test_categorize_with_import() {
        let transpiler = Transpiler::new();
        let import = import_expr("std::io");
        let exprs = vec![import];
        let result = transpiler.categorize_block_expressions(&exprs);
        assert!(result.is_ok());
        let (_, _, _, _, _, imports, _) = result.unwrap();
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn test_categorize_with_statement() {
        let transpiler = Transpiler::new();
        let stmt = int_expr(42);
        let exprs = vec![stmt];
        let result = transpiler.categorize_block_expressions(&exprs);
        assert!(result.is_ok());
        let (_, statements, _, _, _, _, _) = result.unwrap();
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_categorize_skips_main_call_when_main_exists() {
        let transpiler = Transpiler::new();
        let main_func = func_expr("main", int_expr(1));
        let main_call = call_expr("main", vec![]);
        let exprs = vec![main_func, main_call];
        let result = transpiler.categorize_block_expressions(&exprs);
        assert!(result.is_ok());
        let (_, statements, _, has_main, _, _, _) = result.unwrap();
        assert!(has_main);
        // main() call should be skipped, so statements should be empty
        assert!(statements.is_empty());
    }

    // ========================================================================
    // categorize_statement tests
    // ========================================================================

    #[test]
    fn test_categorize_statement_adds_semicolon() {
        let transpiler = Transpiler::new();
        let mut statements = Vec::new();
        let expr = int_expr(42);
        let result = transpiler.categorize_statement(&expr, &mut statements);
        assert!(result.is_ok());
        assert_eq!(statements.len(), 1);
        let stmt_str = statements[0].to_string();
        assert!(stmt_str.ends_with(';'));
    }

    #[test]
    fn test_categorize_statement_no_double_semicolon() {
        let transpiler = Transpiler::new();
        let mut statements = Vec::new();
        // A block ends with }, so it shouldn't get a semicolon
        let block = block_expr(vec![int_expr(1)]);
        let result = transpiler.categorize_statement(&block, &mut statements);
        assert!(result.is_ok());
        assert_eq!(statements.len(), 1);
    }

    // ========================================================================
    // categorize_function tests
    // ========================================================================

    #[test]
    fn test_categorize_function_regular() {
        let transpiler = Transpiler::new();
        let func = func_expr("my_func", int_expr(42));
        let mut functions = Vec::new();
        let mut has_main = false;
        let mut main_expr = None;
        let result = transpiler.categorize_function(
            &func,
            "my_func",
            &mut functions,
            &mut has_main,
            &mut main_expr,
        );
        assert!(result.is_ok());
        assert_eq!(functions.len(), 1);
        assert!(!has_main);
        assert!(main_expr.is_none());
    }

    #[test]
    fn test_categorize_function_main() {
        let transpiler = Transpiler::new();
        let func = func_expr("main", int_expr(42));
        let mut functions = Vec::new();
        let mut has_main = false;
        let mut main_expr = None;
        let result = transpiler.categorize_function(
            &func,
            "main",
            &mut functions,
            &mut has_main,
            &mut main_expr,
        );
        assert!(result.is_ok());
        assert!(functions.is_empty()); // main is not added to functions
        assert!(has_main);
        assert!(main_expr.is_some());
    }

    // ========================================================================
    // generate_global_declarations tests
    // ========================================================================

    fn typed_let_expr(
        name: &str,
        value: Expr,
        is_mutable: bool,
        type_ann: Option<crate::frontend::ast::Type>,
    ) -> Expr {
        make_expr(ExprKind::Let {
            name: name.to_string(),
            value: Box::new(value),
            body: Box::new(int_expr(0)),
            type_annotation: type_ann,
            is_mutable,
            else_block: None,
        })
    }

    fn make_type_ann(name: &str) -> crate::frontend::ast::Type {
        crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named(name.to_string()),
            span: crate::frontend::ast::Span::default(),
        }
    }

    #[test]
    fn test_generate_global_const_declaration() {
        let transpiler = Transpiler::new();
        let exprs = vec![typed_let_expr(
            "MAX",
            int_expr(100),
            false,
            Some(make_type_ann("i32")),
        )];
        let global_var_names = std::collections::HashSet::new();
        let mut const_var_names = std::collections::HashSet::new();
        const_var_names.insert("MAX".to_string());
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert_eq!(result.len(), 1, "Should generate one const declaration");
        let code = result[0].to_string();
        assert!(code.contains("const"), "Should be a const declaration");
        assert!(code.contains("MAX"), "Should contain const name");
        assert!(code.contains("i32"), "Should contain explicit type");
    }

    #[test]
    fn test_generate_global_const_inferred_type() {
        let transpiler = Transpiler::new();
        // No type annotation - type should be inferred from literal
        let exprs = vec![typed_let_expr("PI", float_expr(3.14), false, None)];
        let global_var_names = std::collections::HashSet::new();
        let mut const_var_names = std::collections::HashSet::new();
        const_var_names.insert("PI".to_string());
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains("const"), "Should generate const");
        assert!(code.contains("PI"), "Should have name PI");
    }

    #[test]
    fn test_generate_global_mutable_declaration() {
        let transpiler = Transpiler::new();
        let exprs = vec![typed_let_expr(
            "counter",
            int_expr(0),
            true,
            Some(make_type_ann("i32")),
        )];
        let mut global_var_names = std::collections::HashSet::new();
        global_var_names.insert("counter".to_string());
        let const_var_names = std::collections::HashSet::new();
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert_eq!(result.len(), 1, "Should generate one mutable global");
        let code = result[0].to_string();
        assert!(code.contains("static"), "Should be static");
        assert!(code.contains("LazyLock"), "Should use LazyLock");
        assert!(code.contains("Mutex"), "Should use Mutex");
        assert!(code.contains("counter"), "Should contain variable name");
    }

    #[test]
    fn test_generate_global_mutable_inferred_type() {
        let transpiler = Transpiler::new();
        let exprs = vec![typed_let_expr("count", string_expr("hello"), true, None)];
        let mut global_var_names = std::collections::HashSet::new();
        global_var_names.insert("count".to_string());
        let const_var_names = std::collections::HashSet::new();
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains("LazyLock"), "Should use LazyLock pattern");
    }

    #[test]
    fn test_generate_global_empty_sets() {
        let transpiler = Transpiler::new();
        let exprs = vec![let_expr("x", int_expr(1), false)];
        let global_var_names = std::collections::HashSet::new();
        let const_var_names = std::collections::HashSet::new();
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert!(
            result.is_empty(),
            "Empty name sets should produce no globals"
        );
    }

    #[test]
    fn test_generate_global_both_const_and_mutable() {
        let transpiler = Transpiler::new();
        let exprs = vec![
            typed_let_expr("MAX", int_expr(100), false, Some(make_type_ann("i32"))),
            typed_let_expr("count", int_expr(0), true, Some(make_type_ann("i32"))),
        ];
        let mut global_var_names = std::collections::HashSet::new();
        global_var_names.insert("count".to_string());
        let mut const_var_names = std::collections::HashSet::new();
        const_var_names.insert("MAX".to_string());
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert_eq!(result.len(), 2, "Should generate both const and mutable");
    }

    #[test]
    fn test_generate_global_non_matching_expr_skipped() {
        let transpiler = Transpiler::new();
        // An expression that is not a Let should be skipped
        let exprs = vec![call_expr("println", vec![string_expr("hi")])];
        let mut const_var_names = std::collections::HashSet::new();
        const_var_names.insert("X".to_string());
        let global_var_names = std::collections::HashSet::new();
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert!(result.is_empty(), "Non-let exprs should be skipped");
    }

    #[test]
    fn test_generate_global_immutable_let_not_in_global_names() {
        let transpiler = Transpiler::new();
        // Let is immutable, so it won't match in global_var_names (which requires is_mutable)
        let exprs = vec![typed_let_expr(
            "x",
            int_expr(1),
            false,
            Some(make_type_ann("i32")),
        )];
        let mut global_var_names = std::collections::HashSet::new();
        global_var_names.insert("x".to_string());
        let const_var_names = std::collections::HashSet::new();
        let result = transpiler
            .generate_global_declarations(&exprs, &global_var_names, &const_var_names)
            .unwrap();
        assert!(
            result.is_empty(),
            "Immutable let should not generate mutable global"
        );
    }
}
