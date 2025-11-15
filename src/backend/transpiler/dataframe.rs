//! `DataFrame` transpilation for Polars integration
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::doc_markdown)]
use super::*;
use crate::frontend::ast::{AggregateOp, DataFrameColumn, DataFrameOp, JoinType};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
impl Transpiler {
    /// Transpiles DataFrame literals (df![] syntax)
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let result = transpiler.transpile_dataframe(&[]);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_dataframe(&self, columns: &[DataFrameColumn]) -> Result<TokenStream> {
        if columns.is_empty() {
            // Empty DataFrame
            return Ok(quote! {
                polars::prelude::DataFrame::empty()
            });
        }
        let mut series_tokens = Vec::new();
        for column in columns {
            let col_name = &column.name;
            // Transpile the column values
            let values_tokens = if column.values.is_empty() {
                quote! { vec![] }
            } else {
                // Collect all values into a vector
                let value_tokens: Result<Vec<_>> = column
                    .values
                    .iter()
                    .map(|v| self.transpile_expr(v))
                    .collect();
                let value_tokens = value_tokens?;
                quote! { vec![#(#value_tokens),*] }
            };
            // Create a Series from the values using NamedFrom trait
            series_tokens.push(quote! {
                {
                    use polars::prelude::NamedFrom;
                    polars::prelude::Series::new(#col_name, #values_tokens)
                }
            });
        }
        // Create DataFrame from series
        Ok(quote! {
            {
                use polars::prelude::NamedFrom;
                polars::prelude::DataFrame::new(vec![
                    #(#series_tokens),*
                ]).expect("Failed to create DataFrame from columns")
            }
        })
    }
    /// Transpiles DataFrame operations
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Expr;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let df = Expr::literal(42.into());
    /// let operation = Expr::literal("filter".into());
    /// let result = transpiler.transpile_dataframe_operation(&df, &operation);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_dataframe_operation(
        &self,
        df: &Expr,
        op: &DataFrameOp,
    ) -> Result<TokenStream> {
        let df_tokens = self.transpile_expr(df)?;
        match op {
            DataFrameOp::Select(columns) => {
                let col_tokens: Vec<TokenStream> =
                    columns.iter().map(|col| quote! { #col }).collect();
                Ok(quote! {
                    #df_tokens.select(&[#(#col_tokens),*]).expect("Failed to select DataFrame columns")
                })
            }
            DataFrameOp::Filter(condition) => {
                let cond_tokens = self.transpile_expr(condition)?;
                Ok(quote! {
                    #df_tokens.filter(&#cond_tokens).expect("Failed to filter DataFrame")
                })
            }
            DataFrameOp::GroupBy(columns) => {
                let col_tokens: Vec<TokenStream> =
                    columns.iter().map(|col| quote! { #col }).collect();
                Ok(quote! {
                    #df_tokens.groupby(&[#(#col_tokens),*]).expect("Failed to group DataFrame")
                })
            }
            DataFrameOp::Sort(columns) => {
                // Sort by multiple columns
                let col_tokens: Vec<TokenStream> =
                    columns.iter().map(|col| quote! { #col }).collect();
                Ok(quote! {
                    #df_tokens.sort(&[#(#col_tokens),*], false)
                        .expect("DataFrame sort operation should not fail with valid columns")
                })
            }
            DataFrameOp::Join { other, on, how } => {
                let other_tokens = self.transpile_expr(other)?;
                let on_tokens: Vec<TokenStream> = on.iter().map(|col| quote! { #col }).collect();
                let join_type = match how {
                    JoinType::Left => quote! { polars::prelude::JoinType::Left },
                    JoinType::Right => quote! { polars::prelude::JoinType::Right },
                    JoinType::Inner => quote! { polars::prelude::JoinType::Inner },
                    JoinType::Outer => quote! { polars::prelude::JoinType::Outer },
                };
                Ok(quote! {
                    #df_tokens.join(
                        &#other_tokens,
                        &[#(#on_tokens),*],
                        &[#(#on_tokens),*],
                        #join_type
                    ).expect("DataFrame join operation should not fail with valid parameters")
                })
            }
            DataFrameOp::Aggregate(agg_ops) => {
                // Convert AggregateOp to expressions
                let agg_exprs: Vec<TokenStream> = agg_ops
                    .iter()
                    .map(|op| match op {
                        AggregateOp::Sum(col) => quote! { col(#col).sum() },
                        AggregateOp::Mean(col) => quote! { col(#col).mean() },
                        AggregateOp::Min(col) => quote! { col(#col).min() },
                        AggregateOp::Max(col) => quote! { col(#col).max() },
                        AggregateOp::Count(col) => quote! { col(#col).count() },
                        AggregateOp::Std(col) => quote! { col(#col).std() },
                        AggregateOp::Var(col) => quote! { col(#col).var() },
                    })
                    .collect();
                Ok(quote! {
                    #df_tokens.agg(&[#(#agg_exprs),*])
                        .expect("DataFrame aggregation should not fail with valid expressions")
                })
            }
            DataFrameOp::Limit(n) => Ok(quote! {
                #df_tokens.limit(#n)
            }),
            DataFrameOp::Head(n) => Ok(quote! {
                #df_tokens.head(Some(#n))
            }),
            DataFrameOp::Tail(n) => Ok(quote! {
                #df_tokens.tail(Some(#n))
            }),
        }
    }
    /// Transpiles DataFrame method calls (alternative to operation enum)
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Expr;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let df_expr = Expr::literal(42.into());
    /// let method = "select";
    /// let args = vec![];
    /// let result = transpiler.transpile_dataframe_method(&df_expr, method, &args);
    /// assert!(result.is_ok());
    /// ```
    pub fn transpile_dataframe_method(
        &self,
        df_expr: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let df_tokens = self.transpile_expr(df_expr)?;
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;
        // Map Ruchy DataFrame methods to correct Polars API
        match method {
            // Builder pattern methods
            "column" | "build" => self.transpile_builder_method(&df_tokens, method, &arg_tokens),
            // Inspection methods
            "rows" | "columns" | "get" => {
                self.transpile_inspection_method(&df_tokens, method, &arg_tokens)
            }
            // DataFrame operations
            "select" | "filter" | "sort" => {
                self.transpile_lazy_operation(&df_tokens, method, &arg_tokens)
            }
            "groupby" | "group_by" => self.transpile_groupby(&df_tokens, &arg_tokens),
            "agg" | "join" => self.transpile_simple_operation(&df_tokens, method, &arg_tokens),
            // Statistical methods
            "mean" | "std" | "min" | "max" | "sum" | "count" => {
                self.transpile_statistical_method(&df_tokens, method)
            }
            // Head/tail methods
            "head" | "tail" => self.transpile_head_tail(&df_tokens, method, &arg_tokens),
            // Default case
            _ => self.transpile_default_method(&df_tokens, method, &arg_tokens),
        }
    }
    fn transpile_builder_method(
        &self,
        df_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "column" => {
                if arg_tokens.len() == 2 {
                    let name = &arg_tokens[0];
                    let data = &arg_tokens[1];
                    Ok(quote! { #df_tokens.column(#name, #data) })
                } else {
                    Ok(quote! { #df_tokens.column(#(#arg_tokens),*) })
                }
            }
            "build" => Ok(quote! { #df_tokens }),
            _ => unreachable!("Invalid builder method: {}", method),
        }
    }
    fn transpile_inspection_method(
        &self,
        df_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            // DEFECT-TRANSPILER-DF-003: Map rows() → height() (returns usize)
            "rows" => Ok(quote! { #df_tokens.height() }),
            // DEFECT-TRANSPILER-DF-003: Map columns() → width() (returns usize)
            "columns" => Ok(quote! { #df_tokens.width() }),
            "get" => {
                if arg_tokens.len() == 1 {
                    let col_name = &arg_tokens[0];
                    Ok(quote! { #df_tokens.column(#col_name) })
                } else {
                    Ok(quote! { #df_tokens.get(#(#arg_tokens),*) })
                }
            }
            _ => unreachable!("Invalid inspection method: {}", method),
        }
    }
    fn transpile_lazy_operation(
        &self,
        df_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        Ok(quote! {
            #df_tokens.lazy().#method_ident(#(#arg_tokens),*).collect()
                .expect("DataFrame lazy operation collection should not fail")
        })
    }
    fn transpile_groupby(
        &self,
        df_tokens: &TokenStream,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        Ok(quote! {
            #df_tokens.group_by(#(#arg_tokens),*)
                .expect("DataFrame group_by operation should not fail with valid columns")
        })
    }
    fn transpile_simple_operation(
        &self,
        df_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        Ok(quote! {
            #df_tokens.#method_ident(#(#arg_tokens),*)
                .expect("DataFrame operation should not fail with valid parameters")
        })
    }
    fn transpile_statistical_method(
        &self,
        df_tokens: &TokenStream,
        method: &str,
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        Ok(quote! {
            #df_tokens.#method_ident()
        })
    }
    fn transpile_head_tail(
        &self,
        df_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        if arg_tokens.is_empty() {
            Ok(quote! { #df_tokens.#method_ident(Some(5)) })
        } else {
            Ok(quote! { #df_tokens.#method_ident(Some(#(#arg_tokens),*)) })
        }
    }
    fn transpile_default_method(
        &self,
        df_tokens: &TokenStream,
        method: &str,
        arg_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        Ok(quote! {
            #df_tokens.#method_ident(#(#arg_tokens),*)
        })
    }
    /// Check if an expression is a DataFrame type
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Expr;
    ///
    /// let mut transpiler = Transpiler::new();
    /// let expr = Expr::literal(42.into());
    /// let result = Transpiler::is_dataframe_expr(&expr);
    /// // Returns boolean, not Result
    /// ```
    pub fn is_dataframe_expr(expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            // Variable named "df" is likely a DataFrame
            ExprKind::Identifier(name) if name == "df" => true,
            // DataFrame constructor calls
            ExprKind::Call { func, .. } => {
                if let ExprKind::QualifiedName { module, name } = &func.kind {
                    module == "DataFrame"
                        && (name == "new"
                            || name == "from_csv"
                            || name == "from_json"
                            || name == "from_csv_string")
                } else {
                    false
                }
            }
            // Method calls that return DataFrames
            ExprKind::MethodCall {
                receiver, method, ..
            } => {
                // Check if it's a DataFrame method chain
                matches!(
                    method.as_str(),
                    "column"
                        | "build"
                        | "select"
                        | "filter"
                        | "sort"
                        | "head"
                        | "tail"
                        | "drop_nulls"
                        | "fill_null"
                ) || Self::is_dataframe_expr(receiver)
            }
            // DataFrame literals
            ExprKind::DataFrame { .. } => true,
            _ => false,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
    fn make_test_transpiler() -> Transpiler {
        Transpiler::new()
    }
    fn make_literal_expr(val: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(val, None)),
            span: Span::new(0, 10),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }
    #[test]
    fn test_empty_dataframe() {
        let transpiler = make_test_transpiler();
        let result = transpiler.transpile_dataframe(&[]).unwrap();
        let output = result.to_string();
        assert!(output.contains("DataFrame"));
        assert!(output.contains("empty"));
    }
    #[test]
    fn test_dataframe_with_columns() {
        let transpiler = make_test_transpiler();
        let columns = vec![
            DataFrameColumn {
                name: "col1".to_string(),
                values: vec![make_literal_expr(1), make_literal_expr(2)],
            },
            DataFrameColumn {
                name: "col2".to_string(),
                values: vec![make_literal_expr(3), make_literal_expr(4)],
            },
        ];
        let result = transpiler.transpile_dataframe(&columns).unwrap();
        let output = result.to_string();
        assert!(output.contains("DataFrame"));
        assert!(output.contains("Series"));
        assert!(output.contains("col1"));
        assert!(output.contains("col2"));
    }
    #[test]
    fn test_dataframe_select_operation() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0); // Placeholder
        let op = DataFrameOp::Select(vec!["col1".to_string(), "col2".to_string()]);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("select"));
        assert!(output.contains("col1"));
        assert!(output.contains("col2"));
    }
    #[test]
    fn test_dataframe_filter_operation() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let condition = make_literal_expr(1);
        let op = DataFrameOp::Filter(Box::new(condition));
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("filter"));
    }
    #[test]
    fn test_dataframe_groupby_operation() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::GroupBy(vec!["group_col".to_string()]);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("groupby"));
        assert!(output.contains("group_col"));
    }
    #[test]
    fn test_dataframe_sort_operation() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::Sort(vec!["sort_col".to_string()]);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("sort"));
        assert!(output.contains("sort_col"));
    }
    #[test]
    fn test_dataframe_join_operations() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let other_expr = make_literal_expr(1);
        let join_types = vec![
            (JoinType::Inner, "Inner"),
            (JoinType::Left, "Left"),
            (JoinType::Right, "Right"),
        ];
        for (join_type, expected) in join_types {
            let op = DataFrameOp::Join {
                other: Box::new(other_expr.clone()),
                on: vec!["id".to_string()],
                how: join_type,
            };
            let result = transpiler
                .transpile_dataframe_operation(&df_expr, &op)
                .unwrap();
            let output = result.to_string();
            assert!(output.contains("join"));
            assert!(output.contains(expected));
        }
    }
    #[test]
    fn test_dataframe_aggregate_operations() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let agg_ops = vec![
            AggregateOp::Mean("col1".to_string()),
            AggregateOp::Sum("col2".to_string()),
            AggregateOp::Min("col3".to_string()),
            AggregateOp::Max("col4".to_string()),
            AggregateOp::Count("col5".to_string()),
            AggregateOp::Std("col6".to_string()),
        ];
        let op = DataFrameOp::Aggregate(agg_ops);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        // Check that it produces some output
        assert!(!output.is_empty());
    }
    #[test]
    fn test_dataframe_limit_operations() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        // Test Limit
        let op = DataFrameOp::Limit(10);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("limit"));
        // Test Head
        let op = DataFrameOp::Head(5);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("head"));
        // Test Tail
        let op = DataFrameOp::Tail(5);
        let result = transpiler
            .transpile_dataframe_operation(&df_expr, &op)
            .unwrap();
        let output = result.to_string();
        assert!(output.contains("tail"));
    }
    #[test]
    fn test_dataframe_with_empty_column_values() {
        let transpiler = make_test_transpiler();
        let columns = vec![DataFrameColumn {
            name: "empty_col".to_string(),
            values: vec![],
        }];
        let result = transpiler.transpile_dataframe(&columns).unwrap();
        let output = result.to_string();
        assert!(output.contains("Series"));
        assert!(output.contains("empty_col"));
        assert!(output.contains("vec"));
    }

    // Test 1: transpile_dataframe_method with select
    #[test]
    fn test_transpile_dataframe_method_select() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let args = vec![
            Expr {
                kind: ExprKind::Literal(Literal::String("col1".to_string())),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
        ];
        let result = transpiler.transpile_dataframe_method(&df_expr, "select", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("lazy"));
    }

    // Test 2: transpile_dataframe_method with rows (inspection)
    #[test]
    fn test_transpile_dataframe_method_rows() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let result = transpiler.transpile_dataframe_method(&df_expr, "rows", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("height"));
    }

    // Test 3: transpile_dataframe_method with columns (inspection)
    #[test]
    fn test_transpile_dataframe_method_columns() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let result = transpiler.transpile_dataframe_method(&df_expr, "columns", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("width"));
    }

    // Test 4: transpile_dataframe_method with mean (statistical)
    #[test]
    fn test_transpile_dataframe_method_mean() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let result = transpiler.transpile_dataframe_method(&df_expr, "mean", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("mean"));
    }

    // Test 5: transpile_dataframe_method with head (no args, default 5)
    #[test]
    fn test_transpile_dataframe_method_head_no_args() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let result = transpiler.transpile_dataframe_method(&df_expr, "head", &[]);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("head"));
        assert!(output.contains("5"));
    }

    // Test 6: transpile_dataframe_method with tail (with arg)
    #[test]
    fn test_transpile_dataframe_method_tail_with_arg() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(10)];
        let result = transpiler.transpile_dataframe_method(&df_expr, "tail", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("tail"));
    }

    // Test 7: transpile_dataframe_method with groupby
    #[test]
    fn test_transpile_dataframe_method_groupby() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let args = vec![
            Expr {
                kind: ExprKind::Literal(Literal::String("group_col".to_string())),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
        ];
        let result = transpiler.transpile_dataframe_method(&df_expr, "groupby", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("group_by"));
    }

    // Test 8: transpile_dataframe_method with default method
    #[test]
    fn test_transpile_dataframe_method_default() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let result = transpiler.transpile_dataframe_method(&df_expr, "unknown_method", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string().contains("unknown_method"));
    }

    // Test 9: is_dataframe_expr with DataFrame literal
    #[test]
    fn test_is_dataframe_expr_literal() {
        let expr = Expr {
            kind: ExprKind::DataFrame {
                columns: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 10: is_dataframe_expr with identifier "df"
    #[test]
    fn test_is_dataframe_expr_identifier_df() {
        let expr = Expr {
            kind: ExprKind::Identifier("df".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 11: is_dataframe_expr with method call chain
    #[test]
    fn test_is_dataframe_expr_method_call() {
        let receiver = Box::new(Expr {
            kind: ExprKind::Identifier("df".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver,
                method: "select".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 12: is_dataframe_expr with QualifiedName DataFrame::new
    #[test]
    fn test_is_dataframe_expr_qualified_name() {
        let func = Box::new(Expr {
            kind: ExprKind::QualifiedName {
                module: "DataFrame".to_string(),
                name: "new".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::Call { func, args: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 13: Join with Outer type
    #[test]
    fn test_dataframe_join_outer() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let other_expr = make_literal_expr(1);
        let op = DataFrameOp::Join {
            other: Box::new(other_expr),
            on: vec!["id".to_string()],
            how: JoinType::Outer,
        };
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("join"));
        assert!(output.contains("Outer"));
    }

    // Test 14: Aggregate with Var operation
    #[test]
    fn test_dataframe_aggregate_var() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let agg_ops = vec![AggregateOp::Var("variance_col".to_string())];
        let op = DataFrameOp::Aggregate(agg_ops);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("var"));
        assert!(output.contains("variance_col"));
    }

    // Test 15: is_dataframe_expr with non-DataFrame identifier
    #[test]
    fn test_is_dataframe_expr_false() {
        let expr = Expr {
            kind: ExprKind::Identifier("not_df".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!Transpiler::is_dataframe_expr(&expr));
    }

    // Test 16: transpile_builder_method with "build"
    #[test]
    fn test_transpile_builder_method_build() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { df_builder };
        let result = transpiler.transpile_builder_method(&df_tokens, "build", &[]);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert_eq!(output, "df_builder");
    }

    // Test 17: transpile_builder_method with "column" and 2 args
    #[test]
    fn test_transpile_builder_method_column_two_args() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { df_builder };
        let args = vec![quote! { "name" }, quote! { vec![1, 2, 3] }];
        let result = transpiler.transpile_builder_method(&df_tokens, "column", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("column"));
    }

    // Test 18: transpile_inspection_method with "get" and multiple args
    #[test]
    fn test_transpile_inspection_method_get_multi_args() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let args = vec![quote! { "col1" }, quote! { 0 }];
        let result = transpiler.transpile_inspection_method(&df_tokens, "get", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("get"));
    }

    // Test 19: transpile_lazy_operation for select
    #[test]
    fn test_transpile_lazy_operation_select() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let args = vec![quote! { "col1" }];
        let result = transpiler.transpile_lazy_operation(&df_tokens, "select", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("lazy"));
        assert!(output.contains("select"));
        assert!(output.contains("collect"));
    }

    // Test 20: transpile_groupby generates correct tokens
    #[test]
    fn test_transpile_groupby_tokens() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { df_var };
        let args = vec![quote! { "group_col" }];
        let result = transpiler.transpile_groupby(&df_tokens, &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("group_by"));
    }

    // Test 21: transpile_simple_operation for "agg"
    #[test]
    fn test_transpile_simple_operation_agg() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let args = vec![quote! { col("x").sum() }];
        let result = transpiler.transpile_simple_operation(&df_tokens, "agg", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("agg"));
    }

    // Test 22: transpile_simple_operation for "join"
    #[test]
    fn test_transpile_simple_operation_join() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { df1 };
        let args = vec![quote! { df2 }, quote! { "id" }];
        let result = transpiler.transpile_simple_operation(&df_tokens, "join", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("join"));
    }

    // Test 23: transpile_statistical_method - mean
    #[test]
    fn test_transpile_statistical_method_mean() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let result = transpiler.transpile_statistical_method(&df_tokens, "mean");
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("mean"));
    }

    // Test 24: transpile_statistical_method - std
    #[test]
    fn test_transpile_statistical_method_std() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let result = transpiler.transpile_statistical_method(&df_tokens, "std");
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("std"));
    }

    // Test 25: transpile_head_tail with empty args (default 5)
    #[test]
    fn test_transpile_head_tail_empty_args() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let result = transpiler.transpile_head_tail(&df_tokens, "head", &[]);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("head"));
        assert!(output.contains("Some (5)"));
    }

    // Test 26: transpile_head_tail with args
    #[test]
    fn test_transpile_head_tail_with_args() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let args = vec![quote! { 10 }];
        let result = transpiler.transpile_head_tail(&df_tokens, "tail", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("tail"));
        assert!(output.contains("Some (10)"));
    }

    // Test 27: transpile_default_method for unknown method
    #[test]
    fn test_transpile_default_method_unknown() {
        use quote::quote;
        let transpiler = make_test_transpiler();
        let df_tokens = quote! { my_df };
        let args = vec![quote! { "arg1" }];
        let result = transpiler.transpile_default_method(&df_tokens, "custom_method", &args);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("custom_method"));
    }

    // Test 28: transpile_dataframe with column containing empty values
    #[test]
    fn test_transpile_dataframe_column_empty_values() {
        let transpiler = make_test_transpiler();
        let columns = vec![DataFrameColumn {
            name: "empty_col".to_string(),
            values: vec![],
        }];
        let result = transpiler.transpile_dataframe(&columns);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("empty_col"));
        assert!(output.contains("vec !"));
    }

    // Test 29: transpile_dataframe_operation - Limit
    #[test]
    fn test_transpile_dataframe_operation_limit() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::Limit(100);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("limit"));
        assert!(output.contains("100"));
    }

    // Test 30: transpile_dataframe_operation - Head
    #[test]
    fn test_transpile_dataframe_operation_head() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::Head(20);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("head"));
        assert!(output.contains("Some (20)"));
    }

    // Test 31: transpile_dataframe_operation - Tail
    #[test]
    fn test_transpile_dataframe_operation_tail() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::Tail(15);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("tail"));
        assert!(output.contains("Some (15)"));
    }

    // Test 32: is_dataframe_expr with DataFrame::from_csv
    #[test]
    fn test_is_dataframe_expr_from_csv() {
        let func = Box::new(Expr {
            kind: ExprKind::QualifiedName {
                module: "DataFrame".to_string(),
                name: "from_csv".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::Call { func, args: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 33: is_dataframe_expr with DataFrame::from_json
    #[test]
    fn test_is_dataframe_expr_from_json() {
        let func = Box::new(Expr {
            kind: ExprKind::QualifiedName {
                module: "DataFrame".to_string(),
                name: "from_json".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::Call { func, args: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 34: is_dataframe_expr with sort method
    #[test]
    fn test_is_dataframe_expr_sort_method() {
        let receiver = Box::new(Expr {
            kind: ExprKind::Identifier("data".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver,
                method: "sort".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 35: is_dataframe_expr with filter method
    #[test]
    fn test_is_dataframe_expr_filter_method() {
        let receiver = Box::new(Expr {
            kind: ExprKind::Identifier("dataset".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver,
                method: "filter".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 36: is_dataframe_expr with drop_nulls method
    #[test]
    fn test_is_dataframe_expr_drop_nulls() {
        let receiver = Box::new(Expr {
            kind: ExprKind::Identifier("data".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver,
                method: "drop_nulls".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 37: is_dataframe_expr with fill_null method
    #[test]
    fn test_is_dataframe_expr_fill_null() {
        let receiver = Box::new(Expr {
            kind: ExprKind::Identifier("df_clean".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver,
                method: "fill_null".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_dataframe_expr(&expr));
    }

    // Test 38: is_dataframe_expr with non-DataFrame method
    #[test]
    fn test_is_dataframe_expr_non_df_method() {
        let receiver = Box::new(Expr {
            kind: ExprKind::Identifier("obj".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver,
                method: "compute".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!Transpiler::is_dataframe_expr(&expr));
    }

    // Test 39: Aggregate with all AggregateOp variants
    #[test]
    fn test_dataframe_aggregate_all_variants() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let agg_ops = vec![
            AggregateOp::Sum("sum_col".to_string()),
            AggregateOp::Mean("mean_col".to_string()),
            AggregateOp::Min("min_col".to_string()),
            AggregateOp::Max("max_col".to_string()),
            AggregateOp::Count("count_col".to_string()),
            AggregateOp::Std("std_col".to_string()),
            AggregateOp::Var("var_col".to_string()),
        ];
        let op = DataFrameOp::Aggregate(agg_ops);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("sum"));
        assert!(output.contains("mean"));
        assert!(output.contains("min"));
        assert!(output.contains("max"));
        assert!(output.contains("count"));
        assert!(output.contains("std"));
        assert!(output.contains("var"));
    }

    // Test 40: Join with Left type
    #[test]
    fn test_dataframe_join_left() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let other_expr = make_literal_expr(1);
        let op = DataFrameOp::Join {
            other: Box::new(other_expr),
            on: vec!["key".to_string()],
            how: JoinType::Left,
        };
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("join"));
        assert!(output.contains("Left"));
    }

    // Test 41: Join with Right type
    #[test]
    fn test_dataframe_join_right() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let other_expr = make_literal_expr(1);
        let op = DataFrameOp::Join {
            other: Box::new(other_expr),
            on: vec!["id".to_string()],
            how: JoinType::Right,
        };
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("join"));
        assert!(output.contains("Right"));
    }

    // Test 42: transpile_dataframe_method with "rows" → height()
    #[test]
    fn test_transpile_dataframe_method_rows() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let result = transpiler.transpile_dataframe_method(&df_expr, "rows", &[]);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("height"));
    }
}
#[cfg(test)]
mod property_tests_dataframe {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    #[test]
    fn test_transpile_dataframe_never_panics() {
        // Property: transpile_dataframe never panics on any input
        let transpiler = super::Transpiler::new();

        // Test with empty columns (common edge case)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = transpiler.transpile_dataframe(&[]);
        }));
        assert!(
            result.is_ok(),
            "transpile_dataframe should not panic on empty input"
        );

        // Test with malformed column data (should handle gracefully)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let bad_columns = vec![DataFrameColumn {
                name: String::new(), // Empty name
                values: vec![],      // Empty values
            }];
            let _ = transpiler.transpile_dataframe(&bad_columns);
        }));
        assert!(
            result.is_ok(),
            "transpile_dataframe should handle malformed data gracefully"
        );
    }

    #[test]
    fn test_coverage_boost_dataframe() {
        let transpiler = Transpiler::new();

        // Test basic functionality to boost coverage
        let columns = vec![
            DataFrameColumn {
                name: "id".to_string(),
                values: vec![
                    Expr {
                        kind: ExprKind::Literal(Literal::Integer(1, None)),
                        span: Span::default(),
                        attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
                    },
                    Expr {
                        kind: ExprKind::Literal(Literal::Integer(2, None)),
                        span: Span::default(),
                        attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
                    },
                ],
            },
            DataFrameColumn {
                name: "name".to_string(),
                values: vec![
                    Expr {
                        kind: ExprKind::Literal(Literal::String("Alice".to_string())),
                        span: Span::default(),
                        attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
                    },
                    Expr {
                        kind: ExprKind::Literal(Literal::String("Bob".to_string())),
                        span: Span::default(),
                        attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
                    },
                ],
            },
        ];

        let result = transpiler.transpile_dataframe(&columns);
        assert!(result.is_ok());

        // Test empty dataframe
        let empty_columns = vec![];
        let result = transpiler.transpile_dataframe(&empty_columns);
        assert!(result.is_ok());
    }
}
