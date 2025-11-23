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
    /// let instance = TestConfig::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::types::TestConfig;
    ///
    /// let instance = TestConfig::new();
    /// // Verify behavior
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
    /// let mut instance = NotebookParser::new();
    /// let result = instance.parse();
    /// // Verify behavior
    /// ```
    pub fn parse(&self, content: &str) -> anyhow::Result<Notebook> {
        serde_json::from_str(content).map_err(|e| anyhow::anyhow!("Failed to parse notebook: {e}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // EXTREME TDD: Comprehensive test coverage for notebook testing types

    #[test]
    fn test_test_config_new() {
        let config = TestConfig::new();
        assert_eq!(config.tolerance, 1e-6);
        assert!(!config.coverage);
        assert!(!config.mutation);
        assert_eq!(config.golden_dir, PathBuf::from("test/golden"));
        assert_eq!(config.max_time, Duration::from_secs(30));
        assert_eq!(config.max_memory, 512 * 1024 * 1024);
        assert!(!config.update_golden);
    }

    #[test]
    fn test_test_config_default() {
        let config = TestConfig::default();
        assert_eq!(config.tolerance, 1e-6);
        assert!(!config.coverage);
        assert!(!config.mutation);
    }

    #[test]
    fn test_test_config_clone() {
        let config = TestConfig::new();
        let cloned = config.clone();
        assert_eq!(cloned.tolerance, config.tolerance);
        assert_eq!(cloned.coverage, config.coverage);
        assert_eq!(cloned.golden_dir, config.golden_dir);
    }

    #[test]
    fn test_test_config_custom() {
        let custom_config = TestConfig {
            tolerance: 1e-3,
            coverage: true,
            mutation: true,
            golden_dir: PathBuf::from("custom/golden"),
            max_time: Duration::from_secs(60),
            max_memory: 1024 * 1024 * 1024,
            update_golden: true,
        };
        assert_eq!(custom_config.tolerance, 1e-3);
        assert!(custom_config.coverage);
        assert!(custom_config.mutation);
        assert!(custom_config.update_golden);
    }

    #[test]
    fn test_test_result_pass() {
        let result = TestResult::Pass;
        assert_eq!(result, TestResult::Pass);
    }

    #[test]
    fn test_test_result_fail() {
        let result = TestResult::Fail("Error message".to_string());
        assert_eq!(result, TestResult::Fail("Error message".to_string()));
    }

    #[test]
    fn test_test_result_skip() {
        let result = TestResult::Skip;
        assert_eq!(result, TestResult::Skip);
    }

    #[test]
    fn test_test_result_numeric_divergence() {
        let result = TestResult::NumericDivergence { max_delta: 0.01 };
        assert_eq!(result, TestResult::NumericDivergence { max_delta: 0.01 });
    }

    #[test]
    fn test_test_result_type_mismatch() {
        let result = TestResult::TypeMismatch;
        assert_eq!(result, TestResult::TypeMismatch);
    }

    #[test]
    fn test_test_result_shape_mismatch() {
        let result = TestResult::ShapeMismatch;
        assert_eq!(result, TestResult::ShapeMismatch);
    }

    #[test]
    fn test_test_result_categorical_mismatch() {
        let result = TestResult::CategoricalMismatch {
            col: "column1".to_string(),
        };
        assert_eq!(
            result,
            TestResult::CategoricalMismatch {
                col: "column1".to_string()
            }
        );
    }

    #[test]
    fn test_test_result_timeout() {
        let result = TestResult::Timeout;
        assert_eq!(result, TestResult::Timeout);
    }

    #[test]
    fn test_test_result_clone() {
        let result = TestResult::Fail("test error".to_string());
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn test_cell_test_type_deterministic() {
        let test_type = CellTestType::Deterministic {
            expected: "42".to_string(),
            tolerance: Some(0.001),
            golden: Some(PathBuf::from("test.golden")),
        };

        if let CellTestType::Deterministic {
            expected,
            tolerance,
            golden,
        } = test_type
        {
            assert_eq!(expected, "42");
            assert_eq!(tolerance, Some(0.001));
            assert_eq!(golden, Some(PathBuf::from("test.golden")));
        } else {
            panic!("Expected Deterministic variant");
        }
    }

    #[test]
    fn test_cell_test_type_property() {
        let mut generators = HashMap::new();
        generators.insert("x".to_string(), "integer".to_string());

        let test_type = CellTestType::Property {
            invariants: vec!["x > 0".to_string()],
            generators,
        };

        if let CellTestType::Property {
            invariants,
            generators,
        } = test_type
        {
            assert_eq!(invariants.len(), 1);
            assert_eq!(generators.get("x"), Some(&"integer".to_string()));
        } else {
            panic!("Expected Property variant");
        }
    }

    #[test]
    fn test_cell_test_type_regression() {
        let test_type = CellTestType::Regression {
            baseline: PathBuf::from("baseline.json"),
            max_time_factor: 1.5,
            max_memory_factor: 2.0,
        };

        if let CellTestType::Regression {
            baseline,
            max_time_factor,
            max_memory_factor,
        } = test_type
        {
            assert_eq!(baseline, PathBuf::from("baseline.json"));
            assert_eq!(max_time_factor, 1.5);
            assert_eq!(max_memory_factor, 2.0);
        } else {
            panic!("Expected Regression variant");
        }
    }

    #[test]
    fn test_cell_test_type_skip() {
        let test_type = CellTestType::Skip;
        assert!(matches!(test_type, CellTestType::Skip));
    }

    #[test]
    fn test_cell_output_value() {
        let output = CellOutput::Value("test_value".to_string());
        assert_eq!(output, CellOutput::Value("test_value".to_string()));
    }

    #[test]
    fn test_cell_output_dataframe() {
        let df_data = DataFrameData {
            columns: vec!["col1".to_string(), "col2".to_string()],
            rows: vec![vec!["1".to_string(), "2".to_string()]],
        };
        let output = CellOutput::DataFrame(df_data.clone());
        assert_eq!(output, CellOutput::DataFrame(df_data));
    }

    #[test]
    fn test_cell_output_plot() {
        let plot_data = PlotData {
            format: "png".to_string(),
            data: vec![1, 2, 3, 4],
            perceptual_hash: Some("abc123".to_string()),
        };
        let output = CellOutput::Plot(plot_data.clone());
        assert_eq!(output, CellOutput::Plot(plot_data));
    }

    #[test]
    fn test_cell_output_error() {
        let output = CellOutput::Error("Runtime error".to_string());
        assert_eq!(output, CellOutput::Error("Runtime error".to_string()));
    }

    #[test]
    fn test_cell_output_html() {
        let output = CellOutput::Html("<div>HTML content</div>".to_string());
        assert_eq!(
            output,
            CellOutput::Html("<div>HTML content</div>".to_string())
        );
    }

    #[test]
    fn test_cell_output_none() {
        let output = CellOutput::None;
        assert_eq!(output, CellOutput::None);
    }

    #[test]
    fn test_dataframe_data_creation() {
        let df_data = DataFrameData {
            columns: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            rows: vec![
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
                vec!["4".to_string(), "5".to_string(), "6".to_string()],
            ],
        };

        assert_eq!(df_data.columns.len(), 3);
        assert_eq!(df_data.rows.len(), 2);
        assert_eq!(df_data.rows[0].len(), 3);
    }

    #[test]
    fn test_dataframe_data_clone() {
        let df_data = DataFrameData {
            columns: vec!["test".to_string()],
            rows: vec![vec!["value".to_string()]],
        };
        let cloned = df_data.clone();
        assert_eq!(df_data, cloned);
    }

    #[test]
    fn test_plot_data_creation() {
        let plot_data = PlotData {
            format: "svg".to_string(),
            data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header bytes
            perceptual_hash: Some("def456".to_string()),
        };

        assert_eq!(plot_data.format, "svg");
        assert_eq!(plot_data.data.len(), 4);
        assert_eq!(plot_data.perceptual_hash, Some("def456".to_string()));
    }

    #[test]
    fn test_plot_data_no_hash() {
        let plot_data = PlotData {
            format: "jpg".to_string(),
            data: vec![1, 2, 3],
            perceptual_hash: None,
        };

        assert_eq!(plot_data.format, "jpg");
        assert!(plot_data.perceptual_hash.is_none());
    }

    #[test]
    fn test_cell_creation() {
        let cell = Cell {
            id: "cell_1".to_string(),
            source: "print('hello')".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata { test: None },
        };

        assert_eq!(cell.id, "cell_1");
        assert_eq!(cell.source, "print('hello')");
        assert!(matches!(cell.cell_type, CellType::Code));
        assert!(cell.metadata.test.is_none());
    }

    #[test]
    fn test_cell_type_code() {
        let cell_type = CellType::Code;
        assert!(matches!(cell_type, CellType::Code));
    }

    #[test]
    fn test_cell_type_markdown() {
        let cell_type = CellType::Markdown;
        assert!(matches!(cell_type, CellType::Markdown));
    }

    #[test]
    fn test_cell_metadata_default() {
        let metadata = CellMetadata::default();
        assert!(metadata.test.is_none());
    }

    #[test]
    fn test_cell_metadata_with_test() {
        let test_metadata = CellTestMetadata {
            test_type: CellTestType::Skip,
            stop_on_failure: true,
        };
        let metadata = CellMetadata {
            test: Some(test_metadata),
        };
        assert!(metadata.test.is_some());
    }

    #[test]
    fn test_cell_test_metadata_creation() {
        let test_metadata = CellTestMetadata {
            test_type: CellTestType::Deterministic {
                expected: "output".to_string(),
                tolerance: None,
                golden: None,
            },
            stop_on_failure: false,
        };

        assert!(!test_metadata.stop_on_failure);
        assert!(matches!(
            test_metadata.test_type,
            CellTestType::Deterministic { .. }
        ));
    }

    #[test]
    fn test_notebook_creation() {
        let notebook = Notebook {
            cells: vec![],
            metadata: None,
        };

        assert!(notebook.cells.is_empty());
        assert!(notebook.metadata.is_none());
    }

    #[test]
    fn test_notebook_with_cells() {
        let cell = Cell {
            id: "test_cell".to_string(),
            source: "x = 1".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata::default(),
        };

        let notebook = Notebook {
            cells: vec![cell],
            metadata: Some(NotebookMetadata {
                name: Some("Test Notebook".to_string()),
                version: Some("1.0".to_string()),
            }),
        };

        assert_eq!(notebook.cells.len(), 1);
        assert!(notebook.metadata.is_some());
    }

    #[test]
    fn test_notebook_metadata_creation() {
        let metadata = NotebookMetadata {
            name: Some("My Notebook".to_string()),
            version: Some("2.1".to_string()),
        };

        assert_eq!(metadata.name, Some("My Notebook".to_string()));
        assert_eq!(metadata.version, Some("2.1".to_string()));
    }

    #[test]
    fn test_notebook_metadata_partial() {
        let metadata = NotebookMetadata {
            name: Some("Partial Notebook".to_string()),
            version: None,
        };

        assert_eq!(metadata.name, Some("Partial Notebook".to_string()));
        assert!(metadata.version.is_none());
    }

    #[test]
    fn test_notebook_parser_new() {
        let _parser = NotebookParser::new();
        // Parser should be created successfully
    }

    #[test]
    fn test_notebook_parser_default() {
        let _parser = NotebookParser;
        // Parser should be created successfully via default
    }

    #[test]
    fn test_notebook_parser_parse_valid_json() {
        let parser = NotebookParser::new();
        let json =
            r#"{"cells": [{"id": "1", "source": "test", "cell_type": "code", "metadata": {}}]}"#;

        let result = parser.parse(json);
        assert!(result.is_ok());

        let notebook = result.expect("operation should succeed in test");
        assert_eq!(notebook.cells.len(), 1);
        assert_eq!(notebook.cells[0].id, "1");
    }

    #[test]
    fn test_notebook_parser_parse_invalid_json() {
        let parser = NotebookParser::new();
        let invalid_json = r#"{"cells": [invalid json"#;

        let result = parser.parse(invalid_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse notebook"));
    }

    #[test]
    fn test_notebook_parser_validate_empty_cells() {
        let parser = NotebookParser::new();
        let notebook = Notebook {
            cells: vec![],
            metadata: None,
        };

        let result = parser.validate(&notebook);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Notebook has no cells"));
    }

    #[test]
    fn test_notebook_parser_validate_missing_cell_id() {
        let parser = NotebookParser::new();
        let cell = Cell {
            id: String::new(), // Empty ID
            source: "test".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata::default(),
        };
        let notebook = Notebook {
            cells: vec![cell],
            metadata: None,
        };

        let result = parser.validate(&notebook);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cell missing ID"));
    }

    #[test]
    fn test_notebook_parser_validate_valid_notebook() {
        let parser = NotebookParser::new();
        let cell = Cell {
            id: "valid_id".to_string(),
            source: "print('hello')".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata::default(),
        };
        let notebook = Notebook {
            cells: vec![cell],
            metadata: None,
        };

        let result = parser.validate(&notebook);
        assert!(result.is_ok());
    }

    #[test]
    fn test_test_report_creation() {
        let report = TestReport {
            total_tests: 10,
            passed_tests: 8,
            failed_tests: 2,
            skipped_tests: 0,
            execution_time: Duration::from_millis(500),
            coverage: None,
            failures: vec![],
            results: vec![],
        };

        assert_eq!(report.total_tests, 10);
        assert_eq!(report.passed_tests, 8);
        assert_eq!(report.failed_tests, 2);
        assert_eq!(report.skipped_tests, 0);
        assert!(report.coverage.is_none());
    }

    #[test]
    fn test_test_report_with_coverage() {
        let coverage = CoverageReport {
            line_coverage: 85.5,
            branch_coverage: 78.2,
            uncovered_sections: vec!["line 42".to_string()],
        };

        let report = TestReport {
            total_tests: 5,
            passed_tests: 5,
            failed_tests: 0,
            skipped_tests: 0,
            execution_time: Duration::from_millis(200),
            coverage: Some(coverage),
            failures: vec![],
            results: vec![TestResult::Pass, TestResult::Pass],
        };

        assert!(report.coverage.is_some());
        assert_eq!(report.results.len(), 2);
    }

    #[test]
    fn test_coverage_report_creation() {
        let coverage = CoverageReport {
            line_coverage: 92.3,
            branch_coverage: 88.7,
            uncovered_sections: vec![
                "function foo, line 15".to_string(),
                "branch in bar, line 28".to_string(),
            ],
        };

        assert_eq!(coverage.line_coverage, 92.3);
        assert_eq!(coverage.branch_coverage, 88.7);
        assert_eq!(coverage.uncovered_sections.len(), 2);
    }

    #[test]
    fn test_test_failure_creation() {
        let failure = TestFailure {
            cell_id: "cell_5".to_string(),
            expected: "42".to_string(),
            actual: "43".to_string(),
            message: "Values do not match".to_string(),
        };

        assert_eq!(failure.cell_id, "cell_5");
        assert_eq!(failure.expected, "42");
        assert_eq!(failure.actual, "43");
        assert_eq!(failure.message, "Values do not match");
    }

    #[test]
    fn test_serde_serialization_round_trip() {
        let config = TestConfig::new();

        // Serialize to JSON
        let json = serde_json::to_string(&config).expect("operation should succeed in test");
        assert!(json.contains("tolerance"));
        assert!(json.contains("1e-6"));

        // Deserialize back
        let deserialized: TestConfig =
            serde_json::from_str(&json).expect("operation should succeed in test");
        assert_eq!(deserialized.tolerance, config.tolerance);
        assert_eq!(deserialized.coverage, config.coverage);
    }

    #[test]
    fn test_cell_type_serde() {
        let code_type = CellType::Code;
        let markdown_type = CellType::Markdown;

        let code_json =
            serde_json::to_string(&code_type).expect("operation should succeed in test");
        let markdown_json =
            serde_json::to_string(&markdown_type).expect("operation should succeed in test");

        assert_eq!(code_json, "\"code\"");
        assert_eq!(markdown_json, "\"markdown\"");
    }

    #[test]
    fn test_comprehensive_notebook_serde() {
        let test_metadata = CellTestMetadata {
            test_type: CellTestType::Deterministic {
                expected: "result".to_string(),
                tolerance: Some(0.001),
                golden: None,
            },
            stop_on_failure: true,
        };

        let cell = Cell {
            id: "test_cell".to_string(),
            source: "x = 42".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata {
                test: Some(test_metadata),
            },
        };

        let notebook = Notebook {
            cells: vec![cell],
            metadata: Some(NotebookMetadata {
                name: Some("Test".to_string()),
                version: Some("1.0".to_string()),
            }),
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&notebook).expect("operation should succeed in test");
        let deserialized: Notebook =
            serde_json::from_str(&json).expect("operation should succeed in test");

        assert_eq!(deserialized.cells.len(), 1);
        assert_eq!(deserialized.cells[0].id, "test_cell");
        assert!(deserialized.metadata.is_some());
    }
}
