// TEST-004: Core types for notebook testing framework
// PMAT Complexity: Each struct/enum kept simple (<10)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub tolerance: f64,
    pub coverage: bool,
    pub mutation: bool,
    pub golden_dir: PathBuf,
    pub max_time: std::time::Duration,
    pub max_memory: usize,
    pub update_golden: bool,
}

impl TestConfig {
/// # Examples
///
/// ```
/// use ruchy::notebook::testing::types::TestConfig;
///
let instance = TestConfig::new();
// Verify behavior
/// ```
/// # Examples
///
/// ```
/// use ruchy::notebook::testing::types::TestConfig;
///
let instance = TestConfig::new();
// Verify behavior
/// ```
pub fn new() -> Self {
        Self {
            tolerance: 1e-6,
            coverage: false,
            mutation: false,
            golden_dir: PathBuf::from("test/golden"),
            max_time: std::time::Duration::from_secs(30),
            max_memory: 512 * 1024 * 1024, // 512MB
            update_golden: false,
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}
/// Result of a test execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestResult {
    Pass,
    Fail(String),
    Skip,
    NumericDivergence { max_delta: f64 },
    TypeMismatch,
    ShapeMismatch,
    CategoricalMismatch { col: String },
    Timeout,
}
/// Type of test for a cell
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CellTestType {
    #[serde(rename = "deterministic")]
    Deterministic {
        expected: String,
        tolerance: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        golden: Option<PathBuf>,
    },
    #[serde(rename = "property")]
    Property {
        invariants: Vec<String>,
        generators: HashMap<String, String>,
    },
    #[serde(rename = "regression")]
    Regression {
        baseline: PathBuf,
        max_time_factor: f64,
        max_memory_factor: f64,
    },
    #[serde(rename = "skip")]
    Skip,
}
/// Output from a cell execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellOutput {
    Value(String),
    DataFrame(DataFrameData),
    Plot(PlotData),
    Error(String),
    Html(String),
    None,
}
/// Simplified `DataFrame` representation for testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataFrameData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
/// Plot data for perceptual comparison
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotData {
    pub format: String,
    pub data: Vec<u8>,
    pub perceptual_hash: Option<String>,
}
/// Notebook cell structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    pub source: String,
    pub cell_type: CellType,
    pub metadata: CellMetadata,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Code,
    Markdown,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<CellTestMetadata>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellTestMetadata {
    #[serde(flatten)]
    pub test_type: CellTestType,
    #[serde(default)]
    pub stop_on_failure: bool,
}
/// Complete notebook structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub cells: Vec<Cell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NotebookMetadata>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
}
/// Parser for .ruchynb files
pub struct NotebookParser;
impl Default for NotebookParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NotebookParser {
    pub fn new() -> Self {
        Self
    }
    /// Parse a notebook from JSON
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::types::NotebookParser;
/// 
let mut instance = NotebookParser::new();
let result = instance.parse();
// Verify behavior
/// ```
pub fn parse(&self, content: &str) -> anyhow::Result<Notebook> {
        serde_json::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse notebook: {}", e))
    }
    /// Validate notebook structure
/// # Examples
/// 
/// ```ignore
/// use ruchy::notebook::testing::types::validate;
/// 
/// let result = validate(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn validate(&self, notebook: &Notebook) -> anyhow::Result<()> {
        if notebook.cells.is_empty() {
            return Err(anyhow::anyhow!("Notebook has no cells"));
        }
        for cell in &notebook.cells {
            if cell.id.is_empty() {
                return Err(anyhow::anyhow!("Cell missing ID"));
            }
        }
        Ok(())
    }
}
/// Test report for notebook tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub execution_time: std::time::Duration,
    pub coverage: Option<CoverageReport>,
    pub failures: Vec<TestFailure>,
    pub results: Vec<TestResult>,
}
/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub uncovered_sections: Vec<String>,
}
/// Test failure details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub cell_id: String,
    pub expected: String,
    pub actual: String,
    pub message: String,
}
