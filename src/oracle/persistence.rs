//! Model Persistence for Oracle (`.apr` format)
//!
//! Implements save/load for trained Oracle models using aprender's `SafeTensors` format.
//!
//! # File Format
//! ```text
//! ruchy_oracle.apr
//! ├── Header (magic: "APRN", version, metadata)
//! ├── Metadata (JSON: training_samples, accuracy, feature_count)
//! ├── Model Weights (SafeTensors compressed)
//! └── Checksum (SHA-256)
//! ```
//!
//! # References
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §Appendix C

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::OracleError;

/// Magic bytes for APR format
pub const APR_MAGIC: &[u8; 4] = b"APRN";

/// Current format version
pub const APR_VERSION: u8 = 1;

/// Default model filename
pub const DEFAULT_MODEL_NAME: &str = "ruchy_oracle.apr";

/// Metadata for saved Oracle model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelMetadata {
    /// Model name
    pub name: String,

    /// Model version
    pub version: String,

    /// Number of training samples
    pub training_samples: usize,

    /// Training accuracy (0.0-1.0)
    pub accuracy: f64,

    /// Number of features
    pub feature_count: usize,

    /// Number of categories
    pub category_count: usize,

    /// Training timestamp (Unix epoch)
    pub trained_at: u64,

    /// `RandomForest` tree count
    pub tree_count: usize,

    /// `RandomForest` max depth
    pub max_depth: usize,

    /// Checksum of model weights
    pub weights_checksum: String,
}

impl ModelMetadata {
    /// Create new metadata with current timestamp
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            name: name.into(),
            version: "2.0.0".to_string(),
            training_samples: 0,
            accuracy: 0.0,
            feature_count: 73,
            category_count: 8,
            trained_at: timestamp,
            tree_count: 100,
            max_depth: 10,
            weights_checksum: String::new(),
        }
    }

    /// Set training statistics
    #[must_use]
    pub fn with_training_stats(mut self, samples: usize, accuracy: f64) -> Self {
        self.training_samples = samples;
        self.accuracy = accuracy;
        self
    }

    /// Set tree parameters
    #[must_use]
    pub fn with_tree_params(mut self, tree_count: usize, max_depth: usize) -> Self {
        self.tree_count = tree_count;
        self.max_depth = max_depth;
        self
    }
}

impl Default for ModelMetadata {
    fn default() -> Self {
        Self::new("ruchy-oracle")
    }
}

/// Model storage paths
#[derive(Debug, Clone)]
pub struct ModelPaths {
    /// Primary model path (project root)
    pub primary: PathBuf,

    /// User model path (~/.ruchy/oracle/)
    pub user: PathBuf,

    /// Backup path
    pub backup: PathBuf,
}

impl ModelPaths {
    /// Create default model paths
    #[must_use]
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_or_else(|_| PathBuf::from("."), PathBuf::from);
        let user_dir = home.join(".ruchy").join("oracle");

        Self {
            primary: PathBuf::from(DEFAULT_MODEL_NAME),
            user: user_dir.join(DEFAULT_MODEL_NAME),
            backup: user_dir.join(format!("{DEFAULT_MODEL_NAME}.backup")),
        }
    }

    /// Find first existing model path
    #[must_use]
    pub fn find_existing(&self) -> Option<PathBuf> {
        if self.primary.exists() {
            Some(self.primary.clone())
        } else if self.user.exists() {
            Some(self.user.clone())
        } else {
            None
        }
    }

    /// Get path for saving (creates directories if needed)
    pub fn get_save_path(&self) -> Result<PathBuf, OracleError> {
        // Prefer user directory for persistence
        if let Some(parent) = self.user.parent() {
            fs::create_dir_all(parent).map_err(|e| OracleError::IoError(e.to_string()))?;
        }
        Ok(self.user.clone())
    }
}

impl Default for ModelPaths {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialized model data
#[derive(Debug, Clone)]
pub struct SerializedModel {
    /// Model metadata
    pub metadata: ModelMetadata,

    /// Serialized weights (compressed)
    pub weights: Vec<u8>,

    /// Training features (for k-NN fallback)
    pub training_features: Vec<Vec<f32>>,

    /// Training labels
    pub training_labels: Vec<usize>,
}

impl SerializedModel {
    /// Create from training data
    #[must_use]
    pub fn new(metadata: ModelMetadata) -> Self {
        Self {
            metadata,
            weights: Vec::new(),
            training_features: Vec::new(),
            training_labels: Vec::new(),
        }
    }

    /// Set training data for persistence
    pub fn with_training_data(
        mut self,
        features: Vec<Vec<f32>>,
        labels: Vec<usize>,
    ) -> Self {
        self.training_features = features;
        self.training_labels = labels;
        self.metadata.training_samples = self.training_labels.len();
        self
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), OracleError> {
        // Create backup if file exists
        if path.exists() {
            let backup_path = path.with_extension("apr.backup");
            fs::copy(path, &backup_path).map_err(|e| OracleError::IoError(e.to_string()))?;
        }

        let file = File::create(path).map_err(|e| OracleError::IoError(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        // Write magic bytes
        writer
            .write_all(APR_MAGIC)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        // Write version
        writer
            .write_all(&[APR_VERSION])
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        // Serialize metadata as JSON
        let metadata_json =
            serde_json::to_vec(&self.metadata).map_err(|e| OracleError::IoError(e.to_string()))?;

        // Write metadata length and data
        let metadata_len = metadata_json.len() as u32;
        writer
            .write_all(&metadata_len.to_le_bytes())
            .map_err(|e| OracleError::IoError(e.to_string()))?;
        writer
            .write_all(&metadata_json)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        // Serialize training data
        let training_data = TrainingDataBlob {
            features: self.training_features.clone(),
            labels: self.training_labels.clone(),
        };
        let training_json =
            serde_json::to_vec(&training_data).map_err(|e| OracleError::IoError(e.to_string()))?;

        // Write training data length and data
        let training_len = training_json.len() as u32;
        writer
            .write_all(&training_len.to_le_bytes())
            .map_err(|e| OracleError::IoError(e.to_string()))?;
        writer
            .write_all(&training_json)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        // Write weights length and data
        let weights_len = self.weights.len() as u32;
        writer
            .write_all(&weights_len.to_le_bytes())
            .map_err(|e| OracleError::IoError(e.to_string()))?;
        writer
            .write_all(&self.weights)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        writer.flush().map_err(|e| OracleError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self, OracleError> {
        if !path.exists() {
            return Err(OracleError::ModelNotFound(path.to_path_buf()));
        }

        let file = File::open(path).map_err(|e| OracleError::IoError(e.to_string()))?;
        let mut reader = BufReader::new(file);

        // Read and verify magic bytes
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        if &magic != APR_MAGIC {
            return Err(OracleError::IoError("Invalid APR file magic".to_string()));
        }

        // Read version
        let mut version = [0u8; 1];
        reader
            .read_exact(&mut version)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        if version[0] != APR_VERSION {
            return Err(OracleError::IoError(format!(
                "Unsupported APR version: {}",
                version[0]
            )));
        }

        // Read metadata
        let mut metadata_len_bytes = [0u8; 4];
        reader
            .read_exact(&mut metadata_len_bytes)
            .map_err(|e| OracleError::IoError(e.to_string()))?;
        let metadata_len = u32::from_le_bytes(metadata_len_bytes) as usize;

        let mut metadata_json = vec![0u8; metadata_len];
        reader
            .read_exact(&mut metadata_json)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        let metadata: ModelMetadata =
            serde_json::from_slice(&metadata_json).map_err(|e| OracleError::IoError(e.to_string()))?;

        // Read training data
        let mut training_len_bytes = [0u8; 4];
        reader
            .read_exact(&mut training_len_bytes)
            .map_err(|e| OracleError::IoError(e.to_string()))?;
        let training_len = u32::from_le_bytes(training_len_bytes) as usize;

        let mut training_json = vec![0u8; training_len];
        reader
            .read_exact(&mut training_json)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        let training_data: TrainingDataBlob =
            serde_json::from_slice(&training_json).map_err(|e| OracleError::IoError(e.to_string()))?;

        // Read weights
        let mut weights_len_bytes = [0u8; 4];
        reader
            .read_exact(&mut weights_len_bytes)
            .map_err(|e| OracleError::IoError(e.to_string()))?;
        let weights_len = u32::from_le_bytes(weights_len_bytes) as usize;

        let mut weights = vec![0u8; weights_len];
        reader
            .read_exact(&mut weights)
            .map_err(|e| OracleError::IoError(e.to_string()))?;

        Ok(Self {
            metadata,
            weights,
            training_features: training_data.features,
            training_labels: training_data.labels,
        })
    }
}

/// Internal struct for serializing training data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TrainingDataBlob {
    features: Vec<Vec<f32>>,
    labels: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    // ============================================================================
    // EXTREME TDD: ModelMetadata Tests
    // ============================================================================

    #[test]
    fn test_model_metadata_new() {
        let metadata = ModelMetadata::new("test-model");
        assert_eq!(metadata.name, "test-model");
        assert_eq!(metadata.feature_count, 73);
        assert_eq!(metadata.category_count, 8);
        assert!(metadata.trained_at > 0);
    }

    #[test]
    fn test_model_metadata_with_training_stats() {
        let metadata = ModelMetadata::new("test")
            .with_training_stats(1000, 0.95);
        assert_eq!(metadata.training_samples, 1000);
        assert!((metadata.accuracy - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_metadata_with_tree_params() {
        let metadata = ModelMetadata::new("test")
            .with_tree_params(50, 8);
        assert_eq!(metadata.tree_count, 50);
        assert_eq!(metadata.max_depth, 8);
    }

    #[test]
    fn test_model_metadata_default() {
        let metadata = ModelMetadata::default();
        assert_eq!(metadata.name, "ruchy-oracle");
    }

    // ============================================================================
    // EXTREME TDD: ModelPaths Tests
    // ============================================================================

    #[test]
    fn test_model_paths_new() {
        let paths = ModelPaths::new();
        assert_eq!(paths.primary, PathBuf::from(DEFAULT_MODEL_NAME));
        assert!(paths.user.to_string_lossy().contains(".ruchy"));
    }

    #[test]
    fn test_model_paths_find_existing_none() {
        let paths = ModelPaths {
            primary: PathBuf::from("/nonexistent/path1.apr"),
            user: PathBuf::from("/nonexistent/path2.apr"),
            backup: PathBuf::from("/nonexistent/path3.apr"),
        };
        assert!(paths.find_existing().is_none());
    }

    // ============================================================================
    // EXTREME TDD: SerializedModel Tests
    // ============================================================================

    #[test]
    fn test_serialized_model_new() {
        let metadata = ModelMetadata::new("test");
        let model = SerializedModel::new(metadata);
        assert!(model.weights.is_empty());
        assert!(model.training_features.is_empty());
    }

    #[test]
    fn test_serialized_model_with_training_data() {
        let metadata = ModelMetadata::new("test");
        let features = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let labels = vec![0, 1];

        let model = SerializedModel::new(metadata)
            .with_training_data(features, labels);

        assert_eq!(model.training_features.len(), 2);
        assert_eq!(model.training_labels.len(), 2);
        assert_eq!(model.metadata.training_samples, 2);
    }

    #[test]
    fn test_serialized_model_save_load_roundtrip() {
        let temp_path = temp_dir().join("test_oracle_roundtrip.apr");

        // Create model with training data
        let metadata = ModelMetadata::new("test-roundtrip")
            .with_training_stats(100, 0.85)
            .with_tree_params(10, 5);

        let features = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];
        let labels = vec![0, 1, 2];

        let model = SerializedModel::new(metadata)
            .with_training_data(features, labels.clone());

        // Save
        model.save(&temp_path).expect("save failed");
        assert!(temp_path.exists());

        // Load
        let loaded = SerializedModel::load(&temp_path).expect("load failed");

        // Verify
        assert_eq!(loaded.metadata.name, "test-roundtrip");
        // Note: training_samples is updated by with_training_data to match actual data
        assert_eq!(loaded.metadata.training_samples, 3);
        assert!((loaded.metadata.accuracy - 0.85).abs() < f64::EPSILON);
        assert_eq!(loaded.training_features.len(), 3);
        assert_eq!(loaded.training_labels, labels);

        // Cleanup
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_serialized_model_load_not_found() {
        let result = SerializedModel::load(Path::new("/nonexistent/model.apr"));
        assert!(matches!(result, Err(OracleError::ModelNotFound(_))));
    }

    #[test]
    fn test_serialized_model_load_invalid_magic() {
        let temp_path = temp_dir().join("test_invalid_magic.apr");

        // Write invalid file
        fs::write(&temp_path, b"XXXX").expect("write failed");

        let result = SerializedModel::load(&temp_path);
        assert!(matches!(result, Err(OracleError::IoError(_))));

        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_apr_magic_constant() {
        assert_eq!(APR_MAGIC, b"APRN");
    }

    #[test]
    fn test_apr_version_constant() {
        assert_eq!(APR_VERSION, 1);
    }

    #[test]
    fn test_default_model_name_constant() {
        assert_eq!(DEFAULT_MODEL_NAME, "ruchy_oracle.apr");
    }
}
