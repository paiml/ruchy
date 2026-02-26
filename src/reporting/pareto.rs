//! Pareto Analysis (Error Clustering) [8]
//!
//! Implements Juran's Pareto Principle (vital few vs trivial many)
//! to prioritize error fixes for maximum impact.

use std::collections::HashMap;

/// Blocker priority level based on Pareto analysis [8]
///
/// Thresholds:
/// - P0-CRITICAL: >20% of corpus OR ≥50 occurrences
/// - P1-HIGH: >10% of corpus OR ≥20 occurrences
/// - P2-MEDIUM: >5% of corpus OR ≥10 occurrences
/// - P3-LOW: <5% of corpus AND <10 occurrences
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlockerPriority {
    /// Critical - blocks >20% of corpus
    P0Critical,
    /// High - blocks 10-20% of corpus
    P1High,
    /// Medium - blocks 5-10% of corpus
    P2Medium,
    /// Low - blocks <5% of corpus
    P3Low,
}

impl BlockerPriority {
    /// Calculate priority from count and total
    #[must_use]
    pub fn from_frequency(count: usize, total: usize) -> Self {
        if total == 0 {
            return Self::P3Low;
        }

        let percentage = (count as f64 / total as f64) * 100.0;

        if percentage > 20.0 || count >= 50 {
            Self::P0Critical
        } else if percentage > 10.0 || count >= 20 {
            Self::P1High
        } else if percentage > 5.0 || count >= 10 {
            Self::P2Medium
        } else {
            Self::P3Low
        }
    }

    /// Get display label
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::P0Critical => "P0-CRITICAL",
            Self::P1High => "P1-HIGH",
            Self::P2Medium => "P2-MEDIUM",
            Self::P3Low => "P3-LOW",
        }
    }

    /// Get short label
    #[must_use]
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::P0Critical => "P0",
            Self::P1High => "P1",
            Self::P2Medium => "P2",
            Self::P3Low => "P3",
        }
    }
}

impl std::fmt::Display for BlockerPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Error cluster for Pareto analysis
#[derive(Debug, Clone)]
pub struct ErrorCluster {
    /// Error code (e.g., "E0308")
    pub code: String,
    /// Number of occurrences
    pub count: usize,
    /// Percentage of total failures
    pub percentage: f64,
    /// Cumulative percentage (for Pareto chart)
    pub cumulative: f64,
    /// Blocker priority
    pub priority: BlockerPriority,
    /// Root cause description
    pub root_cause: String,
    /// Recommended fix action
    pub fix_action: String,
}

impl ErrorCluster {
    /// Create new error cluster
    #[must_use]
    pub fn new(code: impl Into<String>, count: usize, total: usize) -> Self {
        let code = code.into();
        let percentage = if total > 0 {
            (count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let priority = BlockerPriority::from_frequency(count, total);
        let (root_cause, fix_action) = Self::suggest_fix(&code);

        Self {
            code,
            count,
            percentage,
            cumulative: 0.0, // Set by ParetoAnalysis
            priority,
            root_cause,
            fix_action,
        }
    }

    /// Suggest fix based on error code
    fn suggest_fix(code: &str) -> (String, String) {
        match code {
            "E0308" => (
                "Type inference failure".to_string(),
                "Improve bidirectional type inference in transpiler".to_string(),
            ),
            "E0382" => (
                "Ownership violation".to_string(),
                "Add Clone derive or use Rc/Arc for shared ownership".to_string(),
            ),
            "E0412" => (
                "Generic parameter unresolved".to_string(),
                "Resolve generic type parameters from context".to_string(),
            ),
            "E0425" => (
                "Missing import/binding".to_string(),
                "Add missing imports or variable bindings".to_string(),
            ),
            "E0282" => (
                "Insufficient type info".to_string(),
                "Add explicit type annotations".to_string(),
            ),
            "E0277" => (
                "Missing trait implementation".to_string(),
                "Implement required traits or add trait bounds".to_string(),
            ),
            "E0502" | "E0503" | "E0505" => (
                "Borrow checker conflict".to_string(),
                "Restructure code to satisfy borrow checker".to_string(),
            ),
            "E0106" | "E0621" => (
                "Lifetime annotation needed".to_string(),
                "Add explicit lifetime annotations".to_string(),
            ),
            _ => (
                "Transpilation error".to_string(),
                "Investigate specific error pattern".to_string(),
            ),
        }
    }
}

/// Pareto analysis result
#[derive(Debug, Clone)]
pub struct ParetoAnalysis {
    /// Clusters sorted by count (descending)
    pub clusters: Vec<ErrorCluster>,
    /// Total error count
    pub total_errors: usize,
    /// Errors in vital few (causing 80% of failures)
    pub vital_few_count: usize,
    /// Percentage covered by vital few
    pub vital_few_coverage: f64,
}

impl ParetoAnalysis {
    /// Perform Pareto analysis on error counts
    #[must_use]
    pub fn analyze(error_counts: &HashMap<String, usize>) -> Self {
        let total_errors: usize = error_counts.values().sum();

        if total_errors == 0 {
            return Self {
                clusters: Vec::new(),
                total_errors: 0,
                vital_few_count: 0,
                vital_few_coverage: 0.0,
            };
        }

        // Create clusters and sort by count
        let mut clusters: Vec<ErrorCluster> = error_counts
            .iter()
            .map(|(code, &count)| ErrorCluster::new(code.clone(), count, total_errors))
            .collect();

        clusters.sort_by(|a, b| b.count.cmp(&a.count));

        // Calculate cumulative percentages
        let mut cumulative = 0.0;
        for cluster in &mut clusters {
            cumulative += cluster.percentage;
            cluster.cumulative = cumulative;
        }

        // Find vital few (covering 80%)
        let vital_few_count = clusters.iter().take_while(|c| c.cumulative <= 80.0).count() + 1; // Include the one that crosses 80%

        let clusters_len = clusters.len();
        let vital_few_count = vital_few_count.min(clusters_len);

        let vital_few_coverage = clusters
            .iter()
            .take(vital_few_count)
            .map(|c| c.percentage)
            .sum();

        Self {
            clusters,
            total_errors,
            vital_few_count,
            vital_few_coverage,
        }
    }

    /// Get clusters by priority level
    #[must_use]
    pub fn by_priority(&self, priority: BlockerPriority) -> Vec<&ErrorCluster> {
        self.clusters
            .iter()
            .filter(|c| c.priority == priority)
            .collect()
    }

    /// Get vital few clusters (Pareto principle)
    #[must_use]
    pub fn vital_few(&self) -> &[ErrorCluster] {
        &self.clusters[..self.vital_few_count.min(self.clusters.len())]
    }

    /// Check if analysis indicates critical issues
    #[must_use]
    pub fn has_critical(&self) -> bool {
        self.clusters
            .iter()
            .any(|c| c.priority == BlockerPriority::P0Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: RED PHASE - BlockerPriority Tests
    // ============================================================

    #[test]
    fn test_priority_p0_critical_by_percentage() {
        // >20% = P0
        assert_eq!(
            BlockerPriority::from_frequency(25, 100),
            BlockerPriority::P0Critical
        );
        assert_eq!(
            BlockerPriority::from_frequency(21, 100),
            BlockerPriority::P0Critical
        );
    }

    #[test]
    fn test_priority_p0_critical_by_count() {
        // ≥50 = P0 regardless of percentage
        assert_eq!(
            BlockerPriority::from_frequency(50, 1000),
            BlockerPriority::P0Critical
        );
        assert_eq!(
            BlockerPriority::from_frequency(100, 10000),
            BlockerPriority::P0Critical
        );
    }

    #[test]
    fn test_priority_p1_high() {
        // >10% or ≥20 = P1
        assert_eq!(
            BlockerPriority::from_frequency(15, 100),
            BlockerPriority::P1High
        );
        assert_eq!(
            BlockerPriority::from_frequency(20, 1000),
            BlockerPriority::P1High
        );
    }

    #[test]
    fn test_priority_p2_medium() {
        // >5% or ≥10 = P2
        assert_eq!(
            BlockerPriority::from_frequency(7, 100),
            BlockerPriority::P2Medium
        );
        assert_eq!(
            BlockerPriority::from_frequency(10, 1000),
            BlockerPriority::P2Medium
        );
    }

    #[test]
    fn test_priority_p3_low() {
        // <5% and <10 = P3
        assert_eq!(
            BlockerPriority::from_frequency(3, 100),
            BlockerPriority::P3Low
        );
        assert_eq!(
            BlockerPriority::from_frequency(5, 1000),
            BlockerPriority::P3Low
        );
    }

    #[test]
    fn test_priority_zero_total() {
        assert_eq!(
            BlockerPriority::from_frequency(0, 0),
            BlockerPriority::P3Low
        );
    }

    #[test]
    fn test_priority_labels() {
        assert_eq!(BlockerPriority::P0Critical.label(), "P0-CRITICAL");
        assert_eq!(BlockerPriority::P1High.label(), "P1-HIGH");
        assert_eq!(BlockerPriority::P2Medium.label(), "P2-MEDIUM");
        assert_eq!(BlockerPriority::P3Low.label(), "P3-LOW");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(BlockerPriority::P0Critical < BlockerPriority::P1High);
        assert!(BlockerPriority::P1High < BlockerPriority::P2Medium);
        assert!(BlockerPriority::P2Medium < BlockerPriority::P3Low);
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - ErrorCluster Tests
    // ============================================================

    #[test]
    fn test_error_cluster_new() {
        let cluster = ErrorCluster::new("E0308", 25, 100);
        assert_eq!(cluster.code, "E0308");
        assert_eq!(cluster.count, 25);
        assert!((cluster.percentage - 25.0).abs() < 0.01);
        assert_eq!(cluster.priority, BlockerPriority::P0Critical);
    }

    #[test]
    fn test_error_cluster_fix_suggestions() {
        let cluster = ErrorCluster::new("E0308", 10, 100);
        assert!(cluster.root_cause.contains("Type inference"));

        let cluster = ErrorCluster::new("E0382", 10, 100);
        assert!(cluster.root_cause.contains("Ownership"));

        let cluster = ErrorCluster::new("E9999", 10, 100);
        assert!(cluster.root_cause.contains("Transpilation"));
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - ParetoAnalysis Tests
    // ============================================================

    #[test]
    fn test_pareto_empty() {
        let counts: HashMap<String, usize> = HashMap::new();
        let analysis = ParetoAnalysis::analyze(&counts);

        assert_eq!(analysis.total_errors, 0);
        assert!(analysis.clusters.is_empty());
    }

    #[test]
    fn test_pareto_single_error() {
        let mut counts = HashMap::new();
        counts.insert("E0308".to_string(), 100);

        let analysis = ParetoAnalysis::analyze(&counts);

        assert_eq!(analysis.total_errors, 100);
        assert_eq!(analysis.clusters.len(), 1);
        assert!((analysis.clusters[0].percentage - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_pareto_sorting() {
        let mut counts = HashMap::new();
        counts.insert("E0308".to_string(), 50);
        counts.insert("E0382".to_string(), 30);
        counts.insert("E0425".to_string(), 20);

        let analysis = ParetoAnalysis::analyze(&counts);

        assert_eq!(analysis.clusters[0].code, "E0308");
        assert_eq!(analysis.clusters[1].code, "E0382");
        assert_eq!(analysis.clusters[2].code, "E0425");
    }

    #[test]
    fn test_pareto_cumulative() {
        let mut counts = HashMap::new();
        counts.insert("E0308".to_string(), 50);
        counts.insert("E0382".to_string(), 30);
        counts.insert("E0425".to_string(), 20);

        let analysis = ParetoAnalysis::analyze(&counts);

        assert!((analysis.clusters[0].cumulative - 50.0).abs() < 0.01);
        assert!((analysis.clusters[1].cumulative - 80.0).abs() < 0.01);
        assert!((analysis.clusters[2].cumulative - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_pareto_vital_few() {
        let mut counts = HashMap::new();
        counts.insert("E0308".to_string(), 60); // 60%
        counts.insert("E0382".to_string(), 25); // 25%
        counts.insert("E0425".to_string(), 10); // 10%
        counts.insert("E0412".to_string(), 5); // 5%

        let analysis = ParetoAnalysis::analyze(&counts);

        // First two cover 85% (60% + 25%), so vital_few = 2
        assert_eq!(analysis.vital_few_count, 2);
        assert!((analysis.vital_few_coverage - 85.0).abs() < 0.01);
    }

    #[test]
    fn test_pareto_by_priority() {
        // Create counts that result in specific priorities
        // Total = 200, so percentages are: 50/200=25%, 30/200=15%, 6/200=3%
        let mut counts = HashMap::new();
        counts.insert("E0308".to_string(), 50); // 25% > 20% = P0
        counts.insert("E0382".to_string(), 30); // 15% > 10% = P1
        counts.insert("E0425".to_string(), 6); // 3% < 5% and < 10 = P3

        // Add more to increase total for clearer percentages
        counts.insert("E0412".to_string(), 114); // filler to make total = 200

        let analysis = ParetoAnalysis::analyze(&counts);

        let p0 = analysis.by_priority(BlockerPriority::P0Critical);
        let p1 = analysis.by_priority(BlockerPriority::P1High);
        let p3 = analysis.by_priority(BlockerPriority::P3Low);

        // E0412 is 114/200 = 57% = P0, E0308 is 25% = P0
        assert_eq!(p0.len(), 2);
        // E0382 is 15% = P1
        assert_eq!(p1.len(), 1);
        // E0425 is 3% = P3
        assert_eq!(p3.len(), 1);
    }

    #[test]
    fn test_pareto_has_critical() {
        // Single error with count 50 = P0 (≥50 occurrences rule)
        let mut counts = HashMap::new();
        counts.insert("E0308".to_string(), 50);

        let analysis = ParetoAnalysis::analyze(&counts);
        assert!(analysis.has_critical());

        // Multiple small errors: 3 and 3 out of 100 total (add filler)
        // Each is 3% < 5% and < 10 count, so P3 (not critical)
        let mut counts2 = HashMap::new();
        counts2.insert("E0308".to_string(), 3);
        counts2.insert("E0382".to_string(), 3);
        // filler: 94% but only if > 20 count...
        // Actually 94/100 = 94% which is > 20%, so it's P0
        counts2.insert("E0999".to_string(), 94);

        // Let's use counts where no error is critical
        let mut counts3 = HashMap::new();
        counts3.insert("E0308".to_string(), 3); // 3% and <10 = P3
        counts3.insert("E0382".to_string(), 3); // 3% and <10 = P3
        counts3.insert("E0425".to_string(), 4); // 4% and <10 = P3
        // Total = 10, each is 30-40% but all <10 occurrences
        // 3/10 = 30% > 20% = P0! Need larger total

        // Use 100 total with small individual counts
        let mut counts4 = HashMap::new();
        for i in 0..20 {
            // Each error has count 5 = 5/100 = 5%, and count 5 < 10
            // 5% is NOT >5%, it's equal to 5%, so P2 Medium (>5% threshold)
            // Actually, looking at from_frequency: >5% means strictly greater
            counts4.insert(format!("E0{i:03}"), 5);
        }
        // Each is 5%, not >5%, so P3
        // But actually 5% equals the threshold boundary. Let me check the logic again.
        // if percentage > 5.0 || count >= 10 => P2
        // 5.0 is NOT > 5.0, and 5 is NOT >= 10, so P3

        let analysis4 = ParetoAnalysis::analyze(&counts4);
        assert!(!analysis4.has_critical());
    }
}
