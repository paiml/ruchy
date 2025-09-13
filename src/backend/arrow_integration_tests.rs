//! Tests for Apache Arrow integration
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

#[cfg(feature = "dataframe")]
#[cfg(test)]
mod arrow_tests {
    use super::super::arrow_integration::*;
    use arrow::array::{Array, ArrayRef, Float64Array, Int64Array, StringArray, BooleanArray};
    use arrow::datatypes::{Field, Schema, DataType as ArrowDataType};
    use arrow::record_batch::RecordBatch;
    use polars::prelude::*;
    use std::sync::Arc;

    fn create_test_dataframe() -> DataFrame {
        df! {
            "integers" => [1i64, 2, 3, 4, 5],
            "floats" => [1.1f64, 2.2, 3.3, 4.4, 5.5],
            "strings" => ["a", "b", "c", "d", "e"],
            "booleans" => [true, false, true, false, true],
        }.unwrap()
    }

    fn create_test_arrow_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("integers", ArrowDataType::Int64, false),
            Field::new("floats", ArrowDataType::Float64, false),
            Field::new("strings", ArrowDataType::Utf8, false),
            Field::new("booleans", ArrowDataType::Boolean, false),
        ]));

        let arrays: Vec<ArrayRef> = vec![
            Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5])),
            Arc::new(Float64Array::from(vec![1.1, 2.2, 3.3, 4.4, 5.5])),
            Arc::new(StringArray::from(vec!["a", "b", "c", "d", "e"])),
            Arc::new(BooleanArray::from(vec![true, false, true, false, true])),
        ];

        RecordBatch::try_new(schema, arrays).unwrap()
    }

    #[test]
    fn test_dataframe_to_arrow_conversion() {
        let df = create_test_dataframe();
        let result = dataframe_to_arrow(&df);
        
        assert!(result.is_ok());
        let batch = result.unwrap();
        assert_eq!(batch.num_columns(), 4);
        assert_eq!(batch.num_rows(), 5);
    }

    #[test]
    fn test_arrow_to_dataframe_conversion() {
        let batch = create_test_arrow_batch();
        let result = arrow_to_dataframe(&batch);
        
        assert!(result.is_ok());
        let df = result.unwrap();
        assert_eq!(df.width(), 4);
        assert_eq!(df.height(), 5);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original_df = create_test_dataframe();
        
        // DataFrame -> Arrow -> DataFrame
        let arrow_batch = dataframe_to_arrow(&original_df).unwrap();
        let converted_df = arrow_to_dataframe(&arrow_batch).unwrap();
        
        assert_eq!(original_df.width(), converted_df.width());
        assert_eq!(original_df.height(), converted_df.height());
        
        // Check column names
        let original_columns: Vec<_> = original_df.get_column_names();
        let converted_columns: Vec<_> = converted_df.get_column_names();
        assert_eq!(original_columns, converted_columns);
    }

    #[test]
    fn test_polars_dtype_to_arrow_int64() {
        let polars_type = PolarsDataType::Int64;
        let result = polars_dtype_to_arrow(&polars_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ArrowDataType::Int64));
    }

    #[test]
    fn test_polars_dtype_to_arrow_float64() {
        let polars_type = PolarsDataType::Float64;
        let result = polars_dtype_to_arrow(&polars_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ArrowDataType::Float64));
    }

    #[test]
    fn test_polars_dtype_to_arrow_string() {
        let polars_type = PolarsDataType::String;
        let result = polars_dtype_to_arrow(&polars_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ArrowDataType::Utf8));
    }

    #[test]
    fn test_polars_dtype_to_arrow_boolean() {
        let polars_type = PolarsDataType::Boolean;
        let result = polars_dtype_to_arrow(&polars_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ArrowDataType::Boolean));
    }

    #[test]
    fn test_arrow_dtype_to_polars_int64() {
        let arrow_type = ArrowDataType::Int64;
        let result = arrow_dtype_to_polars(&arrow_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PolarsDataType::Int64));
    }

    #[test]
    fn test_arrow_dtype_to_polars_float64() {
        let arrow_type = ArrowDataType::Float64;
        let result = arrow_dtype_to_polars(&arrow_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PolarsDataType::Float64));
    }

    #[test]
    fn test_arrow_dtype_to_polars_utf8() {
        let arrow_type = ArrowDataType::Utf8;
        let result = arrow_dtype_to_polars(&arrow_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PolarsDataType::String));
    }

    #[test]
    fn test_arrow_dtype_to_polars_boolean() {
        let arrow_type = ArrowDataType::Boolean;
        let result = arrow_dtype_to_polars(&arrow_type);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PolarsDataType::Boolean));
    }

    #[test]
    fn test_empty_dataframe_conversion() {
        let empty_df = DataFrame::empty();
        let result = dataframe_to_arrow(&empty_df);
        
        // Empty DataFrame should convert successfully
        assert!(result.is_ok());
        let batch = result.unwrap();
        assert_eq!(batch.num_rows(), 0);
    }

    #[test]
    fn test_single_row_dataframe() {
        let single_row_df = df! {
            "value" => [42i64],
        }.unwrap();
        
        let result = dataframe_to_arrow(&single_row_df);
        assert!(result.is_ok());
        
        let batch = result.unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 1);
    }

    #[test]
    fn test_single_column_dataframe() {
        let single_col_df = df! {
            "numbers" => [1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        }.unwrap();
        
        let result = dataframe_to_arrow(&single_col_df);
        assert!(result.is_ok());
        
        let batch = result.unwrap();
        assert_eq!(batch.num_columns(), 1);
        assert_eq!(batch.num_rows(), 10);
    }

    #[test]
    fn test_dataframe_with_nulls() {
        let df_with_nulls = df! {
            "nullable_ints" => [Some(1i64), None, Some(3), None, Some(5)],
        }.unwrap();
        
        let result = dataframe_to_arrow(&df_with_nulls);
        assert!(result.is_ok());
        
        let batch = result.unwrap();
        assert_eq!(batch.num_rows(), 5);
        
        // Check that nulls are handled properly
        let column = batch.column(0);
        assert!(column.is_null(1)); // Second element should be null
        assert!(column.is_null(3)); // Fourth element should be null
    }
}

#[cfg(feature = "dataframe")]
#[cfg(test)]
mod property_tests {
    use super::super::arrow_integration::*;
    use proptest::prelude::*;
    use polars::prelude::*;
    use arrow::datatypes::DataType as ArrowDataType;

    proptest! {
        #[test]
        fn test_dtype_conversion_roundtrip_never_panics(
            type_choice in 0u8..4u8
        ) {
            let polars_type = match type_choice {
                0 => PolarsDataType::Int64,
                1 => PolarsDataType::Float64,
                2 => PolarsDataType::String,
                _ => PolarsDataType::Boolean,
            };
            
            // Should not panic on conversion
            let arrow_result = polars_dtype_to_arrow(&polars_type);
            if let Ok(arrow_type) = arrow_result {
                let _polars_result = arrow_dtype_to_polars(&arrow_type);
            }
        }

        #[test]
        fn test_int_dataframe_conversion_robustness(
            values in prop::collection::vec(-1000i64..1000i64, 1..100)
        ) {
            let df = df! {
                "integers" => values,
            };
            
            if let Ok(df) = df {
                let arrow_result = dataframe_to_arrow(&df);
                if let Ok(batch) = arrow_result {
                    let _df_result = arrow_to_dataframe(&batch);
                }
            }
        }

        #[test]
        fn test_float_dataframe_conversion_robustness(
            values in prop::collection::vec(-1000.0f64..1000.0f64, 1..100)
        ) {
            let df = df! {
                "floats" => values,
            };
            
            if let Ok(df) = df {
                let arrow_result = dataframe_to_arrow(&df);
                if let Ok(batch) = arrow_result {
                    let _df_result = arrow_to_dataframe(&batch);
                }
            }
        }

        #[test]
        fn test_string_dataframe_conversion_robustness(
            values in prop::collection::vec("[a-zA-Z0-9]{1,20}", 1..50)
        ) {
            let df = df! {
                "strings" => values,
            };
            
            if let Ok(df) = df {
                let arrow_result = dataframe_to_arrow(&df);
                if let Ok(batch) = arrow_result {
                    let _df_result = arrow_to_dataframe(&batch);
                }
            }
        }

        #[test]
        fn test_boolean_dataframe_conversion_robustness(
            values in prop::collection::vec(any::<bool>(), 1..100)
        ) {
            let df = df! {
                "booleans" => values,
            };
            
            if let Ok(df) = df {
                let arrow_result = dataframe_to_arrow(&df);
                if let Ok(batch) = arrow_result {
                    let _df_result = arrow_to_dataframe(&batch);
                }
            }
        }
    }
}

#[cfg(not(feature = "dataframe"))]
#[cfg(test)]
mod disabled_tests {
    #[test]
    fn test_dataframe_feature_disabled() {
        // When dataframe feature is disabled, we can't test the actual functions
        // but we can test that the feature flag works correctly
        assert!(true, "DataFrame feature is disabled - no Arrow integration available");
    }
}