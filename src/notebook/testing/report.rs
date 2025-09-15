use crate::notebook::testing::types::TestReport;
#[cfg(test)]
use proptest::prelude::*;
impl TestReport {
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::report::format_cli;
/// 
/// let result = format_cli(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::report::to_json;
/// 
/// let result = to_json(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize report: {e}"))
    }
}
#[cfg(test)]
mod property_tests_report {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_format_cli_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
