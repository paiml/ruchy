//! TDD tests for grammar coverage module
//! Target: 0% -> 80%+ coverage with complexity ≤10 per test

#[cfg(test)]
mod tests {
    use ruchy::runtime::grammar_coverage::{GrammarCoverageMatrix, ProductionStats, GRAMMAR_PRODUCTIONS};
    use ruchy::frontend::parser::Parser;
    use std::time::Duration;
    
    // Helper function to create a parser result (complexity: 3)
    fn parse_code(code: &str) -> anyhow::Result<ruchy::frontend::ast::Expr> {
        let mut parser = Parser::new(code);
        parser.parse_expr().map_err(|e| anyhow::anyhow!("{:?}", e))
    }
    
    // Test 1: Create new coverage matrix (complexity: 2)
    #[test]
    fn test_new_coverage_matrix() {
        let matrix = GrammarCoverageMatrix::new();
        assert!(matrix.productions.is_empty());
        assert!(matrix.ast_variants.is_empty());
        assert!(matrix.uncovered.is_empty());
    }
    
    // Test 2: Record successful parse (complexity: 5)
    #[test]
    fn test_record_successful_parse() {
        let mut matrix = GrammarCoverageMatrix::new();
        let result = parse_code("42");
        let duration = Duration::from_millis(10);
        
        matrix.record("literal_int", "42", result, duration);
        
        assert_eq!(matrix.productions.len(), 1);
        assert!(matrix.productions.contains_key("literal_int"));
        let stats = &matrix.productions["literal_int"];
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.success_count, 1);
        assert!(stats.avg_latency_ns > 0);
    }
    
    // Test 3: Record failed parse (complexity: 5)
    #[test]
    fn test_record_failed_parse() {
        let mut matrix = GrammarCoverageMatrix::new();
        let result = parse_code("@#$%"); // Invalid syntax
        let duration = Duration::from_millis(5);
        
        matrix.record("invalid", "@#$%", result, duration);
        
        let stats = &matrix.productions["invalid"];
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.error_patterns.len(), 1);
    }
    
    // Test 4: Record multiple attempts (complexity: 6)
    #[test]
    fn test_record_multiple_attempts() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Record successful attempts
        for i in 0..3 {
            let result = parse_code("42");
            matrix.record("test_prod", "42", result, Duration::from_millis(i as u64));
        }
        
        // Record failed attempts with different errors  
        let result1 = Err(anyhow::anyhow!("Error 1"));
        matrix.record("test_prod", "@invalid", result1, Duration::from_millis(3));
        
        let result2 = Err(anyhow::anyhow!("Error 2"));
        matrix.record("test_prod", "#invalid2", result2, Duration::from_millis(4));
        
        let stats = &matrix.productions["test_prod"];
        assert_eq!(stats.hit_count, 5);
        assert_eq!(stats.success_count, 3);
        assert_eq!(stats.error_patterns.len(), 2); // Two different error messages
    }
    
    // Test 5: Test AST variant recording (complexity: 7)
    #[test]
    fn test_ast_variant_recording() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Test different AST variants
        let test_cases = vec![
            ("42", "literal_int"),
            ("true", "literal_bool"),
            ("\"hello\"", "literal_string"),
            ("x + y", "binary_op"),
            ("if true { 1 } else { 0 }", "if_expr"),
        ];
        
        for (code, production) in test_cases {
            let result = parse_code(code);
            matrix.record(production, code, result, Duration::from_millis(1));
        }
        
        assert!(matrix.ast_variants.len() >= 3); // At least Literal, Binary, If
    }
    
    // Test 6: Average latency calculation (complexity: 6)
    #[test]
    fn test_average_latency_calculation() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Record with different durations
        let durations = [10, 20, 30, 40, 50];
        for ms in &durations {
            let result = parse_code("42");
            matrix.record("timing_test", "42", result, Duration::from_millis(*ms));
        }
        
        let stats = &matrix.productions["timing_test"];
        let expected_avg = 30_000_000; // 30ms in nanoseconds
        assert!(stats.avg_latency_ns > expected_avg - 1_000_000);
        assert!(stats.avg_latency_ns < expected_avg + 1_000_000);
    }
    
    // Test 7: Is complete check (complexity: 4)
    #[test]
    fn test_is_complete() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Should not be complete initially
        assert!(!matrix.is_complete(5));
        
        // Add 5 productions
        for i in 0..5 {
            let production = format!("prod_{i}");
            let result = parse_code("42");
            matrix.record(Box::leak(production.into_boxed_str()), "42", result, Duration::from_millis(1));
        }
        
        assert!(matrix.is_complete(5));
        assert!(!matrix.is_complete(10)); // Not complete for higher requirement
    }
    
    // Test 8: Assert complete (complexity: 5)
    #[test]
    fn test_assert_complete_success() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Add required productions
        for i in 0..3 {
            let production = format!("prod_{i}");
            let result = parse_code("42");
            matrix.record(Box::leak(production.into_boxed_str()), "42", result, Duration::from_millis(1));
        }
        
        matrix.assert_complete(3); // Should not panic
    }
    
    // Test 9: Assert complete failure (complexity: 4)
    #[test]
    #[should_panic(expected = "Only 2 of 5 productions covered")]
    fn test_assert_complete_failure() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Only add 2 productions
        for i in 0..2 {
            let production = format!("prod_{i}");
            let result = parse_code("42");
            matrix.record(Box::leak(production.into_boxed_str()), "42", result, Duration::from_millis(1));
        }
        
        matrix.assert_complete(5); // Should panic
    }
    
    // Test 10: Report generation (complexity: 7)
    #[test]
    fn test_report_generation() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Add some data
        matrix.record("fast", "42", parse_code("42"), Duration::from_millis(1));
        matrix.record("slow", "complex", parse_code("if true { 1 } else { 0 }"), Duration::from_millis(100));
        matrix.record("failed", "invalid", parse_code("@#$"), Duration::from_millis(5));
        
        let report = matrix.report();
        
        assert!(report.contains("Grammar Coverage Report"));
        assert!(report.contains("Productions covered: 3"));
        assert!(report.contains("Total attempts: 3"));
        assert!(report.contains("Success rate:"));
        assert!(report.contains("Slowest productions:"));
    }
    
    // Test 11: Uncovered productions in report (complexity: 5)
    #[test]
    fn test_uncovered_in_report() {
        let mut matrix = GrammarCoverageMatrix::new();
        matrix.uncovered.push("missing_prod_1");
        matrix.uncovered.push("missing_prod_2");
        
        let report = matrix.report();
        
        assert!(report.contains("Uncovered productions:"));
        assert!(report.contains("missing_prod_1"));
        assert!(report.contains("missing_prod_2"));
    }
    
    // Test 12: Error pattern deduplication (complexity: 6)
    #[test]
    fn test_error_pattern_deduplication() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Record same error multiple times
        for _ in 0..3 {
            let error = Err(anyhow::anyhow!("Same error"));
            matrix.record("error_test", "@invalid", error, Duration::from_millis(1));
        }
        
        let stats = &matrix.productions["error_test"];
        assert_eq!(stats.hit_count, 3);
        assert_eq!(stats.success_count, 0); // All failed
        assert_eq!(stats.error_patterns.len(), 1); // Only one unique error pattern
    }
    
    // Test 13: All AST variants coverage (complexity: 8)
    #[test]
    fn test_all_ast_variants() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        let test_cases = vec![
            ("42", "Literal"),
            ("x", "Identifier"),
            ("x + y", "Binary"),
            ("-x", "Unary"),
            ("f(x)", "Call"),
            ("[1, 2, 3]", "List"),
            ("0..10", "Range"),
            ("x = 5", "Assign"),
        ];
        
        for (code, _expected) in test_cases {
            let result = parse_code(code);
            matrix.record("variant_test", code, result, Duration::from_millis(1));
        }
        
        assert!(matrix.ast_variants.len() >= 5);
        assert!(matrix.ast_variants.contains("Literal"));
    }
    
    // Test 14: Grammar productions constant (complexity: 3)
    #[test]
    fn test_grammar_productions_constant() {
        assert!(!GRAMMAR_PRODUCTIONS.is_empty());
        
        // Verify structure
        for (name, code) in GRAMMAR_PRODUCTIONS {
            assert!(!name.is_empty());
            assert!(!code.is_empty());
        }
        
        // Check we have all major categories
        let has_literal = GRAMMAR_PRODUCTIONS.iter().any(|(n, _)| n.starts_with("literal_"));
        let has_op = GRAMMAR_PRODUCTIONS.iter().any(|(n, _)| n.starts_with("op_"));
        let has_control = GRAMMAR_PRODUCTIONS.iter().any(|(n, _)| n.contains("if") || n.contains("loop"));
        
        assert!(has_literal);
        assert!(has_op);
        assert!(has_control);
    }
    
    // Test 15: Production stats default (complexity: 2)
    #[test]
    fn test_production_stats_default() {
        let stats = ProductionStats::default();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.avg_latency_ns, 0);
        assert!(stats.error_patterns.is_empty());
    }
    
    // Test 16: Success rate calculation (complexity: 7)
    #[test]
    fn test_success_rate_calculation() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // 3 successful, 2 failed
        matrix.record("test1", "42", parse_code("42"), Duration::from_millis(1));
        matrix.record("test2", "true", parse_code("true"), Duration::from_millis(1));
        matrix.record("test3", "x", parse_code("x"), Duration::from_millis(1));
        matrix.record("test4", "@", parse_code("@"), Duration::from_millis(1));
        matrix.record("test5", "#", parse_code("#"), Duration::from_millis(1));
        
        let report = matrix.report();
        assert!(report.contains("Success rate: 60")); // 3/5 = 60%
    }
    
    // Test 17: Empty matrix report (complexity: 3)
    #[test]
    fn test_empty_matrix_report() {
        let matrix = GrammarCoverageMatrix::new();
        let report = matrix.report();
        
        assert!(report.contains("Productions covered: 0"));
        assert!(report.contains("AST variants seen: 0"));
        assert!(report.contains("Total attempts: 0"));
        assert!(report.contains("Success rate: 0.00%"));
    }
    
    // Test 18: Complex expression coverage (complexity: 8)
    #[test]
    fn test_complex_expression_coverage() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Complex nested expression
        let code = "if x > 0 { x * 2 } else { -x }";
        let result = parse_code(code);
        matrix.record("complex", code, result, Duration::from_millis(10));
        
        // Should record multiple AST variants from one expression
        assert!(matrix.productions.contains_key("complex"));
        assert!(!matrix.ast_variants.is_empty()); // At least If variant
    }
    
    // Test 19: Duration overflow handling (complexity: 5)
    #[test]
    fn test_duration_overflow_handling() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Use very large duration
        let huge_duration = Duration::from_secs(1_000_000);
        let result = parse_code("42");
        matrix.record("overflow_test", "42", result, huge_duration);
        
        let stats = &matrix.productions["overflow_test"];
        assert!(stats.avg_latency_ns > 0);
        assert!(stats.avg_latency_ns <= u64::MAX);
    }
    
    // Test 20: Multiple error patterns (complexity: 7)
    #[test]
    fn test_multiple_different_errors() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        let invalid_codes = vec!["@", "#", "$", "%", "&"];
        for code in invalid_codes {
            let result = parse_code(code);
            matrix.record("errors", code, result, Duration::from_millis(1));
        }
        
        let stats = &matrix.productions["errors"];
        assert_eq!(stats.hit_count, 5);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.error_patterns.len(), 5); // Each produces different error
    }
}