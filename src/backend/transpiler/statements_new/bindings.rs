//! Variable bindings and pattern matching transpilation
//!
//! Handles let statements, pattern destructuring, and mutability detection

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal, Pattern};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

impl Transpiler {
    /// Transpile let binding (complexity: 9)
    pub fn transpile_let(
        &self,
        name: &str,
        value: &Expr,
        body: &Expr,
        is_mutable: bool,
    ) -> Result<TokenStream> {
        let safe_name = self.make_safe_identifier(name);
        let name_ident = format_ident!("{}", safe_name);
        
        let effective_mutability = self.determine_mutability(name, body, is_mutable);
        let value_tokens = self.prepare_value_tokens(value)?;
        
        self.generate_let_binding(&name_ident, &value_tokens, body, effective_mutability)
    }

    /// Transpile let pattern binding (complexity: 6)
    pub fn transpile_let_pattern(
        &self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let value_tokens = self.transpile_expr(value)?;
        
        if self.is_unit_body(body) {
            Ok(quote! { let #pattern_tokens = #value_tokens })
        } else {
            let body_tokens = self.transpile_expr(body)?;
            Ok(quote! {
                {
                    let #pattern_tokens = #value_tokens;
                    #body_tokens
                }
            })
        }
    }

    /// Helper: Make identifier safe for Rust (complexity: 3)
    fn make_safe_identifier(&self, name: &str) -> String {
        if Self::is_rust_reserved_keyword(name) {
            format!("r#{}", name)
        } else {
            name.to_string()
        }
    }

    /// Helper: Determine effective mutability (complexity: 4)
    fn determine_mutability(&self, name: &str, body: &Expr, is_mutable: bool) -> bool {
        is_mutable || 
        self.mutable_vars.contains(name) || 
        Self::is_variable_mutated(name, body)
    }

    /// Helper: Prepare value tokens with string conversion (complexity: 5)
    fn prepare_value_tokens(&self, value: &Expr) -> Result<TokenStream> {
        match &value.kind {
            ExprKind::Literal(Literal::String(s)) => {
                // Convert string literals to owned String
                Ok(quote! { #s.to_string() })
            }
            _ => self.transpile_expr(value)
        }
    }

    /// Helper: Generate let binding with proper scoping (complexity: 8)
    fn generate_let_binding(
        &self,
        name_ident: &proc_macro2::Ident,
        value_tokens: &TokenStream,
        body: &Expr,
        is_mutable: bool,
    ) -> Result<TokenStream> {
        if self.is_unit_body(body) {
            // Top-level let without scoping
            if is_mutable {
                Ok(quote! { let mut #name_ident = #value_tokens; })
            } else {
                Ok(quote! { let #name_ident = #value_tokens; })
            }
        } else {
            // Let-in expression with scoping
            let body_tokens = self.transpile_expr(body)?;
            if is_mutable {
                Ok(quote! {
                    {
                        let mut #name_ident = #value_tokens;
                        #body_tokens
                    }
                })
            } else {
                Ok(quote! {
                    {
                        let #name_ident = #value_tokens;
                        #body_tokens
                    }
                })
            }
        }
    }

    /// Helper: Check if body is unit literal (complexity: 2)
    fn is_unit_body(&self, body: &Expr) -> bool {
        matches!(body.kind, ExprKind::Literal(Literal::Unit))
    }

    /// Check if a variable is mutated in an expression tree (complexity: 10)
    pub fn is_variable_mutated(name: &str, expr: &Expr) -> bool {
        match &expr.kind {
            // Direct assignment
            ExprKind::Assign { target, .. } => {
                Self::is_identifier_match(target, name)
            }
            // Compound assignment
            ExprKind::CompoundAssign { target, .. } => {
                Self::is_identifier_match(target, name)
            }
            // Increment/decrement
            ExprKind::PreIncrement { target } | 
            ExprKind::PostIncrement { target } |
            ExprKind::PreDecrement { target } |
            ExprKind::PostDecrement { target } => {
                Self::is_identifier_match(target, name)
            }
            // Recursive cases
            ExprKind::Block(exprs) => {
                exprs.iter().any(|e| Self::is_variable_mutated(name, e))
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                Self::is_variable_mutated(name, condition) ||
                Self::is_variable_mutated(name, then_branch) ||
                else_branch.as_ref().map_or(false, |e| Self::is_variable_mutated(name, e))
            }
            _ => false
        }
    }

    /// Helper: Check if expression is identifier matching name (complexity: 3)
    fn is_identifier_match(expr: &Expr, name: &str) -> bool {
        matches!(&expr.kind, ExprKind::Identifier(n) if n == name)
    }
}