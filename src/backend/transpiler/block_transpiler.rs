//! Block and Pipeline Transpilation
//!
//! This module handles transpilation of block expressions and pipeline stages
//! with proper semicolon placement and nested block flattening.
//!
//! **EXTREME TDD Round 71**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, PipelineStage};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpile block expressions with smart brace handling
    /// Issue #141: Nested single-expression blocks are flattened
    /// Complexity: 7 (within Toyota Way limits)
    pub(crate) fn transpile_block_impl(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.is_empty() {
            return Ok(quote! { {} });
        }

        // Flatten nested single-expression blocks
        if exprs.len() == 1 {
            if let ExprKind::Block(inner_exprs) = &exprs[0].kind {
                return self.transpile_block_impl(inner_exprs);
            }
        }

        let mut statements = Vec::new();
        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            let is_last = i == exprs.len() - 1;
            let is_let = Self::is_let_expr(expr);

            if is_last {
                // Last expression - no semicolon (return value)
                statements.push(expr_tokens);
            } else if is_let {
                // Let expressions include their own semicolons
                statements.push(expr_tokens);
            } else {
                // Add semicolon to non-last, non-let expressions
                statements.push(quote! { #expr_tokens; });
            }
        }

        Ok(quote! { { #(#statements)* } })
    }

    /// Check if expression is a let binding
    fn is_let_expr(expr: &Expr) -> bool {
        matches!(
            &expr.kind,
            ExprKind::Let { .. } | ExprKind::LetPattern { .. }
        )
    }

    /// Transpile pipeline expressions (|> operator chains)
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn transpile_pipeline_impl(
        &self,
        expr: &Expr,
        stages: &[PipelineStage],
    ) -> Result<TokenStream> {
        let mut result = self.transpile_expr(expr)?;

        for stage in stages {
            result = self.apply_pipeline_stage_impl(result, stage)?;
        }

        Ok(result)
    }

    /// Apply a single pipeline stage to the accumulated result
    fn apply_pipeline_stage_impl(
        &self,
        prev: TokenStream,
        stage: &PipelineStage,
    ) -> Result<TokenStream> {
        let stage_expr = &stage.op;
        match &stage_expr.kind {
            ExprKind::Call { func, args } => {
                let func_tokens = self.transpile_expr(func)?;
                let arg_tokens: Result<Vec<_>> =
                    args.iter().map(|a| self.transpile_expr(a)).collect();
                let arg_tokens = arg_tokens?;
                Ok(quote! { #func_tokens(#prev #(, #arg_tokens)*) })
            }
            ExprKind::MethodCall { method, args, .. } => {
                let method_ident = format_ident!("{}", method);
                let arg_tokens: Result<Vec<_>> =
                    args.iter().map(|a| self.transpile_expr(a)).collect();
                let arg_tokens = arg_tokens?;
                Ok(quote! { #prev.#method_ident(#(#arg_tokens),*) })
            }
            _ => {
                let stage_tokens = self.transpile_expr(stage_expr)?;
                Ok(quote! { #stage_tokens(#prev) })
            }
        }
    }

    /// Transpile block without braces (for function bodies)
    pub(crate) fn transpile_block_contents(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.is_empty() {
            return Ok(quote! {});
        }

        let mut statements = Vec::new();
        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            let is_last = i == exprs.len() - 1;
            let is_let = Self::is_let_expr(expr);

            if is_last {
                statements.push(expr_tokens);
            } else if is_let {
                statements.push(expr_tokens);
            } else {
                statements.push(quote! { #expr_tokens; });
            }
        }

        Ok(quote! { #(#statements)* })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Pattern, Span};

    fn make_transpiler() -> Transpiler {
        Transpiler::new()
    }

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

    fn make_stage(op: Expr) -> PipelineStage {
        PipelineStage {
            op: Box::new(op),
            span: Span::default(),
        }
    }

    // ========================================================================
    // transpile_block_impl tests
    // ========================================================================

    #[test]
    fn test_block_empty() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_block_impl(&[]).unwrap();
        assert!(result.to_string().contains("{"));
        assert!(result.to_string().contains("}"));
    }

    #[test]
    fn test_block_single_expression() {
        let transpiler = make_transpiler();
        let exprs = vec![int_expr(42)];
        let result = transpiler.transpile_block_impl(&exprs).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_block_multiple_expressions() {
        let transpiler = make_transpiler();
        let exprs = vec![ident_expr("a"), ident_expr("b"), int_expr(42)];
        let result = transpiler.transpile_block_impl(&exprs).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains(";"));
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_block_nested_flattening() {
        let transpiler = make_transpiler();
        let inner_block = make_expr(ExprKind::Block(vec![int_expr(42)]));
        let exprs = vec![inner_block];
        let result = transpiler.transpile_block_impl(&exprs).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_block_with_let() {
        let transpiler = make_transpiler();
        let let_expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_expr(1)),
            body: Box::new(ident_expr("x")),
            is_mutable: false,
            else_block: None,
        });
        let exprs = vec![let_expr, int_expr(42)];
        let result = transpiler.transpile_block_impl(&exprs).unwrap();
        assert!(result.to_string().contains("42"));
    }

    // ========================================================================
    // is_let_expr tests
    // ========================================================================

    #[test]
    fn test_is_let_expr_let() {
        let expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_expr(1)),
            body: Box::new(int_expr(0)),
            is_mutable: false,
            else_block: None,
        });
        assert!(Transpiler::is_let_expr(&expr));
    }

    #[test]
    fn test_is_let_expr_let_pattern() {
        let expr = make_expr(ExprKind::LetPattern {
            pattern: Pattern::Wildcard,
            type_annotation: None,
            value: Box::new(int_expr(1)),
            body: Box::new(int_expr(0)),
            is_mutable: false,
            else_block: None,
        });
        assert!(Transpiler::is_let_expr(&expr));
    }

    #[test]
    fn test_is_let_expr_not_let() {
        let expr = int_expr(42);
        assert!(!Transpiler::is_let_expr(&expr));
    }

    // ========================================================================
    // transpile_pipeline_impl tests
    // ========================================================================

    #[test]
    fn test_pipeline_method_call() {
        let transpiler = make_transpiler();
        let expr = int_expr(5);
        let method_call = make_expr(ExprKind::MethodCall {
            receiver: Box::new(int_expr(0)), // Placeholder
            method: "to_string".to_string(),
            args: vec![],
        });
        let stages = vec![make_stage(method_call)];
        let result = transpiler.transpile_pipeline_impl(&expr, &stages).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("5"));
        assert!(result_str.contains("to_string"));
    }

    #[test]
    fn test_pipeline_function_call() {
        let transpiler = make_transpiler();
        let expr = int_expr(42);
        let func_call = make_expr(ExprKind::Call {
            func: Box::new(ident_expr("process")),
            args: vec![],
        });
        let stages = vec![make_stage(func_call)];
        let result = transpiler.transpile_pipeline_impl(&expr, &stages).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("process"));
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_pipeline_chained() {
        let transpiler = make_transpiler();
        let expr = ident_expr("data");
        let filter = make_expr(ExprKind::MethodCall {
            receiver: Box::new(int_expr(0)),
            method: "filter".to_string(),
            args: vec![],
        });
        let map = make_expr(ExprKind::MethodCall {
            receiver: Box::new(int_expr(0)),
            method: "map".to_string(),
            args: vec![],
        });
        let stages = vec![make_stage(filter), make_stage(map)];
        let result = transpiler.transpile_pipeline_impl(&expr, &stages).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("data"));
        assert!(result_str.contains("filter"));
        assert!(result_str.contains("map"));
    }

    // ========================================================================
    // transpile_block_contents tests
    // ========================================================================

    #[test]
    fn test_block_contents_empty() {
        let transpiler = make_transpiler();
        let result = transpiler.transpile_block_contents(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_block_contents_single() {
        let transpiler = make_transpiler();
        let exprs = vec![int_expr(42)];
        let result = transpiler.transpile_block_contents(&exprs).unwrap();
        assert!(result.to_string().contains("42"));
    }

    #[test]
    fn test_block_contents_multiple() {
        let transpiler = make_transpiler();
        let exprs = vec![ident_expr("a"), int_expr(42)];
        let result = transpiler.transpile_block_contents(&exprs).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("a"));
        assert!(result_str.contains("42"));
    }
}
