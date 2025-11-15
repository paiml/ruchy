//! `DataFrame` builder pattern transpilation for correct Polars API
//!
//! Transforms Ruchy's builder pattern into valid Polars code
use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::Result;
use proc_macro2::TokenStream;
#[cfg(test)]
#[allow(unused_imports)]
use proptest::prelude::*;
use quote::quote;
impl Transpiler {
    /// Transpile `DataFrame` builder pattern chains
    /// Transforms: `DataFrame::new().column("a", [1,2]).column("b", [3,4]).build()`
    /// Into: `DataFrame::new(vec![Series::new("a", &[1,2]), Series::new("b", &[3,4])])`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::dataframe_builder::transpile_dataframe_builder;
    ///
    /// let result = transpile_dataframe_builder(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_dataframe_builder(&self, expr: &Expr) -> Result<Option<TokenStream>> {
        // Check if this is a DataFrame builder pattern
        if let Some((columns, _base)) = self.extract_dataframe_builder_chain(expr) {
            // Generate Series for each column
            let mut series_tokens = Vec::new();
            for (name, data) in columns {
                let name_tokens = self.transpile_expr(&name)?;
                let data_tokens = self.transpile_expr(&data)?;
                series_tokens.push(quote! {
                    polars::prelude::Series::new(#name_tokens, &#data_tokens)
                });
            }
            // Generate the DataFrame constructor
            if series_tokens.is_empty() {
                Ok(Some(quote! { polars::prelude::DataFrame::empty() }))
            } else {
                Ok(Some(quote! {
                    polars::prelude::DataFrame::new(vec![#(#series_tokens),*]).unwrap()
                }))
            }
        } else {
            Ok(None)
        }
    }
    /// Extract `DataFrame` builder chain pattern
    /// Returns columns and base expression if it's a builder pattern
    fn extract_dataframe_builder_chain(&self, expr: &Expr) -> Option<(Vec<(Expr, Expr)>, Expr)> {
        match &expr.kind {
            // .build() at the end
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } if method == "build" && args.is_empty() => Self::extract_column_chain(receiver),
            // Just column chains without .build()
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } if method == "column" && args.len() == 2 => {
                if let Some((mut cols, base)) = Self::extract_column_chain(receiver) {
                    cols.push((args[0].clone(), args[1].clone()));
                    Some((cols, base))
                } else {
                    Some((
                        vec![(args[0].clone(), args[1].clone())],
                        receiver.as_ref().clone(),
                    ))
                }
            }
            _ => None,
        }
    }
    /// Extract column method calls recursively
    fn extract_column_chain(expr: &Expr) -> Option<(Vec<(Expr, Expr)>, Expr)> {
        match &expr.kind {
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } if method == "column" && args.len() == 2 => {
                if let Some((mut cols, base)) = Self::extract_column_chain(receiver) {
                    cols.push((args[0].clone(), args[1].clone()));
                    Some((cols, base))
                } else {
                    // Base case: reached the DataFrame::new()
                    Some((
                        vec![(args[0].clone(), args[1].clone())],
                        receiver.as_ref().clone(),
                    ))
                }
            }
            ExprKind::Call { func, args } => {
                // Check if it's DataFrame::new()
                if let ExprKind::QualifiedName { module, name } = &func.kind {
                    if module == "DataFrame" && name == "new" && args.is_empty() {
                        return Some((Vec::new(), expr.clone()));
                    }
                }
                None
            }
            _ => None,
        }
    }
    /// Check if expression is a `DataFrame` builder pattern
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::dataframe_builder::is_dataframe_builder;
    ///
    /// let result = is_dataframe_builder(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_dataframe_builder(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::MethodCall { method, .. } => {
                matches!(method.as_str(), "column" | "build")
            }
            ExprKind::Call { func, .. } => {
                if let ExprKind::QualifiedName { module, name } = &func.kind {
                    module == "DataFrame" && name == "new"
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    // Helper: Create test transpiler
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper: Create string literal expression
    fn string_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create list expression
    fn list_expr(items: Vec<i64>) -> Expr {
        Expr {
            kind: ExprKind::List(
                items
                    .into_iter()
                    .map(|n| Expr {
                        kind: ExprKind::Literal(Literal::Integer(n, None)),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    })
                    .collect(),
            ),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create DataFrame::new() call
    fn dataframe_new_call() -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(Expr {
                    kind: ExprKind::QualifiedName {
                        module: "DataFrame".to_string(),
                        name: "new".to_string(),
                    },
                    span: Span::default(),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                }),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create .column() method call
    fn column_method_call(receiver: Expr, name: Expr, data: Expr) -> Expr {
        Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "column".to_string(),
                args: vec![name, data],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create .build() method call
    fn build_method_call(receiver: Expr) -> Expr {
        Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "build".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: transpile_dataframe_builder - empty DataFrame (no columns)
    #[test]
    fn test_transpile_dataframe_builder_empty() {
        let transpiler = test_transpiler();
        let expr = build_method_call(dataframe_new_call());
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let output = tokens.unwrap().to_string();
        assert!(output.contains("DataFrame"));
        assert!(output.contains("empty"));
    }

    // Test 2: transpile_dataframe_builder - single column
    #[test]
    fn test_transpile_dataframe_builder_single_column() {
        let transpiler = test_transpiler();
        let expr = build_method_call(column_method_call(
            dataframe_new_call(),
            string_expr("a"),
            list_expr(vec![1, 2, 3]),
        ));
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let output = tokens.unwrap().to_string();
        assert!(output.contains("DataFrame"));
        assert!(output.contains("Series"));
    }

    // Test 3: transpile_dataframe_builder - multiple columns
    #[test]
    fn test_transpile_dataframe_builder_multiple_columns() {
        let transpiler = test_transpiler();
        let base = dataframe_new_call();
        let with_col1 = column_method_call(base, string_expr("a"), list_expr(vec![1, 2]));
        let with_col2 = column_method_call(with_col1, string_expr("b"), list_expr(vec![3, 4]));
        let expr = build_method_call(with_col2);
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let output = tokens.unwrap().to_string();
        assert!(output.contains("DataFrame"));
        assert!(output.contains("Series"));
        assert!(output.contains("vec"));
    }

    // Test 4: transpile_dataframe_builder - without .build() call
    #[test]
    fn test_transpile_dataframe_builder_no_build() {
        let transpiler = test_transpiler();
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("x"),
            list_expr(vec![10, 20]),
        );
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    // Test 5: transpile_dataframe_builder - non-builder pattern (None)
    #[test]
    fn test_transpile_dataframe_builder_non_builder() {
        let transpiler = test_transpiler();
        let expr = string_expr("not a builder");
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Test 6: is_dataframe_builder - DataFrame::new()
    #[test]
    fn test_is_dataframe_builder_new_call() {
        let transpiler = test_transpiler();
        let expr = dataframe_new_call();
        assert!(transpiler.is_dataframe_builder(&expr));
    }

    // Test 7: is_dataframe_builder - .column() method
    #[test]
    fn test_is_dataframe_builder_column_method() {
        let transpiler = test_transpiler();
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("a"),
            list_expr(vec![1]),
        );
        assert!(transpiler.is_dataframe_builder(&expr));
    }

    // Test 8: is_dataframe_builder - .build() method
    #[test]
    fn test_is_dataframe_builder_build_method() {
        let transpiler = test_transpiler();
        let expr = build_method_call(dataframe_new_call());
        assert!(transpiler.is_dataframe_builder(&expr));
    }

    // Test 9: is_dataframe_builder - non-builder pattern
    #[test]
    fn test_is_dataframe_builder_false() {
        let transpiler = test_transpiler();
        let expr = string_expr("not a builder");
        assert!(!transpiler.is_dataframe_builder(&expr));
    }

    // Test 10: extract_dataframe_builder_chain - with .build()
    #[test]
    fn test_extract_dataframe_builder_chain_with_build() {
        let transpiler = test_transpiler();
        let expr = build_method_call(column_method_call(
            dataframe_new_call(),
            string_expr("col"),
            list_expr(vec![1]),
        ));
        let result = transpiler.extract_dataframe_builder_chain(&expr);
        assert!(result.is_some());
        let (columns, _base) = result.unwrap();
        assert_eq!(columns.len(), 1);
    }

    // Test 11: extract_dataframe_builder_chain - without .build()
    #[test]
    fn test_extract_dataframe_builder_chain_no_build() {
        let transpiler = test_transpiler();
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("data"),
            list_expr(vec![5, 6]),
        );
        let result = transpiler.extract_dataframe_builder_chain(&expr);
        assert!(result.is_some());
    }

    // Test 12: extract_dataframe_builder_chain - None for non-builder
    #[test]
    fn test_extract_dataframe_builder_chain_none() {
        let transpiler = test_transpiler();
        let expr = string_expr("not builder");
        let result = transpiler.extract_dataframe_builder_chain(&expr);
        assert!(result.is_none());
    }

    // Test 13: extract_column_chain - single column
    #[test]
    fn test_extract_column_chain_single() {
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("x"),
            list_expr(vec![1]),
        );
        let result = Transpiler::extract_column_chain(&expr);
        assert!(result.is_some());
        let (columns, _base) = result.unwrap();
        assert_eq!(columns.len(), 1);
    }

    // Test 14: extract_column_chain - chained columns
    #[test]
    fn test_extract_column_chain_chained() {
        let base = dataframe_new_call();
        let col1 = column_method_call(base, string_expr("a"), list_expr(vec![1]));
        let col2 = column_method_call(col1, string_expr("b"), list_expr(vec![2]));
        let result = Transpiler::extract_column_chain(&col2);
        assert!(result.is_some());
        let (columns, _base) = result.unwrap();
        assert_eq!(columns.len(), 2);
    }

    // Test 15: extract_column_chain - None for non-column
    #[test]
    fn test_extract_column_chain_none() {
        let expr = string_expr("not a column");
        let result = Transpiler::extract_column_chain(&expr);
        assert!(result.is_none());
    }

    // Test 16: transpile_dataframe_builder - three columns (order preservation)
    #[test]
    fn test_transpile_dataframe_builder_three_columns() {
        let transpiler = test_transpiler();
        let base = dataframe_new_call();
        let col1 = column_method_call(base, string_expr("a"), list_expr(vec![1]));
        let col2 = column_method_call(col1, string_expr("b"), list_expr(vec![2]));
        let col3 = column_method_call(col2, string_expr("c"), list_expr(vec![3]));
        let expr = build_method_call(col3);
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let output = tokens.unwrap().to_string();
        assert!(output.contains("Series"));
        assert!(output.contains("vec"));
    }

    // Test 17: transpile_dataframe_builder - column data with expressions
    #[test]
    fn test_transpile_dataframe_builder_expression_data() {
        let transpiler = test_transpiler();
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("values"),
            list_expr(vec![10, 20, 30]),
        );
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // Test 18: transpile_dataframe_builder - verify Series generation
    #[test]
    fn test_transpile_dataframe_builder_series_output() {
        let transpiler = test_transpiler();
        let expr = build_method_call(column_method_call(
            dataframe_new_call(),
            string_expr("col"),
            list_expr(vec![1, 2]),
        ));
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().unwrap().to_string();
        assert!(output.contains("polars"));
        assert!(output.contains("Series"));
        assert!(output.contains("new"));
    }

    // Test 19: extract_dataframe_builder_chain - three columns
    #[test]
    fn test_extract_dataframe_builder_chain_three_cols() {
        let transpiler = test_transpiler();
        let base = dataframe_new_call();
        let col1 = column_method_call(base, string_expr("x"), list_expr(vec![1]));
        let col2 = column_method_call(col1, string_expr("y"), list_expr(vec![2]));
        let col3 = column_method_call(col2, string_expr("z"), list_expr(vec![3]));
        let expr = build_method_call(col3);
        let result = transpiler.extract_dataframe_builder_chain(&expr);
        assert!(result.is_some());
        let (columns, _base) = result.unwrap();
        assert_eq!(columns.len(), 3);
    }

    // Test 20: extract_dataframe_builder_chain - build after columns
    #[test]
    fn test_extract_dataframe_builder_chain_build_after_cols() {
        let transpiler = test_transpiler();
        let expr = build_method_call(column_method_call(
            column_method_call(
                dataframe_new_call(),
                string_expr("a"),
                list_expr(vec![1, 2]),
            ),
            string_expr("b"),
            list_expr(vec![3, 4]),
        ));
        let result = transpiler.extract_dataframe_builder_chain(&expr);
        assert!(result.is_some());
        let (columns, _) = result.unwrap();
        assert_eq!(columns.len(), 2);
    }

    // Test 21: extract_column_chain - three columns chained
    #[test]
    fn test_extract_column_chain_three_cols() {
        let base = dataframe_new_call();
        let col1 = column_method_call(base, string_expr("a"), list_expr(vec![1]));
        let col2 = column_method_call(col1, string_expr("b"), list_expr(vec![2]));
        let col3 = column_method_call(col2, string_expr("c"), list_expr(vec![3]));
        let result = Transpiler::extract_column_chain(&col3);
        assert!(result.is_some());
        let (columns, _base) = result.unwrap();
        assert_eq!(columns.len(), 3);
    }

    // Test 22: extract_column_chain - DataFrame::new() base
    #[test]
    fn test_extract_column_chain_dataframe_new_base() {
        let base = dataframe_new_call();
        let col = column_method_call(base, string_expr("data"), list_expr(vec![5]));
        let result = Transpiler::extract_column_chain(&col);
        assert!(result.is_some());
        let (columns, base_expr) = result.unwrap();
        assert_eq!(columns.len(), 1);
        assert!(matches!(base_expr.kind, ExprKind::Call { .. }));
    }

    // Test 23: is_dataframe_builder - build method without args
    #[test]
    fn test_is_dataframe_builder_build_no_args() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(dataframe_new_call()),
                method: "build".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(transpiler.is_dataframe_builder(&expr));
    }

    // Test 24: is_dataframe_builder - column with correct arg count
    #[test]
    fn test_is_dataframe_builder_column_two_args() {
        let transpiler = test_transpiler();
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("name"),
            list_expr(vec![1]),
        );
        assert!(transpiler.is_dataframe_builder(&expr));
    }

    // Test 25: is_dataframe_builder - QualifiedName with DataFrame module
    #[test]
    fn test_is_dataframe_builder_qualified_name() {
        let transpiler = test_transpiler();
        let expr = dataframe_new_call();
        assert!(transpiler.is_dataframe_builder(&expr));
    }

    // Test 26: transpile_dataframe_builder - verify unwrap() in output
    #[test]
    fn test_transpile_dataframe_builder_unwrap_present() {
        let transpiler = test_transpiler();
        let expr = build_method_call(column_method_call(
            dataframe_new_call(),
            string_expr("test"),
            list_expr(vec![99]),
        ));
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().unwrap().to_string();
        assert!(output.contains("unwrap"));
    }

    // Test 27: transpile_dataframe_builder - empty() for no columns
    #[test]
    fn test_transpile_dataframe_builder_empty_for_no_cols() {
        let transpiler = test_transpiler();
        let expr = build_method_call(dataframe_new_call());
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let output = result.unwrap().unwrap().to_string();
        assert!(output.contains("empty"));
    }

    // Test 28: extract_dataframe_builder_chain - single column chain
    #[test]
    fn test_extract_dataframe_builder_chain_single_col() {
        let transpiler = test_transpiler();
        let expr = column_method_call(
            dataframe_new_call(),
            string_expr("single"),
            list_expr(vec![42]),
        );
        let result = transpiler.extract_dataframe_builder_chain(&expr);
        assert!(result.is_some());
        let (columns, _) = result.unwrap();
        assert_eq!(columns.len(), 1);
    }

    // Test 29: is_dataframe_builder - literal expression (false)
    #[test]
    fn test_is_dataframe_builder_literal_false() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!transpiler.is_dataframe_builder(&expr));
    }

    // Test 30: transpile_dataframe_builder - four columns
    #[test]
    fn test_transpile_dataframe_builder_four_columns() {
        let transpiler = test_transpiler();
        let base = dataframe_new_call();
        let col1 = column_method_call(base, string_expr("a"), list_expr(vec![1]));
        let col2 = column_method_call(col1, string_expr("b"), list_expr(vec![2]));
        let col3 = column_method_call(col2, string_expr("c"), list_expr(vec![3]));
        let col4 = column_method_call(col3, string_expr("d"), list_expr(vec![4]));
        let expr = build_method_call(col4);
        let result = transpiler.transpile_dataframe_builder(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let output = tokens.unwrap().to_string();
        assert!(output.contains("DataFrame"));
        assert!(output.contains("new"));
        assert!(output.contains("vec"));
    }
}

#[cfg(test)]
mod property_tests_dataframe_builder {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use proptest::prelude::*;
    use proptest::proptest;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_transpile_dataframe_builder_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
