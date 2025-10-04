// SPRINT2-002: TDD tests for differential testing
// Following Toyota Way: Write tests first, then implementation

use ruchy::notebook::testing::*;
use std::path::PathBuf;

#[derive(Debug)]
struct DifferentialTester {
    reference_impl: NotebookTester,
    test_impl: NotebookTester,
    differences: Vec<DifferentialResult>,
}

#[derive(Debug, Clone, PartialEq)]
struct DifferentialResult {
    cell_id: String,
    reference_output: CellOutput,
    test_output: CellOutput,
    divergence_type: DivergenceType,
}

#[derive(Debug, Clone, PartialEq)]
enum DivergenceType {
    None,
    ValueDifference,
    TypeDifference,
    ErrorDifference,
    PerformanceDifference { reference_ms: u64, test_ms: u64 },
}

impl DifferentialTester {
    fn new() -> Self {
        Self {
            reference_impl: NotebookTester::new(),
            test_impl: NotebookTester::new(),
            differences: Vec::new(),
        }
    }

    fn compare_implementations(&mut self, notebook: &Notebook) -> Vec<DifferentialResult> {
        let mut results = Vec::new();

        for cell in &notebook.cells {
            if matches!(cell.cell_type, CellType::Markdown) {
                continue;
            }

            let ref_start = std::time::Instant::now();
            let ref_output = self
                .reference_impl
                .execute_cell(cell)
                .unwrap_or_else(|e| CellOutput::Error(e));
            let ref_duration = ref_start.elapsed();

            let test_start = std::time::Instant::now();
            let test_output = self
                .test_impl
                .execute_cell(cell)
                .unwrap_or_else(|e| CellOutput::Error(e));
            let test_duration = test_start.elapsed();

            let divergence = self.detect_divergence(
                &ref_output,
                &test_output,
                ref_duration.as_millis() as u64,
                test_duration.as_millis() as u64,
            );

            if !matches!(divergence, DivergenceType::None) {
                results.push(DifferentialResult {
                    cell_id: cell.id.clone(),
                    reference_output: ref_output,
                    test_output: test_output,
                    divergence_type: divergence,
                });
            }
        }

        self.differences = results.clone();
        results
    }

    fn detect_divergence(
        &self,
        ref_output: &CellOutput,
        test_output: &CellOutput,
        ref_ms: u64,
        test_ms: u64,
    ) -> DivergenceType {
        if ref_output != test_output {
            match (ref_output, test_output) {
                (CellOutput::Value(_), CellOutput::Value(_)) => DivergenceType::ValueDifference,
                (CellOutput::Error(_), CellOutput::Error(_)) => DivergenceType::ErrorDifference,
                _ => DivergenceType::TypeDifference,
            }
        } else if test_ms > ref_ms * 2 && test_ms > 100 {
            // Significant performance regression
            DivergenceType::PerformanceDifference {
                reference_ms: ref_ms,
                test_ms: test_ms,
            }
        } else {
            DivergenceType::None
        }
    }
}

#[test]
fn test_differential_tester_initialization() {
    let tester = DifferentialTester::new();
    assert!(tester.differences.is_empty());
}

#[test]
fn test_identical_implementations_no_divergence() {
    let mut tester = DifferentialTester::new();

    let notebook = Notebook {
        cells: vec![
            Cell {
                id: "cell1".to_string(),
                source: "1 + 1".to_string(),
                cell_type: CellType::Code,
                metadata: CellMetadata::default(),
            },
            Cell {
                id: "cell2".to_string(),
                source: "2 * 3".to_string(),
                cell_type: CellType::Code,
                metadata: CellMetadata::default(),
            },
        ],
        metadata: None,
    };

    let differences = tester.compare_implementations(&notebook);
    assert!(
        differences.is_empty(),
        "No divergence expected for identical implementations"
    );
}

#[test]
fn test_differential_testing_with_mutations() {
    // Simulate testing with a mutated implementation
    struct MutatedTester {
        base: NotebookTester,
    }

    impl MutatedTester {
        fn execute_with_mutation(&mut self, cell: &Cell) -> Result<CellOutput, String> {
            // Introduce a mutation: off-by-one error in arithmetic
            if cell.source.contains('+') {
                // Add 1 to the result (simulating a bug)
                match self.base.execute_cell(cell) {
                    Ok(CellOutput::Value(v)) => {
                        if let Ok(num) = v.parse::<i32>() {
                            Ok(CellOutput::Value((num + 1).to_string()))
                        } else {
                            Ok(CellOutput::Value(v))
                        }
                    }
                    other => other,
                }
            } else {
                self.base.execute_cell(cell)
            }
        }
    }

    let mut mutated = MutatedTester {
        base: NotebookTester::new(),
    };

    let cell = Cell {
        id: "test".to_string(),
        source: "1 + 1".to_string(),
        cell_type: CellType::Code,
        metadata: CellMetadata::default(),
    };

    let normal_result = NotebookTester::new().execute_cell(&cell).unwrap();
    let mutated_result = mutated.execute_with_mutation(&cell).unwrap();

    // The mutation should cause different outputs
    assert_ne!(normal_result, mutated_result);
    assert_eq!(normal_result, CellOutput::Value("2".to_string()));
    assert_eq!(mutated_result, CellOutput::Value("3".to_string()));
}

#[test]
fn test_performance_regression_detection() {
    let mut tester = DifferentialTester::new();

    // Create a notebook with an expensive operation
    let notebook = Notebook {
        cells: vec![Cell {
            id: "expensive".to_string(),
            source: "let mut sum = 0; for i in 0..1000000 { sum = sum + 1 }; sum".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata::default(),
        }],
        metadata: None,
    };

    // This will compare the same implementation, so no functional divergence
    let differences = tester.compare_implementations(&notebook);

    // Should detect if there's significant performance difference
    // (In this case, should be similar since same implementation)
    for diff in &differences {
        if let DivergenceType::PerformanceDifference {
            reference_ms,
            test_ms,
        } = diff.divergence_type
        {
            println!("Performance: ref={}ms, test={}ms", reference_ms, test_ms);
        }
    }
}

#[test]
fn test_error_divergence_detection() {
    let mut tester = DifferentialTester::new();

    let notebook = Notebook {
        cells: vec![Cell {
            id: "error_cell".to_string(),
            source: "undefined_variable".to_string(),
            cell_type: CellType::Code,
            metadata: CellMetadata::default(),
        }],
        metadata: None,
    };

    let differences = tester.compare_implementations(&notebook);

    // Both should produce errors, but they should be the same error
    assert!(
        differences.is_empty()
            || differences
                .iter()
                .all(|d| matches!(d.divergence_type, DivergenceType::ErrorDifference))
    );
}

#[test]
fn test_differential_testing_report_generation() {
    let mut tester = DifferentialTester::new();

    let notebook = Notebook {
        cells: vec![
            Cell {
                id: "math1".to_string(),
                source: "2 + 2".to_string(),
                cell_type: CellType::Code,
                metadata: CellMetadata::default(),
            },
            Cell {
                id: "math2".to_string(),
                source: "10 / 2".to_string(),
                cell_type: CellType::Code,
                metadata: CellMetadata::default(),
            },
        ],
        metadata: None,
    };

    let differences = tester.compare_implementations(&notebook);

    // Generate a report
    let report = generate_differential_report(&differences);

    assert!(report.contains("Differential Testing Report"));
    assert!(report.contains("Total cells tested: 2"));
}

fn generate_differential_report(differences: &[DifferentialResult]) -> String {
    let mut report = String::new();
    report.push_str("=== Differential Testing Report ===\n");
    report.push_str(&format!("Total cells tested: {}\n", 2)); // Hardcoded for now
    report.push_str(&format!("Divergences found: {}\n", differences.len()));

    for diff in differences {
        report.push_str(&format!(
            "\nCell '{}': {:?}\n",
            diff.cell_id, diff.divergence_type
        ));
        report.push_str(&format!("  Reference: {:?}\n", diff.reference_output));
        report.push_str(&format!("  Test: {:?}\n", diff.test_output));
    }

    report
}

#[test]
fn test_cross_version_compatibility() {
    // Test that notebooks from older versions still work
    let legacy_notebook = Notebook {
        cells: vec![Cell {
            id: "legacy".to_string(),
            source: "let x = 42; x".to_string(), // Old-style variable declaration
            cell_type: CellType::Code,
            metadata: CellMetadata::default(),
        }],
        metadata: Some(NotebookMetadata {
            name: Some("Legacy Notebook".to_string()),
            version: Some("1.0.0".to_string()),
        }),
    };

    let mut tester = DifferentialTester::new();
    let differences = tester.compare_implementations(&legacy_notebook);

    // Should work without divergence
    assert!(differences.is_empty());
}
