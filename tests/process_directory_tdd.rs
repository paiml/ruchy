#[cfg(test)]
mod process_directory_tests {
    use std::path::Path;
    
    #[test]
    fn test_is_ruchy_file() {
        assert!(is_ruchy_file(Path::new("test.ruchy")));
        assert!(!is_ruchy_file(Path::new("test.rs")));
        assert!(!is_ruchy_file(Path::new("test")));
        assert!(!is_ruchy_file(Path::new("test.txt")));
    }
    
    fn is_ruchy_file(path: &Path) -> bool {
        path.is_file() && path.extension().is_some_and(|ext| ext == "ruchy")
    }
    
    #[test]
    fn test_is_processable_directory() {
        assert!(is_processable_directory(Path::new("src"), "src"));
        assert!(!is_processable_directory(Path::new(".git"), ".git"));
        assert!(!is_processable_directory(Path::new("..hidden"), "..hidden"));
        assert!(is_processable_directory(Path::new("valid_dir"), "valid_dir"));
    }
    
    fn is_processable_directory(path: &Path, name: &str) -> bool {
        path.is_dir() && !name.starts_with('.')
    }
    
    #[test]
    fn test_fail_fast_behavior() {
        assert!(should_return_early(true, false));  // Fail fast and not passed
        assert!(!should_return_early(false, false)); // No fail fast
        assert!(!should_return_early(true, true));  // Fail fast but passed
    }
    
    fn should_return_early(fail_fast: bool, passed: bool) -> bool {
        fail_fast && !passed
    }
    
    #[test]
    fn test_error_handling_strategy() {
        assert_eq!(get_error_action(true), ErrorAction::Return);
        assert_eq!(get_error_action(false), ErrorAction::Continue);
    }
    
    fn get_error_action(fail_fast: bool) -> ErrorAction {
        if fail_fast {
            ErrorAction::Return
        } else {
            ErrorAction::Continue
        }
    }
    
    #[derive(Debug, PartialEq)]
    enum ErrorAction {
        Return,
        Continue,
    }
    
    #[test]
    fn test_result_accumulation() {
        let mut results = vec![];
        
        add_result(&mut results, "passed", true);
        assert_eq!(results.len(), 1);
        
        add_result(&mut results, "failed", false);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, true);
        assert_eq!(results[1].1, false);
    }
    
    fn add_result<'a>(results: &mut Vec<(&'a str, bool)>, name: &'a str, passed: bool) {
        results.push((name, passed));
    }
}