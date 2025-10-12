#!/usr/bin/env rust-script
//! MD Book → Notebook Converter
//!
//! Converts MD Book chapters to .rnb (Ruchy Notebook) format.
//!
//! Usage: rust-script md_to_notebook.rs <input.md> <output.rnb>
//!
//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Type of notebook cell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CellType {
    /// Executable Ruchy code
    Code,
    /// Markdown documentation
    Markdown,
}

/// A single cell in a notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cell {
    /// Type of cell (code or markdown)
    cell_type: CellType,
    /// Source code or markdown content
    source: String,
    /// Output from execution (for code cells only)
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    /// Execution count (for code cells only)
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_count: Option<u32>,
}

/// Metadata for a notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotebookMetadata {
    /// Language name
    language: String,
    /// Notebook format version
    version: String,
    /// Kernel name
    kernel: String,
}

impl Default for NotebookMetadata {
    fn default() -> Self {
        Self {
            language: "ruchy".to_string(),
            version: "1.0.0".to_string(),
            kernel: "ruchy".to_string(),
        }
    }
}

/// A Ruchy notebook containing multiple cells
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Notebook {
    /// List of cells in execution order
    cells: Vec<Cell>,
    /// Notebook metadata
    metadata: NotebookMetadata,
}

/// Parse MD file into notebook cells
fn parse_md_to_cells(content: &str) -> Vec<Cell> {
    let mut cells = Vec::new();
    let mut current_markdown = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut code_block_lang = String::new();

    for line in content.lines() {
        if line.starts_with("```ruchy") {
            // Start of code block
            in_code_block = true;
            code_block_lang = "ruchy".to_string();

            // Save accumulated markdown as markdown cell
            if !current_markdown.trim().is_empty() {
                cells.push(Cell {
                    cell_type: CellType::Markdown,
                    source: current_markdown.trim().to_string(),
                    output: None,
                    execution_count: None,
                });
                current_markdown.clear();
            }
        } else if line.starts_with("```") && in_code_block {
            // End of code block
            in_code_block = false;

            // Save code block as code cell
            if !code_block_content.trim().is_empty() {
                cells.push(Cell {
                    cell_type: CellType::Code,
                    source: code_block_content.trim().to_string(),
                    output: None,
                    execution_count: None,
                });
            }

            code_block_content.clear();
            code_block_lang.clear();
        } else if in_code_block {
            // Inside code block - accumulate code
            code_block_content.push_str(line);
            code_block_content.push('\n');
        } else {
            // Regular markdown - accumulate
            current_markdown.push_str(line);
            current_markdown.push('\n');
        }
    }

    // Save any remaining markdown
    if !current_markdown.trim().is_empty() {
        cells.push(Cell {
            cell_type: CellType::Markdown,
            source: current_markdown.trim().to_string(),
            output: None,
            execution_count: None,
        });
    }

    cells
}

/// Convert MD file to notebook
fn convert_md_to_notebook(md_path: &Path) -> Result<Notebook, String> {
    let content = fs::read_to_string(md_path)
        .map_err(|e| format!("Failed to read {}: {}", md_path.display(), e))?;

    let cells = parse_md_to_cells(&content);

    Ok(Notebook {
        cells,
        metadata: NotebookMetadata::default(),
    })
}

/// Save notebook to .rnb file
fn save_notebook(notebook: &Notebook, output_path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(notebook)
        .map_err(|e| format!("Failed to serialize notebook: {e}"))?;

    fs::write(output_path, json)
        .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.md> <output.rnb>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!(
            "  {} docs/notebook/book/src/01-basic-syntax/01-literals.md output/01-literals.rnb",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    // Convert MD to notebook
    let notebook = match convert_md_to_notebook(input_path) {
        Ok(nb) => nb,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // Save notebook
    if let Err(e) = save_notebook(&notebook, output_path) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    println!(
        "✅ Converted {} to {}",
        input_path.display(),
        output_path.display()
    );
    println!(
        "   📊 {} cells ({} markdown, {} code)",
        notebook.cells.len(),
        notebook
            .cells
            .iter()
            .filter(|c| c.cell_type == CellType::Markdown)
            .count(),
        notebook
            .cells
            .iter()
            .filter(|c| c.cell_type == CellType::Code)
            .count()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_md() {
        let md = r#"# Hello

Some text

```ruchy
42
```

More text
"#;

        let cells = parse_md_to_cells(md);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].cell_type, CellType::Markdown);
        assert!(cells[0].source.contains("# Hello"));
        assert_eq!(cells[1].cell_type, CellType::Code);
        assert_eq!(cells[1].source.trim(), "42");
        assert_eq!(cells[2].cell_type, CellType::Markdown);
        assert!(cells[2].source.contains("More text"));
    }

    #[test]
    fn test_parse_multiple_code_blocks() {
        let md = r#"# Test

```ruchy
1 + 1
```

Middle section

```ruchy
2 + 2
```
"#;

        let cells = parse_md_to_cells(md);
        assert_eq!(cells.len(), 4); // MD, Code, MD, Code
        assert_eq!(cells[0].cell_type, CellType::Markdown);
        assert_eq!(cells[1].cell_type, CellType::Code);
        assert_eq!(cells[1].source.trim(), "1 + 1");
        assert_eq!(cells[2].cell_type, CellType::Markdown);
        assert_eq!(cells[3].cell_type, CellType::Code);
        assert_eq!(cells[3].source.trim(), "2 + 2");
    }

    #[test]
    fn test_empty_code_blocks_filtered() {
        let md = r#"# Test

```ruchy
```

Text after
"#;

        let cells = parse_md_to_cells(md);
        // Empty code block should be skipped
        assert!(cells.iter().all(|c| c.cell_type == CellType::Markdown));
    }
}
