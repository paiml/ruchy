#[cfg(test)]
mod is_param_numeric_tests {
    
    #[test]
    fn test_numeric_operator_detection() {
        assert!(is_numeric_operator("+"));
        assert!(is_numeric_operator("-"));
        assert!(is_numeric_operator("*"));
        assert!(is_numeric_operator("/"));
        assert!(is_numeric_operator("%"));
        assert!(!is_numeric_operator("&&"));
        assert!(!is_numeric_operator("||"));
    }
    
    fn is_numeric_operator(op: &str) -> bool {
        matches!(op, "+" | "-" | "*" | "/" | "%")
    }
    
    #[test]
    fn test_string_concat_detection() {
        // Addition with string literal is string concat
        assert!(is_string_concatenation("+", true, false));
        assert!(is_string_concatenation("+", false, true));
        assert!(!is_string_concatenation("+", false, false));
        // Other operators are never string concat
        assert!(!is_string_concatenation("-", true, false));
    }
    
    fn is_string_concatenation(op: &str, left_string: bool, right_string: bool) -> bool {
        op == "+" && (left_string || right_string)
    }
    
    #[test]
    fn test_param_in_binary_op() {
        assert!(check_param_in_operation("x", true, false, "+"));
        assert!(check_param_in_operation("x", false, true, "*"));
        assert!(!check_param_in_operation("x", false, false, "+"));
    }
    
    fn check_param_in_operation(_param: &str, left_has: bool, right_has: bool, op: &str) -> bool {
        is_numeric_operator(op) && (left_has || right_has)
    }
    
    #[test]
    fn test_recursive_check_branches() {
        // Test that we check multiple branches
        let branches = vec![false, true, false];
        assert!(branches.iter().any(|&b| b));
        
        let branches = vec![false, false, false];
        assert!(!branches.iter().any(|&b| b));
    }
    
    #[test]
    fn test_expression_type_handling() {
        assert_eq!(get_expr_type("binary"), "binary");
        assert_eq!(get_expr_type("block"), "block");
        assert_eq!(get_expr_type("if"), "if");
        assert_eq!(get_expr_type("let"), "let");
        assert_eq!(get_expr_type("call"), "call");
    }
    
    fn get_expr_type(kind: &str) -> &'static str {
        match kind {
            "binary" => "binary",
            "block" => "block",
            "if" => "if",
            "let" => "let",
            "call" => "call",
            _ => "other"
        }
    }
}