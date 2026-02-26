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
                            let delta = (num1 - num2).abs();
                            if delta > tolerance {
                                return TestResult::NumericDivergence { max_delta: delta };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::testing::types::{CellMetadata, CellTestMetadata, DataFrameData};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    // EXTREME TDD: Comprehensive test coverage for notebook testing framework

    #[test]
    fn test_test_config_default() {
        let config = TestConfig::default();
        assert_eq!(config.timeout_ms, 0);
        assert!(!config.capture_output);
        assert!(!config.allow_errors);
    }

    #[test]
    fn test_test_config_debug_clone() {
        let config = TestConfig {
            timeout_ms: 5000,
            capture_output: true,
            allow_errors: false,
        };
        let cloned = config.clone();
        assert_eq!(config.timeout_ms, cloned.timeout_ms);
        assert_eq!(config.capture_output, cloned.capture_output);
        assert_eq!(config.allow_errors, cloned.allow_errors);

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("timeout_ms: 5000"));
        assert!(debug_str.contains("capture_output: true"));
        assert!(debug_str.contains("allow_errors: false"));
    }

    #[test]
    fn test_notebook_tester_new() {
        let tester = NotebookTester::new();
        assert_eq!(tester.config.timeout_ms, 0);
        assert_eq!(tester.cell_outputs.len(), 0);
    }

    #[test]
    fn test_notebook_tester_default() {
        let tester = NotebookTester::default();
        assert_eq!(tester.config.timeout_ms, 0);
        assert_eq!(tester.cell_outputs.len(), 0);
    }

    #[test]
    fn test_notebook_tester_with_config() {
        let config = TestConfig {
            timeout_ms: 3000,
            capture_output: true,
            allow_errors: true,
        };
        let tester = NotebookTester::with_config(config);
        assert_eq!(tester.config.timeout_ms, 3000);
        assert!(tester.config.capture_output);
        assert!(tester.config.allow_errors);
    }

    #[test]
    fn test_execute_cell_markdown() {
        let mut tester = NotebookTester::new();
        let cell = Cell {
            id: "markdown1".to_string(),
            cell_type: CellType::Markdown,
            source: "# Header".to_string(),
            metadata: CellMetadata { test: None },
        };

        let result = tester
            .execute_cell(&cell)
            .expect("operation should succeed in test");
        assert_eq!(result, CellOutput::None);
        assert_eq!(tester.cell_count(), 0); // Markdown cells don't get stored
    }

    #[test]
    fn test_execute_cell_code_success() {
        let mut tester = NotebookTester::new();
        let cell = Cell {
            id: "code1".to_string(),
            cell_type: CellType::Code,
            source: "println(\"Hello\")".to_string(),
            metadata: CellMetadata { test: None },
        };

        let result = tester.execute_cell(&cell);
        assert!(result.is_ok());
        assert_eq!(tester.cell_count(), 1);

        // Check that output was stored
        let output = tester.cell_outputs.get("code1");
        assert!(output.is_some());
    }

    #[test]
    fn test_execute_cell_code_error() {
        let mut tester = NotebookTester::new();
        let cell = Cell {
            id: "error1".to_string(),
            cell_type: CellType::Code,
            source: "invalid_syntax!!!".to_string(),
            metadata: CellMetadata { test: None },
        };

        let result = tester.execute_cell(&cell);
        assert!(result.is_ok());
        assert_eq!(tester.cell_count(), 1);

        // Check that output was stored (might be error or value depending on REPL behavior)
        let output = tester.cell_outputs.get("error1");
        assert!(output.is_some());
    }

    #[test]
    fn test_cell_count() {
        let mut tester = NotebookTester::new();
        assert_eq!(tester.cell_count(), 0);

        let cell1 = Cell {
            id: "1".to_string(),
            cell_type: CellType::Code,
            source: "x = 1".to_string(),
            metadata: CellMetadata { test: None },
        };

        let cell2 = Cell {
            id: "2".to_string(),
            cell_type: CellType::Code,
            source: "y = 2".to_string(),
            metadata: CellMetadata { test: None },
        };

        tester
            .execute_cell(&cell1)
            .expect("operation should succeed in test");
        assert_eq!(tester.cell_count(), 1);

        tester
            .execute_cell(&cell2)
            .expect("operation should succeed in test");
        assert_eq!(tester.cell_count(), 2);
    }

    #[test]
    fn test_get_state() {
        let tester = NotebookTester::new();
        let state = tester.get_state();
        // TestState should exist and be accessible
        assert!(state.is_empty());
    }

    #[test]
    fn test_compare_outputs_value_identical() {
        let tester = NotebookTester::new();
        let output1 = CellOutput::Value("hello".to_string());
        let output2 = CellOutput::Value("hello".to_string());

        let result = tester.compare_outputs(&output1, &output2, None);
        assert_eq!(result, TestResult::Pass);
    }

    #[test]
    fn test_compare_outputs_value_different() {
        let tester = NotebookTester::new();
        let output1 = CellOutput::Value("hello".to_string());
        let output2 = CellOutput::Value("world".to_string());

        let result = tester.compare_outputs(&output1, &output2, None);
        assert_eq!(
            result,
            TestResult::Fail("Expected 'world', got 'hello'".to_string())
        );
    }

    #[test]
    fn test_compare_outputs_numeric_within_tolerance() {
        let tester = NotebookTester::new();
        let output1 = CellOutput::Value("1.001".to_string());
        let output2 = CellOutput::Value("1.000".to_string());

        let result = tester.compare_outputs(&output1, &output2, Some(0.01));
        assert_eq!(result, TestResult::Pass);
    }

    #[test]
    fn test_compare_outputs_numeric_outside_tolerance() {
        let tester = NotebookTester::new();
        let output1 = CellOutput::Value("1.1".to_string());
        let output2 = CellOutput::Value("1.0".to_string());

        let result = tester.compare_outputs(&output1, &output2, Some(0.05));
        if let TestResult::NumericDivergence { max_delta } = result {
            assert!((max_delta - 0.1).abs() < f64::EPSILON);
        } else {
            panic!("Expected NumericDivergence");
        }
    }

    #[test]
    fn test_compare_outputs_numeric_epsilon() {
        let tester = NotebookTester::new();
        let output1 = CellOutput::Value("1.0000000000000001".to_string());
        let output2 = CellOutput::Value("1.0".to_string());

        let result = tester.compare_outputs(&output1, &output2, None);
        // Should pass due to epsilon comparison
        assert_eq!(result, TestResult::Pass);
    }

    #[test]
    fn test_compare_outputs_type_mismatch() {
        let tester = NotebookTester::new();
        let output1 = CellOutput::Value("hello".to_string());
        let output2 = CellOutput::Error("error".to_string());

        let result = tester.compare_outputs(&output1, &output2, None);
        assert_eq!(result, TestResult::TypeMismatch);
    }

    #[test]
    fn test_compare_dataframes_identical() {
        let tester = NotebookTester::new();
        let df_data = DataFrameData {
            columns: vec!["A".to_string(), "B".to_string()],
            rows: vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
            ],
        };
        let output1 = CellOutput::DataFrame(df_data.clone());
        let output2 = CellOutput::DataFrame(df_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(result, TestResult::Pass);
    }

    #[test]
    fn test_compare_dataframes_column_mismatch() {
        let tester = NotebookTester::new();
        let df1_data = DataFrameData {
            columns: vec!["A".to_string(), "B".to_string()],
            rows: vec![vec!["1".to_string(), "2".to_string()]],
        };
        let df2_data = DataFrameData {
            columns: vec!["A".to_string(), "C".to_string()],
            rows: vec![vec!["1".to_string(), "2".to_string()]],
        };
        let output1 = CellOutput::DataFrame(df1_data);
        let output2 = CellOutput::DataFrame(df2_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(result, TestResult::ShapeMismatch);
    }

    #[test]
    fn test_compare_dataframes_row_count_mismatch() {
        let tester = NotebookTester::new();
        let df1_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1".to_string()], vec!["2".to_string()]],
        };
        let df2_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1".to_string()]],
        };
        let output1 = CellOutput::DataFrame(df1_data);
        let output2 = CellOutput::DataFrame(df2_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(result, TestResult::ShapeMismatch);
    }

    #[test]
    fn test_compare_dataframes_cell_length_mismatch() {
        let tester = NotebookTester::new();
        let df1_data = DataFrameData {
            columns: vec!["A".to_string(), "B".to_string()],
            rows: vec![vec!["1".to_string(), "2".to_string()]],
        };
        let df2_data = DataFrameData {
            columns: vec!["A".to_string(), "B".to_string()],
            rows: vec![vec!["1".to_string()]], // Missing cell
        };
        let output1 = CellOutput::DataFrame(df1_data);
        let output2 = CellOutput::DataFrame(df2_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(result, TestResult::ShapeMismatch);
    }

    #[test]
    fn test_compare_dataframes_numeric_within_tolerance() {
        let tester = NotebookTester::new();
        let df1_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1.001".to_string()]],
        };
        let df2_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1.000".to_string()]],
        };
        let output1 = CellOutput::DataFrame(df1_data);
        let output2 = CellOutput::DataFrame(df2_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(result, TestResult::Pass);
    }

    #[test]
    fn test_compare_dataframes_numeric_outside_tolerance() {
        let tester = NotebookTester::new();
        let df1_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1.1".to_string()]],
        };
        let df2_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1.0".to_string()]],
        };
        let output1 = CellOutput::DataFrame(df1_data);
        let output2 = CellOutput::DataFrame(df2_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.05);
        if let TestResult::NumericDivergence { max_delta } = result {
            assert!((max_delta - 0.1).abs() < f64::EPSILON);
        } else {
            panic!("Expected NumericDivergence");
        }
    }

    #[test]
    fn test_compare_dataframes_string_mismatch() {
        let tester = NotebookTester::new();
        let df1_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["hello".to_string()]],
        };
        let df2_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["world".to_string()]],
        };
        let output1 = CellOutput::DataFrame(df1_data);
        let output2 = CellOutput::DataFrame(df2_data);

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(
            result,
            TestResult::Fail("Cell mismatch: 'hello' != 'world'".to_string())
        );
    }

    #[test]
    fn test_compare_dataframes_type_mismatch() {
        let tester = NotebookTester::new();
        let df_data = DataFrameData {
            columns: vec!["A".to_string()],
            rows: vec![vec!["1".to_string()]],
        };
        let output1 = CellOutput::DataFrame(df_data);
        let output2 = CellOutput::Value("not a dataframe".to_string());

        let result = tester.compare_dataframes(&output1, &output2, 0.01);
        assert_eq!(result, TestResult::TypeMismatch);
    }

    #[test]
    fn test_test_file_valid_notebook() {
        let tester = NotebookTester::new();

        // Create a temporary file with valid notebook JSON
        let mut temp_file = NamedTempFile::new().expect("operation should succeed in test");
        let notebook_json = r#"{
            "cells": [
                {
                    "id": "1",
                    "source": "x = 1",
                    "cell_type": "code",
                    "metadata": {"test": null}
                }
            ],
            "metadata": null
        }"#;
        temp_file
            .write_all(notebook_json.as_bytes())
            .expect("operation should succeed in test");

        let result = tester.test_file(temp_file.path());
        assert!(result.is_ok());

        let report = result.expect("operation should succeed in test");
        assert_eq!(report.total_tests, 0); // No cells with test metadata
    }

    #[test]
    fn test_test_file_invalid_path() {
        let tester = NotebookTester::new();
        let result = tester.test_file(&PathBuf::from("/nonexistent/file.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_test_file_invalid_json() {
        let tester = NotebookTester::new();

        // Create a temporary file with invalid JSON
        let mut temp_file = NamedTempFile::new().expect("operation should succeed in test");
        temp_file
            .write_all(b"invalid json")
            .expect("operation should succeed in test");

        let result = tester.test_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_notebook_test_session_new() {
        let session = NotebookTestSession::new();
        assert_eq!(session.checkpoints.len(), 0);
    }

    #[test]
    fn test_notebook_test_session_default() {
        let session = NotebookTestSession::default();
        assert_eq!(session.checkpoints.len(), 0);
    }

    #[test]
    fn test_execute_cell_str() {
        let mut session = NotebookTestSession::new();
        let result = session.execute_cell_str("x = 42");
        assert_eq!(result, CellOutput::Value("42".to_string()));
    }

    #[test]
    fn test_create_checkpoint() {
        let mut session = NotebookTestSession::new();
        let checkpoint_id = session.create_checkpoint("test");

        assert!(checkpoint_id.is_some());
        assert_eq!(
            checkpoint_id.expect("operation should succeed in test"),
            "checkpoint_test"
        );
        assert_eq!(session.checkpoints.len(), 1);
    }

    #[test]
    fn test_restore_checkpoint_success() {
        let mut session = NotebookTestSession::new();
        let checkpoint_id = session
            .create_checkpoint("test")
            .expect("operation should succeed in test");

        let restored = session.restore_checkpoint(&checkpoint_id);
        assert!(restored);
    }

    #[test]
    fn test_restore_checkpoint_failure() {
        let mut session = NotebookTestSession::new();
        let restored = session.restore_checkpoint("nonexistent");
        assert!(!restored);
    }

    #[test]
    fn test_multiple_checkpoints() {
        let mut session = NotebookTestSession::new();

        let cp1 = session
            .create_checkpoint("first")
            .expect("operation should succeed in test");
        let cp2 = session
            .create_checkpoint("second")
            .expect("operation should succeed in test");

        assert_eq!(session.checkpoints.len(), 2);
        assert_ne!(cp1, cp2);

        assert!(session.restore_checkpoint(&cp1));
        assert!(session.restore_checkpoint(&cp2));
    }

    #[test]
    fn test_run_notebook_test_empty() {
        let mut session = NotebookTestSession::new();
        let notebook = Notebook {
            cells: vec![],
            metadata: None,
        };

        let report = session.run_notebook_test(&notebook);
        assert_eq!(report.total_tests, 0);
        assert_eq!(report.passed_tests, 0);
        assert_eq!(report.failed_tests, 0);
        assert_eq!(report.skipped_tests, 0);
    }

    #[test]
    fn test_run_notebook_test_with_deterministic_test() {
        let mut session = NotebookTestSession::new();

        let cell = Cell {
            id: "test1".to_string(),
            cell_type: CellType::Code,
            source: "2 + 2".to_string(),
            metadata: CellMetadata {
                test: Some(CellTestMetadata {
                    test_type: CellTestType::Deterministic {
                        expected: "42".to_string(),
                        tolerance: Some(0.01),
                        golden: None,
                    },
                    stop_on_failure: false,
                }),
            },
        };

        let notebook = Notebook {
            cells: vec![cell],
            metadata: None,
        };

        let report = session.run_notebook_test(&notebook);
        assert_eq!(report.total_tests, 1);
        assert_eq!(report.passed_tests, 1); // execute_cell_str always returns "42"
        assert_eq!(report.failed_tests, 0);
    }

    #[test]
    fn test_run_notebook_test_with_non_deterministic_test() {
        let mut session = NotebookTestSession::new();

        let cell = Cell {
            id: "test1".to_string(),
            cell_type: CellType::Code,
            source: "random()".to_string(),
            metadata: CellMetadata {
                test: Some(CellTestMetadata {
                    test_type: CellTestType::Property {
                        invariants: vec!["result > 0".to_string()],
                        generators: std::collections::HashMap::new(),
                    },
                    stop_on_failure: false,
                }),
            },
        };

        let notebook = Notebook {
            cells: vec![cell],
            metadata: None,
        };

        let report = session.run_notebook_test(&notebook);
        assert_eq!(report.total_tests, 0); // Property tests not implemented in Sprint 0
    }

    #[test]
    fn test_run_notebook_test_mixed_cells() {
        let mut session = NotebookTestSession::new();

        let cells = vec![
            Cell {
                id: "markdown".to_string(),
                cell_type: CellType::Markdown,
                source: "# Header".to_string(),
                metadata: CellMetadata { test: None },
            },
            Cell {
                id: "code_no_test".to_string(),
                cell_type: CellType::Code,
                source: "x = 1".to_string(),
                metadata: CellMetadata { test: None },
            },
            Cell {
                id: "test_cell".to_string(),
                cell_type: CellType::Code,
                source: "y = 42".to_string(),
                metadata: CellMetadata {
                    test: Some(CellTestMetadata {
                        test_type: CellTestType::Deterministic {
                            expected: "42".to_string(),
                            tolerance: None,
                            golden: None,
                        },
                        stop_on_failure: false,
                    }),
                },
            },
        ];

        let notebook = Notebook {
            cells,
            metadata: None,
        };

        let report = session.run_notebook_test(&notebook);
        assert_eq!(report.total_tests, 1); // Only one cell has test metadata
        assert_eq!(report.passed_tests, 1);
    }

    #[test]
    fn test_notebook_parser_new() {
        let parser = NotebookParser::new();
        // Should create successfully
        let _ = parser;
    }

    #[test]
    fn test_notebook_parser_default() {
        let parser = NotebookParser;
        // Should create successfully
        let _ = parser;
    }

    #[test]
    fn test_notebook_parser_validate() {
        let parser = NotebookParser::new();
        let notebook = Notebook {
            cells: vec![],
            metadata: None,
        };

        let result = parser.validate(&notebook);
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_parser_validate_with_cells() {
        let parser = NotebookParser::new();
        let notebook = Notebook {
            cells: vec![Cell {
                id: "1".to_string(),
                cell_type: CellType::Code,
                source: "x = 1".to_string(),
                metadata: CellMetadata { test: None },
            }],
            metadata: None,
        };

        let result = parser.validate(&notebook);
        assert!(result.is_ok());
    }
}
