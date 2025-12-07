//! Hunt Planner: Error Classification and Pattern Selection
//!
//! Implements the PLAN phase of Hunt Mode's PDCA cycle.
//! Uses error clustering to identify high-impact failure patterns.
//!
//! # Toyota Way: Heijunka (Level the Workload)
//!
//! Process highest-impact patterns first to ensure maximum improvement per cycle.
//! Pareto principle: 20% of patterns cause 80% of failures.
//!
//! # References
//! - [1] Pareto, V. (1896). 80/20 rule.
//! - [17] Rother & Shook (1999). Value Stream Mapping.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Error cluster representing a group of similar errors
#[derive(Debug, Clone)]
pub struct ErrorCluster {
    /// Error code (e.g., "E0308")
    pub code: String,

    /// Number of occurrences
    pub count: usize,

    /// Representative error message
    pub representative: String,

    /// Sample file paths
    pub sample_files: Vec<String>,

    /// Severity score (higher = more critical)
    pub severity: f64,
}

impl ErrorCluster {
    /// Create new error cluster
    #[must_use]
    pub fn new(code: impl Into<String>, representative: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            count: 1,
            representative: representative.into(),
            sample_files: Vec::new(),
            severity: 1.0,
        }
    }

    /// Add occurrence to cluster
    pub fn add_occurrence(&mut self, file: Option<&str>) {
        self.count += 1;
        if let Some(f) = file {
            if self.sample_files.len() < 3 {
                self.sample_files.push(f.to_string());
            }
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: f64) -> Self {
        self.severity = severity;
        self
    }

    /// Calculate priority score (frequency * severity)
    #[must_use]
    pub fn priority_score(&self) -> f64 {
        self.count as f64 * self.severity
    }
}

/// Failure pattern identified for fixing
#[derive(Debug, Clone)]
pub struct FailurePattern {
    /// Unique pattern ID
    pub id: String,

    /// Error code
    pub error_code: String,

    /// Pattern description
    pub description: String,

    /// Number of files affected
    pub affected_count: usize,

    /// Estimated fix complexity (1-10)
    pub complexity: u8,

    /// Sample code that exhibits the error
    pub sample_code: Option<String>,

    /// Sample error message
    pub sample_error: Option<String>,
}

impl FailurePattern {
    /// Create new failure pattern
    #[must_use]
    pub fn new(id: impl Into<String>, error_code: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            error_code: error_code.into(),
            description: String::new(),
            affected_count: 0,
            complexity: 5,
            sample_code: None,
            sample_error: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set affected count
    pub fn with_affected_count(mut self, count: usize) -> Self {
        self.affected_count = count;
        self
    }

    /// Set complexity
    pub fn with_complexity(mut self, complexity: u8) -> Self {
        self.complexity = complexity;
        self
    }

    /// Set sample code
    pub fn with_sample_code(mut self, code: impl Into<String>) -> Self {
        self.sample_code = Some(code.into());
        self
    }

    /// Set sample error
    pub fn with_sample_error(mut self, error: impl Into<String>) -> Self {
        self.sample_error = Some(error.into());
        self
    }
}

/// Prioritized pattern wrapper for heap ordering
#[derive(Debug, Clone)]
pub struct PrioritizedPattern {
    /// The pattern
    pub pattern: FailurePattern,

    /// Priority score (higher = more important)
    pub priority: f64,
}

impl PartialEq for PrioritizedPattern {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedPattern {}

impl PartialOrd for PrioritizedPattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedPattern {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        self.priority
            .partial_cmp(&other.priority)
            .unwrap_or(Ordering::Equal)
    }
}

/// Hunt Planner for pattern identification and selection
#[derive(Debug)]
pub struct HuntPlanner {
    /// Error clusters by code
    clusters: HashMap<String, ErrorCluster>,

    /// Priority queue of patterns
    priority_queue: BinaryHeap<PrioritizedPattern>,

    /// Patterns already processed
    processed: Vec<String>,
}

impl Default for HuntPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl HuntPlanner {
    /// Create new planner
    #[must_use]
    pub fn new() -> Self {
        Self {
            clusters: HashMap::new(),
            priority_queue: BinaryHeap::new(),
            processed: Vec::new(),
        }
    }

    /// Add error to clustering
    pub fn add_error(
        &mut self,
        code: &str,
        message: &str,
        file: Option<&str>,
        severity: f64,
    ) {
        if let Some(cluster) = self.clusters.get_mut(code) {
            // Cluster exists, add occurrence
            cluster.add_occurrence(file);
        } else {
            // Create new cluster with initial count of 1
            let mut cluster = ErrorCluster::new(code, message).with_severity(severity);
            if let Some(f) = file {
                cluster.sample_files.push(f.to_string());
            }
            self.clusters.insert(code.to_string(), cluster);
        }
    }

    /// Build priority queue from clusters
    pub fn build_priority_queue(&mut self) {
        self.priority_queue.clear();

        for cluster in self.clusters.values() {
            // Skip already processed patterns
            if self.processed.contains(&cluster.code) {
                continue;
            }

            let pattern = FailurePattern::new(
                format!("PAT-{}", cluster.code),
                &cluster.code,
            )
            .with_description(&cluster.representative)
            .with_affected_count(cluster.count);

            let priority = cluster.priority_score();

            self.priority_queue.push(PrioritizedPattern { pattern, priority });
        }
    }

    /// Select next target pattern (Heijunka - highest impact first)
    #[must_use]
    pub fn select_next_target(&mut self) -> Option<FailurePattern> {
        // Build queue if empty
        if self.priority_queue.is_empty() {
            self.build_priority_queue();
        }

        self.priority_queue.pop().map(|p| {
            self.processed.push(p.pattern.error_code.clone());
            p.pattern
        })
    }

    /// Get all clusters
    #[must_use]
    pub fn clusters(&self) -> &HashMap<String, ErrorCluster> {
        &self.clusters
    }

    /// Get top N clusters by priority
    #[must_use]
    pub fn top_clusters(&self, n: usize) -> Vec<&ErrorCluster> {
        let mut clusters: Vec<_> = self.clusters.values().collect();
        clusters.sort_by(|a, b| {
            b.priority_score()
                .partial_cmp(&a.priority_score())
                .unwrap_or(Ordering::Equal)
        });
        clusters.into_iter().take(n).collect()
    }

    /// Get total error count
    #[must_use]
    pub fn total_errors(&self) -> usize {
        self.clusters.values().map(|c| c.count).sum()
    }

    /// Get unique error count
    #[must_use]
    pub fn unique_errors(&self) -> usize {
        self.clusters.len()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.clusters.clear();
        self.priority_queue.clear();
        self.processed.clear();
    }

    /// Reset processed patterns (for retry)
    pub fn reset_processed(&mut self) {
        self.processed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: RED PHASE - ErrorCluster Tests
    // ============================================================================

    #[test]
    fn test_error_cluster_new() {
        let cluster = ErrorCluster::new("E0308", "mismatched types");
        assert_eq!(cluster.code, "E0308");
        assert_eq!(cluster.count, 1);
    }

    #[test]
    fn test_error_cluster_add_occurrence() {
        let mut cluster = ErrorCluster::new("E0308", "mismatched types");
        cluster.add_occurrence(Some("test.rs"));
        assert_eq!(cluster.count, 2);
        assert_eq!(cluster.sample_files.len(), 1);
    }

    #[test]
    fn test_error_cluster_max_samples() {
        let mut cluster = ErrorCluster::new("E0308", "mismatched types");
        for i in 0..5 {
            cluster.add_occurrence(Some(&format!("test{i}.rs")));
        }
        assert_eq!(cluster.sample_files.len(), 3);
    }

    #[test]
    fn test_error_cluster_with_severity() {
        let cluster = ErrorCluster::new("E0308", "mismatched types").with_severity(2.0);
        assert!((cluster.severity - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_error_cluster_priority_score() {
        let mut cluster = ErrorCluster::new("E0308", "mismatched types").with_severity(2.0);
        cluster.add_occurrence(None);
        cluster.add_occurrence(None);
        // count=3, severity=2.0, score=6.0
        assert!((cluster.priority_score() - 6.0).abs() < f64::EPSILON);
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - FailurePattern Tests
    // ============================================================================

    #[test]
    fn test_failure_pattern_new() {
        let pattern = FailurePattern::new("PAT-001", "E0308");
        assert_eq!(pattern.id, "PAT-001");
        assert_eq!(pattern.error_code, "E0308");
    }

    #[test]
    fn test_failure_pattern_with_description() {
        let pattern = FailurePattern::new("PAT-001", "E0308")
            .with_description("Type mismatch error");
        assert_eq!(pattern.description, "Type mismatch error");
    }

    #[test]
    fn test_failure_pattern_with_affected_count() {
        let pattern = FailurePattern::new("PAT-001", "E0308")
            .with_affected_count(10);
        assert_eq!(pattern.affected_count, 10);
    }

    #[test]
    fn test_failure_pattern_with_complexity() {
        let pattern = FailurePattern::new("PAT-001", "E0308")
            .with_complexity(8);
        assert_eq!(pattern.complexity, 8);
    }

    #[test]
    fn test_failure_pattern_with_sample_code() {
        let pattern = FailurePattern::new("PAT-001", "E0308")
            .with_sample_code("fn foo() {}");
        assert_eq!(pattern.sample_code, Some("fn foo() {}".to_string()));
    }

    #[test]
    fn test_failure_pattern_with_sample_error() {
        let pattern = FailurePattern::new("PAT-001", "E0308")
            .with_sample_error("expected i32, found String");
        assert_eq!(pattern.sample_error, Some("expected i32, found String".to_string()));
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - PrioritizedPattern Tests
    // ============================================================================

    #[test]
    fn test_prioritized_pattern_ordering() {
        let p1 = PrioritizedPattern {
            pattern: FailurePattern::new("PAT-001", "E0308"),
            priority: 10.0,
        };
        let p2 = PrioritizedPattern {
            pattern: FailurePattern::new("PAT-002", "E0599"),
            priority: 5.0,
        };
        assert!(p1 > p2);
    }

    #[test]
    fn test_prioritized_pattern_equality() {
        let p1 = PrioritizedPattern {
            pattern: FailurePattern::new("PAT-001", "E0308"),
            priority: 10.0,
        };
        let p2 = PrioritizedPattern {
            pattern: FailurePattern::new("PAT-002", "E0599"),
            priority: 10.0,
        };
        assert_eq!(p1, p2);
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - HuntPlanner Tests
    // ============================================================================

    #[test]
    fn test_hunt_planner_new() {
        let planner = HuntPlanner::new();
        assert!(planner.clusters().is_empty());
    }

    #[test]
    fn test_hunt_planner_default() {
        let planner = HuntPlanner::default();
        assert!(planner.clusters().is_empty());
    }

    #[test]
    fn test_hunt_planner_add_error() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", Some("test.rs"), 1.0);
        assert_eq!(planner.unique_errors(), 1);
    }

    #[test]
    fn test_hunt_planner_add_multiple_same_code() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", Some("test1.rs"), 1.0);
        planner.add_error("E0308", "mismatched types", Some("test2.rs"), 1.0);
        assert_eq!(planner.unique_errors(), 1);
        // First add creates cluster with count=1
        // Second add increments count to 2
        assert_eq!(planner.total_errors(), 2);
    }

    #[test]
    fn test_hunt_planner_add_different_codes() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", None, 1.0);
        planner.add_error("E0599", "method not found", None, 1.0);
        assert_eq!(planner.unique_errors(), 2);
    }

    #[test]
    fn test_hunt_planner_select_next_target_empty() {
        let mut planner = HuntPlanner::new();
        assert!(planner.select_next_target().is_none());
    }

    #[test]
    fn test_hunt_planner_select_next_target() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", None, 1.0);
        let pattern = planner.select_next_target();
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().error_code, "E0308");
    }

    #[test]
    fn test_hunt_planner_select_highest_priority() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", None, 1.0);
        planner.add_error("E0599", "method not found", None, 2.0); // Higher severity

        let pattern = planner.select_next_target();
        assert!(pattern.is_some());
        // E0599 should be selected first (higher severity)
        assert_eq!(pattern.unwrap().error_code, "E0599");
    }

    #[test]
    fn test_hunt_planner_top_clusters() {
        let mut planner = HuntPlanner::new();
        for _ in 0..10 {
            planner.add_error("E0308", "mismatched types", None, 1.0);
        }
        for _ in 0..5 {
            planner.add_error("E0599", "method not found", None, 1.0);
        }

        let top = planner.top_clusters(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].code, "E0308");
    }

    #[test]
    fn test_hunt_planner_clear() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", None, 1.0);
        planner.clear();
        assert!(planner.clusters().is_empty());
    }

    #[test]
    fn test_hunt_planner_reset_processed() {
        let mut planner = HuntPlanner::new();
        planner.add_error("E0308", "mismatched types", None, 1.0);

        // Select and process
        let _ = planner.select_next_target();
        assert!(planner.select_next_target().is_none());

        // Reset
        planner.reset_processed();
        assert!(planner.select_next_target().is_some());
    }
}
