//! Five Whys Analyzer: Root Cause Analysis
//!
//! Implements Toyota's Five Whys technique for root cause analysis.
//! Each compilation failure triggers analysis to identify the true root cause.
//!
//! # Toyota Way: Root Cause Analysis
//!
//! "Ask 'why' five times about every matter." - Taiichi Ohno
//!
//! # References
//! - [20] Ohno (1988). "Ask 'why' five times about every matter." TPS.

use super::isolator::ReproCase;

/// A single "Why" in the chain
#[derive(Debug, Clone)]
pub struct Why {
    /// Depth (1-5)
    pub depth: u8,

    /// Question asked
    pub question: String,

    /// Answer/observation
    pub description: String,

    /// Whether this is the root cause
    pub is_root_cause: bool,

    /// Deeper cause (if not root)
    pub deeper_cause: Option<String>,

    /// Preventive measure (if root cause)
    pub preventive_measure: Option<String>,
}

impl Why {
    /// Create new Why
    #[must_use]
    pub fn new(depth: u8, description: impl Into<String>) -> Self {
        Self {
            depth,
            question: format!("Why #{depth}?"),
            description: description.into(),
            is_root_cause: false,
            deeper_cause: None,
            preventive_measure: None,
        }
    }

    /// Mark as root cause
    pub fn as_root_cause(mut self, measure: impl Into<String>) -> Self {
        self.is_root_cause = true;
        self.preventive_measure = Some(measure.into());
        self
    }

    /// Set deeper cause
    pub fn with_deeper_cause(mut self, cause: impl Into<String>) -> Self {
        self.deeper_cause = Some(cause.into());
        self
    }
}

/// Root cause identified by Five Whys
#[derive(Debug, Clone)]
pub struct RootCause {
    /// Root cause description
    pub description: String,

    /// Preventive measure
    pub preventive_measure: String,

    /// Affected component (e.g., "parser", "`type_inference`")
    pub component: String,

    /// Estimated fix complexity (1-10)
    pub complexity: u8,
}

impl RootCause {
    /// Create new root cause
    #[must_use]
    pub fn new(description: impl Into<String>, measure: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            preventive_measure: measure.into(),
            component: String::new(),
            complexity: 5,
        }
    }

    /// Set component
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = component.into();
        self
    }

    /// Set complexity
    pub fn with_complexity(mut self, complexity: u8) -> Self {
        self.complexity = complexity;
        self
    }
}

/// Chain of Why questions leading to root cause
#[derive(Debug, Clone)]
pub struct RootCauseChain {
    /// The Why chain
    pub whys: Vec<Why>,

    /// Final root cause
    pub root_cause: Option<RootCause>,
}

impl RootCauseChain {
    /// Create empty chain
    #[must_use]
    pub fn new() -> Self {
        Self {
            whys: Vec::new(),
            root_cause: None,
        }
    }

    /// Add a Why to the chain
    pub fn add_why(&mut self, why: Why) {
        if why.is_root_cause {
            self.root_cause = Some(RootCause::new(
                &why.description,
                why.preventive_measure.as_deref().unwrap_or(""),
            ));
        }
        self.whys.push(why);
    }

    /// Get depth reached
    #[must_use]
    pub fn depth(&self) -> usize {
        self.whys.len()
    }

    /// Check if root cause was found
    #[must_use]
    pub fn found_root_cause(&self) -> bool {
        self.root_cause.is_some()
    }

    /// Get summary
    #[must_use]
    pub fn summary(&self) -> String {
        if let Some(ref root) = self.root_cause {
            format!(
                "Root cause: {} (Fix: {})",
                root.description, root.preventive_measure
            )
        } else {
            format!("Analysis incomplete after {} whys", self.whys.len())
        }
    }
}

impl Default for RootCauseChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Five Whys Analyzer
#[derive(Debug)]
pub struct FiveWhysAnalyzer {
    /// Maximum depth (typically 5)
    max_depth: u8,

    /// Error pattern database for analysis
    error_patterns: Vec<ErrorPattern>,
}

/// Pattern for error analysis
#[derive(Debug, Clone)]
struct ErrorPattern {
    /// Error code
    code: String,

    /// Known root causes
    root_causes: Vec<(String, String, String)>, // (why_chain, root_cause, fix)
}

impl Default for FiveWhysAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FiveWhysAnalyzer {
    /// Create new analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_depth: 5,
            error_patterns: Self::init_patterns(),
        }
    }

    /// Initialize known error patterns
    fn init_patterns() -> Vec<ErrorPattern> {
        vec![
            ErrorPattern {
                code: "E0308".to_string(),
                root_causes: vec![(
                    "Type mismatch in return".to_string(),
                    "Return type inference not checking context".to_string(),
                    "Update type_inference.rs to check return type".to_string(),
                )],
            },
            ErrorPattern {
                code: "E0599".to_string(),
                root_causes: vec![(
                    "Method not found on type".to_string(),
                    "Method mapping missing for type".to_string(),
                    "Add method mapping to method_resolver.rs".to_string(),
                )],
            },
            ErrorPattern {
                code: "E0432".to_string(),
                root_causes: vec![(
                    "Unresolved import".to_string(),
                    "Missing dependency or incorrect module path".to_string(),
                    "Add dependency to Cargo.toml or fix import path".to_string(),
                )],
            },
        ]
    }

    /// Analyze reproduction case using Five Whys
    #[must_use]
    pub fn analyze(&self, repro: &ReproCase) -> RootCauseChain {
        let mut chain = RootCauseChain::new();
        let error_code = &repro.expected_error;

        // Find matching pattern
        let pattern = self.error_patterns.iter().find(|p| p.code == *error_code);

        // Generate Whys based on pattern
        if let Some(pattern) = pattern {
            if let Some((why_chain, root_cause, fix)) = pattern.root_causes.first() {
                // Why 1: What error occurred?
                chain.add_why(
                    Why::new(1, format!("Error {error_code} occurred"))
                        .with_deeper_cause(why_chain),
                );

                // Why 2: Why did this error occur?
                chain.add_why(
                    Why::new(2, why_chain.clone()).with_deeper_cause("Code generation issue"),
                );

                // Why 3: Why is there a code generation issue?
                chain.add_why(
                    Why::new(3, "Code generation doesn't handle this case")
                        .with_deeper_cause("Missing pattern in transpiler"),
                );

                // Why 4: Why is the pattern missing?
                chain.add_why(Why::new(4, "Pattern not implemented").with_deeper_cause(root_cause));

                // Why 5: Root cause
                chain.add_why(Why::new(5, root_cause.clone()).as_root_cause(fix));
            }
        } else {
            // Generic analysis
            chain.add_why(
                Why::new(1, format!("Error {error_code} occurred"))
                    .with_deeper_cause("Unknown error pattern"),
            );

            chain.add_why(
                Why::new(2, "Unknown error pattern")
                    .as_root_cause("Investigate and add pattern to analyzer"),
            );
        }

        chain
    }

    /// Set maximum depth
    #[must_use]
    pub fn with_max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Get maximum depth
    #[must_use]
    pub fn max_depth(&self) -> u8 {
        self.max_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: RED PHASE - Why Tests
    // ============================================================================

    #[test]
    fn test_why_new() {
        let why = Why::new(1, "Test description");
        assert_eq!(why.depth, 1);
        assert_eq!(why.description, "Test description");
        assert!(!why.is_root_cause);
    }

    #[test]
    fn test_why_question_format() {
        let why = Why::new(3, "Test");
        assert_eq!(why.question, "Why #3?");
    }

    #[test]
    fn test_why_as_root_cause() {
        let why = Why::new(5, "Root cause found").as_root_cause("Fix the bug");
        assert!(why.is_root_cause);
        assert_eq!(why.preventive_measure, Some("Fix the bug".to_string()));
    }

    #[test]
    fn test_why_with_deeper_cause() {
        let why = Why::new(1, "Surface issue").with_deeper_cause("Deeper problem");
        assert_eq!(why.deeper_cause, Some("Deeper problem".to_string()));
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - RootCause Tests
    // ============================================================================

    #[test]
    fn test_root_cause_new() {
        let cause = RootCause::new("Missing handler", "Add handler");
        assert_eq!(cause.description, "Missing handler");
        assert_eq!(cause.preventive_measure, "Add handler");
    }

    #[test]
    fn test_root_cause_with_component() {
        let cause = RootCause::new("Bug", "Fix").with_component("parser");
        assert_eq!(cause.component, "parser");
    }

    #[test]
    fn test_root_cause_with_complexity() {
        let cause = RootCause::new("Bug", "Fix").with_complexity(8);
        assert_eq!(cause.complexity, 8);
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - RootCauseChain Tests
    // ============================================================================

    #[test]
    fn test_root_cause_chain_new() {
        let chain = RootCauseChain::new();
        assert!(chain.whys.is_empty());
        assert!(chain.root_cause.is_none());
    }

    #[test]
    fn test_root_cause_chain_default() {
        let chain = RootCauseChain::default();
        assert!(chain.whys.is_empty());
    }

    #[test]
    fn test_root_cause_chain_add_why() {
        let mut chain = RootCauseChain::new();
        chain.add_why(Why::new(1, "First why"));
        assert_eq!(chain.depth(), 1);
    }

    #[test]
    fn test_root_cause_chain_add_root_why() {
        let mut chain = RootCauseChain::new();
        chain.add_why(Why::new(5, "Root cause").as_root_cause("Fix it"));
        assert!(chain.found_root_cause());
    }

    #[test]
    fn test_root_cause_chain_summary_with_root() {
        let mut chain = RootCauseChain::new();
        chain.add_why(Why::new(5, "Root cause").as_root_cause("Fix it"));
        let summary = chain.summary();
        assert!(summary.contains("Root cause"));
        assert!(summary.contains("Fix it"));
    }

    #[test]
    fn test_root_cause_chain_summary_incomplete() {
        let mut chain = RootCauseChain::new();
        chain.add_why(Why::new(1, "First why"));
        let summary = chain.summary();
        assert!(summary.contains("incomplete"));
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - FiveWhysAnalyzer Tests
    // ============================================================================

    #[test]
    fn test_five_whys_analyzer_new() {
        let analyzer = FiveWhysAnalyzer::new();
        assert_eq!(analyzer.max_depth(), 5);
    }

    #[test]
    fn test_five_whys_analyzer_default() {
        let analyzer = FiveWhysAnalyzer::default();
        assert_eq!(analyzer.max_depth(), 5);
    }

    #[test]
    fn test_five_whys_analyzer_with_max_depth() {
        let analyzer = FiveWhysAnalyzer::new().with_max_depth(7);
        assert_eq!(analyzer.max_depth(), 7);
    }

    #[test]
    fn test_five_whys_analyzer_analyze_e0308() {
        let analyzer = FiveWhysAnalyzer::new();
        let repro = ReproCase::new("fn test() -> i32 { \"hello\" }", "E0308");
        let chain = analyzer.analyze(&repro);

        assert!(chain.depth() > 0);
        assert!(chain.found_root_cause());
    }

    #[test]
    fn test_five_whys_analyzer_analyze_e0599() {
        let analyzer = FiveWhysAnalyzer::new();
        let repro = ReproCase::new("x.nonexistent()", "E0599");
        let chain = analyzer.analyze(&repro);

        assert!(chain.depth() > 0);
        assert!(chain.found_root_cause());
    }

    #[test]
    fn test_five_whys_analyzer_analyze_unknown() {
        let analyzer = FiveWhysAnalyzer::new();
        let repro = ReproCase::new("unknown code", "E9999");
        let chain = analyzer.analyze(&repro);

        // Should still produce analysis
        assert!(chain.depth() > 0);
    }

    #[test]
    fn test_five_whys_analyzer_chain_has_whys() {
        let analyzer = FiveWhysAnalyzer::new();
        let repro = ReproCase::new("test", "E0308");
        let chain = analyzer.analyze(&repro);

        // Should have multiple whys
        assert!(chain.whys.len() >= 2);
    }
}
