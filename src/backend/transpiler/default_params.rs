//! Default Parameter Transpilation
//!
//! PDCA-21: Handles transpilation of functions with default parameter values.
//! Rust doesn't support default parameters, so we fill in defaults at call sites
//! when fewer arguments are provided than the function expects.

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::Result;
use proc_macro2::TokenStream;

impl Transpiler {
    /// Fill in default values for missing arguments at call sites.
    /// When a function has default parameters and is called with fewer args,
    /// append the transpiled default values.
    pub(super) fn fill_default_args(
        &self,
        func_name: &str,
        provided_args: Vec<TokenStream>,
    ) -> Result<Vec<TokenStream>> {
        let sig = match self.function_signatures.get(func_name) {
            Some(sig) => sig,
            None => return Ok(provided_args),
        };

        let expected_count = sig.param_types.len();
        let provided_count = provided_args.len();

        if provided_count >= expected_count {
            return Ok(provided_args);
        }

        let defaults = match &sig.default_values {
            Some(dv) => dv,
            None => return Ok(provided_args),
        };

        let mut result = provided_args;
        for i in provided_count..expected_count {
            if let Some(Some(default_expr)) = defaults.get(i) {
                let default_tokens = self.transpile_expr(default_expr)?;
                // String literals need .to_string() for String params
                let param_type = sig.param_types.get(i).map(|s| s.as_str()).unwrap_or("");
                if param_type == "String" || param_type == "Unknown" || param_type == "Any" {
                    if matches!(
                        &default_expr.kind,
                        crate::frontend::ast::ExprKind::Literal(
                            crate::frontend::ast::Literal::String(_)
                        )
                    ) {
                        result.push(quote::quote! { #default_tokens.to_string() });
                        continue;
                    }
                }
                result.push(default_tokens);
            } else {
                break; // No default for this position, can't fill further
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::transpiler::FunctionSignature;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use quote::quote;

    fn make_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn string_expr(s: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(s.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            contracts: Vec::new(),
        }
    }

    fn int_expr(n: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(n, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            contracts: Vec::new(),
        }
    }

    #[test]
    fn test_fill_default_args_no_signature() {
        let transpiler = make_transpiler();
        let args = vec![quote! { "Smith".to_string() }];
        let result = transpiler
            .fill_default_args("unknown_fn", args.clone())
            .unwrap();
        assert_eq!(result.len(), 1, "No signature should return args as-is");
    }

    #[test]
    fn test_fill_default_args_all_provided() {
        let mut transpiler = make_transpiler();
        transpiler.function_signatures.insert(
            "greet".to_string(),
            FunctionSignature {
                name: "greet".to_string(),
                param_types: vec!["String".to_string(), "String".to_string()],
                default_values: Some(vec![None, Some(Box::new(string_expr("Mr.")))]),
            },
        );
        let args = vec![quote! { name }, quote! { title }];
        let result = transpiler.fill_default_args("greet", args).unwrap();
        assert_eq!(result.len(), 2, "All args provided should not add defaults");
    }

    #[test]
    fn test_fill_default_args_missing_one_with_default() {
        let mut transpiler = make_transpiler();
        transpiler.function_signatures.insert(
            "greet".to_string(),
            FunctionSignature {
                name: "greet".to_string(),
                param_types: vec!["String".to_string(), "String".to_string()],
                default_values: Some(vec![None, Some(Box::new(string_expr("Mr.")))]),
            },
        );
        let args = vec![quote! { "Smith".to_string() }];
        let result = transpiler.fill_default_args("greet", args).unwrap();
        assert_eq!(result.len(), 2, "Should fill in default for missing arg");
        let second_arg = result[1].to_string();
        assert!(
            second_arg.contains("Mr."),
            "Default value should contain 'Mr.', got: {second_arg}"
        );
    }

    #[test]
    fn test_fill_default_args_integer_default() {
        let mut transpiler = make_transpiler();
        transpiler.function_signatures.insert(
            "repeat".to_string(),
            FunctionSignature {
                name: "repeat".to_string(),
                param_types: vec!["String".to_string(), "i32".to_string()],
                default_values: Some(vec![None, Some(Box::new(int_expr(1)))]),
            },
        );
        let args = vec![quote! { "hello".to_string() }];
        let result = transpiler.fill_default_args("repeat", args).unwrap();
        assert_eq!(result.len(), 2);
        let second_arg = result[1].to_string();
        assert!(
            second_arg.contains('1'),
            "Default int should be 1, got: {second_arg}"
        );
    }

    #[test]
    fn test_fill_default_args_no_defaults_stored() {
        let mut transpiler = make_transpiler();
        transpiler.function_signatures.insert(
            "add".to_string(),
            FunctionSignature {
                name: "add".to_string(),
                param_types: vec!["i32".to_string(), "i32".to_string()],
                default_values: None,
            },
        );
        let args = vec![quote! { 1 }];
        let result = transpiler.fill_default_args("add", args).unwrap();
        assert_eq!(result.len(), 1, "No defaults should not add anything");
    }
}
