// SPRINT2-002: Differential testing implementation
// PMAT Complexity: <10 per function
use crate::notebook::testing::tester::NotebookTester;
use crate::notebook::testing::types::{Cell, CellOutput, CellType, Notebook};
use std::time::{Duration, Instant};
#[derive(Debug, Clone)]
pub struct DifferentialConfig {
    pub performance_threshold_ms: u64,
    pub track_performance: bool,
}
impl Default for DifferentialConfig {
    fn default() -> Self {
        Self {
            performance_threshold_ms: 100,
            track_performance: true,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DifferentialResult {
    pub cell_id: String,
    pub reference_output: CellOutput,
    pub candidate_output: CellOutput,
    pub divergence: DivergenceType,
    pub reference_time: Duration,
    pub candidate_time: Duration,
}
#[derive(Debug, Clone, PartialEq)]
pub enum DivergenceType {
    None,
    OutputMismatch,
    TypeMismatch,
    PerformanceRegression,
    BothFailed,
}

/// Differential testing for comparing implementations
pub struct DifferentialTester {
    reference: NotebookTester,
    candidate: NotebookTester,
    config: DifferentialConfig,
}

impl Default for DifferentialTester {
    fn default() -> Self {
        Self::new()
    }
}

impl DifferentialTester {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::differential::DifferentialTester;
    ///
    /// let instance = DifferentialTester::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            reference: NotebookTester::new(),
            candidate: NotebookTester::new(),
            config: DifferentialConfig::default(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::differential::DifferentialTester;
    ///
    /// let mut instance = DifferentialTester::new();
    /// let result = instance.with_config();
    /// // Verify behavior
    /// ```
    pub fn with_config(config: DifferentialConfig) -> Self {
        Self {
            reference: NotebookTester::new(),
            candidate: NotebookTester::new(),
            config,
        }
    }
    /// Compare two implementations on a notebook
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::differential::DifferentialTester;
    ///
    /// let mut instance = DifferentialTester::new();
    /// let result = instance.compare();
    /// // Verify behavior
    /// ```
    pub fn compare(&mut self, notebook: &Notebook) -> Vec<DifferentialResult> {
        let mut results = Vec::new();
        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Markdown) {
                continue;
            }
            let result = self.compare_cell(cell);
            if !matches!(result.divergence, DivergenceType::None) {
                results.push(result);
            }
        }
        results
    }
    fn compare_cell(&mut self, cell: &Cell) -> DifferentialResult {
        // Execute on reference implementation
        let ref_start = Instant::now();
        let ref_output = self
            .reference
            .execute_cell(cell)
            .unwrap_or_else(CellOutput::Error);
        let ref_time = ref_start.elapsed();
        // Execute on candidate implementation
        let cand_start = Instant::now();
        let cand_output = self
            .candidate
            .execute_cell(cell)
            .unwrap_or_else(CellOutput::Error);
        let cand_time = cand_start.elapsed();
        // Determine divergence type
        let divergence = self.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        DifferentialResult {
            cell_id: cell.id.clone(),
            reference_output: ref_output,
            candidate_output: cand_output,
            divergence,
            reference_time: ref_time,
            candidate_time: cand_time,
        }
    }
    fn classify_divergence(
        &self,
        ref_output: &CellOutput,
        cand_output: &CellOutput,
        ref_time: Duration,
        cand_time: Duration,
    ) -> DivergenceType {
        // Check for output differences
        if ref_output != cand_output {
            match (ref_output, cand_output) {
                (CellOutput::Error(_), CellOutput::Error(_)) => DivergenceType::BothFailed,
                (CellOutput::Value(_), CellOutput::Value(_))
                | (CellOutput::DataFrame(_), CellOutput::DataFrame(_)) => {
                    DivergenceType::OutputMismatch
                }
                _ => DivergenceType::TypeMismatch,
            }
        } else if self.config.track_performance {
            // Check for performance regression
            let threshold = Duration::from_millis(self.config.performance_threshold_ms);
            if cand_time > ref_time + threshold && cand_time > ref_time * 2 {
                DivergenceType::PerformanceRegression
            } else {
                DivergenceType::None
            }
        } else {
            DivergenceType::None
        }
    }
    /// Generate a report of all divergences
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::differential::generate_report;
    ///
    /// let result = generate_report(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_report(&self, results: &[DifferentialResult]) -> String {
        let mut report = String::new();
        report.push_str("=== Differential Testing Report ===\n\n");
        if results.is_empty() {
            report.push_str("No divergences found. Implementations are consistent.\n");
        } else {
            report.push_str(&format!("Found {} divergences:\n\n", results.len()));
            for (i, result) in results.iter().enumerate() {
                report.push_str(&format!("{}. Cell '{}'\n", i + 1, result.cell_id));
                report.push_str(&format!("   Divergence: {:?}\n", result.divergence));
                if self.config.track_performance {
                    report.push_str(&format!(
                        "   Performance: ref={:?}, candidate={:?}\n",
                        result.reference_time, result.candidate_time
                    ));
                }
                if !matches!(result.divergence, DivergenceType::PerformanceRegression) {
                    report.push_str(&format!(
                        "   Reference: {:?}\n   Candidate: {:?}\n",
                        result.reference_output, result.candidate_output
                    ));
                }
                report.push('\n');
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::testing::types::CellMetadata;

    // EXTREME TDD: Comprehensive test coverage for differential testing system

    #[test]
    fn test_differential_config_default() {
        let config = DifferentialConfig::default();
        assert_eq!(config.performance_threshold_ms, 100);
        assert!(config.track_performance);
    }

    #[test]
    fn test_differential_config_custom() {
        let config = DifferentialConfig {
            performance_threshold_ms: 500,
            track_performance: false,
        };
        assert_eq!(config.performance_threshold_ms, 500);
        assert!(!config.track_performance);
    }

    #[test]
    fn test_differential_config_clone() {
        let config = DifferentialConfig {
            performance_threshold_ms: 250,
            track_performance: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.performance_threshold_ms, 250);
        assert!(cloned.track_performance);
    }

    #[test]
    fn test_divergence_type_enum_variants() {
        let variants = vec![
            DivergenceType::None,
            DivergenceType::OutputMismatch,
            DivergenceType::TypeMismatch,
            DivergenceType::PerformanceRegression,
            DivergenceType::BothFailed,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_divergence_type_debug_format() {
        assert_eq!(format!("{:?}", DivergenceType::None), "None");
        assert_eq!(
            format!("{:?}", DivergenceType::OutputMismatch),
            "OutputMismatch"
        );
        assert_eq!(
            format!("{:?}", DivergenceType::TypeMismatch),
            "TypeMismatch"
        );
        assert_eq!(
            format!("{:?}", DivergenceType::PerformanceRegression),
            "PerformanceRegression"
        );
        assert_eq!(format!("{:?}", DivergenceType::BothFailed), "BothFailed");
    }

    #[test]
    fn test_divergence_type_partial_eq() {
        assert_eq!(DivergenceType::None, DivergenceType::None);
        assert_eq!(
            DivergenceType::OutputMismatch,
            DivergenceType::OutputMismatch
        );
        assert_ne!(DivergenceType::None, DivergenceType::OutputMismatch);
        assert_ne!(DivergenceType::TypeMismatch, DivergenceType::BothFailed);
    }

    #[test]
    fn test_differential_result_creation() {
        let result = DifferentialResult {
            cell_id: "test_cell".to_string(),
            reference_output: CellOutput::Value("ref_value".to_string()),
            candidate_output: CellOutput::Value("cand_value".to_string()),
            divergence: DivergenceType::OutputMismatch,
            reference_time: Duration::from_millis(50),
            candidate_time: Duration::from_millis(75),
        };

        assert_eq!(result.cell_id, "test_cell");
        assert!(matches!(result.reference_output, CellOutput::Value(_)));
        assert!(matches!(result.candidate_output, CellOutput::Value(_)));
        assert_eq!(result.divergence, DivergenceType::OutputMismatch);
        assert_eq!(result.reference_time, Duration::from_millis(50));
        assert_eq!(result.candidate_time, Duration::from_millis(75));
    }

    #[test]
    fn test_differential_result_clone() {
        let result = DifferentialResult {
            cell_id: "clone_test".to_string(),
            reference_output: CellOutput::Error("error".to_string()),
            candidate_output: CellOutput::None,
            divergence: DivergenceType::TypeMismatch,
            reference_time: Duration::from_millis(100),
            candidate_time: Duration::from_millis(200),
        };

        let cloned = result.clone();
        assert_eq!(cloned.cell_id, "clone_test");
        assert_eq!(cloned.divergence, DivergenceType::TypeMismatch);
    }

    #[test]
    fn test_differential_result_partial_eq() {
        let result1 = DifferentialResult {
            cell_id: "test".to_string(),
            reference_output: CellOutput::Value("same".to_string()),
            candidate_output: CellOutput::Value("same".to_string()),
            divergence: DivergenceType::None,
            reference_time: Duration::from_millis(50),
            candidate_time: Duration::from_millis(50),
        };

        let result2 = DifferentialResult {
            cell_id: "test".to_string(),
            reference_output: CellOutput::Value("same".to_string()),
            candidate_output: CellOutput::Value("same".to_string()),
            divergence: DivergenceType::None,
            reference_time: Duration::from_millis(50),
            candidate_time: Duration::from_millis(50),
        };

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_differential_tester_new() {
        let tester = DifferentialTester::new();
        assert_eq!(tester.config.performance_threshold_ms, 100);
        assert!(tester.config.track_performance);
    }

    #[test]
    fn test_differential_tester_default() {
        let tester = DifferentialTester::default();
        assert_eq!(tester.config.performance_threshold_ms, 100);
        assert!(tester.config.track_performance);
    }

    #[test]
    fn test_differential_tester_with_config() {
        let config = DifferentialConfig {
            performance_threshold_ms: 300,
            track_performance: false,
        };
        let tester = DifferentialTester::with_config(config);
        assert_eq!(tester.config.performance_threshold_ms, 300);
        assert!(!tester.config.track_performance);
    }

    #[test]
    fn test_compare_empty_notebook() {
        let mut tester = DifferentialTester::new();
        let notebook = Notebook {
            cells: vec![],
            metadata: None,
        };

        let results = tester.compare(&notebook);
        assert!(results.is_empty());
    }

    #[test]
    fn test_compare_markdown_cells_skipped() {
        let mut tester = DifferentialTester::new();
        let markdown_cell = Cell {
            id: "md1".to_string(),
            source: "# Header".to_string(),
            cell_type: CellType::Markdown,
            metadata: CellMetadata { test: None },
        };
        let notebook = Notebook {
            cells: vec![markdown_cell],
            metadata: None,
        };

        let results = tester.compare(&notebook);
        assert!(results.is_empty());
    }

    #[test]
    fn test_classify_divergence_identical_outputs() {
        let tester = DifferentialTester::new();
        let ref_output = CellOutput::Value("same".to_string());
        let cand_output = CellOutput::Value("same".to_string());
        let ref_time = Duration::from_millis(50);
        let cand_time = Duration::from_millis(55);

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::None);
    }

    #[test]
    fn test_classify_divergence_output_mismatch_values() {
        let tester = DifferentialTester::new();
        let ref_output = CellOutput::Value("reference".to_string());
        let cand_output = CellOutput::Value("candidate".to_string());
        let ref_time = Duration::from_millis(50);
        let cand_time = Duration::from_millis(50);

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::OutputMismatch);
    }

    #[test]
    fn test_classify_divergence_type_mismatch() {
        let tester = DifferentialTester::new();
        let ref_output = CellOutput::Value("value".to_string());
        let cand_output = CellOutput::Error("error".to_string());
        let ref_time = Duration::from_millis(50);
        let cand_time = Duration::from_millis(50);

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::TypeMismatch);
    }

    #[test]
    fn test_classify_divergence_both_failed() {
        let tester = DifferentialTester::new();
        let ref_output = CellOutput::Error("ref error".to_string());
        let cand_output = CellOutput::Error("cand error".to_string());
        let ref_time = Duration::from_millis(50);
        let cand_time = Duration::from_millis(50);

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::BothFailed);
    }

    #[test]
    fn test_classify_divergence_performance_regression() {
        let mut config = DifferentialConfig::default();
        config.performance_threshold_ms = 50;
        let tester = DifferentialTester::with_config(config);

        let ref_output = CellOutput::Value("same".to_string());
        let cand_output = CellOutput::Value("same".to_string());
        let ref_time = Duration::from_millis(100);
        let cand_time = Duration::from_millis(300); // 3x slower + above threshold

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::PerformanceRegression);
    }

    #[test]
    fn test_classify_divergence_no_performance_regression_under_threshold() {
        let tester = DifferentialTester::new();
        let ref_output = CellOutput::Value("same".to_string());
        let cand_output = CellOutput::Value("same".to_string());
        let ref_time = Duration::from_millis(100);
        let cand_time = Duration::from_millis(150); // Only 1.5x slower

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::None);
    }

    #[test]
    fn test_classify_divergence_performance_tracking_disabled() {
        let config = DifferentialConfig {
            performance_threshold_ms: 50,
            track_performance: false,
        };
        let tester = DifferentialTester::with_config(config);

        let ref_output = CellOutput::Value("same".to_string());
        let cand_output = CellOutput::Value("same".to_string());
        let ref_time = Duration::from_millis(100);
        let cand_time = Duration::from_millis(1000); // Much slower but tracking disabled

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::None);
    }

    #[test]
    fn test_generate_report_no_divergences() {
        let tester = DifferentialTester::new();
        let results = vec![];

        let report = tester.generate_report(&results);
        assert!(report.contains("=== Differential Testing Report ==="));
        assert!(report.contains("No divergences found"));
        assert!(report.contains("Implementations are consistent"));
    }

    #[test]
    fn test_generate_report_with_divergences() {
        let tester = DifferentialTester::new();
        let result = DifferentialResult {
            cell_id: "cell1".to_string(),
            reference_output: CellOutput::Value("ref".to_string()),
            candidate_output: CellOutput::Value("cand".to_string()),
            divergence: DivergenceType::OutputMismatch,
            reference_time: Duration::from_millis(50),
            candidate_time: Duration::from_millis(75),
        };
        let results = vec![result];

        let report = tester.generate_report(&results);
        assert!(report.contains("=== Differential Testing Report ==="));
        assert!(report.contains("Found 1 divergences"));
        assert!(report.contains("Cell 'cell1'"));
        assert!(report.contains("OutputMismatch"));
        assert!(report.contains("Performance: ref="));
        assert!(report.contains("Reference: Value(\"ref\")"));
        assert!(report.contains("Candidate: Value(\"cand\")"));
    }

    #[test]
    fn test_generate_report_performance_regression_only() {
        let tester = DifferentialTester::new();
        let result = DifferentialResult {
            cell_id: "slow_cell".to_string(),
            reference_output: CellOutput::Value("same".to_string()),
            candidate_output: CellOutput::Value("same".to_string()),
            divergence: DivergenceType::PerformanceRegression,
            reference_time: Duration::from_millis(100),
            candidate_time: Duration::from_millis(500),
        };
        let results = vec![result];

        let report = tester.generate_report(&results);
        assert!(report.contains("PerformanceRegression"));
        assert!(report.contains("Performance: ref="));
        assert!(!report.contains("Reference: Value")); // Should not show outputs for perf regression
    }

    #[test]
    fn test_generate_report_multiple_divergences() {
        let tester = DifferentialTester::new();
        let results = vec![
            DifferentialResult {
                cell_id: "cell1".to_string(),
                reference_output: CellOutput::Value("ref1".to_string()),
                candidate_output: CellOutput::Value("cand1".to_string()),
                divergence: DivergenceType::OutputMismatch,
                reference_time: Duration::from_millis(50),
                candidate_time: Duration::from_millis(60),
            },
            DifferentialResult {
                cell_id: "cell2".to_string(),
                reference_output: CellOutput::Value("ok".to_string()),
                candidate_output: CellOutput::Error("fail".to_string()),
                divergence: DivergenceType::TypeMismatch,
                reference_time: Duration::from_millis(30),
                candidate_time: Duration::from_millis(40),
            },
        ];

        let report = tester.generate_report(&results);
        assert!(report.contains("Found 2 divergences"));
        assert!(report.contains("1. Cell 'cell1'"));
        assert!(report.contains("2. Cell 'cell2'"));
        assert!(report.contains("OutputMismatch"));
        assert!(report.contains("TypeMismatch"));
    }

    #[test]
    fn test_generate_report_no_performance_tracking() {
        let config = DifferentialConfig {
            performance_threshold_ms: 100,
            track_performance: false,
        };
        let tester = DifferentialTester::with_config(config);
        let result = DifferentialResult {
            cell_id: "cell1".to_string(),
            reference_output: CellOutput::Value("ref".to_string()),
            candidate_output: CellOutput::Value("cand".to_string()),
            divergence: DivergenceType::OutputMismatch,
            reference_time: Duration::from_millis(50),
            candidate_time: Duration::from_millis(75),
        };
        let results = vec![result];

        let report = tester.generate_report(&results);
        assert!(report.contains("OutputMismatch"));
        assert!(!report.contains("Performance:")); // Should not show performance when disabled
    }

    #[test]
    fn test_large_threshold_no_regression() {
        let config = DifferentialConfig {
            performance_threshold_ms: 1000, // Very large threshold
            track_performance: true,
        };
        let tester = DifferentialTester::with_config(config);

        let ref_output = CellOutput::Value("same".to_string());
        let cand_output = CellOutput::Value("same".to_string());
        let ref_time = Duration::from_millis(100);
        let cand_time = Duration::from_millis(300);

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::None);
    }

    #[test]
    fn test_edge_case_zero_times() {
        let tester = DifferentialTester::new();
        let ref_output = CellOutput::Value("fast".to_string());
        let cand_output = CellOutput::Value("fast".to_string());
        let ref_time = Duration::from_millis(0);
        let cand_time = Duration::from_millis(0);

        let divergence = tester.classify_divergence(&ref_output, &cand_output, ref_time, cand_time);
        assert_eq!(divergence, DivergenceType::None);
    }
}
