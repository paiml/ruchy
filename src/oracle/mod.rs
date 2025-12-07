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
//! # Architecture
//! ```text
//! Ruchy Source → Transpiler → Rust Code → rustc → Errors
//!                                            ↓
//!                                    Oracle Classifier
//!                                            ↓
//!                                    Pattern Store (.apr)
//!                                            ↓
//!                                    Suggested Fix → AutoFixer
//! ```
//!
//! # References
//! - [3] Breiman, L. (2001). "Random Forests." Machine Learning, 45(1), 5-32.
//! - [6] Amershi et al. (2014). "Power to the People: CITL" AI Magazine, 35(4).

mod category;
mod classifier;
mod drift;
mod features;
mod patterns;

pub use category::ErrorCategory;
pub use classifier::{Classification, CompilationError, OracleError, OracleMetadata, RuchyOracle};
pub use drift::{DriftDetector, DriftStatus};
pub use features::{ErrorFeatures, FEATURE_COUNT};
pub use patterns::{FixPattern, FixSuggestion, PatternStore};

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
}
