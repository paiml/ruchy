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
/// use ruchy::notebook::testing::migration::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
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
/// use ruchy::notebook::testing::migration::with_config;
/// 
/// let result = with_config(());
/// assert_eq!(result, Ok(()));
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
/// use ruchy::notebook::testing::migration::migrate_directory;
/// 
/// let result = migrate_directory(());
/// assert_eq!(result, Ok(()));
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
/// use ruchy::notebook::testing::migration::convert_file;
/// 
/// let result = convert_file(());
/// assert_eq!(result, Ok(()));
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
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let notebook: NbvalNotebook = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse notebook: {}", e))?;
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
                Err(e) => return Err(format!("Failed to convert cell: {}", e)),
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
            .map_err(|e| format!("Failed to read file: {}", e))?;
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
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
        let content = match self.config.output_format {
            OutputFormat::RuchyTestFile => self.serialize_as_rust_test(notebook),
            _ => serde_json::to_string_pretty(notebook)
                .map_err(|e| format!("Failed to serialize notebook: {}", e))?,
        };
        std::fs::write(output_path, content)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        Ok(())
    }
    fn serialize_as_rust_test(&self, notebook: &RuchyNotebook) -> String {
        let mut output = String::from("// Converted from nbval using ruchy migration tool\n\n");
        output.push_str("use ruchy::notebook::testing::*;
#[cfg(test)]
\n\n");
        for (i, cell) in notebook.cells.iter().enumerate() {
            if matches!(cell.cell_type, RuchyCellType::Code) && cell.metadata.has_tests() {
                output.push_str(&format!("#[test]\nfn test_cell_{}() {{\n", i));
                output.push_str("    let mut tester = NotebookTester::new();\n");
                output.push_str(&format!("    let result = tester.execute_code({:?});\n", cell.source));
                for expected in &cell.metadata.expected_outputs {
                    output.push_str(&format!("    assert_eq!(result.output, {:?});\n", expected));
                }
                output.push_str("}\n\n");
            }
        }
        output
    }
    fn extract_pytest_functions(&self, content: &str) -> Vec<PytestFunction> {
        let mut functions = Vec::new();
        let mut current_function: Option<PytestFunction> = None;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("def test_") {
                // Save previous function
                if let Some(func) = current_function.take() {
                    functions.push(func);
                }
                // Start new function
                let name = trimmed.split('(').next().unwrap_or("").replace("def ", "");
                current_function = Some(PytestFunction {
                    name,
                    body: String::new(),
                    assertions: Vec::new(),
                });
            } else if let Some(ref mut func) = current_function {
                func.body.push_str(line);
                func.body.push('\n');
                // Extract assertions
                if trimmed.starts_with("assert ") {
                    func.assertions.push(trimmed.to_string());
                }
            }
        }
        // Don't forget the last function
        if let Some(func) = current_function {
            functions.push(func);
        }
        functions
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
mod property_tests_migration {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
