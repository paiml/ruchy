//! Advanced Math Function Transpilation
//!
//! This module handles transpilation of advanced math functions:
//! - Trigonometric: sin, cos, tan
//! - Logarithmic: log (natural), log10
//! - Random: `random()`
//! - Trueno SIMD: `trueno_sum`, `trueno_mean`, `trueno_variance`, `trueno_std_dev`, `trueno_dot`
//!
//! **EXTREME TDD Round 59**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Try to transpile advanced math functions (sin, cos, tan, log, log10, random)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ruchy::backend::transpiler::Transpiler;
    /// let mut transpiler = Transpiler::new();
    /// // sin(x) -> x.sin()
    /// // cos(x) -> x.cos()
    /// // log(x) -> x.ln()
    /// // random() -> pseudo-random f64
    /// ```
    /// Complexity: 7 (within Toyota Way limits)
    pub fn try_transpile_math_functions(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "sin" | "cos" | "tan" => {
                if args.len() != 1 {
                    bail!("{base_name}() expects exactly 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                let method = proc_macro2::Ident::new(base_name, proc_macro2::Span::call_site());
                Ok(Some(quote! { ((#value as f64).#method()) }))
            }
            "log" => {
                if args.len() != 1 {
                    bail!("log() expects exactly 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { ((#value as f64).ln()) }))
            }
            "log10" => {
                if args.len() != 1 {
                    bail!("log10() expects exactly 1 argument");
                }
                let value = self.transpile_expr(&args[0])?;
                Ok(Some(quote! { ((#value as f64).log10()) }))
            }
            "random" => {
                if !args.is_empty() {
                    bail!("random() expects no arguments");
                }
                // Use a simple pseudo-random generator
                Ok(Some(quote! {
                    {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_else(|_| std::time::Duration::from_secs(1))
                            .as_nanos() as u64;
                        // Use a safe LCG that won't overflow
                        let a = 1664525u64;
                        let c = 1013904223u64;
                        let m = 1u64 << 32;
                        ((seed.wrapping_mul(a).wrapping_add(c)) % m) as f64 / m as f64
                    }
                }))
            }
            _ => Ok(None),
        }
    }

    /// Handle Trueno SIMD-accelerated numeric functions (TRUENO-001)
    ///
    /// Per spec Section 5.1: Trueno primitives for SIMD-accelerated tensor operations.
    /// These functions use Kahan summation and SIMD backends for numerical stability.
    ///
    /// # Supported Functions
    /// - `trueno_sum(arr)` - Kahan-compensated summation (O(ε) error)
    /// - `trueno_mean(arr)` - Mean using Kahan summation
    /// - `trueno_variance(arr)` - Two-pass variance with Kahan
    /// - `trueno_std_dev(arr)` - Standard deviation
    /// - `trueno_dot(a, b)` - SIMD-accelerated dot product
    ///
    /// Complexity: 6 (within Toyota Way limits)
    pub fn try_transpile_trueno_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "trueno_sum" => {
                if args.len() != 1 {
                    bail!("trueno_sum() expects exactly 1 argument (slice of f64)");
                }
                let arr = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    ruchy::stdlib::trueno_bridge::kahan_sum(&#arr)
                }))
            }
            "trueno_mean" => {
                if args.len() != 1 {
                    bail!("trueno_mean() expects exactly 1 argument (slice of f64)");
                }
                let arr = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    ruchy::stdlib::trueno_bridge::mean(&#arr)
                }))
            }
            "trueno_variance" => {
                if args.len() != 1 {
                    bail!("trueno_variance() expects exactly 1 argument (slice of f64)");
                }
                let arr = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    ruchy::stdlib::trueno_bridge::variance(&#arr)
                }))
            }
            "trueno_std_dev" => {
                if args.len() != 1 {
                    bail!("trueno_std_dev() expects exactly 1 argument (slice of f64)");
                }
                let arr = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    ruchy::stdlib::trueno_bridge::std_dev(&#arr)
                }))
            }
            "trueno_dot" => {
                if args.len() != 2 {
                    bail!("trueno_dot() expects exactly 2 arguments (two slices)");
                }
                let a = self.transpile_expr(&args[0])?;
                let b = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    ruchy::stdlib::trueno_bridge::dot(&#a, &#b).expect("dot product failed")
                }))
            }
            _ => Ok(None),
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

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn float_expr(f: f64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Float(f)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn list_expr(items: Vec<Expr>) -> Expr {
        make_expr(ExprKind::List(items))
    }

    // ========================================================================
    // try_transpile_math_functions tests - trig
    // ========================================================================

    #[test]
    fn test_try_transpile_math_functions_sin() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(1.0)];
        let result = transpiler.try_transpile_math_functions("sin", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("sin"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_try_transpile_math_functions_cos() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(0.0)];
        let result = transpiler.try_transpile_math_functions("cos", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("cos"));
    }

    #[test]
    fn test_try_transpile_math_functions_tan() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(0.5)];
        let result = transpiler.try_transpile_math_functions("tan", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("tan"));
    }

    #[test]
    fn test_try_transpile_math_functions_trig_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(1.0), float_expr(2.0)];
        let result = transpiler.try_transpile_math_functions("sin", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exactly 1 argument"));
    }

    // ========================================================================
    // try_transpile_math_functions tests - log
    // ========================================================================

    #[test]
    fn test_try_transpile_math_functions_log() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(2.718)];
        let result = transpiler.try_transpile_math_functions("log", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("ln"));
    }

    #[test]
    fn test_try_transpile_math_functions_log10() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(100.0)];
        let result = transpiler.try_transpile_math_functions("log10", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("log10"));
    }

    #[test]
    fn test_try_transpile_math_functions_log_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_math_functions("log", &args);
        assert!(result.is_err());
    }

    // ========================================================================
    // try_transpile_math_functions tests - random
    // ========================================================================

    #[test]
    fn test_try_transpile_math_functions_random() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_math_functions("random", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("SystemTime"));
        assert!(tokens_str.contains("UNIX_EPOCH"));
    }

    #[test]
    fn test_try_transpile_math_functions_random_with_args() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(1.0)];
        let result = transpiler.try_transpile_math_functions("random", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no arguments"));
    }

    #[test]
    fn test_try_transpile_math_functions_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(1.0)];
        let result = transpiler.try_transpile_math_functions("unknown_func", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // try_transpile_trueno_function tests
    // ========================================================================

    #[test]
    fn test_try_transpile_trueno_sum() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("arr")];
        let result = transpiler.try_transpile_trueno_function("trueno_sum", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("kahan_sum"));
    }

    #[test]
    fn test_try_transpile_trueno_mean() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("data")];
        let result = transpiler.try_transpile_trueno_function("trueno_mean", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("mean"));
    }

    #[test]
    fn test_try_transpile_trueno_variance() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("values")];
        let result = transpiler.try_transpile_trueno_function("trueno_variance", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("variance"));
    }

    #[test]
    fn test_try_transpile_trueno_std_dev() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("samples")];
        let result = transpiler.try_transpile_trueno_function("trueno_std_dev", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("std_dev"));
    }

    #[test]
    fn test_try_transpile_trueno_dot() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("a"), ident_expr("b")];
        let result = transpiler.try_transpile_trueno_function("trueno_dot", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("dot"));
    }

    #[test]
    fn test_try_transpile_trueno_sum_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_trueno_function("trueno_sum", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_transpile_trueno_dot_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("a")];
        let result = transpiler.try_transpile_trueno_function("trueno_dot", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exactly 2 arguments"));
    }

    #[test]
    fn test_try_transpile_trueno_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("x")];
        let result = transpiler.try_transpile_trueno_function("trueno_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_trueno_mean_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("a"), ident_expr("b")];
        let result = transpiler.try_transpile_trueno_function("trueno_mean", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_transpile_trueno_variance_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_trueno_function("trueno_variance", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_transpile_trueno_std_dev_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("a"), ident_expr("b")];
        let result = transpiler.try_transpile_trueno_function("trueno_std_dev", &args);
        assert!(result.is_err());
    }

    // ========================================================================
    // Integration tests with list expressions
    // ========================================================================

    #[test]
    fn test_trueno_with_list_expr() {
        let transpiler = Transpiler::new();
        let args = vec![list_expr(vec![float_expr(1.0), float_expr(2.0)])];
        let result = transpiler.try_transpile_trueno_function("trueno_sum", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }
}
