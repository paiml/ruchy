// TEST-003: Notebook Testing Framework Core Module
// Sprint 1: Core Testing Infrastructure
// PMAT Complexity Target: <10 per function
// Toyota Way: Zero defect tolerance
pub mod anticheat;
pub mod complexity;
pub mod coverage;
pub mod differential;
pub mod educational;
pub mod formal;
pub mod golden;
pub mod grading;
pub mod incremental;
pub mod integration;
pub mod migration;
pub mod mutation;
pub mod performance;
pub mod progressive;
pub mod property;
pub mod report;
pub mod sandbox;
pub mod smt;
pub mod state;
pub mod tester;
pub mod tutorial;
pub mod types;
pub use tester::{
    NotebookParser as TestNotebookParser, NotebookTestSession, NotebookTester, TestConfig,
};
// pub use golden::GoldenManager;  // Not implemented
// pub use coverage::{CoverageTracker, InstrumentedCell};  // Not implemented
pub use anticheat::{ObfuscationDetector, PatternAnalyzer, PlagiarismResult, Submission};
pub use complexity::{
    ComplexityConfig, ComplexityResult, Hotspot, SpaceComplexity, TimeComplexity,
};
pub use differential::{DifferentialConfig, DifferentialResult, DivergenceType};
pub use educational::{
    Assignment, EventType, Grade, LearningAnalytics, LearningEvent, RubricItem, StudentSubmission,
    TestCase as EduTestCase,
};
pub use formal::{
    Constraint, ConstraintSeverity, ExecutionPath, FormalConfig, FunctionSpec, Invariant,
    LoopInvariant, VerificationResult,
};
pub use grading::{Difficulty, Exercise, ExerciseValidator, GradingConfig, ValidationResult};
pub use incremental::{IncrementalConfig, IncrementalResult, TestResultCache};
pub use integration::{Alert, AlertAction, CiCdConfig, CiProvider, ContinuousMonitor, Metric};
pub use migration::{MigrationConfig, MigrationResult, MigrationTool, TestFramework};
pub use mutation::{Mutation, MutationConfig, MutationResult, MutationType};
pub use performance::{
    BenchmarkResult, CacheStats, RegressionDetector, RegressionResult, ResourceMonitor,
    ResourceUsage, TestCache, TestPrioritizer, TestSharder,
};
pub use progressive::{DisclosureConfig, StudentProgress, TestHierarchy};
pub use property::{PropertyTestConfig, PropertyTester};
pub use sandbox::{
    ExecutionResult, Exercise as SandboxExercise, ProblemGenerator, ResourceLimits,
    SandboxCoordinator, SandboxError, WasmSandbox,
};
pub use smt::{BoundedModelChecker, SmtQuery, SmtResult, SolverType};
pub use state::TestState;
pub use tutorial::{AdaptiveHintSystem, StepResult, TutorialStep, ValidationRule};
pub use types::{
    Cell,
    CellMetadata,
    CellOutput,
    CellTestMetadata,
    // TestConfig, TestResult,  // Not all exist
    CellTestType,
    CellType,
    CoverageReport,
    DataFrameData,
    Notebook,
    NotebookMetadata,
    NotebookParser,
    PlotData,
    TestFailure,
    TestReport,
};

// Test modules (to be added once API stabilizes)

/// Run tests on a notebook file
pub fn test_notebook(path: &std::path::Path, config: TestConfig) -> anyhow::Result<TestReport> {
    let tester = NotebookTester::with_config(config);
    tester.test_file(path)
}
