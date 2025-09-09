use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Cell provenance tracking for dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellProvenance {
    pub cell_id: String,
    pub depends_on: HashSet<String>,
    pub provides: HashSet<String>, // Variables/functions defined
    pub execution_order: usize,
    pub last_executed: Option<u64>,
}

/// Dependency graph for the notebook
#[derive(Debug, Default)]
pub struct DependencyGraph {
    pub cells: HashMap<String, CellProvenance>,
    pub next_execution_order: usize,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_cell(&mut self, cell_id: String, depends_on: HashSet<String>, provides: HashSet<String>) {
        let provenance = CellProvenance {
            cell_id: cell_id.clone(),
            depends_on,
            provides,
            execution_order: self.next_execution_order,
            last_executed: None,
        };
        self.next_execution_order += 1;
        self.cells.insert(cell_id, provenance);
    }
    
    pub fn get_execution_order(&self) -> Vec<String> {
        let mut cells: Vec<_> = self.cells.values().collect();
        cells.sort_by_key(|c| c.execution_order);
        cells.into_iter().map(|c| c.cell_id.clone()).collect()
    }
}