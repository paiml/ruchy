//! Transfer Learning Configuration
//!
//! Implements transfer learning for cold start mitigation from spec §2.3:
//! - Stage 1: Pre-train on Rust error corpus (100K+ samples)
//! - Stage 2: Fine-tune on Ruchy-specific data
//!
//! # Benefits
//! - Leverages vast Rust compiler error knowledge
//! - Better "long tail" coverage from real-world Rust errors
//! - Faster convergence on Ruchy-specific patterns
//! - More robust to unseen error combinations
//!
//! # References
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §2.3

use std::path::PathBuf;

/// Configuration for transfer learning
#[derive(Debug, Clone)]
pub struct TransferLearningConfig {
    /// Path to pre-trained Rust error model
    pub pretrained_model: Option<PathBuf>,

    /// Layers to freeze during fine-tuning
    pub frozen_layers: Vec<String>,

    /// Learning rate for fine-tuning (lower than pre-training)
    pub fine_tune_lr: f64,

    /// Whether to use feature extraction only (freeze all)
    pub feature_extraction_only: bool,

    /// Number of fine-tuning epochs
    pub fine_tune_epochs: usize,

    /// Batch size for fine-tuning
    pub batch_size: usize,

    /// Early stopping patience (epochs without improvement)
    pub early_stopping_patience: usize,

    /// Validation split ratio
    pub validation_split: f64,
}

impl Default for TransferLearningConfig {
    fn default() -> Self {
        Self {
            pretrained_model: None,
            frozen_layers: Vec::new(),
            fine_tune_lr: 0.001,
            feature_extraction_only: false,
            fine_tune_epochs: 10,
            batch_size: 32,
            early_stopping_patience: 3,
            validation_split: 0.2,
        }
    }
}

impl TransferLearningConfig {
    /// Create new config with pretrained model path
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set pretrained model path
    #[must_use]
    pub fn with_pretrained(mut self, path: impl Into<PathBuf>) -> Self {
        self.pretrained_model = Some(path.into());
        self
    }

    /// Set learning rate
    #[must_use]
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.fine_tune_lr = lr;
        self
    }

    /// Set feature extraction only mode
    #[must_use]
    pub fn with_feature_extraction_only(mut self, enabled: bool) -> Self {
        self.feature_extraction_only = enabled;
        self
    }

    /// Add layer to freeze
    #[must_use]
    pub fn freeze_layer(mut self, layer: impl Into<String>) -> Self {
        self.frozen_layers.push(layer.into());
        self
    }

    /// Set fine-tuning epochs
    #[must_use]
    pub fn with_epochs(mut self, epochs: usize) -> Self {
        self.fine_tune_epochs = epochs;
        self
    }

    /// Set batch size
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set early stopping patience
    #[must_use]
    pub fn with_early_stopping(mut self, patience: usize) -> Self {
        self.early_stopping_patience = patience;
        self
    }

    /// Set validation split
    #[must_use]
    pub fn with_validation_split(mut self, split: f64) -> Self {
        self.validation_split = split.clamp(0.0, 0.5);
        self
    }

    /// Check if transfer learning is configured
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.pretrained_model.is_some()
    }
}

/// Transfer learning status
#[derive(Debug, Clone, PartialEq)]
pub enum TransferStatus {
    /// Not using transfer learning
    Disabled,

    /// Pre-trained model loaded
    PretrainedLoaded {
        /// Path to loaded model
        model_path: PathBuf,
        /// Number of frozen layers
        frozen_layers: usize,
    },

    /// Fine-tuning in progress
    FineTuning {
        /// Current epoch
        epoch: usize,
        /// Total epochs
        total_epochs: usize,
        /// Current loss
        loss: f64,
    },

    /// Fine-tuning complete
    Complete {
        /// Final accuracy
        accuracy: f64,
        /// Epochs trained
        epochs_trained: usize,
    },

    /// Error during transfer learning
    Error {
        /// Error message
        message: String,
    },
}

impl TransferStatus {
    /// Check if transfer learning is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TransferStatus::PretrainedLoaded { .. } | TransferStatus::FineTuning { .. }
        )
    }

    /// Check if transfer learning completed successfully
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self, TransferStatus::Complete { .. })
    }

    /// Check if there was an error
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, TransferStatus::Error { .. })
    }
}

/// Transfer learning manager
#[derive(Debug)]
pub struct TransferLearner {
    /// Configuration
    config: TransferLearningConfig,

    /// Current status
    status: TransferStatus,

    /// Training history (epoch → `(train_loss, val_loss)`)
    history: Vec<(f64, f64)>,

    /// Best validation accuracy seen
    best_val_accuracy: f64,

    /// Epochs since improvement (for early stopping)
    epochs_without_improvement: usize,
}

impl TransferLearner {
    /// Create new transfer learner
    #[must_use]
    pub fn new(config: TransferLearningConfig) -> Self {
        let status = if config.is_configured() {
            TransferStatus::Disabled // Will be updated when model loads
        } else {
            TransferStatus::Disabled
        };

        Self {
            config,
            status,
            history: Vec::new(),
            best_val_accuracy: 0.0,
            epochs_without_improvement: 0,
        }
    }

    /// Get current status
    #[must_use]
    pub fn status(&self) -> &TransferStatus {
        &self.status
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &TransferLearningConfig {
        &self.config
    }

    /// Get training history
    #[must_use]
    pub fn history(&self) -> &[(f64, f64)] {
        &self.history
    }

    /// Load pre-trained model
    pub fn load_pretrained(&mut self) -> Result<(), String> {
        let Some(ref path) = self.config.pretrained_model else {
            return Err("No pretrained model path configured".to_string());
        };

        if !path.exists() {
            return Err(format!("Pretrained model not found: {}", path.display()));
        }

        // In full implementation, would load model weights here
        self.status = TransferStatus::PretrainedLoaded {
            model_path: path.clone(),
            frozen_layers: self.config.frozen_layers.len(),
        };

        Ok(())
    }

    /// Record training epoch result
    pub fn record_epoch(&mut self, epoch: usize, train_loss: f64, val_loss: f64, val_accuracy: f64) {
        self.history.push((train_loss, val_loss));

        // Update status
        self.status = TransferStatus::FineTuning {
            epoch,
            total_epochs: self.config.fine_tune_epochs,
            loss: train_loss,
        };

        // Check for improvement
        if val_accuracy > self.best_val_accuracy {
            self.best_val_accuracy = val_accuracy;
            self.epochs_without_improvement = 0;
        } else {
            self.epochs_without_improvement += 1;
        }
    }

    /// Check if should early stop
    #[must_use]
    pub fn should_early_stop(&self) -> bool {
        self.epochs_without_improvement >= self.config.early_stopping_patience
    }

    /// Mark training as complete
    pub fn mark_complete(&mut self, final_accuracy: f64) {
        self.status = TransferStatus::Complete {
            accuracy: final_accuracy,
            epochs_trained: self.history.len(),
        };
    }

    /// Mark training as failed
    pub fn mark_error(&mut self, message: impl Into<String>) {
        self.status = TransferStatus::Error {
            message: message.into(),
        };
    }

    /// Reset learner state
    pub fn reset(&mut self) {
        self.status = TransferStatus::Disabled;
        self.history.clear();
        self.best_val_accuracy = 0.0;
        self.epochs_without_improvement = 0;
    }
}

impl Default for TransferLearner {
    fn default() -> Self {
        Self::new(TransferLearningConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: TransferLearningConfig Tests
    // ============================================================================

    #[test]
    fn test_transfer_config_default() {
        let config = TransferLearningConfig::default();
        assert!(config.pretrained_model.is_none());
        assert!(config.frozen_layers.is_empty());
        assert!((config.fine_tune_lr - 0.001).abs() < f64::EPSILON);
        assert!(!config.feature_extraction_only);
        assert_eq!(config.fine_tune_epochs, 10);
    }

    #[test]
    fn test_transfer_config_with_pretrained() {
        let config = TransferLearningConfig::new()
            .with_pretrained("/path/to/model.apr");
        assert_eq!(
            config.pretrained_model,
            Some(PathBuf::from("/path/to/model.apr"))
        );
    }

    #[test]
    fn test_transfer_config_with_learning_rate() {
        let config = TransferLearningConfig::new()
            .with_learning_rate(0.0001);
        assert!((config.fine_tune_lr - 0.0001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_config_with_feature_extraction_only() {
        let config = TransferLearningConfig::new()
            .with_feature_extraction_only(true);
        assert!(config.feature_extraction_only);
    }

    #[test]
    fn test_transfer_config_freeze_layer() {
        let config = TransferLearningConfig::new()
            .freeze_layer("encoder")
            .freeze_layer("embedding");
        assert_eq!(config.frozen_layers.len(), 2);
        assert!(config.frozen_layers.contains(&"encoder".to_string()));
    }

    #[test]
    fn test_transfer_config_with_epochs() {
        let config = TransferLearningConfig::new()
            .with_epochs(20);
        assert_eq!(config.fine_tune_epochs, 20);
    }

    #[test]
    fn test_transfer_config_with_batch_size() {
        let config = TransferLearningConfig::new()
            .with_batch_size(64);
        assert_eq!(config.batch_size, 64);
    }

    #[test]
    fn test_transfer_config_with_early_stopping() {
        let config = TransferLearningConfig::new()
            .with_early_stopping(5);
        assert_eq!(config.early_stopping_patience, 5);
    }

    #[test]
    fn test_transfer_config_with_validation_split() {
        let config = TransferLearningConfig::new()
            .with_validation_split(0.3);
        assert!((config.validation_split - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_config_validation_split_clamped() {
        let config = TransferLearningConfig::new()
            .with_validation_split(0.8);
        assert!((config.validation_split - 0.5).abs() < f64::EPSILON);

        let config2 = TransferLearningConfig::new()
            .with_validation_split(-0.1);
        assert!((config2.validation_split - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_config_is_configured() {
        let config1 = TransferLearningConfig::new();
        assert!(!config1.is_configured());

        let config2 = TransferLearningConfig::new()
            .with_pretrained("/path/to/model.apr");
        assert!(config2.is_configured());
    }

    // ============================================================================
    // EXTREME TDD: TransferStatus Tests
    // ============================================================================

    #[test]
    fn test_transfer_status_disabled() {
        let status = TransferStatus::Disabled;
        assert!(!status.is_active());
        assert!(!status.is_complete());
        assert!(!status.is_error());
    }

    #[test]
    fn test_transfer_status_pretrained_loaded() {
        let status = TransferStatus::PretrainedLoaded {
            model_path: PathBuf::from("/path/to/model.apr"),
            frozen_layers: 2,
        };
        assert!(status.is_active());
        assert!(!status.is_complete());
    }

    #[test]
    fn test_transfer_status_fine_tuning() {
        let status = TransferStatus::FineTuning {
            epoch: 5,
            total_epochs: 10,
            loss: 0.5,
        };
        assert!(status.is_active());
        assert!(!status.is_complete());
    }

    #[test]
    fn test_transfer_status_complete() {
        let status = TransferStatus::Complete {
            accuracy: 0.95,
            epochs_trained: 10,
        };
        assert!(!status.is_active());
        assert!(status.is_complete());
    }

    #[test]
    fn test_transfer_status_error() {
        let status = TransferStatus::Error {
            message: "Model not found".to_string(),
        };
        assert!(!status.is_active());
        assert!(!status.is_complete());
        assert!(status.is_error());
    }

    // ============================================================================
    // EXTREME TDD: TransferLearner Tests
    // ============================================================================

    #[test]
    fn test_transfer_learner_new() {
        let config = TransferLearningConfig::default();
        let learner = TransferLearner::new(config);
        assert_eq!(*learner.status(), TransferStatus::Disabled);
        assert!(learner.history().is_empty());
    }

    #[test]
    fn test_transfer_learner_load_pretrained_no_path() {
        let config = TransferLearningConfig::default();
        let mut learner = TransferLearner::new(config);

        let result = learner.load_pretrained();
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_learner_load_pretrained_not_found() {
        let config = TransferLearningConfig::new()
            .with_pretrained("/nonexistent/model.apr");
        let mut learner = TransferLearner::new(config);

        let result = learner.load_pretrained();
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_learner_record_epoch() {
        let config = TransferLearningConfig::default();
        let mut learner = TransferLearner::new(config);

        learner.record_epoch(1, 0.5, 0.4, 0.8);

        assert_eq!(learner.history().len(), 1);
        assert!(matches!(
            learner.status(),
            TransferStatus::FineTuning { epoch: 1, .. }
        ));
    }

    #[test]
    fn test_transfer_learner_should_early_stop() {
        let config = TransferLearningConfig::new()
            .with_early_stopping(2);
        let mut learner = TransferLearner::new(config);

        // First epoch - improvement
        learner.record_epoch(1, 0.5, 0.4, 0.8);
        assert!(!learner.should_early_stop());

        // Second epoch - no improvement
        learner.record_epoch(2, 0.4, 0.35, 0.75);
        assert!(!learner.should_early_stop());

        // Third epoch - still no improvement (patience exceeded)
        learner.record_epoch(3, 0.35, 0.3, 0.7);
        assert!(learner.should_early_stop());
    }

    #[test]
    fn test_transfer_learner_mark_complete() {
        let config = TransferLearningConfig::default();
        let mut learner = TransferLearner::new(config);

        learner.record_epoch(1, 0.5, 0.4, 0.8);
        learner.mark_complete(0.95);

        assert!(learner.status().is_complete());
        if let TransferStatus::Complete { accuracy, epochs_trained } = learner.status() {
            assert!((*accuracy - 0.95).abs() < f64::EPSILON);
            assert_eq!(*epochs_trained, 1);
        }
    }

    #[test]
    fn test_transfer_learner_mark_error() {
        let config = TransferLearningConfig::default();
        let mut learner = TransferLearner::new(config);

        learner.mark_error("Training failed");

        assert!(learner.status().is_error());
    }

    #[test]
    fn test_transfer_learner_reset() {
        let config = TransferLearningConfig::default();
        let mut learner = TransferLearner::new(config);

        learner.record_epoch(1, 0.5, 0.4, 0.8);
        learner.reset();

        assert_eq!(*learner.status(), TransferStatus::Disabled);
        assert!(learner.history().is_empty());
    }

    #[test]
    fn test_transfer_learner_default() {
        let learner = TransferLearner::default();
        assert_eq!(*learner.status(), TransferStatus::Disabled);
    }
}
