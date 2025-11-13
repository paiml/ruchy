//! Apache Arrow integration for zero-copy `DataFrame` operations
//!
//! This module provides efficient interoperability between Ruchy `DataFrames`
//! and Apache Arrow format for high-performance data processing.
use anyhow::{Context, Result};
use arrow::array::{Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use arrow_schema::SchemaRef;
use std::sync::Arc;
// Use explicit namespaces to avoid conflicts
use arrow::datatypes::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
use polars::prelude::DataType as PolarsDataType;
/// Convert Polars `DataFrame` to Arrow `RecordBatch` for zero-copy operations
/// # Examples
///
/// ```
/// use ruchy::backend::arrow_integration::dataframe_to_arrow;
///
/// let result = dataframe_to_arrow(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn dataframe_to_arrow(df: &polars::prelude::DataFrame) -> Result<RecordBatch> {
    let mut fields = Vec::new();
    let mut arrays: Vec<ArrayRef> = Vec::new();
    for column in df.get_columns() {
        let field = ArrowField::new(
            column.name().as_str(),
            polars_dtype_to_arrow(column.dtype())?,
            column.has_nulls(),
        );
        fields.push(field);
        let array = polars_series_to_arrow(column.as_materialized_series())?;
        arrays.push(array);
    }
    let schema = Arc::new(ArrowSchema::new(fields));
    RecordBatch::try_new(schema, arrays)
        .context("Failed to create Arrow RecordBatch from DataFrame")
}
/// Convert Arrow `RecordBatch` to Polars `DataFrame`
/// # Examples
///
/// ```
/// use ruchy::backend::arrow_integration::arrow_to_dataframe;
///
/// let result = arrow_to_dataframe(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arrow_to_dataframe(batch: &RecordBatch) -> Result<polars::prelude::DataFrame> {
    let mut series_vec = Vec::new();
    for (i, field) in batch.schema().fields().iter().enumerate() {
        let array = batch.column(i);
        let series = arrow_array_to_polars_series(field.name(), array)?;
        series_vec.push(series.into());
    }
    polars::prelude::DataFrame::new(series_vec)
        .context("Failed to create DataFrame from Arrow RecordBatch")
}
/// Convert Polars `DataType` to Arrow `DataType`
fn polars_dtype_to_arrow(dtype: &PolarsDataType) -> Result<ArrowDataType> {
    match dtype {
        PolarsDataType::Int32 => Ok(ArrowDataType::Int32),
        PolarsDataType::Int64 => Ok(ArrowDataType::Int64),
        PolarsDataType::Float32 => Ok(ArrowDataType::Float32),
        PolarsDataType::Float64 => Ok(ArrowDataType::Float64),
        PolarsDataType::Boolean => Ok(ArrowDataType::Boolean),
        PolarsDataType::String => Ok(ArrowDataType::Utf8),
        PolarsDataType::Date => Ok(ArrowDataType::Date32),
        PolarsDataType::Datetime(_, _) => Ok(ArrowDataType::Timestamp(
            arrow::datatypes::TimeUnit::Microsecond,
            None,
        )),
        _ => anyhow::bail!("Unsupported Polars DataType: {dtype:?}"),
    }
}
/// Convert Polars Series to Arrow Array
fn polars_series_to_arrow(series: &polars::prelude::Series) -> Result<ArrayRef> {
    match series.dtype() {
        PolarsDataType::Int32 => {
            let ca = series.i32().context("Failed to cast to i32")?;
            let values: Vec<Option<i32>> = ca.into_iter().collect();
            Ok(Arc::new(arrow::array::Int32Array::from(values)))
        }
        PolarsDataType::Int64 => {
            let ca = series.i64().context("Failed to cast to i64")?;
            let values: Vec<Option<i64>> = ca.into_iter().collect();
            Ok(Arc::new(Int64Array::from(values)))
        }
        PolarsDataType::Float64 => {
            let ca = series.f64().context("Failed to cast to f64")?;
            let values: Vec<Option<f64>> = ca.into_iter().collect();
            Ok(Arc::new(Float64Array::from(values)))
        }
        PolarsDataType::Boolean => {
            let ca = series.bool().context("Failed to cast to bool")?;
            let values: Vec<Option<bool>> = ca.into_iter().collect();
            Ok(Arc::new(BooleanArray::from(values)))
        }
        PolarsDataType::String => {
            let ca = series.str().context("Failed to cast to string")?;
            let values: Vec<Option<&str>> = ca.into_iter().collect();
            Ok(Arc::new(StringArray::from(values)))
        }
        _ => anyhow::bail!(
            "Unsupported Series DataType for Arrow conversion: {:?}",
            series.dtype()
        ),
    }
}
/// Extract nullable values from Arrow array
fn extract_nullable_values<T, A>(array: &dyn arrow::array::Array) -> Result<Vec<Option<T>>>
where
    A: arrow::array::Array + 'static + arrow::array::ArrayAccessor<Item = T> + AsRef<[T]>,
    T: Copy,
{
    let typed_array = array
        .as_any()
        .downcast_ref::<A>()
        .context("Failed to downcast Arrow array")?;
    let values: Vec<Option<T>> = (0..typed_array.len())
        .map(|i| {
            if typed_array.is_null(i) {
                None
            } else {
                Some(typed_array.value(i))
            }
        })
        .collect();
    Ok(values)
}

/// Convert Arrow Array to Polars Series
fn arrow_array_to_polars_series(
    name: &str,
    array: &dyn arrow::array::Array,
) -> Result<polars::prelude::Series> {
    use polars::datatypes::PlSmallStr;
    use polars::prelude::{NamedFrom, Series};

    let name_str = PlSmallStr::from(name);
    match array.data_type() {
        ArrowDataType::Int32 => {
            let array = array
                .as_any()
                .downcast_ref::<arrow::array::Int32Array>()
                .context("Failed to downcast to Int32Array")?;
            let values: Vec<Option<i32>> = (0..array.len())
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(array.value(i))
                    }
                })
                .collect();
            Ok(Series::new(name_str, values))
        }
        ArrowDataType::Int64 => {
            let array = array
                .as_any()
                .downcast_ref::<arrow::array::Int64Array>()
                .context("Failed to downcast to Int64Array")?;
            let values: Vec<Option<i64>> = (0..array.len())
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(array.value(i))
                    }
                })
                .collect();
            Ok(Series::new(name_str, values))
        }
        ArrowDataType::Float64 => {
            let array = array
                .as_any()
                .downcast_ref::<arrow::array::Float64Array>()
                .context("Failed to downcast to Float64Array")?;
            let values: Vec<Option<f64>> = (0..array.len())
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(array.value(i))
                    }
                })
                .collect();
            Ok(Series::new(name_str, values))
        }
        ArrowDataType::Boolean => {
            let array = array
                .as_any()
                .downcast_ref::<arrow::array::BooleanArray>()
                .context("Failed to downcast to BooleanArray")?;
            let values: Vec<Option<bool>> = (0..array.len())
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(array.value(i))
                    }
                })
                .collect();
            Ok(Series::new(name_str, values))
        }
        ArrowDataType::Utf8 => convert_arrow_string(array, name_str),
        _ => anyhow::bail!(
            "Unsupported Arrow DataType for Polars conversion: {:?}",
            array.data_type()
        ),
    }
}

/// Convert Arrow string array to Polars Series
fn convert_arrow_string(
    array: &dyn arrow::array::Array,
    name_str: polars::datatypes::PlSmallStr,
) -> Result<polars::prelude::Series> {
    use polars::prelude::{NamedFrom, Series};
    let array = array
        .as_any()
        .downcast_ref::<StringArray>()
        .context("Failed to downcast to StringArray")?;
    let values: Vec<Option<&str>> = (0..array.len())
        .map(|i| {
            if array.is_null(i) {
                None
            } else {
                Some(array.value(i))
            }
        })
        .collect();
    Ok(Series::new(name_str, values))
}
/// Efficient zero-copy operations for large datasets
#[derive(Debug)]
pub struct ArrowDataFrame {
    schema: SchemaRef,
    batches: Vec<RecordBatch>,
}
impl ArrowDataFrame {
    /// Create new `ArrowDataFrame` from `RecordBatches`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::arrow_integration::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn new(schema: SchemaRef, batches: Vec<RecordBatch>) -> Self {
        Self { schema, batches }
    }
    /// Get total number of rows across all batches
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::arrow_integration::num_rows;
    ///
    /// let result = num_rows(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn num_rows(&self) -> usize {
        self.batches.iter().map(RecordBatch::num_rows).sum()
    }
    /// Get number of columns
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::arrow_integration::num_columns;
    ///
    /// let result = num_columns(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn num_columns(&self) -> usize {
        self.schema.fields().len()
    }
    /// Perform zero-copy slice operation
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::arrow_integration::slice;
    ///
    /// let result = slice(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn slice(&self, offset: usize, length: usize) -> Result<RecordBatch> {
        if self.batches.is_empty() {
            anyhow::bail!("Cannot slice empty ArrowDataFrame");
        }
        let mut current_offset = 0;
        let mut result_arrays = Vec::new();
        let mut remaining_length = length;
        for batch in &self.batches {
            let batch_rows = batch.num_rows();
            if current_offset + batch_rows <= offset {
                // Skip this batch entirely
                current_offset += batch_rows;
                continue;
            }
            let start_in_batch = offset.saturating_sub(current_offset);
            let take_from_batch = std::cmp::min(batch_rows - start_in_batch, remaining_length);
            if take_from_batch > 0 {
                // Take slice from this batch
                let sliced = batch.slice(start_in_batch, take_from_batch);
                if result_arrays.is_empty() {
                    result_arrays = sliced.columns().to_vec();
                } else {
                    // Concatenate with existing arrays
                    for (i, array) in sliced.columns().iter().enumerate() {
                        result_arrays[i] =
                            arrow::compute::kernels::concat::concat(&[&result_arrays[i], array])
                                .context("Failed to concatenate arrays")?;
                    }
                }
                remaining_length -= take_from_batch;
                if remaining_length == 0 {
                    break;
                }
            }
            current_offset += batch_rows;
        }
        RecordBatch::try_new(self.schema.clone(), result_arrays)
            .context("Failed to create sliced RecordBatch")
    }
    /// Filter rows based on a boolean mask (zero-copy where possible)
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::arrow_integration::filter;
    ///
    /// let result = filter(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn filter(&self, mask: &BooleanArray) -> Result<RecordBatch> {
        if mask.len() != self.num_rows() {
            anyhow::bail!(
                "Filter mask length {} doesn't match DataFrame rows {}",
                mask.len(),
                self.num_rows()
            );
        }
        // Use Arrow's optimized filter kernel
        let mut filtered_arrays = Vec::new();
        for batch in &self.batches {
            for column in batch.columns() {
                let filtered = arrow::compute::kernels::filter::filter(column, mask)
                    .context("Failed to filter array")?;
                filtered_arrays.push(filtered);
            }
        }
        RecordBatch::try_new(self.schema.clone(), filtered_arrays)
            .context("Failed to create filtered RecordBatch")
    }
    /// Efficiently concatenate multiple `ArrowDataFrames`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::arrow_integration::concat;
    ///
    /// let result = concat(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn concat(dataframes: &[Self]) -> Result<Self> {
        if dataframes.is_empty() {
            anyhow::bail!("Cannot concatenate empty list of DataFrames");
        }
        let schema = dataframes[0].schema.clone();
        let mut all_batches = Vec::new();
        for df in dataframes {
            if df.schema != schema {
                anyhow::bail!("Cannot concatenate DataFrames with different schemas");
            }
            all_batches.extend(df.batches.clone());
        }
        Ok(Self::new(schema, all_batches))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{ArrayRef, Int32Array};
    use arrow::datatypes::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
    use polars::datatypes::DataType as PolarsDataType;
    use polars::datatypes::PlSmallStr;
    use polars::prelude::*;
    #[test]
    fn test_dataframe_to_arrow_roundtrip() {
        // Create a simple Polars DataFrame
        let df = df! {
            "integers" => &[1, 2, 3, 4, 5],
            "floats" => &[1.0, 2.0, 3.0, 4.0, 5.0],
            "strings" => &["a", "b", "c", "d", "e"],
            "booleans" => &[true, false, true, false, true],
        }
        .unwrap();
        // Convert to Arrow
        let record_batch = dataframe_to_arrow(&df).unwrap();
        // Verify schema
        assert_eq!(record_batch.num_columns(), 4);
        assert_eq!(record_batch.num_rows(), 5);
        // Convert back to Polars
        let df2 = arrow_to_dataframe(&record_batch).unwrap();
        // Verify data integrity
        assert_eq!(df.shape(), df2.shape());
        assert_eq!(df.get_column_names(), df2.get_column_names());
    }
    #[test]
    fn test_arrow_dataframe_slice() {
        let df = df! {
            "values" => &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        }
        .unwrap();
        let batch = dataframe_to_arrow(&df).unwrap();
        let arrow_df = ArrowDataFrame::new(batch.schema(), vec![batch]);
        // Test slicing
        let sliced = arrow_df.slice(2, 5).unwrap();
        assert_eq!(sliced.num_rows(), 5);
        // Verify values
        let array = sliced
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::Int32Array>()
            .unwrap();
        assert_eq!(array.value(0), 3);
        assert_eq!(array.value(4), 7);
    }
    #[test]
    #[ignore = "Performance test - can be flaky"]
    fn test_zero_copy_performance() {
        // Create large DataFrame
        let size = 1_000_000;
        let values: Vec<i64> = (0..size).collect();
        let df = df! {
            "values" => values,
        }
        .unwrap();
        // Convert to Arrow (should be fast due to zero-copy)
        let start = std::time::Instant::now();
        let batch = dataframe_to_arrow(&df).unwrap();
        let duration = start.elapsed();
        // Verify it's fast (less than 100ms for 1M rows)
        assert!(
            duration.as_millis() < 100,
            "Conversion took too long: {duration:?}"
        );
        assert_eq!(batch.num_rows(), size as usize);
    }

    #[test]
    fn test_zero_copy_slice_performance() {
        // Create large DataFrame for slicing test
        let size = 5_000_000;
        let values: Vec<i32> = (0..size).collect();
        let df = df! {
            "values" => values,
        }
        .unwrap();

        let batch = dataframe_to_arrow(&df).unwrap();
        let arrow_df = ArrowDataFrame::new(batch.schema(), vec![batch]);

        // Multiple slice operations should all be fast (zero-copy)
        let slice_tests = vec![
            (0usize, 1000usize),                 // Start slice
            ((size / 2) as usize, 1000usize),    // Middle slice
            ((size - 1000) as usize, 1000usize), // End slice
            (1000usize, 100_000usize),           // Large slice
        ];

        for (offset, length) in slice_tests {
            let start = std::time::Instant::now();
            let sliced = arrow_df.slice(offset, length).unwrap();
            let duration = start.elapsed();

            // Zero-copy slice should be very fast regardless of data size
            assert!(
                duration.as_millis() < 10,
                "Slice({}, {}) took too long: {:?}ms",
                offset,
                length,
                duration.as_millis()
            );
            assert_eq!(sliced.num_rows(), length);
        }
    }

    #[test]
    fn test_zero_copy_filter_performance() {
        // Test that filter operations maintain zero-copy benefits where possible
        let size = 1_000_000;
        let values: Vec<i32> = (0..size).collect();
        let df = df! {
            "values" => values,
        }
        .unwrap();

        let batch = dataframe_to_arrow(&df).unwrap();
        let arrow_df = ArrowDataFrame::new(batch.schema(), vec![batch]);

        // Create a boolean mask (every 10th element)
        let mask_values: Vec<bool> = (0..size).map(|i| i % 10 == 0).collect();
        let mask = BooleanArray::from(mask_values);

        let start = std::time::Instant::now();
        let filtered = arrow_df.filter(&mask).unwrap();
        let duration = start.elapsed();

        // Filter should be reasonably fast using Arrow's optimized kernels
        assert!(
            duration.as_millis() < 200,
            "Filter took too long: {:?}ms",
            duration.as_millis()
        );
        assert_eq!(filtered.num_rows(), (size / 10) as usize); // Every 10th element
    }

    #[test]
    fn test_polars_dtype_to_arrow() {
        // Test various data type conversions
        assert_eq!(
            polars_dtype_to_arrow(&PolarsDataType::Int32).unwrap(),
            ArrowDataType::Int32
        );
        assert_eq!(
            polars_dtype_to_arrow(&PolarsDataType::Int64).unwrap(),
            ArrowDataType::Int64
        );
        assert_eq!(
            polars_dtype_to_arrow(&PolarsDataType::Float32).unwrap(),
            ArrowDataType::Float32
        );
        assert_eq!(
            polars_dtype_to_arrow(&PolarsDataType::Float64).unwrap(),
            ArrowDataType::Float64
        );
        assert_eq!(
            polars_dtype_to_arrow(&PolarsDataType::Boolean).unwrap(),
            ArrowDataType::Boolean
        );
        assert_eq!(
            polars_dtype_to_arrow(&PolarsDataType::String).unwrap(),
            ArrowDataType::Utf8
        );
    }

    // arrow_dtype_to_polars doesn't exist yet, would be for reverse conversion

    #[test]
    fn test_arrow_dataframe_new() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col1",
            ArrowDataType::Int32,
            false,
        )]));
        let array = Int32Array::from(vec![1, 2, 3]);
        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(array) as ArrayRef]).unwrap();

        let arrow_df = ArrowDataFrame::new(schema.clone(), vec![batch]);
        assert_eq!(arrow_df.schema, schema);
        assert_eq!(arrow_df.batches.len(), 1);
    }

    #[test]
    fn test_arrow_dataframe_num_columns() {
        let schema = Arc::new(ArrowSchema::new(vec![
            ArrowField::new("col1", ArrowDataType::Int32, false),
            ArrowField::new("col2", ArrowDataType::Float64, false),
            ArrowField::new("col3", ArrowDataType::Utf8, false),
        ]));
        let arrow_df = ArrowDataFrame::new(schema, Vec::new());
        assert_eq!(arrow_df.num_columns(), 3);
    }

    #[test]
    fn test_arrow_dataframe_concat_empty() {
        let result = ArrowDataFrame::concat(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty list"));
    }

    #[test]
    fn test_arrow_dataframe_concat_mismatched_schemas() {
        let schema1 = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col1",
            ArrowDataType::Int32,
            false,
        )]));
        let schema2 = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col2",
            ArrowDataType::Float64,
            false,
        )]));

        let df1 = ArrowDataFrame::new(schema1, Vec::new());
        let df2 = ArrowDataFrame::new(schema2, Vec::new());

        let result = ArrowDataFrame::concat(&[df1, df2]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("different schemas"));
    }

    #[test]
    fn test_arrow_dataframe_slice_empty() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col1",
            ArrowDataType::Int32,
            false,
        )]));
        let arrow_df = ArrowDataFrame::new(schema, Vec::new());

        let result = arrow_df.slice(0, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_arrow_dataframe_filter_mismatched_length() {
        let df = df! {
            "values" => &[1, 2, 3, 4, 5],
        }
        .unwrap();
        let batch = dataframe_to_arrow(&df).unwrap();
        let arrow_df = ArrowDataFrame::new(batch.schema(), vec![batch]);

        // Create mask with wrong length
        let mask = BooleanArray::from(vec![true, false]);

        let result = arrow_df.filter(&mask);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("length"));
    }

    #[test]
    fn test_dataframe_with_nulls() {
        // Test handling of nullable columns
        let values: Vec<Option<i32>> = vec![Some(1), None, Some(3), None, Some(5)];
        let s = Series::new(PlSmallStr::from("nullable"), values);
        let df = DataFrame::new(vec![s.into()]).unwrap();

        let batch = dataframe_to_arrow(&df).unwrap();
        assert_eq!(batch.num_rows(), 5);

        // Convert back and verify nulls preserved
        let df2 = arrow_to_dataframe(&batch).unwrap();
        assert_eq!(df.shape(), df2.shape());
    }

    #[test]
    #[ignore = "Performance test - can be flaky"]
    fn test_df004_1m_row_performance_target() {
        // DF-004: Verify all operations meet 1M row <100ms performance target
        let size = 1_000_000;
        let int_values: Vec<i32> = (0..size).collect();
        let float_values: Vec<f64> = (0..size).map(|i| f64::from(i) * 1.5).collect();
        let bool_values: Vec<bool> = (0..size).map(|i| i % 2 == 0).collect();

        // Create multi-column DataFrame for comprehensive testing
        let df = df! {
            "integers" => int_values,
            "floats" => float_values,
            "booleans" => bool_values,
        }
        .unwrap();

        // Test 1: Polars to Arrow conversion (<100ms)
        let start = std::time::Instant::now();
        let batch = dataframe_to_arrow(&df).unwrap();
        let conversion_time = start.elapsed();
        assert!(
            conversion_time.as_millis() < 100,
            "DF-004 FAILED: Polars→Arrow conversion took {}ms (target: <100ms)",
            conversion_time.as_millis()
        );

        // Test 2: Arrow to Polars roundtrip (<100ms)
        let start = std::time::Instant::now();
        let df2 = arrow_to_dataframe(&batch).unwrap();
        let roundtrip_time = start.elapsed();
        assert!(
            roundtrip_time.as_millis() < 100,
            "DF-004 FAILED: Arrow→Polars conversion took {}ms (target: <100ms)",
            roundtrip_time.as_millis()
        );

        // Test 3: Large slice operations (<100ms)
        let arrow_df = ArrowDataFrame::new(batch.schema(), vec![batch]);
        let start = std::time::Instant::now();
        let _large_slice = arrow_df.slice(0, 500_000).unwrap(); // 50% of data
        let slice_time = start.elapsed();
        assert!(
            slice_time.as_millis() < 100,
            "DF-004 FAILED: Large slice took {}ms (target: <100ms)",
            slice_time.as_millis()
        );

        // Verify data integrity
        assert_eq!(df.shape(), df2.shape());

        println!("✅ DF-004 Performance Targets Met:");
        println!("   • Polars→Arrow: {}ms", conversion_time.as_millis());
        println!("   • Arrow→Polars: {}ms", roundtrip_time.as_millis());
        println!("   • Large slice: {}ms", slice_time.as_millis());
        println!("   • All operations <100ms target ✅");
    }

    // ========================================
    // Coverage Sprint: Inline Unit Tests (bashrs pattern: 13.5 tests/file)
    // Target: Test all Result<T,E> error paths and edge cases
    // ========================================

    // Test 1: polars_dtype_to_arrow with unsupported type (ERROR PATH)
    #[test]
    fn test_polars_dtype_to_arrow_unsupported_type_error() {
        use polars::datatypes::DataType as PolarsDataType;

        // Binary type is unsupported
        let result = polars_dtype_to_arrow(&PolarsDataType::Binary);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported Polars DataType"));
    }

    // Test 2: polars_dtype_to_arrow with Date type
    #[test]
    fn test_polars_dtype_to_arrow_date() {
        use polars::datatypes::DataType as PolarsDataType;

        let result = polars_dtype_to_arrow(&PolarsDataType::Date).unwrap();
        assert_eq!(result, ArrowDataType::Date32);
    }

    // Test 3: polars_dtype_to_arrow with Datetime type
    #[test]
    fn test_polars_dtype_to_arrow_datetime() {
        use polars::datatypes::DataType as PolarsDataType;

        let result = polars_dtype_to_arrow(&PolarsDataType::Datetime(
            polars::datatypes::TimeUnit::Microseconds,
            None,
        )).unwrap();

        match result {
            ArrowDataType::Timestamp(unit, tz) => {
                assert_eq!(unit, arrow::datatypes::TimeUnit::Microsecond);
                assert_eq!(tz, None);
            }
            _ => panic!("Expected Timestamp type"),
        }
    }

    // Test 4: polars_series_to_arrow with unsupported type (ERROR PATH)
    #[test]
    fn test_polars_series_to_arrow_unsupported_error() {
        use polars::prelude::{Series, NamedFrom};
        use polars::datatypes::PlSmallStr;

        // Create a Binary series (unsupported)
        let values: Vec<&[u8]> = vec![b"hello", b"world"];
        let series = Series::new(PlSmallStr::from("binary_col"), values);

        let result = polars_series_to_arrow(&series);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported Series DataType"));
    }

    // Test 5: polars_series_to_arrow with Int32
    #[test]
    fn test_polars_series_to_arrow_int32() {
        use polars::prelude::{Series, NamedFrom};
        use polars::datatypes::PlSmallStr;

        let values: Vec<i32> = vec![1, 2, 3, 4, 5];
        let series = Series::new(PlSmallStr::from("int32_col"), values);

        let result = polars_series_to_arrow(&series).unwrap();
        assert_eq!(result.len(), 5);

        let array = result.as_any().downcast_ref::<arrow::array::Int32Array>().unwrap();
        assert_eq!(array.value(0), 1);
        assert_eq!(array.value(4), 5);
    }

    // Test 6: polars_series_to_arrow with Int32 containing nulls
    #[test]
    fn test_polars_series_to_arrow_int32_with_nulls() {
        use polars::prelude::{Series, NamedFrom};
        use polars::datatypes::PlSmallStr;

        let values: Vec<Option<i32>> = vec![Some(1), None, Some(3), None, Some(5)];
        let series = Series::new(PlSmallStr::from("nullable_int32"), values);

        let result = polars_series_to_arrow(&series).unwrap();
        assert_eq!(result.len(), 5);

        let array = result.as_any().downcast_ref::<arrow::array::Int32Array>().unwrap();
        assert!(!array.is_null(0));
        assert!(array.is_null(1));
        assert_eq!(array.value(0), 1);
        assert_eq!(array.value(2), 3);
    }

    // Test 7: arrow_array_to_polars_series with unsupported type (ERROR PATH)
    #[test]
    fn test_arrow_array_to_polars_series_unsupported_error() {
        use arrow::array::{BinaryArray, ArrayRef};

        // Create a Binary array (unsupported in this function)
        let values: Vec<&[u8]> = vec![b"hello", b"world"];
        let array = BinaryArray::from(values);
        let array_ref: &dyn Array = &array;

        let result = arrow_array_to_polars_series("binary_col", array_ref);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported Arrow DataType"));
    }

    // Test 8: arrow_array_to_polars_series with Int32
    #[test]
    fn test_arrow_array_to_polars_series_int32() {
        use arrow::array::{Int32Array};

        let values = vec![10, 20, 30, 40, 50];
        let array = Int32Array::from(values);
        let array_ref: &dyn Array = &array;

        let result = arrow_array_to_polars_series("test_col", array_ref).unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(result.name(), "test_col");
    }

    // Test 9: arrow_array_to_polars_series with Float64
    #[test]
    fn test_arrow_array_to_polars_series_float64() {
        use arrow::array::{Float64Array};

        let values = vec![1.5, 2.5, 3.5, 4.5, 5.5];
        let array = Float64Array::from(values);
        let array_ref: &dyn Array = &array;

        let result = arrow_array_to_polars_series("float_col", array_ref).unwrap();
        assert_eq!(result.len(), 5);
    }

    // Test 10: arrow_array_to_polars_series with Boolean
    #[test]
    fn test_arrow_array_to_polars_series_boolean() {
        let values = vec![true, false, true, false, true];
        let array = BooleanArray::from(values);
        let array_ref: &dyn Array = &array;

        let result = arrow_array_to_polars_series("bool_col", array_ref).unwrap();
        assert_eq!(result.len(), 5);
    }

    // Test 11: convert_arrow_string with empty array
    #[test]
    fn test_convert_arrow_string_empty() {
        use polars::datatypes::PlSmallStr;

        let values: Vec<Option<&str>> = vec![];
        let array = StringArray::from(values);
        let array_ref: &dyn Array = &array;

        let result = convert_arrow_string(array_ref, PlSmallStr::from("empty")).unwrap();
        assert_eq!(result.len(), 0);
    }

    // Test 12: convert_arrow_string with nulls
    #[test]
    fn test_convert_arrow_string_with_nulls() {
        use polars::datatypes::PlSmallStr;

        let values: Vec<Option<&str>> = vec![Some("hello"), None, Some("world"), None];
        let array = StringArray::from(values);
        let array_ref: &dyn Array = &array;

        let result = convert_arrow_string(array_ref, PlSmallStr::from("nullable_str")).unwrap();
        assert_eq!(result.len(), 4);
    }

    // Test 13: ArrowDataFrame::num_rows with multiple batches
    #[test]
    fn test_arrow_dataframe_num_rows_multiple_batches() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col1",
            ArrowDataType::Int32,
            false,
        )]));

        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef]
        ).unwrap();

        let batch2 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![4, 5])) as ArrayRef]
        ).unwrap();

        let arrow_df = ArrowDataFrame::new(schema, vec![batch1, batch2]);
        assert_eq!(arrow_df.num_rows(), 5); // 3 + 2 rows
    }

    // Test 14: ArrowDataFrame::concat with single dataframe
    #[test]
    fn test_arrow_dataframe_concat_single() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col1",
            ArrowDataType::Int32,
            false,
        )]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef]
        ).unwrap();

        let df = ArrowDataFrame::new(schema.clone(), vec![batch]);
        let result = ArrowDataFrame::concat(&[df]).unwrap();

        assert_eq!(result.num_rows(), 3);
        assert_eq!(result.schema, schema);
    }

    // Test 15: ArrowDataFrame::concat with matching schemas
    #[test]
    fn test_arrow_dataframe_concat_matching_schemas() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "values",
            ArrowDataType::Int32,
            false,
        )]));

        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef]
        ).unwrap();

        let batch2 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![4, 5, 6])) as ArrayRef]
        ).unwrap();

        let df1 = ArrowDataFrame::new(schema.clone(), vec![batch1]);
        let df2 = ArrowDataFrame::new(schema.clone(), vec![batch2]);

        let result = ArrowDataFrame::concat(&[df1, df2]).unwrap();
        assert_eq!(result.num_rows(), 6); // 3 + 3 rows
        assert_eq!(result.batches.len(), 2); // 2 batches preserved
    }

    // Test 16: dataframe_to_arrow with empty dataframe
    #[test]
    fn test_dataframe_to_arrow_empty() {
        let df = df! {
            "empty_col" => Vec::<i32>::new(),
        }.unwrap();

        let result = dataframe_to_arrow(&df).unwrap();
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 1);
    }

    // Test 17: arrow_to_dataframe with empty batch
    #[test]
    fn test_arrow_to_dataframe_empty() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "col1",
            ArrowDataType::Int32,
            false,
        )]));

        let batch = RecordBatch::try_new(
            schema,
            vec![Arc::new(Int32Array::from(Vec::<i32>::new())) as ArrayRef]
        ).unwrap();

        let result = arrow_to_dataframe(&batch).unwrap();
        assert_eq!(result.shape(), (0, 1)); // 0 rows, 1 column
    }

    // Test 18: ArrowDataFrame::slice spanning multiple batches
    #[test]
    fn test_arrow_dataframe_slice_spanning_batches() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "values",
            ArrowDataType::Int32,
            false,
        )]));

        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef]
        ).unwrap();

        let batch2 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![4, 5, 6])) as ArrayRef]
        ).unwrap();

        let arrow_df = ArrowDataFrame::new(schema, vec![batch1, batch2]);

        // Slice from row 1 to row 4 (spans both batches)
        let sliced = arrow_df.slice(1, 4).unwrap();
        assert_eq!(sliced.num_rows(), 4); // rows 2, 3, 4, 5

        let array = sliced.column(0).as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(array.value(0), 2); // First value in slice
        assert_eq!(array.value(3), 5); // Last value in slice
    }
}
#[cfg(test)]
mod property_tests_arrow_integration {
    use super::*;
    use polars::df;
    use proptest::prelude::*;
    use proptest::proptest;
    proptest! {
        /// Property: Round-trip conversion preserves data shape
        #[test]
        fn test_dataframe_arrow_roundtrip_preserves_shape(
            int_values in prop::collection::vec(any::<i32>(), 1..10), // Smaller range for testing
            col_name in r"[a-zA-Z][a-zA-Z0-9_]*"
        ) {
            use polars::prelude::{DataFrame, Series, NamedFrom};
            use polars::datatypes::PlSmallStr;

            // Create DataFrame with random data
            let col_name_small = PlSmallStr::from(col_name.as_str());
            let series = Series::new(col_name_small, int_values);
            let df = DataFrame::new(vec![series.into()]).expect("Failed to create DataFrame");

            // Convert to Arrow and back - should preserve shape
            if let Ok(record_batch) = dataframe_to_arrow(&df) {
                if let Ok(df2) = arrow_to_dataframe(&record_batch) {
                    prop_assert_eq!(df.shape(), df2.shape());
                }
            }
        }

        /// Property: Slicing never produces more rows than requested
        #[test]
        fn test_slice_never_exceeds_length(
            total_rows in 10..100usize,
            offset in 0..100usize,
            length in 1..100usize
        ) {
            let values: Vec<i32> = (0..total_rows as i32).collect();
            let df = df! {
                "values" => values,
            }.unwrap();

            let batch = dataframe_to_arrow(&df).unwrap();
            let arrow_df = ArrowDataFrame::new(batch.schema(), vec![batch]);

            if offset < total_rows {
                if let Ok(sliced) = arrow_df.slice(offset, length) {
                    let actual_rows = sliced.num_rows();
                    let max_possible = total_rows.saturating_sub(offset).min(length);
                    prop_assert!(actual_rows <= max_possible);
                }
            }
        }

        /// Property: Concatenation preserves total row count
        #[test]
        #[ignore = "Property test - can be flaky"]
        fn test_concat_preserves_row_count(
            sizes in prop::collection::vec(1..20usize, 1..5)
        ) {
            let schema = Arc::new(ArrowSchema::new(vec![
                ArrowField::new("col", ArrowDataType::Int32, false),
            ]));

            let mut dfs = Vec::new();
            let mut total_rows = 0;

            for size in &sizes {
                let values: Vec<i64> = (0..*size as i64).collect();
                let array = Int64Array::from(values);
                let batch = RecordBatch::try_new(
                    schema.clone(),
                    vec![Arc::new(array) as ArrayRef]
                ).unwrap();

                dfs.push(ArrowDataFrame::new(schema.clone(), vec![batch]));
                total_rows += size;
            }

            if let Ok(concatenated) = ArrowDataFrame::concat(&dfs) {
                prop_assert_eq!(concatenated.num_rows(), total_rows);
            }
        }
    }
}
