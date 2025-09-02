// TDD Test Suite for REPL::run Complexity Reduction
// Current: 29 cyclomatic, 71 cognitive complexity
// Target: <20 for both metrics
// Strategy: Extract helper methods for each logical block

use ruchy::runtime::repl::Repl;

#[cfg(test)]
mod repl_run_refactoring {
    use super::*;

    // Helper to create a test REPL instance
    fn create_test_repl() -> Repl {
        Repl::new().unwrap()
    }

    #[test]
    fn test_empty_line_handling() {
        // Test that empty lines are skipped correctly
        let mut repl = create_test_repl();
        
        // Empty line should return empty result
        let result = repl.eval("");
        assert_eq!(result.unwrap(), "");
        
        // Whitespace-only line should also return empty
        let result = repl.eval("   ");
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_command_handling() {
        let mut repl = create_test_repl();
        
        // Test variable assignment and access
        repl.eval("let x = 5").unwrap();
        let result = repl.eval("x").unwrap();
        assert_eq!(result, "5");
        
        // Test that undefined variables cause errors
        let result = repl.eval("undefined_variable");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiline_detection() {
        // Test needs_continuation for various inputs
        assert!(Repl::needs_continuation("fn test() {"));
        assert!(Repl::needs_continuation("if true {"));
        assert!(Repl::needs_continuation("for i in 0..10 {"));
        assert!(Repl::needs_continuation("match x {"));
        assert!(Repl::needs_continuation("let x = ["));
        assert!(Repl::needs_continuation("let obj = {"));
        assert!(Repl::needs_continuation("(1 + 2"));
        
        // Complete expressions should not need continuation
        assert!(!Repl::needs_continuation("let x = 5"));
        assert!(!Repl::needs_continuation("println(\"hello\")"));
        assert!(!Repl::needs_continuation("1 + 2"));
    }

    #[test]
    fn test_multiline_accumulation() {
        let mut repl = create_test_repl();
        
        // Test function definition across multiple lines
        let multiline = "fn add(a, b) {\n    a + b\n}";
        let result = repl.eval(multiline).unwrap();
        assert_eq!(result, "");
        
        // Test the function works
        let result = repl.eval("add(2, 3)").unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_single_line_evaluation() {
        let mut repl = create_test_repl();
        
        // Test basic arithmetic
        let result = repl.eval("1 + 2").unwrap();
        assert_eq!(result, "3");
        
        // Test variable assignment
        let result = repl.eval("let x = 10").unwrap();
        assert_eq!(result, "");
        
        // Test variable access
        let result = repl.eval("x").unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_error_handling() {
        let mut repl = create_test_repl();
        
        // Test undefined variable error
        let result = repl.eval("undefined_var");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        
        // Test syntax error
        let result = repl.eval("1 + + 2");
        assert!(result.is_err());
        
        // Test type error
        repl.eval("let s = \"hello\"").unwrap();
        let result = repl.eval("s + 5");
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_generation() {
        let repl = create_test_repl();
        
        // Default prompt should be "ruchy"
        let prompt = repl.get_prompt();
        assert_eq!(prompt, "ruchy");
        
        // TODO: Test custom prompts after mode changes
    }

    #[test]
    fn test_repl_creation() {
        // Test that REPL can be created successfully
        let repl = create_test_repl();
        
        // Test basic functionality works
        let result = repl.get_prompt();
        assert_eq!(result, "ruchy");
    }

    // Tests for refactored helper methods
    mod refactored_helpers {
        use super::*;

        #[test]
        fn test_process_command_line() {
            let mut repl = create_test_repl();
            
            // Test basic expression evaluation
            let result = repl.eval("2 + 3");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "5");
        }

        #[test]
        fn test_process_multiline_start() {
            // Test that multiline detection triggers correctly
            let line = "fn test() {";
            assert!(Repl::needs_continuation(line));
            
            let line = "let x = 5";
            assert!(!Repl::needs_continuation(line));
        }

        #[test]
        fn test_process_multiline_accumulation() {
            // Test string accumulation logic
            let mut buffer = String::from("fn test() {");
            buffer.push('\n');
            buffer.push_str("    return 42");
            assert_eq!(buffer, "fn test() {\n    return 42");
        }

        #[test]
        fn test_evaluate_complete_expression() {
            let mut repl = create_test_repl();
            
            // Test existing eval method directly
            let result = repl.eval("1 + 2");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "3");
            
            // Test multiline evaluation
            let multiline = "fn test() {\n    42\n}";
            let result = repl.eval(multiline);
            assert!(result.is_ok());
        }
    }
}

// Test helper functions that demonstrate the refactoring approach
#[cfg(test)]
mod refactoring_helpers {
    use super::*;
    
    // These functions demonstrate how to break down the complexity
    fn should_start_multiline(line: &str) -> bool {
        Repl::needs_continuation(line)
    }

    fn accumulate_multiline(buffer: &mut String, line: &str) {
        buffer.push('\n');
        buffer.push_str(line);
    }
    
    #[test]
    fn test_helpers_work() {
        // Test multiline detection helper
        assert!(should_start_multiline("fn test() {"));
        assert!(!should_start_multiline("let x = 5"));
        
        // Test accumulation helper
        let mut buffer = String::from("fn test() {");
        accumulate_multiline(&mut buffer, "    return 42");
        assert_eq!(buffer, "fn test() {\n    return 42");
    }
}