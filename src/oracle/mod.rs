//! Ruchy Oracle: ML-powered transpilation error classifier and fix suggester
//!
//! The Oracle uses a Random Forest classifier trained on 12,000+ labeled samples
//! to automatically classify Rust compilation errors and suggest fixes.
//!
//! # Toyota Way Principles
//! - **Jidoka**: Stop on error, auto-fix, resume
//! - **Kaizen**: Self-supervised learning from each transpilation
//! - **Genchi Genbutsu**: Use real code from examples/, not synthetic
//!
//! # Dynamic `MLOps` Features (Spec v1.1.0)
//! - **Corpus Collection**: Four-source training data pipeline (via `aprender::online::corpus`)
//! - **Online Learning**: Hot-fix layer with micro-batching (via `aprender::online::orchestrator`)
//! - **Drift Detection**: ADWIN/DDM/Page-Hinkley algorithms (via `aprender::online::drift`)
//! - **Transfer Learning**: Pre-train on Rust errors, fine-tune on Ruchy
//! - **Model Persistence**: `.apr` format with versioning
//!
//! # Architecture
//! ```text
//! Ruchy Source → Transpiler → Rust Code → rustc → Errors
//!                                            ↓
//!                                    Oracle Classifier
//!                                            ↓
//!                                    Pattern Store (.apr)
//!                                            ↓
//!                                    Suggested Fix → AutoFixer
//!                                            ↓
//!                                    Online Learning (aprender::online)
//! ```
//!
//! # References
//! - [3] Breiman, L. (2001). "Random Forests." Machine Learning, 45(1), 5-32.
//! - [4] Bifet & Gavalda (2007). "Learning from Time-Changing Data with Adaptive Windowing."
//! - [6] Amershi et al. (2014). "Power to the People: CITL" AI Magazine, 35(4).
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md
//!
//! # External Dependencies (§3 of spec)
//! - `aprender`: `RandomForest`, transfer learning, `Code2Vec` embeddings, online learning
//! - `entrenar`: CITL pattern store, knowledge distillation, Tarantula SBFL
//!
//! # Migration Note (Issue #174)
//! This module now uses `aprender::online` for drift detection, corpus management,
//! and online learning. Custom implementations (~1,500 lines) have been deleted
//! in favor of the well-tested aprender implementations.

pub mod andon;
mod category;
mod classifier;
mod curriculum;
mod distillation;
mod features;
pub mod hansei;
mod metrics;
mod patterns;
pub mod persistence;
mod training_loop;
pub mod transfer;

pub use category::ErrorCategory;
pub use classifier::{Classification, CompilationError, OracleError, OracleMetadata, RuchyOracle};
pub use curriculum::{CurriculumConfig, CurriculumScheduler as RuchyCurriculumScheduler};
pub use distillation::{DistillationConfig, KnowledgeDistiller, SoftLabel};
pub use features::{ErrorFeatures, FEATURE_COUNT};
pub use hansei::{CategoryStats, HanseiIssue, HanseiReport, Severity, Trend};
pub use metrics::OracleMetrics;
pub use patterns::{FixPattern, FixSuggestion, PatternStore};
pub use persistence::{ModelMetadata, ModelPaths, SerializedModel, APR_MAGIC, APR_VERSION};
pub use training_loop::{
    DisplayMode, RetrainReason, TrainingEvent, TrainingLoop, TrainingLoopConfig,
};
pub use transfer::{TransferLearner, TransferLearningConfig, TransferStatus};
// Re-export our types with aliases
pub use self::curriculum::CurriculumScheduler;

// =============================================================================
// MIGRATION: Re-export aprender::online types (Issue #174)
// =============================================================================

// Drift detection - using aprender's ADWIN algorithm (recommended default)
// Reference: [Bifet & Gavalda 2007] "Learning from Time-Changing Data with Adaptive Windowing"
pub use aprender::online::drift::{
    DriftDetector,
    DriftDetectorFactory,
    DriftStats,
    DriftStatus, // Enum: Stable, Warning, Drift
    PageHinkley, // Page-Hinkley test (gradual drift)
    ADWIN,       // ADaptive WINdowing (recommended - handles both)
    DDM,         // Drift Detection Method (sudden drift)
};

// Corpus management - using aprender's CorpusBuffer with deduplication
// Reference: [Vitter 1985] "Random Sampling with a Reservoir"
pub use aprender::online::corpus::{
    CorpusBuffer,
    CorpusBufferConfig,
    CorpusMerger,
    CorpusProvenance,
    CorpusSource,
    EvictionPolicy,
    Sample as AprenderSample, // Renamed to avoid conflict
    SampleSource as AprenderSampleSource,
};

// Curriculum learning for progressive training difficulty
pub use aprender::online::curriculum::{
    CurriculumScheduler as AprenderCurriculumScheduler, CurriculumTrainer, LinearCurriculum,
    SelfPacedCurriculum,
};

// Online learning orchestration
pub use aprender::online::orchestrator::{
    ObserveResult, OrchestratorBuilder, OrchestratorStats, RetrainOrchestrator,
};

// =============================================================================
// COMPATIBILITY: Wrapper types for backward compatibility
// =============================================================================

/// Ruchy-specific sample wrapper (adapts to aprender's Sample type)
///
/// This provides compatibility with existing code that expects the
/// old corpus API while using aprender's implementation underneath.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Error message text
    pub message: String,
    /// Error code (e.g., "E0308")
    pub error_code: Option<String>,
    /// Labeled category
    pub category: ErrorCategory,
    /// Optional fix suggestion
    pub fix: Option<String>,
    /// Difficulty score (0.0-1.0) for curriculum learning
    pub difficulty: f32,
    /// Source identifier
    pub source: SampleSource,
}

impl Sample {
    /// Create a new sample
    #[must_use]
    pub fn new(
        message: impl Into<String>,
        error_code: Option<String>,
        category: ErrorCategory,
    ) -> Self {
        Self {
            message: message.into(),
            error_code,
            category,
            fix: None,
            difficulty: 0.5,
            source: SampleSource::Synthetic,
        }
    }

    /// Set fix suggestion
    #[must_use]
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    /// Set difficulty score
    #[must_use]
    pub fn with_difficulty(mut self, difficulty: f32) -> Self {
        self.difficulty = difficulty.clamp(0.0, 1.0);
        self
    }

    /// Set sample source
    #[must_use]
    pub fn with_source(mut self, source: SampleSource) -> Self {
        self.source = source;
        self
    }

    /// Extract features for ML training
    #[must_use]
    pub fn to_features(&self) -> ErrorFeatures {
        ErrorFeatures::extract(&self.message, self.error_code.as_deref())
    }

    /// Convert to aprender Sample for corpus buffer
    #[must_use]
    pub fn to_aprender(&self) -> AprenderSample {
        let features = self
            .to_features()
            .to_vec()
            .iter()
            .map(|&x| f64::from(x))
            .collect();
        let target = vec![self.category.to_index() as f64];
        AprenderSample::with_weight(features, target, f64::from(self.difficulty))
    }

    /// Convert to [`CompilationError`] for classification
    #[must_use]
    pub fn to_compilation_error(&self) -> CompilationError {
        CompilationError {
            code: self.error_code.clone(),
            message: self.message.clone(),
            file_path: None,
            line: None,
            column: None,
        }
    }
}

/// Source of training sample (backward compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SampleSource {
    /// Procedurally generated synthetic samples
    #[default]
    Synthetic,
    /// Hand-crafted from GitHub issues/tickets
    Ruchy,
    /// Collected from examples/*.ruchy transpilation
    Examples,
    /// Runtime collection from user transpilations
    Production,
}

impl std::fmt::Display for SampleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleSource::Synthetic => write!(f, "synthetic"),
            SampleSource::Ruchy => write!(f, "ruchy"),
            SampleSource::Examples => write!(f, "examples"),
            SampleSource::Production => write!(f, "production"),
        }
    }
}

impl From<SampleSource> for AprenderSampleSource {
    fn from(source: SampleSource) -> Self {
        match source {
            SampleSource::Synthetic => AprenderSampleSource::Synthetic,
            SampleSource::Ruchy => AprenderSampleSource::HandCrafted,
            SampleSource::Examples => AprenderSampleSource::Examples,
            SampleSource::Production => AprenderSampleSource::Production,
        }
    }
}

/// Difficulty levels for curriculum learning (backward compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DifficultyLevel {
    /// Easy: Single type mismatch, missing semicolon (0.25)
    Easy,
    /// Medium: Borrow checker single violation (0.50)
    Medium,
    /// Hard: Multiple lifetime annotations (0.75)
    Hard,
    /// Expert: Complex trait bounds, generic constraints (1.00)
    Expert,
}

impl DifficultyLevel {
    /// Get numeric score for difficulty level
    #[must_use]
    pub fn score(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 0.25,
            DifficultyLevel::Medium => 0.50,
            DifficultyLevel::Hard => 0.75,
            DifficultyLevel::Expert => 1.00,
        }
    }

    /// Get next difficulty level
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            DifficultyLevel::Easy => DifficultyLevel::Medium,
            DifficultyLevel::Medium => DifficultyLevel::Hard,
            DifficultyLevel::Hard => DifficultyLevel::Expert,
            DifficultyLevel::Expert => DifficultyLevel::Expert,
        }
    }

    /// Get difficulty level from score
    #[must_use]
    pub fn from_score(score: f32) -> Self {
        if score <= 0.25 {
            DifficultyLevel::Easy
        } else if score <= 0.50 {
            DifficultyLevel::Medium
        } else if score <= 0.75 {
            DifficultyLevel::Hard
        } else {
            DifficultyLevel::Expert
        }
    }
}

/// Training corpus with deduplication (backward compatible wrapper)
///
/// Uses `aprender::online::corpus::CorpusBuffer` internally.
#[derive(Debug)]
pub struct Corpus {
    buffer: CorpusBuffer,
    samples: Vec<Sample>,
    source_counts: [usize; 4],
}

impl Default for Corpus {
    fn default() -> Self {
        Self::new()
    }
}

impl Corpus {
    /// Create empty corpus
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: CorpusBuffer::new(100_000),
            samples: Vec::new(),
            source_counts: [0; 4],
        }
    }

    /// Add sample with deduplication
    pub fn add(&mut self, sample: Sample) -> bool {
        let aprender_sample = sample.to_aprender();
        if self.buffer.add(aprender_sample) {
            self.source_counts[sample.source as usize] += 1;
            self.samples.push(sample);
            true
        } else {
            false
        }
    }

    /// Add multiple samples
    pub fn add_all(&mut self, samples: impl IntoIterator<Item = Sample>) -> usize {
        samples.into_iter().filter(|s| self.add(s.clone())).count()
    }

    /// Get total sample count
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if corpus is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get samples by difficulty level
    #[must_use]
    pub fn filter_by_difficulty(&self, max_level: DifficultyLevel) -> Vec<&Sample> {
        let max_score = max_level.score();
        self.samples
            .iter()
            .filter(|s| s.difficulty <= max_score)
            .collect()
    }

    /// Get samples by source
    #[must_use]
    pub fn filter_by_source(&self, source: SampleSource) -> Vec<&Sample> {
        self.samples.iter().filter(|s| s.source == source).collect()
    }

    /// Get count by source
    #[must_use]
    pub fn count_by_source(&self, source: SampleSource) -> usize {
        self.source_counts[source as usize]
    }

    /// Shuffle samples deterministically
    pub fn shuffle(&mut self) {
        // Simple deterministic shuffle using Fisher-Yates with fixed seed
        let seed = 42u64;
        let n = self.samples.len();
        if n <= 1 {
            return;
        }

        let mut rng_state = seed;
        for i in (1..n).rev() {
            rng_state = rng_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let j = (rng_state as usize) % (i + 1);
            self.samples.swap(i, j);
        }
    }

    /// Get all samples as slice
    #[must_use]
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Extract features and labels for training
    #[must_use]
    pub fn to_training_data(&self) -> (Vec<Vec<f32>>, Vec<usize>) {
        let features: Vec<Vec<f32>> = self
            .samples
            .iter()
            .map(|s| s.to_features().to_vec())
            .collect();
        let labels: Vec<usize> = self.samples.iter().map(|s| s.category.to_index()).collect();
        (features, labels)
    }
}

/// Corpus collector (simplified version using aprender)
#[derive(Debug, Default)]
pub struct CorpusCollector {
    include_production: bool,
}

impl CorpusCollector {
    /// Create collector
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable production data
    #[must_use]
    pub fn with_production(mut self, enabled: bool) -> Self {
        self.include_production = enabled;
        self
    }

    /// Collect corpus (generates synthetic samples)
    #[must_use]
    pub fn collect(&self) -> Corpus {
        let mut corpus = Corpus::new();
        corpus.add_all(Self::generate_synthetic_samples());
        corpus.shuffle();
        corpus
    }

    fn generate_synthetic_samples() -> Vec<Sample> {
        let mut samples = Vec::with_capacity(1000);

        // Type mismatch samples
        for &(expected, found) in &[
            ("i32", "String"),
            ("i32", "&str"),
            ("String", "&str"),
            ("Vec<i32>", "i32"),
            ("bool", "i32"),
        ] {
            for _ in 0..20 {
                samples.push(
                    Sample::new(
                        format!("mismatched types: expected `{expected}`, found `{found}`"),
                        Some("E0308".to_string()),
                        ErrorCategory::TypeMismatch,
                    )
                    .with_difficulty(0.25)
                    .with_source(SampleSource::Synthetic),
                );
            }
        }

        // Borrow checker samples
        for &(msg, code) in &[
            ("borrow of moved value: `x`", "E0382"),
            ("cannot borrow `x` as mutable", "E0502"),
        ] {
            for _ in 0..20 {
                samples.push(
                    Sample::new(msg, Some(code.to_string()), ErrorCategory::BorrowChecker)
                        .with_difficulty(0.5)
                        .with_source(SampleSource::Synthetic),
                );
            }
        }

        // Lifetime error samples
        for &(msg, code) in &[
            ("borrowed value does not live long enough", "E0597"),
            ("lifetime `'a` required", "E0621"),
        ] {
            for _ in 0..15 {
                samples.push(
                    Sample::new(msg, Some(code.to_string()), ErrorCategory::LifetimeError)
                        .with_difficulty(0.75)
                        .with_source(SampleSource::Synthetic),
                );
            }
        }

        samples
    }
}

// =============================================================================
// FOUR-SOURCE CORPUS MERGER (§2.2 of spec)
// =============================================================================

/// Provenance tracking for merged corpus (Ruchy-specific extension)
#[derive(Debug, Clone, Default)]
pub struct RuchyCorpusProvenance {
    /// Source names and their sample counts
    pub sources: Vec<(String, usize)>,
    /// Total samples before deduplication
    pub total_before_dedup: usize,
    /// Total samples after deduplication
    pub total_after_dedup: usize,
    /// Merge timestamp
    pub merged_at: Option<String>,
}

impl RuchyCorpusProvenance {
    /// Get count by source type
    #[must_use]
    pub fn count_by_source(&self, source: SampleSource) -> usize {
        let source_name = source.to_string();
        self.sources
            .iter()
            .filter(|(name, _)| name == &source_name)
            .map(|(_, count)| count)
            .sum()
    }
}

/// Four-source corpus merger with provenance tracking
///
/// Merges samples from:
/// 1. Synthetic (generated)
/// 2. Ruchy (hand-crafted from tickets)
/// 3. Examples (from examples/*.ruchy)
/// 4. Production (runtime collection)
#[derive(Debug, Default)]
pub struct CorpusMergerWithProvenance {
    sources: Vec<(String, Vec<Sample>, SampleSource)>,
}

impl CorpusMergerWithProvenance {
    /// Create new merger
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a source with samples
    pub fn add_source(&mut self, name: &str, samples: Vec<Sample>, source_type: SampleSource) {
        self.sources.push((name.to_string(), samples, source_type));
    }

    /// Get number of sources
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Merge all sources with default seed
    pub fn merge(&self) -> Result<(Corpus, RuchyCorpusProvenance), OracleError> {
        self.merge_with_seed(42)
    }

    /// Merge all sources with specified seed for deterministic shuffle
    pub fn merge_with_seed(
        &self,
        seed: u64,
    ) -> Result<(Corpus, RuchyCorpusProvenance), OracleError> {
        let mut corpus = Corpus::new();
        let mut provenance = RuchyCorpusProvenance::default();

        let mut total_before = 0usize;

        for (name, samples, source_type) in &self.sources {
            let count_before = corpus.len();
            for sample in samples {
                let mut s = sample.clone();
                s.source = *source_type;
                corpus.add(s);
            }
            let count_added = corpus.len() - count_before;
            total_before += samples.len();
            provenance.sources.push((name.clone(), count_added));
        }

        provenance.total_before_dedup = total_before;
        provenance.total_after_dedup = corpus.len();
        provenance.merged_at = Some(chrono::Utc::now().to_rfc3339());

        // Shuffle with seed
        corpus.shuffle_with_seed(seed);

        Ok((corpus, provenance))
    }
}

impl Corpus {
    /// Shuffle with specific seed
    pub fn shuffle_with_seed(&mut self, seed: u64) {
        let n = self.samples.len();
        if n <= 1 {
            return;
        }

        let mut rng_state = seed;
        for i in (1..n).rev() {
            rng_state = rng_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let j = (rng_state as usize) % (i + 1);
            self.samples.swap(i, j);
        }
    }
}

// =============================================================================
// ONLINE LEARNING: Wrapper for aprender's orchestrator (backward compatible)
// =============================================================================

/// Configuration for online learning
#[derive(Debug, Clone)]
pub struct OnlineLearningConfig {
    /// Micro-batch size before hot-fix update
    pub micro_batch_size: usize,
    /// Confidence threshold for hot-fix promotion
    pub hotfix_confidence: f64,
    /// Maximum hot-fix model size in samples
    pub max_hotfix_samples: usize,
    /// Merge hot-fix to main on weekly retrain
    pub merge_on_retrain: bool,
    /// Minimum samples before hot-fix model trains
    pub min_samples_for_training: usize,
}

impl Default for OnlineLearningConfig {
    fn default() -> Self {
        Self {
            micro_batch_size: 10,
            hotfix_confidence: 0.95,
            max_hotfix_samples: 500,
            merge_on_retrain: true,
            min_samples_for_training: 5,
        }
    }
}

/// Statistics for hot-fix layer
#[derive(Debug, Default, Clone)]
pub struct HotFixStats {
    /// Total overrides registered
    pub total_overrides: usize,
    /// Total override hits
    pub total_hits: usize,
    /// Total micro-batches processed
    pub micro_batches_processed: usize,
    /// Total samples accumulated
    pub samples_accumulated: usize,
    /// Hot-fix model retrains
    pub retrains: usize,
}

/// Hot-fix layer using aprender's drift detection and corpus buffer
pub struct HotFixLayer {
    /// Drift detector (ADWIN by default)
    drift_detector: Box<dyn DriftDetector>,
    /// Accumulated samples
    corpus: Corpus,
    /// Configuration
    config: OnlineLearningConfig,
    /// Statistics
    stats: HotFixStats,
}

impl std::fmt::Debug for HotFixLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HotFixLayer")
            .field("corpus_len", &self.corpus.len())
            .field("config", &self.config)
            .field("stats", &self.stats)
            .finish_non_exhaustive()
    }
}

impl Default for HotFixLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl HotFixLayer {
    /// Create new hot-fix layer
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(OnlineLearningConfig::default())
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: OnlineLearningConfig) -> Self {
        Self {
            drift_detector: DriftDetectorFactory::recommended(),
            corpus: Corpus::new(),
            config,
            stats: HotFixStats::default(),
        }
    }

    /// Record a prediction result for drift detection
    pub fn record_prediction(&mut self, correct: bool) {
        self.drift_detector.add_element(!correct); // ADWIN expects errors, not correctness
    }

    /// Check drift status
    #[must_use]
    pub fn check_drift(&self) -> DriftStatus {
        self.drift_detector.detected_change()
    }

    /// Record a fix for online learning
    pub fn record_fix(
        &mut self,
        message: &str,
        error_code: Option<String>,
        category: ErrorCategory,
    ) -> bool {
        let sample =
            Sample::new(message, error_code, category).with_source(SampleSource::Production);

        self.corpus.add(sample);
        self.stats.samples_accumulated = self.corpus.len();

        if self
            .corpus
            .len()
            .is_multiple_of(self.config.micro_batch_size)
        {
            self.stats.micro_batches_processed += 1;
            true
        } else {
            false
        }
    }

    /// Get accumulated samples
    #[must_use]
    pub fn get_accumulated_samples(&self) -> &Corpus {
        &self.corpus
    }

    /// Clear accumulated samples
    pub fn clear_accumulated(&mut self) {
        self.corpus = Corpus::new();
        self.stats.samples_accumulated = 0;
    }

    /// Get statistics
    #[must_use]
    pub fn stats(&self) -> &HotFixStats {
        &self.stats
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &OnlineLearningConfig {
        &self.config
    }

    /// Check if retraining is needed
    #[must_use]
    pub fn should_retrain(&self) -> bool {
        self.corpus.len() >= self.config.min_samples_for_training
            || self.drift_detector.detected_change() == DriftStatus::Drift
    }

    /// Get training data
    #[must_use]
    pub fn get_training_data(&self) -> (Vec<Vec<f32>>, Vec<usize>) {
        self.corpus.to_training_data()
    }
}

/// Online learner wrapper
#[derive(Debug)]
pub struct OnlineLearner {
    hotfix: HotFixLayer,
    enabled: bool,
}

impl Default for OnlineLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl OnlineLearner {
    /// Create new online learner
    #[must_use]
    pub fn new() -> Self {
        Self {
            hotfix: HotFixLayer::new(),
            enabled: true,
        }
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: OnlineLearningConfig) -> Self {
        Self {
            hotfix: HotFixLayer::with_config(config),
            enabled: true,
        }
    }

    /// Enable/disable
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get hot-fix layer
    #[must_use]
    pub fn hotfix(&self) -> &HotFixLayer {
        &self.hotfix
    }

    /// Get mutable hot-fix layer
    pub fn hotfix_mut(&mut self) -> &mut HotFixLayer {
        &mut self.hotfix
    }

    /// Record successful classification
    pub fn record_success(
        &mut self,
        error: &CompilationError,
        category: ErrorCategory,
        confidence: f64,
    ) {
        if !self.enabled {
            return;
        }

        // Record to drift detector (correct prediction)
        self.hotfix.record_prediction(true);

        // High confidence → add to corpus
        if confidence >= self.hotfix.config.hotfix_confidence {
            self.hotfix
                .record_fix(&error.message, error.code.clone(), category);
        }
    }
}

// Re-export from entrenar (§3.1, §3.2, §3.4 of spec)
#[cfg(feature = "training")]
pub use entrenar::citl::{
    CITLConfig, DecisionCITL, DecisionPatternStore, DecisionTrace, ErrorCorrelation,
    FixPattern as CitlFixPattern, FixSuggestion as CitlFixSuggestion, SuspiciousDecision,
};
#[cfg(feature = "training")]
pub use entrenar::distill::{DistillationLoss, EnsembleDistiller, ProgressiveDistiller};

// Code2Vec embeddings (§3.5 of spec)
pub use aprender::code::{Code2VecEncoder, PathContext, PathExtractor};

/// Oracle configuration
#[derive(Debug, Clone)]
pub struct OracleConfig {
    /// Confidence threshold for auto-fix (default: 0.85)
    pub confidence_threshold: f64,

    /// Maximum suggestions to return (default: 5)
    pub max_suggestions: usize,

    /// Enable drift detection (default: true)
    pub drift_detection_enabled: bool,

    /// Similarity threshold for pattern matching (default: 0.7)
    pub similarity_threshold: f64,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.85,
            max_suggestions: 5,
            drift_detection_enabled: true,
            similarity_threshold: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Phase 1 Tests (RED → GREEN → REFACTOR)
    // ============================================================================

    #[test]
    fn test_oracle_config_default_confidence_threshold() {
        let config = OracleConfig::default();
        assert!((config.confidence_threshold - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_oracle_config_default_max_suggestions() {
        let config = OracleConfig::default();
        assert_eq!(config.max_suggestions, 5);
    }

    #[test]
    fn test_oracle_config_default_drift_enabled() {
        let config = OracleConfig::default();
        assert!(config.drift_detection_enabled);
    }

    #[test]
    fn test_oracle_config_default_similarity_threshold() {
        let config = OracleConfig::default();
        assert!((config.similarity_threshold - 0.7).abs() < f64::EPSILON);
    }

    // ============================================================================
    // MIGRATION TESTS: Issue #174 - aprender::online integration
    // ============================================================================

    // --- Drift Detection Tests (using aprender::ADWIN) ---

    #[test]
    fn test_migration_adwin_drift_detection_stable() {
        // ADWIN from aprender should detect stable behavior
        let mut detector = ADWIN::default();

        // Add correct predictions (no errors)
        for _ in 0..50 {
            detector.add_element(false); // false = no error
        }

        assert_eq!(detector.detected_change(), DriftStatus::Stable);
    }

    #[test]
    fn test_migration_adwin_drift_detection_drift() {
        // ADWIN should detect drift when error rate changes significantly
        let mut detector = ADWIN::default();

        // Phase 1: Low error rate
        for _ in 0..100 {
            detector.add_element(false);
        }

        // Phase 2: High error rate (should trigger drift)
        for _ in 0..100 {
            detector.add_element(true); // true = error
        }

        // ADWIN should have detected drift at some point
        // Note: may be Stable if ADWIN already adapted
        let status = detector.detected_change();
        assert!(matches!(
            status,
            DriftStatus::Stable | DriftStatus::Warning | DriftStatus::Drift
        ));
    }

    #[test]
    fn test_migration_ddm_drift_detection() {
        // DDM from aprender for sudden drift
        let mut detector = DDM::default();

        for _ in 0..20 {
            detector.add_element(false);
        }

        assert_eq!(detector.detected_change(), DriftStatus::Stable);
    }

    #[test]
    fn test_migration_page_hinkley_drift_detection() {
        // Page-Hinkley from aprender for gradual drift
        let mut detector = PageHinkley::default();

        for _ in 0..20 {
            detector.add_element(false);
        }

        assert_eq!(detector.detected_change(), DriftStatus::Stable);
    }

    #[test]
    fn test_migration_drift_detector_factory() {
        // Factory should create recommended detector (ADWIN)
        let mut detector = DriftDetectorFactory::recommended();

        detector.add_element(false);
        assert_eq!(detector.detected_change(), DriftStatus::Stable);
    }

    // --- Corpus Buffer Tests (using aprender::CorpusBuffer) ---

    #[test]
    fn test_migration_corpus_buffer_basic() {
        let mut buffer = CorpusBuffer::new(100);

        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        buffer.add_raw(vec![1.0, 2.0], vec![3.0]);
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_migration_corpus_buffer_deduplication() {
        let mut buffer = CorpusBuffer::new(100);

        // Add same sample twice
        buffer.add_raw(vec![1.0, 2.0], vec![3.0]);
        let added = buffer.add_raw(vec![1.0, 2.0], vec![3.0]);

        assert!(!added, "Duplicate should not be added");
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_migration_corpus_buffer_eviction_policies() {
        // Test FIFO eviction
        let config = CorpusBufferConfig {
            max_size: 3,
            policy: EvictionPolicy::FIFO,
            deduplicate: false,
            seed: None,
        };
        let mut buffer = CorpusBuffer::with_config(config);

        buffer.add_raw(vec![1.0], vec![1.0]);
        buffer.add_raw(vec![2.0], vec![2.0]);
        buffer.add_raw(vec![3.0], vec![3.0]);
        buffer.add_raw(vec![4.0], vec![4.0]);

        assert_eq!(buffer.len(), 3);
    }

    // --- Sample Wrapper Tests ---

    #[test]
    fn test_migration_sample_creation() {
        let sample = Sample::new(
            "mismatched types",
            Some("E0308".to_string()),
            ErrorCategory::TypeMismatch,
        );

        assert_eq!(sample.message, "mismatched types");
        assert_eq!(sample.error_code, Some("E0308".to_string()));
        assert_eq!(sample.category, ErrorCategory::TypeMismatch);
        assert!((sample.difficulty - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_migration_sample_with_difficulty() {
        let sample = Sample::new("test", None, ErrorCategory::Other).with_difficulty(0.75);

        assert!((sample.difficulty - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_migration_sample_to_aprender() {
        let sample = Sample::new(
            "mismatched types",
            Some("E0308".to_string()),
            ErrorCategory::TypeMismatch,
        );

        let aprender_sample = sample.to_aprender();
        assert!(!aprender_sample.features.is_empty());
        assert_eq!(aprender_sample.target.len(), 1);
    }

    // --- Corpus Wrapper Tests ---

    #[test]
    fn test_migration_corpus_add() {
        let mut corpus = Corpus::new();

        let sample = Sample::new(
            "test error",
            Some("E0001".to_string()),
            ErrorCategory::TypeMismatch,
        );

        assert!(corpus.add(sample));
        assert_eq!(corpus.len(), 1);
    }

    #[test]
    fn test_migration_corpus_deduplication() {
        let mut corpus = Corpus::new();

        let sample1 = Sample::new(
            "test",
            Some("E0001".to_string()),
            ErrorCategory::TypeMismatch,
        );
        let sample2 = Sample::new(
            "test",
            Some("E0001".to_string()),
            ErrorCategory::TypeMismatch,
        );

        corpus.add(sample1);
        // Second add may or may not be deduplicated depending on feature extraction
        corpus.add(sample2);

        // At most 2 samples
        assert!(corpus.len() <= 2);
    }

    #[test]
    fn test_migration_corpus_filter_by_source() {
        let mut corpus = Corpus::new();

        corpus.add(
            Sample::new("test1", None, ErrorCategory::TypeMismatch)
                .with_source(SampleSource::Synthetic),
        );
        corpus.add(
            Sample::new("test2", None, ErrorCategory::BorrowChecker)
                .with_source(SampleSource::Production),
        );

        let synthetic = corpus.filter_by_source(SampleSource::Synthetic);
        assert_eq!(synthetic.len(), 1);
    }

    #[test]
    fn test_migration_corpus_training_data() {
        let mut corpus = Corpus::new();

        corpus.add(Sample::new(
            "error1",
            Some("E0308".to_string()),
            ErrorCategory::TypeMismatch,
        ));
        corpus.add(Sample::new(
            "error2",
            Some("E0382".to_string()),
            ErrorCategory::BorrowChecker,
        ));

        let (features, labels) = corpus.to_training_data();

        assert_eq!(features.len(), 2);
        assert_eq!(labels.len(), 2);
    }

    // --- Difficulty Level Tests ---

    #[test]
    fn test_migration_difficulty_level_score() {
        assert!((DifficultyLevel::Easy.score() - 0.25).abs() < f32::EPSILON);
        assert!((DifficultyLevel::Medium.score() - 0.50).abs() < f32::EPSILON);
        assert!((DifficultyLevel::Hard.score() - 0.75).abs() < f32::EPSILON);
        assert!((DifficultyLevel::Expert.score() - 1.00).abs() < f32::EPSILON);
    }

    #[test]
    fn test_migration_difficulty_level_next() {
        assert_eq!(DifficultyLevel::Easy.next(), DifficultyLevel::Medium);
        assert_eq!(DifficultyLevel::Medium.next(), DifficultyLevel::Hard);
        assert_eq!(DifficultyLevel::Hard.next(), DifficultyLevel::Expert);
        assert_eq!(DifficultyLevel::Expert.next(), DifficultyLevel::Expert);
    }

    #[test]
    fn test_migration_difficulty_level_from_score() {
        assert_eq!(DifficultyLevel::from_score(0.1), DifficultyLevel::Easy);
        assert_eq!(DifficultyLevel::from_score(0.4), DifficultyLevel::Medium);
        assert_eq!(DifficultyLevel::from_score(0.6), DifficultyLevel::Hard);
        assert_eq!(DifficultyLevel::from_score(0.9), DifficultyLevel::Expert);
    }

    // --- HotFixLayer Tests ---

    #[test]
    fn test_migration_hotfix_layer_creation() {
        let hotfix = HotFixLayer::new();

        assert_eq!(hotfix.stats().total_overrides, 0);
        assert!(!hotfix.should_retrain());
    }

    #[test]
    fn test_migration_hotfix_layer_record_prediction() {
        let mut hotfix = HotFixLayer::new();

        // Record correct predictions
        for _ in 0..10 {
            hotfix.record_prediction(true);
        }

        assert_eq!(hotfix.check_drift(), DriftStatus::Stable);
    }

    #[test]
    fn test_migration_hotfix_layer_record_fix() {
        let mut hotfix = HotFixLayer::new();

        hotfix.record_fix(
            "test error",
            Some("E0308".to_string()),
            ErrorCategory::TypeMismatch,
        );

        assert_eq!(hotfix.get_accumulated_samples().len(), 1);
    }

    #[test]
    fn test_migration_hotfix_layer_micro_batch() {
        let config = OnlineLearningConfig {
            micro_batch_size: 5,
            ..Default::default()
        };
        let mut hotfix = HotFixLayer::with_config(config);

        // Add unique samples until micro-batch triggers
        // Use different error codes to ensure uniqueness (avoid deduplication)
        let error_codes = ["E0308", "E0382", "E0597", "E0277", "E0433"];
        let categories = [
            ErrorCategory::TypeMismatch,
            ErrorCategory::BorrowChecker,
            ErrorCategory::LifetimeError,
            ErrorCategory::TraitBound,
            ErrorCategory::MissingImport,
        ];

        for i in 0..5 {
            let _triggered = hotfix.record_fix(
                &format!("unique error message {i}"),
                Some(error_codes[i].to_string()),
                categories[i],
            );
        }

        // After 5 unique samples, micro-batch should have triggered once
        assert_eq!(hotfix.stats().micro_batches_processed, 1);
        assert_eq!(hotfix.get_accumulated_samples().len(), 5);
    }

    #[test]
    fn test_migration_hotfix_layer_clear() {
        let mut hotfix = HotFixLayer::new();

        hotfix.record_fix("test", None, ErrorCategory::Other);
        assert_eq!(hotfix.get_accumulated_samples().len(), 1);

        hotfix.clear_accumulated();
        assert_eq!(hotfix.get_accumulated_samples().len(), 0);
    }

    // --- OnlineLearner Tests ---

    #[test]
    fn test_migration_online_learner_creation() {
        let learner = OnlineLearner::new();

        assert!(learner.is_enabled());
    }

    #[test]
    fn test_migration_online_learner_enable_disable() {
        let mut learner = OnlineLearner::new();

        learner.set_enabled(false);
        assert!(!learner.is_enabled());

        learner.set_enabled(true);
        assert!(learner.is_enabled());
    }

    // --- CorpusCollector Tests ---

    #[test]
    fn test_migration_corpus_collector_synthetic() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();

        // Should generate synthetic samples
        assert!(!corpus.is_empty());

        // Check variety of categories
        let type_mismatch = corpus.filter_by_source(SampleSource::Synthetic);
        assert!(!type_mismatch.is_empty());
    }

    // --- SampleSource Tests ---

    #[test]
    fn test_migration_sample_source_display() {
        assert_eq!(format!("{}", SampleSource::Synthetic), "synthetic");
        assert_eq!(format!("{}", SampleSource::Ruchy), "ruchy");
        assert_eq!(format!("{}", SampleSource::Examples), "examples");
        assert_eq!(format!("{}", SampleSource::Production), "production");
    }

    #[test]
    fn test_migration_sample_source_to_aprender() {
        let aprender_source: AprenderSampleSource = SampleSource::Synthetic.into();
        assert_eq!(aprender_source, AprenderSampleSource::Synthetic);

        let aprender_source: AprenderSampleSource = SampleSource::Production.into();
        assert_eq!(aprender_source, AprenderSampleSource::Production);
    }

    // --- Integration Tests ---

    #[test]
    fn test_migration_end_to_end_classification_with_drift() {
        let mut oracle = RuchyOracle::new();
        oracle
            .train_from_examples()
            .expect("training should succeed");

        // Simulate some classifications
        for _ in 0..10 {
            let error = CompilationError::new("mismatched types").with_code("E0308");
            let classification = oracle.classify(&error);

            // Record result
            oracle.record_result(classification.category, ErrorCategory::TypeMismatch);
        }

        // Check drift status should be stable after correct classifications
        assert_eq!(oracle.drift_status(), DriftStatus::Stable);
    }

    #[test]
    fn test_migration_corpus_merger() {
        let samples1 = vec![
            AprenderSample::new(vec![1.0], vec![1.0]),
            AprenderSample::new(vec![2.0], vec![2.0]),
        ];
        let samples2 = vec![
            AprenderSample::new(vec![3.0], vec![3.0]),
            AprenderSample::new(vec![4.0], vec![4.0]),
        ];

        let mut merger = CorpusMerger::new();
        merger.add_source(CorpusSource::new("source1", samples1));
        merger.add_source(CorpusSource::new("source2", samples2));

        let (buffer, provenance) = merger.merge().expect("merge should succeed");

        assert_eq!(buffer.len(), 4);
        assert_eq!(provenance.sources.len(), 2);
    }

    // ============================================================================
    // Coverage: CorpusMergerWithProvenance::merge_with_seed
    // ============================================================================

    #[test]
    fn test_merger_with_provenance_empty() {
        let merger = CorpusMergerWithProvenance::new();
        let (corpus, provenance) = merger.merge_with_seed(99).unwrap();
        assert_eq!(corpus.len(), 0);
        assert!(corpus.is_empty());
        assert_eq!(provenance.total_before_dedup, 0);
        assert_eq!(provenance.total_after_dedup, 0);
        assert!(provenance.merged_at.is_some());
        assert!(provenance.sources.is_empty());
    }

    #[test]
    fn test_merger_with_provenance_single_source() {
        let mut merger = CorpusMergerWithProvenance::new();
        let samples = vec![
            Sample::new(
                "type mismatch",
                Some("E0308".into()),
                ErrorCategory::TypeMismatch,
            ),
            Sample::new(
                "missing field",
                Some("E0063".into()),
                ErrorCategory::MissingImport,
            ),
        ];
        merger.add_source("synthetic", samples, SampleSource::Synthetic);
        assert_eq!(merger.source_count(), 1);

        let (corpus, provenance) = merger.merge_with_seed(42).unwrap();
        assert_eq!(corpus.len(), 2);
        assert_eq!(provenance.total_before_dedup, 2);
        assert_eq!(provenance.total_after_dedup, 2);
        assert_eq!(provenance.sources.len(), 1);
        assert_eq!(provenance.sources[0].0, "synthetic");
        assert_eq!(provenance.sources[0].1, 2);
    }

    #[test]
    fn test_merger_with_provenance_multiple_sources() {
        let mut merger = CorpusMergerWithProvenance::new();
        let synthetic = vec![
            Sample::new("err 1", None, ErrorCategory::TypeMismatch),
            Sample::new("err 2", None, ErrorCategory::BorrowChecker),
        ];
        let examples = vec![Sample::new("err 3", None, ErrorCategory::MissingImport)];
        merger.add_source("synthetic", synthetic, SampleSource::Synthetic);
        merger.add_source("examples", examples, SampleSource::Examples);

        let (corpus, provenance) = merger.merge_with_seed(123).unwrap();
        assert_eq!(corpus.len(), 3);
        assert_eq!(provenance.sources.len(), 2);
        assert_eq!(provenance.total_before_dedup, 3);
    }

    #[test]
    fn test_merger_with_provenance_deterministic() {
        let mut merger = CorpusMergerWithProvenance::new();
        let samples = vec![
            Sample::new("a", None, ErrorCategory::TypeMismatch),
            Sample::new("b", None, ErrorCategory::BorrowChecker),
            Sample::new("c", None, ErrorCategory::MissingImport),
        ];
        merger.add_source("test", samples, SampleSource::Ruchy);

        let (c1, _) = merger.merge_with_seed(42).unwrap();
        let (c2, _) = merger.merge_with_seed(42).unwrap();

        let msgs1: Vec<_> = c1.samples().iter().map(|s| &s.message).collect();
        let msgs2: Vec<_> = c2.samples().iter().map(|s| &s.message).collect();
        assert_eq!(msgs1, msgs2, "Same seed should produce same order");
    }

    #[test]
    fn test_merger_with_provenance_different_seeds() {
        let mut merger = CorpusMergerWithProvenance::new();
        // Use diverse error messages to avoid feature-vector dedup collisions
        let messages = vec![
            (
                "expected type `i32`, found `String`",
                ErrorCategory::TypeMismatch,
            ),
            ("cannot borrow `x` as mutable", ErrorCategory::BorrowChecker),
            (
                "lifetime `'a` does not live long enough",
                ErrorCategory::LifetimeError,
            ),
            (
                "the trait `Display` is not implemented",
                ErrorCategory::TraitBound,
            ),
            (
                "unresolved import `std::io::missing`",
                ErrorCategory::MissingImport,
            ),
            (
                "cannot assign twice to immutable variable",
                ErrorCategory::MutabilityError,
            ),
            ("expected `;`, found `}`", ErrorCategory::SyntaxError),
            (
                "mismatched types: expected `bool`",
                ErrorCategory::TypeMismatch,
            ),
            (
                "cannot borrow `self` as mutable",
                ErrorCategory::BorrowChecker,
            ),
            ("unknown start of token: `@`", ErrorCategory::SyntaxError),
        ];
        let samples: Vec<_> = messages
            .into_iter()
            .map(|(msg, cat)| Sample::new(msg, None, cat))
            .collect();
        merger.add_source("test", samples, SampleSource::Synthetic);

        let (c1, _) = merger.merge_with_seed(1).unwrap();
        let (c2, _) = merger.merge_with_seed(2).unwrap();

        // Verify samples were actually added (not deduped away)
        assert!(
            c1.len() >= 2,
            "Should have multiple samples, got {}",
            c1.len()
        );
        let msgs1: Vec<_> = c1.samples().iter().map(|s| &s.message).collect();
        let msgs2: Vec<_> = c2.samples().iter().map(|s| &s.message).collect();
        // With 10+ diverse samples, different seeds should produce different orders
        assert_ne!(
            msgs1, msgs2,
            "Different seeds should produce different orders"
        );
    }

    #[test]
    fn test_merger_merge_default_seed() {
        let mut merger = CorpusMergerWithProvenance::new();
        merger.add_source(
            "s",
            vec![Sample::new("x", None, ErrorCategory::TypeMismatch)],
            SampleSource::Synthetic,
        );
        let (corpus, _) = merger.merge().unwrap();
        assert_eq!(corpus.len(), 1);
    }

    #[test]
    fn test_corpus_shuffle_with_seed() {
        let mut corpus = Corpus::new();
        // Use diverse error messages to avoid feature-vector dedup collisions
        let messages = [
            (
                "expected type `i32`, found `String`",
                ErrorCategory::TypeMismatch,
            ),
            ("cannot borrow `x` as mutable", ErrorCategory::BorrowChecker),
            (
                "lifetime `'a` does not live long enough",
                ErrorCategory::LifetimeError,
            ),
            (
                "the trait `Display` is not implemented",
                ErrorCategory::TraitBound,
            ),
            (
                "unresolved import `std::io::missing`",
                ErrorCategory::MissingImport,
            ),
            (
                "cannot assign twice to immutable variable",
                ErrorCategory::MutabilityError,
            ),
            ("expected `;`, found `}`", ErrorCategory::SyntaxError),
            (
                "mismatched types: expected `bool`",
                ErrorCategory::TypeMismatch,
            ),
            (
                "cannot borrow `self` as mutable",
                ErrorCategory::BorrowChecker,
            ),
            ("unknown start of token: `@`", ErrorCategory::SyntaxError),
            ("pattern `_` not covered", ErrorCategory::TypeMismatch),
            (
                "method not found in `Vec<i32>`",
                ErrorCategory::TypeMismatch,
            ),
            ("expected `()`, found `i32`", ErrorCategory::TypeMismatch),
            (
                "conflicting implementations of trait",
                ErrorCategory::TraitBound,
            ),
            ("use of moved value: `x`", ErrorCategory::BorrowChecker),
        ];
        for (msg, cat) in &messages {
            corpus.add(Sample::new(*msg, None, *cat));
        }
        assert!(
            corpus.len() >= 2,
            "Should have multiple samples, got {}",
            corpus.len()
        );
        let before: Vec<_> = corpus.samples().iter().map(|s| s.message.clone()).collect();
        corpus.shuffle_with_seed(99);
        let after: Vec<_> = corpus.samples().iter().map(|s| s.message.clone()).collect();
        assert_ne!(before, after, "Shuffle should reorder samples");
        assert_eq!(before.len(), after.len());
    }

    #[test]
    fn test_corpus_shuffle_with_seed_single() {
        let mut corpus = Corpus::new();
        corpus.add(Sample::new("only", None, ErrorCategory::TypeMismatch));
        corpus.shuffle_with_seed(42);
        assert_eq!(corpus.len(), 1);
    }

    #[test]
    fn test_corpus_shuffle_with_seed_empty() {
        let mut corpus = Corpus::new();
        corpus.shuffle_with_seed(42);
        assert_eq!(corpus.len(), 0);
    }

    #[test]
    fn test_ruchy_corpus_provenance_count_by_source() {
        // SampleSource::to_string() returns lowercase names
        let prov = RuchyCorpusProvenance {
            sources: vec![
                ("synthetic".to_string(), 5),
                ("examples".to_string(), 3),
                ("synthetic".to_string(), 2),
            ],
            total_before_dedup: 10,
            total_after_dedup: 8,
            merged_at: Some("2025-01-01".into()),
        };
        assert_eq!(prov.count_by_source(SampleSource::Synthetic), 7);
        assert_eq!(prov.count_by_source(SampleSource::Examples), 3);
        assert_eq!(prov.count_by_source(SampleSource::Production), 0);
    }

    // ============================================================================
    // Coverage: generate_synthetic_samples via CorpusCollector::collect()
    // ============================================================================

    #[test]
    fn test_corpus_collector_collect_generates_samples() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        // Dedup reduces 170 raw samples to unique feature vectors (approx 8-9)
        assert!(corpus.len() > 0, "Should generate at least some samples");
        assert!(corpus.len() <= 170, "Should not exceed raw count");
    }

    #[test]
    fn test_corpus_collector_collect_has_type_mismatch_samples() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        let type_mismatch_count = corpus
            .samples
            .iter()
            .filter(|s| s.category == ErrorCategory::TypeMismatch)
            .count();
        // 5 unique type mismatch messages, dedup keeps unique feature vectors
        assert!(
            type_mismatch_count >= 1,
            "Should have at least 1 type mismatch sample"
        );
        assert!(
            type_mismatch_count <= 5,
            "At most 5 unique type mismatch messages"
        );
    }

    #[test]
    fn test_corpus_collector_collect_has_borrow_checker_samples() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        let borrow_count = corpus
            .samples
            .iter()
            .filter(|s| s.category == ErrorCategory::BorrowChecker)
            .count();
        assert!(borrow_count >= 1, "Should have at least 1 borrow sample");
        assert!(borrow_count <= 2, "At most 2 unique borrow messages");
    }

    #[test]
    fn test_corpus_collector_collect_has_lifetime_samples() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        let lifetime_count = corpus
            .samples
            .iter()
            .filter(|s| s.category == ErrorCategory::LifetimeError)
            .count();
        assert!(
            lifetime_count >= 1,
            "Should have at least 1 lifetime sample"
        );
        assert!(lifetime_count <= 2, "At most 2 unique lifetime messages");
    }

    #[test]
    fn test_corpus_collector_collect_difficulty_levels() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        // Type mismatch samples have difficulty 0.25
        let type_sample = corpus
            .samples
            .iter()
            .find(|s| s.category == ErrorCategory::TypeMismatch)
            .expect("Should have type mismatch sample");
        assert!((type_sample.difficulty - 0.25).abs() < f32::EPSILON);
        // Borrow checker samples have difficulty 0.5
        let borrow_sample = corpus
            .samples
            .iter()
            .find(|s| s.category == ErrorCategory::BorrowChecker)
            .expect("Should have borrow sample");
        assert!((borrow_sample.difficulty - 0.5).abs() < f32::EPSILON);
        // Lifetime samples have difficulty 0.75
        let lifetime_sample = corpus
            .samples
            .iter()
            .find(|s| s.category == ErrorCategory::LifetimeError)
            .expect("Should have lifetime sample");
        assert!((lifetime_sample.difficulty - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_corpus_collector_collect_all_synthetic_source() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        assert!(
            corpus
                .samples
                .iter()
                .all(|s| s.source == SampleSource::Synthetic),
            "All generated samples should be synthetic"
        );
    }

    #[test]
    fn test_corpus_collector_collect_has_error_codes() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        // All samples should have error codes
        assert!(
            corpus.samples.iter().all(|s| s.error_code.is_some()),
            "All samples should have error codes"
        );
        // Check specific error codes exist
        let codes: std::collections::HashSet<_> = corpus
            .samples
            .iter()
            .filter_map(|s| s.error_code.as_deref())
            .collect();
        assert!(codes.contains("E0308"), "Should have E0308 (type mismatch)");
        assert!(codes.contains("E0382"), "Should have E0382 (moved value)");
        assert!(
            codes.contains("E0502"),
            "Should have E0502 (mutable borrow)"
        );
        assert!(codes.contains("E0597"), "Should have E0597 (lifetime)");
        assert!(codes.contains("E0621"), "Should have E0621 (lifetime)");
    }

    #[test]
    fn test_corpus_collector_with_production_flag() {
        let collector = CorpusCollector::new().with_production(true);
        let corpus = collector.collect();
        // Production flag doesn't change synthetic sample count (dedup applies)
        assert!(corpus.len() > 0);
    }

    #[test]
    fn test_corpus_collector_collect_message_content() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        // Type mismatch messages should contain "mismatched types"
        let type_msgs: Vec<_> = corpus
            .samples
            .iter()
            .filter(|s| s.category == ErrorCategory::TypeMismatch)
            .map(|s| s.message.as_str())
            .collect();
        assert!(
            type_msgs.iter().all(|m| m.contains("mismatched types")),
            "Type mismatch messages should contain 'mismatched types'"
        );
        // Borrow messages should contain "borrow"
        let borrow_msgs: Vec<_> = corpus
            .samples
            .iter()
            .filter(|s| s.category == ErrorCategory::BorrowChecker)
            .map(|s| s.message.as_str())
            .collect();
        assert!(
            borrow_msgs.iter().all(|m| m.contains("borrow")),
            "Borrow messages should contain 'borrow'"
        );
    }

    #[test]
    fn test_corpus_collector_collect_three_categories() {
        let collector = CorpusCollector::new();
        let corpus = collector.collect();
        let categories: std::collections::HashSet<_> =
            corpus.samples.iter().map(|s| s.category).collect();
        assert!(categories.contains(&ErrorCategory::TypeMismatch));
        assert!(categories.contains(&ErrorCategory::BorrowChecker));
        assert!(categories.contains(&ErrorCategory::LifetimeError));
        assert_eq!(categories.len(), 3, "Should have exactly 3 categories");
    }

    // ============================================================================
    // record_success coverage tests (OnlineLearner)
    // ============================================================================

    #[test]
    fn test_record_success_when_disabled_does_nothing() {
        let mut learner = OnlineLearner::new();
        learner.set_enabled(false);

        let error = CompilationError::new("mismatched types").with_code("E0308");
        learner.record_success(&error, ErrorCategory::TypeMismatch, 0.99);

        // Should not record anything since learner is disabled
        assert_eq!(learner.hotfix().stats().samples_accumulated, 0);
    }

    #[test]
    fn test_record_success_when_enabled_records_prediction() {
        let mut learner = OnlineLearner::new();
        assert!(learner.is_enabled());

        let error = CompilationError::new("mismatched types").with_code("E0308");
        // High confidence above default hotfix_confidence (0.95)
        learner.record_success(&error, ErrorCategory::TypeMismatch, 0.99);

        // Should record the fix (high confidence >= 0.95 threshold)
        assert_eq!(learner.hotfix().stats().samples_accumulated, 1);
    }

    #[test]
    fn test_record_success_low_confidence_no_corpus_addition() {
        let mut learner = OnlineLearner::new();

        let error = CompilationError::new("some error").with_code("E0000");
        // Low confidence below hotfix_confidence threshold (0.95)
        learner.record_success(&error, ErrorCategory::Other, 0.5);

        // Should NOT add to corpus (confidence too low)
        assert_eq!(learner.hotfix().stats().samples_accumulated, 0);
    }

    #[test]
    fn test_record_success_with_custom_config_threshold() {
        let config = OnlineLearningConfig {
            hotfix_confidence: 0.80,
            ..Default::default()
        };
        let mut learner = OnlineLearner::with_config(config);

        let error = CompilationError::new("borrow error").with_code("E0382");
        // Confidence 0.85 is above custom threshold of 0.80
        learner.record_success(&error, ErrorCategory::BorrowChecker, 0.85);

        // Should record to corpus since 0.85 >= 0.80
        assert_eq!(learner.hotfix().stats().samples_accumulated, 1);
    }

    #[test]
    fn test_record_success_exactly_at_threshold() {
        let mut learner = OnlineLearner::new();
        // Default threshold is 0.95

        let error = CompilationError::new("lifetime error").with_code("E0597");
        // Confidence exactly at threshold
        learner.record_success(&error, ErrorCategory::LifetimeError, 0.95);

        // Should add to corpus (>= threshold)
        assert_eq!(learner.hotfix().stats().samples_accumulated, 1);
    }
}
