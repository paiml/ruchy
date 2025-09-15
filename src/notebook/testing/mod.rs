// TEST-003: Notebook Testing Framework Core Module
// Sprint 1: Core Testing Infrastructure
// PMAT Complexity Target: <10 per function
// Toyota Way: Zero defect tolerance
pub mod tester;
pub mod golden;
pub mod coverage;
pub mod state;
pub mod report;
pub mod types;
pub mod property;
pub mod differential;
pub mod mutation;
pub mod formal;
pub mod complexity;
pub mod educational;
pub mod grading;
pub mod tutorial;
pub mod integration;
pub mod performance;
pub mod sandbox;
pub mod anticheat;
pub mod migration;
pub mod incremental;
pub mod smt;
pub mod progressive;
pub use tester::{NotebookTester, NotebookTestSession, NotebookParser as TestNotebookParser, TestConfig};
// pub use golden::GoldenManager;  // Not implemented
// pub use coverage::{CoverageTracker, InstrumentedCell};  // Not implemented
pub use state::TestState;
pub use types::{
    // TestConfig, TestResult,  // Not all exist
    CellTestType, CellOutput,
    Cell, CellType, CellMetadata, CellTestMetadata,
    Notebook, NotebookParser, DataFrameData, NotebookMetadata,
    TestReport, TestFailure, CoverageReport, PlotData,
};
pub use property::{PropertyTester, PropertyTestConfig};
pub use differential::{DifferentialConfig, DifferentialResult, DivergenceType};
pub use mutation::{MutationConfig, MutationType, Mutation, MutationResult};
pub use formal::{FormalConfig, Invariant, Constraint, ConstraintSeverity,
                 VerificationResult, FunctionSpec, ExecutionPath, LoopInvariant};
pub use complexity::{ComplexityConfig, TimeComplexity, SpaceComplexity,
                     ComplexityResult, Hotspot};
pub use educational::{Assignment, RubricItem, TestCase as EduTestCase,
                      StudentSubmission, Grade, LearningAnalytics, LearningEvent, EventType};
pub use grading::{GradingConfig, ExerciseValidator, Exercise, Difficulty, ValidationResult};
pub use tutorial::{TutorialStep, ValidationRule, StepResult, AdaptiveHintSystem};
pub use integration::{CiCdConfig, CiProvider,
                      ContinuousMonitor, Alert, Metric, AlertAction};
pub use performance::{BenchmarkResult,
                      TestCache, CacheStats, ResourceMonitor, ResourceUsage, TestSharder,
                      RegressionDetector, RegressionResult, TestPrioritizer};
pub use sandbox::{WasmSandbox, ResourceLimits, SandboxError, ExecutionResult, SandboxCoordinator,
                  ProblemGenerator, Exercise as SandboxExercise};
pub use anticheat::{Submission, PlagiarismResult, ObfuscationDetector, PatternAnalyzer};
pub use migration::{MigrationTool, TestFramework, MigrationConfig, MigrationResult};
pub use incremental::{IncrementalConfig, IncrementalResult, TestResultCache};
pub use smt::{SolverType, SmtQuery, SmtResult, BoundedModelChecker};
pub use progressive::{DisclosureConfig, StudentProgress, TestHierarchy};

// Test modules (to be added once API stabilizes)

/// Run tests on a notebook file
pub fn test_notebook(path: &std::path::Path, config: TestConfig) -> anyhow::Result<TestReport> {
    let tester = NotebookTester::with_config(config);
    tester.test_file(path)
}

