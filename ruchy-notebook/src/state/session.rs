use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Session state for a notebook execution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub created_at: u64, // Unix timestamp
    pub last_accessed: u64,
    pub execution_count: usize,
    pub cell_execution_order: Vec<String>,
}

/// Execution context for a single cell
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub cell_id: String,
    pub session_id: String,
    pub execution_number: usize,
    pub start_time: SystemTime,
    pub timeout: Option<Duration>,
    pub max_memory: Option<usize>,
}

impl SessionState {
    pub fn new(session_id: String) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            session_id,
            created_at: now,
            last_accessed: now,
            execution_count: 0,
            cell_execution_order: Vec::new(),
        }
    }
    
    pub fn execute_cell(&mut self, cell_id: String) {
        self.execution_count += 1;
        self.last_accessed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.cell_execution_order.push(cell_id);
    }
}

impl ExecutionContext {
    pub fn new(cell_id: String, session_id: String, execution_number: usize) -> Self {
        Self {
            cell_id,
            session_id,
            execution_number,
            start_time: SystemTime::now(),
            timeout: Some(Duration::from_secs(30)), // Default 30s timeout
            max_memory: Some(256 * 1024 * 1024), // Default 256MB limit
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        SystemTime::now().duration_since(self.start_time).unwrap_or_default()
    }
    
    pub fn is_timeout(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.elapsed() > timeout
        } else {
            false
        }
    }
}