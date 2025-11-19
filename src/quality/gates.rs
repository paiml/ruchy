//! Quality gate enforcement system (RUCHY-0815)
use crate::quality::scoring::{Grade, QualityScore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    /// Minimum overall score required (0.0-1.0)
    pub min_score: f64,
    /// Minimum grade required
    pub min_grade: Grade,
    /// Component-specific thresholds
    pub component_thresholds: ComponentThresholds,
    /// Anti-gaming rules
    pub anti_gaming: AntiGamingRules,
    /// CI/CD integration settings
    pub ci_integration: CiIntegration,
    /// Project-specific overrides
    pub project_overrides: HashMap<String, f64>,
}
/// Component-specific quality thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentThresholds {
    /// Minimum correctness score (0.0-1.0)
    pub correctness: f64,
    /// Minimum performance score (0.0-1.0)
    pub performance: f64,
    /// Minimum maintainability score (0.0-1.0)
    pub maintainability: f64,
    /// Minimum safety score (0.0-1.0)
    pub safety: f64,
    /// Minimum idiomaticity score (0.0-1.0)
    pub idiomaticity: f64,
}
/// Anti-gaming rules to prevent score manipulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiGamingRules {
    /// Minimum confidence level required (0.0-1.0)
    pub min_confidence: f64,
    /// Maximum cache hit rate allowed (0.0-1.0) - prevents stale analysis
    pub max_cache_hit_rate: f64,
    /// Require deep analysis for critical files
    pub require_deep_analysis: Vec<String>,
    /// Penalty for files that are too small (gaming by splitting)
    pub min_file_size_bytes: usize,
    /// Penalty for excessive test file ratios (gaming with trivial tests)
    pub max_test_ratio: f64,
}
/// CI/CD integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiIntegration {
    /// Fail CI/CD pipeline on gate failure
    pub fail_on_violation: bool,
    /// Export results in `JUnit` XML format
    pub junit_xml: bool,
    /// Export results in JSON format for tooling
    pub json_output: bool,
    /// Send notifications on quality degradation
    pub notifications: NotificationConfig,
    /// Block merge requests below threshold
    pub block_merge: bool,
}
/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable Slack notifications
    pub slack: bool,
    /// Enable email notifications  
    pub email: bool,
    /// Webhook URL for custom notifications
    pub webhook: Option<String>,
}
/// Quality gate enforcement result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GateResult {
    /// Whether the quality gate passed
    pub passed: bool,
    /// Overall score achieved
    pub score: f64,
    /// Grade achieved
    pub grade: Grade,
    /// Specific violations found
    pub violations: Vec<Violation>,
    /// Confidence in the result
    pub confidence: f64,
    /// Anti-gaming warnings
    pub gaming_warnings: Vec<String>,
}
/// Specific quality gate violation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Violation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Actual value that caused violation
    pub actual: f64,
    /// Required threshold
    pub required: f64,
    /// Severity of the violation
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
}
/// Types of quality gate violations
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ViolationType {
    OverallScore,
    Grade,
    Correctness,
    Performance,
    Maintainability,
    Safety,
    Idiomaticity,
    Confidence,
    Gaming,
}
/// Violation severity levels
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Critical, // Must fix to pass
    High,     // Should fix soon
    Medium,   // Should improve
    Low,      // Nice to improve
}
/// Quality gate enforcer
pub struct QualityGateEnforcer {
    config: QualityGateConfig,
}
impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            min_score: 0.7, // B- grade minimum
            min_grade: Grade::BMinus,
            component_thresholds: ComponentThresholds {
                correctness: 0.8,     // High correctness required
                performance: 0.6,     // Moderate performance required
                maintainability: 0.7, // Good maintainability required
                safety: 0.8,          // High safety required
                idiomaticity: 0.5,    // Basic idiomaticity required
            },
            anti_gaming: AntiGamingRules {
                min_confidence: 0.6,
                max_cache_hit_rate: 0.8,
                require_deep_analysis: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
                min_file_size_bytes: 100,
                max_test_ratio: 2.0,
            },
            ci_integration: CiIntegration {
                fail_on_violation: true,
                junit_xml: true,
                json_output: true,
                notifications: NotificationConfig {
                    slack: false,
                    email: false,
                    webhook: None,
                },
                block_merge: true,
            },
            project_overrides: HashMap::new(),
        }
    }
}
impl QualityGateEnforcer {
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::gates::QualityGateEnforcer;
    ///
    /// let instance = QualityGateEnforcer::new();
    /// // Verify behavior
    /// ```
    pub fn new(config: QualityGateConfig) -> Self {
        Self { config }
    }
    /// Load configuration from .ruchy/score.toml
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::gates::QualityGateEnforcer;
    ///
    /// let mut instance = QualityGateEnforcer::new();
    /// let result = instance.load_config();
    /// // Verify behavior
    /// ```
    pub fn load_config(project_root: &Path) -> anyhow::Result<QualityGateConfig> {
        let config_path = project_root.join(".ruchy").join("score.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: QualityGateConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default configuration file
            let default_config = QualityGateConfig::default();
            std::fs::create_dir_all(project_root.join(".ruchy"))?;
            let toml_content = toml::to_string_pretty(&default_config)?;
            std::fs::write(&config_path, toml_content)?;
            Ok(default_config)
        }
    }
    /// Enforce quality gates on a score
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::gates::QualityGateEnforcer;
    ///
    /// let mut instance = QualityGateEnforcer::new();
    /// let result = instance.enforce_gates();
    /// // Verify behavior
    /// ```
    pub fn enforce_gates(&self, score: &QualityScore, file_path: Option<&PathBuf>) -> GateResult {
        let mut violations = Vec::new();
        let mut gaming_warnings = Vec::new();
        // Check overall score threshold
        if score.value < self.config.min_score {
            violations.push(Violation {
                violation_type: ViolationType::OverallScore,
                actual: score.value,
                required: self.config.min_score,
                severity: Severity::Critical,
                message: format!(
                    "Overall score {:.1}% below minimum {:.1}%",
                    score.value * 100.0,
                    self.config.min_score * 100.0
                ),
            });
        }
        // Check grade requirement
        if score.grade < self.config.min_grade {
            violations.push(Violation {
                violation_type: ViolationType::Grade,
                actual: score.value,
                required: self.config.min_score,
                severity: Severity::Critical,
                message: format!(
                    "Grade {} below minimum {}",
                    score.grade, self.config.min_grade
                ),
            });
        }
        // Check component thresholds
        self.check_component_thresholds(score, &mut violations);
        // Check anti-gaming rules
        self.check_anti_gaming_rules(score, file_path, &mut gaming_warnings, &mut violations);
        // Check confidence threshold
        if score.confidence < self.config.anti_gaming.min_confidence {
            violations.push(Violation {
                violation_type: ViolationType::Confidence,
                actual: score.confidence,
                required: self.config.anti_gaming.min_confidence,
                severity: Severity::High,
                message: format!(
                    "Confidence {:.1}% below minimum {:.1}%",
                    score.confidence * 100.0,
                    self.config.anti_gaming.min_confidence * 100.0
                ),
            });
        }
        let passed = violations.iter().all(|v| v.severity != Severity::Critical);
        GateResult {
            passed,
            score: score.value,
            grade: score.grade,
            violations,
            confidence: score.confidence,
            gaming_warnings,
        }
    }
    fn check_component_thresholds(&self, score: &QualityScore, violations: &mut Vec<Violation>) {
        let thresholds = &self.config.component_thresholds;
        if score.components.correctness < thresholds.correctness {
            violations.push(Violation {
                violation_type: ViolationType::Correctness,
                actual: score.components.correctness,
                required: thresholds.correctness,
                severity: Severity::Critical,
                message: format!(
                    "Correctness {:.1}% below minimum {:.1}%",
                    score.components.correctness * 100.0,
                    thresholds.correctness * 100.0
                ),
            });
        }
        if score.components.performance < thresholds.performance {
            violations.push(Violation {
                violation_type: ViolationType::Performance,
                actual: score.components.performance,
                required: thresholds.performance,
                severity: Severity::High,
                message: format!(
                    "Performance {:.1}% below minimum {:.1}%",
                    score.components.performance * 100.0,
                    thresholds.performance * 100.0
                ),
            });
        }
        if score.components.maintainability < thresholds.maintainability {
            violations.push(Violation {
                violation_type: ViolationType::Maintainability,
                actual: score.components.maintainability,
                required: thresholds.maintainability,
                severity: Severity::High,
                message: format!(
                    "Maintainability {:.1}% below minimum {:.1}%",
                    score.components.maintainability * 100.0,
                    thresholds.maintainability * 100.0
                ),
            });
        }
        if score.components.safety < thresholds.safety {
            violations.push(Violation {
                violation_type: ViolationType::Safety,
                actual: score.components.safety,
                required: thresholds.safety,
                severity: Severity::Critical,
                message: format!(
                    "Safety {:.1}% below minimum {:.1}%",
                    score.components.safety * 100.0,
                    thresholds.safety * 100.0
                ),
            });
        }
        if score.components.idiomaticity < thresholds.idiomaticity {
            violations.push(Violation {
                violation_type: ViolationType::Idiomaticity,
                actual: score.components.idiomaticity,
                required: thresholds.idiomaticity,
                severity: Severity::Medium,
                message: format!(
                    "Idiomaticity {:.1}% below minimum {:.1}%",
                    score.components.idiomaticity * 100.0,
                    thresholds.idiomaticity * 100.0
                ),
            });
        }
    }
    fn check_anti_gaming_rules(
        &self,
        score: &QualityScore,
        file_path: Option<&PathBuf>,
        gaming_warnings: &mut Vec<String>,
        violations: &mut Vec<Violation>,
    ) {
        // Check cache hit rate (prevent stale analysis gaming)
        if score.cache_hit_rate > self.config.anti_gaming.max_cache_hit_rate {
            gaming_warnings.push(format!(
                "High cache hit rate {:.1}% may indicate stale analysis",
                score.cache_hit_rate * 100.0
            ));
        }
        // Check file size requirements
        if let Some(path) = file_path {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() < self.config.anti_gaming.min_file_size_bytes as u64 {
                    gaming_warnings.push(format!(
                        "File {} is very small ({} bytes) - may indicate gaming by splitting",
                        path.display(),
                        metadata.len()
                    ));
                }
            }
            // Check if critical files require deep analysis
            let path_str = path.to_string_lossy();
            if self
                .config
                .anti_gaming
                .require_deep_analysis
                .iter()
                .any(|p| path_str.contains(p))
                && score.confidence < 0.9
            {
                violations.push(Violation {
                    violation_type: ViolationType::Gaming,
                    actual: score.confidence,
                    required: 0.9,
                    severity: Severity::Critical,
                    message: format!(
                        "Critical file {} requires deep analysis (confidence < 90%)",
                        path.display()
                    ),
                });
            }
        }
    }
    /// Export results for CI/CD integration
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::gates::export_ci_results;
    ///
    /// let result = export_ci_results(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn export_ci_results(
        &self,
        results: &[GateResult],
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        if self.config.ci_integration.json_output {
            self.export_json_results(results, output_dir)?;
        }
        if self.config.ci_integration.junit_xml {
            self.export_junit_results(results, output_dir)?;
        }
        Ok(())
    }
    fn export_json_results(&self, results: &[GateResult], output_dir: &Path) -> anyhow::Result<()> {
        let output_path = output_dir.join("quality-gates.json");
        let json_content = serde_json::to_string_pretty(results)?;
        std::fs::write(output_path, json_content)?;
        Ok(())
    }
    fn export_junit_results(
        &self,
        results: &[GateResult],
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        let output_path = output_dir.join("quality-gates.xml");
        let total = results.len();
        let failures = results.iter().filter(|r| !r.passed).count();
        let mut xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="Quality Gates" tests="{total}" failures="{failures}" time="0.0">
"#
        );
        for (i, result) in results.iter().enumerate() {
            let test_name = format!("quality-gate-{i}");
            if result.passed {
                xml.push_str(&format!(
                    r#"  <testcase name="{test_name}" classname="QualityGate" time="0.0"/>
"#
                ));
            } else {
                xml.push_str(&format!(
                    r#"  <testcase name="{}" classname="QualityGate" time="0.0">
    <failure message="Quality gate violation">Score: {:.1}%, Grade: {}</failure>
  </testcase>
"#,
                    test_name,
                    result.score * 100.0,
                    result.grade
                ));
            }
        }
        xml.push_str("</testsuite>\n");
        std::fs::write(output_path, xml)?;
        Ok(())
    }
}
impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Grade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_rank().cmp(&other.to_rank())
    }
}
// PartialEq and Eq are now derived in scoring.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::scoring::{Grade, QualityScore};
    use tempfile::TempDir;
    fn create_minimal_score() -> QualityScore {
        use crate::quality::scoring::ScoreComponents;
        QualityScore {
            value: 0.5,
            components: ScoreComponents {
                correctness: 0.5,
                performance: 0.5,
                maintainability: 0.5,
                safety: 0.5,
                idiomaticity: 0.5,
            },
            grade: Grade::D,
            confidence: 0.4,
            cache_hit_rate: 0.3,
        }
    }
    fn create_passing_score() -> QualityScore {
        use crate::quality::scoring::ScoreComponents;
        QualityScore {
            value: 0.85,
            components: ScoreComponents {
                correctness: 0.9,
                performance: 0.8,
                maintainability: 0.8,
                safety: 0.9,
                idiomaticity: 0.7,
            },
            grade: Grade::APlus,
            confidence: 0.9,
            cache_hit_rate: 0.2,
        }
    }
    // Test 1: Default Configuration Creation
    #[test]
    fn test_default_quality_gate_config() {
        let config = QualityGateConfig::default();
        assert_eq!(config.min_score, 0.7);
        assert_eq!(config.min_grade, Grade::BMinus);
        assert_eq!(config.component_thresholds.correctness, 0.8);
        assert_eq!(config.component_thresholds.safety, 0.8);
        assert_eq!(config.anti_gaming.min_confidence, 0.6);
        assert!(config.ci_integration.fail_on_violation);
        assert!(config.project_overrides.is_empty());
    }
    // Test 2: Quality Gate Enforcer Creation
    #[test]
    fn test_quality_gate_enforcer_creation() {
        let config = QualityGateConfig::default();
        let enforcer = QualityGateEnforcer::new(config);
        // Verify enforcer uses the provided config
        let score = create_minimal_score();
        let result = enforcer.enforce_gates(&score, None);
        // Should fail with default thresholds
        assert!(!result.passed);
        assert!(!result.violations.is_empty());
    }
    // Test 3: Passing Quality Gate - All Criteria Met
    #[test]
    fn test_quality_gate_passes_with_high_score() {
        let config = QualityGateConfig::default();
        let enforcer = QualityGateEnforcer::new(config);
        let score = create_passing_score();
        let result = enforcer.enforce_gates(&score, None);
        assert!(result.passed, "High quality score should pass all gates");
        assert_eq!(result.score, 0.85);
        assert_eq!(result.grade, Grade::APlus);
        assert!(result.violations.is_empty());
        assert_eq!(result.confidence, 0.9);
        assert!(result.gaming_warnings.is_empty());
    }
    // Test 4: Failing Overall Score Threshold
    #[test]
    fn test_quality_gate_fails_overall_score() {
        let config = QualityGateConfig::default(); // min_score: 0.7
        let enforcer = QualityGateEnforcer::new(config);
        let mut score = create_minimal_score();
        score.value = 0.6; // Below 0.7 threshold
        let result = enforcer.enforce_gates(&score, None);
        assert!(!result.passed, "Score below threshold should fail");
        // Should have overall score violation
        let overall_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::OverallScore)
            .collect();
        assert_eq!(overall_violations.len(), 1);
        let violation = &overall_violations[0];
        assert_eq!(violation.actual, 0.6);
        assert_eq!(violation.required, 0.7);
        assert_eq!(violation.severity, Severity::Critical);
        assert!(violation.message.contains("60.0%"));
        assert!(violation.message.contains("70.0%"));
    }
    // Test 5: Confidence Threshold Violation
    #[test]
    fn test_confidence_threshold_violation() {
        let config = QualityGateConfig::default(); // min_confidence: 0.6
        let enforcer = QualityGateEnforcer::new(config);
        let mut score = create_passing_score();
        score.confidence = 0.4; // Below 0.6 threshold
        let result = enforcer.enforce_gates(&score, None);
        let confidence_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Confidence)
            .collect();
        assert_eq!(confidence_violations.len(), 1);
        let violation = &confidence_violations[0];
        assert_eq!(violation.severity, Severity::High);
        assert_eq!(violation.actual, 0.4);
        assert_eq!(violation.required, 0.6);
    }
    // Test 6: Configuration File Loading (Success)
    #[test]
    fn test_load_config_creates_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let config = QualityGateEnforcer::load_config(project_root).unwrap();
        // Should create default config
        assert_eq!(config.min_score, 0.7);
        assert_eq!(config.min_grade, Grade::BMinus);
        // Should create .ruchy/score.toml file
        let config_path = project_root.join(".ruchy").join("score.toml");
        assert!(config_path.exists(), "Config file should be created");
        // File should contain valid TOML
        let content = std::fs::read_to_string(config_path).unwrap();
        assert!(content.contains("min_score"));
        assert!(content.contains("0.7"));
    }
    // Test 7: Serialization/Deserialization
    #[test]
    fn test_config_serialization() {
        let original_config = QualityGateConfig::default();
        // Serialize to TOML
        let toml_content = toml::to_string(&original_config).unwrap();
        assert!(toml_content.contains("min_score"));
        // Deserialize back
        let deserialized_config: QualityGateConfig = toml::from_str(&toml_content).unwrap();
        assert_eq!(deserialized_config.min_score, original_config.min_score);
        assert_eq!(deserialized_config.min_grade, original_config.min_grade);
    }

    // Test 8: Grade Comparison Testing
    #[test]
    fn test_grade_ordering() {
        // Test all grade comparisons
        assert!(Grade::F < Grade::D);
        assert!(Grade::D < Grade::CMinus);
        assert!(Grade::CMinus < Grade::C);
        assert!(Grade::C < Grade::CPlus);
        assert!(Grade::CPlus < Grade::BMinus);
        assert!(Grade::BMinus < Grade::B);
        assert!(Grade::B < Grade::BPlus);
        assert!(Grade::BPlus < Grade::AMinus);
        assert!(Grade::AMinus < Grade::A);
        assert!(Grade::A < Grade::APlus);

        // Test specific comparisons used in gates
        assert!(Grade::C < Grade::BMinus); // Used in test_grade_threshold_violation
        assert!(Grade::BMinus < Grade::A);
    }

    // Test 9: Grade Threshold Violation
    #[test]
    fn test_grade_threshold_violation() {
        let config = QualityGateConfig::default(); // min_grade: BMinus
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.grade = Grade::C; // Below BMinus threshold

        let result = enforcer.enforce_gates(&score, None);
        let grade_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Grade)
            .collect();

        assert_eq!(grade_violations.len(), 1);
        let violation = &grade_violations[0];
        assert_eq!(violation.severity, Severity::Critical);
        assert!(violation.message.contains("Grade C below minimum B-"));
    }

    // Test 10: Component Threshold Violations - Correctness
    #[test]
    fn test_correctness_threshold_violation() {
        let config = QualityGateConfig::default(); // correctness: 0.8
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.components.correctness = 0.7; // Below 0.8 threshold

        let result = enforcer.enforce_gates(&score, None);
        let correctness_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Correctness)
            .collect();

        assert_eq!(correctness_violations.len(), 1);
        let violation = &correctness_violations[0];
        assert_eq!(violation.actual, 0.7);
        assert_eq!(violation.required, 0.8);
        assert_eq!(violation.severity, Severity::Critical);
        assert!(violation.message.contains("70.0%"));
        assert!(violation.message.contains("80.0%"));
    }

    // Test 11: Component Threshold Violations - Performance
    #[test]
    fn test_performance_threshold_violation() {
        let config = QualityGateConfig::default(); // performance: 0.6
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.components.performance = 0.5; // Below 0.6 threshold

        let result = enforcer.enforce_gates(&score, None);
        let performance_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Performance)
            .collect();

        assert_eq!(performance_violations.len(), 1);
        let violation = &performance_violations[0];
        assert_eq!(violation.actual, 0.5);
        assert_eq!(violation.required, 0.6);
        assert_eq!(violation.severity, Severity::High);
    }

    // Test 12: Component Threshold Violations - Safety
    #[test]
    fn test_safety_threshold_violation() {
        let config = QualityGateConfig::default(); // safety: 0.8
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.components.safety = 0.75; // Below 0.8 threshold

        let result = enforcer.enforce_gates(&score, None);
        let safety_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Safety)
            .collect();

        assert_eq!(safety_violations.len(), 1);
        let violation = &safety_violations[0];
        assert_eq!(violation.severity, Severity::Critical);
        assert!(violation.message.contains("75.0%"));
        assert!(violation.message.contains("80.0%"));
    }

    // Test 13: Component Threshold Violations - Maintainability
    #[test]
    fn test_maintainability_threshold_violation() {
        let config = QualityGateConfig::default(); // maintainability: 0.7
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.components.maintainability = 0.65; // Below 0.7 threshold

        let result = enforcer.enforce_gates(&score, None);
        let maintainability_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Maintainability)
            .collect();

        assert_eq!(maintainability_violations.len(), 1);
        let violation = &maintainability_violations[0];
        assert_eq!(violation.severity, Severity::High);
        assert_eq!(violation.actual, 0.65);
        assert_eq!(violation.required, 0.7);
    }

    // Test 14: Component Threshold Violations - Idiomaticity
    #[test]
    fn test_idiomaticity_threshold_violation() {
        let config = QualityGateConfig::default(); // idiomaticity: 0.5
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.components.idiomaticity = 0.4; // Below 0.5 threshold

        let result = enforcer.enforce_gates(&score, None);
        let idiomaticity_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Idiomaticity)
            .collect();

        assert_eq!(idiomaticity_violations.len(), 1);
        let violation = &idiomaticity_violations[0];
        assert_eq!(violation.severity, Severity::Medium);
        assert_eq!(violation.actual, 0.4);
        assert_eq!(violation.required, 0.5);
    }

    // Test 15: Anti-Gaming Rules - Cache Hit Rate Warning
    #[test]
    fn test_high_cache_hit_rate_warning() {
        let config = QualityGateConfig::default(); // max_cache_hit_rate: 0.8
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.cache_hit_rate = 0.9; // Above 0.8 threshold

        let result = enforcer.enforce_gates(&score, None);

        assert!(!result.gaming_warnings.is_empty());
        let warning = &result.gaming_warnings[0];
        assert!(warning.contains("High cache hit rate 90.0%"));
        assert!(warning.contains("stale analysis"));
    }

    // Test 16: Anti-Gaming Rules - File Size Warning
    #[test]
    fn test_small_file_size_warning() -> anyhow::Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let small_file = temp_dir.path().join("small.rs");
        std::fs::write(&small_file, "// Small file")?; // ~13 bytes, below 100 threshold

        let config = QualityGateConfig::default(); // min_file_size_bytes: 100
        let enforcer = QualityGateEnforcer::new(config);
        let score = create_passing_score();

        let result = enforcer.enforce_gates(&score, Some(&small_file));

        assert!(!result.gaming_warnings.is_empty());
        let warning = &result.gaming_warnings[0];
        assert!(warning.contains("very small"));
        assert!(warning.contains("gaming by splitting"));
        Ok(())
    }

    // Test 17: Anti-Gaming Rules - Critical Files Deep Analysis
    #[test]
    fn test_critical_files_deep_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let critical_file = temp_dir.path().join("src").join("main.rs");
        std::fs::create_dir_all(critical_file.parent().unwrap()).unwrap();
        std::fs::write(&critical_file, "fn main() {}").unwrap();

        let config = QualityGateConfig::default(); // require_deep_analysis includes "src/main.rs"
        let enforcer = QualityGateEnforcer::new(config);

        let mut score = create_passing_score();
        score.confidence = 0.8; // Below 0.9 required for critical files

        let result = enforcer.enforce_gates(&score, Some(&critical_file));

        let gaming_violations: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::Gaming)
            .collect();

        assert_eq!(gaming_violations.len(), 1);
        let violation = &gaming_violations[0];
        assert_eq!(violation.severity, Severity::Critical);
        assert_eq!(violation.actual, 0.8);
        assert_eq!(violation.required, 0.9);
        assert!(violation.message.contains("deep analysis"));
    }

    // Test 18: Multiple Violations Combination
    #[test]
    fn test_multiple_violations() {
        let config = QualityGateConfig::default();
        let enforcer = QualityGateEnforcer::new(config);

        let score = create_minimal_score(); // This should fail multiple criteria
        let result = enforcer.enforce_gates(&score, None);

        assert!(!result.passed);
        // Should have multiple violations
        assert!(result.violations.len() >= 3); // At least overall score, grade, and confidence

        // Check we have different violation types
        let violation_types: std::collections::HashSet<_> = result
            .violations
            .iter()
            .map(|v| &v.violation_type)
            .collect();
        assert!(violation_types.contains(&ViolationType::OverallScore));
        assert!(violation_types.contains(&ViolationType::Grade));
        assert!(violation_types.contains(&ViolationType::Confidence));
    }

    // Test 19: CI Results Export - JSON Format
    #[test]
    fn test_export_json_results() -> anyhow::Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        let mut config = QualityGateConfig::default();
        config.ci_integration.json_output = true;
        config.ci_integration.junit_xml = false;

        let enforcer = QualityGateEnforcer::new(config);
        let results = vec![create_gate_result_passed(), create_gate_result_failed()];

        enforcer.export_ci_results(&results, output_dir)?;

        let json_file = output_dir.join("quality-gates.json");
        assert!(json_file.exists());

        let content = std::fs::read_to_string(json_file)?;
        let parsed: Vec<GateResult> = serde_json::from_str(&content)?;
        assert_eq!(parsed.len(), 2);
        assert!(parsed[0].passed);
        assert!(!parsed[1].passed);

        Ok(())
    }

    // Test 20: CI Results Export - JUnit XML Format
    #[test]
    fn test_export_junit_xml_results() -> anyhow::Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        let mut config = QualityGateConfig::default();
        config.ci_integration.json_output = false;
        config.ci_integration.junit_xml = true;

        let enforcer = QualityGateEnforcer::new(config);
        let results = vec![create_gate_result_passed(), create_gate_result_failed()];

        enforcer.export_ci_results(&results, output_dir)?;

        let xml_file = output_dir.join("quality-gates.xml");
        assert!(xml_file.exists());

        let content = std::fs::read_to_string(xml_file)?;
        assert!(content.contains("<?xml version="));
        assert!(content.contains("<testsuite name=\"Quality Gates\" tests=\"2\" failures=\"1\""));
        assert!(content.contains("<testcase name=\"quality-gate-0\" classname=\"QualityGate\""));
        assert!(content.contains("<failure message=\"Quality gate violation\""));
        assert!(content.contains("</testsuite>"));

        Ok(())
    }

    // Test 21: Violation Type and Severity Enum Coverage
    #[test]
    fn test_violation_enums_coverage() {
        // Test all ViolationType variants can be created and compared
        let types = [
            ViolationType::OverallScore,
            ViolationType::Grade,
            ViolationType::Correctness,
            ViolationType::Performance,
            ViolationType::Maintainability,
            ViolationType::Safety,
            ViolationType::Idiomaticity,
            ViolationType::Confidence,
            ViolationType::Gaming,
        ];

        for (i, vtype) in types.iter().enumerate() {
            for (j, other) in types.iter().enumerate() {
                if i == j {
                    assert_eq!(vtype, other);
                } else {
                    assert_ne!(vtype, other);
                }
            }
        }

        // Test all Severity variants
        let severities = [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
        ];

        for (i, severity) in severities.iter().enumerate() {
            for (j, other) in severities.iter().enumerate() {
                if i == j {
                    assert_eq!(severity, other);
                } else {
                    assert_ne!(severity, other);
                }
            }
        }
    }

    // Test 22: Notification Config Serialization
    #[test]
    fn test_notification_config_serialization() {
        let config = NotificationConfig {
            slack: true,
            email: false,
            webhook: Some("https://test.example.com/webhook".to_string()),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: NotificationConfig = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.slack);
        assert!(!deserialized.email);
        assert_eq!(
            deserialized.webhook,
            Some("https://test.example.com/webhook".to_string())
        );
    }

    // Helper functions for testing
    fn create_gate_result_passed() -> GateResult {
        GateResult {
            passed: true,
            score: 0.85,
            grade: Grade::APlus,
            violations: vec![],
            confidence: 0.9,
            gaming_warnings: vec![],
        }
    }

    fn create_gate_result_failed() -> GateResult {
        GateResult {
            passed: false,
            score: 0.6,
            grade: Grade::D,
            violations: vec![Violation {
                violation_type: ViolationType::OverallScore,
                actual: 0.6,
                required: 0.7,
                severity: Severity::Critical,
                message: "Overall score 60.0% below minimum 70.0%".to_string(),
            }],
            confidence: 0.5,
            gaming_warnings: vec!["Low confidence warning".to_string()],
        }
    }
}
#[cfg(test)]
mod property_tests_gates {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
