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
/// ]).unwrap();
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
/// let df = dataframe::read_csv("data.csv").unwrap();
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
/// let mut df = dataframe::from_columns(vec![("age", vec![25, 30])]).unwrap();
/// dataframe::write_csv(&mut df, "output.csv").unwrap();
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
/// ]).unwrap();
/// let subset = dataframe::select(&df, &["age"]).unwrap();
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
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35, 40])]).unwrap();
/// let top3 = dataframe::head(&df, 3).unwrap();
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
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35, 40])]).unwrap();
/// let bottom3 = dataframe::tail(&df, 3).unwrap();
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
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
/// let (rows, cols) = dataframe::shape(&df).unwrap();
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
/// ]).unwrap();
/// let names = dataframe::columns(&df).unwrap();
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
/// let df = dataframe::from_columns(vec![("age", vec![25, 30, 35])]).unwrap();
/// let count = dataframe::row_count(&df).unwrap();
/// # }
/// ```
pub fn row_count(df: &DataFrame) -> Result<usize, String> {
    Ok(df.height())
}
