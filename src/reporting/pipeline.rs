//! 5-Phase Corpus Pipeline with Blocker Priority [8]
//!
//! Implements a structured pipeline for corpus processing with
//! blocker priority integration to focus on critical failures.
//!
//! # Phases
//! 1. Parse (syntax validation)
//! 2. Type check
//! 3. Transpile to Rust
//! 4. Compile Rust
//! 5. Execute & validate
//!
//! # Academic Foundation
//! - [5] Ohno (1988). Toyota Production System. Flow efficiency.
//! - [7] Imai (1986). Kaizen. Continuous improvement through phases.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use crate::reporting::pareto::BlockerPriority;

/// Pipeline phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Phase {
    /// Parse source code (syntax)
    Parse = 0,
    /// Type check AST
    TypeCheck = 1,
    /// Transpile to Rust
    Transpile = 2,
    /// Compile Rust code
    Compile = 3,
    /// Execute and validate
    Execute = 4,
}

impl Phase {
    /// Get all phases in order
    #[must_use]
    pub fn all() -> &'static [Phase] {
        &[
            Phase::Parse,
            Phase::TypeCheck,
            Phase::Transpile,
            Phase::Compile,
            Phase::Execute,
        ]
    }

    /// Get phase name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Phase::Parse => "Parse",
            Phase::TypeCheck => "TypeCheck",
            Phase::Transpile => "Transpile",
            Phase::Compile => "Compile",
            Phase::Execute => "Execute",
        }
    }

    /// Get phase number (1-indexed for display)
    #[must_use]
    pub fn number(&self) -> usize {
        (*self as usize) + 1
    }

    /// Get next phase
    #[must_use]
    pub fn next(&self) -> Option<Phase> {
        match self {
            Phase::Parse => Some(Phase::TypeCheck),
            Phase::TypeCheck => Some(Phase::Transpile),
            Phase::Transpile => Some(Phase::Compile),
            Phase::Compile => Some(Phase::Execute),
            Phase::Execute => None,
        }
    }

    /// Get icon for phase
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Phase::Parse => "📝",
            Phase::TypeCheck => "🔍",
            Phase::Transpile => "🔄",
            Phase::Compile => "🔨",
            Phase::Execute => "▶",
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Result of a single phase execution
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// Phase that was executed
    pub phase: Phase,
    /// Whether phase succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration of phase execution
    pub duration: Duration,
    /// Output artifact (if any)
    pub artifact: Option<String>,
}

impl PhaseResult {
    /// Create successful phase result
    #[must_use]
    pub fn success(phase: Phase, duration: Duration) -> Self {
        Self {
            phase,
            success: true,
            error: None,
            duration,
            artifact: None,
        }
    }

    /// Create failed phase result
    #[must_use]
    pub fn failure(phase: Phase, error: impl Into<String>, duration: Duration) -> Self {
        Self {
            phase,
            success: false,
            error: Some(error.into()),
            duration,
            artifact: None,
        }
    }

    /// Add artifact to result
    pub fn with_artifact(mut self, artifact: impl Into<String>) -> Self {
        self.artifact = Some(artifact.into());
        self
    }

    /// Get status icon
    #[must_use]
    pub fn status_icon(&self) -> &'static str {
        if self.success {
            "✓"
        } else {
            "✗"
        }
    }
}

/// Pipeline execution result for a single file
#[derive(Debug, Clone)]
pub struct PipelineExecution {
    /// File path
    pub path: String,
    /// Phase results
    pub phases: Vec<PhaseResult>,
    /// Last successful phase
    pub last_success: Option<Phase>,
    /// Failing phase (if any)
    pub failed_at: Option<Phase>,
    /// Total duration
    pub total_duration: Duration,
    /// Blocker priority (if failed)
    pub priority: Option<BlockerPriority>,
}

impl PipelineExecution {
    /// Create new execution for file
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            phases: Vec::new(),
            last_success: None,
            failed_at: None,
            total_duration: Duration::ZERO,
            priority: None,
        }
    }

    /// Record phase result
    pub fn record(&mut self, result: PhaseResult) {
        self.total_duration += result.duration;

        if result.success {
            self.last_success = Some(result.phase);
        } else {
            self.failed_at = Some(result.phase);
        }

        self.phases.push(result);
    }

    /// Set blocker priority
    pub fn with_priority(mut self, priority: BlockerPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Check if all phases passed
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.failed_at.is_none() && self.last_success == Some(Phase::Execute)
    }

    /// Get completion percentage (phases passed / total phases)
    #[must_use]
    pub fn completion_percent(&self) -> f64 {
        let passed = self.phases.iter().filter(|p| p.success).count();
        (passed as f64 / 5.0) * 100.0
    }

    /// Get failure stage as string
    #[must_use]
    pub fn failure_stage(&self) -> Option<String> {
        self.failed_at
            .map(|p| format!("Phase {} ({})", p.number(), p.name()))
    }
}

/// Statistics for a single phase across corpus
#[derive(Debug, Clone, Default)]
pub struct PhaseStatistics {
    /// Phase being tracked
    pub phase: Option<Phase>,
    /// Total files processed
    pub total: usize,
    /// Files that passed this phase
    pub passed: usize,
    /// Files that failed at this phase
    pub failed_at: usize,
    /// Total duration across all files
    pub total_duration: Duration,
    /// Average duration per file
    pub avg_duration: Duration,
    /// Error counts by message
    pub error_counts: HashMap<String, usize>,
}

impl PhaseStatistics {
    /// Create new statistics for phase
    #[must_use]
    pub fn new(phase: Phase) -> Self {
        Self {
            phase: Some(phase),
            ..Default::default()
        }
    }

    /// Record a phase result
    pub fn record(&mut self, result: &PhaseResult) {
        self.total += 1;
        self.total_duration += result.duration;

        if result.success {
            self.passed += 1;
        } else {
            self.failed_at += 1;
            if let Some(ref error) = result.error {
                // Extract first line of error for grouping
                let key = error.lines().next().unwrap_or("Unknown error").to_string();
                *self.error_counts.entry(key).or_insert(0) += 1;
            }
        }

        // Update average
        if self.total > 0 {
            self.avg_duration = self.total_duration / self.total as u32;
        }
    }

    /// Get pass rate as percentage
    #[must_use]
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    /// Get top errors (most frequent)
    #[must_use]
    pub fn top_errors(&self, limit: usize) -> Vec<(&str, usize)> {
        let mut errors: Vec<_> = self.error_counts.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        errors
            .into_iter()
            .take(limit)
            .map(|(k, v)| (k.as_str(), *v))
            .collect()
    }
}

/// Pipeline tracker for corpus processing
#[derive(Debug, Clone)]
pub struct CorpusPipeline {
    /// All executions
    executions: Vec<PipelineExecution>,
    /// Phase statistics
    phase_stats: HashMap<Phase, PhaseStatistics>,
    /// Start time
    start_time: Option<Instant>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed: usize,
}

impl Default for CorpusPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl CorpusPipeline {
    /// Create new pipeline
    #[must_use]
    pub fn new() -> Self {
        let mut phase_stats = HashMap::new();
        for phase in Phase::all() {
            phase_stats.insert(*phase, PhaseStatistics::new(*phase));
        }

        Self {
            executions: Vec::new(),
            phase_stats,
            start_time: None,
            total_files: 0,
            processed: 0,
        }
    }

    /// Set total files for progress tracking
    pub fn with_total(mut self, total: usize) -> Self {
        self.total_files = total;
        self
    }

    /// Start pipeline processing
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Record execution result
    pub fn record(&mut self, execution: PipelineExecution) {
        // Update phase statistics
        for result in &execution.phases {
            if let Some(stats) = self.phase_stats.get_mut(&result.phase) {
                stats.record(result);
            }
        }

        self.executions.push(execution);
        self.processed += 1;
    }

    /// Get elapsed time
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |s| s.elapsed())
    }

    /// Get progress percentage
    #[must_use]
    pub fn progress_percent(&self) -> f64 {
        if self.total_files == 0 {
            100.0
        } else {
            (self.processed as f64 / self.total_files as f64) * 100.0
        }
    }

    /// Get phase statistics
    #[must_use]
    pub fn phase_stats(&self, phase: Phase) -> Option<&PhaseStatistics> {
        self.phase_stats.get(&phase)
    }

    /// Get all executions
    #[must_use]
    pub fn executions(&self) -> &[PipelineExecution] {
        &self.executions
    }

    /// Get passing files
    #[must_use]
    pub fn passing(&self) -> Vec<&PipelineExecution> {
        self.executions.iter().filter(|e| e.all_passed()).collect()
    }

    /// Get failing files
    #[must_use]
    pub fn failing(&self) -> Vec<&PipelineExecution> {
        self.executions.iter().filter(|e| !e.all_passed()).collect()
    }

    /// Get files failing at specific phase
    #[must_use]
    pub fn failing_at(&self, phase: Phase) -> Vec<&PipelineExecution> {
        self.executions
            .iter()
            .filter(|e| e.failed_at == Some(phase))
            .collect()
    }

    /// Get failures by blocker priority
    #[must_use]
    pub fn by_priority(&self) -> HashMap<BlockerPriority, Vec<&PipelineExecution>> {
        let mut result: HashMap<BlockerPriority, Vec<_>> = HashMap::new();

        for exec in &self.executions {
            if let Some(priority) = exec.priority {
                result.entry(priority).or_default().push(exec);
            }
        }

        result
    }

    /// Calculate overall success rate
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.processed == 0 {
            0.0
        } else {
            (self.passing().len() as f64 / self.processed as f64) * 100.0
        }
    }

    /// Generate phase funnel visualization
    #[must_use]
    pub fn render_funnel(&self, width: usize) -> String {
        let mut lines = vec![format!(
            "{}╭{}╮",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        )];

        lines.push(format!(
            "{}│ {:^width$} │",
            " ".repeat(2),
            "PIPELINE FUNNEL",
            width = width.saturating_sub(6)
        ));

        lines.push(format!(
            "{}├{}┤",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        let max_count = self.processed.max(1);
        let bar_width = width.saturating_sub(35);

        for phase in Phase::all() {
            if let Some(stats) = self.phase_stats.get(phase) {
                let bar_len = if max_count > 0 {
                    (stats.passed * bar_width) / max_count
                } else {
                    0
                };
                let bar = "█".repeat(bar_len);
                let padding = " ".repeat(bar_width.saturating_sub(bar_len));

                lines.push(format!(
                    "{}│ {} {:10} {:4}/{:4} {}{} │",
                    " ".repeat(2),
                    phase.icon(),
                    phase.name(),
                    stats.passed,
                    stats.total.max(stats.passed),
                    bar,
                    padding
                ));
            }
        }

        // Summary
        lines.push(format!(
            "{}├{}┤",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        let passed = self.passing().len();
        let failed = self.failing().len();
        lines.push(format!(
            "{}│ ✓ Passed: {:4}  ✗ Failed: {:4}  Rate: {:5.1}%{:width$} │",
            " ".repeat(2),
            passed,
            failed,
            self.success_rate(),
            "",
            width = width.saturating_sub(50)
        ));

        lines.push(format!(
            "{}╰{}╯",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        lines.join("\n")
    }

    /// Generate blocker priority breakdown
    #[must_use]
    pub fn render_blockers(&self, width: usize) -> String {
        let mut lines = vec![format!(
            "{}╭{}╮",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        )];

        lines.push(format!(
            "{}│ {:^width$} │",
            " ".repeat(2),
            "BLOCKER PRIORITY BREAKDOWN",
            width = width.saturating_sub(6)
        ));

        lines.push(format!(
            "{}├{}┤",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        let by_priority = self.by_priority();

        for priority in &[
            BlockerPriority::P0Critical,
            BlockerPriority::P1High,
            BlockerPriority::P2Medium,
            BlockerPriority::P3Low,
        ] {
            let count = by_priority.get(priority).map_or(0, Vec::len);
            let icon = match priority {
                BlockerPriority::P0Critical => "🔴",
                BlockerPriority::P1High => "🟠",
                BlockerPriority::P2Medium => "🟡",
                BlockerPriority::P3Low => "🟢",
            };

            lines.push(format!(
                "{}│ {} {:12} {:4} files{:width$} │",
                " ".repeat(2),
                icon,
                format!("{:?}", priority),
                count,
                "",
                width = width.saturating_sub(32)
            ));
        }

        lines.push(format!(
            "{}╰{}╯",
            " ".repeat(2),
            "─".repeat(width.saturating_sub(4))
        ));

        lines.join("\n")
    }
}

/// Pipeline builder for fluent API
#[derive(Debug, Default)]
pub struct PipelineBuilder {
    path: String,
    results: Vec<PhaseResult>,
}

impl PipelineBuilder {
    /// Create new builder for file
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            results: Vec::new(),
        }
    }

    /// Record successful phase
    pub fn pass(mut self, phase: Phase, duration: Duration) -> Self {
        self.results.push(PhaseResult::success(phase, duration));
        self
    }

    /// Record failed phase
    pub fn fail(mut self, phase: Phase, error: impl Into<String>, duration: Duration) -> Self {
        self.results
            .push(PhaseResult::failure(phase, error, duration));
        self
    }

    /// Build execution result
    #[must_use]
    pub fn build(self) -> PipelineExecution {
        let mut exec = PipelineExecution::new(self.path);
        for result in self.results {
            exec.record(result);
        }
        exec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: Phase Tests
    // ============================================================

    #[test]
    fn test_phase_all() {
        assert_eq!(Phase::all().len(), 5);
    }

    #[test]
    fn test_phase_name() {
        assert_eq!(Phase::Parse.name(), "Parse");
        assert_eq!(Phase::Execute.name(), "Execute");
    }

    #[test]
    fn test_phase_number() {
        assert_eq!(Phase::Parse.number(), 1);
        assert_eq!(Phase::Execute.number(), 5);
    }

    #[test]
    fn test_phase_next() {
        assert_eq!(Phase::Parse.next(), Some(Phase::TypeCheck));
        assert_eq!(Phase::Execute.next(), None);
    }

    #[test]
    fn test_phase_display() {
        assert_eq!(format!("{}", Phase::Parse), "Parse");
    }

    #[test]
    fn test_phase_order() {
        assert!(Phase::Parse < Phase::TypeCheck);
        assert!(Phase::TypeCheck < Phase::Transpile);
    }

    // ============================================================
    // EXTREME TDD: PhaseResult Tests
    // ============================================================

    #[test]
    fn test_phase_result_success() {
        let result = PhaseResult::success(Phase::Parse, Duration::from_millis(100));
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.status_icon(), "✓");
    }

    #[test]
    fn test_phase_result_failure() {
        let result = PhaseResult::failure(Phase::Parse, "syntax error", Duration::from_millis(50));
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("syntax error"));
        assert_eq!(result.status_icon(), "✗");
    }

    #[test]
    fn test_phase_result_with_artifact() {
        let result = PhaseResult::success(Phase::Transpile, Duration::from_millis(200))
            .with_artifact("output.rs");
        assert_eq!(result.artifact.as_deref(), Some("output.rs"));
    }

    // ============================================================
    // EXTREME TDD: PipelineExecution Tests
    // ============================================================

    #[test]
    fn test_pipeline_execution_new() {
        let exec = PipelineExecution::new("test.ruchy");
        assert_eq!(exec.path, "test.ruchy");
        assert!(exec.phases.is_empty());
        assert!(exec.last_success.is_none());
    }

    #[test]
    fn test_pipeline_execution_record() {
        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(100),
        ));

        assert_eq!(exec.phases.len(), 1);
        assert_eq!(exec.last_success, Some(Phase::Parse));
    }

    #[test]
    fn test_pipeline_execution_all_passed() {
        let mut exec = PipelineExecution::new("test.ruchy");
        for phase in Phase::all() {
            exec.record(PhaseResult::success(*phase, Duration::from_millis(10)));
        }

        assert!(exec.all_passed());
    }

    #[test]
    fn test_pipeline_execution_failed() {
        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));
        exec.record(PhaseResult::failure(
            Phase::TypeCheck,
            "type error",
            Duration::from_millis(5),
        ));

        assert!(!exec.all_passed());
        assert_eq!(exec.failed_at, Some(Phase::TypeCheck));
        assert_eq!(exec.last_success, Some(Phase::Parse));
    }

    #[test]
    fn test_pipeline_execution_completion() {
        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));
        exec.record(PhaseResult::success(
            Phase::TypeCheck,
            Duration::from_millis(10),
        ));
        // 2 of 5 phases passed

        assert!((exec.completion_percent() - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_pipeline_execution_failure_stage() {
        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::failure(
            Phase::Compile,
            "compile error",
            Duration::from_millis(10),
        ));

        assert_eq!(exec.failure_stage(), Some("Phase 4 (Compile)".to_string()));
    }

    // ============================================================
    // EXTREME TDD: PhaseStatistics Tests
    // ============================================================

    #[test]
    fn test_phase_statistics_new() {
        let stats = PhaseStatistics::new(Phase::Parse);
        assert_eq!(stats.phase, Some(Phase::Parse));
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn test_phase_statistics_record() {
        let mut stats = PhaseStatistics::new(Phase::Parse);
        stats.record(&PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(100),
        ));
        stats.record(&PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(50),
        ));

        assert_eq!(stats.total, 2);
        assert_eq!(stats.passed, 2);
    }

    #[test]
    fn test_phase_statistics_pass_rate() {
        let mut stats = PhaseStatistics::new(Phase::Parse);
        stats.record(&PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));
        stats.record(&PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));
        stats.record(&PhaseResult::failure(
            Phase::Parse,
            "error",
            Duration::from_millis(5),
        ));

        assert!((stats.pass_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_phase_statistics_top_errors() {
        let mut stats = PhaseStatistics::new(Phase::Parse);
        stats.record(&PhaseResult::failure(
            Phase::Parse,
            "error A",
            Duration::from_millis(5),
        ));
        stats.record(&PhaseResult::failure(
            Phase::Parse,
            "error A",
            Duration::from_millis(5),
        ));
        stats.record(&PhaseResult::failure(
            Phase::Parse,
            "error B",
            Duration::from_millis(5),
        ));

        let top = stats.top_errors(2);
        assert_eq!(top[0].0, "error A");
        assert_eq!(top[0].1, 2);
    }

    // ============================================================
    // EXTREME TDD: CorpusPipeline Tests
    // ============================================================

    #[test]
    fn test_corpus_pipeline_new() {
        let pipeline = CorpusPipeline::new();
        assert_eq!(pipeline.processed, 0);
    }

    #[test]
    fn test_corpus_pipeline_record() {
        let mut pipeline = CorpusPipeline::new();
        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));

        pipeline.record(exec);
        assert_eq!(pipeline.processed, 1);
    }

    #[test]
    fn test_corpus_pipeline_passing_failing() {
        let mut pipeline = CorpusPipeline::new();

        // All phases pass
        let mut exec1 = PipelineExecution::new("pass.ruchy");
        for phase in Phase::all() {
            exec1.record(PhaseResult::success(*phase, Duration::from_millis(10)));
        }
        pipeline.record(exec1);

        // Fails at compile
        let mut exec2 = PipelineExecution::new("fail.ruchy");
        exec2.record(PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));
        exec2.record(PhaseResult::failure(
            Phase::TypeCheck,
            "error",
            Duration::from_millis(5),
        ));
        pipeline.record(exec2);

        assert_eq!(pipeline.passing().len(), 1);
        assert_eq!(pipeline.failing().len(), 1);
    }

    #[test]
    fn test_corpus_pipeline_failing_at() {
        let mut pipeline = CorpusPipeline::new();

        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::failure(
            Phase::Compile,
            "error",
            Duration::from_millis(10),
        ));
        pipeline.record(exec);

        assert_eq!(pipeline.failing_at(Phase::Compile).len(), 1);
        assert_eq!(pipeline.failing_at(Phase::Parse).len(), 0);
    }

    #[test]
    fn test_corpus_pipeline_success_rate() {
        let mut pipeline = CorpusPipeline::new();

        // 2 passing
        for _ in 0..2 {
            let mut exec = PipelineExecution::new("pass.ruchy");
            for phase in Phase::all() {
                exec.record(PhaseResult::success(*phase, Duration::from_millis(10)));
            }
            pipeline.record(exec);
        }

        // 1 failing
        let mut exec = PipelineExecution::new("fail.ruchy");
        exec.record(PhaseResult::failure(
            Phase::Parse,
            "error",
            Duration::from_millis(5),
        ));
        pipeline.record(exec);

        assert!((pipeline.success_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_corpus_pipeline_by_priority() {
        let mut pipeline = CorpusPipeline::new();

        let mut exec = PipelineExecution::new("critical.ruchy");
        exec.record(PhaseResult::failure(
            Phase::Parse,
            "error",
            Duration::from_millis(5),
        ));
        let exec = exec.with_priority(BlockerPriority::P0Critical);
        pipeline.record(exec);

        let by_priority = pipeline.by_priority();
        assert_eq!(
            by_priority.get(&BlockerPriority::P0Critical).map(Vec::len),
            Some(1)
        );
    }

    #[test]
    fn test_corpus_pipeline_render_funnel() {
        let mut pipeline = CorpusPipeline::new();

        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::success(
            Phase::Parse,
            Duration::from_millis(10),
        ));
        pipeline.record(exec);

        let output = pipeline.render_funnel(60);
        assert!(output.contains("PIPELINE FUNNEL"));
        assert!(output.contains("Parse"));
    }

    #[test]
    fn test_corpus_pipeline_render_blockers() {
        let mut pipeline = CorpusPipeline::new();

        let mut exec = PipelineExecution::new("test.ruchy");
        exec.record(PhaseResult::failure(
            Phase::Parse,
            "error",
            Duration::from_millis(5),
        ));
        let exec = exec.with_priority(BlockerPriority::P1High);
        pipeline.record(exec);

        let output = pipeline.render_blockers(60);
        assert!(output.contains("BLOCKER PRIORITY"));
    }

    // ============================================================
    // EXTREME TDD: PipelineBuilder Tests
    // ============================================================

    #[test]
    fn test_pipeline_builder() {
        let exec = PipelineBuilder::new("test.ruchy")
            .pass(Phase::Parse, Duration::from_millis(10))
            .pass(Phase::TypeCheck, Duration::from_millis(20))
            .fail(
                Phase::Transpile,
                "transpile error",
                Duration::from_millis(5),
            )
            .build();

        assert_eq!(exec.path, "test.ruchy");
        assert_eq!(exec.phases.len(), 3);
        assert_eq!(exec.last_success, Some(Phase::TypeCheck));
        assert_eq!(exec.failed_at, Some(Phase::Transpile));
    }
}
