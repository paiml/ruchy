#[cfg(test)]
mod handle_quality_gate_tests {
    use std::path::Path;
    
    #[test]
    fn test_complexity_check() {
        assert!(check_complexity_gate(15, 10).is_err());
        assert!(check_complexity_gate(8, 10).is_ok());
        assert!(check_complexity_gate(10, 10).is_ok());
    }
    
    fn check_complexity_gate(actual: usize, limit: usize) -> Result<String, String> {
        if actual > limit {
            Err(format!("❌ Complexity {} exceeds limit {}", actual, limit))
        } else {
            Ok(format!("✅ Complexity {} within limit", actual))
        }
    }
    
    #[test]
    fn test_satd_detection() {
        assert!(has_satd_comment("// TODO: fix this"));
        assert!(has_satd_comment("// FIXME: broken"));
        assert!(has_satd_comment("// HACK: workaround"));
        assert!(!has_satd_comment("// This is a normal comment"));
        assert!(!has_satd_comment("let todo = \"not a comment\""));
    }
    
    fn has_satd_comment(line: &str) -> bool {
        if let Some(comment_pos) = line.find("//") {
            let comment = &line[comment_pos..];
            comment.contains("TODO") || comment.contains("FIXME") || comment.contains("HACK")
        } else {
            false
        }
    }
    
    #[test]
    fn test_gate_results_collection() {
        let mut results = Vec::new();
        
        add_gate_result(&mut results, true, "Complexity check");
        assert_eq!(results.len(), 1);
        assert!(results[0].starts_with("✅"));
        
        add_gate_result(&mut results, false, "SATD check");
        assert_eq!(results.len(), 2);
        assert!(results[1].starts_with("❌"));
    }
    
    fn add_gate_result(results: &mut Vec<String>, passed: bool, gate_name: &str) {
        if passed {
            results.push(format!("✅ {} passed", gate_name));
        } else {
            results.push(format!("❌ {} failed", gate_name));
        }
    }
    
    #[test]
    fn test_output_format_selection() {
        assert_eq!(select_output_format(true), "json");
        assert_eq!(select_output_format(false), "text");
    }
    
    fn select_output_format(json: bool) -> &'static str {
        if json { "json" } else { "text" }
    }
    
    #[test]
    fn test_strict_mode_handling() {
        assert!(should_exit_on_failure(false, true));
        assert!(!should_exit_on_failure(true, true));
        assert!(!should_exit_on_failure(false, false));
    }
    
    fn should_exit_on_failure(passed: bool, strict: bool) -> bool {
        !passed && strict
    }
}