//! `DataFrame` Operations Module (ruchy/std/dataframe)
//!
//! Thin wrappers around Polars `DataFrame` for data manipulation.
//!
//! **Design**: Thin wrappers (complexity ≤3 per function) around polars crate.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.
//! **Feature**: Only available when compiled with `--features dataframe`

use polars::prelude::*;
use std::fs::File;

/// Create `DataFrame` from column name-value pairs
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![
///     ("age", vec![25, 30, 35]),
///     ("score", vec![95, 87, 92])
/// ]).expect("Failed to create DataFrame from columns");
/// # }
/// ```
///
/// # Errors
///
/// Returns error if columns have mismatched lengths
pub fn from_columns(columns: Vec<(&str, Vec<i64>)>) -> Result<DataFrame, String> {
    use polars::datatypes::PlSmallStr;
    use polars::prelude::{NamedFrom, Series};

    if columns.is_empty() {
        return Ok(DataFrame::default());
    }

    // Check all columns have same length
    if columns.len() > 1 {
        let first_len = columns[0].1.len();
        for (name, values) in &columns {
            if values.len() != first_len {
                return Err(format!(
                    "Column '{}' has length {} but expected {} (all columns must have same length)",
                    name,
                    values.len(),
                    first_len
                ));
            }
        }
    }

    let cols: Vec<Column> = columns
        .into_iter()
        .map(|(name, values)| {
            let name_str = PlSmallStr::from(name);
            Series::new(name_str, values).into()
        })
        .collect();

    DataFrame::new(cols).map_err(|e| e.to_string())
}

/// Read CSV file into `DataFrame`
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::read_csv("data.csv").expect("Failed to read CSV file");
/// # }
/// ```
///
/// # Errors
///
/// Returns error if file doesn't exist or has invalid CSV format
pub fn read_csv(path: &str) -> Result<DataFrame, String> {
    CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(path.into()))
        .map_err(|e| e.to_string())?
        .finish()
        .map_err(|e| e.to_string())
}

/// Write `DataFrame` to CSV file
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let mut df = dataframe::from_columns(vec![("age", vec![25, 30])]).expect("Failed to create DataFrame");
/// dataframe::write_csv(&mut df, "output.csv").expect("Failed to write CSV file");
/// # }
/// ```
///
/// # Errors
///
/// Returns error if file cannot be created or written
pub fn write_csv(df: &mut DataFrame, path: &str) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| e.to_string())?;

    CsvWriter::new(&mut file)
        .finish(df)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Select specific columns from `DataFrame`
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![
///     ("age", vec![25, 30]),
///     ("score", vec![95, 87])
/// ]).expect("Failed to create DataFrame");
/// let subset = dataframe::select(&df, &["age"]).expect("Failed to select columns");
/// # }
/// ```
///
/// # Errors
///
/// Returns error if column doesn't exist
pub fn select(df: &DataFrame, columns: &[&str]) -> Result<DataFrame, String> {
    let col_names: Vec<_> = columns.iter().map(|&s| s.to_string()).collect();
    df.select(col_names).map_err(|e| e.to_string())
}

/// Get first n rows of `DataFrame`
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35, 40])]).expect("Failed to create DataFrame");
/// let top3 = dataframe::head(&df, 3).expect("Failed to get head of DataFrame");
/// # }
/// ```
pub fn head(df: &DataFrame, n: usize) -> Result<DataFrame, String> {
    Ok(df.head(Some(n)))
}

/// Get last n rows of `DataFrame`
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35, 40])]).expect("Failed to create DataFrame");
/// let bottom3 = dataframe::tail(&df, 3).expect("Failed to get tail of DataFrame");
/// # }
/// ```
pub fn tail(df: &DataFrame, n: usize) -> Result<DataFrame, String> {
    Ok(df.tail(Some(n)))
}

/// Get `DataFrame` dimensions (rows, columns)
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).expect("Failed to create DataFrame");
/// let (rows, cols) = dataframe::shape(&df).expect("Failed to get DataFrame shape");
/// # }
/// ```
pub fn shape(df: &DataFrame) -> Result<(usize, usize), String> {
    Ok(df.shape())
}

/// Get column names from `DataFrame`
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![
///     ("age", vec![25, 30]),
///     ("score", vec![95, 87])
/// ]).expect("Failed to create DataFrame");
/// let names = dataframe::columns(&df).expect("Failed to get column names");
/// # }
/// ```
pub fn columns(df: &DataFrame) -> Result<Vec<String>, String> {
    Ok(df
        .get_column_names_owned()
        .iter()
        .map(polars::prelude::PlSmallStr::to_string)
        .collect())
}

/// Get row count of `DataFrame`
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "dataframe")]
/// # {
/// use ruchy::stdlib::dataframe;
///
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).expect("Failed to create DataFrame");
/// let count = dataframe::row_count(&df).expect("Failed to get row count");
/// # }
/// ```
pub fn row_count(df: &DataFrame) -> Result<usize, String> {
    Ok(df.height())
}

#[cfg(all(test, feature = "dataframe"))]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_from_columns_basic() {
        let df = from_columns(vec![("age", vec![25, 30, 35])])
            .expect("operation should succeed in test");
        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 1);
    }

    #[test]
    fn test_from_columns_multiple() {
        let df = from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])])
            .expect("operation should succeed in test");
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn test_from_columns_empty() {
        let df = from_columns(vec![]).expect("operation should succeed in test");
        assert_eq!(df.height(), 0);
        assert_eq!(df.width(), 0);
    }

    #[test]
    fn test_from_columns_mismatched_lengths() {
        let result = from_columns(vec![("age", vec![25, 30]), ("score", vec![95])]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("length"));
        assert!(err.contains("score"));
    }

    #[test]
    fn test_from_columns_single_column_various_lengths() {
        // Single column with 0 elements
        let df = from_columns(vec![("empty", vec![])]).expect("operation should succeed in test");
        assert_eq!(df.height(), 0);
        assert_eq!(df.width(), 1);

        // Single column with 1 element
        let df =
            from_columns(vec![("single", vec![42])]).expect("operation should succeed in test");
        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 1);

        // Single column with many elements
        let df = from_columns(vec![("many", vec![1, 2, 3, 4, 5])])
            .expect("operation should succeed in test");
        assert_eq!(df.height(), 5);
        assert_eq!(df.width(), 1);
    }

    #[test]
    fn test_read_csv_nonexistent() {
        let result = read_csv("/nonexistent/path/file.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_csv_basic() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let csv_path = temp_dir.path().join("test.csv");
        let csv_path_str = csv_path.to_str().expect("operation should succeed in test");

        let mut df =
            from_columns(vec![("age", vec![25, 30])]).expect("operation should succeed in test");
        write_csv(&mut df, csv_path_str).expect("operation should succeed in test");

        // Verify file exists
        assert!(csv_path.exists());

        // Verify content can be read back
        let contents = fs::read_to_string(&csv_path).expect("operation should succeed in test");
        assert!(contents.contains("age"));
    }

    #[test]
    fn test_write_csv_multiple_columns() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let csv_path = temp_dir.path().join("multi.csv");
        let csv_path_str = csv_path.to_str().expect("operation should succeed in test");

        let mut df = from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])])
            .expect("operation should succeed in test");
        write_csv(&mut df, csv_path_str).expect("operation should succeed in test");

        // Verify content
        let contents = fs::read_to_string(&csv_path).expect("operation should succeed in test");
        assert!(contents.contains("age"));
        assert!(contents.contains("score"));
    }

    #[test]
    fn test_read_write_csv_roundtrip() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let csv_path = temp_dir.path().join("roundtrip.csv");
        let csv_path_str = csv_path.to_str().expect("operation should succeed in test");

        // Write
        let mut df1 = from_columns(vec![("age", vec![25, 30, 35])])
            .expect("operation should succeed in test");
        write_csv(&mut df1, csv_path_str).expect("operation should succeed in test");

        // Read back
        let df2 = read_csv(csv_path_str).expect("operation should succeed in test");
        assert_eq!(df2.height(), 3);
        assert_eq!(df2.width(), 1);
    }

    #[test]
    fn test_select_single_column() {
        let df = from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])])
            .expect("operation should succeed in test");
        let subset = select(&df, &["age"]).expect("operation should succeed in test");
        assert_eq!(subset.width(), 1);
        assert_eq!(subset.height(), 2);
    }

    #[test]
    fn test_select_multiple_columns() {
        let df = from_columns(vec![
            ("age", vec![25, 30]),
            ("score", vec![95, 87]),
            ("grade", vec![1, 2]),
        ])
        .expect("operation should succeed in test");
        let subset = select(&df, &["age", "score"]).expect("operation should succeed in test");
        assert_eq!(subset.width(), 2);
        assert_eq!(subset.height(), 2);
    }

    #[test]
    fn test_select_nonexistent_column() {
        let df =
            from_columns(vec![("age", vec![25, 30])]).expect("operation should succeed in test");
        let result = select(&df, &["nonexistent"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_head_basic() {
        let df = from_columns(vec![("age", vec![25, 30, 35, 40, 45])])
            .expect("operation should succeed in test");
        let top3 = head(&df, 3).expect("operation should succeed in test");
        assert_eq!(top3.height(), 3);
    }

    #[test]
    fn test_head_more_than_length() {
        let df =
            from_columns(vec![("age", vec![25, 30])]).expect("operation should succeed in test");
        let top10 = head(&df, 10).expect("operation should succeed in test");
        assert_eq!(top10.height(), 2); // Returns all available rows
    }

    #[test]
    fn test_head_zero() {
        let df = from_columns(vec![("age", vec![25, 30, 35])])
            .expect("operation should succeed in test");
        let top0 = head(&df, 0).expect("operation should succeed in test");
        assert_eq!(top0.height(), 0);
    }

    #[test]
    fn test_tail_basic() {
        let df = from_columns(vec![("age", vec![25, 30, 35, 40, 45])])
            .expect("operation should succeed in test");
        let bottom3 = tail(&df, 3).expect("operation should succeed in test");
        assert_eq!(bottom3.height(), 3);
    }

    #[test]
    fn test_tail_more_than_length() {
        let df =
            from_columns(vec![("age", vec![25, 30])]).expect("operation should succeed in test");
        let bottom10 = tail(&df, 10).expect("operation should succeed in test");
        assert_eq!(bottom10.height(), 2); // Returns all available rows
    }

    #[test]
    fn test_tail_zero() {
        let df = from_columns(vec![("age", vec![25, 30, 35])])
            .expect("operation should succeed in test");
        let bottom0 = tail(&df, 0).expect("operation should succeed in test");
        assert_eq!(bottom0.height(), 0);
    }

    #[test]
    fn test_shape_basic() {
        let df = from_columns(vec![("age", vec![25, 30, 35])])
            .expect("operation should succeed in test");
        let (rows, cols) = shape(&df).expect("operation should succeed in test");
        assert_eq!(rows, 3);
        assert_eq!(cols, 1);
    }

    #[test]
    fn test_shape_multiple_columns() {
        let df = from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])])
            .expect("operation should succeed in test");
        let (rows, cols) = shape(&df).expect("operation should succeed in test");
        assert_eq!(rows, 2);
        assert_eq!(cols, 2);
    }

    #[test]
    fn test_shape_empty() {
        let df = from_columns(vec![]).expect("operation should succeed in test");
        let (rows, cols) = shape(&df).expect("operation should succeed in test");
        assert_eq!(rows, 0);
        assert_eq!(cols, 0);
    }

    #[test]
    fn test_columns_basic() {
        let df =
            from_columns(vec![("age", vec![25, 30])]).expect("operation should succeed in test");
        let names = columns(&df).expect("operation should succeed in test");
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "age");
    }

    #[test]
    fn test_columns_multiple() {
        let df = from_columns(vec![
            ("age", vec![25]),
            ("score", vec![95]),
            ("grade", vec![1]),
        ])
        .expect("operation should succeed in test");
        let names = columns(&df).expect("operation should succeed in test");
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"age".to_string()));
        assert!(names.contains(&"score".to_string()));
        assert!(names.contains(&"grade".to_string()));
    }

    #[test]
    fn test_columns_empty() {
        let df = from_columns(vec![]).expect("operation should succeed in test");
        let names = columns(&df).expect("operation should succeed in test");
        assert_eq!(names.len(), 0);
    }

    #[test]
    fn test_row_count_basic() {
        let df = from_columns(vec![("age", vec![25, 30, 35])])
            .expect("operation should succeed in test");
        let count = row_count(&df).expect("operation should succeed in test");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_row_count_empty() {
        let df = from_columns(vec![("age", vec![])]).expect("operation should succeed in test");
        let count = row_count(&df).expect("operation should succeed in test");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_row_count_multiple_columns() {
        let df = from_columns(vec![("age", vec![25, 30]), ("score", vec![95, 87])])
            .expect("operation should succeed in test");
        let count = row_count(&df).expect("operation should succeed in test");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_dataframe_workflow() {
        // Complete workflow: create, select, head, tail, shape, columns, row_count
        let df = from_columns(vec![
            ("age", vec![25, 30, 35, 40, 45]),
            ("score", vec![95, 87, 92, 88, 90]),
        ])
        .expect("operation should succeed in test");

        // Select
        let subset = select(&df, &["age"]).expect("operation should succeed in test");
        assert_eq!(subset.width(), 1);

        // Head
        let top2 = head(&df, 2).expect("operation should succeed in test");
        assert_eq!(top2.height(), 2);

        // Tail
        let bottom2 = tail(&df, 2).expect("operation should succeed in test");
        assert_eq!(bottom2.height(), 2);

        // Shape
        let (rows, cols) = shape(&df).expect("operation should succeed in test");
        assert_eq!(rows, 5);
        assert_eq!(cols, 2);

        // Columns
        let names = columns(&df).expect("operation should succeed in test");
        assert_eq!(names.len(), 2);

        // Row count
        let count = row_count(&df).expect("operation should succeed in test");
        assert_eq!(count, 5);
    }
}
