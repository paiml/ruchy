//! Integration tests for stdlib::dataframe module
//!
//! Target: 0% → 100% coverage for stdlib/dataframe.rs (178 lines)
//! Protocol: EXTREME TDD - External integration tests provide llvm-cov coverage
//!
//! Root Cause: #[cfg(test)] unit tests exist but aren't tracked by coverage.
//! Solution: Integration tests in tests/ directory ARE tracked by llvm-cov.
//!
//! **Note**: This module is feature-gated - only available with `--features dataframe`

#![cfg(feature = "dataframe")]

use ruchy::stdlib::dataframe;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// from_columns() TESTS
// ============================================================================

#[test]
fn test_dataframe_from_columns_basic() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
    assert_eq!(df.height(), 3, "Should have 3 rows");
    assert_eq!(df.width(), 1, "Should have 1 column");
}

#[test]
fn test_dataframe_from_columns_multiple() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])]).unwrap();
    assert_eq!(df.height(), 2, "Should have 2 rows");
    assert_eq!(df.width(), 2, "Should have 2 columns");
}

#[test]
fn test_dataframe_from_columns_empty() {
    let df = dataframe::from_columns(vec![]).unwrap();
    assert_eq!(df.height(), 0, "Empty dataframe should have 0 rows");
    assert_eq!(df.width(), 0, "Empty dataframe should have 0 columns");
}

#[test]
fn test_dataframe_from_columns_mismatched_lengths() {
    let result = dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95])]);
    assert!(result.is_err(), "Mismatched lengths should return error");
    let err = result.unwrap_err();
    assert!(err.contains("length"), "Error should mention 'length'");
    assert!(err.contains("score"), "Error should mention 'score' column");
}

#[test]
fn test_dataframe_from_columns_single_column_various_lengths() {
    // Single column with 0 elements
    let df = dataframe::from_columns(vec![("empty", vec![])]).unwrap();
    assert_eq!(df.height(), 0, "Empty column should have 0 rows");
    assert_eq!(df.width(), 1, "Should have 1 column");

    // Single column with 1 element
    let df = dataframe::from_columns(vec![("single", vec![42])]).unwrap();
    assert_eq!(df.height(), 1, "Should have 1 row");
    assert_eq!(df.width(), 1, "Should have 1 column");

    // Single column with many elements
    let df = dataframe::from_columns(vec![("many", vec![1, 2, 3, 4, 5])]).unwrap();
    assert_eq!(df.height(), 5, "Should have 5 rows");
    assert_eq!(df.width(), 1, "Should have 1 column");
}

// ============================================================================
// CSV I/O TESTS
// ============================================================================

#[test]
fn test_dataframe_read_csv_nonexistent() {
    let result = dataframe::read_csv("/nonexistent/path/file.csv");
    assert!(result.is_err(), "Reading nonexistent file should return error");
}

#[test]
fn test_dataframe_write_csv_basic() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("test.csv");
    let csv_path_str = csv_path.to_str().unwrap();

    let mut df = dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();
    dataframe::write_csv(&mut df, csv_path_str).unwrap();

    // Verify file exists
    assert!(csv_path.exists(), "CSV file should be created");

    // Verify content can be read back
    let contents = fs::read_to_string(&csv_path).unwrap();
    assert!(contents.contains("age"), "CSV should contain column name");
}

#[test]
fn test_dataframe_write_csv_multiple_columns() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("multi.csv");
    let csv_path_str = csv_path.to_str().unwrap();

    let mut df =
        dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])]).unwrap();
    dataframe::write_csv(&mut df, csv_path_str).unwrap();

    // Verify content
    let contents = fs::read_to_string(&csv_path).unwrap();
    assert!(contents.contains("age"), "CSV should contain 'age' column");
    assert!(contents.contains("score"), "CSV should contain 'score' column");
}

#[test]
fn test_dataframe_read_write_csv_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("roundtrip.csv");
    let csv_path_str = csv_path.to_str().unwrap();

    // Write
    let mut df1 = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
    dataframe::write_csv(&mut df1, csv_path_str).unwrap();

    // Read back
    let df2 = dataframe::read_csv(csv_path_str).unwrap();
    assert_eq!(df2.height(), 3, "Roundtrip should preserve row count");
    assert_eq!(df2.width(), 1, "Roundtrip should preserve column count");
}

// ============================================================================
// select() TESTS
// ============================================================================

#[test]
fn test_dataframe_select_single_column() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])]).unwrap();
    let subset = dataframe::select(&df, &["age"]).unwrap();
    assert_eq!(subset.width(), 1, "Should select 1 column");
    assert_eq!(subset.height(), 2, "Should preserve row count");
}

#[test]
fn test_dataframe_select_multiple_columns() {
    let df = dataframe::from_columns(vec![
        ("age", vec![25, 30]),
        ("score", vec![95, 87]),
        ("grade", vec![1, 2]),
    ])
    .unwrap();
    let subset = dataframe::select(&df, &["age", "score"]).unwrap();
    assert_eq!(subset.width(), 2, "Should select 2 columns");
    assert_eq!(subset.height(), 2, "Should preserve row count");
}

#[test]
fn test_dataframe_select_nonexistent_column() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();
    let result = dataframe::select(&df, &["nonexistent"]);
    assert!(result.is_err(), "Selecting nonexistent column should return error");
}

// ============================================================================
// head() TESTS
// ============================================================================

#[test]
fn test_dataframe_head_basic() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35, 40, 45])]).unwrap();
    let top3 = dataframe::head(&df, 3).unwrap();
    assert_eq!(top3.height(), 3, "head(3) should return 3 rows");
}

#[test]
fn test_dataframe_head_more_than_length() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();
    let top10 = dataframe::head(&df, 10).unwrap();
    assert_eq!(top10.height(), 2, "head(10) should return all 2 rows");
}

#[test]
fn test_dataframe_head_zero() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
    let top0 = dataframe::head(&df, 0).unwrap();
    assert_eq!(top0.height(), 0, "head(0) should return 0 rows");
}

// ============================================================================
// tail() TESTS
// ============================================================================

#[test]
fn test_dataframe_tail_basic() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35, 40, 45])]).unwrap();
    let bottom3 = dataframe::tail(&df, 3).unwrap();
    assert_eq!(bottom3.height(), 3, "tail(3) should return 3 rows");
}

#[test]
fn test_dataframe_tail_more_than_length() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();
    let bottom10 = dataframe::tail(&df, 10).unwrap();
    assert_eq!(bottom10.height(), 2, "tail(10) should return all 2 rows");
}

#[test]
fn test_dataframe_tail_zero() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
    let bottom0 = dataframe::tail(&df, 0).unwrap();
    assert_eq!(bottom0.height(), 0, "tail(0) should return 0 rows");
}

// ============================================================================
// shape() TESTS
// ============================================================================

#[test]
fn test_dataframe_shape_basic() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
    let (rows, cols) = dataframe::shape(&df).unwrap();
    assert_eq!(rows, 3, "Should have 3 rows");
    assert_eq!(cols, 1, "Should have 1 column");
}

#[test]
fn test_dataframe_shape_multiple_columns() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])]).unwrap();
    let (rows, cols) = dataframe::shape(&df).unwrap();
    assert_eq!(rows, 2, "Should have 2 rows");
    assert_eq!(cols, 2, "Should have 2 columns");
}

#[test]
fn test_dataframe_shape_empty() {
    let df = dataframe::from_columns(vec![]).unwrap();
    let (rows, cols) = dataframe::shape(&df).unwrap();
    assert_eq!(rows, 0, "Empty dataframe should have 0 rows");
    assert_eq!(cols, 0, "Empty dataframe should have 0 columns");
}

// ============================================================================
// columns() TESTS
// ============================================================================

#[test]
fn test_dataframe_columns_basic() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();
    let names = dataframe::columns(&df).unwrap();
    assert_eq!(names.len(), 1, "Should have 1 column name");
    assert_eq!(names[0], "age", "Column name should be 'age'");
}

#[test]
fn test_dataframe_columns_multiple() {
    let df = dataframe::from_columns(vec![
        ("age", vec![25]),
        ("score", vec![95]),
        ("grade", vec![1]),
    ])
    .unwrap();
    let names = dataframe::columns(&df).unwrap();
    assert_eq!(names.len(), 3, "Should have 3 column names");
    assert!(names.contains(&"age".to_string()), "Should contain 'age'");
    assert!(names.contains(&"score".to_string()), "Should contain 'score'");
    assert!(names.contains(&"grade".to_string()), "Should contain 'grade'");
}

#[test]
fn test_dataframe_columns_empty() {
    let df = dataframe::from_columns(vec![]).unwrap();
    let names = dataframe::columns(&df).unwrap();
    assert_eq!(names.len(), 0, "Empty dataframe should have no column names");
}

// ============================================================================
// row_count() TESTS
// ============================================================================

#[test]
fn test_dataframe_row_count_basic() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
    let count = dataframe::row_count(&df).unwrap();
    assert_eq!(count, 3, "Should have 3 rows");
}

#[test]
fn test_dataframe_row_count_empty() {
    let df = dataframe::from_columns(vec![("age", vec![])]).unwrap();
    let count = dataframe::row_count(&df).unwrap();
    assert_eq!(count, 0, "Empty column should have 0 rows");
}

#[test]
fn test_dataframe_row_count_multiple_columns() {
    let df = dataframe::from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])]).unwrap();
    let count = dataframe::row_count(&df).unwrap();
    assert_eq!(count, 2, "Should have 2 rows");
}

// ============================================================================
// WORKFLOW TEST
// ============================================================================

#[test]
fn test_dataframe_complete_workflow() {
    // Complete workflow: create, select, head, tail, shape, columns, row_count
    let df = dataframe::from_columns(vec![
        ("age", vec![25, 30, 35, 40, 45]),
        ("score", vec![95, 87, 92, 88, 90]),
    ])
    .unwrap();

    // Select
    let subset = dataframe::select(&df, &["age"]).unwrap();
    assert_eq!(subset.width(), 1, "Select should return 1 column");

    // Head
    let top2 = dataframe::head(&df, 2).unwrap();
    assert_eq!(top2.height(), 2, "Head should return 2 rows");

    // Tail
    let bottom2 = dataframe::tail(&df, 2).unwrap();
    assert_eq!(bottom2.height(), 2, "Tail should return 2 rows");

    // Shape
    let (rows, cols) = dataframe::shape(&df).unwrap();
    assert_eq!(rows, 5, "Shape should report 5 rows");
    assert_eq!(cols, 2, "Shape should report 2 columns");

    // Columns
    let names = dataframe::columns(&df).unwrap();
    assert_eq!(names.len(), 2, "Should have 2 column names");

    // Row count
    let count = dataframe::row_count(&df).unwrap();
    assert_eq!(count, 5, "Row count should be 5");
}

#[test]
fn test_dataframe_csv_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("workflow.csv");
    let csv_path_str = csv_path.to_str().unwrap();

    // Create dataframe
    let mut df = dataframe::from_columns(vec![
        ("age", vec![25, 30, 35]),
        ("score", vec![95, 87, 92]),
    ])
    .unwrap();

    // Check initial state
    let (rows, cols) = dataframe::shape(&df).unwrap();
    assert_eq!(rows, 3, "Initial dataframe should have 3 rows");
    assert_eq!(cols, 2, "Initial dataframe should have 2 columns");

    // Write to CSV
    dataframe::write_csv(&mut df, csv_path_str).unwrap();

    // Read back from CSV
    let df2 = dataframe::read_csv(csv_path_str).unwrap();

    // Verify roundtrip
    let (rows2, cols2) = dataframe::shape(&df2).unwrap();
    assert_eq!(rows2, 3, "Roundtrip should preserve 3 rows");
    assert_eq!(cols2, 2, "Roundtrip should preserve 2 columns");

    // Verify column names
    let names = dataframe::columns(&df2).unwrap();
    assert!(names.contains(&"age".to_string()), "Should have 'age' column after roundtrip");
    assert!(names.contains(&"score".to_string()), "Should have 'score' column after roundtrip");
}
