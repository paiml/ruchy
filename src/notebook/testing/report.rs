/// Test report generation for notebook testing framework
#[derive(Debug, Clone)]
pub struct TestReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub coverage_percentage: f64,
}

impl TestReport {
    /// Create a new test report
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::report::TestReport;
    ///
    /// let report = TestReport::new(100, 95, 5, 80.5);
    /// assert_eq!(report.total_tests, 100);
    /// ```
    pub fn new(total: usize, passed: usize, failed: usize, coverage: f64) -> Self {
        Self {
            total_tests: total,
            passed_tests: passed,
            failed_tests: failed,
            coverage_percentage: coverage,
        }
    }

    /// Get success rate
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::report::TestReport;
    ///
    /// let report = TestReport::new(100, 95, 5, 80.5);
    /// assert_eq!(report.success_rate(), 0.95);
    /// ```
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.passed_tests as f64 / self.total_tests as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // EXTREME TDD: Comprehensive test coverage for test report system

    #[test]
    fn test_test_report_new() {
        let report = TestReport::new(100, 85, 15, 75.5);
        assert_eq!(report.total_tests, 100);
        assert_eq!(report.passed_tests, 85);
        assert_eq!(report.failed_tests, 15);
        assert_eq!(report.coverage_percentage, 75.5);
    }

    #[test]
    fn test_test_report_clone() {
        let report = TestReport::new(50, 40, 10, 60.0);
        let cloned = report.clone();

        assert_eq!(cloned.total_tests, 50);
        assert_eq!(cloned.passed_tests, 40);
        assert_eq!(cloned.failed_tests, 10);
        assert_eq!(cloned.coverage_percentage, 60.0);
    }

    #[test]
    fn test_success_rate_normal() {
        let report = TestReport::new(100, 85, 15, 75.0);
        assert_eq!(report.success_rate(), 0.85);
    }

    #[test]
    fn test_success_rate_perfect() {
        let report = TestReport::new(100, 100, 0, 100.0);
        assert_eq!(report.success_rate(), 1.0);
    }

    #[test]
    fn test_success_rate_zero_tests() {
        let report = TestReport::new(0, 0, 0, 0.0);
        assert_eq!(report.success_rate(), 0.0);
    }

    #[test]
    fn test_success_rate_all_failed() {
        let report = TestReport::new(50, 0, 50, 25.0);
        assert_eq!(report.success_rate(), 0.0);
    }

    #[test]
    fn test_debug_format() {
        let report = TestReport::new(10, 8, 2, 80.0);
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("total_tests: 10"));
        assert!(debug_str.contains("passed_tests: 8"));
        assert!(debug_str.contains("failed_tests: 2"));
        assert!(debug_str.contains("coverage_percentage: 80"));
    }
}
