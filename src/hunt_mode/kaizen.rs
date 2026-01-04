//! Kaizen Metrics and Tracking
//!
//! Implements continuous improvement metrics following the Toyota Way.
//! Each TDD cycle should improve the compilation rate by at least 0.1%.
//!
//! # Toyota Way: Kaizen (Continuous Improvement)
//!
//! "No process can be considered perfect but can always be improved."
//!
//! # References
//! - [7] Imai, M. (1986). Kaizen: The Key to Japan's Competitive Success.
//! - [16] Same, defines small incremental improvements.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::repair::Fix;

/// Kaizen metrics for tracking improvement
#[derive(Debug, Clone)]
pub struct KaizenMetrics {
    /// Current single-shot compile rate (0.0-1.0)
    pub compilation_rate: f64,

    /// Improvement since last cycle
    pub rate_delta: f64,

    /// Cycles since last improvement
    pub cycles_since_improvement: u32,

    /// Total fixes applied
    pub cumulative_fixes: u32,

    /// Total cycles run
    pub total_cycles: u32,

    /// Trend data (last N rates)
    pub trend: Vec<f64>,

    /// Start time
    pub started_at: Option<Instant>,

    /// Total duration
    pub total_duration: Duration,
}

impl Default for KaizenMetrics {
    fn default() -> Self {
        Self {
            compilation_rate: 0.0,
            rate_delta: 0.0,
            cycles_since_improvement: 0,
            cumulative_fixes: 0,
            total_cycles: 0,
            trend: Vec::new(),
            started_at: None,
            total_duration: Duration::ZERO,
        }
    }
}

impl KaizenMetrics {
    /// Create new metrics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if system is improving
    ///
    /// Toyota Way: Small, incremental improvements compound.
    /// Each TDD cycle should improve rate by at least 0.1%.
    #[must_use]
    pub fn is_improving(&self) -> bool {
        self.rate_delta > 0.001 || self.cycles_since_improvement < 5
    }

    /// Check if plateau reached
    #[must_use]
    pub fn is_plateaued(&self, threshold: u32) -> bool {
        self.cycles_since_improvement >= threshold
    }

    /// Get improvement velocity (rate delta per cycle)
    #[must_use]
    pub fn velocity(&self) -> f64 {
        if self.total_cycles == 0 {
            0.0
        } else {
            self.compilation_rate / f64::from(self.total_cycles)
        }
    }

    /// Get estimated cycles to target
    #[must_use]
    pub fn cycles_to_target(&self, target: f64) -> Option<u32> {
        if self.velocity() <= 0.0 || self.compilation_rate >= target {
            return None;
        }

        let remaining = target - self.compilation_rate;
        Some((remaining / self.velocity()).ceil() as u32)
    }

    /// Get success rate as percentage
    #[must_use]
    pub fn success_rate_percent(&self) -> f64 {
        self.compilation_rate * 100.0
    }
}

/// Kaizen tracker for recording improvement
#[derive(Debug)]
pub struct KaizenTracker {
    /// Current metrics
    metrics: KaizenMetrics,

    /// History of compilation rates
    rate_history: VecDeque<f64>,

    /// Maximum history size
    max_history: usize,

    /// Total files in corpus
    total_files: usize,

    /// Currently passing files
    passing_files: usize,
}

impl Default for KaizenTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl KaizenTracker {
    /// Create new tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: KaizenMetrics::new(),
            rate_history: VecDeque::new(),
            max_history: 100,
            total_files: 0,
            passing_files: 0,
        }
    }

    /// Set corpus size
    pub fn set_corpus_size(&mut self, total: usize) {
        self.total_files = total;
        self.update_rate();
    }

    /// Set passing files count
    pub fn set_passing_files(&mut self, passing: usize) {
        self.passing_files = passing;
        self.update_rate();
    }

    /// Start tracking
    pub fn start(&mut self) {
        self.metrics.started_at = Some(Instant::now());
    }

    /// Record a successful fix
    pub fn record_success(&mut self, _fix: &Fix) {
        self.passing_files += 1;
        self.metrics.cumulative_fixes += 1;
        self.update_rate();
    }

    /// Record a cycle completion
    pub fn record_cycle(&mut self, improved: bool) {
        self.metrics.total_cycles += 1;

        if improved {
            self.metrics.cycles_since_improvement = 0;
        } else {
            self.metrics.cycles_since_improvement += 1;
        }

        // Update duration
        if let Some(start) = self.metrics.started_at {
            self.metrics.total_duration = start.elapsed();
        }
    }

    /// Update compilation rate
    fn update_rate(&mut self) {
        let old_rate = self.metrics.compilation_rate;

        if self.total_files > 0 {
            self.metrics.compilation_rate = self.passing_files as f64 / self.total_files as f64;
        }

        self.metrics.rate_delta = self.metrics.compilation_rate - old_rate;

        // Update history
        self.rate_history.push_back(self.metrics.compilation_rate);
        if self.rate_history.len() > self.max_history {
            self.rate_history.pop_front();
        }

        // Update trend (last 7 values)
        self.metrics.trend = self
            .rate_history
            .iter()
            .rev()
            .take(7)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
    }

    /// Get current compilation rate
    #[must_use]
    pub fn current_rate(&self) -> f64 {
        self.metrics.compilation_rate
    }

    /// Get rate delta from last update
    #[must_use]
    pub fn rate_delta(&self) -> f64 {
        self.metrics.rate_delta
    }

    /// Get metrics reference
    #[must_use]
    pub fn metrics(&self) -> &KaizenMetrics {
        &self.metrics
    }

    /// Get mutable metrics
    #[must_use]
    pub fn metrics_mut(&mut self) -> &mut KaizenMetrics {
        &mut self.metrics
    }

    /// Get rate trend
    #[must_use]
    pub fn trend(&self) -> &[f64] {
        &self.metrics.trend
    }

    /// Check if improving
    #[must_use]
    pub fn is_improving(&self) -> bool {
        self.metrics.is_improving()
    }

    /// Render sparkline for trend
    #[must_use]
    pub fn sparkline(&self) -> String {
        if self.metrics.trend.is_empty() {
            return String::new();
        }

        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let min = self
            .metrics
            .trend
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let max = self
            .metrics
            .trend
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        self.metrics
            .trend
            .iter()
            .map(|&v| {
                if range == 0.0 {
                    chars[4]
                } else {
                    let normalized = ((v - min) / range * 7.0).round() as usize;
                    chars[normalized.min(7)]
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: RED PHASE - KaizenMetrics Tests
    // ============================================================================

    #[test]
    fn test_kaizen_metrics_default() {
        let metrics = KaizenMetrics::default();
        assert!((metrics.compilation_rate - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.cumulative_fixes, 0);
        assert_eq!(metrics.total_cycles, 0);
    }

    #[test]
    fn test_kaizen_metrics_new() {
        let metrics = KaizenMetrics::new();
        assert!((metrics.compilation_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_metrics_is_improving_initial() {
        let metrics = KaizenMetrics::new();
        // Initially improving because cycles_since_improvement < 5
        assert!(metrics.is_improving());
    }

    #[test]
    fn test_kaizen_metrics_is_improving_with_delta() {
        let mut metrics = KaizenMetrics::new();
        metrics.rate_delta = 0.01;
        assert!(metrics.is_improving());
    }

    #[test]
    fn test_kaizen_metrics_not_improving_plateau() {
        let mut metrics = KaizenMetrics::new();
        metrics.rate_delta = 0.0;
        metrics.cycles_since_improvement = 10;
        assert!(!metrics.is_improving());
    }

    #[test]
    fn test_kaizen_metrics_is_plateaued() {
        let mut metrics = KaizenMetrics::new();
        metrics.cycles_since_improvement = 5;
        assert!(metrics.is_plateaued(5));
        assert!(!metrics.is_plateaued(6));
    }

    #[test]
    fn test_kaizen_metrics_velocity_zero_cycles() {
        let metrics = KaizenMetrics::new();
        assert!((metrics.velocity() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_metrics_velocity_with_cycles() {
        let mut metrics = KaizenMetrics::new();
        metrics.compilation_rate = 0.5;
        metrics.total_cycles = 10;
        assert!((metrics.velocity() - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_metrics_cycles_to_target_none_at_target() {
        let mut metrics = KaizenMetrics::new();
        metrics.compilation_rate = 0.8;
        assert!(metrics.cycles_to_target(0.8).is_none());
    }

    #[test]
    fn test_kaizen_metrics_cycles_to_target_none_zero_velocity() {
        let metrics = KaizenMetrics::new();
        assert!(metrics.cycles_to_target(0.8).is_none());
    }

    #[test]
    fn test_kaizen_metrics_success_rate_percent() {
        let mut metrics = KaizenMetrics::new();
        metrics.compilation_rate = 0.85;
        assert!((metrics.success_rate_percent() - 85.0).abs() < f64::EPSILON);
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - KaizenTracker Tests
    // ============================================================================

    #[test]
    fn test_kaizen_tracker_new() {
        let tracker = KaizenTracker::new();
        assert!((tracker.current_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_tracker_default() {
        let tracker = KaizenTracker::default();
        assert!((tracker.current_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_tracker_set_corpus_size() {
        let mut tracker = KaizenTracker::new();
        tracker.set_corpus_size(100);
        assert_eq!(tracker.total_files, 100);
    }

    #[test]
    fn test_kaizen_tracker_set_passing_files() {
        let mut tracker = KaizenTracker::new();
        tracker.set_corpus_size(100);
        tracker.set_passing_files(50);
        assert!((tracker.current_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_tracker_record_success() {
        let mut tracker = KaizenTracker::new();
        tracker.set_corpus_size(100);
        tracker.set_passing_files(50);

        let fix = Fix {
            id: "FIX-001".to_string(),
            pattern_id: "PAT-001".to_string(),
            description: "Test fix".to_string(),
            rust_output: String::new(),
            confidence: 0.9,
            transformation: String::new(),
        };

        tracker.record_success(&fix);
        assert!((tracker.current_rate() - 0.51).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_tracker_record_cycle() {
        let mut tracker = KaizenTracker::new();
        tracker.record_cycle(true);
        assert_eq!(tracker.metrics().total_cycles, 1);
        assert_eq!(tracker.metrics().cycles_since_improvement, 0);

        tracker.record_cycle(false);
        assert_eq!(tracker.metrics().total_cycles, 2);
        assert_eq!(tracker.metrics().cycles_since_improvement, 1);
    }

    #[test]
    fn test_kaizen_tracker_rate_delta() {
        let mut tracker = KaizenTracker::new();
        tracker.set_corpus_size(100);
        tracker.set_passing_files(50);
        tracker.set_passing_files(60);
        assert!((tracker.rate_delta() - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kaizen_tracker_trend() {
        let mut tracker = KaizenTracker::new();
        tracker.set_corpus_size(100);

        for i in 1..=10 {
            tracker.set_passing_files(i * 5);
        }

        let trend = tracker.trend();
        assert!(!trend.is_empty());
        assert!(trend.len() <= 7);
    }

    #[test]
    fn test_kaizen_tracker_is_improving() {
        let tracker = KaizenTracker::new();
        assert!(tracker.is_improving());
    }

    #[test]
    fn test_kaizen_tracker_sparkline_empty() {
        let tracker = KaizenTracker::new();
        assert!(tracker.sparkline().is_empty());
    }

    #[test]
    fn test_kaizen_tracker_sparkline_with_data() {
        let mut tracker = KaizenTracker::new();
        tracker.set_corpus_size(100);

        for i in 1..=5 {
            tracker.set_passing_files(i * 10);
        }

        let sparkline = tracker.sparkline();
        assert!(!sparkline.is_empty());
    }

    #[test]
    fn test_kaizen_tracker_start() {
        let mut tracker = KaizenTracker::new();
        tracker.start();
        assert!(tracker.metrics().started_at.is_some());
    }
}
