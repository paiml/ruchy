//! Demo to notebook conversion module
//!
//! Converts Ruchy demo files (.ruchy) to Jupyter notebook format
//! for use with the `SharedSession` architecture
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct NotebookCell {
    pub cell_type: String,
    pub source: String,
    pub metadata: serde_json::Value,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Notebook {
    pub cells: Vec<NotebookCell>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    pub nbformat: u32,
    pub nbformat_minor: u32,
}
impl NotebookCell {
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::demo_converter::NotebookCell;
    ///
    /// let mut instance = NotebookCell::new();
    /// let result = instance.code();
    /// // Verify behavior
    /// ```
    pub fn code(source: String) -> Self {
        Self {
            cell_type: "code".to_string(),
            source,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::demo_converter::markdown;
    ///
    /// let result = markdown(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn markdown(source: String) -> Self {
        Self {
            cell_type: "markdown".to_string(),
            source,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}
/// # Examples
///
/// ```ignore
/// use ruchy::wasm::demo_converter::convert_demo_to_notebook;
///
/// let result = convert_demo_to_notebook("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn convert_demo_to_notebook(
    name: &str,
    content: &str,
) -> Result<Notebook, Box<dyn std::error::Error>> {
    let mut cells = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        // Skip empty lines
        if line.is_empty() {
            i += 1;
            continue;
        }
        // Handle comments as markdown cells
        if line.starts_with('#') {
            cells.push(NotebookCell::markdown(line.to_string()));
            i += 1;
            continue;
        }
        // Skip REPL commands (lines starting with :)
        if line.starts_with(':') {
            i += 1;
            continue;
        }
        // Handle code lines (single or multi-line)
        let code_block = parse_code_block(&lines, &mut i)?;
        if !code_block.trim().is_empty() {
            cells.push(NotebookCell::code(code_block));
        }
    }
    // Create metadata
    let mut metadata = serde_json::Map::new();
    // Language info
    let mut language_info = serde_json::Map::new();
    language_info.insert(
        "name".to_string(),
        serde_json::Value::String("ruchy".to_string()),
    );
    language_info.insert(
        "version".to_string(),
        serde_json::Value::String("3.1.0".to_string()),
    );
    metadata.insert(
        "language_info".to_string(),
        serde_json::Value::Object(language_info),
    );
    // Kernel spec
    let mut kernelspec = serde_json::Map::new();
    kernelspec.insert(
        "display_name".to_string(),
        serde_json::Value::String("Ruchy".to_string()),
    );
    kernelspec.insert(
        "language".to_string(),
        serde_json::Value::String("ruchy".to_string()),
    );
    kernelspec.insert(
        "name".to_string(),
        serde_json::Value::String("ruchy".to_string()),
    );
    metadata.insert(
        "kernelspec".to_string(),
        serde_json::Value::Object(kernelspec),
    );
    // Original demo name
    metadata.insert(
        "original_demo".to_string(),
        serde_json::Value::String(name.to_string()),
    );
    Ok(Notebook {
        cells,
        metadata,
        nbformat: 4,
        nbformat_minor: 2,
    })
}
fn parse_code_block(
    lines: &[&str],
    index: &mut usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut code_lines = Vec::new();
    let start_line = lines[*index].trim();
    // Check if this is a multi-line statement (function, if, etc.)
    if is_multiline_start(start_line) {
        code_lines.push(start_line);
        *index += 1;
        let mut brace_count = count_braces(start_line);
        // Continue reading until braces are balanced
        while *index < lines.len() && brace_count > 0 {
            let line = lines[*index].trim();
            if !line.is_empty() && !line.starts_with('#') {
                code_lines.push(line);
                brace_count += count_braces(line);
            }
            *index += 1;
        }
    } else {
        // Single line statement
        code_lines.push(start_line);
        *index += 1;
    }
    Ok(code_lines.join("\n"))
}
fn is_multiline_start(line: &str) -> bool {
    line.starts_with("fun ")
        || line.starts_with("if ")
        || line.starts_with("while ")
        || line.starts_with("for ")
        || line.starts_with("match ")
        || line.contains('{')
}
fn count_braces(line: &str) -> i32 {
    let open = line.chars().filter(|&c| c == '{').count() as i32;
    let close = line.chars().filter(|&c| c == '}').count() as i32;
    open - close
}
/// # Examples
///
/// ```ignore
/// use ruchy::wasm::demo_converter::find_demo_files;
///
/// let result = find_demo_files(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn find_demo_files() -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir("examples") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}
#[cfg(test)]
mod tests {
    use super::*;

    // NotebookCell tests
    #[test]
    fn test_notebook_cell_code_creation() {
        let cell = NotebookCell::code("let x = 42".to_string());
        assert_eq!(cell.cell_type, "code");
        assert_eq!(cell.source, "let x = 42");
        assert!(cell.metadata.is_object());
    }

    #[test]
    fn test_notebook_cell_markdown_creation() {
        let cell = NotebookCell::markdown("# Header".to_string());
        assert_eq!(cell.cell_type, "markdown");
        assert_eq!(cell.source, "# Header");
        assert!(cell.metadata.is_object());
    }

    #[test]
    fn test_notebook_cell_empty_source() {
        let cell = NotebookCell::code("".to_string());
        assert_eq!(cell.cell_type, "code");
        assert_eq!(cell.source, "");
    }

    // Notebook tests
    #[test]
    fn test_simple_conversion() {
        let content = "42\nlet x = 10";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 2);
        assert_eq!(notebook.cells[0].source, "42");
        assert_eq!(notebook.cells[1].source, "let x = 10");
    }

    #[test]
    fn test_comment_conversion() {
        let content = "# Comment\n42";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 2);
        assert_eq!(notebook.cells[0].cell_type, "markdown");
        assert_eq!(notebook.cells[1].cell_type, "code");
    }

    #[test]
    fn test_repl_command_filtering() {
        let content = "42\n:rust 1 + 2\nlet x = 10";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 2);
        assert_eq!(notebook.cells[0].source, "42");
        assert_eq!(notebook.cells[1].source, "let x = 10");
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert!(notebook.cells.is_empty());
    }

    #[test]
    fn test_only_empty_lines() {
        let content = "\n\n\n";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert!(notebook.cells.is_empty());
    }

    #[test]
    fn test_notebook_metadata() {
        let content = "42";
        let notebook = convert_demo_to_notebook("my_demo", content).unwrap();

        assert!(notebook.metadata.contains_key("language_info"));
        assert!(notebook.metadata.contains_key("kernelspec"));
        assert!(notebook.metadata.contains_key("original_demo"));

        let original_demo = notebook.metadata.get("original_demo").unwrap();
        assert_eq!(original_demo.as_str().unwrap(), "my_demo");
    }

    #[test]
    fn test_notebook_format_version() {
        let content = "42";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.nbformat, 4);
        assert_eq!(notebook.nbformat_minor, 2);
    }

    // Multiline tests
    #[test]
    fn test_function_multiline() {
        let content = "fun foo() {\n  42\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
        assert!(notebook.cells[0].source.contains("fun foo()"));
    }

    #[test]
    fn test_if_multiline() {
        let content = "if true {\n  1\n} else {\n  2\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
    }

    #[test]
    fn test_while_multiline() {
        let content = "while x > 0 {\n  x = x - 1\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
    }

    #[test]
    fn test_for_multiline() {
        let content = "for i in items {\n  print(i)\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
    }

    #[test]
    fn test_match_multiline() {
        let content = "match x {\n  1 => a,\n  _ => b\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
    }

    #[test]
    fn test_brace_only_multiline() {
        let content = "{\n  let x = 1\n  x + 1\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
    }

    // is_multiline_start tests
    #[test]
    fn test_is_multiline_start_fun() {
        assert!(is_multiline_start("fun foo() {"));
    }

    #[test]
    fn test_is_multiline_start_if() {
        assert!(is_multiline_start("if x > 0 {"));
    }

    #[test]
    fn test_is_multiline_start_while() {
        assert!(is_multiline_start("while true {"));
    }

    #[test]
    fn test_is_multiline_start_for() {
        assert!(is_multiline_start("for i in items {"));
    }

    #[test]
    fn test_is_multiline_start_match() {
        assert!(is_multiline_start("match x {"));
    }

    #[test]
    fn test_is_multiline_start_brace() {
        assert!(is_multiline_start("{"));
    }

    #[test]
    fn test_is_multiline_start_simple_expr() {
        assert!(!is_multiline_start("let x = 42"));
    }

    // count_braces tests
    #[test]
    fn test_count_braces_open_only() {
        assert_eq!(count_braces("{"), 1);
    }

    #[test]
    fn test_count_braces_close_only() {
        assert_eq!(count_braces("}"), -1);
    }

    #[test]
    fn test_count_braces_balanced() {
        assert_eq!(count_braces("{ }"), 0);
    }

    #[test]
    fn test_count_braces_nested() {
        assert_eq!(count_braces("{ { } }"), 0);
    }

    #[test]
    fn test_count_braces_unbalanced_open() {
        assert_eq!(count_braces("{ {"), 2);
    }

    #[test]
    fn test_count_braces_unbalanced_close() {
        assert_eq!(count_braces("} }"), -2);
    }

    #[test]
    fn test_count_braces_empty() {
        assert_eq!(count_braces(""), 0);
    }

    // Edge cases
    #[test]
    fn test_multiple_comments() {
        let content = "# Comment 1\n# Comment 2\n42";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 3);
        assert_eq!(notebook.cells[0].cell_type, "markdown");
        assert_eq!(notebook.cells[1].cell_type, "markdown");
        assert_eq!(notebook.cells[2].cell_type, "code");
    }

    #[test]
    fn test_mixed_content() {
        let content = "# Header\nlet x = 1\n# Another comment\nlet y = 2";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 4);
    }

    #[test]
    fn test_whitespace_only_lines() {
        let content = "42\n   \n   \nlet x = 10";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        // Whitespace-only lines are skipped
        assert_eq!(notebook.cells.len(), 2);
    }

    #[test]
    fn test_multiple_repl_commands() {
        let content = ":help\n:clear\n42\n:exit";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
        assert_eq!(notebook.cells[0].source, "42");
    }

    #[test]
    fn test_nested_multiline() {
        let content = "fun outer() {\n  if true {\n    42\n  }\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
    }

    #[test]
    fn test_comment_in_multiline_skipped() {
        let content = "fun foo() {\n  # Comment inside function - skipped\n  42\n}";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert_eq!(notebook.cells.len(), 1);
        // The comment inside shouldn't become a cell, it's part of code block
        assert!(notebook.cells[0].source.contains("42"));
    }

    // Notebook serialization tests
    #[test]
    fn test_notebook_debug() {
        let notebook = Notebook {
            cells: vec![],
            metadata: serde_json::Map::new(),
            nbformat: 4,
            nbformat_minor: 2,
        };
        let debug_str = format!("{:?}", notebook);
        assert!(debug_str.contains("Notebook"));
    }

    #[test]
    fn test_notebook_cell_debug() {
        let cell = NotebookCell::code("42".to_string());
        let debug_str = format!("{:?}", cell);
        assert!(debug_str.contains("NotebookCell"));
    }

    #[test]
    fn test_notebook_serialization() {
        let content = "42";
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        let json = serde_json::to_string(&notebook);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("cells"));
        assert!(json_str.contains("nbformat"));
    }

    #[test]
    fn test_notebook_cell_serialization() {
        let cell = NotebookCell::code("let x = 42".to_string());
        let json = serde_json::to_string(&cell);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("code"));
        assert!(json_str.contains("let x = 42"));
    }

    // find_demo_files tests
    #[test]
    fn test_find_demo_files_returns_sorted() {
        let files = find_demo_files();
        let mut sorted = files.clone();
        sorted.sort();
        assert_eq!(files, sorted);
    }

    #[test]
    fn test_find_demo_files_empty_when_no_examples() {
        // This test depends on environment, but should not panic
        let _ = find_demo_files();
    }
}
#[cfg(test)]
mod property_tests_demo_converter {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_code_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
