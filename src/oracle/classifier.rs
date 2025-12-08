//! Oracle Classifier using `RandomForest` from aprender
//!
//! ML-powered error classification with confidence scores.
//! Uses aprender's `RandomForestClassifier` for accurate predictions.
//!
//! # References
//! - [3] Breiman, L. (2001). "Random Forests." Machine Learning, 45(1), 5-32.
//! - [4] Bifet & Gavalda (2007). "Learning from Time-Changing Data with Adaptive Windowing."
//! - [10] Buitinck et al. (2013). Scikit-learn API design.

use aprender::prelude::Matrix;
use aprender::tree::RandomForestClassifier;
use aprender::online::drift::{DriftDetector, DriftDetectorFactory};
use super::patterns::{FixSuggestion, PatternStore};
use super::{ErrorCategory, ErrorFeatures, OracleConfig};

/// Classification result from the Oracle
#[derive(Debug, Clone)]
pub struct Classification {
    /// Predicted error category
    pub category: ErrorCategory,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Suggested fixes from pattern store
    pub suggestions: Vec<FixSuggestion>,

    /// Whether auto-fix is recommended
    pub should_auto_fix: bool,
}

impl Classification {
    /// Create a new classification result
    #[must_use]
    pub fn new(category: ErrorCategory, confidence: f64) -> Self {
        Self {
            category,
            confidence,
            suggestions: Vec::new(),
            should_auto_fix: false,
        }
    }

    /// Add suggestions from pattern store
    pub fn with_suggestions(mut self, suggestions: Vec<FixSuggestion>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Mark as auto-fixable based on confidence threshold
    pub fn with_auto_fix(mut self, threshold: f64) -> Self {
        self.should_auto_fix = self.confidence >= threshold
            && !self.suggestions.is_empty();
        self
    }
}

/// Compilation error input for classification
#[derive(Debug, Clone)]
pub struct CompilationError {
    /// Error code (e.g., "E0308")
    pub code: Option<String>,

    /// Error message text
    pub message: String,

    /// Source file path
    pub file_path: Option<String>,

    /// Line number
    pub line: Option<u32>,

    /// Column number
    pub column: Option<u32>,
}

impl CompilationError {
    /// Create a new compilation error
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
            file_path: None,
            line: None,
            column: None,
        }
    }

    /// Add error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add file path
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Add location
    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// Oracle metadata for training and versioning
#[derive(Debug, Clone, Default)]
pub struct OracleMetadata {
    /// Training sample count
    pub sample_count: usize,

    /// Training accuracy
    pub training_accuracy: f64,

    /// Model version
    pub version: String,

    /// Training timestamp
    pub trained_at: Option<String>,
}

/// Ruchy Oracle: ML-powered transpilation error classifier
///
/// Uses `RandomForestClassifier` from aprender with 73 features extracted from
/// rustc error messages. Provides fix suggestions from pattern store.
///
/// Uses `aprender::online::drift` for drift detection (ADWIN by default).
pub struct RuchyOracle {
    /// Configuration
    config: OracleConfig,

    /// Training metadata
    metadata: OracleMetadata,

    /// Pattern store for fix suggestions
    pattern_store: PatternStore,

    /// Drift detector for model monitoring (ADWIN from aprender)
    drift_detector: Box<dyn DriftDetector>,

    /// Whether ML model is trained (false = use rule-based fallback)
    is_trained: bool,

    /// `RandomForest` classifier from aprender
    classifier: Option<RandomForestClassifier>,

    /// Training data: features (kept for k-NN fallback)
    training_features: Vec<Vec<f32>>,

    /// Training data: labels (kept for k-NN fallback)
    training_labels: Vec<usize>,
}

impl RuchyOracle {
    /// Create a new Oracle with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(OracleConfig::default())
    }

    /// Create a new Oracle with custom configuration
    #[must_use]
    pub fn with_config(config: OracleConfig) -> Self {
        Self {
            config,
            metadata: OracleMetadata::default(),
            pattern_store: PatternStore::new(),
            // Use ADWIN from aprender (recommended for both sudden and gradual drift)
            drift_detector: DriftDetectorFactory::recommended(),
            is_trained: false,
            classifier: None,
            training_features: Vec::new(),
            training_labels: Vec::new(),
        }
    }

    /// Load a pre-trained Oracle from .apr file
    pub fn load(path: &std::path::Path) -> Result<Self, OracleError> {
        // For now, return a new untrained Oracle
        // Full implementation will load from SafeTensors format
        if !path.exists() {
            return Err(OracleError::ModelNotFound(path.to_path_buf()));
        }
        Ok(Self::new())
    }

    /// Load or train the Oracle
    pub fn load_or_train() -> Result<Self, OracleError> {
        let model_path = std::path::Path::new("oracle_model.apr");
        if model_path.exists() {
            Self::load(model_path)
        } else {
            let mut oracle = Self::new();
            oracle.train_from_examples()?;
            Ok(oracle)
        }
    }

    /// Train the Oracle on labeled samples
    pub fn train(&mut self, features: &[Vec<f32>], labels: &[usize]) -> Result<(), OracleError> {
        if features.is_empty() {
            return Err(OracleError::EmptyTrainingData);
        }

        if features.len() != labels.len() {
            return Err(OracleError::MismatchedData {
                features: features.len(),
                labels: labels.len(),
            });
        }

        // Store training data for k-NN fallback
        self.training_features = features.to_vec();
        self.training_labels = labels.to_vec();

        // Train RandomForestClassifier from aprender
        let n_samples = features.len();
        let n_features = features.first().map_or(0, Vec::len);

        // Flatten features into row-major format for Matrix
        let flat_features: Vec<f32> = features.iter().flatten().copied().collect();
        let x = Matrix::from_vec(n_samples, n_features, flat_features)
            .expect("feature dimensions should match");

        // Create and train RandomForest
        let mut rf = RandomForestClassifier::new(10) // 10 trees for small dataset
            .with_max_depth(5)
            .with_random_state(42);

        if let Err(e) = rf.fit(&x, labels) {
            // Fallback to k-NN if RF training fails
            eprintln!("RandomForest training failed, using k-NN: {e}");
        } else {
            self.classifier = Some(rf);
        }

        // Update metadata
        self.metadata.sample_count = features.len();
        self.metadata.version = "2.0.0-rf".to_string();
        self.is_trained = true;

        Ok(())
    }

    /// Train from examples/ directory
    pub fn train_from_examples(&mut self) -> Result<(), OracleError> {
        // Bootstrap with some synthetic labeled data
        let samples = self.generate_bootstrap_samples();

        let features: Vec<Vec<f32>> = samples.iter()
            .map(|(msg, code, _)| {
                ErrorFeatures::extract(msg, code.as_deref()).to_vec()
            })
            .collect();

        let labels: Vec<usize> = samples.iter()
            .map(|(_, _, cat)| cat.to_index())
            .collect();

        self.train(&features, &labels)
    }

    /// Generate bootstrap training samples
    fn generate_bootstrap_samples(&self) -> Vec<(String, Option<String>, ErrorCategory)> {
        vec![
            // TypeMismatch samples
            ("mismatched types: expected `i32`, found `String`".into(),
             Some("E0308".into()), ErrorCategory::TypeMismatch),
            ("expected `&str`, found `String`".into(),
             Some("E0308".into()), ErrorCategory::TypeMismatch),
            ("type mismatch: expected Vec<i32>".into(),
             Some("E0271".into()), ErrorCategory::TypeMismatch),

            // BorrowChecker samples
            ("borrow of moved value: `x`".into(),
             Some("E0382".into()), ErrorCategory::BorrowChecker),
            ("cannot borrow `x` as mutable".into(),
             Some("E0502".into()), ErrorCategory::BorrowChecker),
            ("value moved here".into(),
             Some("E0382".into()), ErrorCategory::BorrowChecker),

            // LifetimeError samples
            ("borrowed value does not live long enough".into(),
             Some("E0597".into()), ErrorCategory::LifetimeError),
            ("lifetime `'a` required".into(),
             Some("E0621".into()), ErrorCategory::LifetimeError),

            // TraitBound samples
            ("the trait `Debug` is not implemented".into(),
             Some("E0277".into()), ErrorCategory::TraitBound),
            ("no method named `foo` found".into(),
             Some("E0599".into()), ErrorCategory::TraitBound),

            // MissingImport samples
            ("cannot find type `HashMap` in this scope".into(),
             Some("E0433".into()), ErrorCategory::MissingImport),
            ("cannot find value `x` in this scope".into(),
             Some("E0425".into()), ErrorCategory::MissingImport),
            ("failed to resolve: use of undeclared type".into(),
             Some("E0433".into()), ErrorCategory::MissingImport),
            ("use of undeclared type or module".into(),
             Some("E0412".into()), ErrorCategory::MissingImport),

            // MutabilityError samples
            ("cannot borrow `x` as mutable, as it is not declared as mutable".into(),
             Some("E0596".into()), ErrorCategory::MutabilityError),
            ("cannot assign to `x`, as it is not declared as mutable".into(),
             Some("E0594".into()), ErrorCategory::MutabilityError),

            // SyntaxError samples
            ("expected `;`, found `}`".into(),
             Some("E0658".into()), ErrorCategory::SyntaxError),
            ("this function takes 2 arguments but 1 was supplied".into(),
             Some("E0061".into()), ErrorCategory::SyntaxError),

            // Module resolution samples (MissingImport)
            ("Module 'scanner' not resolved".into(),
             None, ErrorCategory::MissingImport),
            ("Failed to resolve module declaration".into(),
             None, ErrorCategory::MissingImport),
            ("Module 'utils' not found".into(),
             None, ErrorCategory::MissingImport),
            ("Failed to find module".into(),
             None, ErrorCategory::MissingImport),

            // Method not found samples (TraitBound)
            ("no method named `resolve` found for struct".into(),
             Some("E0599".into()), ErrorCategory::TraitBound),
            ("no method named `len` found".into(),
             Some("E0599".into()), ErrorCategory::TraitBound),
            ("method not found in".into(),
             Some("E0599".into()), ErrorCategory::TraitBound),

            // Clippy lint samples (SyntaxError - style issues)
            ("item in documentation is missing backticks".into(),
             None, ErrorCategory::SyntaxError),
            ("called `map(<f>).unwrap_or(<a>)` on an Option value".into(),
             None, ErrorCategory::SyntaxError),
            ("redundant closure".into(),
             None, ErrorCategory::SyntaxError),
            ("this function has too many arguments".into(),
             None, ErrorCategory::SyntaxError),
            ("this argument is passed by value, but not consumed".into(),
             None, ErrorCategory::SyntaxError),
        ]
    }

    /// Classify a compilation error
    #[must_use]
    pub fn classify(&self, error: &CompilationError) -> Classification {
        // PRIORITY 1: Known error codes are 100% reliable - use them directly
        // This ensures error codes like E0597->LifetimeError are never overridden by ML
        if let Some(ref code) = error.code {
            let category = ErrorCategory::from_error_code(code);
            if category != ErrorCategory::Other {
                // Known error code - use rule-based with high confidence
                let suggestions = self.pattern_store.query(
                    category,
                    &error.message,
                    self.config.similarity_threshold,
                );
                return Classification::new(category, 0.95)
                    .with_suggestions(suggestions)
                    .with_auto_fix(self.config.confidence_threshold);
            }
        }

        // Extract features for ML-based classification (unknown error codes)
        let features = ErrorFeatures::extract(
            &error.message,
            error.code.as_deref(),
        );

        // Predict category using ML or rules
        let (category, confidence) = if self.is_trained {
            self.predict_with_model(&features)
        } else {
            self.predict_with_rules(&features)
        };

        // Get suggestions from pattern store
        let suggestions = self.pattern_store.query(
            category,
            &error.message,
            self.config.similarity_threshold,
        );

        Classification::new(category, confidence)
            .with_suggestions(suggestions)
            .with_auto_fix(self.config.confidence_threshold)
    }

    /// Predict using trained `RandomForest` model (with k-NN fallback)
    fn predict_with_model(&self, features: &ErrorFeatures) -> (ErrorCategory, f64) {
        // Try RandomForest first
        if let Some(ref rf) = self.classifier {
            let query = features.to_vec();
            let n_features = query.len();
            let x = match Matrix::from_vec(1, n_features, query) {
                Ok(m) => m,
                Err(_) => return self.predict_with_knn(features),
            };

            let predictions = rf.predict(&x);
            if let Some(&label) = predictions.first() {
                let category = ErrorCategory::from_index(label)
                    .unwrap_or(ErrorCategory::Other);

                // Get confidence from prediction probabilities
                let proba = rf.predict_proba(&x);
                // proba is Matrix<f32>, get first row's max probability
                let confidence: f64 = if proba.shape().0 > 0 && proba.shape().1 > 0 {
                    let row = proba.row(0);
                    let max_idx = row.argmax();
                    f64::from(row[max_idx])
                } else {
                    0.8
                };

                return (category, confidence);
            }
        }

        // Fallback to k-NN if RandomForest unavailable
        self.predict_with_knn(features)
    }

    /// Predict using k-NN fallback
    fn predict_with_knn(&self, features: &ErrorFeatures) -> (ErrorCategory, f64) {
        if self.training_features.is_empty() {
            return self.predict_with_rules(features);
        }

        let query = features.to_vec();
        let mut best_dist = f64::MAX;
        let mut best_label = 0usize;

        for (i, train_features) in self.training_features.iter().enumerate() {
            let dist = euclidean_distance(&query, train_features);
            if dist < best_dist {
                best_dist = dist;
                best_label = self.training_labels[i];
            }
        }

        let category = ErrorCategory::from_index(best_label)
            .unwrap_or(ErrorCategory::Other);

        // Convert distance to confidence (closer = higher confidence)
        let confidence = (1.0 / (1.0 + best_dist)).min(1.0);

        (category, confidence)
    }

    /// Predict using rule-based fallback
    fn predict_with_rules(&self, features: &ErrorFeatures) -> (ErrorCategory, f64) {
        let category = features.predict_category_rules();
        let confidence = if category == ErrorCategory::Other { 0.3 } else { 0.7 };
        (category, confidence)
    }

    /// Check if model is trained
    #[must_use]
    pub fn is_trained(&self) -> bool {
        self.is_trained
    }

    /// Get training metadata
    #[must_use]
    pub fn metadata(&self) -> &OracleMetadata {
        &self.metadata
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &OracleConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: OracleConfig) {
        self.config = config;
    }

    /// Get training data for persistence
    #[must_use]
    pub fn get_training_data(&self) -> (Vec<Vec<f32>>, Vec<usize>) {
        (self.training_features.clone(), self.training_labels.clone())
    }

    /// Record classification result for drift detection
    ///
    /// Uses `aprender::online::drift::DriftDetector::add_element(error: bool)`
    /// where `error=true` indicates an incorrect prediction.
    pub fn record_result(&mut self, predicted: ErrorCategory, actual: ErrorCategory) {
        let error = predicted != actual;
        self.drift_detector.add_element(error);
    }

    /// Check current drift status
    #[must_use]
    pub fn drift_status(&self) -> aprender::online::drift::DriftStatus {
        self.drift_detector.detected_change()
    }

    /// Reset drift detector after handling drift
    pub fn reset_drift_detector(&mut self) {
        self.drift_detector.reset();
    }

    // =========================================================================
    // ORACLE-002: Auto-train methods
    // =========================================================================

    /// Record an error for corpus collection
    pub fn record_error(&mut self, message: &str, category: ErrorCategory) {
        let features = ErrorFeatures::extract(message, None);
        self.training_features.push(features.to_vec());
        self.training_labels.push(category.to_index());
    }

    /// Check if retraining is needed (threshold: 100 new samples)
    #[must_use]
    pub fn should_retrain(&self) -> bool {
        const RETRAIN_THRESHOLD: usize = 100;
        self.training_labels.len() > 30 + RETRAIN_THRESHOLD // 30 = bootstrap
    }

    /// Retrain the model with accumulated samples
    pub fn retrain(&mut self) -> Result<(), OracleError> {
        if self.training_features.is_empty() {
            return Err(OracleError::EmptyTrainingData);
        }
        self.train(&self.training_features.clone(), &self.training_labels.clone())
    }

    /// Check if drift has been detected
    #[must_use]
    pub fn drift_detected(&self) -> bool {
        matches!(
            self.drift_detector.detected_change(),
            aprender::online::drift::DriftStatus::Drift
        )
    }

    /// Parse rustc error output into corpus samples
    #[must_use]
    pub fn parse_rustc_errors(stderr: &str) -> Vec<super::Sample> {
        use super::{Sample, SampleSource};
        use regex::Regex;

        let error_re = Regex::new(r"error\[E(\d{4})\]:\s*(.+?)(?:\n|$)").unwrap();
        let mut samples = Vec::new();

        for cap in error_re.captures_iter(stderr) {
            let code = format!("E{}", &cap[1]);
            let message = format!("error[{}]: {}", code, &cap[2]);
            let category = ErrorCategory::from_error_code(&code);
            samples.push(Sample::new(message, Some(code), category)
                .with_source(SampleSource::Production));
        }

        samples
    }

    /// Generate synthetic training samples
    #[must_use]
    pub fn generate_synthetic_samples(count: usize) -> Vec<super::Sample> {
        use super::{Sample, SampleSource};

        let templates = [
            ("error[E0308]: mismatched types", "E0308", ErrorCategory::TypeMismatch),
            ("error[E0271]: type mismatch resolving", "E0271", ErrorCategory::TypeMismatch),
            ("error[E0382]: borrow of moved value", "E0382", ErrorCategory::BorrowChecker),
            ("error[E0502]: cannot borrow as mutable", "E0502", ErrorCategory::BorrowChecker),
            ("error[E0597]: borrowed value does not live long enough", "E0597", ErrorCategory::LifetimeError),
            ("error[E0621]: explicit lifetime required", "E0621", ErrorCategory::LifetimeError),
            ("error[E0277]: the trait bound is not satisfied", "E0277", ErrorCategory::TraitBound),
            ("error[E0599]: no method named", "E0599", ErrorCategory::TraitBound),
            ("error[E0433]: failed to resolve", "E0433", ErrorCategory::MissingImport),
            ("error[E0412]: cannot find type", "E0412", ErrorCategory::MissingImport),
            ("error[E0596]: cannot borrow as mutable", "E0596", ErrorCategory::MutabilityError),
            ("error[E0594]: cannot assign to immutable", "E0594", ErrorCategory::MutabilityError),
            ("error[E0658]: syntax error", "E0658", ErrorCategory::SyntaxError),
            ("error[E0061]: wrong number of arguments", "E0061", ErrorCategory::SyntaxError),
            ("error: unknown error", "", ErrorCategory::Other),
            ("error: internal compiler error", "", ErrorCategory::Other),
        ];

        let per_template = count / templates.len();
        let mut samples = Vec::with_capacity(count);

        for (i, (msg, code, cat)) in templates.iter().cycle().take(count).enumerate() {
            // Add variation to message
            let varied_msg = format!("{} (variant {})", msg, i % (per_template.max(1)));
            let code_opt = if code.is_empty() { None } else { Some((*code).to_string()) };
            samples.push(Sample::new(varied_msg, code_opt, *cat)
                .with_source(SampleSource::Synthetic));
        }

        samples
    }
}

impl Default for RuchyOracle {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Euclidean distance between two feature vectors
fn euclidean_distance(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return f64::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (f64::from(*x) - f64::from(*y)).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Oracle errors
#[derive(Debug, Clone)]
pub enum OracleError {
    /// Model file not found
    ModelNotFound(std::path::PathBuf),

    /// Empty training data
    EmptyTrainingData,

    /// Mismatched features and labels
    MismatchedData { features: usize, labels: usize },

    /// IO error
    IoError(String),

    /// Training failed
    TrainingFailed(String),
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleError::ModelNotFound(path) => {
                write!(f, "Oracle model not found: {}", path.display())
            }
            OracleError::EmptyTrainingData => {
                write!(f, "Cannot train on empty data")
            }
            OracleError::MismatchedData { features, labels } => {
                write!(f, "Mismatched data: {features} features, {labels} labels")
            }
            OracleError::IoError(msg) => write!(f, "IO error: {msg}"),
            OracleError::TrainingFailed(msg) => write!(f, "Training failed: {msg}"),
        }
    }
}

impl std::error::Error for OracleError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Classifier Tests
    // ============================================================================

    #[test]
    fn test_classification_new() {
        let classification = Classification::new(ErrorCategory::TypeMismatch, 0.95);
        assert_eq!(classification.category, ErrorCategory::TypeMismatch);
        assert!((classification.confidence - 0.95).abs() < f64::EPSILON);
        assert!(classification.suggestions.is_empty());
        assert!(!classification.should_auto_fix);
    }

    #[test]
    fn test_classification_with_auto_fix_above_threshold() {
        let suggestions = vec![FixSuggestion::new("add .to_string()")];
        let classification = Classification::new(ErrorCategory::TypeMismatch, 0.95)
            .with_suggestions(suggestions)
            .with_auto_fix(0.85);

        assert!(classification.should_auto_fix);
    }

    #[test]
    fn test_classification_with_auto_fix_below_threshold() {
        let suggestions = vec![FixSuggestion::new("add .to_string()")];
        let classification = Classification::new(ErrorCategory::TypeMismatch, 0.80)
            .with_suggestions(suggestions)
            .with_auto_fix(0.85);

        assert!(!classification.should_auto_fix);
    }

    #[test]
    fn test_classification_no_auto_fix_without_suggestions() {
        let classification = Classification::new(ErrorCategory::TypeMismatch, 0.95)
            .with_auto_fix(0.85);

        assert!(!classification.should_auto_fix); // No suggestions
    }

    #[test]
    fn test_compilation_error_new() {
        let error = CompilationError::new("mismatched types");
        assert_eq!(error.message, "mismatched types");
        assert!(error.code.is_none());
    }

    #[test]
    fn test_compilation_error_with_code() {
        let error = CompilationError::new("mismatched types")
            .with_code("E0308");
        assert_eq!(error.code, Some("E0308".to_string()));
    }

    #[test]
    fn test_compilation_error_with_location() {
        let error = CompilationError::new("error")
            .with_file("main.rs")
            .with_location(10, 5);

        assert_eq!(error.file_path, Some("main.rs".to_string()));
        assert_eq!(error.line, Some(10));
        assert_eq!(error.column, Some(5));
    }

    #[test]
    fn test_oracle_new() {
        let oracle = RuchyOracle::new();
        assert!(!oracle.is_trained());
        assert_eq!(oracle.metadata().sample_count, 0);
    }

    #[test]
    fn test_oracle_with_config() {
        let config = OracleConfig {
            confidence_threshold: 0.90,
            ..Default::default()
        };
        let oracle = RuchyOracle::with_config(config);
        assert!((oracle.config().confidence_threshold - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn test_oracle_train_empty_data() {
        let mut oracle = RuchyOracle::new();
        let result = oracle.train(&[], &[]);
        assert!(matches!(result, Err(OracleError::EmptyTrainingData)));
    }

    #[test]
    fn test_oracle_train_mismatched_data() {
        let mut oracle = RuchyOracle::new();
        let features = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let labels = vec![0]; // Only 1 label for 2 features

        let result = oracle.train(&features, &labels);
        assert!(matches!(result, Err(OracleError::MismatchedData { .. })));
    }

    #[test]
    fn test_oracle_train_success() {
        let mut oracle = RuchyOracle::new();
        let features = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let labels = vec![0, 1];

        let result = oracle.train(&features, &labels);
        assert!(result.is_ok());
        assert!(oracle.is_trained());
        assert_eq!(oracle.metadata().sample_count, 2);
    }

    #[test]
    fn test_oracle_classify_type_mismatch() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap training");

        let error = CompilationError::new("mismatched types: expected `i32`, found `String`")
            .with_code("E0308");

        let classification = oracle.classify(&error);
        assert_eq!(classification.category, ErrorCategory::TypeMismatch);
        // Confidence depends on distance to nearest training sample
        assert!(classification.confidence > 0.0);
    }

    #[test]
    fn test_oracle_classify_borrow_checker() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap training");

        let error = CompilationError::new("borrow of moved value")
            .with_code("E0382");

        let classification = oracle.classify(&error);
        assert_eq!(classification.category, ErrorCategory::BorrowChecker);
    }

    #[test]
    fn test_oracle_classify_untrained_fallback() {
        let oracle = RuchyOracle::new(); // Not trained

        let error = CompilationError::new("mismatched types")
            .with_code("E0308");

        let classification = oracle.classify(&error);
        assert_eq!(classification.category, ErrorCategory::TypeMismatch);
        // Known error codes give 0.95 confidence (high reliability)
        assert!((classification.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_euclidean_distance_same() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((euclidean_distance(&a, &b) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_euclidean_distance_different() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_euclidean_distance_different_lengths() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(euclidean_distance(&a, &b), f64::MAX);
    }

    #[test]
    fn test_oracle_error_display() {
        let err = OracleError::EmptyTrainingData;
        assert_eq!(format!("{err}"), "Cannot train on empty data");

        let err = OracleError::MismatchedData { features: 10, labels: 5 };
        assert!(format!("{err}").contains("10 features"));
    }

    #[test]
    fn test_oracle_default() {
        let oracle = RuchyOracle::default();
        assert!(!oracle.is_trained());
    }

    #[test]
    fn test_oracle_record_result() {
        let mut oracle = RuchyOracle::new();
        oracle.record_result(ErrorCategory::TypeMismatch, ErrorCategory::TypeMismatch);
        // No panic = success
    }
}
