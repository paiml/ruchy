//! SharedSession - Persistent state management for notebook cells
//!
//! This module implements the core abstraction for maintaining state across
//! notebook cell executions, solving the fundamental invariant violation where
//! each cell previously had an isolated REPL instance.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};
use crate::runtime::interpreter::{Interpreter, Value};
use crate::frontend::parser::Parser;
use crate::frontend::ast::Expr;

// ============================================================================
// Core Types
// ============================================================================

/// Unique identifier for variable/function definitions
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DefId(pub u64);

impl DefId {
    fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        DefId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Execution mode for notebook cells
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionMode {
    /// Traditional cell-by-cell execution
    Manual,
    /// Automatic dependency propagation
    Reactive,
}

/// Result of cell execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub success: bool,
    pub cell_id: String,
    pub value: String,
    pub result: String,
    pub error: Option<String>,
    pub execution_time_ms: f64,
}

impl ExecuteResponse {
    pub fn success(value: Value) -> Self {
        ExecuteResponse {
            success: true,
            cell_id: String::new(),
            value: format!("{:?}", value),
            result: format!("{:?}", value),
            error: None,
            execution_time_ms: 0.0,
        }
    }
    
    pub fn error(err: String) -> Self {
        ExecuteResponse {
            success: false,
            cell_id: String::new(),
            value: String::new(),
            result: String::new(),
            error: Some(err),
            execution_time_ms: 0.0,
        }
    }
}

/// Execution plan for reactive mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub primary: String,
    pub cascade: Vec<CascadeStep>,
    pub total_cells: usize,
    pub estimated_total_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeStep {
    pub cell_id: String,
    pub estimated_time: f64,
    pub dependencies: Option<HashSet<DefId>>,
    pub can_skip: bool,
    pub skipped: bool,
}

/// Edge in dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

/// Dependency graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<Edge>,
}

/// Cell provenance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellProvenance {
    pub defines: Vec<String>,
    pub depends_on: Vec<String>,
    pub stale: bool,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub globals_bytes: usize,
    pub checkpoints_count: usize,
    pub checkpoints_bytes: usize,
    pub total_allocated: u32,
}

// ============================================================================
// GlobalRegistry - Persistent namespace
// ============================================================================

/// Registry for persistent bindings across cells
#[derive(Debug, Clone)]
pub struct GlobalRegistry {
    /// Variable bindings with their definition IDs
    pub values: Arc<HashMap<String, (Value, DefId)>>,
    /// Function definitions
    pub functions: Arc<HashMap<String, String>>,
    /// Type information
    pub types: HashMap<String, String>,
    /// Imported modules
    pub imports: HashSet<String>,
    /// Symbol to cell mapping
    pub provenance: HashMap<String, String>,
    /// DefId to variable name mapping
    pub def_to_name: HashMap<DefId, String>,
    /// DefId to source cell mapping
    pub def_sources: HashMap<DefId, String>,
    /// Generation counter for versioning
    generation: u64,
}

impl GlobalRegistry {
    pub fn new() -> Self {
        GlobalRegistry {
            values: Arc::new(HashMap::new()),
            functions: Arc::new(HashMap::new()),
            types: HashMap::new(),
            imports: HashSet::new(),
            provenance: HashMap::new(),
            def_to_name: HashMap::new(),
            def_sources: HashMap::new(),
            generation: 0,
        }
    }
    
    pub fn store_value(&mut self, name: String, value: Value, cell_id: &str) -> DefId {
        let def_id = DefId::next();
        let values = Arc::make_mut(&mut self.values);
        values.insert(name.clone(), (value, def_id));
        
        self.def_to_name.insert(def_id, name.clone());
        self.def_sources.insert(def_id, cell_id.to_string());
        self.provenance.insert(name, cell_id.to_string());
        self.generation += 1;
        
        def_id
    }
    
    pub fn get_value(&self, name: &str) -> Option<Value> {
        self.values.get(name).map(|(v, _)| v.clone())
    }
    
    pub fn get_def_id(&self, name: &str) -> Option<DefId> {
        self.values.get(name).map(|(_, id)| *id)
    }
    
    pub fn cow_checkpoint(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            values: Arc::clone(&self.values),
            functions: Arc::clone(&self.functions),
            generation: self.generation,
        }
    }
    
    pub fn restore_cow(&mut self, snapshot: RegistrySnapshot) {
        self.values = snapshot.values;
        self.functions = snapshot.functions;
        self.generation = snapshot.generation;
    }
    
    pub fn size_bytes(&self) -> usize {
        // Approximate size calculation
        self.values.len() * 64 + self.functions.len() * 128
    }
    
    pub fn serialize_for_inspection(&self) -> serde_json::Value {
        serde_json::json!({
            "values": self.values.keys().cloned().collect::<Vec<_>>(),
            "functions": self.functions.keys().cloned().collect::<Vec<_>>(),
            "imports": self.imports.iter().cloned().collect::<Vec<_>>(),
            "generation": self.generation,
        })
    }
}

impl PartialEq for GlobalRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && 
        Arc::ptr_eq(&self.values, &other.values)
    }
}

/// Snapshot for COW checkpointing
#[derive(Debug, Clone)]
pub struct RegistrySnapshot {
    values: Arc<HashMap<String, (Value, DefId)>>,
    functions: Arc<HashMap<String, String>>,
    generation: u64,
}

// ============================================================================
// SharedSession - Core session manager
// ============================================================================

/// Persistent session that maintains state across cell executions
pub struct SharedSession {
    /// Single interpreter instance (not recreated per cell)
    pub interpreter: Interpreter,
    /// Global persistent namespace
    pub globals: GlobalRegistry,
    /// Dependency graph: cell -> (reads, writes)
    pub def_graph: HashMap<String, (HashSet<DefId>, HashSet<DefId>)>,
    /// Cells marked as stale
    pub stale_cells: HashSet<String>,
    /// Cell code cache for re-execution
    pub cell_cache: HashMap<String, String>,
    /// Checkpoints for rollback
    pub checkpoints: HashMap<String, RegistrySnapshot>,
    /// Current execution mode
    execution_mode: ExecutionMode,
    /// Whether to halt cascade on error
    halt_on_error: bool,
}

impl SharedSession {
    pub fn new() -> Self {
        SharedSession {
            interpreter: Interpreter::new(),
            globals: GlobalRegistry::new(),
            def_graph: HashMap::new(),
            stale_cells: HashSet::new(),
            cell_cache: HashMap::new(),
            checkpoints: HashMap::new(),
            execution_mode: ExecutionMode::Manual,
            halt_on_error: true,
        }
    }
    
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
    }
    
    /// Execute a cell with persistent state management
    pub fn execute(&mut self, cell_id: &str, code: &str) -> Result<ExecuteResponse, String> {
        let start = std::time::Instant::now();
        
        // Store code for potential re-execution
        self.cell_cache.insert(cell_id.to_string(), code.to_string());
        
        // Create COW checkpoint for rollback
        let snapshot = self.globals.cow_checkpoint();
        
        // Hydrate interpreter with global state
        self.hydrate_interpreter();
        
        // Track what this cell reads and writes
        let initial_defs = self.collect_current_defs();
        
        // Parse and execute
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;
        
        // Execute with interpreter
        let result = match self.interpreter.eval_expr(&expr) {
            Ok(value) => {
                // Extract new bindings and persist to globals
                let new_defs = self.extract_new_bindings(cell_id, &initial_defs);
                
                // Update dependency graph
                let reads = self.extract_reads(&expr, &initial_defs);
                let writes = new_defs.clone();
                self.def_graph.insert(cell_id.to_string(), (reads, writes.clone()));
                
                // Invalidate dependent cells
                self.invalidate_consumers(&writes);
                
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                Ok(ExecuteResponse {
                    success: true,
                    cell_id: cell_id.to_string(),
                    value: format!("{:?}", value),
                    result: format!("{:?}", value),
                    error: None,
                    execution_time_ms: elapsed,
                })
            }
            Err(err) => {
                // Rollback on error
                self.globals.restore_cow(snapshot);
                Err(format!("Execution error: {}", err))
            }
        };
        
        result
    }
    
    /// Execute cell in reactive mode with cascade
    pub fn execute_reactive(&mut self, cell_id: &str, code: &str) -> Vec<ExecuteResponse> {
        let mut responses = Vec::new();
        
        // Execute primary cell
        match self.execute(cell_id, code) {
            Ok(response) => {
                responses.push(response.clone());
                
                if response.success && self.execution_mode == ExecutionMode::Reactive {
                    // Find and execute dependent cells
                    let stale = self.find_stale_dependents(cell_id);
                    let order = self.topological_sort(&stale);
                    
                    for dependent_cell in order {
                        if let Some(cell_code) = self.cell_cache.get(&dependent_cell).cloned() {
                            match self.execute(&dependent_cell, &cell_code) {
                                Ok(dep_response) => {
                                    responses.push(dep_response.clone());
                                    if !dep_response.success && self.halt_on_error {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    responses.push(ExecuteResponse::error(e));
                                    if self.halt_on_error {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                responses.push(ExecuteResponse::error(e));
            }
        }
        
        responses
    }
    
    /// Generate execution plan without executing
    pub fn explain_reactive(&self, cell_id: &str) -> ExecutionPlan {
        let stale = self.find_stale_dependents(cell_id);
        let order = self.topological_sort(&stale);
        
        ExecutionPlan {
            primary: cell_id.to_string(),
            cascade: order.iter().map(|cell| {
                CascadeStep {
                    cell_id: cell.clone(),
                    estimated_time: self.estimate_execution_time(cell),
                    dependencies: self.def_graph.get(cell).map(|(d, _)| d.clone()),
                    can_skip: !self.is_critical(cell),
                    skipped: false,
                }
            }).collect(),
            total_cells: order.len() + 1,
            estimated_total_time: self.estimate_total_time(&order),
        }
    }
    
    /// Get dependencies for a cell
    pub fn get_dependencies(&self, cell_id: &str) -> HashSet<DefId> {
        self.def_graph.get(cell_id)
            .map(|(deps, _)| deps.clone())
            .unwrap_or_default()
    }
    
    /// Check for dependency cycles
    pub fn has_dependency_cycle(&self) -> bool {
        // Simple cycle detection using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for cell in self.def_graph.keys() {
            if !visited.contains(cell) {
                if self.has_cycle_dfs(cell, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }
    
    // ========================================================================
    // Private helper methods
    // ========================================================================
    
    fn hydrate_interpreter(&mut self) {
        // Load all global values into interpreter
        // Since env_stack is private, we'll need to work differently
        // For now, just store in our own globals
        // TODO: Add a public method to Interpreter to set bindings
    }
    
    fn collect_current_defs(&self) -> HashSet<DefId> {
        self.globals.values.values().map(|(_, id)| *id).collect()
    }
    
    fn extract_new_bindings(&mut self, cell_id: &str, initial_defs: &HashSet<DefId>) -> HashSet<DefId> {
        let mut new_defs = HashSet::new();
        
        // Find new bindings in interpreter
        // Since env_stack is private, we'll track our own bindings
        // TODO: Add a public method to Interpreter to get bindings
        let bindings: HashMap<String, Value> = HashMap::new();
        
        for (name, value) in bindings {
            if !self.globals.values.contains_key(&name) || 
               !initial_defs.contains(&self.globals.get_def_id(&name).unwrap_or(DefId(0))) {
                let def_id = self.globals.store_value(name, value, cell_id);
                new_defs.insert(def_id);
            }
        }
        
        new_defs
    }
    
    fn extract_reads(&self, _expr: &Expr, initial_defs: &HashSet<DefId>) -> HashSet<DefId> {
        // Simplified: return all existing defs as potential reads
        // Full implementation would walk AST to find variable references
        initial_defs.clone()
    }
    
    fn invalidate_consumers(&mut self, modified_defs: &HashSet<DefId>) {
        for (cell, (deps, _)) in &self.def_graph {
            if !deps.is_disjoint(modified_defs) {
                self.stale_cells.insert(cell.clone());
            }
        }
    }
    
    fn find_stale_dependents(&self, _cell_id: &str) -> HashSet<String> {
        self.stale_cells.clone()
    }
    
    fn find_dependents(&self, cell_id: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        
        if let Some((_, writes)) = self.def_graph.get(cell_id) {
            for (other_cell, (reads, _)) in &self.def_graph {
                if !reads.is_disjoint(writes) && other_cell != cell_id {
                    dependents.push(other_cell.clone());
                }
            }
        }
        
        dependents
    }
    
    fn topological_sort(&self, cells: &HashSet<String>) -> Vec<String> {
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut sorted = Vec::new();
        
        // Build in-degree map
        for cell in cells {
            let deps = self.def_graph.get(cell)
                .map(|(d, _)| d.len())
                .unwrap_or(0);
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
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }
        
        sorted
    }
    
    fn has_cycle_dfs(&self, cell: &str, visited: &mut HashSet<String>, rec_stack: &mut HashSet<String>) -> bool {
        visited.insert(cell.to_string());
        rec_stack.insert(cell.to_string());
        
        for dependent in self.find_dependents(cell) {
            if !visited.contains(&dependent) {
                if self.has_cycle_dfs(&dependent, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&dependent) {
                return true;
            }
        }
        
        rec_stack.remove(cell);
        false
    }
    
    
    fn estimate_execution_time(&self, _cell: &str) -> f64 {
        // Simplified: return constant estimate
        10.0 // milliseconds
    }
    
    fn estimate_total_time(&self, cells: &[String]) -> f64 {
        cells.len() as f64 * 10.0 + 10.0 // Primary + cascade
    }
    
    fn is_critical(&self, _cell: &str) -> bool {
        // Simplified: all cells are critical
        true
    }
}