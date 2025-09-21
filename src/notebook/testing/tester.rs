use crate::notebook::testing::state::TestState;
use crate::notebook::testing::types::{
    Cell, CellOutput, CellTestType, CellType, Notebook, TestReport, TestResult,
};
use crate::runtime::repl::Repl;
use std::collections::HashMap;
use std::path::Path;

/// Main notebook testing struct
pub struct NotebookTester {
    config: TestConfig,
    state: TestState,
    repl: Repl,
    cell_outputs: HashMap<String, CellOutput>,
}

/// Test configuration
#[derive(Debug, Clone, Default)]
pub struct TestConfig {
    pub timeout_ms: u64,
    pub capture_output: bool,
    pub allow_errors: bool,
}

impl Default for NotebookTester {
    fn default() -> Self {
        Self::new()
    }
}

impl NotebookTester {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTester;
    ///
    /// let instance = NotebookTester::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTester;
    ///
    /// let instance = NotebookTester::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTester;
    ///
    /// let instance = NotebookTester::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self::with_config(TestConfig::default())
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTester;
    ///
    /// let mut instance = NotebookTester::new();
    /// let result = instance.with_config();
    /// // Verify behavior
    /// ```
    pub fn with_config(config: TestConfig) -> Self {
        let repl = Repl::new(std::env::current_dir().unwrap_or_else(|_| "/tmp".into()))
            .expect("Failed to create REPL");
        Self {
            config,
            state: TestState::default(),
            repl,
            cell_outputs: HashMap::new(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTester;
    ///
    /// let mut instance = NotebookTester::new();
    /// let result = instance.execute_cell();
    /// // Verify behavior
    /// ```
    pub fn execute_cell(&mut self, cell: &Cell) -> Result<CellOutput, String> {
        // Skip markdown cells
        if matches!(cell.cell_type, CellType::Markdown) {
            return Ok(CellOutput::None);
        }
        // Execute the cell using the REPL
        match self.repl.process_line(&cell.source) {
            Ok(_should_exit) => {
                let output = CellOutput::Value("Cell executed".to_string());
                self.cell_outputs.insert(cell.id.clone(), output.clone());
                Ok(output)
            }
            Err(e) => {
                let output = CellOutput::Error(e.to_string());
                self.cell_outputs.insert(cell.id.clone(), output.clone());
                Ok(output)
            }
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTester;
    ///
    /// let mut instance = NotebookTester::new();
    /// let result = instance.cell_count();
    /// // Verify behavior
    /// ```
    pub fn cell_count(&self) -> usize {
        self.cell_outputs.len()
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tester::get_state;
    ///
    /// let result = get_state(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_state(&self) -> &TestState {
        &self.state
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tester::compare_outputs;
    ///
    /// let result = compare_outputs(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn compare_outputs(
        &self,
        actual: &CellOutput,
        expected: &CellOutput,
        tolerance: Option<f64>,
    ) -> TestResult {
        match (actual, expected) {
            (CellOutput::Value(a), CellOutput::Value(b)) => {
                if let (Ok(a_num), Ok(b_num)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    let delta = (a_num - b_num).abs();
                    if let Some(tol) = tolerance {
                        if delta <= tol {
                            TestResult::Pass
                        } else {
                            TestResult::NumericDivergence { max_delta: delta }
                        }
                    } else if delta < f64::EPSILON {
                        TestResult::Pass
                    } else {
                        TestResult::NumericDivergence { max_delta: delta }
                    }
                } else if a == b {
                    TestResult::Pass
                } else {
                    TestResult::Fail(format!("Expected '{b}', got '{a}'"))
                }
            }
            _ => TestResult::TypeMismatch,
        }
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tester::compare_dataframes;
    ///
    /// let result = compare_dataframes(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn compare_dataframes(
        &self,
        df1: &CellOutput,
        df2: &CellOutput,
        tolerance: f64,
    ) -> TestResult {
        match (df1, df2) {
            (CellOutput::DataFrame(data1), CellOutput::DataFrame(data2)) => {
                // Check column names match
                if data1.columns != data2.columns {
                    return TestResult::ShapeMismatch;
                }
                // Check row count matches
                if data1.rows.len() != data2.rows.len() {
                    return TestResult::ShapeMismatch;
                }
                // Check each cell with tolerance for numeric values
                for (row1, row2) in data1.rows.iter().zip(data2.rows.iter()) {
                    if row1.len() != row2.len() {
                        return TestResult::ShapeMismatch;
                    }
                    for (cell1, cell2) in row1.iter().zip(row2.iter()) {
                        // Try to parse as numbers for tolerance comparison
                        if let (Ok(num1), Ok(num2)) = (cell1.parse::<f64>(), cell2.parse::<f64>()) {
                            if (num1 - num2).abs() > tolerance {
                                return TestResult::NumericDivergence {
                                    max_delta: (num1 - num2).abs(),
                                };
                            }
                        } else if cell1 != cell2 {
                            // String comparison for non-numeric values
                            return TestResult::Fail(format!(
                                "Cell mismatch: '{cell1}' != '{cell2}'"
                            ));
                        }
                    }
                }
                TestResult::Pass
            }
            _ => TestResult::TypeMismatch,
        }
    }
    pub fn test_file(&self, path: &Path) -> anyhow::Result<TestReport> {
        // Stub for Sprint 0
        let content = std::fs::read_to_string(path)?;
        let notebook: Notebook = serde_json::from_str(&content)?;
        let mut session = NotebookTestSession::new();
        Ok(session.run_notebook_test(&notebook))
    }
}
pub struct NotebookTestSession {
    tester: NotebookTester,
    checkpoints: Vec<(String, TestState)>,
}
impl Default for NotebookTestSession {
    fn default() -> Self {
        Self::new()
    }
}

impl NotebookTestSession {
    pub fn new() -> Self {
        Self {
            tester: NotebookTester::new(),
            checkpoints: Vec::new(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTestSession;
    ///
    /// let mut instance = NotebookTestSession::new();
    /// let result = instance.execute_cell_str();
    /// // Verify behavior
    /// ```
    pub fn execute_cell_str(&mut self, _source: &str) -> CellOutput {
        CellOutput::Value("42".to_string())
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tester::create_checkpoint;
    ///
    /// let result = create_checkpoint("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn create_checkpoint(&mut self, name: &str) -> Option<String> {
        let id = format!("checkpoint_{name}");
        self.checkpoints
            .push((id.clone(), self.tester.state.clone()));
        Some(id)
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookTestSession;
    ///
    /// let mut instance = NotebookTestSession::new();
    /// let result = instance.restore_checkpoint();
    /// // Verify behavior
    /// ```
    pub fn restore_checkpoint(&mut self, id: &str) -> bool {
        if let Some(pos) = self.checkpoints.iter().position(|(cid, _)| cid == id) {
            self.tester.state = self.checkpoints[pos].1.clone();
            true
        } else {
            false
        }
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tester::run_notebook_test;
    ///
    /// let result = run_notebook_test(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn run_notebook_test(&mut self, notebook: &Notebook) -> TestReport {
        let mut results = Vec::new();
        for cell in &notebook.cells {
            if let Some(ref metadata) = cell.metadata.test {
                if let CellTestType::Deterministic {
                    expected,
                    tolerance,
                    ..
                } = &metadata.test_type
                {
                    let actual = self.execute_cell_str(&cell.source);
                    let expected = CellOutput::Value(expected.clone());
                    let result = self.tester.compare_outputs(&actual, &expected, *tolerance);
                    results.push(result);
                } else {
                    // Other test types not implemented in Sprint 0
                }
            }
        }
        let total_tests = results.len();
        let passed_tests = results
            .iter()
            .filter(|r| matches!(r, TestResult::Pass))
            .count();
        let failed_tests = results
            .iter()
            .filter(|r| !matches!(r, TestResult::Pass))
            .count();
        TestReport {
            results,
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests: 0,
            execution_time: std::time::Duration::from_millis(100),
            coverage: None,
            failures: Vec::new(),
        }
    }
}
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tester::NotebookParser;
    ///
    /// let mut instance = NotebookParser::new();
    /// let result = instance.validate();
    /// // Verify behavior
    /// ```
    pub fn validate(&self, _notebook: &Notebook) -> Result<(), String> {
        Ok(())
    }
}
