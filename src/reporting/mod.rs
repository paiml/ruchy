//! Visualization-Enhanced Transpile Oracle Reporting
//!
//! Implements rich text reporting for transpilation analysis following
//! Toyota Way principles (Mieruka, Jidoka, Kaizen).
//!
//! # Academic Foundation
//! - [1] Jones et al. (2002). Tarantula fault localization. ICSE '02.
//! - [5] Ohno (1988). Toyota Production System. Mieruka visual management.
//! - [7] Imai (1986). Kaizen continuous improvement.
//! - [8] Juran (1988). Pareto principle (vital few).
//! - [9] Deming (1986). Statistical process control.
//! - [11] Zeller & Hildebrandt (2002). Delta Debugging. IEEE TSE.

pub mod ascii;
pub mod bisect;
pub mod dashboard;
pub mod formats;
pub mod pareto;
pub mod pipeline;
pub mod sbfl;
pub mod semantic;

// Re-exports
pub use ascii::{andon_status, grade, progress_bar, sparkline, AndonStatus, Grade};
pub use bisect::{BisectSession, BisectState, BisectStep, DeltaDebugger, TestResult};
pub use dashboard::{ConvergenceDashboard, ConvergenceIteration, ConvergenceState, FixAttempt};
pub use formats::{OutputFormat, ReportFormatter};
pub use pareto::{BlockerPriority, ParetoAnalysis};
pub use pipeline::{CorpusPipeline, Phase, PhaseResult, PipelineBuilder, PipelineExecution};
pub use sbfl::{SbflFormula, SbflRanking, SuspiciousnessScore};
pub use semantic::{CorpusFilter, SemanticTag, SemanticTagger, TaggedFile};

/// Transpilation report result
#[derive(Debug, Clone)]
pub struct TranspileReport {
    /// Total files processed
    pub total: usize,
    /// Files that passed transpilation + compilation
    pub passed: usize,
    /// Files that failed
    pub failed: usize,
    /// Error taxonomy
    pub errors: Vec<ErrorEntry>,
    /// Trend data (last 7 data points)
    pub trend: Vec<f64>,
}

impl TranspileReport {
    /// Create new report
    #[must_use]
    pub fn new(total: usize, passed: usize, failed: usize) -> Self {
        Self {
            total,
            passed,
            failed,
            errors: Vec::new(),
            trend: Vec::new(),
        }
    }

    /// Calculate success rate as percentage
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    /// Get Andon status based on success rate
    #[must_use]
    pub fn andon(&self) -> AndonStatus {
        andon_status(self.success_rate())
    }

    /// Get grade based on success rate
    #[must_use]
    pub fn grade(&self) -> Grade {
        grade(self.success_rate())
    }

    /// Add error entry
    pub fn add_error(&mut self, error: ErrorEntry) {
        self.errors.push(error);
    }

    /// Set trend data
    pub fn with_trend(mut self, trend: Vec<f64>) -> Self {
        self.trend = trend;
        self
    }
}

/// Error entry for taxonomy
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    /// Error code (e.g., "E0308")
    pub code: String,
    /// Error count
    pub count: usize,
    /// Sample error messages
    pub samples: Vec<String>,
}

impl ErrorEntry {
    /// Create new error entry
    #[must_use]
    pub fn new(code: impl Into<String>, count: usize) -> Self {
        Self {
            code: code.into(),
            count,
            samples: Vec::new(),
        }
    }

    /// Add sample message
    pub fn with_sample(mut self, sample: impl Into<String>) -> Self {
        if self.samples.len() < 3 {
            self.samples.push(sample.into());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: RED PHASE - TranspileReport Tests
    // ============================================================

    #[test]
    fn test_report_new() {
        let report = TranspileReport::new(100, 85, 15);
        assert_eq!(report.total, 100);
        assert_eq!(report.passed, 85);
        assert_eq!(report.failed, 15);
    }

    #[test]
    fn test_report_success_rate() {
        let report = TranspileReport::new(100, 85, 15);
        assert!((report.success_rate() - 85.0).abs() < 0.01);
    }

    #[test]
    fn test_report_success_rate_zero_total() {
        let report = TranspileReport::new(0, 0, 0);
        assert!((report.success_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_report_success_rate_perfect() {
        let report = TranspileReport::new(50, 50, 0);
        assert!((report.success_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_report_andon_green() {
        let report = TranspileReport::new(100, 85, 15);
        assert_eq!(report.andon(), AndonStatus::Green);
    }

    #[test]
    fn test_report_andon_yellow() {
        let report = TranspileReport::new(100, 65, 35);
        assert_eq!(report.andon(), AndonStatus::Yellow);
    }

    #[test]
    fn test_report_andon_red() {
        let report = TranspileReport::new(100, 40, 60);
        assert_eq!(report.andon(), AndonStatus::Red);
    }

    #[test]
    fn test_report_grade_a_plus() {
        let report = TranspileReport::new(100, 97, 3);
        assert_eq!(report.grade(), Grade::APlus);
    }

    #[test]
    fn test_report_add_error() {
        let mut report = TranspileReport::new(100, 85, 15);
        report.add_error(ErrorEntry::new("E0308", 5));
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].code, "E0308");
    }

    #[test]
    fn test_error_entry_with_sample() {
        let entry = ErrorEntry::new("E0308", 5)
            .with_sample("mismatched types: expected i32, found String");
        assert_eq!(entry.samples.len(), 1);
    }

    #[test]
    fn test_error_entry_max_samples() {
        let entry = ErrorEntry::new("E0308", 5)
            .with_sample("sample 1")
            .with_sample("sample 2")
            .with_sample("sample 3")
            .with_sample("sample 4"); // Should be ignored
        assert_eq!(entry.samples.len(), 3);
    }

    #[test]
    fn test_report_with_trend() {
        let report = TranspileReport::new(100, 85, 15)
            .with_trend(vec![70.0, 72.0, 75.0, 78.0, 80.0, 83.0, 85.0]);
        assert_eq!(report.trend.len(), 7);
    }
}
