//! STD-007: `DataFrame` Module Tests (ruchy/std/dataframe)
//!
//! Test suite for `DataFrame` operations module.
//! Thin wrappers around polars-rs for data manipulation.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

#![cfg(feature = "dataframe")]
#![allow(missing_docs)]

use std::fs;
use std::path::Path;

// ===== Creation & I/O Tests =====

#[test]
fn test_std_007_from_columns_single() {
    // STD-007: Test creating DataFrame from single column

    let result = ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30, 35])]);

    assert!(result.is_ok(), "from_columns should succeed");
    let df = result.unwrap();

    let shape = df.shape();
    assert_eq!(shape.0, 3, "Should have 3 rows");
    assert_eq!(shape.1, 1, "Should have 1 column");

    let columns = df.get_column_names();
    assert_eq!(columns.len(), 1, "Should have 1 column name");
    assert_eq!(columns[0], "age", "Column name should be 'age'");
}

#[test]
fn test_std_007_from_columns_multiple() {
    // STD-007: Test creating DataFrame from multiple columns

    let result = ruchy::stdlib::dataframe::from_columns(vec![
        ("name", vec![1, 2, 3]), // Using integers for simplicity
        ("age", vec![25, 30, 35]),
    ]);

    assert!(result.is_ok(), "from_columns should succeed");
    let df = result.unwrap();

    let shape = df.shape();
    assert_eq!(shape.0, 3, "Should have 3 rows");
    assert_eq!(shape.1, 2, "Should have 2 columns");

    let columns = df.get_column_names();
    assert_eq!(columns.len(), 2, "Should have 2 column names");
    assert!(
        columns.iter().any(|c| c.as_str() == "name"),
        "Should have 'name' column"
    );
    assert!(
        columns.iter().any(|c| c.as_str() == "age"),
        "Should have 'age' column"
    );
}

#[test]
fn test_std_007_from_columns_empty() {
    // STD-007: Test creating DataFrame from empty columns

    let result = ruchy::stdlib::dataframe::from_columns(vec![]);

    // Empty DataFrame is valid
    assert!(
        result.is_ok(),
        "from_columns should succeed with empty input"
    );
    let df = result.unwrap();

    let shape = df.shape();
    assert_eq!(shape.0, 0, "Should have 0 rows");
    assert_eq!(shape.1, 0, "Should have 0 columns");
}

#[test]
fn test_std_007_from_columns_mismatched_lengths() {
    // STD-007: Test error on mismatched column lengths

    let result = ruchy::stdlib::dataframe::from_columns(vec![
        ("age", vec![25, 30]),
        ("score", vec![95, 87, 92]), // Different length!
    ]);

    assert!(
        result.is_err(),
        "from_columns should fail with mismatched lengths"
    );
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "Error message should not be empty");
    assert!(
        error.contains("length") || error.contains("mismatch") || error.contains("same"),
        "Error should mention length mismatch"
    );
}

#[test]
fn test_std_007_read_csv_valid() {
    // STD-007: Test reading valid CSV file

    // Create test CSV file
    let test_csv = "/tmp/test_std_007_read.csv";
    fs::write(test_csv, "age,score\n25,95\n30,87\n35,92\n").unwrap();

    let result = ruchy::stdlib::dataframe::read_csv(test_csv);

    assert!(result.is_ok(), "read_csv should succeed");
    let df = result.unwrap();

    let shape = df.shape();
    assert_eq!(shape.0, 3, "Should have 3 rows");
    assert_eq!(shape.1, 2, "Should have 2 columns");

    let columns = df.get_column_names();
    assert!(
        columns.iter().any(|c| c.as_str() == "age"),
        "Should have 'age' column"
    );
    assert!(
        columns.iter().any(|c| c.as_str() == "score"),
        "Should have 'score' column"
    );

    // Cleanup
    fs::remove_file(test_csv).ok();
}

#[test]
fn test_std_007_read_csv_not_found() {
    // STD-007: Test error when CSV file doesn't exist

    let result = ruchy::stdlib::dataframe::read_csv("/tmp/nonexistent_file_xyz_12345.csv");

    assert!(
        result.is_err(),
        "read_csv should fail for non-existent file"
    );
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "Error message should not be empty");
}

#[test]
fn test_std_007_write_csv_success() {
    // STD-007: Test writing DataFrame to CSV

    let mut df = ruchy::stdlib::dataframe::from_columns(vec![
        ("age", vec![25, 30, 35]),
        ("score", vec![95, 87, 92]),
    ])
    .unwrap();

    let test_csv = "/tmp/test_std_007_write.csv";
    let result = ruchy::stdlib::dataframe::write_csv(&mut df, test_csv);

    assert!(result.is_ok(), "write_csv should succeed");
    assert!(Path::new(test_csv).exists(), "CSV file should exist");

    // Verify file has content
    let content = fs::read_to_string(test_csv).unwrap();
    assert!(!content.is_empty(), "CSV should not be empty");
    assert!(content.contains("age"), "CSV should contain 'age' column");
    assert!(
        content.contains("score"),
        "CSV should contain 'score' column"
    );

    // Cleanup
    fs::remove_file(test_csv).ok();
}

// ===== Selection & Filtering Tests =====

#[test]
fn test_std_007_select_single_column() {
    // STD-007: Test selecting single column

    let df = ruchy::stdlib::dataframe::from_columns(vec![
        ("age", vec![25, 30, 35]),
        ("score", vec![95, 87, 92]),
    ])
    .unwrap();

    let result = ruchy::stdlib::dataframe::select(&df, &["age"]);

    assert!(result.is_ok(), "select should succeed");
    let selected = result.unwrap();

    let shape = selected.shape();
    assert_eq!(shape.0, 3, "Should preserve row count");
    assert_eq!(shape.1, 1, "Should have 1 column");

    let columns = selected.get_column_names();
    assert_eq!(columns.len(), 1, "Should have 1 column");
    assert_eq!(columns[0], "age", "Column should be 'age'");
}

#[test]
fn test_std_007_select_multiple_columns() {
    // STD-007: Test selecting multiple columns

    let df = ruchy::stdlib::dataframe::from_columns(vec![
        ("name", vec![1, 2, 3]),
        ("age", vec![25, 30, 35]),
        ("score", vec![95, 87, 92]),
    ])
    .unwrap();

    let result = ruchy::stdlib::dataframe::select(&df, &["age", "score"]);

    assert!(result.is_ok(), "select should succeed");
    let selected = result.unwrap();

    let shape = selected.shape();
    assert_eq!(shape.0, 3, "Should preserve row count");
    assert_eq!(shape.1, 2, "Should have 2 columns");

    let columns = selected.get_column_names();
    assert_eq!(columns.len(), 2, "Should have 2 columns");
    assert!(
        columns.iter().any(|c| c.as_str() == "age"),
        "Should have 'age'"
    );
    assert!(
        columns.iter().any(|c| c.as_str() == "score"),
        "Should have 'score'"
    );
}

#[test]
fn test_std_007_select_nonexistent_column() {
    // STD-007: Test error on non-existent column

    let df = ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();

    let result = ruchy::stdlib::dataframe::select(&df, &["nonexistent"]);

    assert!(
        result.is_err(),
        "select should fail for non-existent column"
    );
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "Error message should not be empty");
}

#[test]
fn test_std_007_head_normal() {
    // STD-007: Test getting first n rows

    let df =
        ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30, 35, 40, 45])]).unwrap();

    let result = ruchy::stdlib::dataframe::head(&df, 3);

    assert!(result.is_ok(), "head should succeed");
    let top = result.unwrap();

    let shape = top.shape();
    assert_eq!(shape.0, 3, "Should have 3 rows");
    assert_eq!(shape.1, 1, "Should preserve column count");
}

#[test]
fn test_std_007_head_exceeds_rows() {
    // STD-007: Test head with n > total rows

    let df = ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();

    let result = ruchy::stdlib::dataframe::head(&df, 10);

    assert!(result.is_ok(), "head should succeed even when n > rows");
    let top = result.unwrap();

    let shape = top.shape();
    assert_eq!(shape.0, 2, "Should return all rows");
}

#[test]
fn test_std_007_tail_normal() {
    // STD-007: Test getting last n rows

    let df =
        ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30, 35, 40, 45])]).unwrap();

    let result = ruchy::stdlib::dataframe::tail(&df, 3);

    assert!(result.is_ok(), "tail should succeed");
    let bottom = result.unwrap();

    let shape = bottom.shape();
    assert_eq!(shape.0, 3, "Should have 3 rows");
    assert_eq!(shape.1, 1, "Should preserve column count");
}

#[test]
fn test_std_007_tail_exceeds_rows() {
    // STD-007: Test tail with n > total rows

    let df = ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();

    let result = ruchy::stdlib::dataframe::tail(&df, 10);

    assert!(result.is_ok(), "tail should succeed even when n > rows");
    let bottom = result.unwrap();

    let shape = bottom.shape();
    assert_eq!(shape.0, 2, "Should return all rows");
}

// ===== Metadata Tests =====

#[test]
fn test_std_007_shape() {
    // STD-007: Test getting DataFrame shape

    let df = ruchy::stdlib::dataframe::from_columns(vec![
        ("age", vec![25, 30, 35]),
        ("score", vec![95, 87, 92]),
    ])
    .unwrap();

    let result = ruchy::stdlib::dataframe::shape(&df);

    assert!(result.is_ok(), "shape should succeed");
    let (rows, cols) = result.unwrap();

    assert_eq!(rows, 3, "Should have 3 rows");
    assert_eq!(cols, 2, "Should have 2 columns");
}

#[test]
fn test_std_007_shape_empty() {
    // STD-007: Test shape of empty DataFrame

    let df = ruchy::stdlib::dataframe::from_columns(vec![]).unwrap();

    let result = ruchy::stdlib::dataframe::shape(&df);

    assert!(result.is_ok(), "shape should succeed for empty DataFrame");
    let (rows, cols) = result.unwrap();

    assert_eq!(rows, 0, "Should have 0 rows");
    assert_eq!(cols, 0, "Should have 0 columns");
}

#[test]
fn test_std_007_columns() {
    // STD-007: Test getting column names

    let df = ruchy::stdlib::dataframe::from_columns(vec![
        ("name", vec![1, 2, 3]),
        ("age", vec![25, 30, 35]),
    ])
    .unwrap();

    let result = ruchy::stdlib::dataframe::columns(&df);

    assert!(result.is_ok(), "columns should succeed");
    let names = result.unwrap();

    assert_eq!(names.len(), 2, "Should have 2 column names");
    assert!(names.contains(&"name".to_string()), "Should contain 'name'");
    assert!(names.contains(&"age".to_string()), "Should contain 'age'");
}

#[test]
fn test_std_007_row_count() {
    // STD-007: Test getting row count

    let df = ruchy::stdlib::dataframe::from_columns(vec![("age", vec![25, 30, 35, 40])]).unwrap();

    let result = ruchy::stdlib::dataframe::row_count(&df);

    assert!(result.is_ok(), "row_count should succeed");
    let count = result.unwrap();

    assert_eq!(count, 4, "Should have 4 rows");
}

#[test]
fn test_std_007_row_count_empty() {
    // STD-007: Test row count of empty DataFrame

    let df = ruchy::stdlib::dataframe::from_columns(vec![]).unwrap();

    let result = ruchy::stdlib::dataframe::row_count(&df);

    assert!(
        result.is_ok(),
        "row_count should succeed for empty DataFrame"
    );
    let count = result.unwrap();

    assert_eq!(count, 0, "Should have 0 rows");
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_007_csv_roundtrip(rows in 1usize..10, cols in 1usize..5) {
            // Property: write_csv → read_csv should preserve shape

            // Create DataFrame with random dimensions
            let col_names: Vec<String> = (0..cols).map(|i| format!("col{i}")).collect();
            let columns: Vec<(&str, Vec<i64>)> = col_names.iter()
                .enumerate()
                .map(|(i, name)| {
                    let values: Vec<i64> = (0..rows).map(|j| (i * 100 + j).try_into().expect("test values fit in i64")).collect();
                    (name.as_str(), values)
                })
                .collect();

            let mut df = ruchy::stdlib::dataframe::from_columns(columns).unwrap();
            let original_shape = df.shape();

            // Write to CSV
            let test_file = format!("/tmp/test_std_007_roundtrip_{rows}_{cols}.csv");
            ruchy::stdlib::dataframe::write_csv(&mut df, &test_file).unwrap();

            // Read back
            let df_read = ruchy::stdlib::dataframe::read_csv(&test_file).unwrap();
            let read_shape = df_read.shape();

            // Cleanup
            std::fs::remove_file(&test_file).ok();

            // Verify shape preserved
            prop_assert_eq!(original_shape.0, read_shape.0, "Rows should be preserved");
            prop_assert_eq!(original_shape.1, read_shape.1, "Columns should be preserved");
        }

        #[test]
        fn test_std_007_never_panics_select(n_cols in 1usize..10) {
            // Property: select should never panic, even with invalid input

            let col_names: Vec<String> = (0..n_cols).map(|i| format!("col{i}")).collect();
            let columns: Vec<(&str, Vec<i64>)> = col_names.iter()
                .map(|name| (name.as_str(), vec![1, 2, 3]))
                .collect();
            let df = ruchy::stdlib::dataframe::from_columns(columns).unwrap();

            // Try selecting non-existent column - should return error, not panic
            let _ = ruchy::stdlib::dataframe::select(&df, &["nonexistent"]);
            // Should not panic
        }

        #[test]
        fn test_std_007_head_tail_consistency(n_rows in 2usize..20, take in 1usize..10) {
            // Property: head and tail should preserve column count

            let df = ruchy::stdlib::dataframe::from_columns(vec![
                ("col1", (0..n_rows).map(|x| x.try_into().expect("test values fit in i64")).collect()),
                ("col2", (0..n_rows).map(|x| (x * 2).try_into().expect("test values fit in i64")).collect()),
            ]).unwrap();

            let original_cols = df.shape().1;

            let head_df = ruchy::stdlib::dataframe::head(&df, take).unwrap();
            let tail_df = ruchy::stdlib::dataframe::tail(&df, take).unwrap();

            prop_assert_eq!(head_df.shape().1, original_cols, "head should preserve columns");
            prop_assert_eq!(tail_df.shape().1, original_cols, "tail should preserve columns");
        }
    }
}
