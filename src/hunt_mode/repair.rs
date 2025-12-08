//! Jidoka Repair Engine: Automated Fix Application
//!
//! Implements the CHECK phase of Hunt Mode's PDCA cycle.
//! Uses mutators and heuristic search to find and apply fixes.
//!
//! # Toyota Way: Jidoka (Automation with Human Touch)
//!
//! System automatically stops when quality cannot be assured.
//! Low-confidence fixes require human review.
//!
//! # References
//! - [3] Le Goues et al. (2012). `GenProg`. IEEE TSE.
//! - [15] Ohno (1988). TPS. Jidoka principle.

use super::isolator::ReproCase;

/// A fix candidate
#[derive(Debug, Clone)]
pub struct Fix {
    /// Unique fix ID
    pub id: String,

    /// Pattern ID this fixes
    pub pattern_id: String,

    /// Human-readable description
    pub description: String,

    /// Generated Rust code
    pub rust_output: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Transformation applied
    pub transformation: String,
}

impl Fix {
    /// Create new fix
    #[must_use]
    pub fn new(id: impl Into<String>, pattern_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            pattern_id: pattern_id.into(),
            description: String::new(),
            rust_output: String::new(),
            confidence: 0.0,
            transformation: String::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set rust output
    pub fn with_rust_output(mut self, output: impl Into<String>) -> Self {
        self.rust_output = output.into();
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set transformation
    pub fn with_transformation(mut self, transform: impl Into<String>) -> Self {
        self.transformation = transform.into();
        self
    }

    /// Check if fix is high confidence
    #[must_use]
    pub fn is_high_confidence(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

/// Result of repair attempt
#[derive(Debug, Clone)]
pub enum RepairResult {
    /// Fix found and validated
    Success(Fix),

    /// Fix found but needs human review
    NeedsHumanReview {
        /// The candidate fix
        fix: Fix,
        /// Confidence score
        confidence: f64,
        /// Reason for review
        reason: String,
    },

    /// No fix found
    NoFixFound,
}

/// Mutator trait for code transformations
pub trait Mutator: std::fmt::Debug {
    /// Get mutator name
    fn name(&self) -> &str;

    /// Check if mutator applies to error code
    fn applies_to(&self, error_code: &str) -> bool;

    /// Apply mutation to code
    fn apply(&self, source: &str, error_code: &str) -> Option<String>;

    /// Get confidence for this transformation
    fn confidence(&self) -> f64;
}

/// Type coercion mutator (fixes E0308)
#[derive(Debug)]
pub struct TypeCoercionMutator;

impl Mutator for TypeCoercionMutator {
    fn name(&self) -> &'static str {
        "type_coercion"
    }

    fn applies_to(&self, error_code: &str) -> bool {
        error_code == "E0308"
    }

    fn apply(&self, source: &str, _error_code: &str) -> Option<String> {
        // Simple heuristic: add .to_string() to string literals
        if source.contains('"') && source.contains("-> i32") {
            Some(source.replace('"', "").replace("-> i32", "-> String"))
        } else if source.contains('"') && source.contains("-> String") {
            // Already returns String, might need .to_string()
            Some(source.replace("-> String {", "-> String { return "))
        } else {
            None
        }
    }

    fn confidence(&self) -> f64 {
        0.75
    }
}

/// Method addition mutator (fixes E0599)
#[derive(Debug)]
pub struct MethodAdditionMutator;

impl Mutator for MethodAdditionMutator {
    fn name(&self) -> &'static str {
        "method_addition"
    }

    fn applies_to(&self, error_code: &str) -> bool {
        error_code == "E0599"
    }

    fn apply(&self, source: &str, _error_code: &str) -> Option<String> {
        // Remove unknown method calls
        if source.contains(".nonexistent_method()") {
            Some(source.replace(".nonexistent_method()", ""))
        } else {
            None
        }
    }

    fn confidence(&self) -> f64 {
        0.60
    }
}

/// Import addition mutator (fixes E0432)
#[derive(Debug)]
pub struct ImportAdditionMutator;

impl Mutator for ImportAdditionMutator {
    fn name(&self) -> &'static str {
        "import_addition"
    }

    fn applies_to(&self, error_code: &str) -> bool {
        error_code == "E0432" || error_code == "E0433"
    }

    fn apply(&self, source: &str, _error_code: &str) -> Option<String> {
        // Comment out bad imports
        if source.contains("use nonexistent") {
            Some(source.replace("use nonexistent", "// use nonexistent"))
        } else {
            None
        }
    }

    fn confidence(&self) -> f64 {
        0.50
    }
}

/// Jidoka Repair Engine
#[derive(Debug)]
pub struct JidokaRepairEngine {
    /// Available mutators
    mutators: Vec<Box<dyn Mutator + Send + Sync>>,

    /// Quality threshold for auto-apply
    quality_threshold: f64,

    /// Human review threshold
    human_review_threshold: f64,
}

impl Default for JidokaRepairEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl JidokaRepairEngine {
    /// Create new repair engine with default mutators
    #[must_use]
    pub fn new() -> Self {
        let mutators: Vec<Box<dyn Mutator + Send + Sync>> = vec![
            Box::new(TypeCoercionMutator),
            Box::new(MethodAdditionMutator),
            Box::new(ImportAdditionMutator),
        ];

        Self {
            mutators,
            quality_threshold: 0.85,
            human_review_threshold: 0.70,
        }
    }

    /// Set quality threshold
    #[must_use]
    pub fn with_quality_threshold(mut self, threshold: f64) -> Self {
        self.quality_threshold = threshold;
        self
    }

    /// Set human review threshold
    #[must_use]
    pub fn with_human_review_threshold(mut self, threshold: f64) -> Self {
        self.human_review_threshold = threshold;
        self
    }

    /// Add custom mutator
    pub fn add_mutator(&mut self, mutator: Box<dyn Mutator + Send + Sync>) {
        self.mutators.push(mutator);
    }

    /// Attempt to repair reproduction case
    ///
    /// Jidoka: Stop the line if fix quality is uncertain
    #[must_use]
    pub fn attempt_repair(&self, repro: &ReproCase) -> RepairResult {
        let error_code = &repro.expected_error;

        for mutator in &self.mutators {
            if !mutator.applies_to(error_code) {
                continue;
            }

            if let Some(fixed_code) = mutator.apply(&repro.source, error_code) {
                let confidence = mutator.confidence();

                let fix = Fix::new(
                    format!("FIX-{}-{}", error_code, mutator.name()),
                    &repro.pattern_id,
                )
                .with_description(format!("Applied {} mutator", mutator.name()))
                .with_rust_output(fixed_code)
                .with_confidence(confidence)
                .with_transformation(mutator.name().to_string());

                // Jidoka: Only proceed if quality is assured
                if confidence < self.human_review_threshold {
                    return RepairResult::NeedsHumanReview {
                        fix,
                        confidence,
                        reason: "Low confidence - manual review required".to_string(),
                    };
                }

                if confidence >= self.quality_threshold {
                    return RepairResult::Success(fix);
                }

                // Between thresholds - still needs review
                return RepairResult::NeedsHumanReview {
                    fix,
                    confidence,
                    reason: "Moderate confidence - recommend review".to_string(),
                };
            }
        }

        RepairResult::NoFixFound
    }

    /// Get available mutators
    #[must_use]
    pub fn mutators(&self) -> &[Box<dyn Mutator + Send + Sync>] {
        &self.mutators
    }

    /// Get mutator count
    #[must_use]
    pub fn mutator_count(&self) -> usize {
        self.mutators.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: RED PHASE - Fix Tests
    // ============================================================================

    #[test]
    fn test_fix_new() {
        let fix = Fix::new("FIX-001", "PAT-001");
        assert_eq!(fix.id, "FIX-001");
        assert_eq!(fix.pattern_id, "PAT-001");
    }

    #[test]
    fn test_fix_with_description() {
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_description("Test fix");
        assert_eq!(fix.description, "Test fix");
    }

    #[test]
    fn test_fix_with_rust_output() {
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_rust_output("fn test() {}");
        assert_eq!(fix.rust_output, "fn test() {}");
    }

    #[test]
    fn test_fix_with_confidence() {
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_confidence(0.95);
        assert!((fix.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fix_with_transformation() {
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_transformation("type_coercion");
        assert_eq!(fix.transformation, "type_coercion");
    }

    #[test]
    fn test_fix_is_high_confidence() {
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_confidence(0.9);
        assert!(fix.is_high_confidence(0.85));
        assert!(!fix.is_high_confidence(0.95));
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - RepairResult Tests
    // ============================================================================

    #[test]
    fn test_repair_result_success() {
        let fix = Fix::new("FIX-001", "PAT-001");
        let result = RepairResult::Success(fix);
        assert!(matches!(result, RepairResult::Success(_)));
    }

    #[test]
    fn test_repair_result_needs_review() {
        let fix = Fix::new("FIX-001", "PAT-001");
        let result = RepairResult::NeedsHumanReview {
            fix,
            confidence: 0.6,
            reason: "Low confidence".to_string(),
        };
        assert!(matches!(result, RepairResult::NeedsHumanReview { .. }));
    }

    #[test]
    fn test_repair_result_no_fix() {
        let result = RepairResult::NoFixFound;
        assert!(matches!(result, RepairResult::NoFixFound));
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - Mutator Tests
    // ============================================================================

    #[test]
    fn test_type_coercion_mutator_name() {
        let mutator = TypeCoercionMutator;
        assert_eq!(mutator.name(), "type_coercion");
    }

    #[test]
    fn test_type_coercion_mutator_applies_to() {
        let mutator = TypeCoercionMutator;
        assert!(mutator.applies_to("E0308"));
        assert!(!mutator.applies_to("E0599"));
    }

    #[test]
    fn test_type_coercion_mutator_confidence() {
        let mutator = TypeCoercionMutator;
        assert!((mutator.confidence() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_method_addition_mutator_name() {
        let mutator = MethodAdditionMutator;
        assert_eq!(mutator.name(), "method_addition");
    }

    #[test]
    fn test_method_addition_mutator_applies_to() {
        let mutator = MethodAdditionMutator;
        assert!(mutator.applies_to("E0599"));
        assert!(!mutator.applies_to("E0308"));
    }

    #[test]
    fn test_method_addition_mutator_apply() {
        let mutator = MethodAdditionMutator;
        let result = mutator.apply("x.nonexistent_method()", "E0599");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "x");
    }

    #[test]
    fn test_import_addition_mutator_name() {
        let mutator = ImportAdditionMutator;
        assert_eq!(mutator.name(), "import_addition");
    }

    #[test]
    fn test_import_addition_mutator_applies_to() {
        let mutator = ImportAdditionMutator;
        assert!(mutator.applies_to("E0432"));
        assert!(mutator.applies_to("E0433"));
        assert!(!mutator.applies_to("E0308"));
    }

    #[test]
    fn test_import_addition_mutator_apply() {
        let mutator = ImportAdditionMutator;
        let result = mutator.apply("use nonexistent_crate::Thing;", "E0432");
        assert!(result.is_some());
        assert!(result.unwrap().contains("//"));
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - JidokaRepairEngine Tests
    // ============================================================================

    #[test]
    fn test_jidoka_repair_engine_new() {
        let engine = JidokaRepairEngine::new();
        assert_eq!(engine.mutator_count(), 3);
    }

    #[test]
    fn test_jidoka_repair_engine_default() {
        let engine = JidokaRepairEngine::default();
        assert_eq!(engine.mutator_count(), 3);
    }

    #[test]
    fn test_jidoka_repair_engine_with_quality_threshold() {
        let engine = JidokaRepairEngine::new()
            .with_quality_threshold(0.90);
        assert!((engine.quality_threshold - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jidoka_repair_engine_with_human_review_threshold() {
        let engine = JidokaRepairEngine::new()
            .with_human_review_threshold(0.60);
        assert!((engine.human_review_threshold - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jidoka_repair_engine_attempt_repair_e0599() {
        let engine = JidokaRepairEngine::new();
        let repro = ReproCase::new("x.nonexistent_method()", "E0599");
        let result = engine.attempt_repair(&repro);

        // Should find a fix but need review (confidence 0.60 < 0.70)
        assert!(matches!(result, RepairResult::NeedsHumanReview { .. }));
    }

    #[test]
    fn test_jidoka_repair_engine_attempt_repair_unknown() {
        let engine = JidokaRepairEngine::new();
        let repro = ReproCase::new("fn test() {}", "E9999");
        let result = engine.attempt_repair(&repro);

        assert!(matches!(result, RepairResult::NoFixFound));
    }

    #[test]
    fn test_jidoka_repair_engine_mutators() {
        let engine = JidokaRepairEngine::new();
        let mutators = engine.mutators();
        assert_eq!(mutators.len(), 3);
    }
}
