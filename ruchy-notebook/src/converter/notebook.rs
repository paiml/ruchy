use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::path::Path;
use std::fs;
use super::parser::{DemoCell, CellMetadata};

/// Notebook format for serialization
#[derive(Debug, Clone, Copy)]
pub enum NotebookFormat {
    /// Jupyter notebook format (.ipynb)
    Jupyter,
    /// Ruchy native format (.ruchy-nb)
    RuchyNative,
}

/// Jupyter notebook structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterNotebook {
    pub cells: Vec<JupyterCell>,
    pub metadata: JupyterMetadata,
    pub nbformat: u32,
    pub nbformat_minor: u32,
}

/// Jupyter cell structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cell_type")]
pub enum JupyterCell {
    #[serde(rename = "markdown")]
    Markdown {
        source: Vec<String>,
        metadata: JsonValue,
    },
    #[serde(rename = "code")]
    Code {
        source: Vec<String>,
        outputs: Vec<JsonValue>,
        execution_count: Option<u32>,
        metadata: JsonValue,
    },
}

/// Jupyter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterMetadata {
    pub kernelspec: KernelSpec,
    pub language_info: LanguageInfo,
}

/// Kernel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSpec {
    pub display_name: String,
    pub language: String,
    pub name: String,
}

/// Language information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub version: String,
    pub file_extension: String,
    pub mimetype: String,
}

/// Converter for notebook formats
pub struct NotebookConverter;

impl NotebookConverter {
    /// Convert demo cells to Jupyter notebook
    pub fn to_jupyter(cells: &[DemoCell]) -> JupyterNotebook {
        let jupyter_cells: Vec<JupyterCell> = cells.iter()
            .filter_map(|cell| Self::convert_cell_to_jupyter(cell))
            .collect();
        
        JupyterNotebook {
            cells: jupyter_cells,
            metadata: JupyterMetadata {
                kernelspec: KernelSpec {
                    display_name: "Ruchy".to_string(),
                    language: "ruchy".to_string(),
                    name: "ruchy".to_string(),
                },
                language_info: LanguageInfo {
                    name: "ruchy".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    file_extension: ".ruchy".to_string(),
                    mimetype: "text/x-ruchy".to_string(),
                },
            },
            nbformat: 4,
            nbformat_minor: 5,
        }
    }
    
    /// Convert a single cell to Jupyter format
    fn convert_cell_to_jupyter(cell: &DemoCell) -> Option<JupyterCell> {
        match cell {
            DemoCell::Markdown { content, metadata } => {
                Some(JupyterCell::Markdown {
                    source: content.lines().map(|l| format!("{}\n", l)).collect(),
                    metadata: Self::metadata_to_json(metadata),
                })
            }
            DemoCell::Code { source, metadata } => {
                Some(JupyterCell::Code {
                    source: source.lines().map(|l| format!("{}\n", l)).collect(),
                    outputs: Vec::new(),
                    execution_count: None,
                    metadata: Self::metadata_to_json(metadata),
                })
            }
            DemoCell::Section { title, level } => {
                let header = "#".repeat(*level);
                let content = format!("{} {}", header, title);
                Some(JupyterCell::Markdown {
                    source: vec![format!("{}\n", content)],
                    metadata: json!({}),
                })
            }
        }
    }
    
    /// Convert metadata to JSON
    fn metadata_to_json(metadata: &CellMetadata) -> JsonValue {
        json!({
            "collapsed": metadata.collapsed,
            "deletable": metadata.deletable,
            "editable": metadata.editable,
            "tags": metadata.tags,
        })
    }
    
    /// Save notebook to file
    pub fn save_notebook(notebook: &JupyterNotebook, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(notebook)?;
        fs::write(path, json)
            .with_context(|| format!("Failed to write notebook to {}", path.display()))?;
        Ok(())
    }
    
    /// Load notebook from file
    pub fn load_notebook(path: &Path) -> Result<JupyterNotebook> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read notebook from {}", path.display()))?;
        let notebook = serde_json::from_str(&content)?;
        Ok(notebook)
    }
    
    /// Convert Jupyter notebook back to Ruchy script
    pub fn from_jupyter(notebook: &JupyterNotebook) -> String {
        let mut script = String::new();
        
        for cell in &notebook.cells {
            match cell {
                JupyterCell::Markdown { source, .. } => {
                    for line in source {
                        let trimmed = line.trim_end();
                        if !trimmed.is_empty() {
                            script.push_str("// ");
                            script.push_str(trimmed);
                        }
                        script.push('\n');
                    }
                    script.push('\n');
                }
                JupyterCell::Code { source, .. } => {
                    for line in source {
                        script.push_str(line.trim_end());
                        script.push('\n');
                    }
                    script.push('\n');
                }
            }
        }
        
        script
    }
    
    /// Round-trip conversion test helper
    pub fn round_trip(cells: &[DemoCell]) -> Result<Vec<DemoCell>> {
        // Convert to Jupyter
        let notebook = Self::to_jupyter(cells);
        
        // Convert back to script
        let script = Self::from_jupyter(&notebook);
        
        // Parse script back to cells
        let mut parser = super::parser::DemoParser::new();
        parser.parse_content(&script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser::DemoParser;
    
    #[test]
    fn test_jupyter_conversion() {
        let cells = vec![
            DemoCell::Section {
                title: "Introduction".to_string(),
                level: 1,
            },
            DemoCell::Markdown {
                content: "This is a test notebook".to_string(),
                metadata: CellMetadata::default(),
            },
            DemoCell::Code {
                source: "let x = 42\nprintln(x)".to_string(),
                metadata: CellMetadata::default(),
            },
        ];
        
        let notebook = NotebookConverter::to_jupyter(&cells);
        
        assert_eq!(notebook.cells.len(), 3);
        assert_eq!(notebook.nbformat, 4);
        assert_eq!(notebook.metadata.kernelspec.name, "ruchy");
    }
    
    #[test]
    fn test_round_trip_conversion() {
        let original = vec![
            DemoCell::Code {
                source: "let x = 42".to_string(),
                metadata: CellMetadata::default(),
            },
            DemoCell::Markdown {
                content: "Test comment".to_string(),
                metadata: CellMetadata::default(),
            },
        ];
        
        let notebook = NotebookConverter::to_jupyter(&original);
        let script = NotebookConverter::from_jupyter(&notebook);
        
        assert!(script.contains("let x = 42"));
        assert!(script.contains("// Test comment"));
    }
    
    #[test]
    fn test_save_load_notebook() {
        use tempfile::NamedTempFile;
        
        let cells = vec![
            DemoCell::Code {
                source: "println(\"Hello\")".to_string(),
                metadata: CellMetadata::default(),
            },
        ];
        
        let notebook = NotebookConverter::to_jupyter(&cells);
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        NotebookConverter::save_notebook(&notebook, path).unwrap();
        let loaded = NotebookConverter::load_notebook(path).unwrap();
        
        assert_eq!(loaded.cells.len(), notebook.cells.len());
    }
}