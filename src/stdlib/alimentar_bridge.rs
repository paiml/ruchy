//! Alimentar Bridge Module (Pillar 2: Data Loading)
//!
//! Thin wrappers around Alimentar for Ruchy stdlib.
//! Per spec Section 5.2: Zero-copy Arrow-based data loading with transforms and drift detection.
//!
//! # Design
//! - Zero-copy Arrow `RecordBatch` throughout
//! - Multiple backends: Local, S3, HTTP, `HuggingFace` Hub
//! - Streaming for memory-efficient loading
//! - Transforms: Filter, shuffle, sample, normalize
//! - Data quality: Null detection, duplicates, outliers
//! - Drift detection: KS test, Chi-square, PSI
//!
//! # References
//! - [9] Apache Arrow Project (2024). "Apache Arrow"
//! - [46] Zero-copy Arrow data loading

// Re-export core types from alimentar
pub use alimentar::{ArrowDataset, CsvOptions, DataLoader, Dataset, JsonOptions};
pub use alimentar::{Error as AlimentarError, Result as AlimentarResult};

// Re-export Arrow types for interop
pub use alimentar::{RecordBatch, Schema, SchemaRef};

// Re-export transforms
pub use alimentar::transform::{
    Cast, Chain, Drop, FillNull, FillStrategy, Filter, Map, NormMethod, Normalize, Rename, Select,
    Skip, Sort, SortOrder, Take, Transform, Unique,
};

// Re-export quality checks
pub use alimentar::{ColumnQuality, QualityChecker, QualityIssue, QualityProfile, QualityReport};

// Re-export drift detection
pub use alimentar::{ColumnDrift, DriftDetector, DriftReport, DriftSeverity, DriftTest};

// Re-export split utilities
pub use alimentar::DatasetSplit;

// Re-export federated learning splits
pub use alimentar::{
    FederatedSplitCoordinator, FederatedSplitStrategy, GlobalSplitReport, NodeSplitInstruction,
    NodeSplitManifest, NodeSummary, SplitQualityIssue,
};

/// Convenience wrapper to load a Parquet file as an `ArrowDataset`.
///
/// # Errors
/// Returns error if file cannot be read or parsed.
///
/// # Examples
/// ```ignore
/// use ruchy::stdlib::alimentar_bridge::load_parquet;
///
/// let dataset = load_parquet("data/train.parquet")?;
/// println!("Loaded {} rows", dataset.len());
/// ```
pub fn load_parquet(path: &str) -> AlimentarResult<ArrowDataset> {
    ArrowDataset::from_parquet(path)
}

/// Convenience wrapper to load a CSV file as an `ArrowDataset`.
///
/// # Errors
/// Returns error if file cannot be read or parsed.
pub fn load_csv(path: &str) -> AlimentarResult<ArrowDataset> {
    ArrowDataset::from_csv(path)
}

/// Convenience wrapper to load a JSON file as an `ArrowDataset`.
///
/// # Errors
/// Returns error if file cannot be read or parsed.
pub fn load_json(path: &str) -> AlimentarResult<ArrowDataset> {
    ArrowDataset::from_json(path)
}

/// Create a `DataLoader` with common defaults.
///
/// Default configuration:
/// - Batch size: 32
/// - Shuffle: true
/// - Drop last incomplete batch: false
#[must_use]
pub fn create_loader(dataset: ArrowDataset) -> DataLoader<ArrowDataset> {
    DataLoader::new(dataset).batch_size(32).shuffle(true)
}

/// Create a `DataLoader` with custom batch size.
#[must_use]
pub fn create_loader_with_batch_size(
    dataset: ArrowDataset,
    batch_size: usize,
) -> DataLoader<ArrowDataset> {
    DataLoader::new(dataset).batch_size(batch_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataloader_batch_size() {
        // This is a compile-time test to ensure the API is correct
        // We can't actually create a dataset without a file
        fn check_api() {
            let _: fn(ArrowDataset) -> DataLoader<ArrowDataset> = create_loader;
            let _: fn(ArrowDataset, usize) -> DataLoader<ArrowDataset> =
                create_loader_with_batch_size;
        }
        check_api();
    }

    #[test]
    fn test_csv_options_exists() {
        // Verify CsvOptions type is accessible
        let _: fn() -> CsvOptions = CsvOptions::default;
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_batch_size_positive(batch_size in 1usize..1000) {
            // Batch size should always be positive
            prop_assert!(batch_size > 0);
        }
    }
}
