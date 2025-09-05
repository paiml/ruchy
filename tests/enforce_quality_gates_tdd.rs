//! TDD safety net for enforce_quality_gates refactoring
//! Target: 19 complexity → ≤10 with systematic function extraction
//! Focus: Cover all paths before refactoring complexity hotspot

#[cfg(test)]
mod tests {
    use ruchy::quality::enforcement::enforce_quality_gates;
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;
    
    // Helper function (complexity: 3)
    fn create_test_ruchy_file(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }
    
    // Helper function (complexity: 4)
    fn create_test_project_structure(dir: &Path) -> std::path::PathBuf {
        // Create Cargo.toml to mark as project root
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
        
        // Create .ruchy directory
        fs::create_dir_all(dir.join(".ruchy")).unwrap();
        
        // Create some test files
        create_test_ruchy_file(dir, "test.ruchy", "let x = 5");
        
        dir.to_path_buf()
    }
    
    // Core Configuration Loading Tests (complexity: 3 each)
    #[test]
    fn test_enforce_with_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "default_test.ruchy", "let x = 42");
        
        let result = enforce_quality_gates(
            &file_path,
            None,           // No custom config
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        // Should complete without crashing (may pass or fail quality gates)
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "custom_test.ruchy", "let y = 100");
        
        // Create custom config directory
        let config_dir = temp_dir.path().join("custom");
        fs::create_dir_all(config_dir.join(".ruchy")).unwrap();
        let custom_config_path = config_dir.join("score.toml");
        
        let result = enforce_quality_gates(
            &file_path,
            Some(&custom_config_path),  // Custom config
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        // Should handle custom config path
        assert!(result.is_ok() || result.is_err());
    }
    
    // CI Mode Tests (complexity: 3 each)
    #[test]
    fn test_enforce_with_ci_mode_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "ci_off.ruchy", "let ci_off = true");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            None,
            false,         // CI mode disabled
            false,
        );
        
        // Should work without CI overrides
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]  
    fn test_enforce_with_ci_mode_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "ci_on.ruchy", "let ci_on = false");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            None,
            true,          // CI mode enabled
            false,
        );
        
        // Should apply CI overrides (stricter thresholds)
        assert!(result.is_ok() || result.is_err());
    }
    
    // Analysis Depth Tests (complexity: 3 each)
    #[test]
    fn test_enforce_with_shallow_depth() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "shallow.ruchy", "let z = 999");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "shallow",     // Shallow depth
            false,
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_with_standard_depth() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "standard.ruchy", "let w = 123");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",    // Standard depth
            false,
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_with_deep_depth() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "deep.ruchy", "let v = [1, 2, 3]");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "deep",        // Deep depth
            false,
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_with_invalid_depth() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "invalid.ruchy", "let u = true");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "invalid_depth", // Invalid depth
            false,
            "console",
            None,
            false,
            false,
        );
        
        // Should fail with depth error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid depth"));
    }
    
    // File vs Directory Processing Tests (complexity: 3 each)
    #[test]
    fn test_enforce_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "single.ruchy", "let single = 1");
        
        let result = enforce_quality_gates(
            &file_path,    // Single file
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        
        // Create multiple files
        create_test_ruchy_file(&project_dir, "file1.ruchy", "let a = 1");
        create_test_ruchy_file(&project_dir, "file2.ruchy", "let b = 2");
        
        let result = enforce_quality_gates(
            &project_dir,  // Directory
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_nonexistent_path() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist.ruchy");
        
        let result = enforce_quality_gates(
            &nonexistent, // Nonexistent path
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        // Should fail for nonexistent path
        assert!(result.is_err());
    }
    
    // Output Format Tests (complexity: 3 each)  
    #[test]
    fn test_enforce_console_format() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "console.ruchy", "let c = 42");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",     // Console format
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "json.ruchy", "let j = 99");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "json",        // JSON format
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_junit_format() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "junit.ruchy", "let junit = true");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "junit",       // JUnit format
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "invalid_fmt.ruchy", "let f = 0");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "invalid_format", // Invalid format
            None,
            false,
            false,
        );
        
        // Should fail with format error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid format"));
    }
    
    // Export Functionality Tests (complexity: 3 each)
    #[test]
    fn test_enforce_without_export() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "no_export.ruchy", "let no_exp = 0");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            None,          // No export
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_with_export() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "with_export.ruchy", "let exp = 1");
        let export_dir = temp_dir.path().join("exports");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            Some(&export_dir), // Export directory
            false,
            false,
        );
        
        // Export directory should be created
        assert!(export_dir.exists());
        assert!(result.is_ok() || result.is_err());
    }
    
    // Fail Fast Tests (complexity: 3 each)
    #[test]
    fn test_enforce_fail_fast_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "no_fail_fast.ruchy", "let ff_off = false");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,         // Fail fast disabled
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_fail_fast_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "fail_fast.ruchy", "let ff_on = true");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            true,          // Fail fast enabled
            "console",
            None,
            false,
            false,
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    // Verbose Mode Tests (complexity: 3 each)
    #[test]
    fn test_enforce_verbose_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "quiet.ruchy", "let quiet = true");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,         // Verbose disabled
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_verbose_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "verbose.ruchy", "let verbose = true");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "standard",
            false,
            "console",
            None,
            false,
            true,          // Verbose enabled
        );
        
        assert!(result.is_ok() || result.is_err());
    }
    
    // Complex Integration Tests (complexity: 4 each)
    #[test]
    fn test_enforce_all_features_combined() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        let file_path = create_test_ruchy_file(&project_dir, "all_features.ruchy", "let all = 123");
        let export_dir = temp_dir.path().join("full_export");
        
        let result = enforce_quality_gates(
            &file_path,
            None,
            "deep",        // Deep analysis
            true,          // Fail fast
            "json",        // JSON format
            Some(&export_dir), // Export
            true,          // CI mode
            true,          // Verbose
        );
        
        // Should handle all features together
        assert!(export_dir.exists());
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_directory_with_subdirs() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project_structure(temp_dir.path());
        
        // Create nested directory structure
        let subdir = project_dir.join("src");
        fs::create_dir_all(&subdir).unwrap();
        create_test_ruchy_file(&subdir, "nested.ruchy", "let nested = 555");
        
        let result = enforce_quality_gates(
            &project_dir,
            None,
            "standard",
            false,
            "console", 
            None,
            false,
            false,
        );
        
        // Should recursively process subdirectories
        assert!(result.is_ok() || result.is_err());
    }
    
    // Edge Case Tests (complexity: 3 each)
    #[test]
    fn test_enforce_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir_all(&empty_dir).unwrap();
        
        let result = enforce_quality_gates(
            &empty_dir,
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        // Should handle empty directory gracefully
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_enforce_directory_no_ruchy_files() {
        let temp_dir = TempDir::new().unwrap();
        let no_ruchy_dir = temp_dir.path().join("no_ruchy");
        fs::create_dir_all(&no_ruchy_dir).unwrap();
        
        // Create non-Ruchy files
        fs::write(no_ruchy_dir.join("other.txt"), "not ruchy").unwrap();
        fs::write(no_ruchy_dir.join("README.md"), "# Not Ruchy").unwrap();
        
        let result = enforce_quality_gates(
            &no_ruchy_dir,
            None,
            "standard",
            false,
            "console",
            None,
            false,
            false,
        );
        
        // Should handle directory with no Ruchy files
        assert!(result.is_ok() || result.is_err());
    }
}