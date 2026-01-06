//! Comprehension Transpilation
//!
//! This module handles transpilation of list, set, and dict comprehensions
//! to Rust iterator chains.
//!
//! **EXTREME TDD Round 53**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{ComprehensionClause, Expr};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Check if variable is a complex pattern (tuple, wildcard, etc.)
    fn is_complex_pattern(var: &str) -> bool {
        var.contains('(') || var.contains(',') || var == "_"
    }

    /// Parse variable pattern into `TokenStream`
    fn parse_var_pattern(var: &str) -> Result<TokenStream> {
        if Self::is_complex_pattern(var) {
            // Complex pattern - parse as TokenStream
            var.parse()
                .map_err(|e| anyhow::anyhow!("Invalid pattern '{var}': {e}"))
        } else {
            // Simple identifier
            let var_ident = format_ident!("{}", var);
            Ok(quote! { #var_ident })
        }
    }

    /// Transpile list comprehension with nested clauses
    /// Complexity: 9 (within Toyota Way limits)
    pub fn transpile_list_comprehension_new(
        &self,
        element: &Expr,
        clauses: &[ComprehensionClause],
    ) -> Result<TokenStream> {
        if clauses.is_empty() {
            bail!("List comprehension must have at least one for clause");
        }

        let element_tokens = self.transpile_expr(element)?;
        let mut result_tokens = None;

        for (i, clause) in clauses.iter().enumerate() {
            let iter_tokens = self.transpile_expr(&clause.iterable)?;
            let var_pattern = Self::parse_var_pattern(&clause.variable)?;

            if i == 0 {
                result_tokens = Some(Self::build_first_clause(
                    &iter_tokens,
                    &var_pattern,
                    clause.condition.as_ref().map(|v| &**v),
                    self,
                )?);
            } else {
                let prev_chain = result_tokens
                    .expect("result_tokens should be Some after first clause iteration (i > 0)");
                let outer_var = &clauses[i - 1].variable;
                let outer_pattern = Self::parse_var_pattern(outer_var)?;

                result_tokens = Some(Self::build_nested_clause(
                    &prev_chain,
                    &outer_pattern,
                    &iter_tokens,
                    &var_pattern,
                    clause.condition.as_ref().map(|v| &**v),
                    self,
                )?);
            }
        }

        let final_var = &clauses
            .last()
            .expect("clauses is non-empty, validated at function entry")
            .variable;
        let final_pattern = Self::parse_var_pattern(final_var)?;

        let final_chain = result_tokens
            .expect("result_tokens should be Some after processing at least one clause");
        Ok(quote! {
            #final_chain
                .map(|#final_pattern| #element_tokens)
                .collect::<Vec<_>>()
        })
    }

    /// Build first clause of comprehension chain
    fn build_first_clause(
        iter_tokens: &TokenStream,
        var_pattern: &TokenStream,
        condition: Option<&Expr>,
        transpiler: &Transpiler,
    ) -> Result<TokenStream> {
        if let Some(cond) = condition {
            let cond_tokens = transpiler.transpile_expr(cond)?;
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter(|#var_pattern| #cond_tokens)
            })
        } else {
            Ok(quote! {
                #iter_tokens.into_iter()
            })
        }
    }

    /// Build nested clause using flat_map
    fn build_nested_clause(
        prev_chain: &TokenStream,
        outer_pattern: &TokenStream,
        iter_tokens: &TokenStream,
        var_pattern: &TokenStream,
        condition: Option<&Expr>,
        transpiler: &Transpiler,
    ) -> Result<TokenStream> {
        if let Some(cond) = condition {
            let cond_tokens = transpiler.transpile_expr(cond)?;
            Ok(quote! {
                #prev_chain
                    .flat_map(|#outer_pattern| {
                        #iter_tokens
                            .into_iter()
                            .filter(|#var_pattern| #cond_tokens)
                    })
            })
        } else {
            Ok(quote! {
                #prev_chain
                    .flat_map(|#outer_pattern| #iter_tokens.into_iter())
            })
        }
    }

    /// Transpiles list comprehensions (legacy single-clause)
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_list_comprehension(
        &self,
        expr: &Expr,
        var: &str,
        iter: &Expr,
        filter: Option<&Expr>,
    ) -> Result<TokenStream> {
        let iter_tokens = self.transpile_expr(iter)?;
        let expr_tokens = self.transpile_expr(expr)?;
        let is_pattern = var.contains('(') && var.contains(')');

        if is_pattern {
            self.transpile_pattern_list_comprehension(var, &iter_tokens, &expr_tokens, filter)
        } else {
            self.transpile_simple_list_comprehension(var, &iter_tokens, &expr_tokens, filter)
        }
    }

    /// Transpile pattern-based list comprehension
    fn transpile_pattern_list_comprehension(
        &self,
        var: &str,
        iter_tokens: &TokenStream,
        expr_tokens: &TokenStream,
        filter: Option<&Expr>,
    ) -> Result<TokenStream> {
        let inner_var = if let Some(start) = var.find('(') {
            if let Some(end) = var.rfind(')') {
                &var[start + 1..end]
            } else {
                var
            }
        } else {
            var
        };

        let inner_var_ident = format_ident!("{}", inner_var);
        let pattern_tokens: TokenStream = var.parse().unwrap_or_else(|_| {
            let ident = format_ident!("{}", var);
            quote! { #ident }
        });

        if let Some(filter_expr) = filter {
            let filter_tokens = self.transpile_expr(filter_expr)?;
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter_map(|item| if let #pattern_tokens = item { Some(#inner_var_ident) } else { None })
                    .filter(|#inner_var_ident| #filter_tokens)
                    .map(|#inner_var_ident| #expr_tokens)
                    .collect::<Vec<_>>()
            })
        } else {
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter_map(|item| if let #pattern_tokens = item { Some(#inner_var_ident) } else { None })
                    .map(|#inner_var_ident| #expr_tokens)
                    .collect::<Vec<_>>()
            })
        }
    }

    /// Transpile simple list comprehension
    fn transpile_simple_list_comprehension(
        &self,
        var: &str,
        iter_tokens: &TokenStream,
        expr_tokens: &TokenStream,
        filter: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var);
        if let Some(filter_expr) = filter {
            let filter_tokens = self.transpile_expr(filter_expr)?;
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter(|#var_ident| #filter_tokens)
                    .map(|#var_ident| #expr_tokens)
                    .collect::<Vec<_>>()
            })
        } else {
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .map(|#var_ident| #expr_tokens)
                    .collect::<Vec<_>>()
            })
        }
    }

    /// Transpile set comprehension with nested clauses
    /// Complexity: 9 (within Toyota Way limits)
    pub fn transpile_set_comprehension_new(
        &self,
        element: &Expr,
        clauses: &[ComprehensionClause],
    ) -> Result<TokenStream> {
        if clauses.is_empty() {
            bail!("Set comprehension must have at least one for clause");
        }

        let element_tokens = self.transpile_expr(element)?;
        let mut result_tokens = None;

        for (i, clause) in clauses.iter().enumerate() {
            let iter_tokens = self.transpile_expr(&clause.iterable)?;
            let var_pattern = Self::parse_var_pattern(&clause.variable)?;

            if i == 0 {
                result_tokens = Some(Self::build_first_clause(
                    &iter_tokens,
                    &var_pattern,
                    clause.condition.as_ref().map(|v| &**v),
                    self,
                )?);
            } else {
                let prev_chain = result_tokens
                    .expect("result_tokens should be Some after first clause iteration (i > 0)");
                let outer_var = &clauses[i - 1].variable;
                let outer_pattern = Self::parse_var_pattern(outer_var)?;

                result_tokens = Some(Self::build_nested_clause(
                    &prev_chain,
                    &outer_pattern,
                    &iter_tokens,
                    &var_pattern,
                    clause.condition.as_ref().map(|v| &**v),
                    self,
                )?);
            }
        }

        let final_var = &clauses
            .last()
            .expect("clauses is non-empty, validated at function entry")
            .variable;
        let final_pattern = Self::parse_var_pattern(final_var)?;

        let final_chain = result_tokens
            .expect("result_tokens should be Some after processing at least one clause");
        Ok(quote! {
            #final_chain
                .map(|#final_pattern| #element_tokens)
                .collect::<std::collections::HashSet<_>>()
        })
    }

    /// Transpile set comprehension to Rust iterator chain with `HashSet`
    pub fn transpile_set_comprehension(
        &self,
        expr: &Expr,
        var: &str,
        iter: &Expr,
        filter: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_pattern = Self::parse_tuple_or_simple_pattern(var)?;
        let iter_tokens = self.transpile_expr(iter)?;
        let expr_tokens = self.transpile_expr(expr)?;

        if let Some(filter_expr) = filter {
            let filter_tokens = self.transpile_expr(filter_expr)?;
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter(|#var_pattern| #filter_tokens)
                    .map(|#var_pattern| #expr_tokens)
                    .collect::<std::collections::HashSet<_>>()
            })
        } else {
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .map(|#var_pattern| #expr_tokens)
                    .collect::<std::collections::HashSet<_>>()
            })
        }
    }

    /// Parse tuple pattern or simple identifier
    fn parse_tuple_or_simple_pattern(var: &str) -> Result<TokenStream> {
        if var.starts_with('(') && var.ends_with(')') {
            var.parse()
                .map_err(|e| anyhow::anyhow!("Invalid pattern in comprehension: {e}"))
        } else {
            let var_ident = format_ident!("{}", var);
            Ok(quote! { #var_ident })
        }
    }

    /// Transpile dict comprehension with nested clauses
    /// Complexity: 9 (within Toyota Way limits)
    pub fn transpile_dict_comprehension_new(
        &self,
        key: &Expr,
        value: &Expr,
        clauses: &[ComprehensionClause],
    ) -> Result<TokenStream> {
        if clauses.is_empty() {
            bail!("Dict comprehension must have at least one for clause");
        }

        let key_tokens = self.transpile_expr(key)?;
        let value_tokens = self.transpile_expr(value)?;
        let mut result_tokens = None;

        for (i, clause) in clauses.iter().enumerate() {
            let iter_tokens = self.transpile_expr(&clause.iterable)?;
            let var_pattern = Self::parse_var_pattern(&clause.variable)?;

            if i == 0 {
                result_tokens = Some(Self::build_first_clause(
                    &iter_tokens,
                    &var_pattern,
                    clause.condition.as_ref().map(|v| &**v),
                    self,
                )?);
            } else {
                let prev_chain = result_tokens
                    .expect("result_tokens should be Some after first clause iteration (i > 0)");
                let outer_var = &clauses[i - 1].variable;
                let outer_pattern = Self::parse_var_pattern(outer_var)?;

                result_tokens = Some(Self::build_nested_clause(
                    &prev_chain,
                    &outer_pattern,
                    &iter_tokens,
                    &var_pattern,
                    clause.condition.as_ref().map(|v| &**v),
                    self,
                )?);
            }
        }

        let final_var = &clauses
            .last()
            .expect("clauses is non-empty, validated at function entry")
            .variable;
        let final_pattern = Self::parse_var_pattern(final_var)?;

        let final_chain = result_tokens
            .expect("result_tokens should be Some after processing at least one clause");
        Ok(quote! {
            #final_chain
                .map(|#final_pattern| (#key_tokens, #value_tokens))
                .collect::<std::collections::HashMap<_, _>>()
        })
    }

    /// Transpile dict comprehension to Rust iterator chain with `HashMap`
    pub fn transpile_dict_comprehension(
        &self,
        key: &Expr,
        value: &Expr,
        var: &str,
        iter: &Expr,
        filter: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_pattern = Self::parse_tuple_or_simple_pattern(var)?;
        let iter_tokens = self.transpile_expr(iter)?;
        let key_tokens = self.transpile_expr(key)?;
        let value_tokens = self.transpile_expr(value)?;

        if let Some(filter_expr) = filter {
            let filter_tokens = self.transpile_expr(filter_expr)?;
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .filter(|#var_pattern| #filter_tokens)
                    .map(|#var_pattern| (#key_tokens, #value_tokens))
                    .collect::<std::collections::HashMap<_, _>>()
            })
        } else {
            Ok(quote! {
                #iter_tokens
                    .into_iter()
                    .map(|#var_pattern| (#key_tokens, #value_tokens))
                    .collect::<std::collections::HashMap<_, _>>()
            })
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    // ========================================================================
    // Helper functions
    // ========================================================================

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

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn list_expr(items: Vec<Expr>) -> Expr {
        make_expr(ExprKind::List(items))
    }

    fn range_expr(start: i64, end: i64) -> Expr {
        make_expr(ExprKind::Range {
            start: Box::new(int_expr(start)),
            end: Box::new(int_expr(end)),
            inclusive: false,
        })
    }

    fn binary_expr(left: Expr, op: crate::frontend::ast::BinaryOp, right: Expr) -> Expr {
        make_expr(ExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn make_clause(var: &str, iterable: Expr, condition: Option<Expr>) -> ComprehensionClause {
        ComprehensionClause {
            variable: var.to_string(),
            iterable: Box::new(iterable),
            condition: condition.map(Box::new),
        }
    }

    // ========================================================================
    // is_complex_pattern tests
    // ========================================================================

    #[test]
    fn test_is_complex_pattern_simple() {
        assert!(!Transpiler::is_complex_pattern("x"));
        assert!(!Transpiler::is_complex_pattern("item"));
    }

    #[test]
    fn test_is_complex_pattern_tuple() {
        assert!(Transpiler::is_complex_pattern("(a, b)"));
        assert!(Transpiler::is_complex_pattern("(x, y, z)"));
    }

    #[test]
    fn test_is_complex_pattern_wildcard() {
        assert!(Transpiler::is_complex_pattern("_"));
    }

    #[test]
    fn test_is_complex_pattern_comma() {
        assert!(Transpiler::is_complex_pattern("a, b"));
    }

    #[test]
    fn test_is_complex_pattern_nested() {
        assert!(Transpiler::is_complex_pattern("((a, b), c)"));
    }

    // ========================================================================
    // parse_var_pattern tests
    // ========================================================================

    #[test]
    fn test_parse_var_pattern_simple() {
        let result = Transpiler::parse_var_pattern("x");
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("x"));
    }

    #[test]
    fn test_parse_var_pattern_tuple() {
        let result = Transpiler::parse_var_pattern("(a, b)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_var_pattern_wildcard() {
        let result = Transpiler::parse_var_pattern("_");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_var_pattern_underscore_name() {
        let result = Transpiler::parse_var_pattern("_unused");
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("_unused"));
    }

    // ========================================================================
    // parse_tuple_or_simple_pattern tests
    // ========================================================================

    #[test]
    fn test_parse_tuple_or_simple_pattern_simple() {
        let result = Transpiler::parse_tuple_or_simple_pattern("item");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tuple_or_simple_pattern_tuple() {
        let result = Transpiler::parse_tuple_or_simple_pattern("(k, v)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tuple_or_simple_pattern_triple() {
        let result = Transpiler::parse_tuple_or_simple_pattern("(a, b, c)");
        assert!(result.is_ok());
    }

    // ========================================================================
    // transpile_list_comprehension_new tests
    // ========================================================================

    #[test]
    fn test_list_comprehension_new_single_clause() {
        let transpiler = Transpiler::new();
        let element = ident_expr("x");
        let clauses = vec![make_clause("x", range_expr(0, 10), None)];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("collect"));
        assert!(code.contains("Vec"));
    }

    #[test]
    fn test_list_comprehension_new_with_condition() {
        let transpiler = Transpiler::new();
        let element = ident_expr("x");
        let condition = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Greater,
            int_expr(5),
        );
        let clauses = vec![make_clause("x", range_expr(0, 10), Some(condition))];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    #[test]
    fn test_list_comprehension_new_nested_clauses() {
        let transpiler = Transpiler::new();
        let element = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Multiply,
            ident_expr("y"),
        );
        let clauses = vec![
            make_clause("x", range_expr(0, 3), None),
            make_clause("y", range_expr(0, 3), None),
        ];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("flat_map"));
    }

    #[test]
    fn test_list_comprehension_new_empty_clauses_error() {
        let transpiler = Transpiler::new();
        let element = ident_expr("x");
        let clauses: Vec<ComprehensionClause> = vec![];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one"));
    }

    #[test]
    fn test_list_comprehension_new_tuple_pattern() {
        let transpiler = Transpiler::new();
        let element = ident_expr("a");
        let clauses = vec![make_clause(
            "(a, b)",
            list_expr(vec![int_expr(1), int_expr(2)]),
            None,
        )];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
    }

    // ========================================================================
    // transpile_list_comprehension (old API) tests
    // ========================================================================

    #[test]
    fn test_list_comprehension_simple() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("x");
        let iter = range_expr(1, 10);

        let result = transpiler.transpile_list_comprehension(&expr, "x", &iter, None);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("map"));
        assert!(code.contains("collect"));
    }

    #[test]
    fn test_list_comprehension_with_filter() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("x");
        let iter = range_expr(1, 10);
        let filter = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Greater,
            int_expr(5),
        );

        let result = transpiler.transpile_list_comprehension(&expr, "x", &iter, Some(&filter));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    #[test]
    fn test_list_comprehension_simple_var() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("item");
        let iter = list_expr(vec![int_expr(1), int_expr(2)]);

        let result = transpiler.transpile_list_comprehension(&expr, "item", &iter, None);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("map"));
    }

    // ========================================================================
    // transpile_set_comprehension_new tests
    // ========================================================================

    #[test]
    fn test_set_comprehension_new_single_clause() {
        let transpiler = Transpiler::new();
        let element = ident_expr("x");
        let clauses = vec![make_clause("x", range_expr(0, 10), None)];

        let result = transpiler.transpile_set_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashSet"));
    }

    #[test]
    fn test_set_comprehension_new_with_condition() {
        let transpiler = Transpiler::new();
        let element = ident_expr("x");
        let condition = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Less,
            int_expr(5),
        );
        let clauses = vec![make_clause("x", range_expr(0, 10), Some(condition))];

        let result = transpiler.transpile_set_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    #[test]
    fn test_set_comprehension_new_empty_clauses_error() {
        let transpiler = Transpiler::new();
        let element = ident_expr("x");
        let clauses: Vec<ComprehensionClause> = vec![];

        let result = transpiler.transpile_set_comprehension_new(&element, &clauses);
        assert!(result.is_err());
    }

    // ========================================================================
    // transpile_set_comprehension (old API) tests
    // ========================================================================

    #[test]
    fn test_set_comprehension_simple() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("x");
        let iter = range_expr(1, 10);

        let result = transpiler.transpile_set_comprehension(&expr, "x", &iter, None);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashSet"));
    }

    #[test]
    fn test_set_comprehension_with_filter() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("x");
        let iter = range_expr(1, 10);
        let filter = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::NotEqual,
            int_expr(5),
        );

        let result = transpiler.transpile_set_comprehension(&expr, "x", &iter, Some(&filter));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    // ========================================================================
    // transpile_dict_comprehension_new tests
    // ========================================================================

    #[test]
    fn test_dict_comprehension_new_single_clause() {
        let transpiler = Transpiler::new();
        let key = ident_expr("k");
        let value = ident_expr("v");
        let clauses = vec![make_clause("(k, v)", list_expr(vec![]), None)];

        let result = transpiler.transpile_dict_comprehension_new(&key, &value, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashMap"));
    }

    #[test]
    fn test_dict_comprehension_new_with_condition() {
        let transpiler = Transpiler::new();
        let key = ident_expr("k");
        let value = ident_expr("v");
        let condition = binary_expr(
            ident_expr("v"),
            crate::frontend::ast::BinaryOp::Greater,
            int_expr(0),
        );
        let clauses = vec![make_clause("(k, v)", list_expr(vec![]), Some(condition))];

        let result = transpiler.transpile_dict_comprehension_new(&key, &value, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    #[test]
    fn test_dict_comprehension_new_empty_clauses_error() {
        let transpiler = Transpiler::new();
        let key = ident_expr("k");
        let value = ident_expr("v");
        let clauses: Vec<ComprehensionClause> = vec![];

        let result = transpiler.transpile_dict_comprehension_new(&key, &value, &clauses);
        assert!(result.is_err());
    }

    // ========================================================================
    // transpile_dict_comprehension (old API) tests
    // ========================================================================

    #[test]
    fn test_dict_comprehension_simple() {
        let transpiler = Transpiler::new();
        let key = ident_expr("x");
        let value = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Multiply,
            int_expr(2),
        );
        let iter = range_expr(1, 5);

        let result = transpiler.transpile_dict_comprehension(&key, &value, "x", &iter, None);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashMap"));
    }

    #[test]
    fn test_dict_comprehension_with_filter() {
        let transpiler = Transpiler::new();
        let key = ident_expr("x");
        let value = ident_expr("x");
        let iter = range_expr(1, 10);
        let filter = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Modulo,
            int_expr(2),
        );

        let result = transpiler.transpile_dict_comprehension(&key, &value, "x", &iter, Some(&filter));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    #[test]
    fn test_dict_comprehension_tuple_pattern() {
        let transpiler = Transpiler::new();
        let key = ident_expr("k");
        let value = ident_expr("v");
        let iter = list_expr(vec![]);

        let result = transpiler.transpile_dict_comprehension(&key, &value, "(k, v)", &iter, None);
        assert!(result.is_ok());
    }

    // ========================================================================
    // build_first_clause tests
    // ========================================================================

    #[test]
    fn test_build_first_clause_no_condition() {
        let transpiler = Transpiler::new();
        let iter_tokens = quote! { (0..10) };
        let var_pattern = quote! { x };

        let result = Transpiler::build_first_clause(&iter_tokens, &var_pattern, None, &transpiler);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("into_iter"));
    }

    #[test]
    fn test_build_first_clause_with_condition() {
        let transpiler = Transpiler::new();
        let iter_tokens = quote! { (0..10) };
        let var_pattern = quote! { x };
        let condition = binary_expr(
            ident_expr("x"),
            crate::frontend::ast::BinaryOp::Greater,
            int_expr(5),
        );

        let result =
            Transpiler::build_first_clause(&iter_tokens, &var_pattern, Some(&condition), &transpiler);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    // ========================================================================
    // build_nested_clause tests
    // ========================================================================

    #[test]
    fn test_build_nested_clause_no_condition() {
        let transpiler = Transpiler::new();
        let prev_chain = quote! { (0..3).into_iter() };
        let outer_pattern = quote! { x };
        let iter_tokens = quote! { (0..3) };
        let var_pattern = quote! { y };

        let result = Transpiler::build_nested_clause(
            &prev_chain,
            &outer_pattern,
            &iter_tokens,
            &var_pattern,
            None,
            &transpiler,
        );
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("flat_map"));
    }

    #[test]
    fn test_build_nested_clause_with_condition() {
        let transpiler = Transpiler::new();
        let prev_chain = quote! { (0..3).into_iter() };
        let outer_pattern = quote! { x };
        let iter_tokens = quote! { (0..3) };
        let var_pattern = quote! { y };
        let condition = binary_expr(
            ident_expr("y"),
            crate::frontend::ast::BinaryOp::Less,
            ident_expr("x"),
        );

        let result = Transpiler::build_nested_clause(
            &prev_chain,
            &outer_pattern,
            &iter_tokens,
            &var_pattern,
            Some(&condition),
            &transpiler,
        );
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("filter"));
    }

    // ========================================================================
    // Pattern comprehension edge cases
    // ========================================================================

    #[test]
    fn test_pattern_comprehension_no_parens() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("x");
        let iter = list_expr(vec![int_expr(1), int_expr(2)]);
        // Variable without parentheses - should use as-is
        let result = transpiler.transpile_list_comprehension(&expr, "simple_var", &iter, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_comprehension_nested_clauses() {
        let transpiler = Transpiler::new();
        let element = ident_expr("y");
        let clauses = vec![
            make_clause("x", range_expr(0, 3), None),
            make_clause("y", range_expr(0, 3), None),
        ];

        let result = transpiler.transpile_set_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("flat_map"));
    }

    #[test]
    fn test_dict_comprehension_nested_clauses() {
        let transpiler = Transpiler::new();
        let key = ident_expr("x");
        let value = ident_expr("y");
        let clauses = vec![
            make_clause("x", range_expr(0, 3), None),
            make_clause("y", range_expr(0, 3), None),
        ];

        let result = transpiler.transpile_dict_comprehension_new(&key, &value, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("flat_map"));
    }

    // ========================================================================
    // Edge cases and error handling
    // ========================================================================

    #[test]
    fn test_comprehension_with_complex_element_expression() {
        let transpiler = Transpiler::new();
        // x * x + 1
        let element = binary_expr(
            binary_expr(
                ident_expr("x"),
                crate::frontend::ast::BinaryOp::Multiply,
                ident_expr("x"),
            ),
            crate::frontend::ast::BinaryOp::Add,
            int_expr(1),
        );
        let clauses = vec![make_clause("x", range_expr(1, 10), None)];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comprehension_triple_nested() {
        let transpiler = Transpiler::new();
        let element = ident_expr("z");
        let clauses = vec![
            make_clause("x", range_expr(0, 2), None),
            make_clause("y", range_expr(0, 2), None),
            make_clause("z", range_expr(0, 2), None),
        ];

        let result = transpiler.transpile_list_comprehension_new(&element, &clauses);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        // Should have nested flat_maps
        assert!(code.matches("flat_map").count() >= 2);
    }
}
