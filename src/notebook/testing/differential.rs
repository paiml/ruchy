// SPRINT2-002: Differential testing implementation
// PMAT Complexity: <10 per function
use crate::notebook::testing::types::{CellOutput, Notebook, CellType, Cell};
use crate::notebook::testing::tester::NotebookTester;
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
let instance = DifferentialTester::new();
// Verify behavior
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
let mut instance = DifferentialTester::new();
let result = instance.with_config();
// Verify behavior
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
let mut instance = DifferentialTester::new();
let result = instance.compare();
// Verify behavior
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
        let ref_output = self.reference.execute_cell(cell)
            .unwrap_or_else(CellOutput::Error);
        let ref_time = ref_start.elapsed();
        // Execute on candidate implementation
        let cand_start = Instant::now();
        let cand_output = self.candidate.execute_cell(cell)
            .unwrap_or_else(CellOutput::Error);
        let cand_time = cand_start.elapsed();
        // Determine divergence type
        let divergence = self.classify_divergence(
            &ref_output,
            &cand_output,
            ref_time,
            cand_time
        );
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
                (CellOutput::Value(_), CellOutput::Value(_)) |
                (CellOutput::DataFrame(_), CellOutput::DataFrame(_)) => {
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
