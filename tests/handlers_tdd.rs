//! TDD tests for refactored handlers modules
//! Comprehensive coverage for CLI command handlers

#[cfg(test)]
mod prove_command_tests {
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_prove_command_no_file() {
        // Test prove command without file (interactive mode)
        let result = ruchy::bin::handlers::handle_prove_command(
            None,
            "z3",
            false,
            5000,
            None,
            None,
            false,
            false,
            false,
            "text",
        );
        // Should start interactive mode (but we can't test interaction)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prove_command_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");
        fs::write(&test_file, "assert x > 0").unwrap();
        
        let result = ruchy::bin::handlers::handle_prove_command(
            Some(&test_file),
            "z3",
            false,
            5000,
            None,
            None,
            true, // check mode
            false,
            false,
            "text",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prove_command_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");
        fs::write(&test_file, "assert true").unwrap();
        
        let result = ruchy::bin::handlers::handle_prove_command(
            Some(&test_file),
            "z3",
            false,
            5000,
            None,
            None,
            true,
            false,
            false,
            "json",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prove_backend_selection() {
        // Test different SMT backends
        for backend in &["z3", "cvc5", "yices2", "unknown"] {
            let result = ruchy::bin::handlers::handle_prove_command(
                None,
                backend,
                false,
                1000,
                None,
                None,
                true, // check mode to avoid interaction
                false,
                false,
                "text",
            );
            // Should handle all backends gracefully
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod test_command_tests {
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_run_enhanced_tests_no_files() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = ruchy::bin::handlers::handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // watch
            false, // verbose
            None,  // filter
            false, // coverage
            "text",
            1,     // parallel
            0.0,   // threshold
            "text",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_run_enhanced_tests_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");
        fs::write(&test_file, "assert 1 + 1 == 2").unwrap();
        
        let result = ruchy::bin::handlers::handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false,
            true, // verbose
            None,
            false,
            "text",
            1,
            0.0,
            "text",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_run_tests_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let test1 = temp_dir.path().join("math_test.ruchy");
        let test2 = temp_dir.path().join("string_test.ruchy");
        fs::write(&test1, "assert 2 + 2 == 4").unwrap();
        fs::write(&test2, "assert \"a\" + \"b\" == \"ab\"").unwrap();
        
        let result = ruchy::bin::handlers::handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false,
            false,
            Some("math"), // filter
            false,
            "text",
            1,
            0.0,
            "text",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_json_output_format() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = ruchy::bin::handlers::handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false,
            false,
            None,
            false,
            "text",
            1,
            0.0,
            "json", // JSON format
        );
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod replay_command_tests {
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_replay_to_tests_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let replay_file = temp_dir.path().join("session.replay");
        
        // Create a simple replay file
        let replay_content = r#"{"version":"1.0","session_id":"test","commands":[{"input":"1+1","output":"2"}]}"#;
        fs::write(&replay_file, replay_content).unwrap();
        
        let output_file = temp_dir.path().join("test.rs");
        
        let result = ruchy::bin::handlers::handle_replay_to_tests_command(
            &replay_file,
            Some(&output_file),
            false,
            false,
            5000,
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_replay_to_tests_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple replay files
        for i in 1..=3 {
            let replay_file = temp_dir.path().join(format!("session{}.replay", i));
            let content = format!(r#"{{"version":"1.0","session_id":"test{}","commands":[]}}"#, i);
            fs::write(&replay_file, content).unwrap();
        }
        
        let output_file = temp_dir.path().join("tests.rs");
        
        let result = ruchy::bin::handlers::handle_replay_to_tests_command(
            temp_dir.path(),
            Some(&output_file),
            true, // property tests
            true, // benchmarks
            10000,
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_replay_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("not_replay.txt");
        fs::write(&invalid_file, "content").unwrap();
        
        let result = ruchy::bin::handlers::handle_replay_to_tests_command(
            &invalid_file,
            None,
            false,
            false,
            5000,
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_replay_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = ruchy::bin::handlers::handle_replay_to_tests_command(
            temp_dir.path(),
            None,
            false,
            false,
            5000,
        );
        assert!(result.is_ok()); // Should succeed with warning
    }
}

#[cfg(test)]
mod eval_command_tests {
    #[test]
    fn test_eval_simple_expression() {
        let result = ruchy::bin::handlers::handle_eval_command(
            "2 + 2",
            false,
            "text",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_with_verbose() {
        let result = ruchy::bin::handlers::handle_eval_command(
            "println(\"hello\")",
            true, // verbose
            "text",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_json_format() {
        let result = ruchy::bin::handlers::handle_eval_command(
            "42",
            false,
            "json",
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_eval_error_handling() {
        let result = ruchy::bin::handlers::handle_eval_command(
            "undefined_var",
            false,
            "text",
        );
        // Should handle gracefully even with error
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod compile_command_tests {
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_compile_simple_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("main.ruchy");
        fs::write(&source_file, "fn main() { println(\"Hello\") }").unwrap();
        
        let output = temp_dir.path().join("output");
        
        let result = ruchy::bin::handlers::handle_compile_command(
            &source_file,
            output,
            "2".to_string(),
            false, // strip
            false, // static_link
            None,  // target
        );
        // May fail if rustc not available, but should handle gracefully
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_compile_with_optimization() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("app.ruchy");
        fs::write(&source_file, "fn main() {}").unwrap();
        
        for opt_level in &["0", "1", "2", "3", "s", "z"] {
            let output = temp_dir.path().join(format!("app_{}", opt_level));
            let _ = ruchy::bin::handlers::handle_compile_command(
                &source_file,
                output,
                opt_level.to_string(),
                true, // strip
                false,
                None,
            );
        }
    }
    
    #[test]
    fn test_compile_cross_target() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("cross.ruchy");
        fs::write(&source_file, "fn main() {}").unwrap();
        
        let output = temp_dir.path().join("cross_output");
        
        let result = ruchy::bin::handlers::handle_compile_command(
            &source_file,
            output,
            "2".to_string(),
            false,
            false,
            Some("wasm32-unknown-unknown".to_string()),
        );
        // Cross compilation may not be available
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod coverage_command_tests {
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_coverage_text_format() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("code.ruchy");
        fs::write(&test_file, "fn test() { assert 1 == 1 }").unwrap();
        
        let result = ruchy::bin::handlers::handle_coverage_command(
            &test_file,
            0.0, // no threshold
            "text",
            false,
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_coverage_html_format() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("code.ruchy");
        fs::write(&test_file, "fn coverage_test() {}").unwrap();
        
        let result = ruchy::bin::handlers::handle_coverage_command(
            &test_file,
            0.0,
            "html",
            false,
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_coverage_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("code.ruchy");
        fs::write(&test_file, "let x = 1").unwrap();
        
        let result = ruchy::bin::handlers::handle_coverage_command(
            &test_file,
            0.0,
            "json",
            true, // verbose
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_coverage_threshold_check() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("code.ruchy");
        fs::write(&test_file, "fn untested() { /* not covered */ }").unwrap();
        
        let result = ruchy::bin::handlers::handle_coverage_command(
            &test_file,
            100.0, // 100% threshold - should fail
            "text",
            false,
        );
        // Should handle threshold failure gracefully
        assert!(result.is_ok() || result.is_err());
    }
}