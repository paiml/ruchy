//! Comprehensive TDD test suite for dataframe.rs transpiler module
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every dataframe operation must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

// ==================== DATAFRAME LITERAL TESTS ====================

#[test]
fn test_transpile_empty_dataframe() {
    let transpiler = Transpiler::new();
    let code = "df![]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("DataFrame::empty"));
}

#[test]
fn test_transpile_single_column_dataframe() {
    let transpiler = Transpiler::new();
    let code = r#"df![
        "name" => ["Alice", "Bob", "Charlie"]
    ]"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Series::new"));
    assert!(transpiled.contains("DataFrame::new"));
}

#[test]
fn test_transpile_multi_column_dataframe() {
    let transpiler = Transpiler::new();
    let code = r#"df![
        "name" => ["Alice", "Bob"],
        "age" => [30, 25]
    ]"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Series::new"));
    assert!(transpiled.contains(r#""name""#));
    assert!(transpiled.contains(r#""age""#));
}

#[test]
fn test_transpile_dataframe_with_mixed_types() {
    let transpiler = Transpiler::new();
    let code = r#"df![
        "id" => [1, 2, 3],
        "value" => [1.5, 2.5, 3.5],
        "active" => [true, false, true]
    ]"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("DataFrame::new"));
    assert!(transpiled.contains("vec!"));
}

// ==================== DATAFRAME OPERATION TESTS ====================

#[test]
fn test_transpile_dataframe_select() {
    let transpiler = Transpiler::new();
    let code = r#"df.select(["name", "age"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("select"));
}

#[test]
fn test_transpile_dataframe_filter() {
    let transpiler = Transpiler::new();
    let code = r#"df.filter(age > 25)"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("filter"));
}

#[test]
fn test_transpile_dataframe_groupby() {
    let transpiler = Transpiler::new();
    let code = r#"df.groupby(["category"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("groupby"));
}

#[test]
fn test_transpile_dataframe_sort() {
    let transpiler = Transpiler::new();
    let code = r#"df.sort(["age"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("sort"));
}

#[test]
fn test_transpile_dataframe_sort_multiple() {
    let transpiler = Transpiler::new();
    let code = r#"df.sort(["category", "age"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("sort"));
}

// ==================== DATAFRAME JOIN TESTS ====================

#[test]
fn test_transpile_dataframe_inner_join() {
    let transpiler = Transpiler::new();
    let code = r#"df1.join(df2, on=["id"], how="inner")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("join"));
}

#[test]
fn test_transpile_dataframe_left_join() {
    let transpiler = Transpiler::new();
    let code = r#"df1.join(df2, on=["id"], how="left")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("join"));
}

#[test]
fn test_transpile_dataframe_right_join() {
    let transpiler = Transpiler::new();
    let code = r#"df1.join(df2, on=["id"], how="right")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("join"));
}

#[test]
fn test_transpile_dataframe_outer_join() {
    let transpiler = Transpiler::new();
    let code = r#"df1.join(df2, on=["id"], how="outer")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("join"));
}

// ==================== DATAFRAME AGGREGATION TESTS ====================

#[test]
fn test_transpile_dataframe_agg_sum() {
    let transpiler = Transpiler::new();
    let code = r#"df.agg(sum("value"))"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("sum"));
}

#[test]
fn test_transpile_dataframe_agg_mean() {
    let transpiler = Transpiler::new();
    let code = r#"df.agg(mean("value"))"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("mean"));
}

#[test]
fn test_transpile_dataframe_agg_max() {
    let transpiler = Transpiler::new();
    let code = r#"df.agg(max("value"))"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("max"));
}

#[test]
fn test_transpile_dataframe_agg_min() {
    let transpiler = Transpiler::new();
    let code = r#"df.agg(min("value"))"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("min"));
}

#[test]
fn test_transpile_dataframe_agg_count() {
    let transpiler = Transpiler::new();
    let code = r#"df.agg(count("value"))"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("count"));
}

// ==================== DATAFRAME CHAIN OPERATIONS ====================

#[test]
fn test_transpile_dataframe_chain_filter_select() {
    let transpiler = Transpiler::new();
    let code = r#"df.filter(age > 25).select(["name", "age"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("filter"));
    assert!(transpiled.contains("select"));
}

#[test]
fn test_transpile_dataframe_chain_groupby_agg() {
    let transpiler = Transpiler::new();
    let code = r#"df.groupby(["category"]).agg(sum("value"))"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("groupby"));
    assert!(transpiled.contains("sum"));
}

#[test]
fn test_transpile_dataframe_chain_multiple() {
    let transpiler = Transpiler::new();
    let code = r#"df.filter(active == true).groupby(["category"]).agg(mean("score")).sort(["score"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("filter"));
    assert!(transpiled.contains("groupby"));
    assert!(transpiled.contains("mean"));
    assert!(transpiled.contains("sort"));
}

// ==================== DATAFRAME COLUMN OPERATIONS ====================

#[test]
fn test_transpile_dataframe_rename() {
    let transpiler = Transpiler::new();
    let code = r#"df.rename({"old_name": "new_name"})"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("rename"));
}

#[test]
fn test_transpile_dataframe_drop() {
    let transpiler = Transpiler::new();
    let code = r#"df.drop(["column1", "column2"])"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("drop"));
}

#[test]
fn test_transpile_dataframe_with_column() {
    let transpiler = Transpiler::new();
    let code = r#"df.with_column("new_col", col("old_col") * 2)"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("with_column"));
}

// ==================== DATAFRAME IO OPERATIONS ====================

#[test]
fn test_transpile_dataframe_read_csv() {
    let transpiler = Transpiler::new();
    let code = r#"DataFrame.read_csv("data.csv")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("read_csv") || transpiled.contains("CsvReader"));
}

#[test]
fn test_transpile_dataframe_write_csv() {
    let transpiler = Transpiler::new();
    let code = r#"df.write_csv("output.csv")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("write_csv") || transpiled.contains("CsvWriter"));
}

#[test]
fn test_transpile_dataframe_head() {
    let transpiler = Transpiler::new();
    let code = r#"df.head(10)"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("head"));
}

#[test]
fn test_transpile_dataframe_tail() {
    let transpiler = Transpiler::new();
    let code = r#"df.tail(5)"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("tail"));
}

// Run all tests with: cargo test dataframe_tdd --test dataframe_tdd