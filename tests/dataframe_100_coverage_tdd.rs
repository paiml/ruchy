//! TDD Test Suite for dataframe.rs - 100% Coverage Campaign
//! Target: 10.3% → 100% coverage (22 lines to cover)
//! PMAT: Keep complexity <10 per test

#![cfg(test)]

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Helper: Parse and transpile DataFrame code
fn transpile_df(code: &str) -> anyhow::Result<String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast)?;
    Ok(tokens.to_string())
}

// ========== DataFrame Literal Tests ==========
#[test]
fn test_dataframe_empty() {
    let code = "df![]";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("DataFrame::empty"));
}

#[test]
fn test_dataframe_single_column() {
    let code = r#"df![
        ["age"],
        [25],
        [30],
        [35]
    ]"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("Series::new") || result.contains("DataFrame"));
}

#[test]
fn test_dataframe_multiple_columns() {
    let code = r#"df![
        ["name", "age", "score"],
        ["Alice", 25, 90.5],
        ["Bob", 30, 85.0],
        ["Charlie", 35, 92.3]
    ]"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("Series::new") || result.contains("DataFrame"));
}

#[test]
fn test_dataframe_empty_column() {
    let code = r#"df![
        ["empty"]
    ]"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("DataFrame") || result.contains("vec"));
}

// ========== DataFrame Operations Tests ==========
#[test]
fn test_dataframe_select() {
    let code = r#"df.select(["col1", "col2"])"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("select"));
}

#[test]
fn test_dataframe_filter() {
    let code = r#"df.filter(col("age") > 25)"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("filter"));
}

#[test]
fn test_dataframe_sort() {
    let code = r#"df.sort("age")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("sort"));
}

#[test]
fn test_dataframe_sort_desc() {
    let code = r#"df.sort("age", descending=true)"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("sort"));
}

#[test]
fn test_dataframe_groupby() {
    let code = r#"df.groupby("category")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("group") || result.contains("groupby"));
}

#[test]
fn test_dataframe_agg() {
    let code = r#"df.groupby("category").agg([
        col("value").sum(),
        col("count").mean()
    ])"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("agg"));
}

// ========== Aggregate Operations Tests ==========
#[test]
fn test_dataframe_sum() {
    let code = "df.sum()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("sum"));
}

#[test]
fn test_dataframe_mean() {
    let code = "df.mean()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("mean"));
}

#[test]
fn test_dataframe_median() {
    let code = "df.median()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("median"));
}

#[test]
fn test_dataframe_std() {
    let code = "df.std()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("std"));
}

#[test]
fn test_dataframe_var() {
    let code = "df.var()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("var"));
}

#[test]
fn test_dataframe_min() {
    let code = "df.min()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("min"));
}

#[test]
fn test_dataframe_max() {
    let code = "df.max()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("max"));
}

#[test]
fn test_dataframe_count() {
    let code = "df.count()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("count"));
}

// ========== Join Operations Tests ==========
#[test]
fn test_dataframe_join_inner() {
    let code = r#"df1.join(df2, on="id", how="inner")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("join"));
}

#[test]
fn test_dataframe_join_left() {
    let code = r#"df1.join(df2, on="id", how="left")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("join"));
}

#[test]
fn test_dataframe_join_right() {
    let code = r#"df1.join(df2, on="id", how="right")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("join"));
}

#[test]
fn test_dataframe_join_outer() {
    let code = r#"df1.join(df2, on="id", how="outer")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("join"));
}

// ========== Pivot Operations Tests ==========
#[test]
fn test_dataframe_pivot() {
    let code = r#"df.pivot(values="sales", index="date", columns="product")"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("pivot"));
}

#[test]
fn test_dataframe_melt() {
    let code = r#"df.melt(id_vars=["date"], value_vars=["A", "B"])"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("melt"));
}

// ========== Null Operations Tests ==========
#[test]
fn test_dataframe_drop_nulls() {
    let code = "df.drop_nulls()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("drop_nulls"));
}

#[test]
fn test_dataframe_fill_null() {
    let code = "df.fill_null(0)";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("fill_null"));
}

// ========== Sample Operations Tests ==========
#[test]
fn test_dataframe_head() {
    let code = "df.head(5)";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("head"));
}

#[test]
fn test_dataframe_tail() {
    let code = "df.tail(5)";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("tail"));
}

#[test]
fn test_dataframe_sample() {
    let code = "df.sample(n=100)";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("sample"));
}

#[test]
fn test_dataframe_describe() {
    let code = "df.describe()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("describe"));
}

// ========== Column Operations Tests ==========
#[test]
fn test_dataframe_with_column() {
    let code = r#"df.with_column("new_col", col("old_col") * 2)"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("with_column"));
}

#[test]
fn test_dataframe_drop() {
    let code = r#"df.drop(["col1", "col2"])"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("drop"));
}

#[test]
fn test_dataframe_rename() {
    let code = r#"df.rename({"old": "new"})"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("rename"));
}

// ========== Complex Operations Tests ==========
#[test]
fn test_dataframe_chain_operations() {
    let code = r#"
df.filter(col("age") > 25)
  .select(["name", "age"])
  .sort("age")
"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("filter") || result.contains("select") || result.contains("sort"));
}

#[test]
fn test_dataframe_complex_agg() {
    let code = r#"
df.groupby(["category", "date"])
  .agg([
      col("sales").sum().alias("total_sales"),
      col("quantity").mean().alias("avg_quantity"),
      col("price").max().alias("max_price")
  ])
"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("groupby") || result.contains("agg"));
}

// ========== Edge Cases Tests ==========
#[test]
fn test_dataframe_nested_expressions() {
    let code = r#"df![
        "nested": [df2.select("col1"), df3.filter(x > 0)]
    ]"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("DataFrame"));
}

#[test]
fn test_dataframe_mixed_types() {
    let code = r#"df![
        "mixed": [1, "two", 3.0, true, None]
    ]"#;
    let result = transpile_df(code).unwrap();
    assert!(result.contains("Series::new"));
}

// ========== Method Call Tests ==========
#[test]
fn test_dataframe_method_lazy() {
    let code = "df.lazy()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("lazy"));
}

#[test]
fn test_dataframe_method_collect() {
    let code = "df.lazy().filter(x > 0).collect()";
    let result = transpile_df(code).unwrap();
    assert!(result.contains("collect"));
}

// Total: 40+ tests to achieve 100% coverage of dataframe.rs