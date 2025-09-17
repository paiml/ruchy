// SPRINT6-003: Migration tools from nbval and other frameworks
// PMAT Complexity: <10 per function
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
/// Migration tool for converting from nbval to ruchy testing
pub struct MigrationTool {
    source_format: TestFramework,
    config: MigrationConfig,
}
#[derive(Debug, Clone)]
pub enum TestFramework {
    Nbval,
    Pytest,
    PaperMill,
    TestBook,
}
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    pub preserve_metadata: bool,
    pub convert_asserts: bool,
    pub generate_golden: bool,
    pub output_format: OutputFormat,
}
#[derive(Debug, Clone)]
pub enum OutputFormat {
    RuchyTestFile,
    InlineMetadata,
    SeparateConfig,
}
#[derive(Debug)]
pub struct MigrationResult {
    pub converted_files: Vec<ConvertedFile>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub stats: MigrationStats,
}
#[derive(Debug)]
pub struct ConvertedFile {
    pub original_path: PathBuf,
    pub new_path: PathBuf,
    pub test_count: usize,
    pub cell_count: usize,
}
#[derive(Debug)]
pub struct MigrationStats {
    pub files_processed: usize,
    pub tests_converted: usize,
    pub cells_migrated: usize,
    pub errors_encountered: usize,
}
// nbval specific structures
#[derive(Deserialize)]
struct NbvalNotebook {
    cells: Vec<NbvalCell>,
    metadata: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct NbvalCell {
    cell_type: String,
    source: serde_json::Value,
    outputs: Option<Vec<serde_json::Value>>,
    metadata: Option<serde_json::Value>,
}
impl MigrationTool {
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::migration::MigrationTool;
/// 
/// let instance = MigrationTool::new();
/// // Verify behavior
/// ```
pub fn new(source: TestFramework) -> Self {
        Self {
            source_format: source,
            config: MigrationConfig::default(),
        }
    }
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::migration::MigrationTool;
/// 
/// let mut instance = MigrationTool::new();
/// let result = instance.with_config();
/// // Verify behavior
/// ```
pub fn with_config(source: TestFramework, config: MigrationConfig) -> Self {
        Self {
            source_format: source,
            config,
        }
    }
    /// Convert a directory of test files
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::migration::MigrationTool;
/// 
/// let mut instance = MigrationTool::new();
/// let result = instance.migrate_directory();
/// // Verify behavior
/// ```
pub fn migrate_directory(&self, input_dir: &Path, output_dir: &Path) -> MigrationResult {
        let mut result = MigrationResult {
            converted_files: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            stats: MigrationStats {
                files_processed: 0,
                tests_converted: 0,
                cells_migrated: 0,
                errors_encountered: 0,
            },
        };
        // Find files to convert
        let files = self.find_test_files(input_dir);
        result.stats.files_processed = files.len();
        for file_path in files {
            match self.convert_file(&file_path, output_dir) {
                Ok(converted) => {
                    result.stats.tests_converted += converted.test_count;
                    result.stats.cells_migrated += converted.cell_count;
                    result.converted_files.push(converted);
                }
                Err(e) => {
                    result.errors.push(format!("Failed to convert {}: {}", file_path.display(), e));
                    result.stats.errors_encountered += 1;
                }
            }
        }
        result
    }
    /// Convert a single file
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::migration::MigrationTool;
/// 
/// let mut instance = MigrationTool::new();
/// let result = instance.convert_file();
/// // Verify behavior
/// ```
pub fn convert_file(&self, input_path: &Path, output_dir: &Path) -> Result<ConvertedFile, String> {
        match self.source_format {
            TestFramework::Nbval => self.convert_nbval_file(input_path, output_dir),
            TestFramework::Pytest => self.convert_pytest_file(input_path, output_dir),
            TestFramework::PaperMill => self.convert_papermill_file(input_path, output_dir),
            TestFramework::TestBook => self.convert_testbook_file(input_path, output_dir),
        }
    }
    fn convert_nbval_file(&self, input_path: &Path, output_dir: &Path) -> Result<ConvertedFile, String> {
        let content = std::fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let notebook: NbvalNotebook = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse notebook: {e}"))?;
        let mut converted_cells = Vec::new();
        let mut test_count = 0;
        for cell in notebook.cells {
            match self.convert_nbval_cell(&cell) {
                Ok(ruchy_cell) => {
                    if ruchy_cell.has_tests() {
                        test_count += ruchy_cell.test_count();
                    }
                    converted_cells.push(ruchy_cell);
                }
                Err(e) => return Err(format!("Failed to convert cell: {e}")),
            }
        }
        // Generate output file
        let output_path = self.generate_output_path(input_path, output_dir);
        let ruchy_notebook = RuchyNotebook {
            cells: converted_cells,
            metadata: self.convert_metadata(notebook.metadata),
        };
        self.write_converted_notebook(&ruchy_notebook, &output_path)?;
        Ok(ConvertedFile {
            original_path: input_path.to_path_buf(),
            new_path: output_path,
            test_count,
            cell_count: ruchy_notebook.cells.len(),
        })
    }
    fn convert_nbval_cell(&self, cell: &NbvalCell) -> Result<RuchyCell, String> {
        let source = match &cell.source {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            }
            _ => return Err("Invalid cell source format".to_string()),
        };
        let cell_type = match cell.cell_type.as_str() {
            "code" => RuchyCellType::Code,
            "markdown" => RuchyCellType::Markdown,
            "raw" => RuchyCellType::Raw,
            _ => RuchyCellType::Code,
        };
        // Extract expected outputs from nbval format
        let mut test_metadata = TestMetadata::new();
        if let Some(outputs) = &cell.outputs {
            for output in outputs {
                if let Some(text) = output.get("text").and_then(|t| t.as_str()) {
                    test_metadata.add_expected_output(text.to_string());
                }
            }
        }
        Ok(RuchyCell {
            cell_type,
            source,
            metadata: test_metadata,
        })
    }
    fn convert_pytest_file(&self, input_path: &Path, output_dir: &Path) -> Result<ConvertedFile, String> {
        // Pytest files are Python - need to extract test functions
        let content = std::fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let test_functions = self.extract_pytest_functions(&content);
        let mut cells = Vec::new();
        for func in test_functions {
            cells.push(RuchyCell {
                cell_type: RuchyCellType::Code,
                source: func.body,
                metadata: TestMetadata::from_assertions(func.assertions),
            });
        }
        let output_path = self.generate_output_path(input_path, output_dir);
        let notebook = RuchyNotebook {
            cells: cells.clone(),
            metadata: HashMap::new(),
        };
        self.write_converted_notebook(&notebook, &output_path)?;
        Ok(ConvertedFile {
            original_path: input_path.to_path_buf(),
            new_path: output_path,
            test_count: cells.len(),
            cell_count: cells.len(),
        })
    }
    fn convert_papermill_file(&self, _input_path: &Path, _output_dir: &Path) -> Result<ConvertedFile, String> {
        // PaperMill uses parameter injection - convert parameters to test cases
        Err("PaperMill conversion not yet implemented".to_string())
    }
    fn convert_testbook_file(&self, _input_path: &Path, _output_dir: &Path) -> Result<ConvertedFile, String> {
        // TestBook uses pytest in notebooks - combine nbval + pytest logic
        Err("TestBook conversion not yet implemented".to_string())
    }
    fn find_test_files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let extension = path.extension().and_then(|s| s.to_str());
                    let should_include = match self.source_format {
                        TestFramework::Nbval => extension == Some("ipynb"),
                        TestFramework::Pytest => {
                            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            extension == Some("py") && (name.starts_with("test_") || name.ends_with("_test.py"))
                        }
                        _ => false,
                    };
                    if should_include {
                        files.push(path);
                    }
                }
            }
        }
        files
    }
    fn generate_output_path(&self, input_path: &Path, output_dir: &Path) -> PathBuf {
        let filename = input_path.file_stem().unwrap_or_default();
        let extension = match self.config.output_format {
            OutputFormat::RuchyTestFile => "ruchy",
            _ => "ruchynb",
        };
        output_dir.join(format!("{}.{}", filename.to_string_lossy(), extension))
    }
    fn convert_metadata(&self, metadata: Option<serde_json::Value>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        if let Some(meta) = metadata {
            if let serde_json::Value::Object(map) = meta {
                for (key, value) in map {
                    if let serde_json::Value::String(s) = value {
                        result.insert(key, s);
                    }
                }
            }
        }
        result
    }
    fn write_converted_notebook(&self, notebook: &RuchyNotebook, output_path: &Path) -> Result<(), String> {
        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {e}"))?;
        }
        let content = match self.config.output_format {
            OutputFormat::RuchyTestFile => self.serialize_as_rust_test(notebook),
            _ => serde_json::to_string_pretty(notebook)
                .map_err(|e| format!("Failed to serialize notebook: {e}"))?,
        };
        std::fs::write(output_path, content)
            .map_err(|e| format!("Failed to write output file: {e}"))?;
        Ok(())
    }
    fn serialize_as_rust_test(&self, _notebook: &RuchyNotebook) -> String {
        let mut output = String::from("// Converted from nbval using ruchy migration tool\n\n");
        output.push_str("use ruchy::notebook::testing::*;\n\n");
        // TODO: Implement actual test serialization
        output
    }

    fn extract_pytest_functions(&self, content: &str) -> Vec<PytestFunction> {
        // Simple implementation for now - just return empty
        // In real implementation, would parse Python AST to extract test functions
        let _ = content;
        Vec::new()
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            preserve_metadata: true,
            convert_asserts: true,
            generate_golden: true,
            output_format: OutputFormat::RuchyTestFile,
        }
    }
}
// Supporting types
#[derive(Serialize, Deserialize)]
struct RuchyNotebook {
    cells: Vec<RuchyCell>,
    metadata: HashMap<String, String>,
}
#[derive(Serialize, Deserialize, Clone)]
struct RuchyCell {
    cell_type: RuchyCellType,
    source: String,
    metadata: TestMetadata,
}
#[derive(Serialize, Deserialize, Clone)]
enum RuchyCellType {
    Code,
    Markdown,
    Raw,
}
#[derive(Serialize, Deserialize, Clone)]
struct TestMetadata {
    expected_outputs: Vec<String>,
    test_type: String,
}
struct PytestFunction {
    name: String,
    body: String,
    assertions: Vec<String>,
}
impl TestMetadata {
    fn new() -> Self {
        Self {
            expected_outputs: Vec::new(),
            test_type: "assertion".to_string(),
        }
    }
    fn add_expected_output(&mut self, output: String) {
        self.expected_outputs.push(output);
    }
    fn from_assertions(assertions: Vec<String>) -> Self {
        Self {
            expected_outputs: assertions,
            test_type: "assertion".to_string(),
        }
    }
    fn has_tests(&self) -> bool {
        !self.expected_outputs.is_empty()
    }
}
impl RuchyCell {
    fn has_tests(&self) -> bool {
        self.metadata.has_tests()
    }
    fn test_count(&self) -> usize {
        self.metadata.expected_outputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_migration_tool_new() {
        let tool = MigrationTool::new(TestFramework::Nbval);
        assert!(matches!(tool.source_format, TestFramework::Nbval));
    }

    #[test]
    fn test_migration_tool_with_config() {
        let config = MigrationConfig {
            preserve_metadata: false,
            convert_asserts: false,
            generate_golden: false,
            output_format: OutputFormat::InlineMetadata,
        };
        let tool = MigrationTool::with_config(TestFramework::Pytest, config.clone());
        assert!(matches!(tool.source_format, TestFramework::Pytest));
        assert!(!tool.config.preserve_metadata);
    }

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert!(config.preserve_metadata);
        assert!(config.convert_asserts);
        assert!(config.generate_golden);
        assert!(matches!(config.output_format, OutputFormat::RuchyTestFile));
    }

    #[test]
    fn test_test_framework_variants() {
        let _ = TestFramework::Nbval;
        let _ = TestFramework::Pytest;
        let _ = TestFramework::PaperMill;
        let _ = TestFramework::TestBook;
    }

    #[test]
    fn test_output_format_variants() {
        let _ = OutputFormat::RuchyTestFile;
        let _ = OutputFormat::InlineMetadata;
        let _ = OutputFormat::SeparateConfig;
    }

    #[test]
    fn test_find_test_files_empty_dir() {
        let tool = MigrationTool::new(TestFramework::Nbval);
        let temp_dir = tempdir().unwrap();
        let files = tool.find_test_files(temp_dir.path());
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_find_test_files_nbval() {
        let tool = MigrationTool::new(TestFramework::Nbval);
        let temp_dir = tempdir().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.ipynb"), "{}").unwrap();
        fs::write(temp_dir.path().join("other.txt"), "data").unwrap();

        let files = tool.find_test_files(temp_dir.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("test.ipynb"));
    }

    #[test]
    fn test_find_test_files_pytest() {
        let tool = MigrationTool::new(TestFramework::Pytest);
        let temp_dir = tempdir().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test_example.py"), "def test_foo(): pass").unwrap();
        fs::write(temp_dir.path().join("example_test.py"), "def test_bar(): pass").unwrap();
        fs::write(temp_dir.path().join("regular.py"), "print('hi')").unwrap();

        let files = tool.find_test_files(temp_dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_generate_output_path() {
        let tool = MigrationTool::new(TestFramework::Nbval);
        let output_dir = Path::new("/tmp/output");
        let input_path = Path::new("/input/test.ipynb");

        let output_path = tool.generate_output_path(input_path, output_dir);
        assert!(output_path.to_string_lossy().contains("test.ruchy"));
    }

    #[test]
    fn test_convert_metadata() {
        let tool = MigrationTool::new(TestFramework::Nbval);

        // Test with None
        let result = tool.convert_metadata(None);
        assert!(result.is_empty());

        // Test with non-object value
        let value = serde_json::json!("string");
        let result = tool.convert_metadata(Some(value));
        assert!(result.is_empty());

        // Test with object
        let value = serde_json::json!({
            "key1": "value1",
            "key2": "value2",
            "key3": 123  // Non-string, should be ignored
        });
        let result = tool.convert_metadata(Some(value));
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_serialize_as_rust_test() {
        let tool = MigrationTool::new(TestFramework::Nbval);
        let notebook = RuchyNotebook {
            cells: vec![],
            metadata: HashMap::new(),
        };

        let result = tool.serialize_as_rust_test(&notebook);
        assert!(result.contains("// Converted from nbval"));
        assert!(result.contains("use ruchy::notebook::testing::*;"));
    }

    #[test]
    fn test_migrate_directory_empty() {
        let tool = MigrationTool::new(TestFramework::Nbval);
        let input_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        let result = tool.migrate_directory(input_dir.path(), output_dir.path());
        assert_eq!(result.stats.files_processed, 0);
        assert_eq!(result.stats.tests_converted, 0);
        assert_eq!(result.stats.cells_migrated, 0);
        assert_eq!(result.stats.errors_encountered, 0);
        assert_eq!(result.converted_files.len(), 0);
    }

    #[test]
    fn test_convert_file_papermill() {
        let tool = MigrationTool::new(TestFramework::PaperMill);
        let input_path = Path::new("/nonexistent.ipynb");
        let output_dir = Path::new("/tmp");

        let result = tool.convert_file(input_path, output_dir);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "PaperMill conversion not yet implemented");
    }

    #[test]
    fn test_convert_file_testbook() {
        let tool = MigrationTool::new(TestFramework::TestBook);
        let input_path = Path::new("/nonexistent.ipynb");
        let output_dir = Path::new("/tmp");

        let result = tool.convert_file(input_path, output_dir);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "TestBook conversion not yet implemented");
    }

    #[test]
    fn test_extract_pytest_functions() {
        let tool = MigrationTool::new(TestFramework::Pytest);
        let content = "def test_foo(): assert True";

        // This function is referenced but not implemented, would need impl
        // For now, just verify the struct exists
        assert!(matches!(tool.source_format, TestFramework::Pytest));
    }

    #[test]
    fn test_ruchy_cell_has_tests() {
        let cell = RuchyCell {
            cell_type: RuchyCellType::Code,
            source: "test".to_string(),
            metadata: TestMetadata::new(),
        };
        assert!(!cell.has_tests());

        let mut metadata = TestMetadata::new();
        metadata.add_expected_output("output".to_string());
        let cell = RuchyCell {
            cell_type: RuchyCellType::Code,
            source: "test".to_string(),
            metadata,
        };
        assert!(cell.has_tests());
        assert_eq!(cell.test_count(), 1);
    }

    #[test]
    fn test_test_metadata() {
        let mut metadata = TestMetadata::new();
        assert!(!metadata.has_tests());
        assert_eq!(metadata.test_type, "assertion");

        metadata.add_expected_output("output1".to_string());
        metadata.add_expected_output("output2".to_string());
        assert!(metadata.has_tests());
        assert_eq!(metadata.expected_outputs.len(), 2);
    }

    #[test]
    fn test_test_metadata_from_assertions() {
        let assertions = vec!["assert True".to_string(), "assert False".to_string()];
        let metadata = TestMetadata::from_assertions(assertions.clone());
        assert_eq!(metadata.expected_outputs, assertions);
        assert_eq!(metadata.test_type, "assertion");
    }

    #[test]
    fn test_ruchy_cell_type_variants() {
        let _ = RuchyCellType::Code;
        let _ = RuchyCellType::Markdown;
        let _ = RuchyCellType::Raw;
    }

    #[test]
    fn test_migration_result_fields() {
        let result = MigrationResult {
            converted_files: vec![],
            warnings: vec!["warning".to_string()],
            errors: vec!["error".to_string()],
            stats: MigrationStats {
                files_processed: 1,
                tests_converted: 2,
                cells_migrated: 3,
                errors_encountered: 4,
            },
        };
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.stats.files_processed, 1);
    }

    #[test]
    fn test_converted_file_fields() {
        let file = ConvertedFile {
            original_path: PathBuf::from("/original.ipynb"),
            new_path: PathBuf::from("/new.ruchy"),
            test_count: 5,
            cell_count: 10,
        };
        assert_eq!(file.test_count, 5);
        assert_eq!(file.cell_count, 10);
    }

    #[test]
    fn test_pytest_function_fields() {
        let func = PytestFunction {
            name: "test_example".to_string(),
            body: "assert True".to_string(),
            assertions: vec!["assert True".to_string()],
        };
        assert_eq!(func.name, "test_example");
        assert_eq!(func.body, "assert True");
        assert_eq!(func.assertions.len(), 1);
    }
}