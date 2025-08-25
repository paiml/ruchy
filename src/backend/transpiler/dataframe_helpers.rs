//! Helper functions for `DataFrame` operations

use proc_macro2::TokenStream;
use quote::quote;

/// Generate helper macros for `DataFrame` operations
#[allow(dead_code)]
pub fn generate_dataframe_helpers() -> TokenStream {
    quote! {
        // Helper trait for DataFrame operations
        trait DataFrameExt {
            fn filter_by<F>(&self, predicate: F) -> Self
            where
                F: Fn(&Row) -> bool;

            fn select_columns(&self, columns: &[&str]) -> Self;

            fn sort_by(&self, columns: &[&str], descending: bool) -> Self;

            fn group_and_agg(&self, group_cols: &[&str], agg_ops: &[AggOp]) -> Self;

            fn head_n(&self, n: usize) -> Self;

            fn tail_n(&self, n: usize) -> Self;

            fn join_with(&self, other: &Self, on: &[&str], how: JoinType) -> Self;
        }

        // Aggregation operation enum
        enum AggOp {
            Sum(String),
            Mean(String),
            Min(String),
            Max(String),
            Count(String),
            Std(String),
            Var(String),
        }

        // Join types
        enum JoinType {
            Inner,
            Left,
            Right,
            Outer,
        }

        // Row type for filtering
        struct Row {
            data: HashMap<String, Value>,
        }

        impl Row {
            fn get<T>(&self, column: &str) -> Option<&T> {
                self.data.get(column).and_then(|v| v.downcast_ref())
            }
        }
    }
}

/// Generate `DataFrame` builder pattern helpers
#[allow(dead_code)]
pub fn generate_dataframe_builder() -> TokenStream {
    quote! {
        // Builder pattern for DataFrames
        struct DataFrameBuilder {
            columns: Vec<(String, Vec<Value>)>,
        }

        impl DataFrameBuilder {
            fn new() -> Self {
                Self { columns: Vec::new() }
            }

            fn column<T>(mut self, name: impl Into<String>, values: Vec<T>) -> Self
            where
                T: Into<Value>
            {
                let values: Vec<Value> = values.into_iter().map(Into::into).collect();
                self.columns.push((name.into(), values));
                self
            }

            fn build(self) -> DataFrame {
                DataFrame::from_columns(self.columns)
            }
        }

        // Convenience macro for DataFrame creation
        macro_rules! dataframe {
            ($($col:ident : $values:expr),* $(,)?) => {{
                DataFrameBuilder::new()
                    $(.column(stringify!($col), $values))*
                    .build()
            }};
        }
    }
}

/// Generate pipeline operation helpers
#[allow(dead_code)]
pub fn generate_pipeline_helpers() -> TokenStream {
    quote! {
        // Pipeline trait for fluent API
        trait Pipeline<T> {
            fn pipe<F, R>(self, f: F) -> R
            where
                F: FnOnce(T) -> R;

            fn pipe_ref<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&T) -> R;

            fn pipe_mut<F, R>(&mut self, f: F) -> R
            where
                F: FnOnce(&mut T) -> R;
        }

        impl<T> Pipeline<T> for T {
            fn pipe<F, R>(self, f: F) -> R
            where
                F: FnOnce(T) -> R
            {
                f(self)
            }

            fn pipe_ref<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&T) -> R
            {
                f(self)
            }

            fn pipe_mut<F, R>(&mut self, f: F) -> R
            where
                F: FnOnce(&mut T) -> R
            {
                f(self)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dataframe_helpers() {
        let helpers = generate_dataframe_helpers();
        let code = helpers.to_string();
        
        // Check trait definition
        assert!(code.contains("trait DataFrameExt"));
        assert!(code.contains("fn filter_by"));
        assert!(code.contains("fn select_columns"));
        assert!(code.contains("fn sort_by"));
        assert!(code.contains("fn group_and_agg"));
        assert!(code.contains("fn head_n"));
        assert!(code.contains("fn tail_n"));
        assert!(code.contains("fn join_with"));
        
        // Check enum types
        assert!(code.contains("enum AggOp"));
        assert!(code.contains("Sum(String)"));
        assert!(code.contains("Mean(String)"));
        assert!(code.contains("enum JoinType"));
        assert!(code.contains("Inner"));
        assert!(code.contains("Left"));
        
        // Check Row struct
        assert!(code.contains("struct Row"));
        assert!(code.contains("HashMap<String, Value>"));
    }

    #[test]
    fn test_generate_dataframe_builder() {
        let builder = generate_dataframe_builder();
        let code = builder.to_string();
        
        // Check builder struct
        assert!(code.contains("struct DataFrameBuilder"));
        assert!(code.contains("columns: Vec<(String, Vec<Value>)>"));
        
        // Check builder methods
        assert!(code.contains("fn new()"));
        assert!(code.contains("fn column"));
        assert!(code.contains("fn build"));
        
        // Check macro
        assert!(code.contains("macro_rules! dataframe"));
        assert!(code.contains("DataFrameBuilder::new()"));
    }

    #[test]
    fn test_generate_pipeline_helpers() {
        let pipeline = generate_pipeline_helpers();
        let code = pipeline.to_string();
        
        // Check trait definition
        assert!(code.contains("trait Pipeline"));
        assert!(code.contains("fn pipe"));
        assert!(code.contains("fn pipe_ref"));
        assert!(code.contains("fn pipe_mut"));
        
        // Check implementation
        assert!(code.contains("impl<T> Pipeline<T> for T"));
        assert!(code.contains("FnOnce(T) -> R"));
        assert!(code.contains("FnOnce(&T) -> R"));
        assert!(code.contains("FnOnce(&mut T) -> R"));
    }
}