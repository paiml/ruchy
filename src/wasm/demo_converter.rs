//! Demo to notebook conversion module
//! 
//! Converts Ruchy demo files (.ruchy) to Jupyter notebook format
//! for use with the SharedSession architecture

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
    pub fn code(source: String) -> Self {
        Self {
            cell_type: "code".to_string(),
            source,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    pub fn markdown(source: String) -> Self {
        Self {
            cell_type: "markdown".to_string(),
            source,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

pub fn convert_demo_to_notebook(name: &str, content: &str) -> Result<Notebook, Box<dyn std::error::Error>> {
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
    language_info.insert("name".to_string(), serde_json::Value::String("ruchy".to_string()));
    language_info.insert("version".to_string(), serde_json::Value::String("3.1.0".to_string()));
    metadata.insert("language_info".to_string(), serde_json::Value::Object(language_info));
    
    // Kernel spec
    let mut kernelspec = serde_json::Map::new();
    kernelspec.insert("display_name".to_string(), serde_json::Value::String("Ruchy".to_string()));
    kernelspec.insert("language".to_string(), serde_json::Value::String("ruchy".to_string()));
    kernelspec.insert("name".to_string(), serde_json::Value::String("ruchy".to_string()));
    metadata.insert("kernelspec".to_string(), serde_json::Value::Object(kernelspec));
    
    // Original demo name
    metadata.insert("original_demo".to_string(), serde_json::Value::String(name.to_string()));
    
    Ok(Notebook {
        cells,
        metadata,
        nbformat: 4,
        nbformat_minor: 2,
    })
}

fn parse_code_block(lines: &[&str], index: &mut usize) -> Result<String, Box<dyn std::error::Error>> {
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
    line.starts_with("fun ") ||
    line.starts_with("if ") ||
    line.starts_with("while ") ||
    line.starts_with("for ") ||
    line.starts_with("match ") ||
    line.contains('{')
}

fn count_braces(line: &str) -> i32 {
    let open = line.chars().filter(|&c| c == '{').count() as i32;
    let close = line.chars().filter(|&c| c == '}').count() as i32;
    open - close
}

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
}