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
            a.accuracy
                .partial_cmp(&b.accuracy)
                .unwrap_or(std::cmp::Ordering::Equal)
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
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
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
        assert!(report
            .issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Error)));
    }

    #[test]
    fn test_hansei_report_with_warning() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Warning;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        // Warning should show warning severity
        assert!(report
            .issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Warning)));
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

    // COVERAGE-95: Additional tests for complete coverage

    #[test]
    fn test_issues_by_severity_empty() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Stable;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        let critical_issues = report.issues_by_severity(Severity::Critical);
        assert!(critical_issues.is_empty());
    }

    #[test]
    fn test_issues_by_severity_warning() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Warning;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        let warnings = report.issues_by_severity(Severity::Warning);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_issues_by_severity_error() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Drift;
        let categories = HashMap::new();

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        let errors = report.issues_by_severity(Severity::Error);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_category_stats_low_accuracy() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Stable;
        let mut categories = HashMap::new();
        categories.insert(ErrorCategory::TypeMismatch, (0.70, 10));

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        assert!(!report.categories.is_empty());
        // Low accuracy should generate a warning
        assert!(report
            .issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Warning)));
    }

    #[test]
    fn test_category_stats_high_accuracy() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Stable;
        let mut categories = HashMap::new();
        categories.insert(ErrorCategory::BorrowChecker, (0.95, 100));

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        assert!(!report.categories.is_empty());
        assert_eq!(report.categories[0].trend, Trend::Improving);
    }

    #[test]
    fn test_category_stats_medium_accuracy() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Stable;
        let mut categories = HashMap::new();
        categories.insert(ErrorCategory::LifetimeError, (0.85, 50));

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        assert!(!report.categories.is_empty());
        assert_eq!(report.categories[0].trend, Trend::Stable);
    }

    #[test]
    fn test_hansei_issue_fields() {
        let issue = HanseiIssue {
            severity: Severity::Warning,
            category: Some(ErrorCategory::TypeMismatch),
            message: "Test message".to_string(),
            recommendation: "Test recommendation".to_string(),
        };
        assert_eq!(issue.severity, Severity::Warning);
        assert!(issue.category.is_some());
        assert!(!issue.message.is_empty());
        assert!(!issue.recommendation.is_empty());
    }

    #[test]
    fn test_hansei_issue_no_category() {
        let issue = HanseiIssue {
            severity: Severity::Info,
            category: None,
            message: "Info message".to_string(),
            recommendation: "No action needed".to_string(),
        };
        assert_eq!(issue.severity, Severity::Info);
        assert!(issue.category.is_none());
    }

    #[test]
    fn test_category_stats_degrading_trend() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Stable;
        let mut categories = HashMap::new();
        categories.insert(ErrorCategory::SyntaxError, (0.60, 30));

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        assert!(!report.categories.is_empty());
        assert_eq!(report.categories[0].trend, Trend::Degrading);
    }

    #[test]
    fn test_multiple_categories_sorted() {
        let metrics = OracleMetrics::default();
        let drift = DriftStatus::Stable;
        let mut categories = HashMap::new();
        categories.insert(ErrorCategory::TypeMismatch, (0.95, 100));
        categories.insert(ErrorCategory::BorrowChecker, (0.60, 50));
        categories.insert(ErrorCategory::LifetimeError, (0.85, 75));

        let report = HanseiReport::generate(&metrics, &drift, &categories);
        assert_eq!(report.categories.len(), 3);
        // Should be sorted by accuracy (lowest first)
        assert!(report.categories[0].accuracy <= report.categories[1].accuracy);
        assert!(report.categories[1].accuracy <= report.categories[2].accuracy);
    }

    #[test]
    fn test_trend_all_variants() {
        assert_eq!(Trend::Oscillating, Trend::Oscillating);
        assert_eq!(Trend::Stable, Trend::Stable);
        let trends = [
            Trend::Improving,
            Trend::Stable,
            Trend::Degrading,
            Trend::Oscillating,
        ];
        for (i, t1) in trends.iter().enumerate() {
            for (j, t2) in trends.iter().enumerate() {
                if i == j {
                    assert_eq!(t1, t2);
                } else {
                    assert_ne!(t1, t2);
                }
            }
        }
    }

    #[test]
    fn test_severity_all_comparisons() {
        let severities = [
            Severity::Info,
            Severity::Warning,
            Severity::Error,
            Severity::Critical,
        ];
        for (i, s1) in severities.iter().enumerate() {
            for (j, s2) in severities.iter().enumerate() {
                if i < j {
                    assert!(s1 < s2);
                } else if i > j {
                    assert!(s1 > s2);
                } else {
                    assert_eq!(s1, s2);
                }
            }
        }
    }
}
