//! EXTREME TDD: Quality Gates Configuration Tests (ACTOR-002)
//!
//! Test-first development for quality enforcement infrastructure.
//! NO IMPLEMENTATION YET - Tests define the quality requirements.
//! Target: 95% minimum coverage, 100% for critical paths.

use std::collections::HashMap;
use std::time::Duration;

#[cfg(test)]
mod quality_gates_tests {
    use super::*;

    // =============================================================================
    // COVERAGE ENFORCEMENT TESTS
    // =============================================================================

    #[test]
    #[ignore = "Coverage enforcement needs implementation"]
    fn test_overall_coverage_requirement() {
        let coverage_config = CoverageConfig::new();

        assert_eq!(
            coverage_config.overall_minimum(),
            95.0,
            "Overall coverage must be 95% minimum"
        );
        assert_eq!(
            coverage_config.new_code_minimum(),
            100.0,
            "New code coverage must be 100%"
        );
    }

    #[test]
    #[ignore = "Coverage enforcement needs implementation"]
    fn test_per_module_coverage_requirements() {
        let coverage_config = CoverageConfig::new();
        let requirements = coverage_config.per_module_requirements();

        // Critical path modules require 100%
        assert_eq!(requirements.get("src/actors/parser.rs"), Some(&100.0));
        assert_eq!(requirements.get("src/actors/typechecker.rs"), Some(&100.0));
        assert_eq!(requirements.get("src/actors/transpiler.rs"), Some(&100.0));

        // Runtime allows slight tolerance for edge cases
        assert_eq!(requirements.get("src/actors/supervision.rs"), Some(&95.0));
        assert_eq!(requirements.get("src/actors/runtime.rs"), Some(&95.0));
    }

    #[test]
    #[ignore = "Coverage enforcement needs implementation"]
    fn test_coverage_validation_passes_good_coverage() {
        let mut mock_coverage = HashMap::new();
        mock_coverage.insert("src/actors/parser.rs".to_string(), 100.0);
        mock_coverage.insert("src/actors/typechecker.rs".to_string(), 100.0);
        mock_coverage.insert("src/actors/transpiler.rs".to_string(), 100.0);
        mock_coverage.insert("src/actors/supervision.rs".to_string(), 97.0);

        let validator = CoverageValidator::new();
        let result = validator.validate(&mock_coverage);

        assert!(result.is_ok(), "Should pass with good coverage");
        assert!(
            result.unwrap().violations.is_empty(),
            "Should have no violations"
        );
    }

    #[test]
    #[ignore = "Coverage enforcement needs implementation"]
    fn test_coverage_validation_fails_bad_coverage() {
        let mut mock_coverage = HashMap::new();
        mock_coverage.insert("src/actors/parser.rs".to_string(), 85.0); // Below 100%
        mock_coverage.insert("src/actors/supervision.rs".to_string(), 90.0); // Below 95%

        let validator = CoverageValidator::new();
        let result = validator.validate(&mock_coverage);

        assert!(result.is_err(), "Should fail with bad coverage");
        let violations = result.unwrap_err().violations;
        assert_eq!(violations.len(), 2, "Should have 2 violations");
    }

    // =============================================================================
    // COMPLEXITY LIMITS TESTS
    // =============================================================================

    #[test]
    #[ignore = "Complexity enforcement needs implementation"]
    fn test_complexity_limits_configuration() {
        let complexity_config = ComplexityConfig::new();

        assert_eq!(
            complexity_config.max_cyclomatic(),
            5,
            "Cyclomatic complexity must be ≤5"
        );
        assert_eq!(
            complexity_config.max_cognitive(),
            8,
            "Cognitive complexity must be ≤8"
        );
        assert_eq!(
            complexity_config.max_nesting(),
            3,
            "Nesting depth must be ≤3"
        );
    }

    #[test]
    #[ignore = "Complexity enforcement needs implementation"]
    fn test_complexity_validation_passes_good_code() {
        let function = MockFunction {
            name: "simple_function".to_string(),
            cyclomatic: 3,
            cognitive: 5,
            nesting: 2,
        };

        let validator = ComplexityValidator::new();
        let result = validator.validate_function(&function);

        assert!(result.is_ok(), "Should pass with good complexity");
    }

    #[test]
    #[ignore = "Complexity enforcement needs implementation"]
    fn test_complexity_validation_fails_high_complexity() {
        let function = MockFunction {
            name: "complex_function".to_string(),
            cyclomatic: 10, // Above limit of 5
            cognitive: 15,  // Above limit of 8
            nesting: 5,     // Above limit of 3
        };

        let validator = ComplexityValidator::new();
        let result = validator.validate_function(&function);

        assert!(result.is_err(), "Should fail with high complexity");
        let violations = result.unwrap_err();
        assert_eq!(violations.len(), 3, "Should have 3 complexity violations");
    }

    // =============================================================================
    // MUTATION TESTING REQUIREMENTS TESTS
    // =============================================================================

    #[test]
    #[ignore = "Mutation testing needs implementation"]
    fn test_mutation_score_requirements() {
        let mutation_config = MutationConfig::new();

        assert_eq!(
            mutation_config.minimum_kill_rate(),
            0.95,
            "Mutation kill rate must be ≥95%"
        );
        assert_eq!(
            mutation_config.timeout(),
            Duration::from_secs(30),
            "Mutation test timeout should be 30s"
        );
    }

    #[test]
    #[ignore = "Mutation testing needs implementation"]
    fn test_critical_mutation_detection() {
        let mutation_config = MutationConfig::new();
        let critical_mutations = mutation_config.critical_mutations();

        let expected_critical = vec![
            "operator_replacement: ! to ?", // Send vs Ask
            "operator_replacement: ? to !", // Ask vs Send
            "constant_replacement: 0 to 1", // Off-by-one errors
            "condition_flip: < to >=",      // Boundary conditions
            "return_value: Ok to Err",      // Error handling
            "timeout_removal",              // Async timeouts
        ];

        for critical in expected_critical {
            assert!(
                critical_mutations.contains(&critical.to_string()),
                "Should detect critical mutation: {}",
                critical
            );
        }
    }

    #[test]
    #[ignore = "Mutation testing needs implementation"]
    fn test_mutation_validation_passes_good_kill_rate() {
        let mock_report = MutationReport {
            total_mutants: 100,
            killed_mutants: 96,
            survived_mutants: 4,
            timeout_mutants: 0,
            kill_rate: 0.96,
        };

        let validator = MutationValidator::new();
        let result = validator.validate(&mock_report);

        assert!(result.is_ok(), "Should pass with 96% kill rate");
    }

    #[test]
    #[ignore = "Mutation testing needs implementation"]
    fn test_mutation_validation_fails_low_kill_rate() {
        let mock_report = MutationReport {
            total_mutants: 100,
            killed_mutants: 90,
            survived_mutants: 10,
            timeout_mutants: 0,
            kill_rate: 0.90,
        };

        let validator = MutationValidator::new();
        let result = validator.validate(&mock_report);

        assert!(result.is_err(), "Should fail with 90% kill rate");
        assert!(result.unwrap_err().contains("kill rate below 95%"));
    }

    // =============================================================================
    // PERFORMANCE REQUIREMENTS TESTS
    // =============================================================================

    #[test]
    #[ignore = "Performance requirements need implementation"]
    fn test_performance_benchmarks_configuration() {
        let perf_config = PerformanceConfig::new();

        assert_eq!(perf_config.actor_spawn_p99(), Duration::from_micros(100));
        assert_eq!(perf_config.message_send_p99(), Duration::from_nanos(1000));
        assert_eq!(perf_config.ask_latency_p99(), Duration::from_micros(10));
        assert_eq!(
            perf_config.supervision_restart_p99(),
            Duration::from_micros(500)
        );
    }

    #[test]
    #[ignore = "Performance requirements need implementation"]
    fn test_performance_validation_passes_good_metrics() {
        let metrics = PerformanceMetrics {
            actor_spawn_p99: Duration::from_micros(50),
            message_send_p99: Duration::from_nanos(500),
            ask_latency_p99: Duration::from_micros(5),
            supervision_restart_p99: Duration::from_micros(200),
        };

        let validator = PerformanceValidator::new();
        let result = validator.validate(&metrics);

        assert!(result.is_ok(), "Should pass with good performance metrics");
    }

    #[test]
    #[ignore = "Performance requirements need implementation"]
    fn test_performance_validation_fails_slow_operations() {
        let metrics = PerformanceMetrics {
            actor_spawn_p99: Duration::from_millis(1),   // Too slow
            message_send_p99: Duration::from_micros(10), // Too slow
            ask_latency_p99: Duration::from_millis(1),   // Too slow
            supervision_restart_p99: Duration::from_millis(2), // Too slow
        };

        let validator = PerformanceValidator::new();
        let result = validator.validate(&metrics);

        assert!(result.is_err(), "Should fail with slow metrics");
        let violations = result.unwrap_err().violations;
        assert_eq!(violations.len(), 4, "Should have 4 performance violations");
    }

    // =============================================================================
    // TEST-TO-CODE RATIO TESTS
    // =============================================================================

    #[test]
    #[ignore = "Test ratio enforcement needs implementation"]
    fn test_test_to_code_ratio_requirement() {
        let ratio_config = TestRatioConfig::new();

        assert_eq!(
            ratio_config.minimum_ratio(),
            3.0,
            "Must have 3:1 test-to-code ratio minimum"
        );
        assert!(
            ratio_config.enforce_on_new_code(),
            "Must enforce ratio on all new code"
        );
    }

    #[test]
    #[ignore = "Test ratio enforcement needs implementation"]
    fn test_test_ratio_validation_passes_good_ratio() {
        let stats = CodeStats {
            source_lines: 1000,
            test_lines: 3500, // 3.5:1 ratio
        };

        let validator = TestRatioValidator::new();
        let result = validator.validate(&stats);

        assert!(result.is_ok(), "Should pass with 3.5:1 ratio");
    }

    #[test]
    #[ignore = "Test ratio enforcement needs implementation"]
    fn test_test_ratio_validation_fails_low_ratio() {
        let stats = CodeStats {
            source_lines: 1000,
            test_lines: 2000, // 2:1 ratio (below 3:1 requirement)
        };

        let validator = TestRatioValidator::new();
        let result = validator.validate(&stats);

        assert!(result.is_err(), "Should fail with 2:1 ratio");
        assert!(result.unwrap_err().contains("below 3:1 requirement"));
    }

    // =============================================================================
    // PROPERTY TEST REQUIREMENTS TESTS
    // =============================================================================

    #[test]
    #[ignore = "Property test requirements need implementation"]
    fn test_property_test_detection() {
        let test_analyzer = PropertyTestAnalyzer::new();

        let test_with_properties = r#"
            #[test]
            fn regular_test() { assert_eq!(1, 1); }

            proptest! {
                #[test]
                fn prop_test(x in 0..100) {
                    assert!(x >= 0);
                }
            }
        "#;

        let analysis = test_analyzer.analyze(test_with_properties);

        assert_eq!(analysis.regular_tests, 1);
        assert_eq!(analysis.property_tests, 1);
        assert!(
            analysis.has_required_properties(),
            "Should have required property tests"
        );
    }

    #[test]
    #[ignore = "Property test requirements need implementation"]
    fn test_property_test_validation_fails_missing_properties() {
        let test_analyzer = PropertyTestAnalyzer::new();

        let test_without_properties = r#"
            #[test]
            fn test1() { assert_eq!(1, 1); }

            #[test]
            fn test2() { assert_eq!(2, 2); }
        "#;

        let analysis = test_analyzer.analyze(test_without_properties);

        assert_eq!(analysis.regular_tests, 2);
        assert_eq!(analysis.property_tests, 0);
        assert!(
            !analysis.has_required_properties(),
            "Should fail without property tests"
        );
    }

    // =============================================================================
    // DOCUMENTATION REQUIREMENTS TESTS
    // =============================================================================

    #[test]
    #[ignore = "Documentation requirements need implementation"]
    fn test_documentation_coverage_requirements() {
        let doc_config = DocumentationConfig::new();

        assert_eq!(
            doc_config.public_items_coverage(),
            100.0,
            "All public items must be documented"
        );
        assert!(
            doc_config.examples_required(),
            "Documentation must include examples"
        );
        assert!(
            doc_config.doctests_required(),
            "Documentation must include doctests"
        );
    }

    #[test]
    #[ignore = "Documentation requirements need implementation"]
    fn test_documentation_validation_passes_good_docs() {
        let code_with_docs = r#"
            /// Counter actor maintains a mutable count.
            ///
            /// # Example
            /// ```
            /// let counter = spawn Counter { value: 0 };
            /// counter ! increment();
            /// let value = counter ? get_value();
            /// assert_eq!(value, 1);
            /// ```
            actor Counter {
                /// Current count value
                value: i32,

                /// Increment the counter by 1.
                /// Never fails.
                receive increment() {
                    self.value += 1
                }
            }
        "#;

        let validator = DocumentationValidator::new();
        let result = validator.validate(code_with_docs);

        assert!(result.is_ok(), "Should pass with good documentation");
    }

    #[test]
    #[ignore = "Documentation requirements need implementation"]
    fn test_documentation_validation_fails_missing_docs() {
        let code_without_docs = r#"
            actor Counter {
                value: i32,
                receive increment() {
                    self.value += 1
                }
            }
        "#;

        let validator = DocumentationValidator::new();
        let result = validator.validate(code_without_docs);

        assert!(result.is_err(), "Should fail without documentation");
        let violations = result.unwrap_err().violations;
        assert!(
            !violations.is_empty(),
            "Should have documentation violations"
        );
    }

    // =============================================================================
    // CONTINUOUS MONITORING TESTS
    // =============================================================================

    #[test]
    #[ignore = "Quality monitoring needs implementation"]
    fn test_quality_metrics_collection() {
        let collector = QualityMetricsCollector::new();
        let metrics = collector.collect_from_path(".");

        assert!(metrics.coverage >= 0.0 && metrics.coverage <= 100.0);
        assert!(metrics.mutation_score >= 0.0 && metrics.mutation_score <= 1.0);
        assert!(metrics.test_ratio >= 0.0);
        assert!(metrics.max_complexity >= 0);
        assert!(metrics.property_test_count >= 0);
        assert!(metrics.benchmark_count >= 0);
    }

    #[test]
    #[ignore = "Quality monitoring needs implementation"]
    fn test_quality_gate_checking() {
        let metrics = QualityMetrics {
            coverage: 97.5,
            mutation_score: 0.96,
            test_ratio: 3.2,
            max_complexity: 4,
            property_test_count: 15,
            benchmark_count: 8,
            doc_coverage: 100.0,
        };

        let gate_checker = QualityGateChecker::new();
        let violations = gate_checker.check_gates(&metrics);

        assert!(
            violations.is_empty(),
            "Should pass all quality gates with good metrics"
        );
    }

    #[test]
    #[ignore = "Quality monitoring needs implementation"]
    fn test_quality_gate_violations() {
        let metrics = QualityMetrics {
            coverage: 90.0,         // Below 95%
            mutation_score: 0.92,   // Below 95%
            test_ratio: 2.5,        // Below 3:1
            max_complexity: 8,      // Above 5
            property_test_count: 0, // No property tests
            benchmark_count: 0,     // No benchmarks
            doc_coverage: 85.0,     // Below 100%
        };

        let gate_checker = QualityGateChecker::new();
        let violations = gate_checker.check_gates(&metrics);

        assert_eq!(violations.len(), 7, "Should have 7 quality gate violations");
        assert!(violations.contains(&"Coverage 90.0% below 95% threshold".to_string()));
        assert!(violations.contains(&"Mutation score 92.0% below 95% threshold".to_string()));
        assert!(violations.contains(&"Test ratio 2.5:1 below 3:1 requirement".to_string()));
    }
}

// =============================================================================
// QUALITY CONFIGURATION TYPES (Test-Driven Design)
// =============================================================================

/// Coverage configuration and requirements
#[derive(Debug)]
pub struct CoverageConfig {
    overall_minimum: f64,
    new_code_minimum: f64,
    per_module: HashMap<String, f64>,
}

impl CoverageConfig {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn overall_minimum(&self) -> f64 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn new_code_minimum(&self) -> f64 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn per_module_requirements(&self) -> &HashMap<String, f64> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

/// Coverage validation results
#[derive(Debug)]
pub struct CoverageValidator;

impl CoverageValidator {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn validate(
        &self,
        coverage: &HashMap<String, f64>,
    ) -> Result<CoverageReport, CoverageError> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct CoverageReport {
    pub violations: Vec<String>,
}

#[derive(Debug)]
pub struct CoverageError {
    pub violations: Vec<String>,
}

/// Complexity configuration and limits
#[derive(Debug)]
pub struct ComplexityConfig {
    max_cyclomatic: u32,
    max_cognitive: u32,
    max_nesting: u32,
}

impl ComplexityConfig {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn max_cyclomatic(&self) -> u32 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn max_cognitive(&self) -> u32 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn max_nesting(&self) -> u32 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct ComplexityValidator;

impl ComplexityValidator {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn validate_function(&self, function: &MockFunction) -> Result<(), Vec<String>> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct MockFunction {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub nesting: u32,
}

/// Mutation testing configuration
#[derive(Debug)]
pub struct MutationConfig {
    minimum_kill_rate: f64,
    timeout: Duration,
    critical_mutations: Vec<String>,
}

impl MutationConfig {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn minimum_kill_rate(&self) -> f64 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn timeout(&self) -> Duration {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn critical_mutations(&self) -> &[String] {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct MutationValidator;

impl MutationValidator {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn validate(&self, report: &MutationReport) -> Result<(), String> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct MutationReport {
    pub total_mutants: u32,
    pub killed_mutants: u32,
    pub survived_mutants: u32,
    pub timeout_mutants: u32,
    pub kill_rate: f64,
}

/// Performance requirements and validation
#[derive(Debug)]
pub struct PerformanceConfig {
    actor_spawn_p99: Duration,
    message_send_p99: Duration,
    ask_latency_p99: Duration,
    supervision_restart_p99: Duration,
}

impl PerformanceConfig {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn actor_spawn_p99(&self) -> Duration {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn message_send_p99(&self) -> Duration {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn ask_latency_p99(&self) -> Duration {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn supervision_restart_p99(&self) -> Duration {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct PerformanceValidator;

impl PerformanceValidator {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn validate(&self, metrics: &PerformanceMetrics) -> Result<(), PerformanceError> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub actor_spawn_p99: Duration,
    pub message_send_p99: Duration,
    pub ask_latency_p99: Duration,
    pub supervision_restart_p99: Duration,
}

#[derive(Debug)]
pub struct PerformanceError {
    pub violations: Vec<String>,
}

/// Test-to-code ratio requirements
#[derive(Debug)]
pub struct TestRatioConfig;

impl TestRatioConfig {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn minimum_ratio(&self) -> f64 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn enforce_on_new_code(&self) -> bool {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct TestRatioValidator;

impl TestRatioValidator {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn validate(&self, stats: &CodeStats) -> Result<(), String> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct CodeStats {
    pub source_lines: usize,
    pub test_lines: usize,
}

/// Property test analysis and requirements
#[derive(Debug)]
pub struct PropertyTestAnalyzer;

impl PropertyTestAnalyzer {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn analyze(&self, code: &str) -> PropertyTestAnalysis {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct PropertyTestAnalysis {
    pub regular_tests: usize,
    pub property_tests: usize,
}

impl PropertyTestAnalysis {
    pub fn has_required_properties(&self) -> bool {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

/// Documentation requirements and validation
#[derive(Debug)]
pub struct DocumentationConfig;

impl DocumentationConfig {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn public_items_coverage(&self) -> f64 {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn examples_required(&self) -> bool {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn doctests_required(&self) -> bool {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct DocumentationValidator;

impl DocumentationValidator {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn validate(&self, code: &str) -> Result<(), DocumentationError> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct DocumentationError {
    pub violations: Vec<String>,
}

/// Quality metrics collection and monitoring
#[derive(Debug)]
pub struct QualityMetricsCollector;

impl QualityMetricsCollector {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn collect_from_path(&self, path: &str) -> QualityMetrics {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}

#[derive(Debug)]
pub struct QualityMetrics {
    pub coverage: f64,
    pub mutation_score: f64,
    pub test_ratio: f64,
    pub max_complexity: u32,
    pub property_test_count: u32,
    pub benchmark_count: u32,
    pub doc_coverage: f64,
}

#[derive(Debug)]
pub struct QualityGateChecker;

impl QualityGateChecker {
    pub fn new() -> Self {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }

    pub fn check_gates(&self, metrics: &QualityMetrics) -> Vec<String> {
        unimplemented!("EXTREME TDD: Implementation follows tests")
    }
}
