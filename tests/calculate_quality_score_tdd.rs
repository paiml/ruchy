#[cfg(test)]
mod calculate_quality_score_tests {
    
    #[test]
    fn test_complexity_penalty() {
        assert_eq!(get_complexity_penalty(3), 1.0);
        assert_eq!(get_complexity_penalty(8), 0.95);
        assert_eq!(get_complexity_penalty(13), 0.85);
        assert_eq!(get_complexity_penalty(18), 0.70);
        assert_eq!(get_complexity_penalty(25), 0.45);
        assert_eq!(get_complexity_penalty(35), 0.25);
        assert_eq!(get_complexity_penalty(45), 0.15);
        assert_eq!(get_complexity_penalty(60), 0.05);
    }
    
    fn get_complexity_penalty(complexity: usize) -> f64 {
        match complexity {
            0..=5 => 1.0,
            6..=10 => 0.95,
            11..=15 => 0.85,
            16..=20 => 0.70,
            21..=30 => 0.45,
            31..=40 => 0.25,
            41..=50 => 0.15,
            _ => 0.05,
        }
    }
    
    #[test]
    fn test_parameter_penalty() {
        assert_eq!(get_parameter_penalty(2), 1.0);
        assert_eq!(get_parameter_penalty(4), 0.90);
        assert_eq!(get_parameter_penalty(6), 0.75);
        assert_eq!(get_parameter_penalty(9), 0.50);
        assert_eq!(get_parameter_penalty(12), 0.25);
        assert_eq!(get_parameter_penalty(20), 0.10);
        assert_eq!(get_parameter_penalty(30), 0.05);
    }
    
    fn get_parameter_penalty(params: usize) -> f64 {
        match params {
            0..=3 => 1.0,
            4..=5 => 0.90,
            6..=7 => 0.75,
            8..=10 => 0.50,
            11..=15 => 0.25,
            16..=25 => 0.10,
            _ => 0.05,
        }
    }
    
    #[test]
    fn test_nesting_penalty() {
        assert_eq!(get_nesting_penalty(1), 1.0);
        assert_eq!(get_nesting_penalty(3), 0.90);
        assert_eq!(get_nesting_penalty(4), 0.75);
        assert_eq!(get_nesting_penalty(5), 0.50);
        assert_eq!(get_nesting_penalty(6), 0.30);
        assert_eq!(get_nesting_penalty(7), 0.15);
        assert_eq!(get_nesting_penalty(10), 0.05);
    }
    
    fn get_nesting_penalty(depth: usize) -> f64 {
        match depth {
            0..=2 => 1.0,
            3 => 0.90,
            4 => 0.75,
            5 => 0.50,
            6 => 0.30,
            7 => 0.15,
            _ => 0.05,
        }
    }
    
    #[test]
    fn test_length_penalty() {
        assert!((get_length_penalty(10.0) - 1.0).abs() < 0.01);
        assert!((get_length_penalty(25.0) - 1.0).abs() < 0.01);  // No penalty under 20
        assert!(get_length_penalty(40.0) < 1.0);  // Penalty over 20
        assert!(get_length_penalty(100.0) < 0.5);  // Severe penalty for very long
    }
    
    fn get_length_penalty(avg_length: f64) -> f64 {
        if avg_length > 20.0 {
            (30.0 / avg_length).clamp(0.3, 1.0)
        } else {
            1.0
        }
    }
    
    #[test]
    fn test_satd_penalty() {
        assert_eq!(get_satd_penalty(false), 1.0);
        assert_eq!(get_satd_penalty(true), 0.70);
    }
    
    fn get_satd_penalty(has_satd: bool) -> f64 {
        if has_satd { 0.70 } else { 1.0 }
    }
    
    #[test]
    fn test_documentation_penalty() {
        assert!((get_documentation_penalty(0.3) - 0.85).abs() < 0.01);  // Poor docs
        assert!((get_documentation_penalty(0.6) - 1.0).abs() < 0.01);   // Average docs
        assert!((get_documentation_penalty(0.9) - 1.05).abs() < 0.01);  // Good docs
        assert!((get_documentation_penalty(1.0) - 1.05).abs() < 0.01);  // Excellent docs
    }
    
    fn get_documentation_penalty(doc_ratio: f64) -> f64 {
        if doc_ratio < 0.5 {
            0.85  // Penalty for poor documentation
        } else if doc_ratio > 0.8 {
            1.05  // Small bonus for good documentation
        } else {
            1.0   // Neutral for average documentation
        }
    }
    
    #[test]
    fn test_complexity_from_metrics() {
        let base_complexity = 5;
        let branches = 3;
        let loops = 2;
        let patterns = 1;
        
        let expected = base_complexity + branches + loops * 2 + patterns;
        assert_eq!(calculate_complexity_from_metrics(base_complexity, branches, loops, patterns), expected);
    }
    
    fn calculate_complexity_from_metrics(base: usize, branches: usize, loops: usize, patterns: usize) -> usize {
        base + branches + loops * 2 + patterns
    }
    
    #[test]
    fn test_score_multiplication() {
        let mut score = 1.0;
        
        score = apply_penalty(score, 0.95);  // Complexity
        assert!((score - 0.95).abs() < 0.01);
        
        score = apply_penalty(score, 0.90);  // Parameters
        assert!((score - 0.855).abs() < 0.01);
        
        score = apply_penalty(score, 0.70);  // SATD
        assert!((score - 0.5985).abs() < 0.01);
    }
    
    fn apply_penalty(score: f64, penalty: f64) -> f64 {
        score * penalty
    }
}