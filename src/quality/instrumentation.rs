//! Code instrumentation for coverage tracking
//!
//! [RUCHY-206] Instrument Ruchy code for runtime coverage collection

use std::collections::{HashMap, HashSet};
use anyhow::Result;

/// Runtime coverage collector
pub struct CoverageInstrumentation {
    /// Map of file -> set of executed line numbers
    pub executed_lines: HashMap<String, HashSet<usize>>,
    /// Map of file -> set of executed function names
    pub executed_functions: HashMap<String, HashSet<String>>,
    /// Map of file -> branch execution counts
    pub executed_branches: HashMap<String, HashMap<String, usize>>,
}

impl CoverageInstrumentation {
    pub fn new() -> Self {
        Self {
            executed_lines: HashMap::new(),
            executed_functions: HashMap::new(),
            executed_branches: HashMap::new(),
        }
    }
    
    /// Mark a line as executed
    pub fn mark_line_executed(&mut self, file: &str, line: usize) {
        self.executed_lines
            .entry(file.to_string())
            .or_default()
            .insert(line);
    }
    
    /// Mark a function as executed
    pub fn mark_function_executed(&mut self, file: &str, function: &str) {
        self.executed_functions
            .entry(file.to_string())
            .or_default()
            .insert(function.to_string());
    }
    
    /// Mark a branch as executed
    pub fn mark_branch_executed(&mut self, file: &str, branch_id: &str) {
        *self.executed_branches
            .entry(file.to_string())
            .or_default()
            .entry(branch_id.to_string())
            .or_default() += 1;
    }
    
    /// Get executed lines for a file
    pub fn get_executed_lines(&self, file: &str) -> Option<&HashSet<usize>> {
        self.executed_lines.get(file)
    }
    
    /// Get executed functions for a file
    pub fn get_executed_functions(&self, file: &str) -> Option<&HashSet<String>> {
        self.executed_functions.get(file)
    }
    
    /// Get branch execution counts for a file
    pub fn get_executed_branches(&self, file: &str) -> Option<&HashMap<String, usize>> {
        self.executed_branches.get(file)
    }
    
    /// Merge coverage data from another instrumentation instance
    pub fn merge(&mut self, other: &CoverageInstrumentation) {
        // Merge executed lines
        for (file, lines) in &other.executed_lines {
            let entry = self.executed_lines.entry(file.clone()).or_default();
            for line in lines {
                entry.insert(*line);
            }
        }
        
        // Merge executed functions
        for (file, functions) in &other.executed_functions {
            let entry = self.executed_functions.entry(file.clone()).or_default();
            for function in functions {
                entry.insert(function.clone());
            }
        }
        
        // Merge branch counts
        for (file, branches) in &other.executed_branches {
            let entry = self.executed_branches.entry(file.clone()).or_default();
            for (branch_id, count) in branches {
                *entry.entry(branch_id.clone()).or_default() += count;
            }
        }
    }
}

impl Default for CoverageInstrumentation {
    fn default() -> Self {
        Self::new()
    }
}

/// Add instrumentation to Ruchy source code
pub fn instrument_source(source: &str, file_path: &str) -> Result<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut instrumented = String::new();
    
    // Add coverage initialization at the top
    instrumented.push_str(&format!(
        "// Coverage instrumentation for {file_path}\n"
    ));
    instrumented.push_str("let __coverage = CoverageInstrumentation::new();\n\n");
    
    for (line_num, line) in lines.iter().enumerate() {
        let actual_line_num = line_num + 1;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            instrumented.push_str(line);
            instrumented.push('\n');
            continue;
        }
        
        // Add line execution tracking before executable statements
        if is_executable_line(trimmed) {
            instrumented.push_str(&format!(
                "__coverage.mark_line_executed(\"{file_path}\", {actual_line_num});\n"
            ));
        }
        
        // Instrument function declarations
        if trimmed.starts_with("fn ") || trimmed.starts_with("fun ") {
            let function_name = extract_function_name(trimmed);
            instrumented.push_str(&format!(
                "__coverage.mark_function_executed(\"{file_path}\", \"{function_name}\");\n"
            ));
        }
        
        // Add the original line
        instrumented.push_str(line);
        instrumented.push('\n');
    }
    
    Ok(instrumented)
}

/// Check if a line contains executable code (not just declarations)
fn is_executable_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // First check if it's a control flow statement (these are executable even with {)
    if trimmed.starts_with("if ") ||
       trimmed.starts_with("while ") ||
       trimmed.starts_with("for ") ||
       trimmed.starts_with("match ") {
        return true;
    }
    
    // Skip function signatures, struct definitions, etc.
    if trimmed.starts_with("fn ") || 
       trimmed.starts_with("fun ") ||
       trimmed.starts_with("struct ") ||
       trimmed.starts_with("enum ") ||
       trimmed.starts_with("use ") ||
       trimmed.starts_with("mod ") ||
       trimmed.starts_with("#[") ||
       (trimmed.ends_with('{') && !trimmed.contains('=')) {
        return false;
    }
    
    // Consider lines with statements/expressions as executable
    trimmed.contains('=') ||
    trimmed.contains("println") ||
    trimmed.contains("assert") ||
    trimmed.contains("return")
}

/// Extract function name from function declaration
fn extract_function_name(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].split('(').next().unwrap_or("unknown").to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coverage_instrumentation() {
        let mut coverage = CoverageInstrumentation::new();
        
        coverage.mark_line_executed("test.ruchy", 5);
        coverage.mark_function_executed("test.ruchy", "main");
        coverage.mark_branch_executed("test.ruchy", "if_1");
        
        assert!(coverage.get_executed_lines("test.ruchy").unwrap().contains(&5));
        assert!(coverage.get_executed_functions("test.ruchy").unwrap().contains("main"));
        assert_eq!(coverage.get_executed_branches("test.ruchy").unwrap().get("if_1"), Some(&1));
    }
    
    #[test] 
    fn test_is_executable_line() {
        assert!(is_executable_line("let x = 5;"));
        assert!(is_executable_line("println(\"hello\");"));
        assert!(is_executable_line("return x + 1;"));
        assert!(is_executable_line("if x > 0 {"));
        
        assert!(!is_executable_line("fn main() {"));
        assert!(!is_executable_line("struct Point {"));
        assert!(!is_executable_line("use std::collections::HashMap;"));
        assert!(!is_executable_line("// comment"));
        assert!(!is_executable_line(""));
    }
    
    #[test]
    fn test_extract_function_name() {
        assert_eq!(extract_function_name("fn main() {"), "main");
        assert_eq!(extract_function_name("fun test_function(x: i32) -> i32 {"), "test_function");
        assert_eq!(extract_function_name("fn add(a: i32, b: i32) -> i32 {"), "add");
    }
    
    #[test]
    fn test_merge_coverage() {
        let mut coverage1 = CoverageInstrumentation::new();
        coverage1.mark_line_executed("test.ruchy", 1);
        coverage1.mark_function_executed("test.ruchy", "func1");
        
        let mut coverage2 = CoverageInstrumentation::new();
        coverage2.mark_line_executed("test.ruchy", 2);
        coverage2.mark_function_executed("test.ruchy", "func2");
        
        coverage1.merge(&coverage2);
        
        let lines = coverage1.get_executed_lines("test.ruchy").unwrap();
        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
        
        let functions = coverage1.get_executed_functions("test.ruchy").unwrap();
        assert!(functions.contains("func1"));
        assert!(functions.contains("func2"));
    }
}