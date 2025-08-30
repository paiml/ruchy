//! Coverage implementation for Ruchy test files
//!
//! [RUCHY-206] Implement coverage collection for .ruchy files

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

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
    pub fn line_coverage(&self) -> f64 {
        if self.total_lines == 0 {
            100.0
        } else {
            (self.covered_lines.len() as f64 / self.total_lines as f64) * 100.0
        }
    }
    
    /// Calculate function coverage percentage
    pub fn function_coverage(&self) -> f64 {
        if self.total_functions == 0 {
            100.0
        } else {
            (self.covered_functions.len() as f64 / self.total_functions as f64) * 100.0
        }
    }
    
    /// Calculate branch coverage percentage
    pub fn branch_coverage(&self) -> f64 {
        if self.total_branches == 0 {
            100.0
        } else {
            (self.covered_branches as f64 / self.total_branches as f64) * 100.0
        }
    }
    
    /// Calculate overall coverage percentage
    pub fn overall_coverage(&self) -> f64 {
        // Weight: 60% lines, 30% functions, 10% branches
        self.line_coverage() * 0.6 + 
        self.function_coverage() * 0.3 + 
        self.branch_coverage() * 0.1
    }
}

/// Coverage collector for Ruchy code
pub struct RuchyCoverageCollector {
    coverage_data: HashMap<String, RuchyCoverage>,
}

impl RuchyCoverageCollector {
    pub fn new() -> Self {
        Self {
            coverage_data: HashMap::new(),
        }
    }
    
    /// Analyze a Ruchy file to determine what needs coverage
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let mut coverage = RuchyCoverage::new(file_path.to_str().unwrap_or("unknown"));
        
        // Count total lines (non-empty, non-comment)
        let lines: Vec<&str> = content.lines().collect();
        coverage.total_lines = lines.iter()
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
                if trimmed.contains("test") || lines.get(line_num.saturating_sub(1))
                    .map(|l| l.contains("#[test]"))
                    .unwrap_or(false) {
                    let func_name = trimmed.split_whitespace()
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
        
        self.coverage_data.insert(coverage.file_path.clone(), coverage);
        Ok(())
    }
    
    /// Mark lines as covered based on test execution
    pub fn mark_covered(&mut self, file_path: &str, line_numbers: Vec<usize>) {
        if let Some(coverage) = self.coverage_data.get_mut(file_path) {
            for line in line_numbers {
                coverage.covered_lines.insert(line);
            }
        }
    }
    
    /// Mark a function as covered
    pub fn mark_function_covered(&mut self, file_path: &str, function_name: &str) {
        if let Some(coverage) = self.coverage_data.get_mut(file_path) {
            coverage.covered_functions.insert(function_name.to_string());
        }
    }
    
    /// Generate a text report
    pub fn generate_text_report(&self) -> String {
        let mut report = String::new();
        report.push_str("\n📊 Coverage Report\n");
        report.push_str("==================\n\n");
        
        let mut total_lines = 0;
        let mut total_covered_lines = 0;
        let mut total_functions = 0;
        let mut total_covered_functions = 0;
        
        for (file_path, coverage) in &self.coverage_data {
            report.push_str(&format!("📄 {}\n", file_path));
            report.push_str(&format!("   Lines: {}/{} ({:.1}%)\n", 
                coverage.covered_lines.len(), 
                coverage.total_lines,
                coverage.line_coverage()));
            report.push_str(&format!("   Functions: {}/{} ({:.1}%)\n", 
                coverage.covered_functions.len(),
                coverage.total_functions,
                coverage.function_coverage()));
            if coverage.total_branches > 0 {
                report.push_str(&format!("   Branches: {}/{} ({:.1}%)\n", 
                    coverage.covered_branches,
                    coverage.total_branches,
                    coverage.branch_coverage()));
            }
            report.push_str(&format!("   Overall: {:.1}%\n\n", coverage.overall_coverage()));
            
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
        
        report.push_str(&format!("Total Lines: {}/{} ({:.1}%)\n", 
            total_covered_lines, total_lines, overall_line_coverage));
        report.push_str(&format!("Total Functions: {}/{} ({:.1}%)\n", 
            total_covered_functions, total_functions, overall_function_coverage));
        report.push_str(&format!("Overall Coverage: {:.1}%\n", 
            overall_line_coverage * 0.7 + overall_function_coverage * 0.3));
        
        report
    }
    
    /// Generate a JSON report
    pub fn generate_json_report(&self) -> String {
        serde_json::to_string_pretty(&self.coverage_data).unwrap_or_else(|_| "{}".to_string())
    }
    
    /// Generate an HTML report
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
            html.push_str(&format!("<div class='summary'>\n"));
            html.push_str(&format!("<h2>{}</h2>\n", file_path));
            html.push_str(&format!("<p>Line Coverage: {:.1}%</p>\n", coverage.line_coverage()));
            html.push_str(&format!("<p>Function Coverage: {:.1}%</p>\n", coverage.function_coverage()));
            html.push_str(&format!("<p>Overall: {:.1}%</p>\n", coverage.overall_coverage()));
            html.push_str("</div>\n");
        }
        
        html.push_str("</body>\n</html>");
        html
    }
    
    /// Check if coverage meets threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        for coverage in self.coverage_data.values() {
            if coverage.overall_coverage() < threshold {
                return false;
            }
        }
        true
    }
}

impl Default for RuchyCoverageCollector {
    fn default() -> Self {
        Self::new()
    }
}