# Sub-spec: Notebook Improvements — Architecture and Implementation

**Parent:** [ruchy-notebook-improvements.md](../ruchy-notebook-improvements.md) Sections 1-3

---

## Problem Statement

The notebook implementation violated the fundamental invariant of computational notebooks: state persistence across cells. Each cell instantiated an isolated REPL, creating semantic discontinuity that rendered the system unusable for iterative development.

### Root Failure Mode

```rust
// Pathological implementation - state lifetime bound to cell execution
pub fn execute_handler(req: ExecuteRequest) -> ExecuteResponse {
    let repl = Repl::new();  // Fresh context per invocation
    repl.eval(&req.code)     // No access to prior bindings
}
```

Debug telemetry confirmed the architectural defect:
```
cell-1: let x = 1     → REPL_1 created, x bound, REPL_1 destroyed
cell-2: println(x)    → REPL_2 created, x undefined, error
```

## Architectural Solution

### Core Abstraction: SharedSession

The fix introduces a persistent session layer that decouples REPL lifetime from cell execution:

```rust
pub struct SharedSession {
    repl: Repl,                               // Singleton evaluator
    globals: GlobalRegistry,                  // Persistent namespace
    dependencies: HashMap<String, HashSet<String>>,  // DAG for invalidation
    checkpoints: HashMap<String, GlobalRegistry>,    // Transactional rollback
}

pub struct GlobalRegistry {
    values: HashMap<String, Value>,           // O(1) binding lookup
    functions: HashMap<String, CompiledFn>,   // Cached bytecode
    types: HashMap<String, TypeInfo>,         // Type definitions
    imports: HashSet<String>,                 // Module registry
    provenance: HashMap<String, String>,      // Symbol → Cell mapping
}
```

### State Propagation Mechanism

```rust
impl Repl {
    pub fn eval_with_globals(
        &mut self, 
        code: &str, 
        globals: &mut GlobalRegistry,
        cell_id: &str
    ) -> Result<Value, Error> {
        // Phase 1: Hydrate context from globals
        for (name, value) in &globals.values {
            self.bindings.insert(name.clone(), value.clone());
        }
        
        // Phase 2: Execute with full namespace
        let ast = self.parse(code)?;
        let value = self.eval_ast(&ast)?;
        
        // Phase 3: Persist new bindings to globals
        for binding in self.extract_new_bindings(&ast) {
            globals.store_value(binding.name, binding.value, cell_id);
        }
        
        Ok(value)
    }
}
```

## Implementation Specifications

### Semantic Dependency Tracking

The system tracks dependencies through unique definition IDs, not names, eliminating shadowing ambiguities:

```rust
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct DefId(u64);  // Monotonic definition identifier

pub struct GlobalRegistry {
    values: HashMap<String, (Value, DefId)>,      // Name → (Value, Definition)
    def_sources: HashMap<DefId, String>,          // DefId → Cell
    next_def_id: AtomicU64,                       // Monotonic counter
}

impl SharedSession {
    fn update_dependencies(&mut self, cell_id: &str, execution: ExecutionResult) {
        // Track semantic dependencies, not lexical names
        let deps: HashSet<DefId> = execution.reads;  // DefIds consumed
        let defs: HashSet<DefId> = execution.writes; // DefIds produced
        
        self.def_graph.insert(cell_id.to_string(), (deps, defs));
        self.invalidate_consumers(&defs);
    }
    
    fn invalidate_consumers(&mut self, modified_defs: &HashSet<DefId>) {
        // Precise invalidation based on actual data flow
        for (cell, (deps, _)) in &self.def_graph {
            if !deps.is_disjoint(modified_defs) {
                self.stale_cells.insert(cell.clone());
            }
        }
    }
}
```

### Reactive Execution Mode

The system supports automatic re-execution of dependent cells through reactive propagation:

```rust
pub enum ExecutionMode {
    Manual,     // Traditional cell-by-cell
    Reactive,   // Automatic dependency propagation
}

impl SharedSession {
    pub fn explain_reactive(&self, cell_id: &str) -> ExecutionPlan {
        // Generate execution plan without executing
        let stale = self.find_stale_dependents(cell_id);
        let order = self.topological_sort(&stale);
        
        ExecutionPlan {
            primary: cell_id.to_string(),
            cascade: order.iter().map(|cell| {
                CascadeStep {
                    cell_id: cell.clone(),
                    estimated_time: self.estimate_execution_time(cell),
                    dependencies: self.def_graph.get(cell).map(|(d,_)| d.clone()),
                    can_skip: !self.is_critical(cell),
                }
            }).collect(),
            total_cells: order.len() + 1,
            estimated_total_time: self.estimate_total_time(&order),
        }
    }
    
    pub fn execute_reactive_with_plan(
        &mut self, 
        cell_id: &str, 
        code: &str,
        plan: ExecutionPlan
    ) -> Vec<ExecuteResponse> {
        let mut responses = Vec::new();
        
        // Execute primary cell
        let primary_response = self.execute(cell_id, code);
        responses.push(primary_response.clone());
        
        if !primary_response.success {
            return responses;  // Halt cascade on primary failure
        }
        
        // Execute only approved cascade steps
        for step in plan.cascade.iter().filter(|s| !s.skipped) {
            if let Some(cell_code) = self.cell_cache.get(&step.cell_id) {
                let response = self.execute(&step.cell_id, cell_code);
                responses.push(response.clone());
                
                if !response.success && self.halt_on_error {
                    break;
                }
            }
        }
        
        responses
    }
}
    
    fn topological_sort(&self, cells: &HashSet<String>) -> Vec<String> {
        // Kahn's algorithm for dependency order
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut sorted = Vec::new();
        
        // Build in-degree map
        for cell in cells {
            let deps = self.def_graph.get(cell).map(|(d, _)| d.len()).unwrap_or(0);
            in_degree.insert(cell.clone(), deps);
            if deps == 0 {
                queue.push_back(cell.clone());
            }
        }
        
        // Process queue
        while let Some(cell) = queue.pop_front() {
            sorted.push(cell.clone());
            
            // Decrement in-degree of dependents
            for dependent in self.find_dependents(&cell) {
                if let Some(degree) = in_degree.get_mut(&dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }
        
        sorted
    }
}
```

### Transactional Execution with Structural Sharing

Each cell execution operates within a transaction boundary with copy-on-write semantics:

```rust
impl SharedSession {
    pub fn execute(&mut self, cell_id: &str, code: &str) -> ExecuteResponse {
        // Structural sharing checkpoint - O(1) time, minimal memory
        let snapshot = self.globals.cow_checkpoint();
        
        match self.repl.eval_with_globals(code, &mut self.globals, cell_id) {
            Ok(value) => {
                self.invalidate_consumers(cell_id);
                ExecuteResponse::success(value)
            }
            Err(err) => {
                self.globals.restore_cow(snapshot);  // Only modified paths copied
                ExecuteResponse::error(err)
            }
        }
    }
}

impl GlobalRegistry {
    fn cow_checkpoint(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            values: Arc::clone(&self.values),        // Arc bump, no copy
            functions: Arc::clone(&self.functions),  // Structural sharing
            generation: self.generation,
        }
    }
    
    fn restore_cow(&mut self, snapshot: RegistrySnapshot) {
        self.values = snapshot.values;
        self.functions = snapshot.functions;
        self.generation = snapshot.generation;
    }
}

// Persistent data structures via battle-tested crates
pub enum Value {
    DataFrame(Arc<polars::DataFrame>),      // Polars native COW
    Array(im::Vector<i32>),                 // im crate: O(log n) ops
    Dict(im::HashMap<String, Value>),       // im crate: HAMT-based
    Scalar(ScalarValue),
}
```

### Hierarchical Namespace Architecture

The system supports module-scoped namespaces for logical isolation:

```rust
pub struct Module {
    name: String,
    parent: Option<ModuleId>,
    exports: HashSet<String>,
    bindings: HashMap<String, (Value, DefId)>,
    submodules: HashMap<String, ModuleId>,
}

pub struct GlobalRegistry {
    root: Module,
    modules: HashMap<ModuleId, Module>,
    current_module: Vec<ModuleId>,  // Module stack for nested scopes
}

impl GlobalRegistry {
    pub fn resolve(&self, path: &str) -> Option<(Value, DefId)> {
        let segments: Vec<&str> = path.split("::").collect();
        
        // Resolution only in current module - explicit imports required
        let current = self.current_module.last()
            .and_then(|id| self.modules.get(id))
            .unwrap_or(&self.root);
        
        current.resolve_local(&segments)
    }
    
    pub fn import(&mut self, from: &str, items: Vec<String>) {
        // Explicit import required for cross-module access
        let source_module = self.find_module(from);
        let current = self.current_module.last()
            .and_then(|id| self.modules.get_mut(id));
        
        for item in items {
            if let Some(value) = source_module.exports.get(&item) {
                current.imports.insert(item, value.clone());
            }
        }
    }
    
    pub fn enter_module(&mut self, name: &str) -> ModuleId {
        let parent = self.current_module.last().copied();
        let module = Module::new(name, parent);
        let id = self.modules.insert(module);
        self.current_module.push(id);
        id
    }
}

// Syntax extension for module blocks
impl Parser {
    fn parse_module(&mut self) -> Result<Statement, ParseError> {
        // module math {
        //     fn sqrt(x) { ... }
        //     export { sqrt }
        // }
        let name = self.expect_ident()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_statements()?;
        let exports = self.parse_exports()?;
        self.expect(Token::RBrace)?;
        
        Ok(Statement::Module { name, body, exports })
    }
}
```

### WASM Integration Layer

The notebook runtime exposes a comprehensive state inspection API:

```rust
#[wasm_bindgen]
impl NotebookRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(SharedSession::new())),
            cells: HashMap::new(),
        }
    }
    
    #[wasm_bindgen(js_name = executeCell)]
    pub fn execute_cell(&mut self, cell_id: &str, code: &str) -> JsValue {
        let mut session = self.session.lock().unwrap();
        let response = session.execute(cell_id, code);
        serde_wasm_bindgen::to_value(&response).unwrap()
    }
    
    #[wasm_bindgen(js_name = executeCellReactive)]
    pub fn execute_cell_reactive(&mut self, cell_id: &str, code: &str) -> JsValue {
        let mut session = self.session.lock().unwrap();
        let responses = session.execute_reactive(cell_id, code);
        serde_wasm_bindgen::to_value(&responses).unwrap()
    }
    
    // State Inspection API
    
    #[wasm_bindgen(js_name = getGlobals)]
    pub fn get_globals(&self) -> JsValue {
        let session = self.session.lock().unwrap();
        let globals = session.globals.serialize_for_inspection();
        serde_wasm_bindgen::to_value(&globals).unwrap()
    }
    
    #[wasm_bindgen(js_name = getDependencyGraph)]
    pub fn get_dependency_graph(&self) -> JsValue {
        let session = self.session.lock().unwrap();
        let graph = DependencyGraph {
            nodes: session.cells.keys().cloned().collect(),
            edges: session.def_graph.iter()
                .flat_map(|(cell, (deps, _))| {
                    deps.iter().filter_map(|def_id| {
                        session.globals.def_sources.get(def_id)
                            .map(|source| Edge { from: source.clone(), to: cell.clone() })
                    })
                })
                .collect(),
        };
        serde_wasm_bindgen::to_value(&graph).unwrap()
    }
    
    #[wasm_bindgen(js_name = getCellProvenance)]
    pub fn get_cell_provenance(&self, cell_id: &str) -> JsValue {
        let session = self.session.lock().unwrap();
        
        let (reads, writes) = session.def_graph.get(cell_id)
            .cloned()
            .unwrap_or_default();
        
        let provenance = CellProvenance {
            defines: writes.iter()
                .filter_map(|def_id| session.globals.def_to_name.get(def_id))
                .cloned()
                .collect(),
            depends_on: reads.iter()
                .filter_map(|def_id| session.globals.def_to_name.get(def_id))
                .cloned()
                .collect(),
            stale: session.stale_cells.contains(cell_id),
        };
        
        serde_wasm_bindgen::to_value(&provenance).unwrap()
    }
    
    #[wasm_bindgen(js_name = restartSession)]
    pub fn restart_session(&mut self) {
        let mut session = self.session.lock().unwrap();
        *session = SharedSession::new();
        self.cells.clear();
    }
    
    #[wasm_bindgen(js_name = getMemoryUsage)]
    pub fn get_memory_usage(&self) -> JsValue {
        let session = self.session.lock().unwrap();
        let usage = MemoryUsage {
            globals_bytes: session.globals.size_bytes(),
            checkpoints_count: session.checkpoints.len(),
            checkpoints_bytes: session.checkpoints.values()
                .map(|c| c.size_bytes())
                .sum(),
            total_allocated: wasm_bindgen::memory().buffer().byte_length(),
        };
        serde_wasm_bindgen::to_value(&usage).unwrap()
    }
}
```

