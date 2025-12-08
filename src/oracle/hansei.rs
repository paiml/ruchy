//! Hansei (反省) Reflection Analysis
//!
//! Toyota Way principle of continuous self-reflection for Oracle performance.
//!
//! # References
//! - [8] Liker, J. K. (2004). "The Toyota Way." McGraw-Hill.
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §5

use super::{DriftStatus, ErrorCategory, OracleMetrics};
use std::collections::HashMap;

/// Trend direction for metric analysis
#[derive(Debug, Clone, PartialEq)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
    Oscillating,
}

/// Issue severity levels (Andon signals)
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// An issue identified during reflection
#[derive(Debug, Clone)]
pub struct HanseiIssue {
    pub severity: Severity,
    pub category: Option<ErrorCategory>,
    pub message: String,
    pub recommendation: String,
}

/// Category performance summary
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: ErrorCategory,
    pub accuracy: f64,
    pub sample_count: usize,
    pub trend: Trend,
}

/// Hansei reflection report
#[derive(Debug)]
pub struct HanseiReport {
    /// Overall accuracy
    pub overall_accuracy: f64,
    /// Single-shot fix rate
    pub fix_rate: f64,
    /// Model age in days
    pub model_age_days: u64,
    /// Category breakdown
    pub categories: Vec<CategoryStats>,
    /// Issues identified
    pub issues: Vec<HanseiIssue>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Overall trend
    pub trend: Trend,
}

impl HanseiReport {
    /// Generate report from metrics and drift status
    #[must_use]
    pub fn generate(
        metrics: &OracleMetrics,
        drift: &DriftStatus,
        category_accuracies: &HashMap<ErrorCategory, (f64, usize)>,
    ) -> Self {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut categories = Vec::new();

        // Analyze each category
        for (&cat, &(accuracy, count)) in category_accuracies {
            let trend = if accuracy >= 0.90 {
                Trend::Improving
            } else if accuracy >= 0.80 {
                Trend::Stable
            } else {
                Trend::Degrading
            };

            categories.push(CategoryStats {
                category: cat,
                accuracy,
                sample_count: count,
                trend: trend.clone(),
            });

            // Flag low accuracy categories
            if accuracy < 0.80 {
                issues.push(HanseiIssue {
                    severity: Severity::Warning,
                    category: Some(cat),
                    message: format!("{cat:?} accuracy below threshold (80%)"),
                    recommendation: format!("Add more {cat:?} training samples"),
                });
            }
        }

        // Check drift status (using aprender::online::drift::DriftStatus)
        match drift {
            DriftStatus::Drift => {
                issues.push(HanseiIssue {
                    severity: Severity::Error,
                    category: None,
                    message: "Model drift detected".to_string(),
                    recommendation: "Retrain model immediately".to_string(),
                });
                recommendations.push("Schedule immediate retraining".to_string());
            }
            DriftStatus::Warning => {
                issues.push(HanseiIssue {
                    severity: Severity::Warning,
                    category: None,
                    message: "Potential drift warning".to_string(),
                    recommendation: "Monitor closely".to_string(),
                });
            }
            DriftStatus::Stable => {}
        }

        // Sort categories by accuracy (lowest first for attention)
        categories.sort_by(|a, b| {
            a.accuracy.partial_cmp(&b.accuracy).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Overall trend
        let trend = if metrics.success_rate() >= 0.90 {
            Trend::Improving
        } else if metrics.success_rate() >= 0.80 {
            Trend::Stable
        } else {
            Trend::Degrading
        };

        Self {
            overall_accuracy: metrics.success_rate(),
            fix_rate: metrics.success_rate(),
            model_age_days: 0, // Would be calculated from metadata
            categories,
            issues,
            recommendations,
            trend,
        }
    }

    /// Check if any critical issues exist
    #[must_use]
    pub fn has_critical(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Critical)
    }

    /// Get issues by severity
    #[must_use]
    pub fn issues_by_severity(&self, severity: Severity) -> Vec<&HanseiIssue> {
        self.issues.iter().filter(|i| i.severity == severity).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hansei_report_generate() {
        let metrics = OracleMetrics::default();
        // aprender::online::drift::DriftStatus uses unit variants
        let drift = DriftStatus::Stable;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        assert!(!report.has_critical());
    }

    #[test]
    fn test_hansei_report_with_drift() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Drift;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        // Drift detection should flag as needing attention
        assert!(report.issues.iter().any(|i| matches!(i.severity, Severity::Error)));
    }

    #[test]
    fn test_hansei_report_with_warning() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Warning;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        // Warning should show warning severity
        assert!(report.issues.iter().any(|i| matches!(i.severity, Severity::Warning)));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn test_trend_variants() {
        assert_eq!(Trend::Improving, Trend::Improving);
        assert_ne!(Trend::Improving, Trend::Degrading);
    }
}
