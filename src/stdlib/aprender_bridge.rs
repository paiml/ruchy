//! Aprender Bridge Module (Pillar 4: Machine Learning)
//!
//! Thin wrappers around Aprender ML primitives for Ruchy stdlib.
//! Per spec Section 5.4: Production-ready ML estimators with scikit-learn-compatible API.
//!
//! # Design
//! - Scikit-learn compatible API (fit/predict/transform)
//! - Built on Trueno SIMD compute substrate
//! - Zero-copy data transfer where possible
//!
//! # Model Persistence (APR/SafeTensors Format)
//!
//! All estimators support `SafeTensors` serialization compatible with:
//! - `HuggingFace` ecosystem
//! - `PyTorch`, `TensorFlow`
//! - GGUF/Ollama (via conversion)
//!
//! ```ignore
//! // Save model
//! model.save_safetensors("model.safetensors")?;
//!
//! // Load model
//! let loaded = LinearRegression::load_safetensors("model.safetensors")?;
//! ```
//!
//! # Quantization (Issue #171, Spec Section 13.4)
//!
//! Re-exports GGUF-compatible quantization from `aprender::format::quantize`:
//! - `Q8_0`: 8-bit quantization (4x size reduction, <0.1% accuracy loss)
//! - `Q4_0`: 4-bit quantization (8x size reduction, <1% accuracy loss)
//!
//! ```ignore
//! use ruchy::stdlib::aprender_bridge::{quantize, dequantize, QuantType};
//!
//! // Quantize weights
//! let q8 = quantize(&weights, &shape, QuantType::Q8_0)?;
//! let q4 = quantize(&weights, &shape, QuantType::Q4_0)?;
//!
//! // Dequantize for inference
//! let restored = dequantize(&q8)?;
//! ```
//!
//! # References
//! - [31] Buitinck et al. (2013). "API design for machine learning software"
//! - [41] Gerganov et al. (2023). "GGML: Tensor Library for Machine Learning"

// Re-export core types from aprender prelude
pub use aprender::prelude::{
    // Metrics
    mae,
    mse,
    r_squared,
    rmse,
    // Optimizers
    Adam,
    // Estimators
    DecisionTreeClassifier,
    DecisionTreeRegressor,
    ElasticNet,
    // Traits
    Estimator,
    KMeans,
    Lasso,
    LinearRegression,
    LogisticRegression,
    // Primitives (from trueno)
    Matrix,
    // Preprocessing
    MinMaxScaler,
    RandomForestRegressor,
    Ridge,
    StandardScaler,
    Transformer,
    UnsupervisedEstimator,
    Vector,
    DBSCAN,
    SGD,
};

// PCA is in preprocessing, not prelude
pub use aprender::preprocessing::PCA;

// Re-export serialization for model persistence (APR format via SafeTensors)
pub use aprender::serialization::SafeTensorsMetadata;

// Re-export quantization (GGUF-compatible Q8_0/Q4_0) - Issue #171
pub use aprender::format::quantize::{
    dequantize, quantization_mse, quantize, Q4_0Quantizer, Q8_0Quantizer, QuantType,
    QuantizationInfo, QuantizedBlock, Quantizer, BLOCK_SIZE,
};

// Re-export GGUF export for llama.cpp compatibility
pub use aprender::format::gguf::{
    export_tensors_to_gguf, GgmlType, GgufHeader, GgufTensor, GgufTensorInfo, GgufValue,
    GgufValueType, GGUF_DEFAULT_ALIGNMENT, GGUF_MAGIC, GGUF_VERSION,
};

// HuggingFace Hub integration available via `aprender` with `hf-hub-integration` feature

// Re-export Ed25519 signing for model provenance (spec §12.3, §13.4)
pub use aprender::format::{
    load_verified, save_signed, SigningKey, VerifyingKey, PUBLIC_KEY_SIZE, SIGNATURE_SIZE,
};

/// Compute R² score for regression predictions.
///
/// # Arguments
/// * `y_true` - Ground truth values
/// * `y_pred` - Predicted values
///
/// # Returns
/// R² coefficient of determination
pub fn compute_r2(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        return 0.0;
    }

    let mean_true: f64 = y_true.iter().sum::<f64>() / y_true.len() as f64;
    let ss_tot: f64 = y_true.iter().map(|&y| (y - mean_true).powi(2)).sum();
    let ss_res: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| (yt - yp).powi(2))
        .sum();

    if ss_tot == 0.0 {
        return 1.0;
    }
    1.0 - (ss_res / ss_tot)
}

/// Compute mean squared error.
pub fn compute_mse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        return 0.0;
    }
    y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| (yt - yp).powi(2))
        .sum::<f64>()
        / y_true.len() as f64
}

/// Compute mean absolute error.
pub fn compute_mae(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        return 0.0;
    }
    y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| (yt - yp).abs())
        .sum::<f64>()
        / y_true.len() as f64
}

/// Compute root mean squared error.
pub fn compute_rmse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    compute_mse(y_true, y_pred).sqrt()
}

// ============================================================================
// Quantization Support (Issue #171, Spec Section 13.4)
// Re-exported from aprender::format::quantize - GGUF-compatible Q8_0/Q4_0
// See aprender/docs/specifications/model-format-spec.md §6.2 for full details
// ============================================================================

/// Compute classification accuracy.
pub fn compute_accuracy(y_true: &[usize], y_pred: &[usize]) -> f64 {
    if y_true.len() != y_pred.len() || y_true.is_empty() {
        return 0.0;
    }
    let correct = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(a, b)| a == b)
        .count();
    correct as f64 / y_true.len() as f64
}

// ============================================================================
// Ruchy 5.0 Beta.1: ML Pipeline Types
// Per ruchy-5.0-sovereign-platform.md: aprender 0.27.5 integration
// ============================================================================

/// ML model training configuration for `ruchy apr run`.
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Learning rate
    pub learning_rate: f64,
    /// Number of training epochs
    pub epochs: usize,
    /// Batch size
    pub batch_size: usize,
    /// Early stopping patience (None = no early stopping)
    pub patience: Option<usize>,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            epochs: 100,
            batch_size: 32,
            patience: Some(10),
            seed: None,
        }
    }
}

impl TrainingConfig {
    /// Set learning rate.
    pub fn with_lr(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    /// Set number of epochs.
    pub fn with_epochs(mut self, epochs: usize) -> Self {
        self.epochs = epochs;
        self
    }

    /// Set batch size.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

/// ML model inference configuration for `ruchy apr serve`.
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Model file path
    pub model_path: String,
    /// Quantization level (None = full precision)
    pub quantization: Option<QuantLevel>,
    /// Maximum batch size for inference
    pub max_batch_size: usize,
    /// HTTP port for serving (default: 8000)
    pub port: u16,
}

/// Quantization levels for model deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantLevel {
    /// 16-bit floating point (half precision)
    F16,
    /// 8-bit integer quantization
    Int8,
    /// 4-bit integer quantization
    Int4,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            quantization: None,
            max_batch_size: 64,
            port: 8000,
        }
    }
}

/// ML pipeline stage definition for `@pipeline` decorator.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStage {
    /// Load and preprocess data
    DataLoad { source: String },
    /// Transform features
    Transform { method: String },
    /// Train model
    Train { config: String },
    /// Evaluate model
    Evaluate { metrics: Vec<String> },
    /// Deploy model
    Deploy { target: String },
}

/// Result of a training run.
#[derive(Debug, Clone)]
pub struct TrainingResult {
    /// Final training loss
    pub final_loss: f64,
    /// Final validation loss (if validation set provided)
    pub val_loss: Option<f64>,
    /// Number of epochs actually run (may be less than config.epochs if early stopped)
    pub epochs_run: usize,
    /// Best epoch (by validation loss)
    pub best_epoch: usize,
    /// Metrics at best epoch
    pub metrics: Vec<(String, f64)>,
}

impl TrainingResult {
    /// Create a result for a completed training run.
    pub fn completed(final_loss: f64, epochs_run: usize) -> Self {
        Self {
            final_loss,
            val_loss: None,
            epochs_run,
            best_epoch: epochs_run,
            metrics: Vec::new(),
        }
    }

    /// Format as a human-readable summary.
    pub fn summary(&self) -> String {
        let val = self
            .val_loss
            .map(|v| format!(", val_loss={v:.6}"))
            .unwrap_or_default();
        format!(
            "Training: {} epochs, loss={:.6}{}, best_epoch={}",
            self.epochs_run, self.final_loss, val, self.best_epoch
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mse_perfect() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.0, 2.0, 3.0];
        let mse_val = compute_mse(&y_true, &y_pred);
        assert!((mse_val - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_mae_basic() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.5, 2.5, 3.5];
        let mae_val = compute_mae(&y_true, &y_pred);
        assert!((mae_val - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_r2_perfect() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.0, 2.0, 3.0];
        let r2 = compute_r2(&y_true, &y_pred);
        assert!((r2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_accuracy_perfect() {
        let y_true = vec![0, 1, 0, 1];
        let y_pred = vec![0, 1, 0, 1];
        let acc = compute_accuracy(&y_true, &y_pred);
        assert!((acc - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_accuracy_half() {
        let y_true = vec![0, 1, 0, 1];
        let y_pred = vec![0, 0, 1, 1];
        let acc = compute_accuracy(&y_true, &y_pred);
        assert!((acc - 0.5).abs() < 1e-10);
    }

    // ============================================================================
    // Quantization Re-export Tests (Issue #171)
    // Tests verify aprender quantization is properly re-exported
    // Full quantization tests are in aprender::format::quantize
    // ============================================================================

    #[test]
    fn test_quantize_reexport_q8() {
        // Verify Q8_0 quantization is accessible via re-export
        let weights: Vec<f32> = (0..32).map(|i| (i as f32 - 16.0) / 10.0).collect();
        let shape = vec![32];

        let quantized = quantize(&weights, &shape, QuantType::Q8_0).expect("Q8_0 quantize");
        let restored = dequantize(&quantized).expect("Q8_0 dequantize");

        assert_eq!(restored.len(), 32);
        let mse = quantization_mse(&weights, &restored);
        assert!(mse < 0.01, "Q8_0 MSE too high: {mse}");
    }

    #[test]
    fn test_quantize_reexport_q4() {
        // Verify Q4_0 quantization is accessible via re-export
        let weights: Vec<f32> = (0..32).map(|i| (i as f32 - 16.0) / 10.0).collect();
        let shape = vec![32];

        let quantized = quantize(&weights, &shape, QuantType::Q4_0).expect("Q4_0 quantize");
        let restored = dequantize(&quantized).expect("Q4_0 dequantize");

        assert_eq!(restored.len(), 32);
        // Q4_0 has higher error but should be reasonable
        let mse = quantization_mse(&weights, &restored);
        assert!(mse < 0.5, "Q4_0 MSE too high: {mse}");
    }

    #[test]
    fn test_quant_type_reexport() {
        // Verify QuantType enum is accessible
        assert_eq!(QuantType::Q8_0 as u8, 0x01);
        assert_eq!(QuantType::Q4_0 as u8, 0x02);
    }

    #[test]
    fn test_block_size_constant() {
        // Verify BLOCK_SIZE constant matches GGUF spec (32)
        assert_eq!(BLOCK_SIZE, 32);
    }

    #[test]
    fn test_gguf_reexport() {
        // Verify GGUF types are accessible
        assert_eq!(GGUF_MAGIC, 0x4655_4747); // "GGUF"
        assert_eq!(GGUF_VERSION, 3);
        assert_eq!(GGUF_DEFAULT_ALIGNMENT, 32);
    }

    // HuggingFace Hub tests require `hf-hub-integration` feature on aprender

    // ============================================================================
    // Ed25519 Signing Re-export Tests (Spec §12.3, §13.4)
    // ============================================================================

    #[test]
    fn test_signing_constants_reexport() {
        // Verify Ed25519 constants are accessible
        assert_eq!(SIGNATURE_SIZE, 64); // Ed25519 signature is 64 bytes
        assert_eq!(PUBLIC_KEY_SIZE, 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    fn test_signing_key_reexport() {
        // Verify SigningKey and VerifyingKey types are accessible
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        // Verify key sizes
        assert_eq!(verifying_key.as_bytes().len(), PUBLIC_KEY_SIZE);
    }

    // ========== Beta.1: ML Pipeline Types Tests ==========

    #[test]
    fn test_training_config_default() {
        let config = TrainingConfig::default();
        assert!((config.learning_rate - 0.001).abs() < 1e-10);
        assert_eq!(config.epochs, 100);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.patience, Some(10));
    }

    #[test]
    fn test_training_config_builder() {
        let config = TrainingConfig::default()
            .with_lr(0.01)
            .with_epochs(50)
            .with_batch_size(64);
        assert!((config.learning_rate - 0.01).abs() < 1e-10);
        assert_eq!(config.epochs, 50);
        assert_eq!(config.batch_size, 64);
    }

    #[test]
    fn test_inference_config_default() {
        let config = InferenceConfig::default();
        assert!(config.model_path.is_empty());
        assert!(config.quantization.is_none());
        assert_eq!(config.max_batch_size, 64);
        assert_eq!(config.port, 8000);
    }

    #[test]
    fn test_quant_levels() {
        assert_ne!(QuantLevel::F16, QuantLevel::Int8);
        assert_ne!(QuantLevel::Int8, QuantLevel::Int4);
    }

    #[test]
    fn test_training_result_completed() {
        let result = TrainingResult::completed(0.05, 42);
        assert!((result.final_loss - 0.05).abs() < 1e-10);
        assert_eq!(result.epochs_run, 42);
        assert_eq!(result.best_epoch, 42);
        assert!(result.val_loss.is_none());
    }

    #[test]
    fn test_training_result_summary() {
        let result = TrainingResult::completed(0.123456, 10);
        let summary = result.summary();
        assert!(summary.contains("10 epochs"));
        assert!(summary.contains("0.123456"));
    }

    #[test]
    fn test_pipeline_stages() {
        let load = PipelineStage::DataLoad {
            source: "data.csv".to_string(),
        };
        let train = PipelineStage::Train {
            config: "default".to_string(),
        };
        assert_ne!(load, train);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_r2_bounded(
            y_true in prop::collection::vec(0.0..100.0f64, 5..20),
            y_pred in prop::collection::vec(0.0..100.0f64, 5..20),
        ) {
            if y_true.len() == y_pred.len() {
                let r2 = compute_r2(&y_true, &y_pred);
                // R² can be negative for very bad predictions, but bounded above by 1
                prop_assert!(r2 <= 1.0 + 1e-10);
            }
        }

        #[test]
        fn prop_mse_non_negative(
            y_true in prop::collection::vec(-100.0..100.0f64, 1..50),
            y_pred in prop::collection::vec(-100.0..100.0f64, 1..50),
        ) {
            if y_true.len() == y_pred.len() {
                let mse_val = compute_mse(&y_true, &y_pred);
                prop_assert!(mse_val >= 0.0);
            }
        }

        #[test]
        fn prop_mae_non_negative(
            y_true in prop::collection::vec(-100.0..100.0f64, 1..50),
            y_pred in prop::collection::vec(-100.0..100.0f64, 1..50),
        ) {
            if y_true.len() == y_pred.len() {
                let mae_val = compute_mae(&y_true, &y_pred);
                prop_assert!(mae_val >= 0.0);
            }
        }

        #[test]
        fn prop_accuracy_bounded(
            y_true in prop::collection::vec(0usize..2, 1..50),
            y_pred in prop::collection::vec(0usize..2, 1..50),
        ) {
            if y_true.len() == y_pred.len() {
                let acc = compute_accuracy(&y_true, &y_pred);
                prop_assert!((0.0..=1.0).contains(&acc));
            }
        }
    }
}
