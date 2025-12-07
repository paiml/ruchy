//! Andon Verifier: Fix Validation and Commit
//!
//! Implements the ACT phase of Hunt Mode's PDCA cycle.
//! Validates fixes and provides visual status feedback.
//!
//! # Toyota Way: Andon (Visual Control / Stop the Line)
//!
//! Real-time visibility into system state with immediate escalation on failure.
//!
//! # References
//! - [19] Baudin (2007). Working with Machines. Andon systems.
//! - [5] Ohno (1988). TPS. Visual management.

use super::isolator::ReproCase;
use super::repair::Fix;

/// Andon status for visual management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndonStatus {
    /// All clear - system operating normally
    Green {
        /// Current compilation rate
        message: String,
    },

    /// Warning - needs attention
    Yellow {
        /// Warning messages
        warnings: Vec<String>,
    },

    /// Critical - stop the line
    Red {
        /// Error message
        error: String,
        /// Whether cycle was halted
        cycle_halted: bool,
    },
}

impl Default for AndonStatus {
    fn default() -> Self {
        Self::Green {
            message: "System ready".to_string(),
        }
    }
}

impl AndonStatus {
    /// Create green status
    #[must_use]
    pub fn green(message: impl Into<String>) -> Self {
        Self::Green {
            message: message.into(),
        }
    }

    /// Create yellow status
    #[must_use]
    pub fn yellow(warnings: Vec<String>) -> Self {
        Self::Yellow { warnings }
    }

    /// Create red status
    #[must_use]
    pub fn red(error: impl Into<String>, cycle_halted: bool) -> Self {
        Self::Red {
            error: error.into(),
            cycle_halted,
        }
    }

    /// Check if status is green
    #[must_use]
    pub fn is_green(&self) -> bool {
        matches!(self, Self::Green { .. })
    }

    /// Check if status is yellow
    #[must_use]
    pub fn is_yellow(&self) -> bool {
        matches!(self, Self::Yellow { .. })
    }

    /// Check if status is red
    #[must_use]
    pub fn is_red(&self) -> bool {
        matches!(self, Self::Red { .. })
    }

    /// Get status icon
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Green { .. } => "🟢",
            Self::Yellow { .. } => "🟡",
            Self::Red { .. } => "🔴",
        }
    }

    /// Get status name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Green { .. } => "GREEN",
            Self::Yellow { .. } => "YELLOW",
            Self::Red { .. } => "RED",
        }
    }
}

/// Result of verification attempt
#[derive(Debug, Clone)]
pub enum VerifyResult {
    /// Fix verified and committed
    Success,

    /// Fix needs manual review
    NeedsReview,

    /// Fix failed verification
    FixFailed(String),
}

impl VerifyResult {
    /// Check if verification succeeded
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if needs review
    #[must_use]
    pub fn needs_review(&self) -> bool {
        matches!(self, Self::NeedsReview)
    }

    /// Check if failed
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::FixFailed(_))
    }
}

/// Andon Verifier for fix validation
#[derive(Debug)]
pub struct AndonVerifier {
    /// Current Andon status
    status: AndonStatus,

    /// Verified fixes
    verified_fixes: Vec<String>,

    /// Failed fixes
    failed_fixes: Vec<String>,

    /// Total verification attempts
    total_attempts: u32,
}

impl Default for AndonVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl AndonVerifier {
    /// Create new verifier
    #[must_use]
    pub fn new() -> Self {
        Self {
            status: AndonStatus::default(),
            verified_fixes: Vec::new(),
            failed_fixes: Vec::new(),
            total_attempts: 0,
        }
    }

    /// Verify and commit a fix
    ///
    /// # Andon Protocol
    ///
    /// 1. Compile the fixed output
    /// 2. Run property tests (if applicable)
    /// 3. Update status based on results
    /// 4. Return verification result
    #[must_use]
    pub fn verify_and_commit(&mut self, fix: &Fix, repro: &ReproCase) -> VerifyResult {
        self.total_attempts += 1;

        // Check if fix has output
        if fix.rust_output.is_empty() {
            self.status = AndonStatus::red("Fix has no output", true);
            self.failed_fixes.push(fix.id.clone());
            return VerifyResult::FixFailed("Empty fix output".to_string());
        }

        // Check confidence threshold
        if fix.confidence < 0.70 {
            self.status = AndonStatus::yellow(vec![
                format!("Low confidence fix: {:.1}%", fix.confidence * 100.0),
                "Manual review recommended".to_string(),
            ]);
            return VerifyResult::NeedsReview;
        }

        // In real implementation, this would:
        // 1. Create ephemeral workspace
        // 2. Compile the fix
        // 3. Verify error is resolved
        // 4. Run property tests

        // Simulate verification
        let compile_success = self.simulate_compilation(&fix.rust_output, repro);

        if compile_success {
            self.status = AndonStatus::green(format!("Fix {} applied successfully", fix.id));
            self.verified_fixes.push(fix.id.clone());
            VerifyResult::Success
        } else {
            self.status = AndonStatus::red(
                format!("Fix {} failed compilation", fix.id),
                true,
            );
            self.failed_fixes.push(fix.id.clone());
            VerifyResult::FixFailed("Compilation failed".to_string())
        }
    }

    /// Simulate compilation (placeholder)
    fn simulate_compilation(&self, rust_output: &str, _repro: &ReproCase) -> bool {
        // In real implementation, use EphemeralWorkspace

        // Simple heuristic: check for basic Rust syntax
        !rust_output.is_empty()
            && !rust_output.contains("compile_error!")
            && (rust_output.contains("fn ")
                || rust_output.contains("pub ")
                || rust_output.contains("let ")
                || rust_output.contains("use ")
                || rust_output.contains("//"))
    }

    /// Get current Andon status
    #[must_use]
    pub fn status(&self) -> AndonStatus {
        self.status.clone()
    }

    /// Get verified fixes count
    #[must_use]
    pub fn verified_count(&self) -> usize {
        self.verified_fixes.len()
    }

    /// Get failed fixes count
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.failed_fixes.len()
    }

    /// Get total attempts
    #[must_use]
    pub fn total_attempts(&self) -> u32 {
        self.total_attempts
    }

    /// Get success rate
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.verified_fixes.len() as f64 / self.total_attempts as f64
        }
    }

    /// Reset verifier state
    pub fn reset(&mut self) {
        self.status = AndonStatus::default();
        self.verified_fixes.clear();
        self.failed_fixes.clear();
        self.total_attempts = 0;
    }

    /// Set status directly (for testing/manual override)
    pub fn set_status(&mut self, status: AndonStatus) {
        self.status = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: RED PHASE - AndonStatus Tests
    // ============================================================================

    #[test]
    fn test_andon_status_default() {
        let status = AndonStatus::default();
        assert!(status.is_green());
    }

    #[test]
    fn test_andon_status_green() {
        let status = AndonStatus::green("All good");
        assert!(status.is_green());
        assert!(!status.is_yellow());
        assert!(!status.is_red());
    }

    #[test]
    fn test_andon_status_yellow() {
        let status = AndonStatus::yellow(vec!["Warning".to_string()]);
        assert!(status.is_yellow());
        assert!(!status.is_green());
        assert!(!status.is_red());
    }

    #[test]
    fn test_andon_status_red() {
        let status = AndonStatus::red("Error", true);
        assert!(status.is_red());
        assert!(!status.is_green());
        assert!(!status.is_yellow());
    }

    #[test]
    fn test_andon_status_icon_green() {
        let status = AndonStatus::green("OK");
        assert_eq!(status.icon(), "🟢");
    }

    #[test]
    fn test_andon_status_icon_yellow() {
        let status = AndonStatus::yellow(vec![]);
        assert_eq!(status.icon(), "🟡");
    }

    #[test]
    fn test_andon_status_icon_red() {
        let status = AndonStatus::red("Error", false);
        assert_eq!(status.icon(), "🔴");
    }

    #[test]
    fn test_andon_status_name() {
        assert_eq!(AndonStatus::green("OK").name(), "GREEN");
        assert_eq!(AndonStatus::yellow(vec![]).name(), "YELLOW");
        assert_eq!(AndonStatus::red("Error", false).name(), "RED");
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - VerifyResult Tests
    // ============================================================================

    #[test]
    fn test_verify_result_success() {
        let result = VerifyResult::Success;
        assert!(result.is_success());
        assert!(!result.needs_review());
        assert!(!result.is_failed());
    }

    #[test]
    fn test_verify_result_needs_review() {
        let result = VerifyResult::NeedsReview;
        assert!(result.needs_review());
        assert!(!result.is_success());
        assert!(!result.is_failed());
    }

    #[test]
    fn test_verify_result_failed() {
        let result = VerifyResult::FixFailed("Error".to_string());
        assert!(result.is_failed());
        assert!(!result.is_success());
        assert!(!result.needs_review());
    }

    // ============================================================================
    // EXTREME TDD: RED PHASE - AndonVerifier Tests
    // ============================================================================

    #[test]
    fn test_andon_verifier_new() {
        let verifier = AndonVerifier::new();
        assert!(verifier.status().is_green());
        assert_eq!(verifier.total_attempts(), 0);
    }

    #[test]
    fn test_andon_verifier_default() {
        let verifier = AndonVerifier::default();
        assert!(verifier.status().is_green());
    }

    #[test]
    fn test_andon_verifier_verify_empty_fix() {
        let mut verifier = AndonVerifier::new();
        let fix = Fix::new("FIX-001", "PAT-001");
        let repro = ReproCase::new("fn test() {}", "E0308");

        let result = verifier.verify_and_commit(&fix, &repro);
        assert!(result.is_failed());
        assert!(verifier.status().is_red());
    }

    #[test]
    fn test_andon_verifier_verify_low_confidence() {
        let mut verifier = AndonVerifier::new();
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_rust_output("fn test() {}")
            .with_confidence(0.5);
        let repro = ReproCase::new("fn test() {}", "E0308");

        let result = verifier.verify_and_commit(&fix, &repro);
        assert!(result.needs_review());
        assert!(verifier.status().is_yellow());
    }

    #[test]
    fn test_andon_verifier_verify_success() {
        let mut verifier = AndonVerifier::new();
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_rust_output("fn test() {}")
            .with_confidence(0.9);
        let repro = ReproCase::new("fn test() {}", "E0308");

        let result = verifier.verify_and_commit(&fix, &repro);
        assert!(result.is_success());
        assert!(verifier.status().is_green());
    }

    #[test]
    fn test_andon_verifier_verified_count() {
        let mut verifier = AndonVerifier::new();
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_rust_output("fn test() {}")
            .with_confidence(0.9);
        let repro = ReproCase::new("fn test() {}", "E0308");

        let _ = verifier.verify_and_commit(&fix, &repro);
        assert_eq!(verifier.verified_count(), 1);
    }

    #[test]
    fn test_andon_verifier_failed_count() {
        let mut verifier = AndonVerifier::new();
        let fix = Fix::new("FIX-001", "PAT-001"); // Empty output
        let repro = ReproCase::new("fn test() {}", "E0308");

        let _ = verifier.verify_and_commit(&fix, &repro);
        assert_eq!(verifier.failed_count(), 1);
    }

    #[test]
    fn test_andon_verifier_success_rate() {
        let mut verifier = AndonVerifier::new();

        // Success
        let fix1 = Fix::new("FIX-001", "PAT-001")
            .with_rust_output("fn test() {}")
            .with_confidence(0.9);
        let repro1 = ReproCase::new("fn test() {}", "E0308");
        let _ = verifier.verify_and_commit(&fix1, &repro1);

        // Failure
        let fix2 = Fix::new("FIX-002", "PAT-002"); // Empty
        let repro2 = ReproCase::new("fn test() {}", "E0308");
        let _ = verifier.verify_and_commit(&fix2, &repro2);

        assert!((verifier.success_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_andon_verifier_reset() {
        let mut verifier = AndonVerifier::new();
        let fix = Fix::new("FIX-001", "PAT-001")
            .with_rust_output("fn test() {}")
            .with_confidence(0.9);
        let repro = ReproCase::new("fn test() {}", "E0308");

        let _ = verifier.verify_and_commit(&fix, &repro);
        verifier.reset();

        assert!(verifier.status().is_green());
        assert_eq!(verifier.total_attempts(), 0);
        assert_eq!(verifier.verified_count(), 0);
    }

    #[test]
    fn test_andon_verifier_set_status() {
        let mut verifier = AndonVerifier::new();
        verifier.set_status(AndonStatus::red("Manual override", false));
        assert!(verifier.status().is_red());
    }
}
