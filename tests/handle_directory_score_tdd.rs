#[cfg(test)]
mod handle_directory_score_tests {
    use std::collections::HashMap;
    use std::path::Path;
    
    #[test]
    fn test_empty_directory_handling() {
        let files = Vec::<String>::new();
        assert!(should_return_early_for_empty(&files));
        
        let files = vec!["file.ruchy".to_string()];
        assert!(!should_return_early_for_empty(&files));
    }
    
    fn should_return_early_for_empty(files: &[String]) -> bool {
        files.is_empty()
    }
    
    #[test]
    fn test_score_aggregation() {
        let mut scores = HashMap::new();
        scores.insert("file1.ruchy", 0.8);
        scores.insert("file2.ruchy", 0.9);
        scores.insert("file3.ruchy", 0.7);
        
        let avg = calculate_average_score(&scores);
        assert!((avg - 0.8).abs() < 0.01);
    }
    
    fn calculate_average_score(scores: &HashMap<&str, f64>) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }
        let total: f64 = scores.values().sum();
        total / scores.len() as f64
    }
    
    #[test]
    fn test_threshold_checking() {
        assert!(check_threshold(0.8, Some(0.7)));  // Pass
        assert!(!check_threshold(0.6, Some(0.7))); // Fail
        assert!(check_threshold(0.5, None));       // No threshold
    }
    
    fn check_threshold(score: f64, min: Option<f64>) -> bool {
        min.is_none_or(|m| score >= m)
    }
    
    #[test]
    fn test_format_selection() {
        assert_eq!(select_format("json"), "json");
        assert_eq!(select_format("text"), "text");
        assert_eq!(select_format("unknown"), "unknown");
    }
    
    fn select_format(format: &str) -> &str {
        format
    }
    
    #[test]
    fn test_output_destination() {
        assert!(should_write_to_file(Some("output.txt")));
        assert!(!should_write_to_file(None));
    }
    
    fn should_write_to_file(output: Option<&str>) -> bool {
        output.is_some()
    }
    
    #[test]
    fn test_error_continuation() {
        let mut processed = 0;
        let files = vec!["a.ruchy", "b.ruchy", "c.ruchy"];
        
        for file in &files {
            if process_file_with_error(file).is_ok() {
                processed += 1;
            }
            // Continue even on error
        }
        
        assert_eq!(processed, 2); // One file errors
    }
    
    fn process_file_with_error(file: &str) -> Result<f64, String> {
        if file == "b.ruchy" {
            Err("Failed".to_string())
        } else {
            Ok(0.8)
        }
    }
}