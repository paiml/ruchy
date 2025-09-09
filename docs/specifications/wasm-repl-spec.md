# Ruchy WASM Notebooks - Technical Specification v4.0

## Executive Summary

A browser-based notebook runtime for Ruchy, compiled to WebAssembly with bytecode interpretation. Delivers <50ms cell execution with Apache Arrow-compatible DataFrames in a <200KB WASM module.

## Architecture

### Bytecode VM

```rust
// Stack-based VM with register optimization
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    Push(Value),
    Pop,
    Dup,
    
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Memory
    LoadLocal(u16),
    StoreLocal(u16),
    LoadGlobal(u16),
    StoreGlobal(u16),
    
    // Control flow
    Jump(i32),
    JumpIf(i32),
    Call(u16),
    Return,
    
    // DataFrame ops
    Select(u16),  // Column indices
    Filter(u16),  // Predicate function
    Project(u16), // Transform function
}

pub struct BytecodeVM {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: Vec<Value>,
    constants: Vec<Value>,
}

impl BytecodeVM {
    pub fn execute(&mut self, chunk: &Chunk) -> Result<Value> {
        let mut ip = 0;
        let mut frame = CallFrame::new(chunk);
        
        loop {
            let op = chunk.code[ip];
            ip += 1;
            
            match op {
                OpCode::Push(v) => self.stack.push(v),
                OpCode::Add => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(a.add(b)?);
                }
                OpCode::LoadGlobal(idx) => {
                    self.stack.push(self.globals[idx as usize].clone());
                }
                OpCode::StoreGlobal(idx) => {
                    let val = self.stack.pop()?;
                    if idx as usize >= self.globals.len() {
                        self.globals.resize(idx as usize + 1, Value::Nil);
                    }
                    self.globals[idx as usize] = val;
                }
                OpCode::Return => {
                    return Ok(self.stack.pop()?);
                }
                _ => todo!()
            }
        }
    }
}
```

### Memory Architecture

```rust
// Zero-copy arena with explicit promotion
pub struct CellMemory {
    transient: BumpArena<256_KB>,  // Reset after execution
    persistent: SlabAllocator,      // Promoted values only
}

impl CellMemory {
    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        // Allocate in transient by default
        self.transient.alloc(size)
    }
    
    pub fn promote(&mut self, value: &Value) -> PromotedRef {
        // Explicitly copy to persistent storage
        match value {
            Value::DataFrame(df) => {
                // Zero-copy promotion for Arrow buffers
                PromotedRef::DataFrame(self.persistent.adopt(df))
            }
            _ => {
                // Deep copy for other types
                PromotedRef::Value(self.persistent.copy(value))
            }
        }
    }
}

// Explicit global promotion
pub struct GlobalRegistry {
    values: HashMap<String, PromotedRef>,
    
    // Track which cell defined each global
    provenance: HashMap<String, CellId>,
}

impl GlobalRegistry {
    pub fn export(&mut self, name: String, value: PromotedRef, cell: CellId) {
        self.provenance.insert(name.clone(), cell);
        self.values.insert(name, value);
    }
}
```

### Apache Arrow DataFrame

```rust
// Arrow-compatible columnar layout
#[repr(C)]
pub struct DataFrame {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,  // Arrow arrays
    row_count: usize,
}

pub type ArrayRef = Arc<dyn Array>;

pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn slice(&self, offset: usize, length: usize) -> ArrayRef;
    fn buffer(&self) -> &Buffer;  // Raw bytes for zero-copy
}

// Concrete array types
pub struct Float64Array {
    data: Buffer,
    null_bitmap: Option<Bitmap>,
}

impl Array for Float64Array {
    fn slice(&self, offset: usize, length: usize) -> ArrayRef {
        // Zero-copy slice
        Arc::new(Float64Array {
            data: self.data.slice(offset * 8, length * 8),
            null_bitmap: self.null_bitmap.as_ref().map(|b| b.slice(offset, length)),
        })
    }
}

impl DataFrame {
    pub fn select(&self, columns: &[&str]) -> DataFrame {
        let selected: Vec<ArrayRef> = self.schema.fields()
            .iter()
            .zip(&self.columns)
            .filter(|(field, _)| columns.contains(&field.name.as_str()))
            .map(|(_, col)| col.clone())
            .collect();
        
        DataFrame {
            schema: Arc::new(self.schema.project(columns)),
            columns: selected,
            row_count: self.row_count,
        }
    }
    
    pub fn filter_indices(&self, indices: &[usize]) -> DataFrame {
        // Build once, apply to all columns
        let columns = self.columns.iter()
            .map(|col| take(col, indices))
            .collect();
        
        DataFrame {
            schema: self.schema.clone(),
            columns,
            row_count: indices.len(),
        }
    }
}

// Efficient gather operation
fn take(array: &ArrayRef, indices: &[usize]) -> ArrayRef {
    match array.data_type() {
        DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            let mut builder = Float64Builder::new(indices.len());
            for &idx in indices {
                builder.append_value(arr.value(idx));
            }
            Arc::new(builder.finish())
        }
        _ => todo!()
    }
}
```

### Structured Error Handling

```rust
#[derive(Serialize, Deserialize)]
pub struct ExecutionError {
    kind: ErrorKind,
    message: String,
    span: Option<Span>,
    backtrace: Vec<Frame>,
    suggestion: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum ErrorKind {
    ParseError,
    TypeError { expected: String, found: String },
    NameError { name: String },
    IndexError { index: usize, length: usize },
    RuntimeError,
}

#[wasm_bindgen]
impl NotebookRuntime {
    pub fn execute(&mut self, cell_id: &str, code: &str) -> Result<JsValue, JsValue> {
        match self.execute_internal(cell_id, code) {
            Ok(value) => Ok(serialize_value(&value)),
            Err(err) => {
                // Structured error for rich display
                let error_obj = ExecutionError {
                    kind: err.kind(),
                    message: err.to_string(),
                    span: err.span(),
                    backtrace: err.backtrace(),
                    suggestion: self.suggest_fix(&err),
                };
                Err(serde_wasm_bindgen::to_value(&error_obj)?)
            }
        }
    }
    
    fn suggest_fix(&self, error: &Error) -> Option<String> {
        match error.kind() {
            ErrorKind::NameError { name } => {
                // Levenshtein distance for typo suggestions
                let closest = self.find_similar_names(name);
                if !closest.is_empty() {
                    Some(format!("Did you mean: {}?", closest.join(", ")))
                } else {
                    None
                }
            }
            _ => None
        }
    }
}
```

## Robust Demo Conversion

```rust
// Simplified AST for structural parsing only
pub enum DemoNode {
    Comment(String),
    Code(Vec<Statement>),
    Section(String),
}

pub struct DemoParser {
    lexer: Lexer,
}

impl DemoParser {
    pub fn parse(&mut self, input: &str) -> Result<Vec<DemoNode>> {
        self.lexer.init(input);
        let mut nodes = Vec::new();
        
        while !self.lexer.is_at_end() {
            if self.lexer.peek_comment() {
                nodes.push(self.parse_comment_block());
            } else if self.lexer.peek_section_marker() {
                nodes.push(self.parse_section());
            } else {
                nodes.push(self.parse_code_block());
            }
        }
        
        Ok(nodes)
    }
    
    fn parse_code_block(&mut self) -> DemoNode {
        let mut statements = Vec::new();
        
        // Parse until we hit a comment or section
        while !self.lexer.peek_comment() && !self.lexer.peek_section_marker() && !self.lexer.is_at_end() {
            if let Ok(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // Skip unparseable lines gracefully
                self.lexer.skip_line();
            }
        }
        
        DemoNode::Code(statements)
    }
}

pub struct NotebookConverter {
    parser: DemoParser,
}

impl NotebookConverter {
    pub fn convert(&self, demo: &str) -> Result<Notebook> {
        let nodes = self.parser.parse(demo)?;
        let mut cells = Vec::new();
        
        for node in nodes {
            match node {
                DemoNode::Section(title) => {
                    cells.push(Cell::Markdown(format!("# {}", title)));
                }
                DemoNode::Comment(text) => {
                    cells.push(Cell::Markdown(text));
                }
                DemoNode::Code(statements) => {
                    // Group related statements
                    let groups = self.group_statements(statements);
                    for group in groups {
                        cells.push(Cell::Code(self.statements_to_code(group)));
                    }
                }
            }
        }
        
        Ok(Notebook { cells })
    }
    
    fn group_statements(&self, statements: Vec<Statement>) -> Vec<Vec<Statement>> {
        // Group by logical units (function definitions, data operations, etc.)
        let mut groups = Vec::new();
        let mut current = Vec::new();
        
        for stmt in statements {
            match stmt {
                Statement::FunctionDef(_) if !current.is_empty() => {
                    // Start new group for functions
                    groups.push(current);
                    current = vec![stmt];
                }
                Statement::Import(_) if !current.is_empty() => {
                    // Imports get their own cell
                    groups.push(current);
                    groups.push(vec![stmt]);
                    current = Vec::new();
                }
                _ => current.push(stmt),
            }
        }
        
        if !current.is_empty() {
            groups.push(current);
        }
        
        groups
    }
}
```

## Global State Management

```rust
// Explicit global promotion with `global` keyword
pub enum Statement {
    Let { name: String, value: Expr },
    GlobalLet { name: String, value: Expr },  // New
    // ...
}

impl BytecodeCompiler {
    fn compile_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Let { name, value } => {
                self.compile_expr(value)?;
                let slot = self.allocate_local(name);
                self.emit(OpCode::StoreLocal(slot));
            }
            Statement::GlobalLet { name, value } => {
                self.compile_expr(value)?;
                let slot = self.allocate_global(name);
                self.emit(OpCode::StoreGlobal(slot));
                
                // Mark as explicitly exported
                self.chunk.metadata.exported_globals.insert(name.clone());
            }
            // ...
        }
        Ok(())
    }
}

// Frontend shows global provenance
#[wasm_bindgen]
impl NotebookRuntime {
    pub fn get_globals(&self) -> JsValue {
        let globals: HashMap<String, GlobalInfo> = self.global_registry
            .values
            .iter()
            .map(|(name, value)| {
                let info = GlobalInfo {
                    name: name.clone(),
                    type_name: value.type_name(),
                    defined_in: self.global_registry.provenance[name].clone(),
                    size_bytes: value.size_of(),
                };
                (name.clone(), info)
            })
            .collect();
        
        serde_wasm_bindgen::to_value(&globals).unwrap()
    }
}
```

## Language Integration Model

### Execution Modes

The system provides three progressive execution contexts, each preserving semantic equivalence:

```rust
pub enum ExecutionMode {
    Script,     // Linear execution, no persistence
    Repl,       // Interactive with session state
    Notebook,   // Cell-based with full persistence
}

impl Runtime {
    pub fn execute(path: &Path, mode: ExecutionMode) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        match mode {
            ExecutionMode::Script => {
                // Single-pass execution
                let ast = parse(&content)?;
                let bytecode = compile(&ast)?;
                self.vm.execute(&bytecode)
            }
            ExecutionMode::Repl => {
                // Session-based with history
                let session = ReplSession::new();
                session.eval_file(&content)?;
                session.interactive_loop()
            }
            ExecutionMode::Notebook => {
                // Cell-segmented execution
                let notebook = self.segment_into_cells(&content)?;
                self.serve_notebook(notebook)
            }
        }
    }
}
```

### File Format Unification

Every `.ruchy` file is simultaneously a valid script and a single-cell notebook:

```rust
// analysis.ruchy - valid in all contexts
let data = read_csv("sales.csv")
let quarterly = data.groupby("quarter").sum()
println(quarterly)
```

Execution semantics remain invariant across modes:
- `ruchy run analysis.ruchy` - Script mode
- `ruchy repl < analysis.ruchy` - REPL mode
- `ruchy notebook analysis.ruchy` - Notebook mode

### AST-Directed Cell Segmentation

```rust
pub struct CellSegmenter {
    ast: AST,
    boundaries: Vec<BoundaryRule>,
}

impl CellSegmenter {
    pub fn segment(&self, ast: AST) -> Vec<Cell> {
        let mut cells = Vec::new();
        let mut current = CellBuilder::new();
        
        for node in ast.traverse_preorder() {
            match self.classify_boundary(&node) {
                Boundary::Function => {
                    cells.push(current.build());
                    current = CellBuilder::new();
                    current.add(node);
                }
                Boundary::DataLoad => {
                    if !current.is_empty() {
                        cells.push(current.build());
                        current = CellBuilder::new();
                    }
                    current.add(node);
                    cells.push(current.build());
                    current = CellBuilder::new();
                }
                Boundary::None => current.add(node),
            }
        }
        
        if !current.is_empty() {
            cells.push(current.build());
        }
        
        cells
    }
}
```

### Bidirectional Transformation

```rust
impl Converter {
    pub fn script_to_notebook(&self, script: &str) -> Notebook {
        let ast = parse(script)?;
        let cells = self.segmenter.segment(ast);
        
        Notebook {
            cells,
            metadata: self.extract_metadata(&ast),
            kernel_spec: KernelSpec::ruchy(),
        }
    }
    
    pub fn notebook_to_script(&self, notebook: &Notebook) -> String {
        let mut script = String::new();
        
        // Topological sort for dependency order
        let ordered = self.resolve_dependencies(&notebook.cells);
        
        for cell in ordered {
            match cell {
                Cell::Code(code) => script.push_str(&code),
                Cell::Markdown(md) => {
                    // Preserve as comments
                    for line in md.lines() {
                        script.push_str(&format!("// {}\n", line));
                    }
                }
            }
        }
        
        script
    }
}
```

### Module System Integration

Notebooks and scripts share the unified module resolver:

```rust
pub struct ModuleResolver {
    search_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    pub fn resolve(&self, import: &ImportPath) -> Result<Module> {
        let path = self.find_module(import)?;
        
        match path.extension() {
            Some("ruchy") => Module::Script(self.load_script(&path)?),
            Some("ipynb") => Module::Notebook(self.load_notebook(&path)?),
            _ => Err(Error::UnknownModuleType)
        }
    }
    
    pub fn load_notebook(&self, path: &Path) -> Result<NotebookModule> {
        let notebook = Notebook::load(path)?;
        
        // Extract only code cells in dependency order
        let code_cells: Vec<_> = notebook.cells
            .iter()
            .filter_map(|c| match c {
                Cell::Code(code) => Some(code.clone()),
                _ => None
            })
            .collect();
        
        Ok(NotebookModule {
            exports: self.extract_exports(&code_cells),
            bytecode: self.compile_cells(&code_cells)?,
        })
    }
}
```

### REPL Session Management

```rust
pub struct ReplSession {
    history: Vec<Command>,
    bindings: GlobalRegistry,
    cell_accumulator: Vec<Cell>,
}

impl ReplSession {
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        let result = self.runtime.execute(input)?;
        
        // Accumulate as potential notebook cell
        self.cell_accumulator.push(Cell::Code(input.to_string()));
        
        Ok(result)
    }
    
    pub fn save_notebook(&self, path: &Path) -> Result<()> {
        let notebook = Notebook {
            cells: self.cell_accumulator.clone(),
            metadata: NotebookMetadata {
                created: Utc::now(),
                kernel: "ruchy",
                origin: Origin::Repl,
            },
        };
        
        notebook.save(path)
    }
    
    pub fn export_script(&self, path: &Path) -> Result<()> {
        // Remove REPL artifacts, consolidate imports
        let script = self.clean_history();
        std::fs::write(path, script)
    }
}
```

### Browser Workflow

```typescript
// Frontend notebook interface
class RuchyNotebook {
    private runtime: WasmRuntime;
    
    async loadFile(file: File) {
        const content = await file.text();
        
        if (file.name.endsWith('.ruchy')) {
            // Auto-segment into cells
            const notebook = await this.runtime.script_to_notebook(content);
            this.render(notebook);
        } else if (file.name.endsWith('.ipynb')) {
            const notebook = JSON.parse(content);
            this.render(notebook);
        }
    }
    
    async export(format: 'ruchy' | 'ipynb' | 'html') {
        switch (format) {
            case 'ruchy':
                return this.runtime.notebook_to_script(this.cells);
            case 'ipynb':
                return JSON.stringify(this.toJupyterFormat());
            case 'html':
                return this.generateStandalone();
        }
    }
    
    private async generateStandalone(): Promise<string> {
        // Inline WASM module and notebook data
        const wasmBase64 = await this.runtime.serialize_base64();
        const notebookJson = JSON.stringify(this.cells);
        
        return `<!DOCTYPE html>
<html>
<head>
    <script>
        const wasmModule = "${wasmBase64}";
        const notebook = ${notebookJson};
        // Self-contained execution
    </script>
</head>
<body>
    <div id="notebook"></div>
</body>
</html>`;
    }
}
```

### Migration Paths

```rust
// Python to Ruchy transpiler
pub struct PythonConverter {
    ast_map: HashMap<PyNode, RuchyNode>,
}

impl PythonConverter {
    pub fn convert(&self, py_code: &str) -> Result<String> {
        let py_ast = python_parser::parse(py_code)?;
        
        let ruchy_ast = self.translate_ast(py_ast)?;
        
        // Preserve semantics where possible
        // Flag incompatible constructs
        self.emit_ruchy(ruchy_ast)
    }
    
    fn translate_ast(&self, py: PyAST) -> Result<AST> {
        match py {
            PyAST::Import("pandas") => {
                Ok(AST::Comment("pandas operations use native DataFrame"))
            }
            PyAST::Call("pd.read_csv", args) => {
                Ok(AST::Call("read_csv", self.translate_args(args)?))
            }
            // Pattern match common idioms
            _ => self.default_translation(py)
        }
    }
}
```

### Compilation Unity

The notebook preserves optimization hints from interactive execution:

```rust
pub struct NotebookCompiler {
    profiling_data: HashMap<CellId, ProfileData>,
}

impl NotebookCompiler {
    pub fn compile_to_native(&self, notebook: &Notebook) -> Binary {
        let mut compiler = NativeCompiler::new();
        
        // Use profiling data for optimization
        for (cell_id, profile) in &self.profiling_data {
            if profile.execution_count > 10 {
                compiler.mark_hot(cell_id);
            }
            
            // Specialize for observed types
            for (var, type_freq) in &profile.type_histogram {
                if type_freq.mode_percentage() > 0.9 {
                    compiler.specialize(var, type_freq.mode());
                }
            }
        }
        
        compiler.compile(notebook)
    }
}
```

## Build Configuration

```toml
[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
arrow2 = { version = "0.18", default-features = false, features = ["compute"] }

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

# Custom allocator for minimal size
[dependencies.wee_alloc]
version = "0.4"
features = ["size_classes"]
```

```bash
#!/bin/bash
# build.sh

# Compile to WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize with binaryen
wasm-opt -Oz \
    --enable-bulk-memory \
    --enable-mutable-globals \
    --converge \
    target/wasm32-unknown-unknown/release/ruchy_notebook.wasm \
    -o ruchy_notebook_opt.wasm

# Generate bindings
wasm-bindgen ruchy_notebook_opt.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

# Verify size
SIZE=$(stat -c%s pkg/ruchy_notebook_bg.wasm)
if [ $SIZE -gt 204800 ]; then
    echo "ERROR: WASM size ${SIZE} exceeds 200KB limit"
    exit 1
fi
```

## Performance Metrics

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Bytecode compilation | <5ms | Achieved via single-pass compiler |
| Cell execution (simple) | <10ms | Stack VM with inline caching |
| Cell execution (1K DataFrame) | <50ms | Zero-copy Arrow operations |
| Memory per cell | 256KB | Arena reset between executions |
| Global promotion | <1ms | Explicit slab allocation |

## Enhanced DataFrame with Arrow Compute Kernels

### Apache Arrow DataFrame with Compute Kernels

```rust
use arrow::compute;
use arrow::array::{ArrayRef, Float64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

pub struct DataFrame {
    batch: RecordBatch,  // Arrow's native batch format
}

impl DataFrame {
    // Use Arrow's optimized kernels
    pub fn filter(&self, predicate: &BooleanArray) -> Result<DataFrame> {
        let filtered = compute::filter_record_batch(&self.batch, predicate)?;
        Ok(DataFrame { batch: filtered })
    }
    
    pub fn select(&self, columns: &[&str]) -> Result<DataFrame> {
        let schema = self.batch.schema();
        let indices: Vec<usize> = columns.iter()
            .filter_map(|name| schema.index_of(name).ok())
            .collect();
        
        let selected = self.batch.project(&indices)?;
        Ok(DataFrame { batch: selected })
    }
    
    pub fn group_by(&self, keys: &[&str]) -> GroupedDataFrame {
        // Use Arrow's aggregation kernels
        let key_columns: Vec<ArrayRef> = keys.iter()
            .map(|k| self.batch.column_by_name(k).unwrap().clone())
            .collect();
        
        GroupedDataFrame {
            source: self.batch.clone(),
            keys: key_columns,
        }
    }
    
    pub fn join(&self, other: &DataFrame, on: &str, how: JoinType) -> Result<DataFrame> {
        // Leverage Arrow's hash join
        let left_col = self.batch.column_by_name(on)?;
        let right_col = other.batch.column_by_name(on)?;
        
        let indices = compute::hash_join(left_col, right_col, how)?;
        let joined = self.merge_on_indices(&other, indices)?;
        
        Ok(DataFrame { batch: joined })
    }
    
    pub fn sort(&self, by: &[&str], ascending: bool) -> Result<DataFrame> {
        let options = compute::SortOptions {
            descending: !ascending,
            nulls_first: false,
        };
        
        let indices = compute::lexsort_to_indices(&self.batch, by, Some(options))?;
        let sorted = compute::take_record_batch(&self.batch, &indices)?;
        
        Ok(DataFrame { batch: sorted })
    }
}

pub struct GroupedDataFrame {
    source: RecordBatch,
    keys: Vec<ArrayRef>,
}

impl GroupedDataFrame {
    pub fn agg(&self, ops: Vec<AggOp>) -> Result<DataFrame> {
        let mut aggregated = Vec::new();
        
        for op in ops {
            let column = self.source.column_by_name(&op.column)?;
            
            let result = match op.function {
                AggFunc::Sum => compute::sum(column)?,
                AggFunc::Mean => compute::mean(column)?,
                AggFunc::Min => compute::min(column)?,
                AggFunc::Max => compute::max(column)?,
                AggFunc::Count => compute::count(column)?,
            };
            
            aggregated.push(result);
        }
        
        Ok(DataFrame::from_columns(aggregated))
    }
}
```

## Project Scaffolding

### Project Initialization

```rust
// CLI command for project setup
pub struct ProjectInit;

impl ProjectInit {
    pub fn create(path: &Path, template: ProjectTemplate) -> Result<()> {
        let project_root = path.canonicalize()?;
        
        // Create standard structure
        fs::create_dir_all(project_root.join("src"))?;
        fs::create_dir_all(project_root.join("notebooks"))?;
        fs::create_dir_all(project_root.join("data"))?;
        fs::create_dir_all(project_root.join("tests"))?;
        
        // Generate ruchy.toml
        let config = ProjectConfig {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            version: "0.1.0",
            entry: "src/main.ruchy",
            notebook_dir: "notebooks",
            dependencies: template.default_deps(),
        };
        
        let toml = toml::to_string_pretty(&config)?;
        fs::write(project_root.join("ruchy.toml"), toml)?;
        
        // Create example files based on template
        match template {
            ProjectTemplate::DataScience => {
                self.create_data_science_template(&project_root)?;
            }
            ProjectTemplate::WebService => {
                self.create_web_service_template(&project_root)?;
            }
            ProjectTemplate::Library => {
                self.create_library_template(&project_root)?;
            }
        }
        
        Ok(())
    }
    
    fn create_data_science_template(&self, root: &Path) -> Result<()> {
        // src/main.ruchy
        fs::write(root.join("src/main.ruchy"), r#"
// Data science project entry point
import "./data_prep.ruchy"
import "./analysis.ruchy"

global let config = {
    data_path: "data/",
    output_path: "results/"
}

fn main() {
    let data = load_dataset(config.data_path)
    let cleaned = prepare_data(data)
    let results = run_analysis(cleaned)
    save_results(results, config.output_path)
}
"#)?;
        
        // notebooks/exploration.ipynb
        let notebook = Notebook {
            cells: vec![
                Cell::Markdown("# Data Exploration\n\nInteractive analysis notebook".into()),
                Cell::Code("import \"../src/data_prep.ruchy\"\nlet df = load_dataset(\"../data/sample.csv\")".into()),
                Cell::Code("df.describe()".into()),
            ],
            metadata: NotebookMetadata::default(),
        };
        
        fs::write(
            root.join("notebooks/exploration.ipynb"),
            serde_json::to_string_pretty(&notebook)?
        )?;
        
        Ok(())
    }
}
```

### CLI Interface

```bash
# Initialize new project
$ ruchy init my_project --template data-science
Created project structure:
my_project/
├── ruchy.toml
├── src/
│   ├── main.ruchy
│   ├── data_prep.ruchy
│   └── analysis.ruchy
├── notebooks/
│   └── exploration.ipynb
├── data/
│   └── .gitkeep
└── tests/
    └── test_analysis.ruchy

# Run in different modes
$ cd my_project
$ ruchy run                    # Script mode: runs src/main.ruchy
$ ruchy repl                   # REPL mode: interactive session
$ ruchy notebook               # Notebook mode: opens browser
$ ruchy test                   # Runs tests/ directory

# Build for deployment
$ ruchy build --release        # Native binary
$ ruchy build --target wasm   # WASM module
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn bytecode_equivalence(code in any::<String>()) {
            let ast = parse(&code)?;
            let bytecode = compile(&ast)?;
            
            let interpreted = interpret(&ast)?;
            let executed = execute(&bytecode)?;
            
            prop_assert_eq!(interpreted, executed);
        }
        
        #[test]
        fn dataframe_zero_copy(size in 1..10000usize) {
            let df = DataFrame::random(size);
            let slice = df.slice(0, size/2);
            
            // Verify slicing doesn't copy data
            let df_ptr = df.columns[0].buffer().as_ptr();
            let slice_ptr = slice.columns[0].buffer().as_ptr();
            prop_assert_eq!(df_ptr, slice_ptr);
        }
    }
}
```

## Implementation Timeline

### Weeks 1-2: Bytecode VM
- OpCode design and stack machine
- Single-pass compiler from AST
- Basic arithmetic and control flow

### Weeks 3-4: Memory Management
- Arena allocator with reset
- Explicit global promotion
- Slab allocator for persistent values

### Weeks 5-6: Arrow DataFrame
- Buffer management and zero-copy slices
- Basic operations (select, filter, take)
- WASM serialization

### Weeks 7-8: Error Handling
- Structured error types
- Span tracking through compilation
- Suggestion engine

### Weeks 9-10: Demo Conversion
- Lightweight AST parser
- Statement grouping logic
- Batch conversion validation

### Weeks 11-12: Integration
- Frontend polish
- Performance optimization
- Documentation

## Conclusion

This specification delivers a performant notebook runtime through disciplined scope management and proven techniques. The bytecode VM provides 5-10x performance over tree-walking while remaining simple to implement. Apache Arrow compatibility ensures future interoperability without current complexity. Explicit global promotion eliminates hidden state dependencies.

The architecture scales from MVP to production without fundamental rewrites, validating the lean principle of building quality in from the start.
## Implementation Architecture

The notebook system ships as a subcommand of the main `ruchy` binary, not a separate installation. Single binary distribution maintains the zero-friction principle.

### Crate Structure

```
ruchy/
├── ruchy-core/          # Parser, type system, bytecode VM
├── ruchy-notebook/      # Notebook runtime (depends on core)
├── ruchy-wasm/          # WASM compilation target
└── ruchy-cli/           # Binary entry point
```

The `ruchy-notebook` crate compiles to both native and WASM targets. Native for local execution, WASM for browser deployment.

### Installation Experience

```bash
# Standard installation includes everything
$ cargo install ruchy
# or
$ brew install ruchy

# Verify notebook support
$ ruchy --version
ruchy 1.90.0 (notebook-enabled)
```

No separate tools. No npm packages. No Python environments. The notebook runtime is embedded in the main binary.

### Execution Modes

```bash
# Mode 1: Local notebook server (default)
$ ruchy notebook
Starting notebook server at http://localhost:8080
Opening browser...

# Mode 2: Convert and run existing script
$ ruchy notebook analysis.ruchy
Converted to notebook with 5 cells
Server running at http://localhost:8080

# Mode 3: Static WASM generation (no server)
$ ruchy notebook build analysis.ruchy --static
Generated: analysis.html (267KB, self-contained)

# Mode 4: Headless execution
$ ruchy notebook run analysis.ipynb --headless
Cell 1: ✓
Cell 2: ✓
Results saved to output/
```

### Runtime Architecture

The notebook server runs as a Rust web server (using `axum` or `warp`) that serves:
1. Static HTML/JS/CSS assets (embedded in binary)
2. WASM module (compiled from `ruchy-notebook`)
3. WebSocket endpoint for hot reload

```rust
// Embedded in ruchy-cli/src/notebook.rs
pub fn run_notebook_server(opts: NotebookOpts) -> Result<()> {
    let wasm_module = include_bytes!(concat!(env!("OUT_DIR"), "/notebook.wasm"));
    let frontend = include_str!("../assets/notebook.html");
    
    let app = Router::new()
        .route("/", get(|| async { Html(frontend) }))
        .route("/notebook.wasm", get(|| async { 
            Response::builder()
                .header("Content-Type", "application/wasm")
                .body(wasm_module.to_vec())
        }))
        .route("/ws", get(websocket_handler));
    
    Server::bind(&opts.addr).serve(app).await
}
```

### Browser Execution Flow

1. User navigates to `localhost:8080`
2. Browser loads minimal HTML shell (~5KB)
3. WASM module streams in (~200KB)
4. Notebook UI initializes
5. Code execution happens entirely client-side

No round-trips to server for execution. Server only handles file I/O and WebSocket for collaboration (future).

### File System Integration

```rust
// Native file access through server
#[wasm_bindgen]
impl NotebookRuntime {
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        // In browser: fetch from server
        let response = fetch(&format!("/files/{}", path)).await?;
        response.bytes().await
    }
}

// Server endpoint
async fn file_handler(Path(path): Path<String>) -> Result<Vec<u8>> {
    // Sandboxed to project directory
    let safe_path = sandbox_path(&path)?;
    tokio::fs::read(safe_path).await
}
```

### Migration Path for Existing Users

```bash
# Existing workflow unchanged
$ ruchy run script.ruchy  # Still works

# Progressive enhancement
$ ruchy notebook script.ruchy  # Opens as notebook

# Jupyter users
$ ruchy notebook import analysis.ipynb --from python
Converting Python notebook to Ruchy...
- pandas.read_csv() → read_csv()
- df.groupby() → df.group_by()
Conversion complete with 3 warnings
```

### Package Distribution

For users who want browser-only experience:

```bash
# Generate standalone package
$ ruchy notebook package my_analysis/
Creating my_analysis.zip:
  - index.html (5KB)
  - notebook.wasm (200KB)
  - notebook.js (12KB)
  - data/ (copied as-is)

# User opens index.html in browser
# Everything runs locally, no server needed
```

### VS Code Integration

```json
// .vscode/settings.json
{
  "ruchy.notebook.enabled": true,
  "ruchy.notebook.port": 8080
}
```

Command palette: "Ruchy: Open as Notebook" converts current file to notebook view within VS Code.

### Performance Characteristics

**Cold start**:
- Native: 50ms to first prompt
- Browser: 200ms to interactive (WASM init)

**Hot reload**:
- File change detected via `notify`
- WebSocket pushes update
- Cell re-executes in <10ms

**Memory overhead**:
- Server: ~20MB RSS
- Browser: ~50MB including WASM heap

### Production Deployment

For classroom/enterprise:

```bash
# Docker image with pre-built WASM
FROM rust:1.75 as builder
COPY . .
RUN cargo build --release --features notebook

FROM debian:slim
COPY --from=builder /target/release/ruchy /usr/local/bin/
EXPOSE 8080
CMD ["ruchy", "notebook", "--host", "0.0.0.0"]
```

Cloud deployment requires only static file hosting:

```bash
# Generate for CDN deployment
$ ruchy notebook build --static --base-url https://notebooks.example.com
$ aws s3 sync output/ s3://notebooks.example.com
```

The key principle: **zero additional dependencies**. If you have Ruchy, you have notebooks. The WASM compilation happens at build time, not install time. Users never see webpack, node_modules, or Python environments.