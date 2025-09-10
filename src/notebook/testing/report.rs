use crate::notebook::testing::types::*;

impl TestReport {
    pub fn format_cli(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("Test Results: {} passed, {} failed, {} skipped\n", 
            self.passed_tests, self.failed_tests, self.skipped_tests));
        
        output.push_str(&format!("Execution time: {:.3}s\n", 
            self.execution_time.as_secs_f64()));
        
        if let Some(ref coverage) = self.coverage {
            output.push_str(&format!("Coverage: {:.0}% lines, {:.0}% branches\n",
                coverage.line_coverage * 100.0,
                coverage.branch_coverage * 100.0));
        }
        
        if !self.failures.is_empty() {
            output.push_str("\nFailures:\n");
            for failure in &self.failures {
                output.push_str(&format!("  Cell {}: {}\n", failure.cell_id, failure.message));
            }
        }
        
        output
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize report: {}", e))
    }
}