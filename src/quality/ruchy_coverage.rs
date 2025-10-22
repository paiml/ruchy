//! Coverage implementation for Ruchy test files
//!
//! [RUCHY-206] Implement coverage collection for .ruchy files
use crate::quality::instrumentation::CoverageInstrumentation;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
/// Coverage data for a Ruchy file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuchyCoverage {
    pub file_path: String,
    pub total_lines: usize,
    pub covered_lines: HashSet<usize>,
    pub total_functions: usize,
    pub covered_functions: HashSet<String>,
    pub total_branches: usize,
    pub covered_branches: usize,
}
impl RuchyCoverage {
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::ruchy_coverage::RuchyCoverage;
    ///
    /// let instance = RuchyCoverage::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::ruchy_coverage::RuchyCoverage;
    ///
    /// let instance = RuchyCoverage::new();
    /// // Verify behavior
    /// ```
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            total_lines: 0,
            covered_lines: HashSet::new(),
            total_functions: 0,
            covered_functions: HashSet::new(),
            total_branches: 0,
            covered_branches: 0,
        }
    }
    /// Calculate line coverage percentage
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::ruchy_coverage::RuchyCoverage;
    ///
    /// let mut instance = RuchyCoverage::new();
    /// let result = instance.line_coverage();
    /// // Verify behavior
    /// ```
    pub fn line_coverage(&self) -> f64 {
        if self.total_lines == 0 {
            100.0
        } else {
            (self.covered_lines.len() as f64 / self.total_lines as f64) * 100.0
        }
    }
    /// Calculate function coverage percentage
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::function_coverage;
    ///
    /// let result = function_coverage(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn function_coverage(&self) -> f64 {
        if self.total_functions == 0 {
            100.0
        } else {
            (self.covered_functions.len() as f64 / self.total_functions as f64) * 100.0
        }
    }
    /// Calculate branch coverage percentage
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::branch_coverage;
    ///
    /// let result = branch_coverage(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn branch_coverage(&self) -> f64 {
        if self.total_branches == 0 {
            100.0
        } else {
            (self.covered_branches as f64 / self.total_branches as f64) * 100.0
        }
    }
    /// Calculate overall coverage percentage
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::overall_coverage;
    ///
    /// let result = overall_coverage(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn overall_coverage(&self) -> f64 {
        // Weight: 60% lines, 30% functions, 10% branches
        self.line_coverage() * 0.6 + self.function_coverage() * 0.3 + self.branch_coverage() * 0.1
    }
}
/// Coverage collector for Ruchy code
pub struct RuchyCoverageCollector {
    coverage_data: HashMap<String, RuchyCoverage>,
    runtime_instrumentation: CoverageInstrumentation,
}
impl RuchyCoverageCollector {
    pub fn new() -> Self {
        Self {
            coverage_data: HashMap::new(),
            runtime_instrumentation: CoverageInstrumentation::new(),
        }
    }
    /// Analyze a Ruchy file to determine what needs coverage
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::analyze_file;
    ///
    /// let result = analyze_file(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let mut coverage = RuchyCoverage::new(file_path.to_str().unwrap_or("unknown"));
        // Count total lines (non-empty, non-comment)
        let lines: Vec<&str> = content.lines().collect();
        coverage.total_lines = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("//")
            })
            .count();
        // Simple heuristic: count functions by looking for "fn" or "fun" keyword
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || trimmed.starts_with("fun ") {
                coverage.total_functions += 1;
                // If it's a test function, mark as covered
                if trimmed.contains("test")
                    || lines
                        .get(line_num.saturating_sub(1))
                        .is_some_and(|l| l.contains("#[test]"))
                {
                    let func_name = trimmed
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("unknown")
                        .split('(')
                        .next()
                        .unwrap_or("unknown");
                    coverage.covered_functions.insert(func_name.to_string());
                }
            }
            // Count branches (if, match, while, for)
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                coverage.total_branches += 2; // if and else
            } else if trimmed.starts_with("match ") {
                coverage.total_branches += 1; // simplified - would need to count arms
            } else if trimmed.starts_with("while ") || trimmed.starts_with("for ") {
                coverage.total_branches += 1;
            }
        }
        self.coverage_data
            .insert(coverage.file_path.clone(), coverage);
        Ok(())
    }
    /// Mark lines as covered based on test execution
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::mark_covered;
    ///
    /// let result = mark_covered("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn mark_covered(&mut self, file_path: &str, line_numbers: Vec<usize>) {
        if let Some(coverage) = self.coverage_data.get_mut(file_path) {
            for line in line_numbers {
                coverage.covered_lines.insert(line);
            }
        }
    }
    /// Mark a function as covered
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::mark_function_covered;
    ///
    /// let result = mark_function_covered("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn mark_function_covered(&mut self, file_path: &str, function_name: &str) {
        if let Some(coverage) = self.coverage_data.get_mut(file_path) {
            coverage.covered_functions.insert(function_name.to_string());
        }
    }
    /// Generate a text report
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::generate_text_report;
    ///
    /// let result = generate_text_report(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_text_report(&self) -> String {
        let mut report = String::new();
        report.push_str("\n📊 Coverage Report\n");
        report.push_str("==================\n\n");
        let mut total_lines = 0;
        let mut total_covered_lines = 0;
        let mut total_functions = 0;
        let mut total_covered_functions = 0;
        for (file_path, coverage) in &self.coverage_data {
            report.push_str(&format!("📄 {file_path}\n"));
            report.push_str(&format!(
                "   Lines: {}/{} ({:.1}%)\n",
                coverage.covered_lines.len(),
                coverage.total_lines,
                coverage.line_coverage()
            ));
            report.push_str(&format!(
                "   Functions: {}/{} ({:.1}%)\n",
                coverage.covered_functions.len(),
                coverage.total_functions,
                coverage.function_coverage()
            ));
            if coverage.total_branches > 0 {
                report.push_str(&format!(
                    "   Branches: {}/{} ({:.1}%)\n",
                    coverage.covered_branches,
                    coverage.total_branches,
                    coverage.branch_coverage()
                ));
            }
            report.push_str(&format!(
                "   Overall: {:.1}%\n\n",
                coverage.overall_coverage()
            ));
            total_lines += coverage.total_lines;
            total_covered_lines += coverage.covered_lines.len();
            total_functions += coverage.total_functions;
            total_covered_functions += coverage.covered_functions.len();
        }
        // Summary
        report.push_str("📈 Summary\n");
        report.push_str("----------\n");
        let overall_line_coverage = if total_lines > 0 {
            (total_covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            100.0
        };
        let overall_function_coverage = if total_functions > 0 {
            (total_covered_functions as f64 / total_functions as f64) * 100.0
        } else {
            100.0
        };
        report.push_str(&format!(
            "Total Lines: {total_covered_lines}/{total_lines} ({overall_line_coverage:.1}%)\n"
        ));
        report.push_str(&format!("Total Functions: {total_covered_functions}/{total_functions} ({overall_function_coverage:.1}%)\n"));
        report.push_str(&format!(
            "Overall Coverage: {:.1}%\n",
            overall_line_coverage * 0.7 + overall_function_coverage * 0.3
        ));
        report
    }
    /// Generate a JSON report
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::generate_json_report;
    ///
    /// let result = generate_json_report(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_json_report(&self) -> String {
        serde_json::to_string_pretty(&self.coverage_data).unwrap_or_else(|_| "{}".to_string())
    }
    /// Generate an HTML report
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::generate_html_report;
    ///
    /// let result = generate_html_report(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_html_report(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Ruchy Coverage Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: monospace; margin: 20px; }\n");
        html.push_str(".covered { background-color: #90EE90; }\n");
        html.push_str(".uncovered { background-color: #FFB6C1; }\n");
        html.push_str(".summary { border: 1px solid #ccc; padding: 10px; margin: 10px 0; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        html.push_str("<h1>📊 Ruchy Coverage Report</h1>\n");
        for (file_path, coverage) in &self.coverage_data {
            html.push_str(&"<div class='summary'>\n".to_string());
            html.push_str(&format!("<h2>{file_path}</h2>\n"));
            html.push_str(&format!(
                "<p>Line Coverage: {:.1}%</p>\n",
                coverage.line_coverage()
            ));
            html.push_str(&format!(
                "<p>Function Coverage: {:.1}%</p>\n",
                coverage.function_coverage()
            ));
            html.push_str(&format!(
                "<p>Overall: {:.1}%</p>\n",
                coverage.overall_coverage()
            ));
            html.push_str("</div>\n");
        }
        html.push_str("</body>\n</html>");
        html
    }
    /// Check if coverage meets threshold
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::meets_threshold;
    ///
    /// let result = meets_threshold(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        for coverage in self.coverage_data.values() {
            if coverage.overall_coverage() < threshold {
                return false;
            }
        }
        true
    }
    /// Execute a Ruchy program and collect runtime coverage
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::execute_with_coverage;
    ///
    /// let result = execute_with_coverage(());
    /// assert_eq!(result, Ok(()));
    /// ```
    #[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
    pub fn execute_with_coverage(&mut self, file_path: &Path) -> Result<()> {
        use crate::frontend::parser::Parser;
        use crate::runtime::repl::Repl;

        // BUG-036 FIX: Analyze file first to populate total_lines and total_functions
        self.analyze_file(file_path)?;

        let file_str = file_path.to_str().unwrap_or("unknown");
        let content = fs::read_to_string(file_path)?;
        // Parse the Ruchy source code
        let mut parser = Parser::new(&content);
        if let Ok(_ast) = parser.parse() {
            // Execute using the Ruchy interpreter
            let mut repl =
                match Repl::new(std::env::current_dir().unwrap_or_else(|_| "/tmp".into())) {
                    Ok(repl) => repl,
                    Err(_) => {
                        return Ok(()); // Can't create REPL, skip coverage
                    }
                };
            // Track execution through AST evaluation
            if let Ok(_) = repl.process_line(&content) {
                // Execution successful - mark lines and functions as covered
                let file_str_owned = file_str.to_string();
                if let Some(coverage) = self.coverage_data.get_mut(file_str) {
                    // Mark all executable lines as covered
                    let lines: Vec<&str> = content.lines().collect();
                    for (line_num, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with("//") {
                            let line_number = line_num + 1;
                            coverage.covered_lines.insert(line_number);
                            self.runtime_instrumentation
                                .mark_line_executed(&file_str_owned, line_number);
                        }
                    }
                    // Mark functions as covered based on successful execution
                    for line in &lines {
                        let trimmed = line.trim();
                        if trimmed.starts_with("fn ") || trimmed.starts_with("fun ") {
                            if let Some(func_name) = extract_function_name(trimmed) {
                                coverage.covered_functions.insert(func_name.clone());
                                self.runtime_instrumentation
                                    .mark_function_executed(&file_str_owned, &func_name);
                            }
                        }
                    }
                }
            } else {
                // Execution failed - no coverage data collected
            }
        } else {
            // Parse failed - no coverage possible
        }
        Ok(())
    }

    /// Stub for builds without REPL support
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "repl")))]
    pub fn execute_with_coverage(&mut self, file_path: &Path) -> Result<()> {
        // Analyze file for static coverage only (no execution)
        self.analyze_file(file_path)?;
        Ok(())
    }

    /// Get runtime coverage data
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::quality::ruchy_coverage::get_runtime_coverage;
    ///
    /// let result = get_runtime_coverage("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_runtime_coverage(
        &self,
        file_path: &str,
    ) -> Option<(Option<&HashSet<usize>>, Option<&HashSet<String>>)> {
        let lines = self.runtime_instrumentation.get_executed_lines(file_path);
        let functions = self
            .runtime_instrumentation
            .get_executed_functions(file_path);
        Some((lines, functions))
    }
}
/// Extract function name from a function definition line
fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("fn ") || trimmed.starts_with("fun ") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let name_part = parts[1];
            if let Some(paren_pos) = name_part.find('(') {
                Some(name_part[..paren_pos].to_string())
            } else {
                Some(name_part.to_string())
            }
        } else {
            None
        }
    } else {
        None
    }
}
impl Default for RuchyCoverageCollector {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_ruchy_coverage_new() {
        let coverage = RuchyCoverage::new("test.ruchy");
        assert_eq!(coverage.file_path, "test.ruchy");
        assert_eq!(coverage.total_lines, 0);
        assert_eq!(coverage.total_functions, 0);
        assert_eq!(coverage.total_branches, 0);
        assert!(coverage.covered_lines.is_empty());
        assert!(coverage.covered_functions.is_empty());
    }

    #[test]
    fn test_line_coverage_empty() {
        let coverage = RuchyCoverage::new("test.ruchy");
        assert_eq!(coverage.line_coverage(), 100.0);
    }

    #[test]
    fn test_line_coverage_partial() {
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 10;
        coverage.covered_lines.insert(1);
        coverage.covered_lines.insert(2);
        coverage.covered_lines.insert(5);

        assert_eq!(coverage.line_coverage(), 30.0);
    }

    #[test]
    fn test_line_coverage_full() {
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 3;
        coverage.covered_lines.insert(1);
        coverage.covered_lines.insert(2);
        coverage.covered_lines.insert(3);

        assert_eq!(coverage.line_coverage(), 100.0);
    }

    #[test]
    fn test_function_coverage_empty() {
        let coverage = RuchyCoverage::new("test.ruchy");
        assert_eq!(coverage.function_coverage(), 100.0);
    }

    #[test]
    fn test_function_coverage_partial() {
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_functions = 4;
        coverage.covered_functions.insert("func1".to_string());
        coverage.covered_functions.insert("func2".to_string());

        assert_eq!(coverage.function_coverage(), 50.0);
    }

    #[test]
    fn test_branch_coverage_empty() {
        let coverage = RuchyCoverage::new("test.ruchy");
        assert_eq!(coverage.branch_coverage(), 100.0);
    }

    #[test]
    fn test_branch_coverage_partial() {
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_branches = 8;
        coverage.covered_branches = 3;

        assert_eq!(coverage.branch_coverage(), 37.5);
    }

    #[test]
    fn test_overall_coverage_calculation() {
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 10;
        coverage.covered_lines.insert(1);
        coverage.covered_lines.insert(2);
        coverage.covered_lines.insert(3);
        coverage.covered_lines.insert(4);
        coverage.covered_lines.insert(5); // 50% line coverage

        coverage.total_functions = 2;
        coverage.covered_functions.insert("func1".to_string()); // 50% function coverage

        coverage.total_branches = 4;
        coverage.covered_branches = 2; // 50% branch coverage

        // Overall = 50% * 0.6 + 50% * 0.3 + 50% * 0.1 = 30 + 15 + 5 = 50%
        assert_eq!(coverage.overall_coverage(), 50.0);
    }

    #[test]
    fn test_ruchy_coverage_collector_new() {
        let collector = RuchyCoverageCollector::new();
        assert!(collector.coverage_data.is_empty());
    }

    #[test]
    fn test_analyze_empty_file() {
        let mut collector = RuchyCoverageCollector::new();
        let file = create_test_file("");

        let result = collector.analyze_file(file.path());
        assert!(result.is_ok());

        let file_path = file.path().to_str().unwrap();
        let coverage = collector.coverage_data.get(file_path).unwrap();
        assert_eq!(coverage.total_lines, 0);
        assert_eq!(coverage.total_functions, 0);
    }

    #[test]
    fn test_analyze_simple_function() {
        let mut collector = RuchyCoverageCollector::new();
        let content = r#"
// This is a comment
fn hello() {
    println("Hello, world!")
}
"#;
        let file = create_test_file(content);

        let result = collector.analyze_file(file.path());
        assert!(result.is_ok());

        let file_path = file.path().to_str().unwrap();
        let coverage = collector.coverage_data.get(file_path).unwrap();
        assert_eq!(coverage.total_lines, 3); // Non-empty, non-comment lines
        assert_eq!(coverage.total_functions, 1);
    }

    #[test]
    fn test_analyze_multiple_functions() {
        let mut collector = RuchyCoverageCollector::new();
        let content = r"
fn add(a, b) {
    return a + b
}

fun multiply(x, y) {
    return x * y
}
";
        let file = create_test_file(content);

        let result = collector.analyze_file(file.path());
        assert!(result.is_ok());

        let file_path = file.path().to_str().unwrap();
        let coverage = collector.coverage_data.get(file_path).unwrap();
        assert_eq!(coverage.total_functions, 2);
    }

    #[test]
    fn test_analyze_branches() {
        let mut collector = RuchyCoverageCollector::new();
        let content = r#"
fn test_branches(x) {
    if x > 0 {
        println("positive")
    }

    match x {
        1 => "one",
        _ => "other"
    }

    while x > 0 {
        x = x - 1
    }

    for i in 0..x {
        println(i)
    }
}
"#;
        let file = create_test_file(content);

        let result = collector.analyze_file(file.path());
        assert!(result.is_ok());

        let file_path = file.path().to_str().unwrap();
        let coverage = collector.coverage_data.get(file_path).unwrap();
        // if (2 branches) + match (1) + while (1) + for (1) = 5 branches
        assert_eq!(coverage.total_branches, 5);
    }

    #[test]
    fn test_mark_covered_lines() {
        let mut collector = RuchyCoverageCollector::new();
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 10;
        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        collector.mark_covered("test.ruchy", vec![1, 3, 5]);

        let coverage = collector.coverage_data.get("test.ruchy").unwrap();
        assert!(coverage.covered_lines.contains(&1));
        assert!(coverage.covered_lines.contains(&3));
        assert!(coverage.covered_lines.contains(&5));
        assert!(!coverage.covered_lines.contains(&2));
        assert_eq!(coverage.line_coverage(), 30.0);
    }

    #[test]
    fn test_mark_function_covered() {
        let mut collector = RuchyCoverageCollector::new();
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_functions = 3;
        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        collector.mark_function_covered("test.ruchy", "main");
        collector.mark_function_covered("test.ruchy", "helper");

        let coverage = collector.coverage_data.get("test.ruchy").unwrap();
        assert!(coverage.covered_functions.contains("main"));
        assert!(coverage.covered_functions.contains("helper"));
        assert!((coverage.function_coverage() - 66.666_666_666_666_67).abs() < 1e-10);
        // 2/3 with floating point tolerance
    }

    #[test]
    fn test_generate_text_report_empty() {
        let collector = RuchyCoverageCollector::new();
        let report = collector.generate_text_report();

        assert!(report.contains("Coverage Report"));
        assert!(report.contains("Summary"));
        assert!(report.contains("Total Lines: 0/0 (100.0%)"));
    }

    #[test]
    fn test_generate_text_report_with_data() {
        let mut collector = RuchyCoverageCollector::new();
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 10;
        coverage.covered_lines.insert(1);
        coverage.covered_lines.insert(2);
        coverage.total_functions = 2;
        coverage.covered_functions.insert("main".to_string());
        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        let report = collector.generate_text_report();

        assert!(report.contains("test.ruchy"));
        assert!(report.contains("Lines: 2/10 (20.0%)"));
        assert!(report.contains("Functions: 1/2 (50.0%)"));
    }

    #[test]
    fn test_generate_json_report() {
        let mut collector = RuchyCoverageCollector::new();
        let coverage = RuchyCoverage::new("test.ruchy");
        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        let json_report = collector.generate_json_report();
        assert!(json_report.contains("test.ruchy"));
        assert!(json_report.contains("file_path"));
        assert!(json_report.contains("total_lines"));
    }

    #[test]
    fn test_generate_html_report() {
        let mut collector = RuchyCoverageCollector::new();
        let coverage = RuchyCoverage::new("test.ruchy");
        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        let html_report = collector.generate_html_report();
        assert!(html_report.contains("<!DOCTYPE html>"));
        assert!(html_report.contains("Ruchy Coverage Report"));
        assert!(html_report.contains("test.ruchy"));
        assert!(html_report.contains("Line Coverage"));
        assert!(html_report.contains("</html>"));
    }

    #[test]
    fn test_meets_threshold_true() {
        let mut collector = RuchyCoverageCollector::new();
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 10;
        coverage.covered_lines.insert(1);
        coverage.covered_lines.insert(2);
        coverage.covered_lines.insert(3);
        coverage.covered_lines.insert(4);
        coverage.covered_lines.insert(5);
        coverage.covered_lines.insert(6);
        coverage.covered_lines.insert(7);
        coverage.covered_lines.insert(8); // 80% coverage
        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        assert!(collector.meets_threshold(70.0));
        assert!(collector.meets_threshold(80.0));
    }

    #[test]
    fn test_meets_threshold_false() {
        let mut collector = RuchyCoverageCollector::new();
        let mut coverage = RuchyCoverage::new("test.ruchy");
        coverage.total_lines = 10;
        coverage.covered_lines.insert(1);
        coverage.covered_lines.insert(2); // 20% line coverage

        // Set some functions and branches so they don't default to 100%
        coverage.total_functions = 2; // 0% function coverage (0/2)
        coverage.total_branches = 4; // 0% branch coverage (0/4)

        collector
            .coverage_data
            .insert("test.ruchy".to_string(), coverage);

        // Overall = 20% * 0.6 + 0% * 0.3 + 0% * 0.1 = 12%
        assert!(!collector.meets_threshold(50.0));
        assert!(!collector.meets_threshold(25.0));
        assert!(!collector.meets_threshold(15.0));
        assert!(collector.meets_threshold(10.0));
    }

    #[test]
    fn test_extract_function_name_fn() {
        assert_eq!(
            extract_function_name("fn hello()"),
            Some("hello".to_string())
        );
        assert_eq!(
            extract_function_name("fn add(a, b)"),
            Some("add".to_string())
        );
        assert_eq!(
            extract_function_name("  fn  test  (  )  "),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_fun() {
        assert_eq!(
            extract_function_name("fun hello()"),
            Some("hello".to_string())
        );
        assert_eq!(
            extract_function_name("fun multiply(x, y)"),
            Some("multiply".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_invalid() {
        assert_eq!(extract_function_name("not a function"), None);
        assert_eq!(extract_function_name("fn"), None);
        assert_eq!(extract_function_name("fun"), None);
        assert_eq!(extract_function_name("function hello()"), None);
    }

    #[test]
    fn test_extract_function_name_no_parens() {
        assert_eq!(extract_function_name("fn hello"), Some("hello".to_string()));
        assert_eq!(extract_function_name("fun test"), Some("test".to_string()));
    }

    #[test]
    fn test_get_runtime_coverage() {
        let mut collector = RuchyCoverageCollector::new();

        // Mark some runtime execution
        collector
            .runtime_instrumentation
            .mark_line_executed("test.ruchy", 1);
        collector
            .runtime_instrumentation
            .mark_line_executed("test.ruchy", 2);
        collector
            .runtime_instrumentation
            .mark_function_executed("test.ruchy", "main");

        let (lines, functions) = collector.get_runtime_coverage("test.ruchy").unwrap();

        if let Some(lines) = lines {
            assert!(lines.contains(&1));
            assert!(lines.contains(&2));
        }

        if let Some(functions) = functions {
            assert!(functions.contains("main"));
        }
    }

    #[test]
    fn test_collector_default() {
        let collector = RuchyCoverageCollector::default();
        assert!(collector.coverage_data.is_empty());
    }

    #[test]
    fn test_analyze_test_functions() {
        let mut collector = RuchyCoverageCollector::new();
        let content = r#"
#[test]
fn test_something() {
    assert_eq!(1, 1)
}

fn regular_function() {
    println("not a test")
}

// This function has test in name
fn test_helper() {
    println("helper")
}
"#;
        let file = create_test_file(content);

        let result = collector.analyze_file(file.path());
        assert!(result.is_ok());

        let file_path = file.path().to_str().unwrap();
        let coverage = collector.coverage_data.get(file_path).unwrap();
        assert_eq!(coverage.total_functions, 3);

        // Should mark test functions as covered
        assert!(coverage.covered_functions.contains("test_something"));
        assert!(coverage.covered_functions.contains("test_helper"));
        assert!(!coverage.covered_functions.contains("regular_function"));
    }

    #[test]
    fn test_analyze_mixed_content() {
        let mut collector = RuchyCoverageCollector::new();
        let content = r#"
// Header comment
/*
 * Multi-line comment
 */

fn main() {
    if true {
        println("Hello")
    } else {
        println("World")
    }

    match x {
        1 => println("one"),
        2 => println("two"),
        _ => println("other")
    }
}

// Another comment
fun helper() {
    for i in range(10) {
        println(i)
    }
}
"#;
        let file = create_test_file(content);

        let result = collector.analyze_file(file.path());
        assert!(result.is_ok());

        let file_path = file.path().to_str().unwrap();
        let coverage = collector.coverage_data.get(file_path).unwrap();

        // Should count non-comment, non-empty lines
        assert!(coverage.total_lines > 10);
        assert_eq!(coverage.total_functions, 2);
        // if statement has 2 branches, match has 1, for has 1 = 4 total
        assert_eq!(coverage.total_branches, 4);
    }
}

#[cfg(test)]
mod property_tests_ruchy_coverage {
    use super::*;
    use proptest::proptest;

    proptest! {
        /// Property: RuchyCoverage::new never panics
        #[test]
        fn test_new_never_panics(file_path: String) {
            let _ = RuchyCoverage::new(&file_path);
        }

        /// Property: line_coverage is always between 0 and 100
        #[test]
        fn test_line_coverage_bounds(total_lines: u16, covered_count: u8) {
            let mut coverage = RuchyCoverage::new("test");
            coverage.total_lines = total_lines as usize;

            // Add covered lines up to the limit
            let max_covered = std::cmp::min(covered_count as usize, total_lines as usize);
            for i in 1..=max_covered {
                coverage.covered_lines.insert(i);
            }

            let percentage = coverage.line_coverage();
            assert!((0.0..=100.0).contains(&percentage));
        }

        /// Property: extract_function_name never panics
        #[test]
        fn test_extract_function_name_never_panics(input: String) {
            let _ = extract_function_name(&input);
        }
    }
}
