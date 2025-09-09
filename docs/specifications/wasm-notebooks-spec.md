# Ruchy WASM Notebooks Specification

## Overview

Transform any Ruchy REPL session into an interactive notebook experience running entirely in the browser. This specification defines how to bridge our existing REPL infrastructure, WASM compilation, and educational demos into a full-featured notebook environment compatible with modern data science workflows.

## Vision Statement

**"Every Ruchy REPL session becomes a shareable, executable notebook"**

- Convert `.ruchy` files from ruchy-repl-demos into interactive notebooks
- Enable real-time collaboration on data science projects
- Provide Jupyter-like experience with Ruchy's performance advantages
- Support educational use cases with built-in assessment capabilities

## Architecture

### Core Components Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser Environment                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐ ┌─────────────────┐│
│  │  Notebook UI    │  │   WASM REPL     │ │  Session Store  ││
│  │  (React/Vue)    │──│  (Core Engine)  │─│  (IndexedDB)    ││
│  └─────────────────┘  └─────────────────┘ └─────────────────┘│
│           │                     │                  │         │
│  ┌─────────────────┐  ┌─────────────────┐ ┌─────────────────┐│
│  │   Cell Manager  │  │  Replay System  │ │   Export Hub    ││
│  │  (Execution)    │──│  (Deterministic)│─│  (Share/Save)   ││
│  └─────────────────┘  └─────────────────┘ └─────────────────┘│
│           │                     │                  │         │
│  ┌─────────────────┐  ┌─────────────────┐ ┌─────────────────┐│
│  │ Visualization   │  │  Data Pipeline  │ │  Collab Engine  ││
│  │   Engine        │──│   (DataFrame)   │─│  (Operational   ││
│  │ (Charts/Plots)  │  │                 │ │  Transform)     ││
│  └─────────────────┘  └─────────────────┘ └─────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Notebook Cell Types

```rust
#[derive(Serialize, Deserialize, Clone)]
pub enum CellType {
    Code {
        source: String,
        language: CellLanguage,
        execution_count: Option<u32>,
        metadata: CellMetadata,
    },
    Markdown {
        source: String,
        rendered: bool,
    },
    Raw {
        source: String,
        format: RawFormat,
    },
    Data {
        source: DataSource,
        preview_rows: Option<usize>,
        schema: Option<DataSchema>,
    },
    Visualization {
        chart_type: ChartType,
        data_ref: String, // Reference to data cell
        config: VisualizationConfig,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CellLanguage {
    Ruchy,
    Rust,     // For compiled output inspection
    SQL,      // For data queries
    Markdown, // For documentation
    JavaScript, // For custom visualizations
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CellOutput {
    output_type: OutputType,
    execution_count: u32,
    data: OutputData,
    metadata: OutputMetadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OutputType {
    ExecuteResult,
    DisplayData,
    Stream,
    Error,
    Interactive, // For widgets/forms
}
```

### Notebook Document Format

```rust
#[derive(Serialize, Deserialize)]
pub struct RuchyNotebook {
    nbformat: u8,           // 4 (Jupyter compatibility)
    nbformat_minor: u8,     // 5
    metadata: NotebookMetadata,
    cells: Vec<NotebookCell>,
    ruchy_metadata: RuchyExtensions,
}

#[derive(Serialize, Deserialize)]
pub struct NotebookMetadata {
    kernelspec: KernelSpec,
    language_info: LanguageInfo,
    ruchy_version: String,
    created: chrono::DateTime<chrono::Utc>,
    modified: chrono::DateTime<chrono::Utc>,
    authors: Vec<Author>,
    title: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RuchyExtensions {
    session_recording: Option<ReplSession>,
    dependencies: Vec<Dependency>,
    data_sources: Vec<DataSource>,
    educational_metadata: Option<EducationalMetadata>,
    performance_profile: Option<PerformanceProfile>,
}

#[derive(Serialize, Deserialize)]
pub struct EducationalMetadata {
    assignment_id: Option<String>,
    course_id: Option<String>,
    learning_objectives: Vec<String>,
    difficulty_level: DifficultyLevel,
    estimated_duration: Duration,
    prerequisites: Vec<String>,
    assessment_criteria: Vec<AssessmentCriterion>,
}
```

## Implementation Phases

### Phase 1: Core Notebook Infrastructure (Weeks 1-3)

#### WASM Notebook Engine

```rust
// crates/ruchy-notebook/src/notebook.rs
use wasm_bindgen::prelude::*;
use crate::cell::{NotebookCell, CellExecution};
use crate::session::SessionManager;

#[wasm_bindgen]
pub struct RuchyNotebook {
    cells: Vec<NotebookCell>,
    session: SessionManager,
    kernel: RuchyKernel,
    metadata: NotebookMetadata,
}

#[wasm_bindgen]
impl RuchyNotebook {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RuchyNotebook, JsValue> {
        console_error_panic_hook::set_once();
        
        Ok(RuchyNotebook {
            cells: Vec::new(),
            session: SessionManager::new()?,
            kernel: RuchyKernel::new()?,
            metadata: NotebookMetadata::default(),
        })
    }
    
    pub fn add_cell(&mut self, cell_type: &str, source: &str) -> Result<String, JsValue> {
        let cell_id = uuid::Uuid::new_v4().to_string();
        let cell = NotebookCell::new(cell_id.clone(), cell_type, source)?;
        self.cells.push(cell);
        Ok(cell_id)
    }
    
    pub async fn execute_cell(&mut self, cell_id: &str) -> Result<JsValue, JsValue> {
        let cell = self.find_cell_mut(cell_id)?;
        let execution = CellExecution::new(cell, &mut self.kernel).await?;
        
        // Record execution in session for replay
        self.session.record_execution(&execution);
        
        // Convert to JS-friendly format
        Ok(serde_wasm_bindgen::to_value(&execution.output)?)
    }
    
    pub fn execute_all_cells(&mut self) -> Result<Vec<JsValue>, JsValue> {
        let mut results = Vec::new();
        
        for cell in &mut self.cells {
            if matches!(cell.cell_type, CellType::Code { .. }) {
                let result = self.execute_cell(&cell.id)?;
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    pub fn save_notebook(&self) -> Result<String, JsValue> {
        let json = serde_json::to_string_pretty(&self)?;
        Ok(json)
    }
    
    pub fn load_notebook(&mut self, json: &str) -> Result<(), JsValue> {
        let notebook: RuchyNotebook = serde_json::from_str(json)?;
        *self = notebook;
        Ok(())
    }
    
    pub fn import_repl_session(&mut self, session_path: &str) -> Result<(), JsValue> {
        // Convert .ruchy demo file to notebook cells
        let demo_content = self.load_demo_file(session_path)?;
        self.parse_demo_to_cells(demo_content)?;
        Ok(())
    }
}
```

#### Cell Execution Engine

```rust
// crates/ruchy-notebook/src/cell.rs
pub struct CellExecution {
    pub cell_id: String,
    pub execution_count: u32,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub output: CellOutput,
    pub status: ExecutionStatus,
}

impl CellExecution {
    pub async fn new(cell: &mut NotebookCell, kernel: &mut RuchyKernel) -> Result<CellExecution, JsValue> {
        let start_time = Instant::now();
        let execution_count = kernel.next_execution_count();
        
        let output = match &cell.cell_type {
            CellType::Code { source, .. } => {
                kernel.execute_code(source).await?
            }
            CellType::Markdown { source, .. } => {
                CellOutput::rendered_markdown(source)
            }
            CellType::Data { source, .. } => {
                kernel.load_data_source(source).await?
            }
            CellType::Visualization { chart_type, data_ref, config } => {
                kernel.create_visualization(chart_type, data_ref, config).await?
            }
        };
        
        Ok(CellExecution {
            cell_id: cell.id.clone(),
            execution_count,
            start_time,
            end_time: Some(Instant::now()),
            output,
            status: ExecutionStatus::Completed,
        })
    }
}
```

### Phase 2: Demo Integration (Weeks 4-5)

#### Demo Import Pipeline

```rust
// crates/ruchy-notebook/src/demo_converter.rs
pub struct DemoConverter {
    parser: RuchyParser,
    cell_splitter: CellSplitter,
    metadata_extractor: MetadataExtractor,
}

impl DemoConverter {
    pub fn convert_demo_to_notebook(&self, demo_path: &Path) -> Result<RuchyNotebook, Error> {
        let content = std::fs::read_to_string(demo_path)?;
        
        // Extract metadata from comments
        let metadata = self.metadata_extractor.extract(&content)?;
        
        // Split content into logical cells
        let cells = self.cell_splitter.split_into_cells(&content)?;
        
        // Create notebook structure
        let mut notebook = RuchyNotebook::new()?;
        notebook.metadata = metadata;
        
        for cell_content in cells {
            let cell = self.create_cell_from_content(cell_content)?;
            notebook.add_cell_object(cell);
        }
        
        Ok(notebook)
    }
    
    fn create_cell_from_content(&self, content: CellContent) -> Result<NotebookCell, Error> {
        match content.content_type {
            ContentType::Comment => NotebookCell::markdown(content.text),
            ContentType::Code => NotebookCell::code(content.text, CellLanguage::Ruchy),
            ContentType::DataDeclaration => NotebookCell::data(content.text),
            ContentType::Visualization => NotebookCell::chart(content.text),
        }
    }
}

pub struct CellSplitter;

impl CellSplitter {
    pub fn split_into_cells(&self, content: &str) -> Result<Vec<CellContent>, Error> {
        let lines: Vec<&str> = content.lines().collect();
        let mut cells = Vec::new();
        let mut current_cell = Vec::new();
        let mut current_type = ContentType::Code;
        
        for line in lines {
            if line.starts_with("//") {
                // Flush current code cell if exists
                if !current_cell.is_empty() && current_type == ContentType::Code {
                    cells.push(CellContent {
                        content_type: current_type,
                        text: current_cell.join("\n"),
                    });
                    current_cell.clear();
                }
                
                // Start markdown cell
                current_type = ContentType::Comment;
                current_cell.push(line.strip_prefix("//").unwrap_or(line).trim());
            } else if line.starts_with("println(\"===") {
                // Section headers become markdown
                if !current_cell.is_empty() {
                    cells.push(CellContent {
                        content_type: current_type,
                        text: current_cell.join("\n"),
                    });
                    current_cell.clear();
                }
                
                current_type = ContentType::Comment;
                let header = line.replace("println(\"", "").replace("\")", "").replace("===", "###");
                current_cell.push(header);
            } else if !line.trim().is_empty() {
                // Regular code
                if current_type == ContentType::Comment {
                    cells.push(CellContent {
                        content_type: current_type,
                        text: current_cell.join("\n"),
                    });
                    current_cell.clear();
                    current_type = ContentType::Code;
                }
                current_cell.push(line);
            }
        }
        
        // Flush remaining cell
        if !current_cell.is_empty() {
            cells.push(CellContent {
                content_type: current_type,
                text: current_cell.join("\n"),
            });
        }
        
        Ok(cells)
    }
}
```

#### Batch Demo Conversion

```bash
#!/bin/bash
# scripts/convert-demos-to-notebooks.sh

set -e

echo "🔄 Converting Ruchy REPL demos to notebooks..."

DEMOS_DIR="../ruchy-repl-demos/demos/repl"
OUTPUT_DIR="./notebooks/demos"

mkdir -p "$OUTPUT_DIR"

# Convert each demo category
for category in $(ls "$DEMOS_DIR"); do
    echo "Converting category: $category"
    mkdir -p "$OUTPUT_DIR/$category"
    
    for demo_file in "$DEMOS_DIR/$category"/*.ruchy; do
        if [[ -f "$demo_file" ]]; then
            basename=$(basename "$demo_file" .ruchy)
            output_file="$OUTPUT_DIR/$category/${basename}.ipynb"
            
            echo "  Converting: $demo_file -> $output_file"
            
            # Use our converter tool
            ruchy convert-to-notebook "$demo_file" --output "$output_file" \
                --format jupyter \
                --include-metadata \
                --split-cells auto
        fi
    done
done

echo "✅ Conversion complete! Generated notebooks in: $OUTPUT_DIR"
echo "📊 Total notebooks created: $(find "$OUTPUT_DIR" -name "*.ipynb" | wc -l)"
```

### Phase 3: Interactive Features (Weeks 6-8)

#### Real-time Visualization

```rust
// crates/ruchy-notebook/src/visualization.rs
use plotters::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VisualizationEngine {
    canvas_context: web_sys::CanvasRenderingContext2d,
    chart_registry: HashMap<String, ChartInstance>,
}

#[wasm_bindgen]
impl VisualizationEngine {
    pub fn create_scatter_plot(&mut self, 
        data_ref: &str, 
        x_col: &str, 
        y_col: &str,
        config: JsValue
    ) -> Result<String, JsValue> {
        let data = self.get_data_from_ref(data_ref)?;
        let plot_config: ScatterPlotConfig = serde_wasm_bindgen::from_value(config)?;
        
        let chart_id = uuid::Uuid::new_v4().to_string();
        
        // Create plotters backend for HTML5 Canvas
        let backend = CanvasBackend::new(&chart_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption(&plot_config.title, ("sans-serif", 50))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                plot_config.x_range.0..plot_config.x_range.1,
                plot_config.y_range.0..plot_config.y_range.1
            )?;

        chart.configure_mesh().draw()?;
        
        // Plot data points
        chart.draw_series(
            data.iter().map(|point| Circle::new((point.x, point.y), 3, BLUE.filled()))
        )?;
        
        root.present()?;
        
        Ok(chart_id)
    }
    
    pub fn create_data_table(&mut self, data_ref: &str, config: JsValue) -> Result<JsValue, JsValue> {
        let data = self.get_data_from_ref(data_ref)?;
        let table_config: TableConfig = serde_wasm_bindgen::from_value(config)?;
        
        let table_html = self.render_data_table(&data, &table_config)?;
        
        Ok(JsValue::from_str(&table_html))
    }
}
```

#### Collaborative Editing

```rust
// crates/ruchy-notebook/src/collaboration.rs
use operational_transform::{Operation, TextOperation};
use web_sys::WebSocket;

#[wasm_bindgen]
pub struct CollaborationEngine {
    websocket: WebSocket,
    document_state: DocumentState,
    pending_operations: Vec<TextOperation>,
    user_id: String,
}

#[wasm_bindgen]
impl CollaborationEngine {
    pub fn new(server_url: &str, notebook_id: &str) -> Result<CollaborationEngine, JsValue> {
        let websocket = WebSocket::new(&format!("{}/notebook/{}", server_url, notebook_id))?;
        
        Ok(CollaborationEngine {
            websocket,
            document_state: DocumentState::new(),
            pending_operations: Vec::new(),
            user_id: uuid::Uuid::new_v4().to_string(),
        })
    }
    
    pub fn apply_local_edit(&mut self, cell_id: &str, edit: JsValue) -> Result<(), JsValue> {
        let operation: TextOperation = serde_wasm_bindgen::from_value(edit)?;
        
        // Apply locally
        self.document_state.apply_operation(&operation)?;
        
        // Send to server
        let message = CollaborationMessage {
            type_: MessageType::Operation,
            user_id: self.user_id.clone(),
            cell_id: cell_id.to_string(),
            operation: operation.clone(),
            timestamp: chrono::Utc::now(),
        };
        
        self.send_message(message)?;
        
        Ok(())
    }
    
    pub fn handle_remote_operation(&mut self, message: JsValue) -> Result<(), JsValue> {
        let msg: CollaborationMessage = serde_wasm_bindgen::from_value(message)?;
        
        if msg.user_id != self.user_id {
            // Transform against pending operations
            let transformed = self.transform_remote_operation(msg.operation)?;
            
            // Apply to document
            self.document_state.apply_operation(&transformed)?;
            
            // Notify UI
            self.emit_document_changed()?;
        }
        
        Ok(())
    }
}
```

### Phase 4: Educational Integration (Weeks 9-10)

#### Assignment Creation

```rust
// crates/ruchy-notebook/src/educational.rs
#[derive(Serialize, Deserialize)]
pub struct NotebookAssignment {
    id: String,
    title: String,
    description: String,
    template_notebook: RuchyNotebook,
    solution_notebook: Option<RuchyNotebook>,
    test_cases: Vec<TestCase>,
    grading_criteria: GradingCriteria,
    deadline: Option<DateTime<Utc>>,
    max_attempts: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct TestCase {
    name: String,
    cell_id: String,
    expected_output: ExpectedOutput,
    points: u32,
    timeout_seconds: u32,
    hidden: bool, // Hidden from students
}

#[wasm_bindgen]
pub struct EducationalNotebook {
    notebook: RuchyNotebook,
    assignment: Option<NotebookAssignment>,
    submission_tracker: SubmissionTracker,
}

#[wasm_bindgen]
impl EducationalNotebook {
    pub fn create_assignment(&mut self, config: JsValue) -> Result<String, JsValue> {
        let assignment_config: AssignmentConfig = serde_wasm_bindgen::from_value(config)?;
        
        let assignment = NotebookAssignment {
            id: uuid::Uuid::new_v4().to_string(),
            title: assignment_config.title,
            description: assignment_config.description,
            template_notebook: self.notebook.clone(),
            solution_notebook: None,
            test_cases: assignment_config.test_cases,
            grading_criteria: assignment_config.grading_criteria,
            deadline: assignment_config.deadline,
            max_attempts: assignment_config.max_attempts,
        };
        
        self.assignment = Some(assignment.clone());
        
        Ok(assignment.id)
    }
    
    pub async fn run_automated_grading(&mut self) -> Result<JsValue, JsValue> {
        let assignment = self.assignment.as_ref().ok_or("No assignment loaded")?;
        
        let mut grading_results = Vec::new();
        
        for test_case in &assignment.test_cases {
            let result = self.execute_test_case(test_case).await?;
            grading_results.push(result);
        }
        
        let total_score = grading_results.iter().map(|r| r.points_awarded).sum::<u32>();
        let max_score = assignment.test_cases.iter().map(|t| t.points).sum::<u32>();
        
        let grade_report = GradeReport {
            assignment_id: assignment.id.clone(),
            total_score,
            max_score,
            percentage: (total_score as f64 / max_score as f64) * 100.0,
            test_results: grading_results,
            feedback: self.generate_feedback(&grading_results),
            timestamp: chrono::Utc::now(),
        };
        
        Ok(serde_wasm_bindgen::to_value(&grade_report)?)
    }
    
    async fn execute_test_case(&mut self, test_case: &TestCase) -> Result<TestResult, JsValue> {
        let cell = self.notebook.find_cell(&test_case.cell_id)?;
        
        // Execute with timeout
        let execution_future = self.notebook.execute_cell(&test_case.cell_id);
        let timeout_future = wasm_timer::Delay::new(Duration::from_secs(test_case.timeout_seconds.into()));
        
        let result = futures::select! {
            exec_result = execution_future.fuse() => exec_result?,
            _ = timeout_future.fuse() => {
                return Ok(TestResult {
                    test_name: test_case.name.clone(),
                    passed: false,
                    points_awarded: 0,
                    error_message: Some("Execution timeout".to_string()),
                    actual_output: None,
                });
            }
        };
        
        // Compare with expected output
        let passed = match &test_case.expected_output {
            ExpectedOutput::Exact(expected) => result.display_text == *expected,
            ExpectedOutput::Pattern(regex) => regex.is_match(&result.display_text),
            ExpectedOutput::Predicate(pred) => pred.evaluate(&result),
        };
        
        Ok(TestResult {
            test_name: test_case.name.clone(),
            passed,
            points_awarded: if passed { test_case.points } else { 0 },
            error_message: None,
            actual_output: Some(result.display_text),
        })
    }
}
```

## Frontend Integration

### React Notebook Component

```typescript
// frontend/src/components/RuchyNotebook.tsx
import React, { useState, useEffect } from 'react';
import { RuchyNotebook as WasmNotebook } from 'ruchy-notebook-wasm';

interface NotebookProps {
  initialNotebook?: string;
  mode: 'edit' | 'view' | 'assignment';
  onSave?: (notebook: string) => void;
  onExecute?: (cellId: string, result: any) => void;
}

export const RuchyNotebook: React.FC<NotebookProps> = ({
  initialNotebook,
  mode,
  onSave,
  onExecute
}) => {
  const [notebook, setNotebook] = useState<WasmNotebook | null>(null);
  const [cells, setCells] = useState<Cell[]>([]);
  const [executing, setExecuting] = useState<Set<string>>(new Set());

  useEffect(() => {
    const initNotebook = async () => {
      try {
        const wasmNotebook = new WasmNotebook();
        
        if (initialNotebook) {
          await wasmNotebook.load_notebook(initialNotebook);
        }
        
        setNotebook(wasmNotebook);
        loadCells(wasmNotebook);
      } catch (error) {
        console.error('Failed to initialize notebook:', error);
      }
    };

    initNotebook();
  }, [initialNotebook]);

  const executeCell = async (cellId: string) => {
    if (!notebook) return;
    
    setExecuting(prev => new Set(prev).add(cellId));
    
    try {
      const result = await notebook.execute_cell(cellId);
      
      // Update cell output
      setCells(prev => prev.map(cell => 
        cell.id === cellId 
          ? { ...cell, output: result, executionCount: (cell.executionCount || 0) + 1 }
          : cell
      ));
      
      onExecute?.(cellId, result);
    } catch (error) {
      console.error(`Failed to execute cell ${cellId}:`, error);
      
      // Show error in cell
      setCells(prev => prev.map(cell => 
        cell.id === cellId 
          ? { ...cell, output: { error: error.toString() } }
          : cell
      ));
    } finally {
      setExecuting(prev => {
        const next = new Set(prev);
        next.delete(cellId);
        return next;
      });
    }
  };

  const addCell = (type: 'code' | 'markdown', index?: number) => {
    if (!notebook) return;
    
    const cellId = notebook.add_cell(type, '');
    const newCell: Cell = {
      id: cellId,
      type,
      source: '',
      output: null,
      executionCount: null,
    };
    
    setCells(prev => {
      const next = [...prev];
      const insertIndex = index !== undefined ? index : next.length;
      next.splice(insertIndex, 0, newCell);
      return next;
    });
  };

  const deleteCell = (cellId: string) => {
    setCells(prev => prev.filter(cell => cell.id !== cellId));
  };

  const saveNotebook = async () => {
    if (!notebook) return;
    
    const notebookJson = notebook.save_notebook();
    onSave?.(notebookJson);
  };

  return (
    <div className="ruchy-notebook">
      <div className="notebook-toolbar">
        <button onClick={() => addCell('code')} disabled={mode === 'view'}>
          + Code
        </button>
        <button onClick={() => addCell('markdown')} disabled={mode === 'view'}>
          + Markdown
        </button>
        <button onClick={saveNotebook} disabled={mode === 'view'}>
          Save
        </button>
        <button onClick={() => notebook?.execute_all_cells()}>
          Run All
        </button>
      </div>
      
      <div className="notebook-cells">
        {cells.map((cell, index) => (
          <NotebookCell
            key={cell.id}
            cell={cell}
            executing={executing.has(cell.id)}
            readOnly={mode === 'view'}
            onExecute={() => executeCell(cell.id)}
            onDelete={() => deleteCell(cell.id)}
            onSourceChange={(source) => updateCellSource(cell.id, source)}
            onAddCell={(type) => addCell(type, index + 1)}
          />
        ))}
      </div>
    </div>
  );
};
```

### Notebook Cell Component

```typescript
// frontend/src/components/NotebookCell.tsx
import React, { useState } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vs } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface NotebookCellProps {
  cell: Cell;
  executing: boolean;
  readOnly: boolean;
  onExecute: () => void;
  onDelete: () => void;
  onSourceChange: (source: string) => void;
  onAddCell: (type: 'code' | 'markdown') => void;
}

export const NotebookCell: React.FC<NotebookCellProps> = ({
  cell,
  executing,
  readOnly,
  onExecute,
  onDelete,
  onSourceChange,
  onAddCell
}) => {
  const [focused, setFocused] = useState(false);
  const [source, setSource] = useState(cell.source);

  const handleSourceChange = (newSource: string) => {
    setSource(newSource);
    onSourceChange(newSource);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      onExecute();
    }
  };

  return (
    <div className={`notebook-cell ${cell.type} ${focused ? 'focused' : ''}`}>
      <div className="cell-sidebar">
        <div className="execution-count">
          {cell.type === 'code' && (
            executing ? '⏳' : cell.executionCount || ' '
          )}
        </div>
        
        {!readOnly && (
          <div className="cell-controls">
            <button onClick={onExecute} disabled={executing || cell.type !== 'code'}>
              ▶
            </button>
            <button onClick={onDelete}>🗑</button>
            <div className="add-cell-buttons">
              <button onClick={() => onAddCell('code')} title="Add code cell">+📝</button>
              <button onClick={() => onAddCell('markdown')} title="Add markdown cell">+📄</button>
            </div>
          </div>
        )}
      </div>
      
      <div className="cell-content">
        <div className="cell-input">
          {cell.type === 'code' ? (
            <CodeEditor
              value={source}
              onChange={handleSourceChange}
              onFocus={() => setFocused(true)}
              onBlur={() => setFocused(false)}
              onKeyDown={handleKeyDown}
              readOnly={readOnly}
              language="ruchy"
            />
          ) : (
            <MarkdownEditor
              value={source}
              onChange={handleSourceChange}
              onFocus={() => setFocused(true)}
              onBlur={() => setFocused(false)}
              readOnly={readOnly}
            />
          )}
        </div>
        
        {cell.output && (
          <div className="cell-output">
            <CellOutput output={cell.output} />
          </div>
        )}
      </div>
    </div>
  );
};
```

## Demo Conversion Examples

### Converting Iris Analysis Demo

**Original Demo Structure** (`iris_analysis_demo.ruchy`):
```ruchy
// REPL Demo: Iris Dataset Analysis - The Hello World of Data Science
// Dataset: Classic Fisher's Iris dataset (150 flowers, 4 measurements)
// Skills: Basic statistics, grouping, filtering, data exploration

println("=== Iris Dataset Analysis - Data Science Fundamentals ===");

// Simulate the famous Iris dataset (normally loaded from CSV)
// In production: let iris = load_csv("datasets/iris.csv")
println("\n1. Loading the Iris Dataset:");
let iris = [
    // Setosa samples (first 5 of 50)
    {species: "setosa", sepal_length: 5.1, sepal_width: 3.5, petal_length: 1.4, petal_width: 0.2},
    // ... more data
];

println(f"Dataset loaded: {iris.len()} samples");

// 2. Dataset Overview - Data Science Step 1
println("\n2. Dataset Overview:");
let species_list = iris.map(|row| row.species);
```

**Converted Notebook Structure**:

**Cell 1** (Markdown):
```markdown
# Iris Dataset Analysis - The Hello World of Data Science

**Dataset**: Classic Fisher's Iris dataset (150 flowers, 4 measurements)  
**Skills**: Basic statistics, grouping, filtering, data exploration

This notebook demonstrates fundamental data science techniques using the famous Iris dataset.
```

**Cell 2** (Code):
```ruchy
// Load the Iris dataset (normally from CSV: load_csv("datasets/iris.csv"))
let iris = [
    // Setosa samples (first 5 of 50)
    {species: "setosa", sepal_length: 5.1, sepal_width: 3.5, petal_length: 1.4, petal_width: 0.2},
    {species: "setosa", sepal_length: 4.9, sepal_width: 3.0, petal_length: 1.4, petal_width: 0.2},
    // ... complete dataset
];

println(f"Dataset loaded: {iris.len()} samples");
println("Features: sepal_length, sepal_width, petal_length, petal_width, species");
```

**Cell 3** (Markdown):
```markdown
## Dataset Overview

Let's examine the structure and basic properties of our dataset.
```

**Cell 4** (Code):
```ruchy
let species_list = iris.map(|row| row.species);
let unique_species = ["setosa", "versicolor", "virginica"]; // Would use unique() function
println(f"Species in dataset: {unique_species}");
```

**Cell 5** (Visualization):
```ruchy
// Create scatter plot of sepal dimensions
create_scatter_plot(iris, "sepal_length", "sepal_width", {
    title: "Sepal Dimensions by Species",
    color_by: "species",
    width: 600,
    height: 400
});
```

### Educational Assignment Version

```json
{
  "assignment_metadata": {
    "title": "Data Science Fundamentals with Iris Dataset",
    "course_id": "DS101",
    "difficulty": "beginner",
    "estimated_duration": "45 minutes",
    "learning_objectives": [
      "Load and inspect datasets",
      "Calculate basic statistics",
      "Create data visualizations",
      "Group data by categories"
    ]
  },
  "test_cases": [
    {
      "name": "Dataset Loading",
      "cell_id": "cell_2",
      "expected_output": {
        "type": "pattern",
        "pattern": "Dataset loaded: \\d+ samples"
      },
      "points": 10
    },
    {
      "name": "Species Count",
      "cell_id": "cell_4", 
      "expected_output": {
        "type": "exact",
        "value": "Species in dataset: [\"setosa\", \"versicolor\", \"virginica\"]"
      },
      "points": 15
    }
  ]
}
```

## Deployment Architecture

### Static Hosting Configuration

```yaml
# .github/workflows/deploy-notebooks.yml
name: Deploy Notebook Platform
on:
  push:
    branches: [main]
    paths:
      - 'crates/ruchy-notebook/**'
      - 'frontend/notebook/**'
      - 'notebooks/**'

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust + WASM
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Build WASM module
        run: |
          cd crates/ruchy-notebook
          wasm-pack build --target web --release
          
      - name: Build frontend
        run: |
          cd frontend/notebook
          npm ci
          npm run build
          
      - name: Convert demo notebooks
        run: |
          ./scripts/convert-demos-to-notebooks.sh
          cp -r ../ruchy-repl-demos/demos/repl/ ./public/demos/
          
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./frontend/notebook/dist
          cname: notebooks.ruchy-lang.org
```

### Progressive Web App Configuration

```json
{
  "name": "Ruchy Notebooks",
  "short_name": "Ruchy NB",
  "description": "Interactive data science notebooks powered by Ruchy",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#0d1117",
  "theme_color": "#58a6ff",
  "icons": [
    {
      "src": "/icons/ruchy-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icons/ruchy-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ],
  "categories": ["education", "productivity", "developer"],
  "screenshots": [
    {
      "src": "/screenshots/notebook-view.png",
      "sizes": "1280x720",
      "type": "image/png",
      "form_factor": "wide"
    }
  ],
  "offline_enabled": true,
  "features": [
    "notebook editing",
    "REPL execution", 
    "data visualization",
    "educational assignments"
  ]
}
```

## Performance Targets

### Size Budget
- **Core notebook engine**: <150KB gzipped
- **Visualization module**: <100KB gzipped  
- **Educational features**: <75KB gzipped
- **Total bundle**: <350KB gzipped (acceptable for educational tools)

### Execution Performance
- **Cell execution**: <100ms for simple code
- **Notebook loading**: <500ms for 50-cell notebook
- **Visualization rendering**: <200ms for 1000-point scatter plot
- **Collaboration sync**: <50ms operation transform latency

### Offline Capabilities
- **Full notebook editing** when offline
- **Code execution** using cached WASM
- **Sync on reconnection** with operational transform
- **Service worker caching** of notebooks and assets

## Educational Integration Strategy

### Assignment Workflow

1. **Instructor creates template** from demo notebook
2. **Students access via URL** (no installation required)
3. **Real-time collaboration** for group assignments
4. **Auto-grading** with hidden test cases
5. **Progress tracking** with learning analytics
6. **Export submissions** for manual review

### Assessment Features

- **Automatic test execution** with timeout limits
- **Plagiarism detection** via AST comparison
- **Performance profiling** for algorithmic assignments
- **Code quality metrics** integration with PMAT
- **Learning objective mapping** to curriculum standards

## Migration Path from v1.89.0

### Version Progression

- **v1.90.0**: Core notebook infrastructure + demo conversion
- **v1.91.0**: Educational features + assignment creation
- **v1.92.0**: Real-time collaboration + advanced visualization
- **v1.93.0**: Performance optimization + offline support
- **v2.0.0**: Full Jupyter compatibility + plugin ecosystem

### Backward Compatibility

- **All existing .ruchy files** convert seamlessly
- **REPL sessions** import as executable notebooks
- **Demo structure** preserved with enhanced interactivity
- **Educational metadata** extracted from comments

## Success Metrics

### Technical Metrics
- **Conversion accuracy**: 100% of demos → notebooks
- **Performance target**: <350KB total bundle size
- **Compatibility**: Chrome 90+, Firefox 88+, Safari 14+
- **Offline capability**: Full editing + execution

### Educational Metrics  
- **Adoption rate**: Target 80% of CS educators
- **Student engagement**: 2x session time vs traditional REPLs
- **Assignment completion**: 95%+ submission rate
- **Learning outcomes**: Measurable improvement in programming skills

## Conclusion

This specification enables a seamless transition from Ruchy's current REPL-based demonstrations to a full-featured notebook environment. By building on existing infrastructure (WASM REPL, replay system, demo collection) and established patterns (Jupyter compatibility, educational workflows), we create a compelling platform for data science education and interactive computing.

The phased approach ensures incremental value delivery while maintaining our commitment to quality and performance. Each phase builds naturally on the previous, creating a robust foundation for the future of interactive Ruchy programming.