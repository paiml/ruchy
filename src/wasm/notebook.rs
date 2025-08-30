//! WebAssembly Notebook support for Ruchy
//!
//! Provides Jupyter-style notebook functionality in the browser.

use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
type JsValue = String;

#[cfg(not(target_arch = "wasm32"))]
use serde::{Serialize, Deserialize};

// ============================================================================
// Notebook Types
// ============================================================================

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct NotebookCell {
    pub id: String,
    pub cell_type: CellType,
    pub source: String,
    pub outputs: Vec<CellOutput>,
    pub execution_count: Option<usize>,
    pub metadata: CellMetadata,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub enum CellType {
    Code,
    Markdown,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub enum CellOutput {
    Text(String),
    Html(String),
    Image { data: String, mime_type: String },
    DataFrame(DataFrameOutput),
    Error { message: String, traceback: Vec<String> },
}

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct DataFrameOutput {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub shape: (usize, usize),
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct CellMetadata {
    pub collapsed: bool,
    pub execution_time_ms: Option<f64>,
    pub tags: Vec<String>,
}

// ============================================================================
// Notebook Document
// ============================================================================

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct Notebook {
    pub version: String,
    pub metadata: NotebookMetadata,
    pub cells: Vec<NotebookCell>,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct NotebookMetadata {
    pub kernel: String,
    pub language: String,
    pub created: String,
    pub modified: String,
    pub ruchy_version: String,
}

// ============================================================================
// Notebook Runtime
// ============================================================================

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct NotebookRuntime {
    notebook: Notebook,
    repl: crate::wasm::repl::WasmRepl,
    execution_count: usize,
    variables: HashMap<String, String>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl NotebookRuntime {
    /// Create a new notebook runtime
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Result<NotebookRuntime, JsValue> {
        #[cfg(target_arch = "wasm32")]
        let repl = crate::wasm::repl::WasmRepl::new()?;
        #[cfg(not(target_arch = "wasm32"))]
        let repl = crate::wasm::repl::WasmRepl::new().map_err(|_| "Error creating REPL".to_string())?;
        Ok(NotebookRuntime {
            notebook: Notebook {
                version: "1.0.0".to_string(),
                metadata: NotebookMetadata {
                    kernel: "wasm".to_string(),
                    language: "ruchy".to_string(),
                    created: current_timestamp(),
                    modified: current_timestamp(),
                    ruchy_version: env!("CARGO_PKG_VERSION").to_string(),
                },
                cells: Vec::new(),
            },
            repl,
            execution_count: 0,
            variables: HashMap::new(),
        })
    }
    
    /// Add a new cell to the notebook
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_cell(&mut self, cell_type: &str, source: &str) -> String {
        let id = generate_cell_id();
        let cell = NotebookCell {
            id: id.clone(),
            cell_type: match cell_type {
                "markdown" => CellType::Markdown,
                _ => CellType::Code,
            },
            source: source.to_string(),
            outputs: Vec::new(),
            execution_count: None,
            metadata: CellMetadata::default(),
        };
        
        self.notebook.cells.push(cell);
        self.notebook.metadata.modified = current_timestamp();
        
        id
    }
    
    /// Execute a cell by ID
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn execute_cell(&mut self, cell_id: &str) -> Result<String, JsValue> {
        let cell = self.notebook.cells.iter_mut()
            .find(|c| c.id == cell_id)
            .ok_or_else(|| {
                #[cfg(target_arch = "wasm32")]
                return JsValue::from_str("Cell not found");
                #[cfg(not(target_arch = "wasm32"))]
                return "Cell not found".to_string();
            })?;
        
        match cell.cell_type {
            CellType::Code => {
                let start = get_timestamp();
                
                // Execute the code
                let result = self.repl.eval(&cell.source).map_err(|e| {
                    #[cfg(target_arch = "wasm32")]
                    return e;
                    #[cfg(not(target_arch = "wasm32"))]
                    return format!("Eval error: {e:?}");
                })?;
                
                // Update execution count
                self.execution_count += 1;
                cell.execution_count = Some(self.execution_count);
                
                // Parse result and create output
                let output = if result.contains("error") {
                    CellOutput::Error {
                        message: result,
                        traceback: vec![],
                    }
                } else {
                    CellOutput::Text(result)
                };
                
                cell.outputs = vec![output];
                cell.metadata.execution_time_ms = Some(get_timestamp() - start);
                
                Ok(serde_json::to_string(&cell).unwrap_or_else(|_| "Error".to_string()))
            }
            CellType::Markdown => {
                // Markdown cells don't execute
                Ok(String::new())
            }
        }
    }
    
    /// Get all cells as JSON
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_cells(&self) -> String {
        serde_json::to_string(&self.notebook.cells)
            .unwrap_or_else(|_| "[]".to_string())
    }
    
    /// Save notebook to JSON
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.notebook)
            .unwrap_or_else(|_| "{}".to_string())
    }
    
    /// Load notebook from JSON
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn from_json(&mut self, json: &str) -> Result<(), JsValue> {
        let notebook: Notebook = serde_json::from_str(json)
            .map_err(|e| {
                #[cfg(target_arch = "wasm32")]
                return JsValue::from_str(&format!("Parse error: {}", e));
                #[cfg(not(target_arch = "wasm32"))]
                return format!("Parse error: {e}");
            })?;
        self.notebook = notebook;
        Ok(())
    }
}

impl Default for NotebookRuntime {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_cell_id() -> String {
    format!("cell-{}", get_timestamp().abs() as u64)
}

fn current_timestamp() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::new_0().to_iso_string().as_string().unwrap()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().to_rfc3339()
    }
}

fn get_timestamp() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }
}

// ============================================================================
// WASM/Native Feature Parity
// ============================================================================

/// Feature availability in WASM vs Native
#[derive(Debug, Clone)]
pub struct FeatureParity {
    pub feature: String,
    pub native_support: bool,
    pub wasm_support: bool,
    pub notes: String,
}

impl FeatureParity {
    pub fn check_all() -> Vec<FeatureParity> {
        vec![
            FeatureParity {
                feature: "Basic Evaluation".to_string(),
                native_support: true,
                wasm_support: true,
                notes: "Full support".to_string(),
            },
            FeatureParity {
                feature: "File I/O".to_string(),
                native_support: true,
                wasm_support: false,
                notes: "WASM uses OPFS or IndexedDB".to_string(),
            },
            FeatureParity {
                feature: "Threading".to_string(),
                native_support: true,
                wasm_support: false,
                notes: "WASM uses Web Workers".to_string(),
            },
            FeatureParity {
                feature: "Networking".to_string(),
                native_support: true,
                wasm_support: false,
                notes: "WASM limited to Fetch API".to_string(),
            },
            FeatureParity {
                feature: "DataFrames".to_string(),
                native_support: true,
                wasm_support: true,
                notes: "Limited size in WASM".to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notebook_creation() {
        let runtime = NotebookRuntime::new();
        assert!(runtime.is_ok());
    }
    
    #[test]
    fn test_add_cell() {
        let mut runtime = NotebookRuntime::new().unwrap();
        let id = runtime.add_cell("code", "let x = 42");
        assert!(id.starts_with("cell-"));
        assert_eq!(runtime.notebook.cells.len(), 1);
    }
    
    #[test]
    fn test_feature_parity() {
        let features = FeatureParity::check_all();
        assert!(!features.is_empty());
        
        // Check that basic evaluation is supported in both
        let basic = features.iter()
            .find(|f| f.feature == "Basic Evaluation")
            .unwrap();
        assert!(basic.native_support);
        assert!(basic.wasm_support);
    }
}