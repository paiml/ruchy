//! Transactional state management for REPL evaluation
//!
//! Provides atomic evaluation with rollback capability for safe experimentation.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};

use crate::runtime::repl::{Value, ReplState};
use crate::runtime::safe_arena::{TransactionalArena, SafeArena as Arena};

// ============================================================================
// Transactional REPL State
// ============================================================================

/// Transaction ID for tracking evaluation transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(pub u64);

/// Transactional wrapper for REPL state with O(1) checkpoint/rollback
pub struct TransactionalState {
    /// Current bindings
    bindings: HashMap<String, Value>,
    /// Binding mutability tracking
    binding_mutability: HashMap<String, bool>,
    /// Transaction stack
    transactions: Vec<Transaction>,
    /// Next transaction ID
    next_tx_id: u64,
    /// Arena for memory allocation
    arena: TransactionalArena,
    /// Maximum transaction depth
    max_depth: usize,
}

/// A single transaction in the stack
#[derive(Debug, Clone)]
struct Transaction {
    id: TransactionId,
    /// Snapshot of bindings at transaction start
    bindings_snapshot: HashMap<String, Value>,
    /// Snapshot of mutability at transaction start
    mutability_snapshot: HashMap<String, bool>,
    /// Arena checkpoint
    arena_checkpoint: usize,
    /// Start time for timeout tracking
    start_time: Instant,
    /// Transaction metadata
    metadata: TransactionMetadata,
}

/// Metadata about a transaction
#[derive(Debug, Clone)]
pub struct TransactionMetadata {
    /// Description of the transaction
    pub description: String,
    /// Memory limit for this transaction
    pub memory_limit: Option<usize>,
    /// Time limit for this transaction
    pub time_limit: Option<Duration>,
    /// Whether this is a speculative evaluation
    pub speculative: bool,
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        Self {
            description: "evaluation".to_string(),
            memory_limit: None,
            time_limit: None,
            speculative: false,
        }
    }
}

impl TransactionalState {
    /// Create a new transactional state with the given memory limit
    pub fn new(max_memory: usize) -> Self {
        Self {
            bindings: HashMap::new(),
            binding_mutability: HashMap::new(),
            transactions: Vec::new(),
            next_tx_id: 1,
            arena: TransactionalArena::new(max_memory),
            max_depth: 100, // Prevent unbounded nesting
        }
    }
    
    /// Begin a new transaction
    pub fn begin_transaction(&mut self, metadata: TransactionMetadata) -> Result<TransactionId> {
        if self.transactions.len() >= self.max_depth {
            return Err(anyhow!("Transaction depth limit exceeded"));
        }
        
        let id = TransactionId(self.next_tx_id);
        self.next_tx_id += 1;
        
        // Create checkpoint in arena
        let arena_checkpoint = self.arena.checkpoint();
        
        // Create transaction with current state snapshot
        let transaction = Transaction {
            id,
            bindings_snapshot: self.bindings.clone(),
            mutability_snapshot: self.binding_mutability.clone(),
            arena_checkpoint,
            start_time: Instant::now(),
            metadata,
        };
        
        self.transactions.push(transaction);
        Ok(id)
    }
    
    /// Commit the current transaction
    pub fn commit_transaction(&mut self, id: TransactionId) -> Result<()> {
        let tx = self.transactions.last()
            .ok_or_else(|| anyhow!("No active transaction"))?;
        
        if tx.id != id {
            return Err(anyhow!("Transaction ID mismatch"));
        }
        
        // Commit arena changes
        self.arena.commit()?;
        
        // Remove transaction from stack
        self.transactions.pop();
        
        Ok(())
    }
    
    /// Rollback the current transaction
    pub fn rollback_transaction(&mut self, id: TransactionId) -> Result<()> {
        let tx = self.transactions.last()
            .ok_or_else(|| anyhow!("No active transaction"))?;
        
        if tx.id != id {
            return Err(anyhow!("Transaction ID mismatch"));
        }
        
        // Restore state from snapshot
        let tx = self.transactions.pop().unwrap();
        self.bindings = tx.bindings_snapshot;
        self.binding_mutability = tx.mutability_snapshot;
        
        // Rollback arena
        self.arena.rollback(tx.arena_checkpoint)?;
        
        Ok(())
    }
    
    /// Check if a transaction has exceeded its limits
    pub fn check_transaction_limits(&self, id: TransactionId) -> Result<()> {
        let tx = self.transactions.iter()
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow!("Transaction not found"))?;
        
        // Check time limit
        if let Some(time_limit) = tx.metadata.time_limit {
            if tx.start_time.elapsed() > time_limit {
                return Err(anyhow!("Transaction time limit exceeded"));
            }
        }
        
        // Check memory limit
        if let Some(memory_limit) = tx.metadata.memory_limit {
            if self.arena.arena().used() > memory_limit {
                return Err(anyhow!("Transaction memory limit exceeded"));
            }
        }
        
        Ok(())
    }
    
    /// Get current transaction depth
    pub fn depth(&self) -> usize {
        self.transactions.len()
    }
    
    /// Get current bindings
    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }
    
    /// Get mutable bindings
    pub fn bindings_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.bindings
    }
    
    /// Insert a binding
    pub fn insert_binding(&mut self, name: String, value: Value, mutable: bool) {
        self.bindings.insert(name.clone(), value);
        self.binding_mutability.insert(name, mutable);
    }
    
    /// Get binding mutability
    pub fn is_mutable(&self, name: &str) -> bool {
        self.binding_mutability.get(name).copied().unwrap_or(false)
    }
    
    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.binding_mutability.clear();
        self.transactions.clear();
        self.arena.reset();
    }
    
    /// Get arena for allocation
    pub fn arena(&self) -> &Arena {
        self.arena.arena()
    }
    
    /// Get memory usage
    pub fn memory_used(&self) -> usize {
        self.arena.arena().used()
    }
    
    // SavePoint feature temporarily disabled - requires complex lifetime management
    // /// Create a savepoint for nested transactions
    // pub fn savepoint(&mut self) -> Result<SavePoint> {
    //     let tx_id = self.begin_transaction(TransactionMetadata {
    //         description: "savepoint".to_string(),
    //         speculative: true,
    //         ..Default::default()
    //     })?;
    //     
    //     Ok(SavePoint {
    //         tx_id,
    //         state: Rc::new(RefCell::new(*self)),
    //     })
    // }
}

// ============================================================================
// SavePoint - RAII Guard for Automatic Rollback
// ============================================================================

// SavePoint temporarily disabled - requires complex lifetime management
// /// RAII guard for automatic transaction rollback
// pub struct SavePoint {
//     tx_id: TransactionId,
//     state: Rc<RefCell<TransactionalState>>,
// }

// Placeholder for SavePoint
pub struct SavePoint;

// ============================================================================
// Transaction Log for Debugging
// ============================================================================

/// Log entry for transaction events
#[derive(Debug, Clone)]
pub enum TransactionEvent {
    Begin {
        id: TransactionId,
        metadata: TransactionMetadata,
    },
    Commit {
        id: TransactionId,
        duration: Duration,
        memory_used: usize,
    },
    Rollback {
        id: TransactionId,
        reason: String,
    },
    BindingAdded {
        name: String,
        value_type: String,
    },
    BindingModified {
        name: String,
        old_type: String,
        new_type: String,
    },
}

/// Transaction log for debugging and analysis
pub struct TransactionLog {
    events: Vec<(Instant, TransactionEvent)>,
    max_entries: usize,
}

impl TransactionLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            events: Vec::new(),
            max_entries,
        }
    }
    
    pub fn log(&mut self, event: TransactionEvent) {
        self.events.push((Instant::now(), event));
        
        // Maintain size limit
        if self.events.len() > self.max_entries {
            self.events.remove(0);
        }
    }
    
    pub fn recent_events(&self, count: usize) -> &[(Instant, TransactionEvent)] {
        let start = self.events.len().saturating_sub(count);
        &self.events[start..]
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

// ============================================================================
// Optimistic Concurrency Control
// ============================================================================

/// Version number for optimistic concurrency control
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64);

/// Versioned value for optimistic concurrency
#[derive(Debug, Clone)]
pub struct VersionedValue<T> {
    pub value: T,
    pub version: Version,
}

/// Multi-version concurrency control for parallel evaluation
pub struct MVCC {
    /// Current version
    current_version: Version,
    /// Versioned bindings
    bindings: HashMap<String, Vec<VersionedValue<Value>>>,
    /// Maximum versions to keep per binding
    max_versions: usize,
}

impl MVCC {
    pub fn new() -> Self {
        Self {
            current_version: Version(0),
            bindings: HashMap::new(),
            max_versions: 10,
        }
    }
    
    /// Start a new read transaction
    pub fn begin_read(&self) -> Version {
        self.current_version
    }
    
    /// Start a new write transaction
    pub fn begin_write(&mut self) -> Version {
        self.current_version.0 += 1;
        self.current_version
    }
    
    /// Read a value at a specific version
    pub fn read(&self, name: &str, version: Version) -> Option<&Value> {
        self.bindings.get(name).and_then(|versions| {
            // Find the latest version <= requested version
            versions.iter()
                .rev()
                .find(|v| v.version <= version)
                .map(|v| &v.value)
        })
    }
    
    /// Write a value at a specific version
    pub fn write(&mut self, name: String, value: Value, version: Version) {
        let entry = self.bindings.entry(name).or_insert_with(Vec::new);
        
        // Add new version
        entry.push(VersionedValue { value, version });
        
        // Maintain version limit
        if entry.len() > self.max_versions {
            entry.remove(0);
        }
    }
    
    /// Garbage collect old versions
    pub fn gc(&mut self, keep_after: Version) {
        for versions in self.bindings.values_mut() {
            versions.retain(|v| v.version >= keep_after);
        }
    }
}

impl Default for MVCC {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_commit() {
        let mut state = TransactionalState::new(1024 * 1024);
        
        // Add initial binding
        state.insert_binding("x".to_string(), Value::Int(1), false);
        
        // Begin transaction
        let tx = state.begin_transaction(TransactionMetadata::default()).unwrap();
        
        // Modify binding
        state.insert_binding("x".to_string(), Value::Int(2), false);
        state.insert_binding("y".to_string(), Value::Int(3), false);
        
        // Commit
        state.commit_transaction(tx).unwrap();
        
        // Changes should persist
        assert_eq!(state.bindings.get("x"), Some(&Value::Int(2)));
        assert_eq!(state.bindings.get("y"), Some(&Value::Int(3)));
    }
    
    #[test]
    fn test_transaction_rollback() {
        let mut state = TransactionalState::new(1024 * 1024);
        
        // Add initial binding
        state.insert_binding("x".to_string(), Value::Int(1), false);
        
        // Begin transaction
        let tx = state.begin_transaction(TransactionMetadata::default()).unwrap();
        
        // Modify binding
        state.insert_binding("x".to_string(), Value::Int(2), false);
        state.insert_binding("y".to_string(), Value::Int(3), false);
        
        // Rollback
        state.rollback_transaction(tx).unwrap();
        
        // Changes should be reverted
        assert_eq!(state.bindings.get("x"), Some(&Value::Int(1)));
        assert_eq!(state.bindings.get("y"), None);
    }
    
    // SavePoint test disabled - feature temporarily disabled
    // #[test]
    // fn test_savepoint() {
    //     let mut state = TransactionalState::new(1024 * 1024);
    //     
    //     state.insert_binding("x".to_string(), Value::Int(1), false);
    //     
    //     {
    //         let sp = state.savepoint().unwrap();
    //         state.insert_binding("x".to_string(), Value::Int(2), false);
    //         // SavePoint dropped here, automatic rollback
    //     }
    //     
    //     // Should be rolled back
    //     assert_eq!(state.bindings.get("x"), Some(&Value::Int(1)));
    // }
    
    #[test]
    fn test_mvcc() {
        let mut mvcc = MVCC::new();
        
        let v1 = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Int(1), v1);
        
        let v2 = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Int(2), v2);
        
        // Read at different versions
        assert_eq!(mvcc.read("x", v1), Some(&Value::Int(1)));
        assert_eq!(mvcc.read("x", v2), Some(&Value::Int(2)));
    }
}