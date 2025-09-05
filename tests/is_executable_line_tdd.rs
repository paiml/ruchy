#[cfg(test)]
mod is_executable_line_refactoring_tests {
    
    #[test]
    fn test_control_flow_statements() {
        assert!(check_control_flow("if x > 0"));
        assert!(check_control_flow("while true"));
        assert!(check_control_flow("for i in 0..10"));
        assert!(check_control_flow("match x"));
        assert!(!check_control_flow("let x = 5"));
    }
    
    fn check_control_flow(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("if ") ||
        trimmed.starts_with("while ") ||
        trimmed.starts_with("for ") ||
        trimmed.starts_with("match ")
    }
    
    #[test]
    fn test_declaration_statements() {
        assert!(check_declarations("fn main()"));
        assert!(check_declarations("fun test()"));
        assert!(check_declarations("struct Point"));
        assert!(check_declarations("enum Color"));
        assert!(check_declarations("use std"));
        assert!(check_declarations("mod test"));
        assert!(check_declarations("#[test]"));
        assert!(!check_declarations("let x = 5"));
    }
    
    fn check_declarations(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("fn ") || 
        trimmed.starts_with("fun ") ||
        trimmed.starts_with("struct ") ||
        trimmed.starts_with("enum ") ||
        trimmed.starts_with("use ") ||
        trimmed.starts_with("mod ") ||
        trimmed.starts_with("#[")
    }
    
    #[test]
    fn test_block_start_detection() {
        assert!(check_block_start("impl Test {"));
        assert!(check_block_start("match x {"));
        assert!(!check_block_start("let x = vec!{1, 2}"));
        assert!(!check_block_start("let f = |x| {x + 1}"));
    }
    
    fn check_block_start(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.ends_with('{') && !trimmed.contains('=')
    }
    
    #[test]
    fn test_executable_statements() {
        assert!(check_executable_statement("let x = 5"));
        assert!(check_executable_statement("x = 10"));
        assert!(check_executable_statement("println!(\"hello\")"));
        assert!(check_executable_statement("assert_eq!(1, 1)"));
        assert!(check_executable_statement("return 42"));
        assert!(!check_executable_statement("// comment"));
    }
    
    fn check_executable_statement(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains('=') ||
        trimmed.contains("println") ||
        trimmed.contains("assert") ||
        trimmed.contains("return")
    }
    
    #[test]
    fn test_complex_cases() {
        // Control flow with assignment
        assert!(is_fully_executable("if let Some(x) = opt"));
        // Function call
        assert!(is_fully_executable("calculate_total()"));
        // Method chain
        assert!(is_fully_executable("list.iter().map(|x| x * 2)"));
        // Simple expression
        assert!(is_fully_executable("x + y"));
        // Not executable - just opening brace
        assert!(!is_fully_executable("impl MyTrait {"));
    }
    
    fn is_fully_executable(line: &str) -> bool {
        let trimmed = line.trim();
        
        // Early returns for control flow
        if check_control_flow(trimmed) {
            return true;
        }
        
        // Skip declarations
        if check_declarations(trimmed) {
            return false;
        }
        
        // Skip block starts (but not assignments with braces)
        if check_block_start(trimmed) {
            return false;
        }
        
        // Check for executable statements or expressions
        check_executable_statement(trimmed) || 
        contains_function_call(trimmed) ||
        contains_expression(trimmed)
    }
    
    fn contains_function_call(line: &str) -> bool {
        line.contains("()") && !line.starts_with("fn ") && !line.starts_with("fun ")
    }
    
    fn contains_expression(line: &str) -> bool {
        // Simple heuristic for expressions
        let has_operator = line.contains('+') || line.contains('-') || 
                          line.contains('*') || line.contains('/') ||
                          line.contains('.') || line.contains('|');
        has_operator && !line.starts_with("//") && !line.starts_with("use ")
    }
}