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
    pub fn transpile_dataframe_operation(&self, df: &Expr, op: &DataFrameOp) -> Result<TokenStream> {
        let df_tokens = self.transpile_expr(df)?;
        
        match op {
            DataFrameOp::Select(columns) => {
                let col_tokens: Vec<TokenStream> = columns
                    .iter()
                    .map(|col| quote! { #col })
                    .collect();
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
                let col_tokens: Vec<TokenStream> = columns
                    .iter()
                    .map(|col| quote! { #col })
                    .collect();
                Ok(quote! {
                    #df_tokens.groupby(&[#(#col_tokens),*]).unwrap()
                })
            }
            DataFrameOp::Sort(columns) => {
                // Sort by multiple columns
                let col_tokens: Vec<TokenStream> = columns
                    .iter()
                    .map(|col| quote! { #col })
                    .collect();
                Ok(quote! {
                    #df_tokens.sort(&[#(#col_tokens),*], false).unwrap()
                })
            }
            DataFrameOp::Join { other, on, how } => {
                let other_tokens = self.transpile_expr(other)?;
                let on_tokens: Vec<TokenStream> = on
                    .iter()
                    .map(|col| quote! { #col })
                    .collect();
                    
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
            DataFrameOp::Limit(n) => {
                Ok(quote! {
                    #df_tokens.limit(#n)
                })
            }
            DataFrameOp::Head(n) => {
                Ok(quote! {
                    #df_tokens.head(Some(#n))
                })
            }
            DataFrameOp::Tail(n) => {
                Ok(quote! {
                    #df_tokens.tail(Some(#n))
                })
            }
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
            "select" | "filter" | "groupby" | "agg" | "sort" | "join" => {
                Ok(quote! {
                    #df_tokens.#method_ident(#(#arg_tokens),*).unwrap()
                })
            }
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