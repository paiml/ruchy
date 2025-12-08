//! Convergence Dashboard (Jidoka) [6]
//!
//! Tracks iterative fix attempts during auto-fix mode, visualizing
//! progress toward successful transpilation.

use crate::reporting::ascii::{boxed_header, detect_trend, progress_bar, sparkline, TrendDirection};

/// Fix attempt result
#[derive(Debug, Clone)]
pub struct FixAttempt {
    /// Fix description
    pub description: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Whether the fix was applied
    pub applied: bool,
    /// Whether the fix succeeded (reduced errors)
    pub succeeded: bool,
}

impl FixAttempt {
    /// Create new fix attempt
    #[must_use]
    pub fn new(description: impl Into<String>, confidence: f64) -> Self {
        Self {
            description: description.into(),
            confidence,
            applied: false,
            succeeded: false,
        }
    }

    /// Mark as applied
    pub fn apply(&mut self) -> &mut Self {
        self.applied = true;
        self
    }

    /// Mark as succeeded
    pub fn succeed(&mut self) -> &mut Self {
        self.succeeded = true;
        self
    }

    /// Get status icon
    #[must_use]
    pub fn status_icon(&self) -> &'static str {
        if !self.applied {
            "○" // Not applied
        } else if self.succeeded {
            "✓" // Applied and succeeded
        } else {
            "✗" // Applied but failed
        }
    }
}

/// Single iteration in convergence loop
#[derive(Debug, Clone)]
pub struct ConvergenceIteration {
    /// Iteration number (1-indexed)
    pub number: usize,
    /// Error count before this iteration
    pub errors_before: usize,
    /// Error count after this iteration
    pub errors_after: usize,
    /// Fixes attempted in this iteration
    pub fixes: Vec<FixAttempt>,
}

impl ConvergenceIteration {
    /// Create new iteration
    #[must_use]
    pub fn new(number: usize, errors_before: usize) -> Self {
        Self {
            number,
            errors_before,
            errors_after: errors_before,
            fixes: Vec::new(),
        }
    }

    /// Add a fix attempt
    pub fn add_fix(&mut self, fix: FixAttempt) {
        self.fixes.push(fix);
    }

    /// Set errors after iteration
    pub fn set_errors_after(&mut self, count: usize) {
        self.errors_after = count;
    }

    /// Calculate error delta (negative = improvement)
    #[must_use]
    pub fn error_delta(&self) -> i32 {
        self.errors_after as i32 - self.errors_before as i32
    }

    /// Check if this iteration improved the situation
    #[must_use]
    pub fn is_improving(&self) -> bool {
        self.errors_after < self.errors_before
    }

    /// Get direction indicator
    #[must_use]
    pub fn direction_indicator(&self) -> &'static str {
        match self.errors_after.cmp(&self.errors_before) {
            std::cmp::Ordering::Less => "▼",    // Improved
            std::cmp::Ordering::Greater => "▲", // Degraded
            std::cmp::Ordering::Equal => "→",   // No change
        }
    }
}

/// Convergence state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceState {
    /// Still running iterations
    InProgress,
    /// Successfully converged to zero errors
    Converged,
    /// Stuck in oscillation (errors flip-flopping)
    Oscillating,
    /// Hit maximum iterations without convergence
    MaxIterations,
    /// Errors increased overall (diverging)
    Diverging,
}

impl ConvergenceState {
    /// Get display label
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::InProgress => "IN PROGRESS",
            Self::Converged => "CONVERGED",
            Self::Oscillating => "OSCILLATING",
            Self::MaxIterations => "MAX ITERATIONS",
            Self::Diverging => "DIVERGING",
        }
    }

    /// Get status icon
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::InProgress => "◉",
            Self::Converged => "✓",
            Self::Oscillating => "↔",
            Self::MaxIterations => "⚠",
            Self::Diverging => "✗",
        }
    }
}

/// Convergence dashboard for tracking iterative fixes
#[derive(Debug, Clone)]
pub struct ConvergenceDashboard {
    /// Initial error count
    pub initial_errors: usize,
    /// Maximum iterations allowed
    pub max_iterations: usize,
    /// All iterations
    pub iterations: Vec<ConvergenceIteration>,
    /// Current state
    pub state: ConvergenceState,
    /// Confidence threshold for auto-applying fixes
    pub confidence_threshold: f64,
}

impl Default for ConvergenceDashboard {
    fn default() -> Self {
        Self::new(0, 10)
    }
}

impl ConvergenceDashboard {
    /// Create new dashboard
    #[must_use]
    pub fn new(initial_errors: usize, max_iterations: usize) -> Self {
        Self {
            initial_errors,
            max_iterations,
            iterations: Vec::new(),
            state: ConvergenceState::InProgress,
            confidence_threshold: 0.85,
        }
    }

    /// Set confidence threshold for auto-applying fixes
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Start a new iteration
    pub fn start_iteration(&mut self) -> &mut ConvergenceIteration {
        let number = self.iterations.len() + 1;
        let errors_before = self
            .iterations
            .last()
            .map_or(self.initial_errors, |it| it.errors_after);

        self.iterations
            .push(ConvergenceIteration::new(number, errors_before));
        self.iterations.last_mut().unwrap()
    }

    /// Complete current iteration with error count
    pub fn complete_iteration(&mut self, errors_after: usize) {
        if let Some(iteration) = self.iterations.last_mut() {
            iteration.set_errors_after(errors_after);
        }
        self.update_state();
    }

    /// Update convergence state based on iterations
    fn update_state(&mut self) {
        let current_errors = self.current_errors();

        // Check for convergence (zero errors)
        if current_errors == 0 {
            self.state = ConvergenceState::Converged;
            return;
        }

        // Check for max iterations
        if self.iterations.len() >= self.max_iterations {
            self.state = ConvergenceState::MaxIterations;
            return;
        }

        // Check for divergence (errors increased overall)
        if current_errors > self.initial_errors {
            self.state = ConvergenceState::Diverging;
            return;
        }

        // Check for oscillation
        if self.iterations.len() >= 3 {
            let error_history: Vec<f64> = self
                .iterations
                .iter()
                .map(|it| it.errors_after as f64)
                .collect();
            if detect_trend(&error_history) == TrendDirection::Oscillating {
                self.state = ConvergenceState::Oscillating;
                return;
            }
        }

        self.state = ConvergenceState::InProgress;
    }

    /// Get current error count
    #[must_use]
    pub fn current_errors(&self) -> usize {
        self.iterations
            .last()
            .map_or(self.initial_errors, |it| it.errors_after)
    }

    /// Get error timeline as vector
    #[must_use]
    pub fn error_timeline(&self) -> Vec<usize> {
        let mut timeline = vec![self.initial_errors];
        for it in &self.iterations {
            timeline.push(it.errors_after);
        }
        timeline
    }

    /// Check if converged
    #[must_use]
    pub fn is_converged(&self) -> bool {
        self.state == ConvergenceState::Converged
    }

    /// Check if should continue iterating
    #[must_use]
    pub fn should_continue(&self) -> bool {
        self.state == ConvergenceState::InProgress
    }

    /// Render dashboard as ASCII
    #[must_use]
    pub fn render(&self, width: usize) -> String {
        let mut lines = vec![boxed_header("CONVERGENCE LOOP PROGRESS", width)];

        // State summary
        lines.push(format!(
            "\n{} State: {} ({})",
            self.state.icon(),
            self.state.label(),
            if self.is_converged() {
                format!("{} errors after {} iterations", self.current_errors(), self.iterations.len())
            } else {
                format!("{} errors remaining", self.current_errors())
            }
        ));

        // Timeline sparkline
        let timeline: Vec<f64> = self.error_timeline().iter().map(|&e| e as f64).collect();
        if !timeline.is_empty() {
            let trend = detect_trend(&timeline);
            lines.push(format!(
                "\nTimeline: {} → {} → {}",
                self.initial_errors,
                self.current_errors(),
                if self.is_converged() { "✓" } else { "..." }
            ));
            lines.push(format!("Trend:    {} ({})", sparkline(&timeline), trend.label()));
        }

        // Progress bar
        if self.initial_errors > 0 {
            let fixed = self.initial_errors.saturating_sub(self.current_errors());
            let bar = progress_bar(fixed, self.initial_errors, 40);
            lines.push(format!("\nFixed:    {bar}"));
        }

        // Iterations detail
        if !self.iterations.is_empty() {
            lines.push(format!("\n{}Iterations{}", "─".repeat(3), "─".repeat(width - 13)));
        }

        for iteration in &self.iterations {
            lines.push(format!(
                "\n◈ Iteration {}/{}",
                iteration.number, self.max_iterations
            ));
            lines.push(format!(
                "  Errors: {} → {} ({}{}) {}",
                iteration.errors_before,
                iteration.errors_after,
                if iteration.error_delta() >= 0 { "+" } else { "" },
                iteration.error_delta(),
                iteration.direction_indicator()
            ));

            // Fixes in this iteration
            for fix in &iteration.fixes {
                lines.push(format!(
                    "  {} {} [confidence: {:.2}]",
                    fix.status_icon(),
                    fix.description,
                    fix.confidence
                ));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: FixAttempt Tests
    // ============================================================

    #[test]
    fn test_fix_attempt_new() {
        let fix = FixAttempt::new("Add Clone derive", 0.92);
        assert_eq!(fix.description, "Add Clone derive");
        assert!((fix.confidence - 0.92).abs() < 0.01);
        assert!(!fix.applied);
        assert!(!fix.succeeded);
    }

    #[test]
    fn test_fix_attempt_apply() {
        let mut fix = FixAttempt::new("Fix type", 0.85);
        fix.apply();
        assert!(fix.applied);
    }

    #[test]
    fn test_fix_attempt_succeed() {
        let mut fix = FixAttempt::new("Fix type", 0.85);
        fix.apply().succeed();
        assert!(fix.applied);
        assert!(fix.succeeded);
    }

    #[test]
    fn test_fix_attempt_status_icon() {
        let fix1 = FixAttempt::new("Test", 0.9);
        assert_eq!(fix1.status_icon(), "○");

        let mut fix2 = FixAttempt::new("Test", 0.9);
        fix2.apply();
        assert_eq!(fix2.status_icon(), "✗");

        let mut fix3 = FixAttempt::new("Test", 0.9);
        fix3.apply().succeed();
        assert_eq!(fix3.status_icon(), "✓");
    }

    // ============================================================
    // EXTREME TDD: ConvergenceIteration Tests
    // ============================================================

    #[test]
    fn test_iteration_new() {
        let it = ConvergenceIteration::new(1, 10);
        assert_eq!(it.number, 1);
        assert_eq!(it.errors_before, 10);
        assert_eq!(it.errors_after, 10);
    }

    #[test]
    fn test_iteration_error_delta() {
        let mut it = ConvergenceIteration::new(1, 10);
        it.set_errors_after(7);
        assert_eq!(it.error_delta(), -3);

        let mut it2 = ConvergenceIteration::new(2, 7);
        it2.set_errors_after(9);
        assert_eq!(it2.error_delta(), 2);
    }

    #[test]
    fn test_iteration_is_improving() {
        let mut it = ConvergenceIteration::new(1, 10);
        it.set_errors_after(7);
        assert!(it.is_improving());

        let mut it2 = ConvergenceIteration::new(2, 7);
        it2.set_errors_after(7);
        assert!(!it2.is_improving());
    }

    #[test]
    fn test_iteration_direction_indicator() {
        let mut it1 = ConvergenceIteration::new(1, 10);
        it1.set_errors_after(7);
        assert_eq!(it1.direction_indicator(), "▼");

        let mut it2 = ConvergenceIteration::new(1, 10);
        it2.set_errors_after(12);
        assert_eq!(it2.direction_indicator(), "▲");

        let mut it3 = ConvergenceIteration::new(1, 10);
        it3.set_errors_after(10);
        assert_eq!(it3.direction_indicator(), "→");
    }

    // ============================================================
    // EXTREME TDD: ConvergenceState Tests
    // ============================================================

    #[test]
    fn test_convergence_state_labels() {
        assert_eq!(ConvergenceState::InProgress.label(), "IN PROGRESS");
        assert_eq!(ConvergenceState::Converged.label(), "CONVERGED");
        assert_eq!(ConvergenceState::Oscillating.label(), "OSCILLATING");
        assert_eq!(ConvergenceState::MaxIterations.label(), "MAX ITERATIONS");
        assert_eq!(ConvergenceState::Diverging.label(), "DIVERGING");
    }

    #[test]
    fn test_convergence_state_icons() {
        assert_eq!(ConvergenceState::Converged.icon(), "✓");
        assert_eq!(ConvergenceState::Diverging.icon(), "✗");
    }

    // ============================================================
    // EXTREME TDD: ConvergenceDashboard Tests
    // ============================================================

    #[test]
    fn test_dashboard_new() {
        let dashboard = ConvergenceDashboard::new(10, 5);
        assert_eq!(dashboard.initial_errors, 10);
        assert_eq!(dashboard.max_iterations, 5);
        assert_eq!(dashboard.state, ConvergenceState::InProgress);
    }

    #[test]
    fn test_dashboard_iteration_flow() {
        let mut dashboard = ConvergenceDashboard::new(10, 5);

        dashboard.start_iteration();
        dashboard.complete_iteration(7);

        assert_eq!(dashboard.iterations.len(), 1);
        assert_eq!(dashboard.current_errors(), 7);
    }

    #[test]
    fn test_dashboard_convergence() {
        let mut dashboard = ConvergenceDashboard::new(5, 10);

        dashboard.start_iteration();
        dashboard.complete_iteration(3);

        dashboard.start_iteration();
        dashboard.complete_iteration(1);

        dashboard.start_iteration();
        dashboard.complete_iteration(0);

        assert!(dashboard.is_converged());
        assert_eq!(dashboard.state, ConvergenceState::Converged);
    }

    #[test]
    fn test_dashboard_max_iterations() {
        let mut dashboard = ConvergenceDashboard::new(10, 3);

        for _ in 0..3 {
            dashboard.start_iteration();
            dashboard.complete_iteration(5); // Still have errors
        }

        assert_eq!(dashboard.state, ConvergenceState::MaxIterations);
    }

    #[test]
    fn test_dashboard_diverging() {
        let mut dashboard = ConvergenceDashboard::new(5, 10);

        dashboard.start_iteration();
        dashboard.complete_iteration(8); // Errors increased

        assert_eq!(dashboard.state, ConvergenceState::Diverging);
    }

    #[test]
    fn test_dashboard_error_timeline() {
        let mut dashboard = ConvergenceDashboard::new(10, 5);

        dashboard.start_iteration();
        dashboard.complete_iteration(7);

        dashboard.start_iteration();
        dashboard.complete_iteration(3);

        let timeline = dashboard.error_timeline();
        assert_eq!(timeline, vec![10, 7, 3]);
    }

    #[test]
    fn test_dashboard_render() {
        let mut dashboard = ConvergenceDashboard::new(10, 5);

        dashboard.start_iteration();
        dashboard.complete_iteration(5);

        let output = dashboard.render(70);
        assert!(output.contains("CONVERGENCE"));
        assert!(output.contains("Iteration"));
    }

    #[test]
    fn test_dashboard_should_continue() {
        let mut dashboard = ConvergenceDashboard::new(10, 5);
        assert!(dashboard.should_continue());

        dashboard.start_iteration();
        dashboard.complete_iteration(0);
        assert!(!dashboard.should_continue()); // Converged
    }
}
