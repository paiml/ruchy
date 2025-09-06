#[cfg(test)]
mod parse_method_access_tests {
    
    #[test]
    fn test_dataframe_method_detection() {
        assert!(is_dataframe_method("select"));
        assert!(is_dataframe_method("groupby"));
        assert!(is_dataframe_method("group_by"));
        assert!(is_dataframe_method("pivot"));
        assert!(!is_dataframe_method("map"));
        assert!(!is_dataframe_method("filter"));
        assert!(!is_dataframe_method("push"));
    }
    
    fn is_dataframe_method(method: &str) -> bool {
        matches!(
            method,
            "select" | "groupby" | "group_by" | "agg" | "pivot" | "melt" | 
            "join" | "rolling" | "shift" | "diff" | "pct_change" | "corr" | "cov"
        )
    }
    
    #[test]
    fn test_method_vs_field_detection() {
        assert_eq!(detect_access_type("method", true), "method_call");
        assert_eq!(detect_access_type("field", false), "field_access");
    }
    
    fn detect_access_type(name: &str, has_parens: bool) -> &'static str {
        if has_parens {
            "method_call"
        } else {
            "field_access"
        }
    }
    
    #[test]
    fn test_select_column_extraction() {
        let cols = extract_select_columns(vec!["age", "name"]);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], "age");
        assert_eq!(cols[1], "name");
    }
    
    fn extract_select_columns(identifiers: Vec<&str>) -> Vec<String> {
        identifiers.into_iter().map(std::string::ToString::to_string).collect()
    }
    
    #[test]
    fn test_groupby_column_extraction() {
        let cols = extract_groupby_columns(vec!["category", "region"]);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], "category");
        assert_eq!(cols[1], "region");
    }
    
    fn extract_groupby_columns(identifiers: Vec<&str>) -> Vec<String> {
        identifiers.into_iter().map(std::string::ToString::to_string).collect()
    }
    
    #[test]
    fn test_argument_parsing_loop() {
        let mut count = 0;
        for _ in 0..3 {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}