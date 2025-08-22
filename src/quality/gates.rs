//! Quality gate enforcement system (RUCHY-0815)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::quality::scoring::{QualityScore, Grade};

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
    
    /// Export results in JUnit XML format
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
                correctness: 0.8,   // High correctness required
                performance: 0.6,   // Moderate performance required
                maintainability: 0.7, // Good maintainability required  
                safety: 0.8,        // High safety required
                idiomaticity: 0.5,  // Basic idiomaticity required
            },
            anti_gaming: AntiGamingRules {
                min_confidence: 0.6,
                max_cache_hit_rate: 0.8,
                require_deep_analysis: vec![
                    "src/main.rs".to_string(),
                    "src/lib.rs".to_string(),
                ],
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
    pub fn new(config: QualityGateConfig) -> Self {
        Self { config }
    }
    
    /// Load configuration from .ruchy/score.toml
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
                    score.grade,
                    self.config.min_grade
                ),
            });
        }
        
        // Check component thresholds
        self.check_component_thresholds(&score, &mut violations);
        
        // Check anti-gaming rules
        self.check_anti_gaming_rules(&score, file_path, &mut gaming_warnings, &mut violations);
        
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
            if self.config.anti_gaming.require_deep_analysis.iter().any(|p| path_str.contains(p)) {
                if score.confidence < 0.9 {
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
    }
    
    /// Export results for CI/CD integration
    pub fn export_ci_results(&self, results: &[GateResult], output_dir: &Path) -> anyhow::Result<()> {
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
    
    fn export_junit_results(&self, results: &[GateResult], output_dir: &Path) -> anyhow::Result<()> {
        let output_path = output_dir.join("quality-gates.xml");
        
        let total = results.len();
        let failures = results.iter().filter(|r| !r.passed).count();
        
        let mut xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="Quality Gates" tests="{}" failures="{}" time="0.0">
"#, total, failures);
        
        for (i, result) in results.iter().enumerate() {
            let test_name = format!("quality-gate-{}", i);
            if result.passed {
                xml.push_str(&format!(
                    r#"  <testcase name="{}" classname="QualityGate" time="0.0"/>
"#,
                    test_name
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
        // use std::cmp::Ordering;
        use Grade::*;
        
        let self_rank = match self {
            F => 0,
            D => 1,
            CMinus => 2,
            C => 3,
            CPlus => 4,
            BMinus => 5,
            B => 6,
            BPlus => 7,
            AMinus => 8,
            A => 9,
            APlus => 10,
        };
        
        let other_rank = match other {
            F => 0,
            D => 1,
            CMinus => 2,
            C => 3,
            CPlus => 4,
            BMinus => 5,
            B => 6,
            BPlus => 7,
            AMinus => 8,
            A => 9,
            APlus => 10,
        };
        
        self_rank.cmp(&other_rank)
    }
}

// PartialEq and Eq are now derived in scoring.rs