//! Control Flow Transpilation
//!
//! This module handles transpilation of control flow statements:
//! - if/else
//! - for loops
//! - while loops
//! - if-let and while-let
//! - loop (infinite)
//! - try-catch-finally
//!
//! **EXTREME TDD Round 53**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{CatchClause, Expr, ExprKind, Pattern};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles if expressions with optional else branch
    /// Complexity: 7 (within Toyota Way limits)
    pub fn transpile_if(
        &self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;

        // Check if then_branch is already a Block to avoid double-wrapping
        let then_tokens = if let ExprKind::Block(stmts) = &then_branch.kind {
            // Directly transpile the block contents without extra wrapping
            self.transpile_block(stmts)?
        } else {
            // Single expression, wrap it
            let expr_tokens = self.transpile_expr(then_branch)?;
            quote! { { #expr_tokens } }
        };

        if let Some(else_expr) = else_branch {
            // Same treatment for else branch
            let else_tokens = if let ExprKind::Block(stmts) = &else_expr.kind {
                self.transpile_block(stmts)?
            } else {
                let expr_tokens = self.transpile_expr(else_expr)?;
                quote! { { #expr_tokens } }
            };

            Ok(quote! {
                if #cond_tokens #then_tokens else #else_tokens
            })
        } else {
            Ok(quote! {
                if #cond_tokens #then_tokens
            })
        }
    }

    /// Transpiles for loops with pattern support
    /// Complexity: 5 (within Toyota Way limits)
    pub fn transpile_for(
        &self,
        var: &str,
        pattern: Option<&Pattern>,
        iter: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let iter_tokens = self.transpile_expr(iter)?;

        // DEFECT-018 FIX: Set loop context flag to enable auto-cloning in function calls
        let was_in_loop = self.in_loop_context.get();
        self.in_loop_context.set(true);
        let body_tokens = self.transpile_expr(body)?;
        self.in_loop_context.set(was_in_loop);

        // If we have a pattern, use it for destructuring
        if let Some(pat) = pattern {
            let pattern_tokens = self.transpile_pattern(pat)?;
            Ok(quote! {
                for #pattern_tokens in #iter_tokens {
                    #body_tokens
                }
            })
        } else {
            // Fall back to simple variable
            let var_ident = format_ident!("{}", var);
            Ok(quote! {
                for #var_ident in #iter_tokens {
                    #body_tokens
                }
            })
        }
    }

    /// Transpiles while loops
    /// Complexity: 3 (within Toyota Way limits)
    pub fn transpile_while(&self, condition: &Expr, body: &Expr) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;

        // DEFECT-018 FIX: Set loop context flag to enable auto-cloning in function calls
        let was_in_loop = self.in_loop_context.get();
        self.in_loop_context.set(true);
        let body_tokens = self.transpile_expr(body)?;
        self.in_loop_context.set(was_in_loop);

        Ok(quote! {
            while #cond_tokens {
                #body_tokens
            }
        })
    }

    /// Transpile if-let expression
    /// Complexity: 5 (within Toyota Way limits)
    pub fn transpile_if_let(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let then_tokens = self.transpile_expr(then_branch)?;
        if let Some(else_expr) = else_branch {
            let else_tokens = self.transpile_expr(else_expr)?;
            Ok(quote! {
                if let #pattern_tokens = #expr_tokens {
                    #then_tokens
                } else {
                    #else_tokens
                }
            })
        } else {
            Ok(quote! {
                if let #pattern_tokens = #expr_tokens {
                    #then_tokens
                }
            })
        }
    }

    /// Transpile while-let expression
    /// Complexity: 4 (within Toyota Way limits)
    pub fn transpile_while_let(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let body_tokens = self.transpile_expr(body)?;
        Ok(quote! {
            while let #pattern_tokens = #expr_tokens {
                #body_tokens
            }
        })
    }

    /// Transpile infinite loop
    /// Complexity: 2 (within Toyota Way limits)
    pub fn transpile_loop(&self, body: &Expr) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;
        Ok(quote! {
            loop {
                #body_tokens
            }
        })
    }

    /// Transpiles try-catch-finally blocks
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_try_catch(
        &self,
        try_block: &Expr,
        catch_clauses: &[CatchClause],
        finally_block: Option<&Expr>,
    ) -> Result<TokenStream> {
        // DEFECT-TRY-CATCH FIX: Use catch_unwind to catch panics from throw
        // throw -> panic!() requires catch_unwind, not Result pattern
        let try_body = self.transpile_expr(try_block)?;
        if catch_clauses.is_empty() {
            bail!("Try block must have at least one catch clause");
        }
        // Generate the catch handling
        let catch_pattern = if let Pattern::Identifier(name) = &catch_clauses[0].pattern {
            let ident = format_ident!("{}", name);
            quote! { #ident }
        } else {
            quote! { _e }
        };
        let catch_body = self.transpile_expr(&catch_clauses[0].body)?;

        // If there's a finally block, we need to ensure it runs
        let result = if let Some(finally) = finally_block {
            let finally_tokens = self.transpile_expr(finally)?;
            quote! {
                {
                    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #try_body
                    }));
                    let _final_result = match _result {
                        Ok(val) => val,
                        Err(panic_err) => {
                            // Convert panic payload to string for catch variable
                            let #catch_pattern = if let Some(s) = panic_err.downcast_ref::<&str>() {
                                s.to_string()
                            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                                s.clone()
                            } else {
                                "Unknown panic".to_string()
                            };
                            #catch_body
                        }
                    };
                    #finally_tokens;
                    _final_result
                }
            }
        } else {
            // Simple try-catch without finally - use catch_unwind to catch panics
            quote! {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    #try_body
                })) {
                    Ok(val) => val,
                    Err(panic_err) => {
                        // Convert panic payload to string for catch variable
                        let #catch_pattern = if let Some(s) = panic_err.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unknown panic".to_string()
                        };
                        #catch_body
                    }
                }
            }
        };
        Ok(result)
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

    fn bool_expr(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
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

    #[test]
    fn test_transpile_if_simple() {
        let transpiler = Transpiler::new();
        let cond = bool_expr(true);
        let then_branch = int_expr(1);
        let result = transpiler.transpile_if(&cond, &then_branch, None);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("if"));
        assert!(tokens.contains("true"));
    }

    #[test]
    fn test_transpile_if_with_else() {
        let transpiler = Transpiler::new();
        let cond = bool_expr(true);
        let then_branch = int_expr(1);
        let else_branch = int_expr(2);
        let result = transpiler.transpile_if(&cond, &then_branch, Some(&else_branch));
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("if"));
        assert!(tokens.contains("else"));
    }

    #[test]
    fn test_transpile_if_with_block() {
        let transpiler = Transpiler::new();
        let cond = bool_expr(true);
        let block_content = vec![int_expr(42)];
        let then_branch = make_expr(ExprKind::Block(block_content));
        let result = transpiler.transpile_if(&cond, &then_branch, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_for_simple() {
        let transpiler = Transpiler::new();
        let iter = list_expr(vec![]);
        let body = int_expr(1);
        let result = transpiler.transpile_for("i", None, &iter, &body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("for"));
        assert!(tokens.contains("in"));
    }

    #[test]
    fn test_transpile_for_with_pattern() {
        let transpiler = Transpiler::new();
        let iter = list_expr(vec![]);
        let body = int_expr(1);
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let result = transpiler.transpile_for("_", Some(&pattern), &iter, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_while_simple() {
        let transpiler = Transpiler::new();
        let cond = bool_expr(true);
        let body = int_expr(1);
        let result = transpiler.transpile_while(&cond, &body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("while"));
    }

    #[test]
    fn test_transpile_if_let_simple() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let expr = ident_expr("opt");
        let then_branch = int_expr(1);
        let result = transpiler.transpile_if_let(&pattern, &expr, &then_branch, None);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("if let"));
    }

    #[test]
    fn test_transpile_if_let_with_else() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let expr = ident_expr("opt");
        let then_branch = int_expr(1);
        let else_branch = int_expr(0);
        let result = transpiler.transpile_if_let(&pattern, &expr, &then_branch, Some(&else_branch));
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("if let"));
        assert!(tokens.contains("else"));
    }

    #[test]
    fn test_transpile_while_let() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let expr = ident_expr("iter");
        let body = int_expr(1);
        let result = transpiler.transpile_while_let(&pattern, &expr, &body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("while let"));
    }

    #[test]
    fn test_transpile_loop() {
        let transpiler = Transpiler::new();
        let body = int_expr(1);
        let result = transpiler.transpile_loop(&body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("loop"));
    }

    #[test]
    fn test_transpile_try_catch_simple() {
        let transpiler = Transpiler::new();
        let try_block = int_expr(1);
        let catch_body = int_expr(0);
        let catch_clauses = vec![CatchClause {
            pattern: Pattern::Identifier("e".to_string()),
            body: Box::new(catch_body),
        }];
        let result = transpiler.transpile_try_catch(&try_block, &catch_clauses, None);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("catch_unwind"));
    }

    #[test]
    fn test_transpile_try_catch_with_finally() {
        let transpiler = Transpiler::new();
        let try_block = int_expr(1);
        let catch_body = int_expr(0);
        let finally_block = int_expr(2);
        let catch_clauses = vec![CatchClause {
            pattern: Pattern::Identifier("e".to_string()),
            body: Box::new(catch_body),
        }];
        let result =
            transpiler.transpile_try_catch(&try_block, &catch_clauses, Some(&finally_block));
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("_final_result"));
    }

    #[test]
    fn test_transpile_try_catch_empty_clauses() {
        let transpiler = Transpiler::new();
        let try_block = int_expr(1);
        let result = transpiler.transpile_try_catch(&try_block, &[], None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least one catch clause"));
    }

    #[test]
    fn test_transpile_try_catch_wildcard_pattern() {
        let transpiler = Transpiler::new();
        let try_block = int_expr(1);
        let catch_body = int_expr(0);
        let catch_clauses = vec![CatchClause {
            pattern: Pattern::Wildcard,
            body: Box::new(catch_body),
        }];
        let result = transpiler.transpile_try_catch(&try_block, &catch_clauses, None);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("_e")); // Uses _e for wildcard pattern
    }

    #[test]
    fn test_loop_context_preserved_for() {
        let transpiler = Transpiler::new();
        assert!(!transpiler.in_loop_context.get());
        let iter = list_expr(vec![]);
        let body = int_expr(1);
        let _ = transpiler.transpile_for("i", None, &iter, &body);
        // Context should be restored after transpilation
        assert!(!transpiler.in_loop_context.get());
    }

    #[test]
    fn test_loop_context_preserved_while() {
        let transpiler = Transpiler::new();
        assert!(!transpiler.in_loop_context.get());
        let cond = bool_expr(false);
        let body = int_expr(1);
        let _ = transpiler.transpile_while(&cond, &body);
        // Context should be restored after transpilation
        assert!(!transpiler.in_loop_context.get());
    }
}
