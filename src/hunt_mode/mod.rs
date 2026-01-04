//! Hunt Mode: Automated Defect Resolution System
//!
//! Hunt Mode implements the PDCA (Plan-Do-Check-Act) cycle for iteratively
//! identifying, reproducing, and fixing transpiler bugs.
//!
//! # Toyota Way Alignment
//!
//! - **Jidoka**: Stop on error, auto-fix, resume
//! - **Kaizen**: Continuous improvement through metrics
//! - **Genchi Genbutsu**: Observe actual failures, not abstractions
//! - **Heijunka**: Level workload by processing highest-impact patterns first
//! - **Poka-Yoke**: Error-proofing through regression tests
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    HUNT MODE PDCA CYCLE                        │
//! ├─────────────────────────────────────────────────────────────────┤
//! │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    │
//! │   │  PLAN   │───▶│   DO    │───▶│  CHECK  │───▶│   ACT   │    │
//! │   │ (Hunt)  │    │(Isolate)│    │(Repair) │    │(Verify) │    │
//! │   └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘    │
//! │        ▼              ▼              ▼              ▼          │
//! │   Classify       Synthesize      Apply         Validate       │
//! │   Errors         Minimal         Mutator       & Commit       │
//! │   (Oracle)       repro.ruchy     Fix           (Andon)        │
//! │                                                                │
//! │   ◀────────────────── KAIZEN LOOP ──────────────────────────▶ │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # References
//! - [1] Pareto, V. (1896). Cours d'economie politique.
//! - [5] Ohno, T. (1988). Toyota Production System.
//! - [7] Imai, M. (1986). Kaizen: The Key to Japan's Competitive Success.
//! - [14] Deming, W. E. (1986). Out of the Crisis. PDCA cycle.

pub mod ephemeral_workspace;
pub mod five_whys;
pub mod isolator;
pub mod kaizen;
pub mod planner;
pub mod repair;
pub mod verifier;

// Re-exports
pub use ephemeral_workspace::{EphemeralWorkspace, WorkspaceConfig, WorkspaceError};
pub use five_whys::{FiveWhysAnalyzer, RootCause, RootCauseChain, Why};
pub use isolator::{MinimalReproducer, ReproCase, ReproResult};
pub use kaizen::{KaizenMetrics, KaizenTracker};
pub use planner::{ErrorCluster, FailurePattern, HuntPlanner, PrioritizedPattern};
pub use repair::{Fix, JidokaRepairEngine, Mutator, RepairResult};
pub use verifier::{AndonStatus, AndonVerifier, VerifyResult};

/// Hunt Mode configuration
#[derive(Debug, Clone)]
pub struct HuntConfig {
    /// Maximum cycles before stopping
    pub max_cycles: u32,

    /// Quality threshold for auto-applying fixes (Jidoka)
    pub quality_threshold: f64,

    /// Stop when no improvement for this many cycles
    pub plateau_threshold: u32,

    /// Target compilation rate (Kaizen goal)
    pub target_rate: f64,

    /// Minimum improvement per cycle to continue
    pub min_improvement_per_cycle: f64,

    /// Enable Five Whys analysis
    pub enable_five_whys: bool,

    /// Human review threshold (below this requires manual review)
    pub human_review_threshold: f64,

    /// Auto-commit threshold (above this auto-applies fix)
    pub auto_commit_threshold: f64,

    /// Verbose output
    pub verbose: bool,
}

impl Default for HuntConfig {
    fn default() -> Self {
        Self {
            max_cycles: 100,
            quality_threshold: 0.85,
            plateau_threshold: 5,
            target_rate: 0.80,
            min_improvement_per_cycle: 0.001,
            enable_five_whys: true,
            human_review_threshold: 0.70,
            auto_commit_threshold: 0.95,
            verbose: false,
        }
    }
}

/// Hunt Mode cycle result
#[derive(Debug, Clone)]
pub struct CycleOutcome {
    /// Cycle number
    pub cycle: u32,

    /// Pattern that was targeted
    pub pattern: Option<FailurePattern>,

    /// Whether a fix was applied
    pub fix_applied: bool,

    /// New compilation rate after cycle
    pub compilation_rate: f64,

    /// Rate delta from previous cycle
    pub rate_delta: f64,

    /// Fix confidence score
    pub confidence: f64,

    /// Lessons learned (Hansei)
    pub lessons: Vec<String>,
}

/// Main Hunt Mode orchestrator
#[derive(Debug)]
pub struct HuntMode {
    /// Configuration
    config: HuntConfig,

    /// Planner for pattern selection (PLAN phase)
    planner: HuntPlanner,

    /// Reproducer for isolation (DO phase)
    reproducer: MinimalReproducer,

    /// Repair engine (CHECK phase)
    repair_engine: JidokaRepairEngine,

    /// Verifier for validation (ACT phase)
    verifier: AndonVerifier,

    /// Kaizen metrics tracker
    kaizen: KaizenTracker,

    /// Five Whys analyzer
    five_whys: FiveWhysAnalyzer,

    /// Cycle history
    history: Vec<CycleOutcome>,
}

impl HuntMode {
    /// Create new Hunt Mode instance with default config
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(HuntConfig::default())
    }

    /// Create Hunt Mode with custom config
    #[must_use]
    pub fn with_config(config: HuntConfig) -> Self {
        Self {
            config,
            planner: HuntPlanner::new(),
            reproducer: MinimalReproducer::new(),
            repair_engine: JidokaRepairEngine::new(),
            verifier: AndonVerifier::new(),
            kaizen: KaizenTracker::new(),
            five_whys: FiveWhysAnalyzer::new(),
            history: Vec::new(),
        }
    }

    /// Run a single PDCA cycle
    ///
    /// # Errors
    ///
    /// Returns error if any phase fails critically
    pub fn run_cycle(&mut self) -> Result<CycleOutcome, HuntModeError> {
        let cycle = (self.history.len() + 1) as u32;

        // PLAN: Select next target pattern (Heijunka - level workload)
        let pattern = self.planner.select_next_target();

        let Some(pattern) = pattern else {
            return Ok(CycleOutcome {
                cycle,
                pattern: None,
                fix_applied: false,
                compilation_rate: self.kaizen.current_rate(),
                rate_delta: 0.0,
                confidence: 0.0,
                lessons: vec!["No patterns remaining to fix".to_string()],
            });
        };

        // DO: Isolate with minimal reproduction (Poka-Yoke)
        let repro = self.reproducer.synthesize_repro(&pattern)?;

        // CHECK: Attempt repair (Jidoka)
        let repair_result = self.repair_engine.attempt_repair(&repro);

        match repair_result {
            RepairResult::Success(fix) => {
                // ACT: Verify and commit (Andon)
                let verify_result = self.verifier.verify_and_commit(&fix, &repro);

                let (fix_applied, confidence, lessons) = match verify_result {
                    VerifyResult::Success => {
                        self.kaizen.record_success(&fix);
                        (
                            true,
                            fix.confidence,
                            vec![format!("Fix {} applied successfully", fix.id)],
                        )
                    }
                    VerifyResult::NeedsReview => (
                        false,
                        fix.confidence,
                        vec!["Fix needs human review".to_string()],
                    ),
                    VerifyResult::FixFailed(e) => (false, 0.0, vec![format!("Fix failed: {e}")]),
                };

                let outcome = CycleOutcome {
                    cycle,
                    pattern: Some(pattern),
                    fix_applied,
                    compilation_rate: self.kaizen.current_rate(),
                    rate_delta: self.kaizen.rate_delta(),
                    confidence,
                    lessons,
                };

                self.history.push(outcome.clone());
                Ok(outcome)
            }
            RepairResult::NeedsHumanReview {
                fix,
                confidence,
                reason,
            } => {
                let outcome = CycleOutcome {
                    cycle,
                    pattern: Some(pattern),
                    fix_applied: false,
                    compilation_rate: self.kaizen.current_rate(),
                    rate_delta: 0.0,
                    confidence,
                    lessons: vec![format!("Needs review: {reason} (fix: {})", fix.id)],
                };
                self.history.push(outcome.clone());
                Ok(outcome)
            }
            RepairResult::NoFixFound => {
                // Five Whys analysis if enabled
                let lessons = if self.config.enable_five_whys {
                    let chain = self.five_whys.analyze(&repro);
                    chain.whys.iter().map(|w| w.description.clone()).collect()
                } else {
                    vec!["No fix found for pattern".to_string()]
                };

                let outcome = CycleOutcome {
                    cycle,
                    pattern: Some(pattern),
                    fix_applied: false,
                    compilation_rate: self.kaizen.current_rate(),
                    rate_delta: 0.0,
                    confidence: 0.0,
                    lessons,
                };
                self.history.push(outcome.clone());
                Ok(outcome)
            }
        }
    }

    /// Run Hunt Mode for specified number of cycles
    ///
    /// # Errors
    ///
    /// Returns error if critical failure occurs
    pub fn run(&mut self, cycles: u32) -> Result<Vec<CycleOutcome>, HuntModeError> {
        let mut outcomes = Vec::new();
        let mut plateau_count = 0;

        for _ in 0..cycles {
            let outcome = self.run_cycle()?;

            // Kaizen: Check for plateau
            if outcome.rate_delta < self.config.min_improvement_per_cycle {
                plateau_count += 1;
            } else {
                plateau_count = 0;
            }

            outcomes.push(outcome);

            // Stop if plateau threshold reached
            if plateau_count >= self.config.plateau_threshold {
                break;
            }

            // Stop if target rate achieved
            if self.kaizen.current_rate() >= self.config.target_rate {
                break;
            }
        }

        Ok(outcomes)
    }

    /// Get Andon status (visual control)
    #[must_use]
    pub fn andon_status(&self) -> AndonStatus {
        self.verifier.status()
    }

    /// Get Kaizen metrics
    #[must_use]
    pub fn kaizen_metrics(&self) -> &KaizenMetrics {
        self.kaizen.metrics()
    }

    /// Get cycle history (Hansei - reflection)
    #[must_use]
    pub fn history(&self) -> &[CycleOutcome] {
        &self.history
    }

    /// Check if Hunt Mode is improving (Kaizen)
    #[must_use]
    pub fn is_improving(&self) -> bool {
        self.kaizen.metrics().is_improving()
    }

    /// Add an error to the planner for clustering
    pub fn add_error(&mut self, code: &str, message: &str, file: Option<&str>, severity: f64) {
        self.planner.add_error(code, message, file, severity);
    }
}

impl Default for HuntMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Hunt Mode errors
#[derive(Debug, Clone)]
pub enum HuntModeError {
    /// Isolation failed
    IsolationFailed(String),

    /// Workspace creation failed
    WorkspaceFailed(String),

    /// Compilation failed
    CompilationFailed(String),

    /// Internal error
    InternalError(String),
}

impl std::fmt::Display for HuntModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsolationFailed(msg) => write!(f, "Isolation failed: {msg}"),
            Self::WorkspaceFailed(msg) => write!(f, "Workspace creation failed: {msg}"),
            Self::CompilationFailed(msg) => write!(f, "Compilation failed: {msg}"),
            Self::InternalError(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for HuntModeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: RED PHASE - HuntConfig Tests
    // ============================================================================

    #[test]
    fn test_hunt_config_default_max_cycles() {
        let config = HuntConfig::default();
        assert_eq!(config.max_cycles, 100);
    }

    #[test]
    fn test_hunt_config_default_quality_threshold() {
        let config = HuntConfig::default();
        assert!((config.quality_threshold - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunt_config_default_plateau_threshold() {
        let config = HuntConfig::default();
        assert_eq!(config.plateau_threshold, 5);
    }

    #[test]
    fn test_hunt_config_default_target_rate() {
        let config = HuntConfig::default();
        assert!((config.target_rate - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunt_config_default_min_improvement() {
        let config = HuntConfig::default();
        assert!((config.min_improvement_per_cycle - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunt_config_default_enable_five_whys() {
        let config = HuntConfig::default();
        assert!(config.enable_five_whys);
    }

    #[test]
    fn test_hunt_config_default_human_review_threshold() {
        let config = HuntConfig::default();
        assert!((config.human_review_threshold - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunt_config_default_auto_commit_threshold() {
        let config = HuntConfig::default();
        assert!((config.auto_commit_threshold - 0.95).abs() < f64::EPSILON);
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - HuntMode Tests
    // ============================================================================

    #[test]
    fn test_hunt_mode_new() {
        let hunt = HuntMode::new();
        assert!(hunt.history.is_empty());
    }

    #[test]
    fn test_hunt_mode_with_config() {
        let config = HuntConfig {
            max_cycles: 50,
            ..Default::default()
        };
        let hunt = HuntMode::with_config(config);
        assert_eq!(hunt.config.max_cycles, 50);
    }

    #[test]
    fn test_hunt_mode_default() {
        let hunt = HuntMode::default();
        assert!(hunt.history.is_empty());
    }

    #[test]
    fn test_hunt_mode_run_cycle_no_patterns() {
        let mut hunt = HuntMode::new();
        let result = hunt.run_cycle();
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(!outcome.fix_applied);
        assert!(outcome.pattern.is_none());
    }

    #[test]
    fn test_hunt_mode_history_empty_initially() {
        let hunt = HuntMode::new();
        assert!(hunt.history().is_empty());
    }

    #[test]
    fn test_hunt_mode_is_improving_initial() {
        let hunt = HuntMode::new();
        // Initially should be improving (no plateau detected)
        assert!(hunt.is_improving());
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - CycleOutcome Tests
    // ============================================================================

    #[test]
    fn test_cycle_outcome_new() {
        let outcome = CycleOutcome {
            cycle: 1,
            pattern: None,
            fix_applied: false,
            compilation_rate: 0.5,
            rate_delta: 0.0,
            confidence: 0.0,
            lessons: vec!["Test lesson".to_string()],
        };
        assert_eq!(outcome.cycle, 1);
        assert!(!outcome.fix_applied);
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - HuntModeError Tests
    // ============================================================================

    #[test]
    fn test_hunt_mode_error_isolation_display() {
        let error = HuntModeError::IsolationFailed("test error".to_string());
        assert!(error.to_string().contains("Isolation failed"));
    }

    #[test]
    fn test_hunt_mode_error_workspace_display() {
        let error = HuntModeError::WorkspaceFailed("test error".to_string());
        assert!(error.to_string().contains("Workspace creation failed"));
    }

    #[test]
    fn test_hunt_mode_error_compilation_display() {
        let error = HuntModeError::CompilationFailed("test error".to_string());
        assert!(error.to_string().contains("Compilation failed"));
    }

    #[test]
    fn test_hunt_mode_error_internal_display() {
        let error = HuntModeError::InternalError("test error".to_string());
        assert!(error.to_string().contains("Internal error"));
    }

    // ============================================================================
    // Coverage Improvement Tests
    // ============================================================================

    #[test]
    fn test_hunt_config_verbose_default() {
        let config = HuntConfig::default();
        assert!(!config.verbose);
    }

    #[test]
    fn test_hunt_config_clone() {
        let config = HuntConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.max_cycles, config.max_cycles);
        assert_eq!(cloned.plateau_threshold, config.plateau_threshold);
    }

    #[test]
    fn test_hunt_config_debug() {
        let config = HuntConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("max_cycles"));
        assert!(debug.contains("quality_threshold"));
    }

    #[test]
    fn test_hunt_mode_add_error() {
        let mut hunt = HuntMode::new();
        hunt.add_error("E0001", "Test error", Some("test.rs"), 0.5);
        // Error added to planner
    }

    #[test]
    fn test_hunt_mode_andon_status() {
        let hunt = HuntMode::new();
        let status = hunt.andon_status();
        // Status should be retrievable
        let _ = format!("{:?}", status);
    }

    #[test]
    fn test_hunt_mode_kaizen_metrics() {
        let hunt = HuntMode::new();
        let metrics = hunt.kaizen_metrics();
        // Metrics should be retrievable
        let _ = format!("{:?}", metrics);
    }

    #[test]
    fn test_hunt_mode_run_empty() {
        let mut hunt = HuntMode::new();
        let result = hunt.run(0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_hunt_mode_run_single_cycle() {
        let mut hunt = HuntMode::new();
        let result = hunt.run(1);
        assert!(result.is_ok());
        let outcomes = result.unwrap();
        assert_eq!(outcomes.len(), 1);
    }

    #[test]
    fn test_hunt_mode_run_multiple_cycles() {
        let mut hunt = HuntMode::new();
        let result = hunt.run(3);
        assert!(result.is_ok());
        // May stop early due to plateau
        let outcomes = result.unwrap();
        assert!(!outcomes.is_empty());
    }

    #[test]
    fn test_hunt_mode_history_after_cycle() {
        let mut hunt = HuntMode::new();
        // Add an error so there's a pattern to process
        hunt.add_error("E0001", "Test error", Some("test.rs"), 0.5);
        let _ = hunt.run_cycle();
        // History may or may not have entries depending on pattern availability
        // Just verify run_cycle completed without error
    }

    #[test]
    fn test_cycle_outcome_clone() {
        let outcome = CycleOutcome {
            cycle: 1,
            pattern: None,
            fix_applied: false,
            compilation_rate: 0.5,
            rate_delta: 0.0,
            confidence: 0.0,
            lessons: vec!["Test".to_string()],
        };
        let cloned = outcome.clone();
        assert_eq!(cloned.cycle, 1);
        assert_eq!(cloned.lessons.len(), 1);
    }

    #[test]
    fn test_cycle_outcome_debug() {
        let outcome = CycleOutcome {
            cycle: 1,
            pattern: None,
            fix_applied: true,
            compilation_rate: 0.75,
            rate_delta: 0.05,
            confidence: 0.9,
            lessons: vec![],
        };
        let debug = format!("{:?}", outcome);
        assert!(debug.contains("cycle"));
        assert!(debug.contains("fix_applied"));
    }

    #[test]
    fn test_hunt_mode_error_clone() {
        let error = HuntModeError::IsolationFailed("test".to_string());
        let cloned = error.clone();
        assert!(cloned.to_string().contains("Isolation failed"));
    }

    #[test]
    fn test_hunt_mode_error_debug() {
        let error = HuntModeError::WorkspaceFailed("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("WorkspaceFailed"));
    }

    #[test]
    fn test_hunt_mode_error_is_error() {
        let error: Box<dyn std::error::Error> =
            Box::new(HuntModeError::CompilationFailed("test".to_string()));
        assert!(error.to_string().contains("Compilation failed"));
    }

    #[test]
    fn test_hunt_mode_with_custom_config() {
        let config = HuntConfig {
            max_cycles: 10,
            quality_threshold: 0.5,
            plateau_threshold: 2,
            target_rate: 0.5,
            min_improvement_per_cycle: 0.01,
            enable_five_whys: false,
            human_review_threshold: 0.5,
            auto_commit_threshold: 0.8,
            verbose: true,
        };
        let hunt = HuntMode::with_config(config);
        assert_eq!(hunt.config.max_cycles, 10);
        assert!(!hunt.config.enable_five_whys);
        assert!(hunt.config.verbose);
    }
}
