//! Tests for DataFrame transpilation functionality
//!
//! This module tests the backend transpiler's dataframe capabilities,
//! targeting the 0% coverage backend/transpiler/dataframe.rs module.

use anyhow::Result;
use ruchy::Transpiler;
use ruchy::frontend::ast::{DataFrameColumn, DataFrameOp, JoinType, Expr, ExprKind, Literal, Span};

/// Test basic dataframe transpiler creation
#[test]
fn test_transpiler_dataframe_basic() {
    let _transpiler = Transpiler::new();
    // Should create successfully
    assert!(true);
}

/// Test empty dataframe transpilation
#[test]
fn test_transpile_empty_dataframe() -> Result<()> {
    let transpiler = Transpiler::new();
    
    // Test empty columns
    let empty_columns = vec![];
    let result = transpiler.transpile_dataframe(&empty_columns)?;
    let rust_code = result.to_string();
    
    // Should generate empty DataFrame code
    assert!(rust_code.contains("DataFrame") && rust_code.contains("empty"));
    
    Ok(())
}

/// Test single column dataframe transpilation
#[test]
fn test_transpile_single_column_dataframe() -> Result<()> {
    let transpiler = Transpiler::new();
    
    // Create a single column with integer values
    let span = Span::new(0, 1);
    let values = vec![
        Expr::new(ExprKind::Literal(Literal::Integer(1)), span),
        Expr::new(ExprKind::Literal(Literal::Integer(2)), span),
        Expr::new(ExprKind::Literal(Literal::Integer(3)), span),
    ];
    
    let column = DataFrameColumn {
        name: "numbers".to_string(),
        values,
    };
    
    let columns = vec![column];
    let result = transpiler.transpile_dataframe(&columns)?;
    let rust_code = result.to_string();
    
    // Should contain DataFrame creation with series (accounting for spaces in generated code)
    assert!(rust_code.contains("DataFrame") && rust_code.contains("Series"));
    assert!(rust_code.contains("numbers"));
    
    Ok(())
}

/// Test multi-column dataframe transpilation
#[test]
fn test_transpile_multi_column_dataframe() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    // Create multiple columns
    let name_column = DataFrameColumn {
        name: "name".to_string(),
        values: vec![
            Expr::new(ExprKind::Literal(Literal::String("Alice".to_string())), span),
            Expr::new(ExprKind::Literal(Literal::String("Bob".to_string())), span),
        ],
    };
    
    let age_column = DataFrameColumn {
        name: "age".to_string(),
        values: vec![
            Expr::new(ExprKind::Literal(Literal::Integer(25)), span),
            Expr::new(ExprKind::Literal(Literal::Integer(30)), span),
        ],
    };
    
    let columns = vec![name_column, age_column];
    let result = transpiler.transpile_dataframe(&columns)?;
    let rust_code = result.to_string();
    
    // Should contain both columns
    assert!(rust_code.contains("name"));
    assert!(rust_code.contains("age"));
    assert!(rust_code.contains("Alice"));
    assert!(rust_code.contains("Bob"));
    
    Ok(())
}

/// Test dataframe with empty column values
#[test]
fn test_transpile_dataframe_empty_column_values() -> Result<()> {
    let transpiler = Transpiler::new();
    
    let empty_column = DataFrameColumn {
        name: "empty_col".to_string(),
        values: vec![],
    };
    
    let columns = vec![empty_column];
    let result = transpiler.transpile_dataframe(&columns)?;
    let rust_code = result.to_string();
    
    // Empty column test - no debug output needed
    
    // Should handle empty column values (accounting for spaces in generated code)
    assert!(rust_code.contains("empty_col"));
    assert!(rust_code.contains("vec"));
    
    Ok(())
}

/// Test dataframe select operation transpilation
#[test]
fn test_transpile_dataframe_select_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    // Create a dummy dataframe expression
    let df_expr = Expr::new(ExprKind::Identifier("df".to_string()), span);
    
    // Create select operation
    let select_columns = vec!["col1".to_string(), "col2".to_string()];
    let select_op = DataFrameOp::Select(select_columns);
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &select_op)?;
    let rust_code = result.to_string();
    
    // Should contain select operation
    assert!(rust_code.contains("select"));
    assert!(rust_code.contains("col1"));
    assert!(rust_code.contains("col2"));
    
    Ok(())
}

/// Test dataframe filter operation transpilation
#[test]
fn test_transpile_dataframe_filter_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    let df_expr = Expr::new(ExprKind::Identifier("df".to_string()), span);
    let condition_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), span);
    
    let filter_op = DataFrameOp::Filter(Box::new(condition_expr));
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &filter_op)?;
    let rust_code = result.to_string();
    
    // Should contain filter operation
    assert!(rust_code.contains("filter"));
    assert!(rust_code.contains("df"));
    
    Ok(())
}

/// Test dataframe groupby operation transpilation
#[test]
fn test_transpile_dataframe_groupby_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    let df_expr = Expr::new(ExprKind::Identifier("df".to_string()), span);
    let group_columns = vec!["category".to_string(), "region".to_string()];
    let groupby_op = DataFrameOp::GroupBy(group_columns);
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &groupby_op)?;
    let rust_code = result.to_string();
    
    // Should contain groupby operation
    assert!(rust_code.contains("groupby"));
    assert!(rust_code.contains("category"));
    assert!(rust_code.contains("region"));
    
    Ok(())
}

/// Test dataframe sort operation transpilation
#[test]
fn test_transpile_dataframe_sort_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    let df_expr = Expr::new(ExprKind::Identifier("df".to_string()), span);
    let sort_columns = vec!["score".to_string(), "name".to_string()];
    let sort_op = DataFrameOp::Sort(sort_columns);
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &sort_op)?;
    let rust_code = result.to_string();
    
    // Should contain sort operation
    assert!(rust_code.contains("sort"));
    assert!(rust_code.contains("score"));
    assert!(rust_code.contains("name"));
    
    Ok(())
}

/// Test dataframe join operation transpilation - Left Join
#[test]
fn test_transpile_dataframe_left_join_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    let df_expr = Expr::new(ExprKind::Identifier("df1".to_string()), span);
    let other_expr = Expr::new(ExprKind::Identifier("df2".to_string()), span);
    let join_columns = vec!["id".to_string()];
    
    let join_op = DataFrameOp::Join {
        other: Box::new(other_expr),
        on: join_columns,
        how: JoinType::Left,
    };
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &join_op)?;
    let rust_code = result.to_string();
    
    // Should contain join operation
    assert!(rust_code.contains("df1"));
    assert!(rust_code.contains("df2"));
    assert!(rust_code.contains("id"));
    assert!(rust_code.contains("Left"));
    
    Ok(())
}

/// Test dataframe join operation transpilation - Right Join
#[test]
fn test_transpile_dataframe_right_join_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    let df_expr = Expr::new(ExprKind::Identifier("df1".to_string()), span);
    let other_expr = Expr::new(ExprKind::Identifier("df2".to_string()), span);
    let join_columns = vec!["key".to_string()];
    
    let join_op = DataFrameOp::Join {
        other: Box::new(other_expr),
        on: join_columns,
        how: JoinType::Right,
    };
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &join_op)?;
    let rust_code = result.to_string();
    
    // Should contain right join
    assert!(rust_code.contains("Right"));
    assert!(rust_code.contains("key"));
    
    Ok(())
}

/// Test dataframe join operation transpilation - Inner Join
#[test]
fn test_transpile_dataframe_inner_join_operation() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    let df_expr = Expr::new(ExprKind::Identifier("users".to_string()), span);
    let other_expr = Expr::new(ExprKind::Identifier("orders".to_string()), span);
    let join_columns = vec!["user_id".to_string()];
    
    let join_op = DataFrameOp::Join {
        other: Box::new(other_expr),
        on: join_columns,
        how: JoinType::Inner,
    };
    
    let result = transpiler.transpile_dataframe_operation(&df_expr, &join_op)?;
    let rust_code = result.to_string();
    
    // Should contain inner join
    assert!(rust_code.contains("Inner"));
    assert!(rust_code.contains("users"));
    assert!(rust_code.contains("orders"));
    assert!(rust_code.contains("user_id"));
    
    Ok(())
}

/// Test dataframe with mixed data types
#[test]
fn test_transpile_dataframe_mixed_types() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    // Create a column with mixed literal types
    let mixed_column = DataFrameColumn {
        name: "mixed".to_string(),
        values: vec![
            Expr::new(ExprKind::Literal(Literal::Integer(42)), span),
            Expr::new(ExprKind::Literal(Literal::Float(3.14159)), span),
            Expr::new(ExprKind::Literal(Literal::String("text".to_string())), span),
            Expr::new(ExprKind::Literal(Literal::Bool(true)), span),
        ],
    };
    
    let columns = vec![mixed_column];
    let result = transpiler.transpile_dataframe(&columns)?;
    let rust_code = result.to_string();
    
    // Should handle all data types
    assert!(rust_code.contains("mixed"));
    assert!(rust_code.contains("42"));
    assert!(rust_code.contains("3.14159"));
    assert!(rust_code.contains("text"));
    assert!(rust_code.contains("true"));
    
    Ok(())
}

/// Test comprehensive dataframe operation chaining
#[test]
fn test_transpile_complex_dataframe_operations() -> Result<()> {
    let transpiler = Transpiler::new();
    let span = Span::new(0, 1);
    
    // Test multiple operations on the same dataframe
    let df_expr = Expr::new(ExprKind::Identifier("sales_data".to_string()), span);
    
    // Test filter
    let condition_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), span);
    let filter_op = DataFrameOp::Filter(Box::new(condition_expr));
    
    let filter_result = transpiler.transpile_dataframe_operation(&df_expr, &filter_op)?;
    assert!(filter_result.to_string().contains("filter"));
    
    // Test select
    let select_columns = vec!["product".to_string(), "revenue".to_string()];
    let select_op = DataFrameOp::Select(select_columns);
    
    let select_result = transpiler.transpile_dataframe_operation(&df_expr, &select_op)?;
    assert!(select_result.to_string().contains("select"));
    assert!(select_result.to_string().contains("product"));
    assert!(select_result.to_string().contains("revenue"));
    
    // Test groupby
    let group_columns = vec!["category".to_string()];
    let groupby_op = DataFrameOp::GroupBy(group_columns);
    
    let groupby_result = transpiler.transpile_dataframe_operation(&df_expr, &groupby_op)?;
    assert!(groupby_result.to_string().contains("groupby"));
    assert!(groupby_result.to_string().contains("category"));
    
    Ok(())
}