#![allow(missing_docs)]
//! DF-002: `DataFrame` `filter()` Property Tests (EXTREME TDD)
//!
//! **CRITICAL**: Property tests with 10,000+ iterations per invariant to prove
//! `filter()` correctness across all possible inputs.
//!
//! **Toyota Way**: Jidoka - Stop the line if ANY property violation found.

use proptest::prelude::*;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
use ruchy::runtime::eval_dataframe_ops::eval_dataframe_filter;
use ruchy::runtime::{DataFrameColumn, Value};

/// Generate arbitrary `DataFrameColumn` for property testing
#[allow(dead_code)]
fn arb_dataframe_column(name: String, size: usize) -> BoxedStrategy<DataFrameColumn> {
    prop::collection::vec(
        prop_oneof![
            any::<i64>().prop_map(Value::Integer),
            any::<f64>()
                .prop_filter("not NaN", |f| !f.is_nan())
                .prop_map(Value::Float),
            any::<bool>().prop_map(Value::Bool),
        ],
        size..=size,
    )
    .prop_map(move |values| DataFrameColumn {
        name: name.clone(),
        values,
    })
    .boxed()
}

/// Generate `DataFrame` with random columns and rows
fn arb_dataframe(num_cols: usize, num_rows: usize) -> BoxedStrategy<Vec<DataFrameColumn>> {
    prop::collection::vec(
        prop::collection::vec(
            prop_oneof![
                any::<i64>().prop_map(Value::Integer),
                any::<f64>()
                    .prop_filter("not NaN", |f| !f.is_nan())
                    .prop_map(Value::Float),
                any::<bool>().prop_map(Value::Bool),
            ],
            num_rows..=num_rows,
        ),
        num_cols..=num_cols,
    )
    .prop_map(|columns_data| {
        columns_data
            .into_iter()
            .enumerate()
            .map(|(i, values)| DataFrameColumn {
                name: format!("col_{i}"),
                values,
            })
            .collect()
    })
    .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    /// Property 1: Filter NEVER increases row count
    /// Invariant: filtered_rows <= original_rows
    #[test]
    fn prop_filter_never_increases_rows(
        columns in arb_dataframe(3, 20)
    ) {
        let original_row_count = if columns.is_empty() {
            0
        } else {
            columns[0].values.len()
        };

        // Always-true condition (keeps all rows as upper bound)
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(true));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            let filtered_row_count = if result_cols.is_empty() {
                0
            } else {
                result_cols[0].values.len()
            };
            prop_assert!(
                filtered_row_count <= original_row_count,
                "Filter increased row count: {} -> {}",
                original_row_count,
                filtered_row_count
            );
        }
    }

    /// Property 2: Filter with always-false predicate returns empty DataFrame
    /// Invariant: filter(false) -> 0 rows
    #[test]
    fn prop_filter_false_returns_empty(
        columns in arb_dataframe(2, 15)
    ) {
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0)
        );
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(false));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            for col in result_cols {
                prop_assert_eq!(col.values.len(), 0, "Filter(false) should return 0 rows");
            }
        }
    }

    /// Property 3: Filter with always-true predicate returns original DataFrame
    /// Invariant: filter(true) -> all rows
    #[test]
    fn prop_filter_true_preserves_all_rows(
        columns in arb_dataframe(2, 15)
    ) {
        let original_row_count = if columns.is_empty() {
            0
        } else {
            columns[0].values.len()
        };

        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(true));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            if !result_cols.is_empty() {
                prop_assert_eq!(
                    result_cols[0].values.len(),
                    original_row_count,
                    "Filter(true) should preserve all rows"
                );
            }
        }
    }

    /// Property 4: Filter preserves schema (column names and order)
    /// Invariant: filtered_columns.names == original_columns.names
    #[test]
    fn prop_filter_preserves_schema(
        columns in arb_dataframe(4, 10)
    ) {
        let original_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(true));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            let filtered_names: Vec<String> = result_cols.iter().map(|c| c.name.clone()).collect();
            prop_assert_eq!(
                filtered_names,
                original_names,
                "Filter must preserve column names and order"
            );
        }
    }

    /// Property 5: Filter is idempotent with constant predicate
    /// Invariant: filter(filter(df, p), p) == filter(df, p)
    #[test]
    fn prop_filter_idempotent_constant(
        columns in arb_dataframe(2, 10)
    ) {
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(true));

        // First filter
        let result1 = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result1.is_ok());

        if let Ok(Value::DataFrame { columns: filtered_once }) = result1 {
            // Second filter on already filtered data
            let result2 = eval_dataframe_filter(&filtered_once, &condition, eval_fn);
            prop_assert!(result2.is_ok());

            if let Ok(Value::DataFrame { columns: filtered_twice }) = result2 {
                prop_assert_eq!(
                    filtered_once[0].values.len(),
                    filtered_twice[0].values.len(),
                    "Filter should be idempotent with constant predicate"
                );
            }
        }
    }

    /// Property 6: Filter with alternating predicate
    /// Test that filter correctly handles row-specific conditions
    #[test]
    fn prop_filter_alternating_predicate(
        columns in arb_dataframe(2, 20)
    ) {
        let original_row_count = if columns.is_empty() {
            0
        } else {
            columns[0].values.len()
        };

        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        // Keep only even rows (0, 2, 4, ...)
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], row_idx: usize| {
            Ok(Value::Bool(row_idx.is_multiple_of(2)))
        };

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            if !result_cols.is_empty() {
                let filtered_count = result_cols[0].values.len();
                let expected_count = original_row_count.div_ceil(2); // Ceiling division
                prop_assert_eq!(
                    filtered_count,
                    expected_count,
                    "Alternating filter should keep half the rows"
                );
            }
        }
    }

    /// Property 7: Empty DataFrame filter always returns empty DataFrame
    /// Invariant: filter(empty_df) -> empty_df
    #[test]
    fn prop_filter_empty_dataframe(_x in 0..1000usize) {
        let columns: Vec<DataFrameColumn> = vec![];
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(true));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            prop_assert_eq!(result_cols.len(), 0, "Empty DataFrame should stay empty");
        }
    }

    /// Property 8: Non-boolean condition always returns error
    /// Invariant: filter(non_bool) -> Err
    #[test]
    fn prop_filter_non_boolean_fails(
        columns in arb_dataframe(1, 5),
        invalid_value in prop_oneof![
            any::<i64>().prop_map(Value::Integer),
            any::<f64>().prop_filter("not NaN", |f| !f.is_nan()).prop_map(Value::Float),
        ]
    ) {
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let captured_value = invalid_value;
        let eval_fn = move |_: &Expr, _: &[DataFrameColumn], _: usize| {
            Ok(captured_value.clone())
        };

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(
            result.is_err(),
            "Filter with non-boolean condition must fail"
        );

        if let Err(e) = result {
            prop_assert!(
                e.to_string().contains("must evaluate to boolean"),
                "Error message should mention boolean requirement"
            );
        }
    }

    /// Property 9: Filter preserves row integrity
    /// Invariant: All values in same row index stay together
    #[test]
    fn prop_filter_preserves_row_integrity(
        num_rows in 1..20usize
    ) {
        // Create DataFrame with identifiable pattern: col_0 = row_index
        let columns = vec![
            DataFrameColumn {
                name: "row_id".to_string(),
                values: (0..num_rows).map(|i| Value::Integer(i.try_into().expect("test values fit in i64"))).collect(),
            },
            DataFrameColumn {
                name: "data".to_string(),
                values: (0..num_rows).map(|i| Value::Integer((i * 10).try_into().expect("test values fit in i64"))).collect(),
            },
        ];

        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        // Keep only rows where row_id is even
        let eval_fn = |_: &Expr, cols: &[DataFrameColumn], row_idx: usize| {
            if let Value::Integer(id) = cols[0].values[row_idx] {
                Ok(Value::Bool(id % 2 == 0))
            } else {
                Ok(Value::Bool(false))
            }
        };

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            // Verify row integrity: data should be row_id * 10
            for (idx, row_id_val) in result_cols[0].values.iter().enumerate() {
                if let Value::Integer(row_id) = row_id_val {
                    if let Value::Integer(data) = result_cols[1].values[idx] {
                        prop_assert_eq!(
                            data,
                            row_id * 10,
                            "Row integrity violated: row_id={}, data={}",
                            row_id,
                            data
                        );
                    }
                }
            }
        }
    }

    /// Property 10: Filter count invariant
    /// Invariant: count(filtered_rows) = count(rows where predicate=true)
    #[test]
    fn prop_filter_count_matches_predicate(
        num_rows in 1..50usize,
        threshold in 0..50i64
    ) {
        let columns = vec![
            DataFrameColumn {
                name: "value".to_string(),
                values: (0..num_rows).map(|i| Value::Integer(i.try_into().expect("test values fit in i64"))).collect(),
            },
        ];

        let expected_count = (0..num_rows).filter(|&i| i.try_into().map(|v: i64| v > threshold).unwrap_or(false)).count();

        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0)
        );
        let eval_fn = move |_: &Expr, cols: &[DataFrameColumn], row_idx: usize| {
            if let Value::Integer(val) = cols[0].values[row_idx] {
                Ok(Value::Bool(val > threshold))
            } else {
                Ok(Value::Bool(false))
            }
        };

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        prop_assert!(result.is_ok());

        if let Ok(Value::DataFrame { columns: result_cols }) = result {
            let actual_count = if result_cols.is_empty() {
                0
            } else {
                result_cols[0].values.len()
            };
            prop_assert_eq!(
                actual_count,
                expected_count,
                "Filtered count must match predicate truth count"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// DF-002: Baseline test - filter with simple true condition
    #[test]
    fn test_df002_filter_basic_true() {
        let columns = vec![DataFrameColumn {
            name: "age".to_string(),
            values: vec![Value::Integer(25), Value::Integer(30), Value::Integer(35)],
        }];

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(true));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        assert!(result.is_ok());

        if let Ok(Value::DataFrame {
            columns: result_cols,
        }) = result
        {
            assert_eq!(result_cols[0].values.len(), 3, "All rows should be kept");
        } else {
            panic!("Expected DataFrame result");
        }
    }

    /// DF-002: Test filter with false condition
    #[test]
    fn test_df002_filter_basic_false() {
        let columns = vec![DataFrameColumn {
            name: "age".to_string(),
            values: vec![Value::Integer(25), Value::Integer(30)],
        }];

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::new(0, 0));
        let eval_fn = |_: &Expr, _: &[DataFrameColumn], _: usize| Ok(Value::Bool(false));

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        assert!(result.is_ok());

        if let Ok(Value::DataFrame {
            columns: result_cols,
        }) = result
        {
            assert_eq!(result_cols[0].values.len(), 0, "No rows should be kept");
        } else {
            panic!("Expected DataFrame result");
        }
    }

    /// DF-002: Test filter with row-specific predicate (age > 25)
    #[test]
    fn test_df002_filter_row_specific_predicate() {
        let columns = vec![DataFrameColumn {
            name: "age".to_string(),
            values: vec![Value::Integer(20), Value::Integer(30), Value::Integer(40)],
        }];

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
        let eval_fn = |_: &Expr, cols: &[DataFrameColumn], row_idx: usize| {
            if let Value::Integer(age) = cols[0].values[row_idx] {
                Ok(Value::Bool(age > 25))
            } else {
                Ok(Value::Bool(false))
            }
        };

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        assert!(result.is_ok());

        if let Ok(Value::DataFrame {
            columns: result_cols,
        }) = result
        {
            assert_eq!(
                result_cols[0].values.len(),
                2,
                "Should keep 2 rows (age > 25)"
            );
            assert_eq!(result_cols[0].values[0], Value::Integer(30));
            assert_eq!(result_cols[0].values[1], Value::Integer(40));
        } else {
            panic!("Expected DataFrame result");
        }
    }

    /// DF-002: Test filter preserves column integrity across multiple columns
    #[test]
    fn test_df002_filter_multi_column_integrity() {
        let columns = vec![
            DataFrameColumn {
                name: "id".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
            },
            DataFrameColumn {
                name: "name".to_string(),
                values: vec![
                    Value::from_string("Alice".to_string()),
                    Value::from_string("Bob".to_string()),
                    Value::from_string("Charlie".to_string()),
                ],
            },
        ];

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
        // Keep only even IDs
        let eval_fn = |_: &Expr, cols: &[DataFrameColumn], row_idx: usize| {
            if let Value::Integer(id) = cols[0].values[row_idx] {
                Ok(Value::Bool(id % 2 == 0))
            } else {
                Ok(Value::Bool(false))
            }
        };

        let result = eval_dataframe_filter(&columns, &condition, eval_fn);
        assert!(result.is_ok());

        if let Ok(Value::DataFrame {
            columns: result_cols,
        }) = result
        {
            assert_eq!(result_cols.len(), 2, "Should have 2 columns");
            assert_eq!(result_cols[0].values.len(), 1, "Should keep 1 row (id=2)");
            assert_eq!(result_cols[0].values[0], Value::Integer(2));
            assert_eq!(
                result_cols[1].values[0],
                Value::from_string("Bob".to_string())
            );
        } else {
            panic!("Expected DataFrame result");
        }
    }
}
