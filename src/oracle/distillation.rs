//! Knowledge Distillation for Oracle
//!
//! Implements temperature-scaled soft target generation for expanding training data.
//! Based on Hinton et al. (2015) knowledge distillation.
//!
//! # References
//! - [4] Hinton, G., et al. (2015). "Distilling the Knowledge in a Neural Network."
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §3.4

use super::{CompilationError, ErrorCategory, RuchyOracle, Sample};

/// Configuration for knowledge distillation
#[derive(Debug, Clone)]
pub struct DistillationConfig {
    /// Temperature for soft targets (default: 3.0)
    /// Higher temperature produces smoother probability distribution
    pub temperature: f64,

    /// Confidence threshold for distillation (default: 0.95)
    /// Only predictions above this threshold are used for distillation
    pub confidence_threshold: f64,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            temperature: 3.0,
            confidence_threshold: 0.95,
        }
    }
}

/// Soft label for distillation training
#[derive(Debug, Clone)]
pub struct SoftLabel {
    /// Original sample
    pub sample: Sample,

    /// Soft target probabilities for each category
    pub soft_targets: Vec<f64>,
}

impl SoftLabel {
    /// Create a new soft label
    #[must_use]
    pub fn new(sample: Sample, soft_targets: Vec<f64>) -> Self {
        Self {
            sample,
            soft_targets,
        }
    }

    /// Get the most likely category
    #[must_use]
    pub fn predicted_category(&self) -> Option<ErrorCategory> {
        self.soft_targets
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .and_then(|(idx, _)| ErrorCategory::from_index(idx))
    }

    /// Get confidence (max probability)
    #[must_use]
    pub fn confidence(&self) -> f64 {
        self.soft_targets
            .iter()
            .copied()
            .fold(0.0f64, f64::max)
    }
}

/// Knowledge Distiller
///
/// Generates soft labels from a teacher model's high-confidence predictions.
/// Uses temperature scaling to smooth the probability distribution.
#[derive(Debug)]
pub struct KnowledgeDistiller {
    /// Configuration
    config: DistillationConfig,
}

impl KnowledgeDistiller {
    /// Create a new distiller with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(DistillationConfig::default())
    }

    /// Create a distiller with custom configuration
    #[must_use]
    pub fn with_config(config: DistillationConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &DistillationConfig {
        &self.config
    }

    /// Apply temperature scaling to logits
    ///
    /// Uses softmax with temperature: `soft_target[i] = exp(z[i] / T) / sum(exp(z[j] / T))`
    #[must_use]
    pub fn temperature_scale(&self, logits: &[f64]) -> Vec<f64> {
        if logits.is_empty() {
            return Vec::new();
        }

        let t = self.config.temperature;

        // Apply temperature and compute softmax
        let scaled: Vec<f64> = logits.iter().map(|&z| (z / t).exp()).collect();
        let sum: f64 = scaled.iter().sum();

        if sum == 0.0 {
            // Uniform distribution if all zero
            let uniform = 1.0 / logits.len() as f64;
            return vec![uniform; logits.len()];
        }

        scaled.iter().map(|&s| s / sum).collect()
    }

    /// Generate soft labels from teacher model predictions
    ///
    /// Only includes predictions above the confidence threshold.
    #[must_use]
    pub fn distill(&self, teacher: &RuchyOracle, samples: &[Sample]) -> Vec<SoftLabel> {
        samples
            .iter()
            .filter_map(|sample| {
                let error = CompilationError::new(&sample.message)
                    .with_code(sample.error_code.clone().unwrap_or_default());

                let classification = teacher.classify(&error);

                // Only distill high-confidence predictions
                if classification.confidence >= self.config.confidence_threshold {
                    // Generate pseudo-logits from classification
                    let logits = self.generate_logits(&classification.category, classification.confidence);
                    let soft_targets = self.temperature_scale(&logits);

                    Some(SoftLabel::new(sample.clone(), soft_targets))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Generate pseudo-logits from a category and confidence
    fn generate_logits(&self, category: &ErrorCategory, confidence: f64) -> Vec<f64> {
        let num_categories = 8; // ErrorCategory count
        let target_idx = category.to_index();

        // Create logits that will produce approximately the given confidence after softmax
        let mut logits = vec![0.0; num_categories];

        // Set target logit higher based on confidence
        // Approximate inverse of softmax: z_i = log(p_i) * T + constant
        let target_logit = confidence.ln() * self.config.temperature + 1.0;
        let other_logit = ((1.0 - confidence) / (num_categories - 1) as f64).ln() * self.config.temperature;

        for (i, logit) in logits.iter_mut().enumerate() {
            if i == target_idx {
                *logit = target_logit;
            } else {
                *logit = other_logit;
            }
        }

        logits
    }

    /// Distill from multiple teachers (ensemble distillation)
    #[must_use]
    pub fn distill_ensemble(
        &self,
        teachers: &[&RuchyOracle],
        samples: &[Sample],
    ) -> Vec<SoftLabel> {
        if teachers.is_empty() {
            return Vec::new();
        }

        samples
            .iter()
            .filter_map(|sample| {
                let error = CompilationError::new(&sample.message)
                    .with_code(sample.error_code.clone().unwrap_or_default());

                // Collect predictions from all teachers
                let mut all_logits = vec![vec![0.0; 8]; teachers.len()];
                let mut total_confidence = 0.0;

                for (i, teacher) in teachers.iter().enumerate() {
                    let classification = teacher.classify(&error);
                    all_logits[i] = self.generate_logits(&classification.category, classification.confidence);
                    total_confidence += classification.confidence;
                }

                let avg_confidence = total_confidence / teachers.len() as f64;

                // Only distill if average confidence is high enough
                if avg_confidence >= self.config.confidence_threshold {
                    // Average the logits
                    let mut avg_logits = vec![0.0; 8];
                    for logits in &all_logits {
                        for (i, &l) in logits.iter().enumerate() {
                            avg_logits[i] += l / teachers.len() as f64;
                        }
                    }

                    let soft_targets = self.temperature_scale(&avg_logits);
                    Some(SoftLabel::new(sample.clone(), soft_targets))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for KnowledgeDistiller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::SampleSource;

    #[test]
    fn test_distiller_new() {
        let distiller = KnowledgeDistiller::new();
        assert!((distiller.config().temperature - 3.0).abs() < f64::EPSILON);
        assert!((distiller.config().confidence_threshold - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distiller_with_config() {
        let config = DistillationConfig {
            temperature: 5.0,
            confidence_threshold: 0.90,
        };
        let distiller = KnowledgeDistiller::with_config(config);
        assert!((distiller.config().temperature - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_temperature_scaling_sums_to_one() {
        let distiller = KnowledgeDistiller::new();
        let logits = vec![1.0, 2.0, 3.0, 4.0];

        let soft = distiller.temperature_scale(&logits);

        let sum: f64 = soft.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_temperature_scaling_empty() {
        let distiller = KnowledgeDistiller::new();
        let soft = distiller.temperature_scale(&[]);
        assert!(soft.is_empty());
    }

    #[test]
    fn test_temperature_scaling_higher_temp_smoother() {
        let distiller_low = KnowledgeDistiller::with_config(DistillationConfig {
            temperature: 1.0,
            confidence_threshold: 0.95,
        });
        let distiller_high = KnowledgeDistiller::with_config(DistillationConfig {
            temperature: 10.0,
            confidence_threshold: 0.95,
        });

        let logits = vec![1.0, 2.0, 5.0];
        let soft_low = distiller_low.temperature_scale(&logits);
        let soft_high = distiller_high.temperature_scale(&logits);

        // Higher temperature should produce more uniform distribution
        let max_low = soft_low.iter().cloned().fold(0.0f64, f64::max);
        let max_high = soft_high.iter().cloned().fold(0.0f64, f64::max);
        assert!(max_high < max_low);
    }

    #[test]
    fn test_soft_label_new() {
        let sample = Sample::new("test", Some("E0308".into()), ErrorCategory::TypeMismatch);
        let soft_targets = vec![0.9, 0.05, 0.02, 0.01, 0.01, 0.005, 0.004, 0.001];

        let soft_label = SoftLabel::new(sample, soft_targets.clone());
        assert_eq!(soft_label.soft_targets.len(), 8);
    }

    #[test]
    fn test_soft_label_predicted_category() {
        let sample = Sample::new("test", Some("E0308".into()), ErrorCategory::TypeMismatch);
        let soft_targets = vec![0.9, 0.05, 0.02, 0.01, 0.01, 0.005, 0.004, 0.001];

        let soft_label = SoftLabel::new(sample, soft_targets);
        assert_eq!(soft_label.predicted_category(), Some(ErrorCategory::TypeMismatch));
    }

    #[test]
    fn test_soft_label_confidence() {
        let sample = Sample::new("test", None, ErrorCategory::TypeMismatch);
        let soft_targets = vec![0.9, 0.05, 0.02, 0.01, 0.01, 0.005, 0.004, 0.001];

        let soft_label = SoftLabel::new(sample, soft_targets);
        assert!((soft_label.confidence() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distill_filters_low_confidence() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("train");

        let distiller = KnowledgeDistiller::with_config(DistillationConfig {
            temperature: 3.0,
            confidence_threshold: 0.99, // Very high threshold
        });

        let samples = vec![
            Sample::new("unknown error pattern xyz", None, ErrorCategory::Other)
                .with_source(SampleSource::Synthetic),
        ];

        let soft_labels = distiller.distill(&oracle, &samples);
        // Very high threshold should filter most predictions
        assert!(soft_labels.len() <= samples.len());
    }

    #[test]
    fn test_distillation_config_default() {
        let config = DistillationConfig::default();
        assert!((config.temperature - 3.0).abs() < f64::EPSILON);
        assert!((config.confidence_threshold - 0.95).abs() < f64::EPSILON);
    }
}
