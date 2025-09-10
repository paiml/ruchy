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

pub use tester::{NotebookTester, NotebookTestSession, NotebookParser as TestNotebookParser};
pub use golden::GoldenManager;
pub use coverage::{CoverageTracker, InstrumentedCell};
pub use state::TestState;
pub use types::{
    TestConfig, TestResult, CellTestType, CellOutput,
    Cell, CellType, CellMetadata, CellTestMetadata,
    Notebook, NotebookParser, DataFrameData, NotebookMetadata,
    TestReport, TestFailure, CoverageReport, PlotData,
};
pub use property::{PropertyTester, PropertyTestConfig};
pub use differential::{DifferentialTester, DifferentialConfig, DifferentialResult, DivergenceType};
pub use mutation::{MutationTester, MutationConfig, MutationType, Mutation, MutationResult};

/// Initialize the testing framework with default configuration
pub fn init() -> NotebookTester {
    NotebookTester::new()
}

/// Run tests on a notebook file
pub fn test_notebook(path: &std::path::Path, config: TestConfig) -> anyhow::Result<TestReport> {
    let tester = NotebookTester::with_config(config);
    tester.test_file(path)
}