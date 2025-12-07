//! ROI Metrics Collection for Oracle
//!
//! Tracks efficiency metrics to measure Oracle value:
//! - Classifications performed
//! - Fix suggestions applied
//! - Time saved estimates
//!
//! # References
//! - Spec: docs/specifications/ruchy-oracle-spec.md Section 8

use std::time::{Duration, Instant};

/// Default time saved per successful classification (5 minutes)
const DEFAULT_TIME_SAVED_SECS: u64 = 300;

/// Default LLM cost per API call
const DEFAULT_LLM_COST: f64 = 0.10;

/// Oracle ROI metrics tracker
#[derive(Debug, Clone)]
pub struct OracleMetrics {
    /// Total classifications performed
    pub classifications: usize,

    /// Successful classifications (high confidence)
    pub successful_classifications: usize,

    /// Fix suggestions provided
    pub suggestions_provided: usize,

    /// Auto-fixes applied
    pub auto_fixes_applied: usize,

    /// Estimated time saved (seconds)
    pub time_saved_secs: u64,

    /// Estimated LLM costs avoided
    pub costs_avoided: f64,

    /// Session start time
    start_time: Instant,

    /// Classification times for averaging
    classification_times: Vec<Duration>,
}

impl OracleMetrics {
    /// Create a new metrics tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            classifications: 0,
            successful_classifications: 0,
            suggestions_provided: 0,
            auto_fixes_applied: 0,
            time_saved_secs: 0,
            costs_avoided: 0.0,
            start_time: Instant::now(),
            classification_times: Vec::new(),
        }
    }

    /// Record a classification
    pub fn record_classification(&mut self, confidence: f64, suggestions: usize, auto_fix: bool) {
        self.classifications += 1;

        if confidence >= 0.7 {
            self.successful_classifications += 1;
            // Estimate 5 minutes saved per successful high-confidence classification
            self.time_saved_secs += DEFAULT_TIME_SAVED_SECS;
            // Estimate 9 LLM calls avoided per successful classification
            self.costs_avoided += DEFAULT_LLM_COST * 9.0;
        }

        self.suggestions_provided += suggestions;

        if auto_fix {
            self.auto_fixes_applied += 1;
            // Additional time saved for auto-fixes
            self.time_saved_secs += 120; // 2 more minutes
        }
    }

    /// Record classification timing
    pub fn record_timing(&mut self, duration: Duration) {
        self.classification_times.push(duration);
    }

    /// Get average classification time
    #[must_use]
    pub fn avg_classification_time(&self) -> Duration {
        if self.classification_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.classification_times.iter().sum();
        total / self.classification_times.len() as u32
    }

    /// Get session duration
    #[must_use]
    pub fn session_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get success rate
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.classifications == 0 {
            return 0.0;
        }
        self.successful_classifications as f64 / self.classifications as f64
    }

    /// Get estimated time saved as human-readable string
    #[must_use]
    pub fn time_saved_formatted(&self) -> String {
        let mins = self.time_saved_secs / 60;
        let secs = self.time_saved_secs % 60;
        if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        }
    }

    /// Get cost savings formatted
    #[must_use]
    pub fn costs_avoided_formatted(&self) -> String {
        format!("${:.2}", self.costs_avoided)
    }

    /// Export metrics as JSON
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "classifications": self.classifications,
            "successful_classifications": self.successful_classifications,
            "suggestions_provided": self.suggestions_provided,
            "auto_fixes_applied": self.auto_fixes_applied,
            "success_rate": self.success_rate(),
            "time_saved_secs": self.time_saved_secs,
            "time_saved_formatted": self.time_saved_formatted(),
            "costs_avoided": self.costs_avoided,
            "costs_avoided_formatted": self.costs_avoided_formatted(),
            "avg_classification_ms": self.avg_classification_time().as_millis(),
            "session_duration_secs": self.session_duration().as_secs(),
        })
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for OracleMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = OracleMetrics::new();
        assert_eq!(metrics.classifications, 0);
        assert_eq!(metrics.successful_classifications, 0);
    }

    #[test]
    fn test_record_classification_high_confidence() {
        let mut metrics = OracleMetrics::new();
        metrics.record_classification(0.9, 2, false);

        assert_eq!(metrics.classifications, 1);
        assert_eq!(metrics.successful_classifications, 1);
        assert_eq!(metrics.suggestions_provided, 2);
        assert_eq!(metrics.time_saved_secs, DEFAULT_TIME_SAVED_SECS);
    }

    #[test]
    fn test_record_classification_low_confidence() {
        let mut metrics = OracleMetrics::new();
        metrics.record_classification(0.5, 1, false);

        assert_eq!(metrics.classifications, 1);
        assert_eq!(metrics.successful_classifications, 0);
        assert_eq!(metrics.time_saved_secs, 0);
    }

    #[test]
    fn test_record_auto_fix() {
        let mut metrics = OracleMetrics::new();
        metrics.record_classification(0.9, 1, true);

        assert_eq!(metrics.auto_fixes_applied, 1);
        // 5 min + 2 min = 7 min = 420 secs
        assert_eq!(metrics.time_saved_secs, DEFAULT_TIME_SAVED_SECS + 120);
    }

    #[test]
    fn test_success_rate() {
        let mut metrics = OracleMetrics::new();
        metrics.record_classification(0.9, 1, false); // success
        metrics.record_classification(0.5, 0, false); // fail

        assert!((metrics.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_success_rate_empty() {
        let metrics = OracleMetrics::new();
        assert!((metrics.success_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_time_saved_formatted() {
        let mut metrics = OracleMetrics::new();
        metrics.time_saved_secs = 125; // 2m 5s
        assert_eq!(metrics.time_saved_formatted(), "2m 5s");

        metrics.time_saved_secs = 30;
        assert_eq!(metrics.time_saved_formatted(), "30s");
    }

    #[test]
    fn test_costs_avoided_formatted() {
        let mut metrics = OracleMetrics::new();
        metrics.costs_avoided = 4.50;
        assert_eq!(metrics.costs_avoided_formatted(), "$4.50");
    }

    #[test]
    fn test_avg_classification_time_empty() {
        let metrics = OracleMetrics::new();
        assert_eq!(metrics.avg_classification_time(), Duration::ZERO);
    }

    #[test]
    fn test_avg_classification_time() {
        let mut metrics = OracleMetrics::new();
        metrics.record_timing(Duration::from_millis(10));
        metrics.record_timing(Duration::from_millis(20));

        assert_eq!(metrics.avg_classification_time(), Duration::from_millis(15));
    }

    #[test]
    fn test_to_json() {
        let mut metrics = OracleMetrics::new();
        metrics.record_classification(0.9, 2, true);

        let json = metrics.to_json();
        assert_eq!(json["classifications"], 1);
        assert_eq!(json["successful_classifications"], 1);
        assert_eq!(json["auto_fixes_applied"], 1);
    }

    #[test]
    fn test_reset() {
        let mut metrics = OracleMetrics::new();
        metrics.record_classification(0.9, 2, true);
        metrics.reset();

        assert_eq!(metrics.classifications, 0);
        assert_eq!(metrics.time_saved_secs, 0);
    }

    #[test]
    fn test_default_impl() {
        let metrics = OracleMetrics::default();
        assert_eq!(metrics.classifications, 0);
    }
}
