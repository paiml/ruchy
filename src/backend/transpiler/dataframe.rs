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

            // Create a Series from the values
            series_tokens.push(quote! {
                polars::prelude::Series::new(#col_name, #values_tokens)
            });
        }

        // Create DataFrame from series
        Ok(quote! {
            polars::prelude::DataFrame::new(vec![
                #(#series_tokens),*
            ]).unwrap()
        })
    }

    /// Transpiles DataFrame operations
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
                    #df_tokens.select(&[#(#col_tokens),*]).unwrap()
                })
            }
            DataFrameOp::Filter(condition) => {
                let cond_tokens = self.transpile_expr(condition)?;
                Ok(quote! {
                    #df_tokens.filter(&#cond_tokens).unwrap()
                })
            }
            DataFrameOp::GroupBy(columns) => {
                let col_tokens: Vec<TokenStream> =
                    columns.iter().map(|col| quote! { #col }).collect();
                Ok(quote! {
                    #df_tokens.groupby(&[#(#col_tokens),*]).unwrap()
                })
            }
            DataFrameOp::Sort(columns) => {
                // Sort by multiple columns
                let col_tokens: Vec<TokenStream> =
                    columns.iter().map(|col| quote! { #col }).collect();
                Ok(quote! {
                    #df_tokens.sort(&[#(#col_tokens),*], false).unwrap()
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
                    ).unwrap()
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
                    #df_tokens.agg(&[#(#agg_exprs),*]).unwrap()
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
    pub fn transpile_dataframe_method(
        &self,
        df_expr: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let df_tokens = self.transpile_expr(df_expr)?;
        let method_ident = format_ident!("{}", method);

        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;

        // Map Ruchy DataFrame methods to Polars methods
        match method {
            "select" | "filter" | "groupby" | "agg" | "sort" | "join" => Ok(quote! {
                #df_tokens.#method_ident(#(#arg_tokens),*).unwrap()
            }),
            "mean" | "std" | "min" | "max" | "sum" | "count" => {
                // These are aggregate functions
                Ok(quote! {
                    #df_tokens.#method_ident()
                })
            }
            "head" | "tail" => {
                if args.is_empty() {
                    Ok(quote! { #df_tokens.#method_ident(Some(5)) })
                } else {
                    Ok(quote! { #df_tokens.#method_ident(Some(#(#arg_tokens),*)) })
                }
            }
            _ => {
                // Default method call
                Ok(quote! {
                    #df_tokens.#method_ident(#(#arg_tokens),*)
                })
            }
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
            kind: ExprKind::Literal(Literal::Integer(val)),
            span: Span::new(0, 10),
            attributes: vec![],
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
        
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
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
        
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("filter"));
    }
    
    #[test]
    fn test_dataframe_groupby_operation() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::GroupBy(vec!["group_col".to_string()]);
        
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("groupby"));
        assert!(output.contains("group_col"));
    }
    
    #[test]
    fn test_dataframe_sort_operation() {
        let transpiler = make_test_transpiler();
        let df_expr = make_literal_expr(0);
        let op = DataFrameOp::Sort(vec!["sort_col".to_string()]);
        
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
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
            
            let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
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
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
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
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("limit"));
        
        // Test Head
        let op = DataFrameOp::Head(5);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("head"));
        
        // Test Tail
        let op = DataFrameOp::Tail(5);
        let result = transpiler.transpile_dataframe_operation(&df_expr, &op).unwrap();
        let output = result.to_string();
        assert!(output.contains("tail"));
    }
    
    #[test]
    fn test_dataframe_with_empty_column_values() {
        let transpiler = make_test_transpiler();
        let columns = vec![
            DataFrameColumn {
                name: "empty_col".to_string(),
                values: vec![],
            },
        ];
        
        let result = transpiler.transpile_dataframe(&columns).unwrap();
        let output = result.to_string();
        assert!(output.contains("Series"));
        assert!(output.contains("empty_col"));
        assert!(output.contains("vec"));
    }
}
