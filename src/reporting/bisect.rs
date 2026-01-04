//! Delta Debugging / Bisection Mode [11]
//!
//! Applies binary search (ddmin algorithm) to isolate minimal
//! failing corpus, following Zeller & Hildebrandt (2002).
//!
//! # Academic Foundation
//! - [11] Zeller & Hildebrandt (2002). Simplifying and isolating
//!   failure-inducing input. IEEE TSE 28(2).

use std::fmt;

/// Result of testing a subset during delta debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    /// Subset reproduces the failure
    Fail,
    /// Subset passes (no failure)
    Pass,
    /// Test inconclusive (e.g., doesn't compile)
    Unresolved,
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fail => write!(f, "FAIL"),
            Self::Pass => write!(f, "PASS"),
            Self::Unresolved => write!(f, "UNRESOLVED"),
        }
    }
}

/// Bisection step for tracking progress
#[derive(Debug, Clone)]
pub struct BisectStep {
    /// Step number (1-indexed)
    pub number: usize,
    /// Indices being tested
    pub indices: Vec<usize>,
    /// Size of subset being tested
    pub subset_size: usize,
    /// Total corpus size
    pub total_size: usize,
    /// Result of this step
    pub result: Option<TestResult>,
    /// Description of what's being tested
    pub description: String,
}

impl BisectStep {
    /// Create new bisect step
    #[must_use]
    pub fn new(number: usize, indices: Vec<usize>, total_size: usize) -> Self {
        let subset_size = indices.len();
        Self {
            number,
            indices,
            subset_size,
            total_size,
            result: None,
            description: String::new(),
        }
    }

    /// Set result
    pub fn with_result(mut self, result: TestResult) -> Self {
        self.result = Some(result);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Get progress as percentage
    #[must_use]
    pub fn progress_percent(&self) -> f64 {
        if self.total_size == 0 {
            100.0
        } else {
            let reduced = self.total_size - self.subset_size;
            (reduced as f64 / self.total_size as f64) * 100.0
        }
    }

    /// Get reduction ratio
    #[must_use]
    pub fn reduction_ratio(&self) -> f64 {
        if self.total_size == 0 {
            1.0
        } else {
            self.subset_size as f64 / self.total_size as f64
        }
    }
}

/// Bisection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BisectState {
    /// Still searching
    InProgress,
    /// Found minimal failing set
    Found,
    /// Could not reduce further
    Minimal,
    /// No failure found in initial set
    NoFailure,
    /// Bisection canceled
    Canceled,
}

impl fmt::Display for BisectState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InProgress => write!(f, "IN PROGRESS"),
            Self::Found => write!(f, "FOUND"),
            Self::Minimal => write!(f, "MINIMAL"),
            Self::NoFailure => write!(f, "NO FAILURE"),
            Self::Canceled => write!(f, "CANCELED"),
        }
    }
}

/// Delta debugging session for bisecting failures
#[derive(Debug, Clone)]
pub struct DeltaDebugger<T> {
    /// Original corpus items
    items: Vec<T>,
    /// Current failing subset indices
    failing_indices: Vec<usize>,
    /// Bisection steps taken
    steps: Vec<BisectStep>,
    /// Current state
    state: BisectState,
    /// Granularity (n in ddmin)
    granularity: usize,
    /// Maximum steps allowed
    max_steps: usize,
}

impl<T: Clone> DeltaDebugger<T> {
    /// Create new delta debugger with corpus
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        let len = items.len();
        Self {
            items,
            failing_indices: (0..len).collect(),
            steps: Vec::new(),
            state: BisectState::InProgress,
            granularity: 2,
            max_steps: 100,
        }
    }

    /// Set maximum steps
    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_steps = max;
        self
    }

    /// Get current state
    #[must_use]
    pub fn state(&self) -> BisectState {
        self.state
    }

    /// Get number of steps taken
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get all steps
    #[must_use]
    pub fn steps(&self) -> &[BisectStep] {
        &self.steps
    }

    /// Get current subset size
    #[must_use]
    pub fn current_size(&self) -> usize {
        self.failing_indices.len()
    }

    /// Get original corpus size
    #[must_use]
    pub fn original_size(&self) -> usize {
        self.items.len()
    }

    /// Get current failing subset
    #[must_use]
    pub fn current_subset(&self) -> Vec<&T> {
        self.failing_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    /// Get current failing indices
    #[must_use]
    pub fn failing_indices(&self) -> &[usize] {
        &self.failing_indices
    }

    /// Get reduction achieved
    #[must_use]
    pub fn reduction_percent(&self) -> f64 {
        if self.items.is_empty() {
            0.0
        } else {
            let reduced = self.items.len() - self.failing_indices.len();
            (reduced as f64 / self.items.len() as f64) * 100.0
        }
    }

    /// Check if done
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.state != BisectState::InProgress
    }

    /// Generate next subset to test (ddmin algorithm)
    ///
    /// Returns None if bisection is complete
    #[must_use]
    pub fn next_subset(&self) -> Option<Vec<usize>> {
        if self.is_done() {
            return None;
        }

        if self.steps.len() >= self.max_steps {
            return None;
        }

        let n = self.failing_indices.len();
        if n <= 1 {
            return None;
        }

        // Split into granularity parts, return first non-empty chunk
        let chunk_size = n.div_ceil(self.granularity);
        if chunk_size == 0 {
            return None;
        }

        // Return first chunk to test
        let subset: Vec<usize> = self
            .failing_indices
            .iter()
            .take(chunk_size)
            .copied()
            .collect();

        if subset.is_empty() {
            None
        } else {
            Some(subset)
        }
    }

    /// Generate complement subset (everything except the test subset)
    #[must_use]
    pub fn complement_of(&self, subset: &[usize]) -> Vec<usize> {
        self.failing_indices
            .iter()
            .filter(|&i| !subset.contains(i))
            .copied()
            .collect()
    }

    /// Record test result and update state
    pub fn record_result(&mut self, tested_indices: Vec<usize>, result: TestResult) {
        let step_num = self.steps.len() + 1;
        let step =
            BisectStep::new(step_num, tested_indices.clone(), self.items.len()).with_result(result);
        self.steps.push(step);

        match result {
            TestResult::Fail => {
                // Subset still fails - reduce to this subset
                if tested_indices.len() < self.failing_indices.len() {
                    self.failing_indices = tested_indices;
                    self.granularity = 2; // Reset granularity
                }
            }
            TestResult::Pass => {
                // Subset passes - try complement or increase granularity
                if self.granularity < self.failing_indices.len() {
                    self.granularity = (self.granularity * 2).min(self.failing_indices.len());
                }
            }
            TestResult::Unresolved => {
                // Inconclusive - increase granularity
                if self.granularity < self.failing_indices.len() {
                    self.granularity += 1;
                }
            }
        }

        // Check termination
        if self.failing_indices.len() <= 1 {
            self.state = BisectState::Minimal;
        } else if self.steps.len() >= self.max_steps {
            self.state = BisectState::Minimal;
        } else if self.granularity >= self.failing_indices.len() && result != TestResult::Fail {
            // Can't reduce further
            self.state = BisectState::Minimal;
        }
    }

    /// Mark as found (minimal failing set identified)
    pub fn mark_found(&mut self) {
        self.state = BisectState::Found;
    }

    /// Mark as no failure (initial test passed)
    pub fn mark_no_failure(&mut self) {
        self.state = BisectState::NoFailure;
    }

    /// Cancel bisection
    pub fn cancel(&mut self) {
        self.state = BisectState::Canceled;
    }

    /// Render bisection progress as ASCII
    #[must_use]
    pub fn render(&self, width: usize) -> String {
        let mut lines = vec![format!(
            "{}╭{}╮",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        )];

        lines.push(format!(
            "{}│ {:^width$} │",
            " ".repeat(2),
            "DELTA DEBUGGING (ddmin)",
            width = width.saturating_sub(6)
        ));

        lines.push(format!(
            "{}├{}┤",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        // State summary
        lines.push(format!(
            "{}│ State: {:width$} │",
            " ".repeat(2),
            self.state.to_string(),
            width = width.saturating_sub(10)
        ));

        lines.push(format!(
            "{}│ Steps: {}/{:width$} │",
            " ".repeat(2),
            self.steps.len(),
            self.max_steps,
            width = width.saturating_sub(12)
        ));

        lines.push(format!(
            "{}│ Size:  {} → {} ({:.1}% reduced){:width$} │",
            " ".repeat(2),
            self.original_size(),
            self.current_size(),
            self.reduction_percent(),
            "",
            width = width.saturating_sub(35)
        ));

        // Recent steps
        if !self.steps.is_empty() {
            lines.push(format!(
                "{}├{}┤",
                " ".repeat(2),
                "─".repeat(width.saturating_sub(4))
            ));

            for step in self.steps.iter().rev().take(5) {
                let result_str = step.result.map_or("?".to_string(), |r| r.to_string());
                let icon = match step.result {
                    Some(TestResult::Fail) => "✗",
                    Some(TestResult::Pass) => "✓",
                    Some(TestResult::Unresolved) => "?",
                    None => "○",
                };
                lines.push(format!(
                    "{}│ {} Step {:2}: {} (n={}){:width$} │",
                    " ".repeat(2),
                    icon,
                    step.number,
                    result_str,
                    step.subset_size,
                    "",
                    width = width.saturating_sub(28)
                ));
            }
        }

        lines.push(format!(
            "{}╰{}╯",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        lines.join("\n")
    }
}

/// Git-style bisect interface for transpilation failures
#[derive(Debug, Clone)]
pub struct BisectSession {
    /// Session identifier
    pub id: String,
    /// File paths being bisected
    pub files: Vec<String>,
    /// Current bisection range (start, end)
    pub range: (usize, usize),
    /// Confirmed good indices
    pub good: Vec<usize>,
    /// Confirmed bad indices
    pub bad: Vec<usize>,
    /// Steps taken
    pub steps: Vec<BisectStep>,
    /// Current state
    pub state: BisectState,
}

impl BisectSession {
    /// Create new bisect session
    #[must_use]
    pub fn new(files: Vec<String>) -> Self {
        let end = files.len().saturating_sub(1);
        Self {
            id: format!(
                "bisect-{}",
                std::time::UNIX_EPOCH
                    .elapsed()
                    .unwrap_or_default()
                    .as_secs()
            ),
            files,
            range: (0, end),
            good: Vec::new(),
            bad: Vec::new(),
            steps: Vec::new(),
            state: BisectState::InProgress,
        }
    }

    /// Get current midpoint to test
    #[must_use]
    pub fn midpoint(&self) -> Option<usize> {
        if self.range.0 >= self.range.1 {
            None
        } else {
            Some(usize::midpoint(self.range.0, self.range.1))
        }
    }

    /// Mark index as good (passes)
    pub fn mark_good(&mut self, index: usize) {
        self.good.push(index);
        if index >= self.range.0 && index < self.range.1 {
            self.range.0 = index + 1;
        }
        self.update_state();
    }

    /// Mark index as bad (fails)
    pub fn mark_bad(&mut self, index: usize) {
        self.bad.push(index);
        if index >= self.range.0 && index <= self.range.1 {
            self.range.1 = index;
        }
        self.update_state();
    }

    /// Update state based on range
    fn update_state(&mut self) {
        if self.range.0 >= self.range.1 {
            self.state = BisectState::Found;
        }
    }

    /// Get steps remaining (log2 estimate)
    #[must_use]
    pub fn steps_remaining(&self) -> usize {
        let range_size = self.range.1.saturating_sub(self.range.0);
        if range_size <= 1 {
            0
        } else {
            (range_size as f64).log2().ceil() as usize
        }
    }

    /// Get current file being tested
    #[must_use]
    pub fn current_file(&self) -> Option<&str> {
        self.midpoint()
            .and_then(|i| self.files.get(i).map(String::as_str))
    }

    /// Get first bad file (result)
    #[must_use]
    pub fn first_bad(&self) -> Option<&str> {
        if self.state == BisectState::Found {
            self.files.get(self.range.0).map(String::as_str)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: TestResult Tests
    // ============================================================

    #[test]
    fn test_test_result_display() {
        assert_eq!(TestResult::Fail.to_string(), "FAIL");
        assert_eq!(TestResult::Pass.to_string(), "PASS");
        assert_eq!(TestResult::Unresolved.to_string(), "UNRESOLVED");
    }

    // ============================================================
    // EXTREME TDD: BisectStep Tests
    // ============================================================

    #[test]
    fn test_bisect_step_new() {
        let step = BisectStep::new(1, vec![0, 1, 2], 10);
        assert_eq!(step.number, 1);
        assert_eq!(step.subset_size, 3);
        assert_eq!(step.total_size, 10);
        assert!(step.result.is_none());
    }

    #[test]
    fn test_bisect_step_with_result() {
        let step = BisectStep::new(1, vec![0, 1], 10).with_result(TestResult::Fail);
        assert_eq!(step.result, Some(TestResult::Fail));
    }

    #[test]
    fn test_bisect_step_progress() {
        let step = BisectStep::new(1, vec![0, 1], 10); // 2 of 10 = 80% reduced
        assert!((step.progress_percent() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_bisect_step_reduction_ratio() {
        let step = BisectStep::new(1, vec![0, 1, 2], 10); // 3/10 = 0.3
        assert!((step.reduction_ratio() - 0.3).abs() < 0.01);
    }

    // ============================================================
    // EXTREME TDD: BisectState Tests
    // ============================================================

    #[test]
    fn test_bisect_state_display() {
        assert_eq!(BisectState::InProgress.to_string(), "IN PROGRESS");
        assert_eq!(BisectState::Found.to_string(), "FOUND");
        assert_eq!(BisectState::Minimal.to_string(), "MINIMAL");
        assert_eq!(BisectState::NoFailure.to_string(), "NO FAILURE");
        assert_eq!(BisectState::Canceled.to_string(), "CANCELED");
    }

    // ============================================================
    // EXTREME TDD: DeltaDebugger Tests
    // ============================================================

    #[test]
    fn test_delta_debugger_new() {
        let items = vec!["a", "b", "c", "d", "e"];
        let dd = DeltaDebugger::new(items);
        assert_eq!(dd.original_size(), 5);
        assert_eq!(dd.current_size(), 5);
        assert_eq!(dd.state(), BisectState::InProgress);
    }

    #[test]
    fn test_delta_debugger_next_subset() {
        let items = vec!["a", "b", "c", "d"];
        let dd = DeltaDebugger::new(items);

        let subset = dd.next_subset().unwrap();
        assert_eq!(subset.len(), 2); // Half of 4
    }

    #[test]
    fn test_delta_debugger_complement() {
        let items = vec!["a", "b", "c", "d"];
        let dd = DeltaDebugger::new(items);

        let subset = vec![0, 1];
        let complement = dd.complement_of(&subset);
        assert_eq!(complement, vec![2, 3]);
    }

    #[test]
    fn test_delta_debugger_record_fail() {
        let items = vec!["a", "b", "c", "d"];
        let mut dd = DeltaDebugger::new(items);

        // Record that subset [0, 1] still fails
        dd.record_result(vec![0, 1], TestResult::Fail);

        assert_eq!(dd.step_count(), 1);
        assert_eq!(dd.current_size(), 2); // Reduced to failing subset
    }

    #[test]
    fn test_delta_debugger_record_pass() {
        let items = vec!["a", "b", "c", "d"];
        let mut dd = DeltaDebugger::new(items);

        // Record that subset [0, 1] passes (doesn't reproduce failure)
        dd.record_result(vec![0, 1], TestResult::Pass);

        assert_eq!(dd.step_count(), 1);
        assert_eq!(dd.current_size(), 4); // Not reduced
    }

    #[test]
    fn test_delta_debugger_reduction_percent() {
        let items = vec!["a", "b", "c", "d", "e"];
        let mut dd = DeltaDebugger::new(items);

        // Reduce to 2 items
        dd.record_result(vec![0, 1], TestResult::Fail);

        assert!((dd.reduction_percent() - 60.0).abs() < 0.01); // 3/5 = 60%
    }

    #[test]
    fn test_delta_debugger_minimal_state() {
        let items = vec!["a", "b"];
        let mut dd = DeltaDebugger::new(items);

        // Reduce to 1 item
        dd.record_result(vec![0], TestResult::Fail);

        assert_eq!(dd.state(), BisectState::Minimal);
    }

    #[test]
    fn test_delta_debugger_current_subset() {
        let items = vec!["a", "b", "c", "d"];
        let mut dd = DeltaDebugger::new(items);

        dd.record_result(vec![1, 2], TestResult::Fail);

        let subset = dd.current_subset();
        assert_eq!(subset, vec![&"b", &"c"]);
    }

    #[test]
    fn test_delta_debugger_max_steps() {
        let items = vec!["a", "b", "c"];
        let dd = DeltaDebugger::new(items).with_max_steps(5);

        assert!(!dd.is_done());
    }

    #[test]
    fn test_delta_debugger_cancel() {
        let items = vec!["a", "b", "c"];
        let mut dd = DeltaDebugger::new(items);

        dd.cancel();
        assert_eq!(dd.state(), BisectState::Canceled);
        assert!(dd.is_done());
    }

    #[test]
    fn test_delta_debugger_render() {
        let items = vec!["a", "b", "c", "d"];
        let mut dd = DeltaDebugger::new(items);
        dd.record_result(vec![0, 1], TestResult::Fail);

        let output = dd.render(50);
        assert!(output.contains("DELTA DEBUGGING"));
        assert!(output.contains("Step"));
    }

    // ============================================================
    // EXTREME TDD: BisectSession Tests (Git-style)
    // ============================================================

    #[test]
    fn test_bisect_session_new() {
        let files = vec![
            "a.ruchy".to_string(),
            "b.ruchy".to_string(),
            "c.ruchy".to_string(),
        ];
        let session = BisectSession::new(files);

        assert_eq!(session.range, (0, 2));
        assert!(session.id.starts_with("bisect-"));
    }

    #[test]
    fn test_bisect_session_midpoint() {
        let files = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let session = BisectSession::new(files);

        assert_eq!(session.midpoint(), Some(2)); // (0+4)/2 = 2
    }

    #[test]
    fn test_bisect_session_mark_good() {
        let files = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let mut session = BisectSession::new(files);

        session.mark_good(1);
        assert_eq!(session.range.0, 2); // Start moved up
    }

    #[test]
    fn test_bisect_session_mark_bad() {
        let files = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let mut session = BisectSession::new(files);

        session.mark_bad(2);
        assert_eq!(session.range.1, 2); // End moved down
    }

    #[test]
    fn test_bisect_session_found() {
        let files = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut session = BisectSession::new(files);

        session.mark_good(0);
        session.mark_bad(1);

        assert_eq!(session.state, BisectState::Found);
        assert_eq!(session.first_bad(), Some("b"));
    }

    #[test]
    fn test_bisect_session_steps_remaining() {
        let files: Vec<String> = (0..16).map(|i| format!("{i}.ruchy")).collect();
        let session = BisectSession::new(files);

        // log2(15) ≈ 4
        assert!(session.steps_remaining() <= 4);
    }

    #[test]
    fn test_bisect_session_current_file() {
        let files = vec![
            "a.ruchy".to_string(),
            "b.ruchy".to_string(),
            "c.ruchy".to_string(),
        ];
        let session = BisectSession::new(files);

        assert_eq!(session.current_file(), Some("b.ruchy"));
    }
}
