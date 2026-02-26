//! Type Conversion Built-in Function Transpilation
//!
//! This module handles transpilation of type conversion functions:
//! - `str()` - convert to string
//! - `int()` - convert to integer
//! - `float()` - convert to float
//! - `bool()` - convert to boolean
//!
//! **EXTREME TDD Round 58**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Try to transpile type conversion functions (str, int, float, bool)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ruchy::backend::transpiler::Transpiler;
    /// let mut transpiler = Transpiler::new();
    /// // str(42) -> 42.to_string()
    /// // int("42") -> "42".parse::<i64>().expect("...")
    /// // float(42) -> 42 as f64
    /// // bool(1) -> 1 != 0
    /// ```
    /// Complexity: 5 (within Toyota Way limits)
    pub fn try_transpile_type_conversion(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        // Delegate to refactored version with reduced complexity
        // Original complexity: 62, New complexity: <20 per function
        self.try_transpile_type_conversion_refactored(base_name, args)
    }

    /// Old implementation kept for reference (will be removed after verification)
    /// Complexity: 5 (within Toyota Way limits)
    pub fn try_transpile_type_conversion_old(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "str" => self.transpile_str_conversion(args).map(Some),
            "int" => self.transpile_int_conversion(args).map(Some),
            "float" => self.transpile_float_conversion(args).map(Some),
            "bool" => self.transpile_bool_conversion(args).map(Some),
            _ => Ok(None),
        }
    }

    /// Handle `str()` type conversion - extract string representation
    /// Complexity: 3 (within Toyota Way limits)
    pub fn transpile_str_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("str() expects exactly 1 argument");
        }
        let value = self.transpile_expr(&args[0])?;
        Ok(quote! { format!("{}", #value) })
    }

    /// Handle `int()` type conversion with literal-specific optimizations
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_int_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("int() expects exactly 1 argument");
        }
        // Check if the argument is a literal for compile-time optimizations
        match &args[0].kind {
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { #value.parse::<i64>().expect("Failed to parse integer") })
            }
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                    let value = self.transpile_expr(&args[0])?;
                    Ok(quote! { #value.parse::<i64>().expect("Failed to parse integer") })
                } else {
                    self.transpile_int_generic(&args[0])
                }
            }
            ExprKind::Literal(Literal::Float(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value as i64) })
            }
            ExprKind::Literal(Literal::Bool(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { if #value { 1i64 } else { 0i64 } })
            }
            _ => self.transpile_int_generic(&args[0]),
        }
    }

    /// Generic int conversion for non-literal expressions
    /// Complexity: 2 (within Toyota Way limits)
    pub fn transpile_int_generic(&self, expr: &Expr) -> Result<TokenStream> {
        let value = self.transpile_expr(expr)?;
        Ok(quote! { (#value as i64) })
    }

    /// Handle `float()` type conversion with literal-specific optimizations
    /// Complexity: 7 (within Toyota Way limits)
    pub fn transpile_float_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("float() expects exactly 1 argument");
        }
        // Check if the argument is a literal for compile-time optimizations
        match &args[0].kind {
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { #value.parse::<f64>().expect("Failed to parse float") })
            }
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                if let crate::frontend::ast::StringPart::Text(_) = &parts[0] {
                    let value = self.transpile_expr(&args[0])?;
                    Ok(quote! { #value.parse::<f64>().expect("Failed to parse float") })
                } else {
                    self.transpile_float_generic(&args[0])
                }
            }
            ExprKind::Literal(Literal::Integer(_, _)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value as f64) })
            }
            _ => self.transpile_float_generic(&args[0]),
        }
    }

    /// Generic float conversion for non-literal expressions
    /// Complexity: 2 (within Toyota Way limits)
    pub fn transpile_float_generic(&self, expr: &Expr) -> Result<TokenStream> {
        let value = self.transpile_expr(expr)?;
        Ok(quote! { (#value as f64) })
    }

    /// Handle `bool()` type conversion with type-specific logic
    /// Complexity: 7 (within Toyota Way limits)
    pub fn transpile_bool_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("bool() expects exactly 1 argument");
        }
        // Check the type of the argument to generate appropriate conversion
        match &args[0].kind {
            ExprKind::Literal(Literal::Integer(_, _)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value != 0) })
            }
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { !#value.is_empty() })
            }
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { !#value.is_empty() })
            }
            ExprKind::Literal(Literal::Bool(_)) => {
                // Boolean already, just pass through
                let value = self.transpile_expr(&args[0])?;
                Ok(value)
            }
            _ => {
                // Generic case - for numbers check != 0
                let value = self.transpile_expr(&args[0])?;
                Ok(quote! { (#value != 0) })
            }
        }
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

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn bool_expr(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // transpile_str_conversion tests
    // ========================================================================

    #[test]
    fn test_transpile_str_conversion_integer() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.transpile_str_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("format"));
    }

    #[test]
    fn test_transpile_str_conversion_float() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.14)];
        let result = transpiler.transpile_str_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("format"));
    }

    #[test]
    fn test_transpile_str_conversion_no_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.transpile_str_conversion(&args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exactly 1 argument"));
    }

    #[test]
    fn test_transpile_str_conversion_too_many_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(1), int_expr(2)];
        let result = transpiler.transpile_str_conversion(&args);
        assert!(result.is_err());
    }

    // ========================================================================
    // transpile_int_conversion tests
    // ========================================================================

    #[test]
    fn test_transpile_int_conversion_from_string() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("42")];
        let result = transpiler.transpile_int_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("parse"));
        assert!(tokens_str.contains("i64"));
    }

    #[test]
    fn test_transpile_int_conversion_from_float() {
        let transpiler = Transpiler::new();
        let args = vec![float_expr(3.7)];
        let result = transpiler.transpile_int_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("as i64"));
    }

    #[test]
    fn test_transpile_int_conversion_from_bool() {
        let transpiler = Transpiler::new();
        let args = vec![bool_expr(true)];
        let result = transpiler.transpile_int_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("1i64"));
        assert!(tokens_str.contains("0i64"));
    }

    #[test]
    fn test_transpile_int_conversion_generic() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("x")];
        let result = transpiler.transpile_int_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("as i64"));
    }

    #[test]
    fn test_transpile_int_conversion_no_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.transpile_int_conversion(&args);
        assert!(result.is_err());
    }

    // ========================================================================
    // transpile_float_conversion tests
    // ========================================================================

    #[test]
    fn test_transpile_float_conversion_from_string() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("3.14")];
        let result = transpiler.transpile_float_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("parse"));
        assert!(tokens_str.contains("f64"));
    }

    #[test]
    fn test_transpile_float_conversion_from_int() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.transpile_float_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("as f64"));
    }

    #[test]
    fn test_transpile_float_conversion_generic() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("x")];
        let result = transpiler.transpile_float_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("as f64"));
    }

    #[test]
    fn test_transpile_float_conversion_no_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.transpile_float_conversion(&args);
        assert!(result.is_err());
    }

    // ========================================================================
    // transpile_bool_conversion tests
    // ========================================================================

    #[test]
    fn test_transpile_bool_conversion_from_int() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(1)];
        let result = transpiler.transpile_bool_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("!= 0"));
    }

    #[test]
    fn test_transpile_bool_conversion_from_string() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("hello")];
        let result = transpiler.transpile_bool_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("is_empty"));
    }

    #[test]
    fn test_transpile_bool_conversion_from_bool() {
        let transpiler = Transpiler::new();
        let args = vec![bool_expr(true)];
        let result = transpiler.transpile_bool_conversion(&args);
        assert!(result.is_ok());
        // Boolean passthrough - should just return the value
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("true"));
    }

    #[test]
    fn test_transpile_bool_conversion_generic() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("x")];
        let result = transpiler.transpile_bool_conversion(&args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("!= 0"));
    }

    #[test]
    fn test_transpile_bool_conversion_no_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.transpile_bool_conversion(&args);
        assert!(result.is_err());
    }

    // ========================================================================
    // transpile_int_generic tests
    // ========================================================================

    #[test]
    fn test_transpile_int_generic() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("value");
        let result = transpiler.transpile_int_generic(&expr);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("as i64"));
    }

    // ========================================================================
    // transpile_float_generic tests
    // ========================================================================

    #[test]
    fn test_transpile_float_generic() {
        let transpiler = Transpiler::new();
        let expr = ident_expr("value");
        let result = transpiler.transpile_float_generic(&expr);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("as f64"));
    }

    // ========================================================================
    // try_transpile_type_conversion_old tests
    // ========================================================================

    #[test]
    fn test_try_transpile_type_conversion_old_str() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_type_conversion_old("str", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_try_transpile_type_conversion_old_int() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("42")];
        let result = transpiler.try_transpile_type_conversion_old("int", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_try_transpile_type_conversion_old_float() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_type_conversion_old("float", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_try_transpile_type_conversion_old_bool() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(1)];
        let result = transpiler.try_transpile_type_conversion_old("bool", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_try_transpile_type_conversion_old_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_type_conversion_old("unknown_type", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
