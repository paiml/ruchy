// EXTREME TDD: Tests for DataFrame as First-Class Type from unified spec
// ALL TESTS MUST FAIL INITIALLY - Implementation comes AFTER

use ruchy::compile;

#[cfg(test)]
mod test_dataframe {
    use super::*;

    // Basic DataFrame creation tests
    #[test]
    fn test_dataframe_from_csv() {
        let code = r#"
            fun load_data() -> DataFrame {
                DataFrame::from_csv("data.csv")?
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame from CSV");
        let output = result.unwrap();
        assert!(output.contains("DataFrame::from_csv"));
    }

    #[test]
    fn test_dataframe_literal() {
        let code = r#"
            fun create_df() -> DataFrame {
                df![
                    "name" => ["Alice", "Bob", "Charlie"],
                    "age" => [25, 30, 35],
                    "score" => [95.5, 87.3, 92.1],
                ]
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame literal");
    }

    // DataFrame operations tests
    #[test]
    fn test_dataframe_filter() {
        let code = r#"
            fun filter_adults(df: DataFrame) -> DataFrame {
                df.filter(col("age") > 18)
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame filter");
        let output = result.unwrap();
        assert!(output.contains("df.filter"));
        assert!(output.contains("col(\"age\") > 18"));
    }

    #[test]
    fn test_dataframe_select() {
        let code = r#"
            fun select_columns(df: DataFrame) -> DataFrame {
                df.select(["name", "age"])
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame select");
    }

    #[test]
    fn test_dataframe_groupby() {
        let code = r#"
            fun group_by_category(df: DataFrame) -> DataFrame {
                df.groupby("category")
                  .agg([
                      mean("value").alias("avg"),
                      std("value").alias("stddev"),
                      count().alias("n"),
                  ])
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame groupby");
        let output = result.unwrap();
        assert!(output.contains("groupby"));
        assert!(output.contains("agg"));
    }

    // Method chaining tests
    #[test]
    fn test_dataframe_chain() {
        let code = r#"
            fun analyze_data(df: DataFrame) -> DataFrame {
                df.filter(col("age") > 18)
                  .groupby("department")
                  .agg([mean("salary").alias("avg_salary")])
                  .sort("avg_salary", descending=true)
                  .limit(10)
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame method chain");
    }

    // SQL macro tests
    #[test]
    fn test_sql_macro_simple() {
        let code = r#"
            fun query_data(df: DataFrame) -> DataFrame {
                sql! {
                    SELECT name, age
                    FROM {df}
                    WHERE age > 18
                }
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile SQL macro");
        let output = result.unwrap();
        assert!(output.contains("sql!"));
    }

    #[test]
    fn test_sql_macro_with_aggregation() {
        let code = r#"
            fun aggregate_data(df: DataFrame) -> DataFrame {
                sql! {
                    SELECT category, AVG(value) as avg_value
                    FROM {df}
                    WHERE status = 'active'
                    GROUP BY category
                    ORDER BY avg_value DESC
                }
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile SQL macro with aggregation"
        );
    }

    #[test]
    fn test_sql_macro_with_join() {
        let code = r#"
            fun join_tables(df1: DataFrame, df2: DataFrame) -> DataFrame {
                sql! {
                    SELECT a.*, b.score
                    FROM {df1} a
                    JOIN {df2} b ON a.id = b.id
                    WHERE b.score > 80
                }
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile SQL macro with join");
    }

    // DataFrame column operations
    #[test]
    fn test_dataframe_add_column() {
        let code = r#"
            fun add_computed_column(df: DataFrame) -> DataFrame {
                df.with_column(
                    col("price") * col("quantity"),
                    "total"
                )
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame add column");
    }

    #[test]
    fn test_dataframe_drop_columns() {
        let code = r#"
            fun drop_unnecessary(df: DataFrame) -> DataFrame {
                df.drop(["temp1", "temp2", "debug"])
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame drop columns");
    }

    #[test]
    fn test_dataframe_rename() {
        let code = r#"
            fun rename_columns(df: DataFrame) -> DataFrame {
                df.rename({
                    "old_name": "new_name",
                    "col1": "feature1",
                })
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame rename");
    }

    // DataFrame statistics
    #[test]
    fn test_dataframe_describe() {
        let code = r#"
            fun get_statistics(df: DataFrame) -> DataFrame {
                df.describe()
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame describe");
    }

    #[test]
    fn test_dataframe_correlation() {
        let code = r#"
            fun correlation_matrix(df: DataFrame) -> DataFrame {
                df.select_numeric().corr()
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame correlation");
    }

    // DataFrame I/O
    #[test]
    fn test_dataframe_to_csv() {
        let code = r#"
            fun save_results(df: DataFrame, path: &str) -> Result<()> {
                df.to_csv(path)?;
                Ok(())
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame to CSV");
    }

    #[test]
    fn test_dataframe_to_parquet() {
        let code = r#"
            fun save_compressed(df: DataFrame, path: &str) -> Result<()> {
                df.to_parquet(path)?;
                Ok(())
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame to Parquet");
    }

    #[test]
    fn test_dataframe_from_json() {
        let code = r#"
            fun load_json_data(path: &str) -> Result<DataFrame> {
                DataFrame::from_json(path)
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame from JSON");
    }

    // DataFrame transformations
    #[test]
    fn test_dataframe_pivot() {
        let code = r#"
            fun pivot_data(df: DataFrame) -> DataFrame {
                df.pivot("date", "category", "value", Agg::Sum)
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame pivot");
    }

    #[test]
    fn test_dataframe_melt() {
        let code = r#"
            fun unpivot_data(df: DataFrame) -> DataFrame {
                df.melt(["id", "date"], ["var1", "var2", "var3"])
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame melt");
    }

    // DataFrame joins
    #[test]
    fn test_dataframe_inner_join() {
        let code = r#"
            fun join_datasets(df1: DataFrame, df2: DataFrame) -> DataFrame {
                df1.join(df2, on="id", how="inner")
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame inner join");
    }

    #[test]
    fn test_dataframe_left_join() {
        let code = r#"
            fun left_merge(df1: DataFrame, df2: DataFrame) -> DataFrame {
                df1.join(df2, on=["key1", "key2"], how="left")
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame left join");
    }

    // Window functions
    #[test]
    fn test_dataframe_window() {
        let code = r#"
            fun rolling_average(df: DataFrame) -> DataFrame {
                df.with_column(
                    col("value").rolling_mean(window_size=7),
                    "rolling_avg"
                )
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame rolling window");
    }

    #[test]
    fn test_dataframe_rank() {
        let code = r#"
            fun add_rank(df: DataFrame) -> DataFrame {
                df.with_column(
                    col("score").rank("dense", descending=true),
                    "rank"
                )
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame rank");
    }

    // DataFrame lazy evaluation
    #[test]
    fn test_dataframe_lazy() {
        let code = r#"
            fun lazy_processing(df: DataFrame) -> DataFrame {
                df.lazy()
                  .filter(col("value") > 0)
                  .select([col("id"), col("value") * 2])
                  .collect()
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile lazy DataFrame");
    }

    // DataFrame with expressions
    #[test]
    fn test_dataframe_when_then() {
        let code = r#"
            fun categorize(df: DataFrame) -> DataFrame {
                df.with_column(
                    when(col("age") < 18).then("minor")
                    .when(col("age") < 65).then("adult")
                    .otherwise("senior"),
                    "category"
                )
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame when/then");
    }

    // DataFrame null handling
    #[test]
    fn test_dataframe_fill_null() {
        let code = r#"
            fun handle_nulls(df: DataFrame) -> DataFrame {
                df.fill_null(0)
                  .drop_nulls(["critical_column"])
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame null handling");
    }

    // DataFrame sampling
    #[test]
    fn test_dataframe_sample() {
        let code = r#"
            fun sample_data(df: DataFrame) -> DataFrame {
                df.sample(n=1000, seed=42)
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile DataFrame sample");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{quickcheck, TestResult};

        fn prop_dataframe_operations(n: u8) -> TestResult {
            if n == 0 {
                return TestResult::discard();
            }

            let code = format!("fun test(df: DataFrame) -> DataFrame {{ df.limit({}) }}", n);
            let result = compile(&code);
            TestResult::from_bool(result.is_ok() || result.is_err())
        }

        quickcheck! {
            fn test_dataframe_limit_values(n: u8) -> TestResult {
                prop_dataframe_operations(n)
            }
        }
    }
}
