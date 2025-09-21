//! Minimal test for EXTREME Quality REPL

#[cfg(test)]
mod tests {
    use super::super::Repl;
    use tempfile::TempDir;

    #[test]
    fn test_extreme_quality_repl_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repl_result = Repl::new(temp_dir.path().to_path_buf());
        assert!(
            repl_result.is_ok(),
            "EXTREME Quality REPL should create successfully"
        );
    }

    #[test]
    fn test_extreme_quality_repl_basic_operation() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Test that process_line doesn't panic with simple input
        let result = repl.process_line("1 + 1");
        assert!(result.is_ok(), "Basic arithmetic should not cause errors");
    }
}
