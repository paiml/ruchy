//! Block and comprehension transpilation
//!
//! Handles blocks, list comprehensions, and complex expressions

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Pattern};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

impl Transpiler {
    /// Transpile block expression (complexity: 8)
    pub fn transpile_block(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.is_empty() {
            return Ok(quote! { {} });
        }
        
        if exprs.len() == 1 {
            return self.transpile_single_expression_block(&exprs[0]);
        }
        
        self.transpile_multi_expression_block(exprs)
    }

    /// Helper: Transpile single expression block (complexity: 4)
    fn transpile_single_expression_block(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { { #expr_tokens } })
    }

    /// Helper: Transpile multi-expression block (complexity: 7)
    fn transpile_multi_expression_block(&self, exprs: &[Expr]) -> Result<TokenStream> {
        let mut statements = Vec::new();
        let last_idx = exprs.len() - 1;
        
        for (i, expr) in exprs.iter().enumerate() {
            let tokens = self.transpile_expr(expr)?;
            
            if i == last_idx {
                // Last expression is the return value
                statements.push(quote! { #tokens });
            } else {
                // Intermediate expressions need semicolons
                statements.push(self.add_semicolon_if_needed(tokens, expr));
            }
        }
        
        Ok(quote! { { #(#statements)* } })
    }

    /// Helper: Add semicolon if expression needs it (complexity: 5)
    fn add_semicolon_if_needed(&self, tokens: TokenStream, expr: &Expr) -> TokenStream {
        if self.needs_semicolon(expr) {
            quote! { #tokens; }
        } else {
            tokens
        }
    }

    /// Helper: Check if expression needs semicolon (complexity: 4)
    fn needs_semicolon(&self, expr: &Expr) -> bool {
        !matches!(&expr.kind, 
            ExprKind::If { .. } |
            ExprKind::While { .. } |
            ExprKind::For { .. } |
            ExprKind::Loop { .. } |
            ExprKind::Match { .. }
        )
    }

    /// Transpile list comprehension (complexity: 10)
    pub fn transpile_list_comprehension(
        &self,
        expr: &Expr,
        var: &str,
        iter: &Expr,
        condition: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var);
        let expr_tokens = self.transpile_expr(expr)?;
        let iter_tokens = self.transpile_expr(iter)?;
        
        if let Some(cond) = condition {
            let cond_tokens = self.transpile_expr(cond)?;
            Ok(quote! {
                {
                    #iter_tokens
                        .into_iter()
                        .filter(|#var_ident| #cond_tokens)
                        .map(|#var_ident| #expr_tokens)
                        .collect::<Vec<_>>()
                }
            })
        } else {
            Ok(quote! {
                {
                    #iter_tokens
                        .into_iter()
                        .map(|#var_ident| #expr_tokens)
                        .collect::<Vec<_>>()
                }
            })
        }
    }

    /// Transpile set comprehension (complexity: 9)
    pub fn transpile_set_comprehension(
        &self,
        expr: &Expr,
        var: &str,
        iter: &Expr,
        condition: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var);
        let expr_tokens = self.transpile_expr(expr)?;
        let iter_tokens = self.transpile_expr(iter)?;
        
        if let Some(cond) = condition {
            let cond_tokens = self.transpile_expr(cond)?;
            Ok(quote! {
                {
                    use std::collections::HashSet;
                    #iter_tokens
                        .into_iter()
                        .filter(|#var_ident| #cond_tokens)
                        .map(|#var_ident| #expr_tokens)
                        .collect::<HashSet<_>>()
                }
            })
        } else {
            Ok(quote! {
                {
                    use std::collections::HashSet;
                    #iter_tokens
                        .into_iter()
                        .map(|#var_ident| #expr_tokens)
                        .collect::<HashSet<_>>()
                }
            })
        }
    }

    /// Transpile dict comprehension (complexity: 10)
    pub fn transpile_dict_comprehension(
        &self,
        key_expr: &Expr,
        value_expr: &Expr,
        var: &str,
        iter: &Expr,
        condition: Option<&Expr>,
    ) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var);
        let key_tokens = self.transpile_expr(key_expr)?;
        let value_tokens = self.transpile_expr(value_expr)?;
        let iter_tokens = self.transpile_expr(iter)?;
        
        if let Some(cond) = condition {
            let cond_tokens = self.transpile_expr(cond)?;
            Ok(quote! {
                {
                    use std::collections::HashMap;
                    #iter_tokens
                        .into_iter()
                        .filter(|#var_ident| #cond_tokens)
                        .map(|#var_ident| (#key_tokens, #value_tokens))
                        .collect::<HashMap<_, _>>()
                }
            })
        } else {
            Ok(quote! {
                {
                    use std::collections::HashMap;
                    #iter_tokens
                        .into_iter()
                        .map(|#var_ident| (#key_tokens, #value_tokens))
                        .collect::<HashMap<_, _>>()
                }
            })
        }
    }

    /// Transpile return statement (complexity: 3)
    pub fn transpile_return(&self, value: Option<&Expr>) -> Result<TokenStream> {
        if let Some(expr) = value {
            let tokens = self.transpile_expr(expr)?;
            Ok(quote! { return #tokens })
        } else {
            Ok(quote! { return })
        }
    }

    /// Transpile break statement (complexity: 3)
    pub fn transpile_break(&self, value: Option<&Expr>) -> Result<TokenStream> {
        if let Some(expr) = value {
            let tokens = self.transpile_expr(expr)?;
            Ok(quote! { break #tokens })
        } else {
            Ok(quote! { break })
        }
    }

    /// Transpile continue statement (complexity: 2)
    pub fn transpile_continue(&self) -> Result<TokenStream> {
        Ok(quote! { continue })
    }
}