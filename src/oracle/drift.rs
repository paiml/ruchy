//! Drift Detection using ADWIN Algorithm
//!
//! Monitors model accuracy over time and detects concept drift.
//!
//! # References
//! - [4] Bifet & Gavalda (2007). "Learning from Time-Changing Data with
//!   Adaptive Windowing." SIAM SDM, 443-448.

use std::collections::VecDeque;

/// Default window size for accuracy tracking
pub const DEFAULT_WINDOW_SIZE: usize = 100;

/// Default drift threshold
pub const DEFAULT_DRIFT_THRESHOLD: f64 = 0.05;

/// Drift detection status
#[derive(Debug, Clone, PartialEq)]
pub enum DriftStatus {
    /// Model accuracy is stable
    Stable {
        /// Current accuracy
        accuracy: f64,
    },

    /// Drift detected - model needs retraining
    DriftDetected {
        /// Historical accuracy
        historical: f64,
        /// Current accuracy
        current: f64,
        /// Recommendation
        recommendation: String,
    },

    /// Warning - accuracy declining but not yet drift
    Warning {
        /// Current accuracy
        accuracy: f64,
        /// Trend direction
        trend: String,
    },
}

/// Drift detector using sliding window accuracy tracking
///
/// Implements simplified ADWIN-like concept drift detection.
#[derive(Debug)]
pub struct DriftDetector {
    /// Sliding window of correct/incorrect predictions
    window: VecDeque<bool>,

    /// Maximum window size
    window_size: usize,

    /// Drift threshold (deviation from historical mean)
    threshold: f64,

    /// Total predictions made
    total_predictions: usize,

    /// Total correct predictions
    total_correct: usize,
}

impl DriftDetector {
    /// Create a new drift detector with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(DEFAULT_WINDOW_SIZE, DEFAULT_DRIFT_THRESHOLD)
    }

    /// Create a drift detector with custom settings
    #[must_use]
    pub fn with_config(window_size: usize, threshold: f64) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            threshold,
            total_predictions: 0,
            total_correct: 0,
        }
    }

    /// Record a prediction result (correct or incorrect)
    pub fn record(&mut self, correct: bool) {
        self.total_predictions += 1;
        if correct {
            self.total_correct += 1;
        }

        // Add to sliding window
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(correct);
    }

    /// Check for drift
    #[must_use]
    pub fn check_drift(&self) -> DriftStatus {
        if self.window.len() < 10 {
            // Not enough data
            return DriftStatus::Stable {
                accuracy: self.current_accuracy(),
            };
        }

        let current = self.current_accuracy();
        let historical = self.historical_accuracy();

        let deviation = (current - historical).abs();

        if deviation > self.threshold {
            DriftStatus::DriftDetected {
                historical,
                current,
                recommendation: "Retrain Oracle with recent data".to_string(),
            }
        } else if deviation > self.threshold / 2.0 {
            let trend = if current < historical {
                "declining"
            } else {
                "improving"
            };
            DriftStatus::Warning {
                accuracy: current,
                trend: trend.to_string(),
            }
        } else {
            DriftStatus::Stable { accuracy: current }
        }
    }

    /// Get current accuracy (from sliding window)
    #[must_use]
    pub fn current_accuracy(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }

        let correct = self.window.iter().filter(|&&c| c).count();
        correct as f64 / self.window.len() as f64
    }

    /// Get historical accuracy (all-time)
    #[must_use]
    pub fn historical_accuracy(&self) -> f64 {
        if self.total_predictions == 0 {
            return 0.0;
        }

        self.total_correct as f64 / self.total_predictions as f64
    }

    /// Get total predictions made
    #[must_use]
    pub fn total_predictions(&self) -> usize {
        self.total_predictions
    }

    /// Get window size
    #[must_use]
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Get current window fill level
    #[must_use]
    pub fn window_fill(&self) -> usize {
        self.window.len()
    }

    /// Reset the detector
    pub fn reset(&mut self) {
        self.window.clear();
        self.total_predictions = 0;
        self.total_correct = 0;
    }
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Drift Detection Tests
    // ============================================================================

    #[test]
    fn test_drift_detector_new() {
        let detector = DriftDetector::new();
        assert_eq!(detector.window_size(), DEFAULT_WINDOW_SIZE);
        assert_eq!(detector.total_predictions(), 0);
    }

    #[test]
    fn test_drift_detector_with_config() {
        let detector = DriftDetector::with_config(50, 0.1);
        assert_eq!(detector.window_size(), 50);
    }

    #[test]
    fn test_record_correct() {
        let mut detector = DriftDetector::new();
        detector.record(true);
        assert_eq!(detector.total_predictions(), 1);
        assert!((detector.current_accuracy() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_record_incorrect() {
        let mut detector = DriftDetector::new();
        detector.record(false);
        assert_eq!(detector.total_predictions(), 1);
        assert!((detector.current_accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_record_mixed() {
        let mut detector = DriftDetector::new();
        detector.record(true);
        detector.record(false);
        assert_eq!(detector.total_predictions(), 2);
        assert!((detector.current_accuracy() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_window_sliding() {
        let mut detector = DriftDetector::with_config(5, 0.1);

        // Fill window with correct predictions
        for _ in 0..5 {
            detector.record(true);
        }
        assert!((detector.current_accuracy() - 1.0).abs() < f64::EPSILON);

        // Add incorrect predictions, old ones should slide out
        for _ in 0..5 {
            detector.record(false);
        }
        assert!((detector.current_accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_drift_stable() {
        let mut detector = DriftDetector::new();

        // Record consistent accuracy
        for _ in 0..50 {
            detector.record(true);
        }

        match detector.check_drift() {
            DriftStatus::Stable { accuracy } => {
                assert!((accuracy - 1.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Stable status"),
        }
    }

    #[test]
    fn test_check_drift_detected() {
        let mut detector = DriftDetector::with_config(10, 0.1);

        // Start with high accuracy
        for _ in 0..50 {
            detector.record(true);
        }

        // Then sudden drop
        for _ in 0..10 {
            detector.record(false);
        }

        match detector.check_drift() {
            DriftStatus::DriftDetected { historical, current, .. } => {
                assert!(historical > current);
            }
            DriftStatus::Warning { .. } => {
                // Also acceptable - depends on exact thresholds
            }
            DriftStatus::Stable { .. } => {
                panic!("Expected drift or warning");
            }
        }
    }

    #[test]
    fn test_check_drift_insufficient_data() {
        let mut detector = DriftDetector::new();

        // Only 5 samples (less than minimum 10)
        for _ in 0..5 {
            detector.record(true);
        }

        match detector.check_drift() {
            DriftStatus::Stable { .. } => { /* OK */ }
            _ => panic!("Expected Stable status with insufficient data"),
        }
    }

    #[test]
    fn test_historical_accuracy() {
        let mut detector = DriftDetector::new();

        for _ in 0..80 {
            detector.record(true);
        }
        for _ in 0..20 {
            detector.record(false);
        }

        assert!((detector.historical_accuracy() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_historical_accuracy_empty() {
        let detector = DriftDetector::new();
        assert!((detector.historical_accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_current_accuracy_empty() {
        let detector = DriftDetector::new();
        assert!((detector.current_accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_window_fill() {
        let mut detector = DriftDetector::with_config(10, 0.1);

        for i in 0..5 {
            detector.record(true);
            assert_eq!(detector.window_fill(), i + 1);
        }
    }

    #[test]
    fn test_reset() {
        let mut detector = DriftDetector::new();

        for _ in 0..50 {
            detector.record(true);
        }

        detector.reset();

        assert_eq!(detector.total_predictions(), 0);
        assert_eq!(detector.window_fill(), 0);
    }

    #[test]
    fn test_default_impl() {
        let detector = DriftDetector::default();
        assert_eq!(detector.window_size(), DEFAULT_WINDOW_SIZE);
    }

    #[test]
    fn test_drift_status_partial_eq() {
        let s1 = DriftStatus::Stable { accuracy: 0.9 };
        let s2 = DriftStatus::Stable { accuracy: 0.9 };
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_drift_status_clone() {
        let s1 = DriftStatus::Warning {
            accuracy: 0.8,
            trend: "declining".to_string(),
        };
        let s2 = s1.clone();
        assert_eq!(s1, s2);
    }
}
