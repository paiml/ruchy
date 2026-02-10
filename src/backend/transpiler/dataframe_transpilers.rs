//! `DataFrame` transpilation helpers
//! EXTREME TDD Round 80: Extracted from statements.rs
//!
//! This module handles `DataFrame` builder pattern transpilation.
//! Note: `try_transpile_dataframe_function_impl` is in `call_transpilation.rs`

use crate::frontend::ast::{Expr, ExprKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use super::Transpiler;

impl Transpiler {
    /// DEFECT-TRANSPILER-DF-002: Inline `DataFrame` builder pattern transpilation
    /// Transforms: `DataFrame::new().column("a", [1,2]).build()`
    /// Into: `DataFrame::new(vec![Series::new("a", &[1,2])])`
    /// EXTREME TDD Round 80: Extracted from statements.rs
    pub(crate) fn try_transpile_dataframe_builder_inline_impl(
        &self,
        expr: &Expr,
    ) -> Result<Option<TokenStream>> {
        // Check if this is a builder pattern ending in .build()
        let (columns, _base) = match &expr.kind {
            ExprKind::MethodCall {
                receiver, method, ..
            } if method == "build" => {
                if let Some(result) = Self::extract_dataframe_columns_impl(receiver) {
                    result
                } else {
                    return Ok(None);
                }
            }
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } if method == "column" && args.len() == 2 => {
                // Builder without .build() - still valid
                let mut cols = vec![(args[0].clone(), args[1].clone())];
                if let Some((mut prev_cols, base)) = Self::extract_dataframe_columns_impl(receiver)
                {
                    prev_cols.append(&mut cols);
                    (prev_cols, base)
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        // Generate Series for each column
        let mut series_tokens = Vec::new();
        for (name, data) in columns {
            let name_tokens = self.transpile_expr(&name)?;
            let data_tokens = self.transpile_expr(&data)?;
            series_tokens.push(quote! {
                polars::prelude::Series::new(#name_tokens, &#data_tokens)
            });
        }

        // Generate DataFrame constructor
        if series_tokens.is_empty() {
            Ok(Some(quote! { polars::prelude::DataFrame::empty() }))
        } else {
            Ok(Some(quote! {
                polars::prelude::DataFrame::new(vec![#(#series_tokens),*])
                    .expect("Failed to create DataFrame")
            }))
        }
    }

    /// Extract `DataFrame` column chain recursively
    /// EXTREME TDD Round 80: Extracted from statements.rs
    fn extract_dataframe_columns_impl(expr: &Expr) -> Option<(Vec<(Expr, Expr)>, Expr)> {
        match &expr.kind {
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } if method == "column" && args.len() == 2 => {
                if let Some((mut cols, base)) = Self::extract_dataframe_columns_impl(receiver) {
                    cols.push((args[0].clone(), args[1].clone()));
                    Some((cols, base))
                } else {
                    // Check if receiver is DataFrame::new()
                    if let ExprKind::Call {
                        func,
                        args: call_args,
                    } = &receiver.kind
                    {
                        // Handle both Identifier("DataFrame::new") and QualifiedName
                        let is_dataframe_new = match &func.kind {
                            ExprKind::Identifier(name) if name == "DataFrame::new" => true,
                            ExprKind::QualifiedName { module, name }
                                if module == "DataFrame" && name == "new" =>
                            {
                                true
                            }
                            _ => false,
                        };
                        if is_dataframe_new && call_args.is_empty() {
                            return Some((
                                vec![(args[0].clone(), args[1].clone())],
                                receiver.as_ref().clone(),
                            ));
                        }
                    }
                    None
                }
            }
            ExprKind::Call { func, args } if args.is_empty() => {
                // Handle both Identifier("DataFrame::new") and QualifiedName
                let is_dataframe_new = match &func.kind {
                    ExprKind::Identifier(name) if name == "DataFrame::new" => true,
                    ExprKind::QualifiedName { module, name }
                        if module == "DataFrame" && name == "new" =>
                    {
                        true
                    }
                    _ => false,
                };
                if is_dataframe_new {
                    return Some((Vec::new(), expr.clone()));
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    #[test]
    fn test_dataframe_builder_inline_empty() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"DataFrame::new().build()"#);
        let ast = parser.parse().expect("parse");
        // Use full transpilation to test indirectly
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_dataframe_columns_not_dataframe() {
        // Non-DataFrame expression should return None
        // Parse a simple expression and verify it's not recognized as DataFrame
        let mut parser = Parser::new(r#"foo()"#);
        let ast = parser.parse().expect("parse");
        let mut transpiler = create_transpiler();
        // Should transpile successfully but not as DataFrame builder
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Should NOT contain polars since it's not a DataFrame
        assert!(!tokens.contains("polars::prelude::DataFrame"));
    }

    #[test]
    fn test_dataframe_builder_with_columns() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"DataFrame::new().column("a", [1, 2]).build()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("polars"));
    }

    #[test]
    fn test_dataframe_builder_not_matching() {
        let mut transpiler = create_transpiler();
        // Regular function call - not a DataFrame builder
        let mut parser = Parser::new(r#"foo()"#);
        let ast = parser.parse().expect("parse");
        // This should still transpile - just not as a DataFrame builder
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    // ===== EXTREME TDD Round 156 - DataFrame Transpilation Tests =====

    #[test]
    fn test_dataframe_builder_multiple_columns() {
        let mut transpiler = create_transpiler();
        let mut parser =
            Parser::new(r#"DataFrame::new().column("a", [1]).column("b", [2]).build()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("polars"));
    }

    #[test]
    fn test_dataframe_builder_string_values() {
        let mut transpiler = create_transpiler();
        let mut parser =
            Parser::new(r#"DataFrame::new().column("name", ["Alice", "Bob"]).build()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_builder_without_build() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"DataFrame::new().column("x", [1, 2, 3])"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_qualified_name() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"DataFrame.new().build()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_dataframe_method_call() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"vec.push(42)"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(!tokens.contains("DataFrame::empty"));
    }

    #[test]
    fn test_dataframe_builder_float_values() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"DataFrame::new().column("vals", [1.0, 2.5, 3.7]).build()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_builder_empty_array() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"DataFrame::new().column("empty", []).build()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_nested_method_chain() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"x.y.z()"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpiler_new_returns_valid_instance() {
        let transpiler = create_transpiler();
        // Verify transpiler can process basic expressions
        let expr = Expr {
            kind: ExprKind::Literal(crate::frontend::ast::Literal::Integer(42, None)),
            span: crate::frontend::ast::Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_builder_inline_impl_none_on_non_build() {
        let transpiler = create_transpiler();
        let expr = Expr {
            kind: ExprKind::Literal(crate::frontend::ast::Literal::Integer(42, None)),
            span: crate::frontend::ast::Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler
            .try_transpile_dataframe_builder_inline_impl(&expr)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_dataframe_builder_three_columns() {
        let mut transpiler = create_transpiler();
        let code = r#"DataFrame::new().column("a", [1]).column("b", [2]).column("c", [3]).build()"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_builder_mixed_types() {
        let mut transpiler = create_transpiler();
        let code = r#"DataFrame::new().column("nums", [1, 2]).column("strs", ["a", "b"]).build()"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_columns_returns_none_for_invalid() {
        // Test that extract_dataframe_columns_impl returns None for invalid expressions
        let expr = Expr {
            kind: ExprKind::Identifier("foo".to_string()),
            span: crate::frontend::ast::Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = Transpiler::extract_dataframe_columns_impl(&expr);
        assert!(result.is_none());
    }

    #[test]
    fn test_dataframe_variable_binding() {
        let mut transpiler = create_transpiler();
        let code = r#"let df = DataFrame::new().column("x", [1]).build()"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    // ========================================================================
    // extract_dataframe_columns_impl direct unit tests
    // ========================================================================

    fn make_expr_direct(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: crate::frontend::ast::Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    #[test]
    fn test_extract_columns_dataframe_new_identifier() {
        // DataFrame::new() expressed as Identifier("DataFrame::new") with empty call
        let func = make_expr_direct(ExprKind::Identifier("DataFrame::new".to_string()));
        let call = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&call);
        assert!(result.is_some());
        let (cols, _base) = result.unwrap();
        assert!(cols.is_empty(), "DataFrame::new() alone should have no columns");
    }

    #[test]
    fn test_extract_columns_dataframe_new_qualified() {
        // DataFrame::new() expressed as QualifiedName
        let func = make_expr_direct(ExprKind::QualifiedName {
            module: "DataFrame".to_string(),
            name: "new".to_string(),
        });
        let call = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&call);
        assert!(result.is_some());
        let (cols, _base) = result.unwrap();
        assert!(cols.is_empty());
    }

    #[test]
    fn test_extract_columns_non_dataframe_call() {
        // foo() — not a DataFrame call
        let func = make_expr_direct(ExprKind::Identifier("foo".to_string()));
        let call = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&call);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_columns_call_with_args() {
        // DataFrame::new(some_arg) — has args, so doesn't match
        let func = make_expr_direct(ExprKind::Identifier("DataFrame::new".to_string()));
        let call = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![make_expr_direct(ExprKind::Literal(
                crate::frontend::ast::Literal::Integer(1, None),
            ))],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&call);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_columns_single_column_on_dataframe_new() {
        // DataFrame::new().column("a", [1])
        let func = make_expr_direct(ExprKind::Identifier("DataFrame::new".to_string()));
        let df_new = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let col_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("a".to_string()),
        ));
        let col_data = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
        )]));
        let with_col = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(df_new),
            method: "column".to_string(),
            args: vec![col_name, col_data],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_col);
        assert!(result.is_some());
        let (cols, _base) = result.unwrap();
        assert_eq!(cols.len(), 1);
    }

    #[test]
    fn test_extract_columns_chained_columns() {
        // DataFrame::new().column("a", [1]).column("b", [2])
        let func = make_expr_direct(ExprKind::Identifier("DataFrame::new".to_string()));
        let df_new = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let col1_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("a".to_string()),
        ));
        let col1_data = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
        )]));
        let with_col1 = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(df_new),
            method: "column".to_string(),
            args: vec![col1_name, col1_data],
        });
        let col2_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("b".to_string()),
        ));
        let col2_data = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(2, None)),
        )]));
        let with_col2 = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(with_col1),
            method: "column".to_string(),
            args: vec![col2_name, col2_data],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_col2);
        assert!(result.is_some());
        let (cols, _base) = result.unwrap();
        assert_eq!(cols.len(), 2);
    }

    #[test]
    fn test_extract_columns_column_on_non_dataframe() {
        // foo().column("a", [1]) — receiver is not DataFrame::new()
        let func = make_expr_direct(ExprKind::Identifier("foo".to_string()));
        let call = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let col_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("a".to_string()),
        ));
        let col_data = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
        )]));
        let with_col = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(call),
            method: "column".to_string(),
            args: vec![col_name, col_data],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_col);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_columns_method_not_column() {
        // DataFrame::new().filter("a", [1]) — method is "filter" not "column"
        let func = make_expr_direct(ExprKind::Identifier("DataFrame::new".to_string()));
        let df_new = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let arg1 = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("a".to_string()),
        ));
        let arg2 = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
        )]));
        let with_filter = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(df_new),
            method: "filter".to_string(),
            args: vec![arg1, arg2],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_filter);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_columns_column_wrong_arg_count() {
        // DataFrame::new().column("a") — only 1 arg, needs 2
        let func = make_expr_direct(ExprKind::Identifier("DataFrame::new".to_string()));
        let df_new = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let col_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("a".to_string()),
        ));
        let with_col = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(df_new),
            method: "column".to_string(),
            args: vec![col_name],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_col);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_columns_qualified_name_in_column_chain() {
        // Using QualifiedName form: DataFrame.new().column("x", [1])
        let func = make_expr_direct(ExprKind::QualifiedName {
            module: "DataFrame".to_string(),
            name: "new".to_string(),
        });
        let df_new = make_expr_direct(ExprKind::Call {
            func: Box::new(func),
            args: vec![],
        });
        let col_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("x".to_string()),
        ));
        let col_data = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
        )]));
        let with_col = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(df_new),
            method: "column".to_string(),
            args: vec![col_name, col_data],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_col);
        assert!(result.is_some());
        let (cols, _base) = result.unwrap();
        assert_eq!(cols.len(), 1);
    }

    #[test]
    fn test_extract_columns_column_on_non_call_receiver() {
        // x.column("a", [1]) — receiver is an identifier, not a Call
        let receiver = make_expr_direct(ExprKind::Identifier("x".to_string()));
        let col_name = make_expr_direct(ExprKind::Literal(
            crate::frontend::ast::Literal::String("a".to_string()),
        ));
        let col_data = make_expr_direct(ExprKind::List(vec![make_expr_direct(
            ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
        )]));
        let with_col = make_expr_direct(ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method: "column".to_string(),
            args: vec![col_name, col_data],
        });
        let result = Transpiler::extract_dataframe_columns_impl(&with_col);
        assert!(result.is_none());
    }
}
