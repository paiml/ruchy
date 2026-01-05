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
    fn test_parse_tuple_or_simple_pattern_simple() {
        let result = Transpiler::parse_tuple_or_simple_pattern("item");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tuple_or_simple_pattern_tuple() {
        let result = Transpiler::parse_tuple_or_simple_pattern("(k, v)");
        assert!(result.is_ok());
    }
}
