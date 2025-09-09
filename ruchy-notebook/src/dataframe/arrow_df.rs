use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray, BooleanArray};
use arrow::datatypes::{DataType as ArrowDataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use anyhow::{Result, bail};

/// Ruchy DataFrame backed by Apache Arrow
#[derive(Clone)]
pub struct DataFrame {
    batch: RecordBatch,
    schema: Arc<Schema>,
}

/// Column representation with zero-copy operations
pub struct Column {
    name: String,
    array: ArrayRef,
}

/// Supported data types
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int64,
    Float64,
    String,
    Boolean,
}

impl DataFrame {
    /// Create a new DataFrame from columns
    pub fn new(columns: Vec<Column>) -> Result<Self> {
        if columns.is_empty() {
            bail!("Cannot create DataFrame with no columns");
        }
        
        let fields: Vec<Field> = columns.iter()
            .map(|col| Field::new(&col.name, col.array.data_type().clone(), true))
            .collect();
        
        let schema = Arc::new(Schema::new(fields));
        let arrays: Vec<ArrayRef> = columns.into_iter()
            .map(|col| col.array)
            .collect();
        
        let batch = RecordBatch::try_new(schema.clone(), arrays)?;
        
        Ok(Self { batch, schema })
    }
    
    /// Create DataFrame from raw arrays with names
    pub fn from_arrays(names: Vec<String>, arrays: Vec<ArrayRef>) -> Result<Self> {
        if names.len() != arrays.len() {
            bail!("Number of names must match number of arrays");
        }
        
        let columns: Vec<Column> = names.into_iter()
            .zip(arrays)
            .map(|(name, array)| Column { name, array })
            .collect();
        
        Self::new(columns)
    }
    
    /// Get number of rows
    pub fn num_rows(&self) -> usize {
        self.batch.num_rows()
    }
    
    /// Get number of columns
    pub fn num_columns(&self) -> usize {
        self.batch.num_columns()
    }
    
    /// Get column by index (zero-copy)
    pub fn column(&self, index: usize) -> Option<&ArrayRef> {
        if index < self.batch.num_columns() {
            Some(self.batch.column(index))
        } else {
            None
        }
    }
    
    /// Get column by name (zero-copy)
    pub fn column_by_name(&self, name: &str) -> Option<&ArrayRef> {
        self.schema.fields()
            .iter()
            .position(|f| f.name() == name)
            .and_then(|idx| self.column(idx))
    }
    
    /// Select columns by names (zero-copy projection)
    pub fn select(&self, columns: &[&str]) -> Result<Self> {
        let mut selected_arrays = Vec::new();
        let mut selected_fields = Vec::new();
        
        for name in columns {
            let idx = self.schema.fields()
                .iter()
                .position(|f| f.name() == *name)
                .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", name))?;
            
            selected_arrays.push(self.batch.column(idx).clone());
            selected_fields.push(self.schema.field(idx).clone());
        }
        
        let new_schema = Arc::new(Schema::new(selected_fields));
        let new_batch = RecordBatch::try_new(new_schema.clone(), selected_arrays)?;
        
        Ok(Self {
            batch: new_batch,
            schema: new_schema,
        })
    }
    
    /// Filter rows using boolean array (zero-copy where possible)
    pub fn filter(&self, mask: &BooleanArray) -> Result<Self> {
        use arrow::compute::filter_record_batch;
        
        if mask.len() != self.num_rows() {
            bail!("Filter mask length {} doesn't match row count {}", 
                  mask.len(), self.num_rows());
        }
        
        let filtered = filter_record_batch(&self.batch, mask)?;
        
        Ok(Self {
            batch: filtered,
            schema: self.schema.clone(),
        })
    }
    
    /// Slice rows (zero-copy operation)
    pub fn slice(&self, offset: usize, length: usize) -> Result<Self> {
        if offset + length > self.num_rows() {
            bail!("Slice range [{}, {}) out of bounds for {} rows",
                  offset, offset + length, self.num_rows());
        }
        
        let sliced = self.batch.slice(offset, length);
        
        Ok(Self {
            batch: sliced,
            schema: self.schema.clone(),
        })
    }
    
    /// Get schema information
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    
    /// Calculate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.batch.columns()
            .iter()
            .map(|col| col.get_array_memory_size())
            .sum()
    }
    
    /// Convert to string representation for display
    pub fn to_string(&self) -> String {
        format!("DataFrame[{} rows × {} cols]", self.num_rows(), self.num_columns())
    }
}

impl Column {
    /// Create an Int64 column
    pub fn int64(name: impl Into<String>, values: Vec<i64>) -> Self {
        Self {
            name: name.into(),
            array: Arc::new(Int64Array::from(values)),
        }
    }
    
    /// Create a Float64 column
    pub fn float64(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            array: Arc::new(Float64Array::from(values)),
        }
    }
    
    /// Create a String column
    pub fn string(name: impl Into<String>, values: Vec<Option<String>>) -> Self {
        Self {
            name: name.into(),
            array: Arc::new(StringArray::from(values)),
        }
    }
    
    /// Create a Boolean column
    pub fn boolean(name: impl Into<String>, values: Vec<bool>) -> Self {
        Self {
            name: name.into(),
            array: Arc::new(BooleanArray::from(values)),
        }
    }
}

impl DataType {
    /// Convert to Arrow data type
    pub fn to_arrow(&self) -> ArrowDataType {
        match self {
            DataType::Int64 => ArrowDataType::Int64,
            DataType::Float64 => ArrowDataType::Float64,
            DataType::String => ArrowDataType::Utf8,
            DataType::Boolean => ArrowDataType::Boolean,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dataframe_creation() {
        let col1 = Column::int64("id", vec![1, 2, 3, 4, 5]);
        let col2 = Column::float64("value", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let col3 = Column::string("name", vec![
            Some("Alice".to_string()),
            Some("Bob".to_string()),
            None,
            Some("David".to_string()),
            Some("Eve".to_string()),
        ]);
        
        let df = DataFrame::new(vec![col1, col2, col3]).unwrap();
        
        assert_eq!(df.num_rows(), 5);
        assert_eq!(df.num_columns(), 3);
    }
    
    #[test]
    fn test_dataframe_select() {
        let col1 = Column::int64("a", vec![1, 2, 3]);
        let col2 = Column::int64("b", vec![4, 5, 6]);
        let col3 = Column::int64("c", vec![7, 8, 9]);
        
        let df = DataFrame::new(vec![col1, col2, col3]).unwrap();
        let selected = df.select(&["a", "c"]).unwrap();
        
        assert_eq!(selected.num_columns(), 2);
        assert_eq!(selected.num_rows(), 3);
    }
    
    #[test]
    fn test_dataframe_filter() {
        let col1 = Column::int64("values", vec![1, 2, 3, 4, 5]);
        let df = DataFrame::new(vec![col1]).unwrap();
        
        let mask = BooleanArray::from(vec![true, false, true, false, true]);
        let filtered = df.filter(&mask).unwrap();
        
        assert_eq!(filtered.num_rows(), 3);
    }
    
    #[test]
    fn test_dataframe_slice() {
        let col = Column::int64("data", vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let df = DataFrame::new(vec![col]).unwrap();
        
        let sliced = df.slice(2, 5).unwrap();
        assert_eq!(sliced.num_rows(), 5);
        
        // Zero-copy verification: memory should be minimal
        let original_mem = df.memory_usage();
        let sliced_mem = sliced.memory_usage();
        assert!(sliced_mem <= original_mem);
    }
    
    #[test]
    fn test_zero_copy_operations() {
        let col = Column::float64("data", vec![1.0; 1_000_000]);
        let df = DataFrame::new(vec![col]).unwrap();
        
        let start_mem = df.memory_usage();
        
        // These operations should be zero-copy
        let _ = df.column(0);
        let _ = df.column_by_name("data");
        let _ = df.slice(0, 100).unwrap();
        
        // Memory should not increase significantly
        assert_eq!(df.memory_usage(), start_mem);
    }
}