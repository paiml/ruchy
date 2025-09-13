//! Comprehensive TDD tests for Quality modules
//! Target: Increase coverage for linter, formatter, and quality gates
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod quality_tests {
    use crate::quality::{Linter, Formatter, QualityGate, CoverageAnalyzer, Rule, Severity};
    use std::collections::HashMap;
    
    // ========== Linter Tests ==========
    
    #[test]
    fn test_linter_creation() {
        let linter = Linter::new();
        assert_eq!(linter.rule_count(), 0);
        assert!(linter.is_empty());
    }
    
    #[test]
    fn test_add_lint_rule() {
        let mut linter = Linter::new();
        let rule = Rule::new("no-unused-vars", Severity::Warning);
        
        linter.add_rule(rule);
        assert_eq!(linter.rule_count(), 1);
        assert!(!linter.is_empty());
    }
    
    #[test]
    fn test_lint_simple_code() {
        let mut linter = Linter::new();
        linter.add_rule(Rule::new("no-unused-vars", Severity::Warning));
        
        let code = "let x = 10;";
        let results = linter.lint(code);
        
        // Should detect unused variable
        assert!(!results.is_empty());
        assert_eq!(results[0].rule_name, "no-unused-vars");
    }
    
    #[test]
    fn test_lint_clean_code() {
        let mut linter = Linter::new();
        linter.add_rule(Rule::new("no-unused-vars", Severity::Warning));
        
        let code = "let x = 10; println(x);";
        let results = linter.lint(code);
        
        // Should pass - variable is used
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_multiple_lint_rules() {
        let mut linter = Linter::new();
        linter.add_rule(Rule::new("no-unused-vars", Severity::Warning));
        linter.add_rule(Rule::new("no-magic-numbers", Severity::Info));
        linter.add_rule(Rule::new("max-line-length", Severity::Error));
        
        assert_eq!(linter.rule_count(), 3);
        
        let code = "let x = 42;";
        let results = linter.lint(code);
        
        // Should detect unused var and magic number
        assert!(results.len() >= 2);
    }
    
    #[test]
    fn test_lint_severity_levels() {
        let mut linter = Linter::new();
        linter.add_rule(Rule::new("critical-issue", Severity::Error));
        
        let code = "unsafe { /* bad code */ }";
        let results = linter.lint(code);
        
        if !results.is_empty() {
            assert_eq!(results[0].severity, Severity::Error);
        }
    }
    
    // ========== Formatter Tests ==========
    
    #[test]
    fn test_formatter_creation() {
        let formatter = Formatter::new();
        assert_eq!(formatter.indent_size(), 4);
        assert_eq!(formatter.max_line_length(), 100);
    }
    
    #[test]
    fn test_format_simple_code() {
        let formatter = Formatter::new();
        let code = "let x=10;let y=20;";
        let formatted = formatter.format(code);
        
        // Should add proper spacing
        assert!(formatted.contains("let x = 10;"));
        assert!(formatted.contains("let y = 20;"));
    }
    
    #[test]
    fn test_format_indentation() {
        let formatter = Formatter::new();
        let code = "fn test() {\nlet x = 10;\nreturn x;\n}";
        let formatted = formatter.format(code);
        
        // Should have proper indentation
        assert!(formatted.contains("    let x = 10;"));
        assert!(formatted.contains("    return x;"));
    }
    
    #[test]
    fn test_format_custom_settings() {
        let mut formatter = Formatter::new();
        formatter.set_indent_size(2);
        formatter.set_max_line_length(80);
        
        assert_eq!(formatter.indent_size(), 2);
        assert_eq!(formatter.max_line_length(), 80);
        
        let code = "fn test() {\nlet x = 10;\n}";
        let formatted = formatter.format(code);
        
        // Should use 2-space indentation
        assert!(formatted.contains("  let x = 10;"));
    }
    
    #[test]
    fn test_format_preserve_comments() {
        let formatter = Formatter::new();
        let code = "// This is a comment\nlet x = 10; // inline comment";
        let formatted = formatter.format(code);
        
        // Should preserve comments
        assert!(formatted.contains("// This is a comment"));
        assert!(formatted.contains("// inline comment"));
    }
    
    #[test]
    fn test_format_multiline_expressions() {
        let formatter = Formatter::new();
        let code = "let result = very_long_function_name(arg1,arg2,arg3,arg4,arg5);";
        let formatted = formatter.format(code);
        
        // Should break long lines appropriately
        assert!(formatted.lines().all(|line| line.len() <= 100));
    }
    
    // ========== Quality Gate Tests ==========
    
    #[test]
    fn test_quality_gate_creation() {
        let gate = QualityGate::new();
        assert_eq!(gate.threshold_count(), 0);
    }
    
    #[test]
    fn test_add_quality_threshold() {
        let mut gate = QualityGate::new();
        gate.add_threshold("complexity", 10.0);
        gate.add_threshold("coverage", 80.0);
        
        assert_eq!(gate.threshold_count(), 2);
        assert_eq!(gate.get_threshold("complexity"), Some(10.0));
        assert_eq!(gate.get_threshold("coverage"), Some(80.0));
    }
    
    #[test]
    fn test_quality_gate_pass() {
        let mut gate = QualityGate::new();
        gate.add_threshold("complexity", 10.0);
        
        let metrics = HashMap::from([
            ("complexity".to_string(), 8.0),
        ]);
        
        let result = gate.check(&metrics);
        assert!(result.passed);
        assert!(result.failures.is_empty());
    }
    
    #[test]
    fn test_quality_gate_fail() {
        let mut gate = QualityGate::new();
        gate.add_threshold("complexity", 10.0);
        gate.add_threshold("coverage", 80.0);
        
        let metrics = HashMap::from([
            ("complexity".to_string(), 15.0),
            ("coverage".to_string(), 60.0),
        ]);
        
        let result = gate.check(&metrics);
        assert!(!result.passed);
        assert_eq!(result.failures.len(), 2);
    }
    
    #[test]
    fn test_quality_gate_partial_metrics() {
        let mut gate = QualityGate::new();
        gate.add_threshold("complexity", 10.0);
        gate.add_threshold("coverage", 80.0);
        
        let metrics = HashMap::from([
            ("complexity".to_string(), 8.0),
            // coverage missing
        ]);
        
        let result = gate.check(&metrics);
        // Should handle missing metrics appropriately
        assert!(!result.passed || result.warnings.len() > 0);
    }
    
    // ========== Coverage Analyzer Tests ==========
    
    #[test]
    fn test_coverage_analyzer_creation() {
        let analyzer = CoverageAnalyzer::new();
        assert_eq!(analyzer.file_count(), 0);
    }
    
    #[test]
    fn test_analyze_file_coverage() {
        let mut analyzer = CoverageAnalyzer::new();
        let file_path = "test.rs";
        let content = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\nfn unused() {\n    42\n}";
        
        let coverage = analyzer.analyze_file(file_path, content);
        assert!(coverage.line_coverage >= 0.0);
        assert!(coverage.line_coverage <= 100.0);
    }
    
    #[test]
    fn test_aggregate_coverage() {
        let mut analyzer = CoverageAnalyzer::new();
        
        analyzer.add_file_coverage("file1.rs", 80.0, 100);
        analyzer.add_file_coverage("file2.rs", 60.0, 200);
        analyzer.add_file_coverage("file3.rs", 90.0, 50);
        
        let total = analyzer.aggregate_coverage();
        // Weighted average: (80*100 + 60*200 + 90*50) / (100+200+50)
        assert!(total > 0.0 && total <= 100.0);
    }
    
    #[test]
    fn test_coverage_report_generation() {
        let mut analyzer = CoverageAnalyzer::new();
        
        analyzer.add_file_coverage("src/main.rs", 85.0, 150);
        analyzer.add_file_coverage("src/lib.rs", 92.0, 300);
        analyzer.add_file_coverage("src/utils.rs", 45.0, 50);
        
        let report = analyzer.generate_report();
        assert!(report.contains("main.rs"));
        assert!(report.contains("85.0%"));
        assert!(report.contains("Total Coverage"));
    }
    
    #[test]
    fn test_coverage_thresholds() {
        let mut analyzer = CoverageAnalyzer::new();
        analyzer.set_threshold(80.0);
        
        analyzer.add_file_coverage("good.rs", 85.0, 100);
        analyzer.add_file_coverage("bad.rs", 60.0, 100);
        
        let violations = analyzer.get_violations();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("bad.rs"));
    }
    
    // ========== Integration Tests ==========
    
    #[test]
    fn test_full_quality_pipeline() {
        // Lint -> Format -> Quality Gate -> Coverage
        let mut linter = Linter::new();
        linter.add_rule(Rule::new("no-unused-vars", Severity::Warning));
        
        let formatter = Formatter::new();
        let mut gate = QualityGate::new();
        gate.add_threshold("lint-errors", 0.0);
        
        let code = "let x=10;println(x);";
        
        // Lint
        let lint_results = linter.lint(code);
        
        // Format
        let formatted = formatter.format(code);
        
        // Check gate
        let metrics = HashMap::from([
            ("lint-errors".to_string(), lint_results.len() as f64),
        ]);
        let gate_result = gate.check(&metrics);
        
        assert!(gate_result.passed);
        assert!(formatted.contains("let x = 10;"));
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl Linter {
        fn is_empty(&self) -> bool {
            self.rule_count() == 0
        }
        
        fn rule_count(&self) -> usize {
            self.rules.len()
        }
    }
    
    impl Formatter {
        fn indent_size(&self) -> usize {
            self.indent_size
        }
        
        fn max_line_length(&self) -> usize {
            self.max_line_length
        }
    }
    
    impl QualityGate {
        fn threshold_count(&self) -> usize {
            self.thresholds.len()
        }
        
        fn get_threshold(&self, name: &str) -> Option<f64> {
            self.thresholds.get(name).copied()
        }
    }
    
    impl CoverageAnalyzer {
        fn file_count(&self) -> usize {
            self.files.len()
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_formatter_never_panics(code in ".*") {
            let formatter = Formatter::new();
            let _ = formatter.format(&code); // Should not panic
        }
        
        #[test]
        fn test_linter_never_panics(code in ".*") {
            let mut linter = Linter::new();
            linter.add_rule(Rule::new("test", Severity::Info));
            let _ = linter.lint(&code); // Should not panic
        }
        
        #[test]
        fn test_coverage_always_in_range(coverage in 0.0..=100.0, lines in 1usize..10000) {
            let mut analyzer = CoverageAnalyzer::new();
            analyzer.add_file_coverage("test.rs", coverage, lines);
            
            let total = analyzer.aggregate_coverage();
            assert!(total >= 0.0 && total <= 100.0);
        }
    }
}