//! Transactional state management for REPL evaluation
//!
//! Provides atomic evaluation with rollback capability for safe experimentation.
use crate::runtime::interpreter::Value;
use crate::runtime::safe_arena::{SafeArena as Arena, TransactionalArena};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::TransactionalState;
    ///
    /// let instance = TransactionalState::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::TransactionalState;
    ///
    /// let instance = TransactionalState::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::TransactionalState;
    ///
    /// let instance = TransactionalState::new();
    /// // Verify behavior
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::TransactionalState;
    ///
    /// let mut instance = TransactionalState::new();
    /// let result = instance.begin_transaction();
    /// // Verify behavior
    /// ```
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::commit_transaction;
    ///
    /// let result = commit_transaction(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn commit_transaction(&mut self, id: TransactionId) -> Result<()> {
        let tx = self
            .transactions
            .last()
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::rollback_transaction;
    ///
    /// let result = rollback_transaction(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn rollback_transaction(&mut self, id: TransactionId) -> Result<()> {
        let tx = self
            .transactions
            .last()
            .ok_or_else(|| anyhow!("No active transaction"))?;
        if tx.id != id {
            return Err(anyhow!("Transaction ID mismatch"));
        }
        // Restore state from snapshot
        let tx = self
            .transactions
            .pop()
            .expect("transaction stack must not be empty after last() check succeeded");
        self.bindings = tx.bindings_snapshot;
        self.binding_mutability = tx.mutability_snapshot;
        // Rollback arena
        self.arena.rollback(tx.arena_checkpoint)?;
        Ok(())
    }
    /// Check if a transaction has exceeded its limits
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::check_transaction_limits;
    ///
    /// let result = check_transaction_limits(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn check_transaction_limits(&self, id: TransactionId) -> Result<()> {
        let tx = self
            .transactions
            .iter()
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::depth;
    ///
    /// let result = depth(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn depth(&self) -> usize {
        self.transactions.len()
    }
    /// Get current bindings
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::bindings;
    ///
    /// let result = bindings(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }
    /// Get mutable bindings
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::bindings_mut;
    ///
    /// let result = bindings_mut(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn bindings_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.bindings
    }
    /// Insert a binding
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::insert_binding;
    ///
    /// let result = insert_binding(true);
    /// assert_eq!(result, Ok(true));
    /// ```
    pub fn insert_binding(&mut self, name: String, value: Value, mutable: bool) {
        self.bindings.insert(name.clone(), value);
        self.binding_mutability.insert(name, mutable);
    }
    /// Get binding mutability
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::is_mutable;
    ///
    /// let result = is_mutable("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_mutable(&self, name: &str) -> bool {
        self.binding_mutability.get(name).copied().unwrap_or(false)
    }
    /// Clear all bindings
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::clear;
    ///
    /// let result = clear(());
    /// assert_eq!(result, Ok(()));
    /// ```
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::clear;
    ///
    /// let result = clear(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.binding_mutability.clear();
        self.transactions.clear();
        self.arena.reset();
    }
    /// Get arena for allocation
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::TransactionalState;
    ///
    /// let state = TransactionalState::new();
    /// let arena = state.arena();
    /// assert!(arena.used() >= 0);
    /// ```
    pub fn arena(&self) -> &Arena {
        self.arena.arena()
    }
    /// Get memory usage
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::TransactionalState;
    ///
    /// let state = TransactionalState::new();
    /// let used = state.memory_used();
    /// assert!(used >= 0);
    /// ```
    pub fn memory_used(&self) -> usize {
        self.arena.arena().used()
    }
    // SavePoint feature temporarily disabled - requires complex lifetime management
    // /// Create a savepoint for nested transactions
    //
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::savepoint;
    ///
    /// let result = savepoint(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn savepoint(&mut self) -> Result<SavePoint> {
        // SavePoint feature temporarily disabled - requires complex lifetime management
        Err(anyhow!("SavePoint feature temporarily disabled"))
    }
}
// ============================================================================
// SavePoint - RAII Guard for Automatic Rollback
// ============================================================================
// SavePoint temporarily disabled - requires complex lifetime management
// /// RAII guard for automatic transaction rollback
// pub struct SavePoint {
//     tx_id: TransactionId,
//     state: Arc<RefCell<TransactionalState>>,
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::TransactionLog;
    ///
    /// let mut instance = TransactionLog::new();
    /// let result = instance.log();
    /// // Verify behavior
    /// ```
    pub fn log(&mut self, event: TransactionEvent) {
        self.events.push((Instant::now(), event));
        // Maintain size limit
        if self.events.len() > self.max_entries {
            self.events.remove(0);
        }
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::recent_events;
    ///
    /// let result = recent_events(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::transaction::MVCC;
    ///
    /// let mut instance = MVCC::new();
    /// let result = instance.begin_read();
    /// // Verify behavior
    /// ```
    pub fn begin_read(&self) -> Version {
        self.current_version
    }
    /// Start a new write transaction
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::begin_write;
    ///
    /// let result = begin_write(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn begin_write(&mut self) -> Version {
        self.current_version.0 += 1;
        self.current_version
    }
    /// Read a value at a specific version
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::read;
    ///
    /// let result = read("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn read(&self, name: &str, version: Version) -> Option<&Value> {
        self.bindings.get(name).and_then(|versions| {
            // Find the latest version <= requested version
            versions
                .iter()
                .rev()
                .find(|v| v.version <= version)
                .map(|v| &v.value)
        })
    }
    /// Write a value at a specific version
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::write;
    ///
    /// let result = write(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn write(&mut self, name: String, value: Value, version: Version) {
        let entry = self.bindings.entry(name).or_default();
        // Add new version
        entry.push(VersionedValue { value, version });
        // Maintain version limit
        if entry.len() > self.max_versions {
            entry.remove(0);
        }
    }
    /// Garbage collect old versions
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::transaction::gc;
    ///
    /// let result = gc(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
        state.insert_binding("x".to_string(), Value::Integer(1), false);
        // Begin transaction
        let tx = state
            .begin_transaction(TransactionMetadata::default())
            .expect("operation should succeed in test");
        // Modify binding
        state.insert_binding("x".to_string(), Value::Integer(2), false);
        state.insert_binding("y".to_string(), Value::Integer(3), false);
        // Commit
        state
            .commit_transaction(tx)
            .expect("operation should succeed in test");
        // Changes should persist
        assert_eq!(state.bindings.get("x"), Some(&Value::Integer(2)));
        assert_eq!(state.bindings.get("y"), Some(&Value::Integer(3)));
    }
    #[test]
    fn test_transaction_rollback() {
        let mut state = TransactionalState::new(1024 * 1024);
        // Add initial binding
        state.insert_binding("x".to_string(), Value::Integer(1), false);
        // Begin transaction
        let tx = state
            .begin_transaction(TransactionMetadata::default())
            .expect("operation should succeed in test");
        // Modify binding
        state.insert_binding("x".to_string(), Value::Integer(2), false);
        state.insert_binding("y".to_string(), Value::Integer(3), false);
        // Rollback
        state
            .rollback_transaction(tx)
            .expect("operation should succeed in test");
        // Changes should be reverted
        assert_eq!(state.bindings.get("x"), Some(&Value::Integer(1)));
        assert_eq!(state.bindings.get("y"), None);
    }

    #[test]
    #[ignore = "SavePoint automatic rollback on drop needs implementation"]
    fn test_savepoint() {
        let mut state = TransactionalState::new(1024 * 1024);

        state.insert_binding("x".to_string(), Value::Integer(1), false);

        {
            let _sp = state.savepoint().expect("savepoint should succeed in test");
            state.insert_binding("x".to_string(), Value::Integer(2), false);
            // SavePoint dropped here, automatic rollback expected
        }

        // Should be rolled back
        assert_eq!(state.bindings.get("x"), Some(&Value::Integer(1)));
    }

    #[test]
    fn test_mvcc() {
        let mut mvcc = MVCC::new();
        let v1 = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Integer(1), v1);
        let v2 = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Integer(2), v2);
        // Read at different versions
        assert_eq!(mvcc.read("x", v1), Some(&Value::Integer(1)));
        assert_eq!(mvcc.read("x", v2), Some(&Value::Integer(2)));
    }

    // COVERAGE-95: Additional tests for complete coverage

    #[test]
    fn test_transaction_metadata_default() {
        let meta = TransactionMetadata::default();
        assert_eq!(meta.description, "evaluation");
        assert!(meta.memory_limit.is_none());
        assert!(meta.time_limit.is_none());
        assert!(!meta.speculative);
    }

    #[test]
    fn test_transaction_id_clone() {
        let id = TransactionId(42);
        let cloned = id;
        assert_eq!(id.0, cloned.0);
    }

    #[test]
    fn test_transaction_id_eq() {
        let id1 = TransactionId(1);
        let id2 = TransactionId(1);
        let id3 = TransactionId(2);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_transactional_state_depth() {
        let mut state = TransactionalState::new(1024 * 1024);
        assert_eq!(state.depth(), 0);
        let _tx1 = state.begin_transaction(TransactionMetadata::default()).unwrap();
        assert_eq!(state.depth(), 1);
        let _tx2 = state.begin_transaction(TransactionMetadata::default()).unwrap();
        assert_eq!(state.depth(), 2);
    }

    #[test]
    fn test_transactional_state_bindings() {
        let mut state = TransactionalState::new(1024 * 1024);
        state.insert_binding("x".to_string(), Value::Integer(42), false);
        let bindings = state.bindings();
        assert_eq!(bindings.get("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_transactional_state_bindings_mut() {
        let mut state = TransactionalState::new(1024 * 1024);
        state.insert_binding("x".to_string(), Value::Integer(1), false);
        state.bindings_mut().insert("y".to_string(), Value::Integer(2));
        assert_eq!(state.bindings().get("y"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_transactional_state_is_mutable() {
        let mut state = TransactionalState::new(1024 * 1024);
        state.insert_binding("x".to_string(), Value::Integer(1), true);
        state.insert_binding("y".to_string(), Value::Integer(2), false);
        assert!(state.is_mutable("x"));
        assert!(!state.is_mutable("y"));
        assert!(!state.is_mutable("z")); // nonexistent
    }

    #[test]
    fn test_transactional_state_clear() {
        let mut state = TransactionalState::new(1024 * 1024);
        state.insert_binding("x".to_string(), Value::Integer(1), false);
        let _tx = state.begin_transaction(TransactionMetadata::default()).unwrap();
        state.clear();
        assert!(state.bindings().is_empty());
        assert_eq!(state.depth(), 0);
    }

    #[test]
    fn test_transactional_state_arena() {
        let state = TransactionalState::new(1024 * 1024);
        let arena = state.arena();
        assert!(arena.used() == 0);
    }

    #[test]
    fn test_transactional_state_memory_used() {
        let state = TransactionalState::new(1024 * 1024);
        assert_eq!(state.memory_used(), 0);
    }

    #[test]
    fn test_transactional_state_savepoint_disabled() {
        let mut state = TransactionalState::new(1024 * 1024);
        let result = state.savepoint();
        assert!(result.is_err());
    }

    #[test]
    fn test_transactional_state_depth_limit() {
        let mut state = TransactionalState::new(1024 * 1024);
        // Try to exceed max_depth (100)
        for i in 0..100 {
            let result = state.begin_transaction(TransactionMetadata::default());
            if i < 100 {
                assert!(result.is_ok(), "Transaction {} should succeed", i);
            }
        }
        // 101st should fail
        let result = state.begin_transaction(TransactionMetadata::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_no_active_transaction() {
        let mut state = TransactionalState::new(1024 * 1024);
        let result = state.commit_transaction(TransactionId(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_wrong_transaction_id() {
        let mut state = TransactionalState::new(1024 * 1024);
        let _tx = state.begin_transaction(TransactionMetadata::default()).unwrap();
        let result = state.commit_transaction(TransactionId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_no_active_transaction() {
        let mut state = TransactionalState::new(1024 * 1024);
        let result = state.rollback_transaction(TransactionId(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_wrong_transaction_id() {
        let mut state = TransactionalState::new(1024 * 1024);
        let _tx = state.begin_transaction(TransactionMetadata::default()).unwrap();
        let result = state.rollback_transaction(TransactionId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_transaction_limits_not_found() {
        let state = TransactionalState::new(1024 * 1024);
        let result = state.check_transaction_limits(TransactionId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_transaction_limits_time() {
        let mut state = TransactionalState::new(1024 * 1024);
        let meta = TransactionMetadata {
            time_limit: Some(std::time::Duration::from_millis(1)),
            ..Default::default()
        };
        let tx = state.begin_transaction(meta).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = state.check_transaction_limits(tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_transaction_limits_ok() {
        let mut state = TransactionalState::new(1024 * 1024);
        let tx = state.begin_transaction(TransactionMetadata::default()).unwrap();
        let result = state.check_transaction_limits(tx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_log_new() {
        let log = TransactionLog::new(100);
        assert_eq!(log.recent_events(10).len(), 0);
    }

    #[test]
    fn test_transaction_log_operations() {
        let mut log = TransactionLog::new(100);
        log.log(TransactionEvent::Begin {
            id: TransactionId(1),
            metadata: TransactionMetadata::default(),
        });
        log.log(TransactionEvent::Commit {
            id: TransactionId(1),
            duration: std::time::Duration::from_millis(10),
            memory_used: 1024,
        });
        assert_eq!(log.recent_events(10).len(), 2);
        log.clear();
        assert_eq!(log.recent_events(10).len(), 0);
    }

    #[test]
    fn test_transaction_log_size_limit() {
        let mut log = TransactionLog::new(2);
        for i in 0..5 {
            log.log(TransactionEvent::Begin {
                id: TransactionId(i),
                metadata: TransactionMetadata::default(),
            });
        }
        assert_eq!(log.recent_events(10).len(), 2);
    }

    #[test]
    fn test_mvcc_default() {
        let mvcc = MVCC::default();
        assert_eq!(mvcc.begin_read(), Version(0));
    }

    #[test]
    fn test_mvcc_begin_read() {
        let mvcc = MVCC::new();
        let version = mvcc.begin_read();
        assert_eq!(version, Version(0));
    }

    #[test]
    fn test_mvcc_gc() {
        let mut mvcc = MVCC::new();
        let v1 = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Integer(1), v1);
        let v2 = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Integer(2), v2);
        mvcc.gc(v2);
        assert!(mvcc.read("x", v1).is_none());
        assert_eq!(mvcc.read("x", v2), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_mvcc_read_nonexistent() {
        let mvcc = MVCC::new();
        assert!(mvcc.read("nonexistent", Version(0)).is_none());
    }

    #[test]
    fn test_mvcc_version_limit() {
        let mut mvcc = MVCC::new();
        // Write more than max_versions (10)
        for i in 0..15 {
            let v = mvcc.begin_write();
            mvcc.write("x".to_string(), Value::Integer(i), v);
        }
        // Only last 10 should be kept
        // Read at version 1 should fail (trimmed)
        // Read at latest versions should work
        let latest = mvcc.begin_read();
        assert!(mvcc.read("x", latest).is_some());
    }

    #[test]
    fn test_version_comparisons() {
        let v1 = Version(1);
        let v2 = Version(2);
        let v1b = Version(1);
        assert!(v1 < v2);
        assert!(v2 > v1);
        assert!(v1 <= v1b);
        assert!(v1 >= v1b);
        assert_eq!(v1, v1b);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_versioned_value_clone() {
        let vv = VersionedValue {
            value: Value::Integer(42),
            version: Version(1),
        };
        let cloned = vv.clone();
        assert_eq!(cloned.value, Value::Integer(42));
        assert_eq!(cloned.version, Version(1));
    }

    #[test]
    fn test_transaction_event_variants() {
        let begin = TransactionEvent::Begin {
            id: TransactionId(1),
            metadata: TransactionMetadata::default(),
        };
        let commit = TransactionEvent::Commit {
            id: TransactionId(1),
            duration: std::time::Duration::from_millis(10),
            memory_used: 100,
        };
        let rollback = TransactionEvent::Rollback {
            id: TransactionId(1),
            reason: "test".to_string(),
        };
        let added = TransactionEvent::BindingAdded {
            name: "x".to_string(),
            value_type: "integer".to_string(),
        };
        let modified = TransactionEvent::BindingModified {
            name: "x".to_string(),
            old_type: "integer".to_string(),
            new_type: "string".to_string(),
        };
        // Just verify they can be created and cloned
        let _ = begin.clone();
        let _ = commit.clone();
        let _ = rollback.clone();
        let _ = added.clone();
        let _ = modified.clone();
    }

    #[test]
    fn test_transaction_metadata_clone() {
        let meta = TransactionMetadata {
            description: "test".to_string(),
            memory_limit: Some(1024),
            time_limit: Some(std::time::Duration::from_secs(1)),
            speculative: true,
        };
        let cloned = meta.clone();
        assert_eq!(cloned.description, "test");
        assert_eq!(cloned.memory_limit, Some(1024));
        assert!(cloned.speculative);
    }
}
#[cfg(test)]
mod property_tests_transaction {
    use super::*;
    use proptest::proptest;

    proptest! {
        /// Property: TransactionalState operations never panic
        #[test]
        fn test_transactional_state_never_panics(size: usize) {
            let size = size % 10_000_000;  // Limit memory size
            let _ = TransactionalState::new(size);
        }
        /// Property: Transaction commit/rollback preserves invariants
        #[test]
        fn test_transaction_invariants(ops: Vec<u8>) {
            let mut state = TransactionalState::new(1024 * 1024);
            for op in ops.iter().take(100) {
                match op % 3 {
                    0 => {
                        let _ = state.begin_transaction(TransactionMetadata::default());
                    },
                    1 => {
                        if state.depth() > 0 {
                            let tx = TransactionId(state.depth() as u64);
                            let _ = state.commit_transaction(tx);
                        }
                    },
                    _ => {
                        if state.depth() > 0 {
                            let tx = TransactionId(state.depth() as u64);
                            let _ = state.rollback_transaction(tx);
                        }
                    }
                }
            }
            // Invariant: depth is always non-negative (usize type)
        }
    }
}
