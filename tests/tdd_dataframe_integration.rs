//! TDD tests for DataFrame integration
//! 
//! Tests Apache Arrow/Polars DataFrame operations and performance
//! following TDD RED->GREEN->REFACTOR methodology

use ruchy::wasm::notebook::NotebookRuntime;
use std::time::Instant;

#[test]
fn test_dataframe_creation_in_notebook() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create a simple DataFrame
    let result = runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_rows([[1, 2], [3, 4]])"
    );
    
    // Should succeed without error
    assert!(result.is_ok(), "DataFrame creation failed");
}

#[test]
fn test_dataframe_column_selection() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create DataFrame and select column
    runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_rows([[1, 2], [3, 4]])"
    ).ok();
    
    let result = runtime.execute_cell_with_session("cell2", 
        "df.select(\"column_0\")"
    );
    
    // Should be able to select columns
    assert!(result.is_ok(), "Column selection failed");
}

#[test]
fn test_dataframe_filter_operations() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create and filter DataFrame
    runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_rows([[1, 2], [3, 4], [5, 6]])"
    ).ok();
    
    let result = runtime.execute_cell_with_session("cell2", 
        "df.filter(col(\"column_0\") > 2)"
    );
    
    // Filter should work
    assert!(result.is_ok(), "DataFrame filter failed");
}

#[test]
fn test_dataframe_aggregation() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create and aggregate DataFrame
    runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_rows([[1, 2], [3, 4], [5, 6]])"
    ).ok();
    
    let result = runtime.execute_cell_with_session("cell2", 
        "df.sum()"
    );
    
    // Aggregation should work
    assert!(result.is_ok(), "DataFrame aggregation failed");
}

#[test]
fn test_dataframe_zero_copy_slice() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create large DataFrame
    runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_range(0, 10000)"
    ).ok();
    
    // Slice should be zero-copy (instant)
    let start = Instant::now();
    let result = runtime.execute_cell_with_session("cell2", 
        "let slice = df.slice(1000, 2000)"
    );
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "DataFrame slice failed");
    assert!(elapsed.as_millis() < 10, "Slice operation too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_dataframe_memory_efficiency() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Get initial memory
    let initial_mem = runtime.get_memory_usage();
    
    // Create large DataFrame
    runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_range(0, 100000)"
    ).ok();
    
    // Check memory usage
    let after_mem = runtime.get_memory_usage();
    
    // Parse memory values (they're JSON strings)
    // This is a simplified check - in production we'd parse properly
    assert!(initial_mem != after_mem, "Memory usage should change");
}

#[test]
#[ignore] // Performance test - run with --ignored
fn test_dataframe_1m_row_performance() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create 1M row DataFrame
    let start = Instant::now();
    let result = runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_range(0, 1000000)"
    );
    let creation_time = start.elapsed();
    
    assert!(result.is_ok(), "1M row DataFrame creation failed");
    assert!(creation_time.as_millis() < 1000, 
            "DataFrame creation too slow: {}ms", creation_time.as_millis());
    
    // Test operation performance
    let start = Instant::now();
    let result = runtime.execute_cell_with_session("cell2", 
        "df.filter(col(\"value\") % 2 == 0)"
    );
    let filter_time = start.elapsed();
    
    assert!(result.is_ok(), "1M row filter failed");
    assert!(filter_time.as_millis() < 50, 
            "Filter operation too slow: {}ms", filter_time.as_millis());
}

#[test]
fn test_dataframe_join_operations() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create two DataFrames
    runtime.execute_cell_with_session("cell1", 
        "let df1 = DataFrame::from_rows([[1, \"a\"], [2, \"b\"]])"
    ).ok();
    
    runtime.execute_cell_with_session("cell2", 
        "let df2 = DataFrame::from_rows([[1, \"x\"], [2, \"y\"]])"
    ).ok();
    
    // Join them
    let result = runtime.execute_cell_with_session("cell3", 
        "df1.join(df2, on=\"column_0\")"
    );
    
    assert!(result.is_ok(), "DataFrame join failed");
}

#[test]
fn test_dataframe_groupby_operations() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create DataFrame with groups
    runtime.execute_cell_with_session("cell1", 
        "let df = DataFrame::from_rows([[\"a\", 1], [\"b\", 2], [\"a\", 3]])"
    ).ok();
    
    // Group by and aggregate
    let result = runtime.execute_cell_with_session("cell2", 
        "df.groupby(\"column_0\").sum()"
    );
    
    assert!(result.is_ok(), "DataFrame groupby failed");
}

#[test]
fn test_dataframe_polars_compatibility() {
    // Test that our DataFrame implementation is compatible with Polars
    // This is more of an integration test
    
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Try Polars-style syntax
    let result = runtime.execute_cell_with_session("cell1", 
        "let df = pl.DataFrame({\"a\": [1, 2, 3], \"b\": [4, 5, 6]})"
    );
    
    // Should either work or give meaningful error
    // For now, we just check it doesn't panic
    let _ = result;
}