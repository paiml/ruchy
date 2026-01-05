//! Math Built-in Function Transpilation
//!
//! This module handles transpilation of math built-in functions:
//! - sqrt, pow, abs
//! - min, max
//! - floor, ceil, round
//!
//! **EXTREME TDD Round 56**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal, UnaryOp};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Handle math functions (sqrt, pow, abs, min, max, floor, ceil, round)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("sqrt(4.0)");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("sqrt"));
    /// ```
    /// Complexity: 6 (within Toyota Way limits)
    pub fn try_transpile_math_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match (base_name, args.len()) {
            ("sqrt", 1) => self.transpile_sqrt(&args[0]).map(Some),
            ("pow", 2) => self.transpile_pow(&args[0], &args[1]).map(Some),
            ("abs", 1) => self.transpile_abs(&args[0]).map(Some),
            ("min", 2) => self.transpile_min(&args[0], &args[1]).map(Some),
            ("max", 2) => self.transpile_max(&args[0], &args[1]).map(Some),
            ("floor", 1) => self.transpile_floor(&args[0]).map(Some),
            ("ceil", 1) => self.transpile_ceil(&args[0]).map(Some),
            ("round", 1) => self.transpile_round(&args[0]).map(Some),
            _ => Ok(None),
        }
    }

    /// Transpile sqrt function
    /// Complexity: 2 (within Toyota Way limits)
    fn transpile_sqrt(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).sqrt() })
    }

    /// Transpile pow function
    /// Complexity: 2 (within Toyota Way limits)
    fn transpile_pow(&self, base: &Expr, exp: &Expr) -> Result<TokenStream> {
        let base_tokens = self.transpile_expr(base)?;
        let exp_tokens = self.transpile_expr(exp)?;
        Ok(quote! { (#base_tokens as f64).powf(#exp_tokens as f64) })
    }

    /// Transpile abs function with type-aware handling
    /// Complexity: 5 (within Toyota Way limits)
    fn transpile_abs(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        // Check if arg is negative literal to handle type
        if let ExprKind::Unary {
            op: UnaryOp::Negate,
            operand,
        } = &arg.kind
        {
            if matches!(&operand.kind, ExprKind::Literal(Literal::Float(_))) {
                return Ok(quote! { (#arg_tokens).abs() });
            }
        }
        // For all other cases, use standard abs
        Ok(quote! { #arg_tokens.abs() })
    }

    /// Transpile min function with type-aware handling
    /// Complexity: 4 (within Toyota Way limits)
    fn transpile_min(&self, a: &Expr, b: &Expr) -> Result<TokenStream> {
        let a_tokens = self.transpile_expr(a)?;
        let b_tokens = self.transpile_expr(b)?;
        // Check if args are float literals to determine type
        let is_float = matches!(&a.kind, ExprKind::Literal(Literal::Float(_)))
            || matches!(&b.kind, ExprKind::Literal(Literal::Float(_)));
        if is_float {
            Ok(quote! { (#a_tokens as f64).min(#b_tokens as f64) })
        } else {
            Ok(quote! { std::cmp::min(#a_tokens, #b_tokens) })
        }
    }

    /// Transpile max function with type-aware handling
    /// Complexity: 4 (within Toyota Way limits)
    fn transpile_max(&self, a: &Expr, b: &Expr) -> Result<TokenStream> {
        let a_tokens = self.transpile_expr(a)?;
        let b_tokens = self.transpile_expr(b)?;
        // Check if args are float literals to determine type
        let is_float = matches!(&a.kind, ExprKind::Literal(Literal::Float(_)))
            || matches!(&b.kind, ExprKind::Literal(Literal::Float(_)));
        if is_float {
            Ok(quote! { (#a_tokens as f64).max(#b_tokens as f64) })
        } else {
            Ok(quote! { std::cmp::max(#a_tokens, #b_tokens) })
        }
    }

    /// Transpile floor function
    /// Complexity: 2 (within Toyota Way limits)
    fn transpile_floor(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).floor() })
    }

    /// Transpile ceil function
    /// Complexity: 2 (within Toyota Way limits)
    fn transpile_ceil(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).ceil() })
    }

    /// Transpile round function
    /// Complexity: 2 (within Toyota Way limits)
    fn transpile_round(&self, arg: &Expr) -> Result<TokenStream> {
        let arg_tokens = self.transpile_expr(arg)?;
        Ok(quote! { (#arg_tokens as f64).round() })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;

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

    fn neg_float_expr(f: f64) -> Expr {
        make_expr(ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(float_expr(f)),
        })
    }

    // ========================================================================
    // try_transpile_math_function tests
    // ========================================================================

    #[test]
    fn test_try_transpile_math_function_sqrt() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(4.0)];
        let result = transpiler.try_transpile_math_function("sqrt", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("sqrt"));
    }

    #[test]
    fn test_try_transpile_math_function_pow() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(2.0), float_expr(3.0)];
        let result = transpiler.try_transpile_math_function("pow", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("powf"));
    }

    #[test]
    fn test_try_transpile_math_function_abs() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(-5)];
        let result = transpiler.try_transpile_math_function("abs", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("abs"));
    }

    #[test]
    fn test_try_transpile_math_function_min_int() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(3), int_expr(5)];
        let result = transpiler.try_transpile_math_function("min", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("std :: cmp :: min"));
    }

    #[test]
    fn test_try_transpile_math_function_min_float() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.0), float_expr(5.0)];
        let result = transpiler.try_transpile_math_function("min", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("min"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_try_transpile_math_function_max_int() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(3), int_expr(5)];
        let result = transpiler.try_transpile_math_function("max", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("std :: cmp :: max"));
    }

    #[test]
    fn test_try_transpile_math_function_max_float() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.0), float_expr(5.0)];
        let result = transpiler.try_transpile_math_function("max", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("max"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_try_transpile_math_function_floor() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.7)];
        let result = transpiler.try_transpile_math_function("floor", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("floor"));
    }

    #[test]
    fn test_try_transpile_math_function_ceil() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.2)];
        let result = transpiler.try_transpile_math_function("ceil", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("ceil"));
    }

    #[test]
    fn test_try_transpile_math_function_round() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.5)];
        let result = transpiler.try_transpile_math_function("round", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("round"));
    }

    #[test]
    fn test_try_transpile_math_function_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.5)];
        let result = transpiler.try_transpile_math_function("unknown_func", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_math_function_wrong_arity() {
        let transpiler = Transpiler::new();
        // sqrt with 2 args should return None
        let args = vec![float_expr(4.0), float_expr(2.0)];
        let result = transpiler.try_transpile_math_function("sqrt", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // Individual function tests
    // ========================================================================

    #[test]
    fn test_transpile_sqrt() {
        let transpiler = Transpiler::new();
        let arg = float_expr(9.0);
        let result = transpiler.transpile_sqrt(&arg);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("sqrt"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_transpile_pow() {
        let transpiler = Transpiler::new();
        let base = float_expr(2.0);
        let exp = float_expr(3.0);
        let result = transpiler.transpile_pow(&base, &exp);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("powf"));
    }

    #[test]
    fn test_transpile_abs_integer() {
        let transpiler = Transpiler::new();
        let arg = int_expr(-5);
        let result = transpiler.transpile_abs(&arg);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("abs"));
    }

    #[test]
    fn test_transpile_abs_negative_float() {
        let transpiler = Transpiler::new();
        let arg = neg_float_expr(5.0);
        let result = transpiler.transpile_abs(&arg);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("abs"));
    }

    #[test]
    fn test_transpile_min_integers() {
        let transpiler = Transpiler::new();
        let a = int_expr(3);
        let b = int_expr(7);
        let result = transpiler.transpile_min(&a, &b);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("std :: cmp :: min"));
    }

    #[test]
    fn test_transpile_min_floats() {
        let transpiler = Transpiler::new();
        let a = float_expr(3.0);
        let b = float_expr(7.0);
        let result = transpiler.transpile_min(&a, &b);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("min"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_transpile_min_mixed() {
        let transpiler = Transpiler::new();
        let a = int_expr(3);
        let b = float_expr(7.0);
        let result = transpiler.transpile_min(&a, &b);
        assert!(result.is_ok());
        // Mixed should use float path due to one being float
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_transpile_max_integers() {
        let transpiler = Transpiler::new();
        let a = int_expr(3);
        let b = int_expr(7);
        let result = transpiler.transpile_max(&a, &b);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("std :: cmp :: max"));
    }

    #[test]
    fn test_transpile_max_floats() {
        let transpiler = Transpiler::new();
        let a = float_expr(3.0);
        let b = float_expr(7.0);
        let result = transpiler.transpile_max(&a, &b);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("max"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_transpile_floor() {
        let transpiler = Transpiler::new();
        let arg = float_expr(3.7);
        let result = transpiler.transpile_floor(&arg);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("floor"));
    }

    #[test]
    fn test_transpile_ceil() {
        let transpiler = Transpiler::new();
        let arg = float_expr(3.2);
        let result = transpiler.transpile_ceil(&arg);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("ceil"));
    }

    #[test]
    fn test_transpile_round() {
        let transpiler = Transpiler::new();
        let arg = float_expr(3.5);
        let result = transpiler.transpile_round(&arg);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("round"));
    }

    #[test]
    fn test_transpile_sqrt_with_integer() {
        let transpiler = Transpiler::new();
        let arg = int_expr(4);
        let result = transpiler.transpile_sqrt(&arg);
        assert!(result.is_ok());
        // Should cast to f64
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_transpile_pow_with_integers() {
        let transpiler = Transpiler::new();
        let base = int_expr(2);
        let exp = int_expr(3);
        let result = transpiler.transpile_pow(&base, &exp);
        assert!(result.is_ok());
        // Should cast both to f64
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("f64"));
        assert!(tokens_str.contains("powf"));
    }
}
