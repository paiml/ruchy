# Sub-spec: WASM REPL -- State Management and Language Integration

**Parent:** [wasm-repl-spec.md](../wasm-repl-spec.md) State Management and Language Integration Sections

---

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

