use crate::notebook::testing::types::CellOutput;
use std::path::{Path, PathBuf};

/// Golden file manager for test snapshots
pub struct GoldenManager {
    base_path: PathBuf,
}

impl GoldenManager {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::golden::GoldenManager;
    ///
    /// let instance = GoldenManager::new();
    /// // Verify behavior
    /// ```
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::golden::GoldenManager;
    ///
    /// let mut instance = GoldenManager::new();
    /// let result = instance.save_golden();
    /// // Verify behavior
    /// ```
    pub fn save_golden(&self, path: &Path, output: &CellOutput) -> Result<(), String> {
        let full_path = self.base_path.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
        let content = match output {
            CellOutput::Value(v) => v.clone(),
            CellOutput::DataFrame(df) => format!("{df:?}"),
            CellOutput::Error(e) => e.clone(),
            CellOutput::Html(h) => h.clone(),
            CellOutput::Plot(p) => format!("{p:?}"),
            CellOutput::None => String::new(),
        };
        std::fs::write(&full_path, content).map_err(|e| format!("Failed to write golden file: {e}"))
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::golden::GoldenManager;
    ///
    /// let mut instance = GoldenManager::new();
    /// let result = instance.load_golden();
    /// // Verify behavior
    /// ```
    pub fn load_golden(&self, path: &Path) -> Result<CellOutput, String> {
        let full_path = self.base_path.join(path);
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read golden file: {e}"))?;
        // For Sprint 0, assume all goldens are simple values
        Ok(CellOutput::Value(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // EXTREME TDD: Comprehensive test coverage for golden file management system

    #[test]
    fn test_golden_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());

        assert_eq!(manager.base_path, temp_dir.path());
    }

    #[test]
    fn test_golden_manager_new_with_pathbuf() {
        let path = PathBuf::from("/tmp/test");
        let manager = GoldenManager::new(&path);

        assert_eq!(manager.base_path, path);
    }

    #[test]
    fn test_save_golden_value_output() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::Value("test_value".to_string());

        let result = manager.save_golden(&PathBuf::from("test.golden"), &output);

        assert!(result.is_ok());

        let saved_content = std::fs::read_to_string(temp_dir.path().join("test.golden")).unwrap();
        assert_eq!(saved_content, "test_value");
    }

    #[test]
    fn test_save_golden_error_output() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::Error("RuntimeError: test error".to_string());

        let result = manager.save_golden(&PathBuf::from("error.golden"), &output);

        assert!(result.is_ok());

        let saved_content = std::fs::read_to_string(temp_dir.path().join("error.golden")).unwrap();
        assert_eq!(saved_content, "RuntimeError: test error");
    }

    #[test]
    fn test_save_golden_html_output() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::Html("<h1>Test HTML</h1>".to_string());

        let result = manager.save_golden(&PathBuf::from("html.golden"), &output);

        assert!(result.is_ok());

        let saved_content = std::fs::read_to_string(temp_dir.path().join("html.golden")).unwrap();
        assert_eq!(saved_content, "<h1>Test HTML</h1>");
    }

    #[test]
    fn test_save_golden_none_output() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::None;

        let result = manager.save_golden(&PathBuf::from("none.golden"), &output);

        assert!(result.is_ok());

        let saved_content = std::fs::read_to_string(temp_dir.path().join("none.golden")).unwrap();
        assert_eq!(saved_content, "");
    }

    #[test]
    fn test_save_golden_with_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::Value("nested_value".to_string());

        let result = manager.save_golden(&PathBuf::from("nested/deep/test.golden"), &output);

        assert!(result.is_ok());

        let saved_content =
            std::fs::read_to_string(temp_dir.path().join("nested/deep/test.golden")).unwrap();
        assert_eq!(saved_content, "nested_value");
    }

    #[test]
    fn test_save_golden_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::Value("test".to_string());

        let nested_path = PathBuf::from("level1/level2/level3/test.golden");
        let result = manager.save_golden(&nested_path, &output);

        assert!(result.is_ok());
        assert!(temp_dir.path().join("level1").exists());
        assert!(temp_dir.path().join("level1/level2").exists());
        assert!(temp_dir.path().join("level1/level2/level3").exists());
    }

    #[test]
    fn test_load_golden_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());

        // First save a golden file
        let output = CellOutput::Value("saved_content".to_string());
        manager
            .save_golden(&PathBuf::from("test.golden"), &output)
            .unwrap();

        // Then load it
        let result = manager.load_golden(&PathBuf::from("test.golden"));

        assert!(result.is_ok());
        match result.unwrap() {
            CellOutput::Value(content) => assert_eq!(content, "saved_content"),
            _ => panic!("Expected Value output"),
        }
    }

    #[test]
    fn test_load_golden_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());

        let result = manager.load_golden(&PathBuf::from("nonexistent.golden"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read golden file"));
    }

    #[test]
    fn test_load_golden_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());

        // Create empty file
        std::fs::write(temp_dir.path().join("empty.golden"), "").unwrap();

        let result = manager.load_golden(&PathBuf::from("empty.golden"));

        assert!(result.is_ok());
        match result.unwrap() {
            CellOutput::Value(content) => assert_eq!(content, ""),
            _ => panic!("Expected Value output"),
        }
    }

    #[test]
    fn test_round_trip_value() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let original = CellOutput::Value("round_trip_test".to_string());

        manager
            .save_golden(&PathBuf::from("round_trip.golden"), &original)
            .unwrap();
        let loaded = manager
            .load_golden(&PathBuf::from("round_trip.golden"))
            .unwrap();

        match loaded {
            CellOutput::Value(content) => assert_eq!(content, "round_trip_test"),
            _ => panic!("Expected Value output"),
        }
    }

    #[test]
    fn test_round_trip_error() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let original = CellOutput::Error("Error message".to_string());

        manager
            .save_golden(&PathBuf::from("error.golden"), &original)
            .unwrap();
        let loaded = manager.load_golden(&PathBuf::from("error.golden")).unwrap();

        match loaded {
            CellOutput::Value(content) => assert_eq!(content, "Error message"),
            _ => panic!("Expected Value output"),
        }
    }

    #[test]
    fn test_save_golden_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let special_content = "Special chars: 🦀 日本語 αβγ \"quotes\" 'apostrophes' \n\t\r";
        let output = CellOutput::Value(special_content.to_string());

        let result = manager.save_golden(&PathBuf::from("special.golden"), &output);

        assert!(result.is_ok());

        let saved_content =
            std::fs::read_to_string(temp_dir.path().join("special.golden")).unwrap();
        assert_eq!(saved_content, special_content);
    }

    #[test]
    fn test_save_golden_large_content() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let large_content = "x".repeat(100000);
        let output = CellOutput::Value(large_content.clone());

        let result = manager.save_golden(&PathBuf::from("large.golden"), &output);

        assert!(result.is_ok());

        let saved_content = std::fs::read_to_string(temp_dir.path().join("large.golden")).unwrap();
        assert_eq!(saved_content, large_content);
    }

    #[test]
    fn test_multiple_files_same_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());

        let outputs = vec![
            ("file1.golden", CellOutput::Value("content1".to_string())),
            ("file2.golden", CellOutput::Error("error2".to_string())),
            (
                "file3.golden",
                CellOutput::Html("<div>html3</div>".to_string()),
            ),
        ];

        for (filename, output) in &outputs {
            let result = manager.save_golden(&PathBuf::from(filename), output);
            assert!(result.is_ok());
        }

        // Verify all files exist and have correct content
        for (filename, expected_output) in outputs {
            let loaded = manager.load_golden(&PathBuf::from(&filename)).unwrap();
            let expected_content = match expected_output {
                CellOutput::Value(s) | CellOutput::Error(s) | CellOutput::Html(s) => s,
                CellOutput::DataFrame(df) => format!("{df:?}"),
                CellOutput::Plot(p) => format!("{p:?}"),
                CellOutput::None => String::new(),
            };

            match loaded {
                CellOutput::Value(content) => assert_eq!(content, expected_content),
                _ => panic!("Expected Value output"),
            }
        }
    }

    #[test]
    fn test_path_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());
        let output = CellOutput::Value("test".to_string());

        let result = manager.save_golden(&PathBuf::from("test.txt.golden"), &output);
        assert!(result.is_ok());

        let loaded = manager
            .load_golden(&PathBuf::from("test.txt.golden"))
            .unwrap();
        match loaded {
            CellOutput::Value(content) => assert_eq!(content, "test"),
            _ => panic!("Expected Value output"),
        }
    }

    #[test]
    fn test_overwrite_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = GoldenManager::new(temp_dir.path());

        // Save initial content
        let initial = CellOutput::Value("initial".to_string());
        manager
            .save_golden(&PathBuf::from("overwrite.golden"), &initial)
            .unwrap();

        // Overwrite with new content
        let updated = CellOutput::Value("updated".to_string());
        manager
            .save_golden(&PathBuf::from("overwrite.golden"), &updated)
            .unwrap();

        // Verify new content
        let loaded = manager
            .load_golden(&PathBuf::from("overwrite.golden"))
            .unwrap();
        match loaded {
            CellOutput::Value(content) => assert_eq!(content, "updated"),
            _ => panic!("Expected Value output"),
        }
    }
}
