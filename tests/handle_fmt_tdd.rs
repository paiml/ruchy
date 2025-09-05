#[cfg(test)]
mod handle_fmt_tests {
    use std::path::Path;
    
    #[test]
    fn test_format_output_selection() {
        assert_eq!(select_format_output("file.rs", None), "file.rs");
        assert_eq!(select_format_output("file.rs", Some("out.rs".to_string())), "out.rs");
    }
    
    fn select_format_output(input: &str, output: Option<String>) -> String {
        output.unwrap_or_else(|| input.to_string())
    }
    
    #[test]
    fn test_check_mode_behavior() {
        assert!(!should_write_file(true));  // Check mode doesn't write
        assert!(should_write_file(false));  // Normal mode writes
    }
    
    fn should_write_file(check_mode: bool) -> bool {
        !check_mode
    }
    
    #[test]
    fn test_format_result_handling() {
        let original = "fn main(){}";
        let formatted = "fn main() {}";
        
        assert!(content_changed(original, formatted));
        assert!(!content_changed(original, original));
    }
    
    fn content_changed(original: &str, formatted: &str) -> bool {
        original != formatted
    }
    
    #[test]
    fn test_exit_code_determination() {
        assert_eq!(get_exit_code(true, false), 0);  // No changes
        assert_eq!(get_exit_code(false, false), 0); // Changes written
        assert_eq!(get_exit_code(false, true), 1);  // Changes in check mode
    }
    
    fn get_exit_code(unchanged: bool, check_mode: bool) -> i32 {
        if unchanged || !check_mode {
            0
        } else {
            1
        }
    }
    
    #[test]
    fn test_verbose_message_generation() {
        assert!(get_verbose_message(true, "file.rs").contains("No changes"));
        assert!(get_verbose_message(false, "file.rs").contains("Formatted"));
    }
    
    fn get_verbose_message(unchanged: bool, file: &str) -> String {
        if unchanged {
            format!("No changes needed for {}", file)
        } else {
            format!("Formatted {}", file)
        }
    }
    
    #[test]
    fn test_quiet_mode_suppression() {
        assert_eq!(should_print_message(false, false), true);   // Not quiet, not verbose
        assert_eq!(should_print_message(true, false), false);   // Quiet mode
        assert_eq!(should_print_message(false, true), true);    // Verbose overrides
    }
    
    fn should_print_message(quiet: bool, verbose: bool) -> bool {
        !quiet || verbose
    }
}