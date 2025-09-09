//! TDD tests for runtime/transaction.rs - achieving 90%+ coverage
//! QDD Metrics Target:
//! - Line Coverage: ≥90%
//! - Branch Coverage: ≥85%
//! - All public APIs: 100%

use ruchy::runtime::transaction::*;
use ruchy::runtime::repl::Value;
use std::time::Duration;

// ============================================================================
// Core Data Structure Tests
// ============================================================================

#[test]
fn test_transaction_id_creation() {
    let id1 = TransactionId(1);
    let id2 = TransactionId(2);
    assert_eq!(id1, TransactionId(1));
    assert_ne!(id1, id2);
}

#[test]
fn test_transaction_id_equality() {
    let id1 = TransactionId(42);
    let id2 = TransactionId(42);
    let id3 = TransactionId(43);
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_transaction_metadata_default() {
    let metadata = TransactionMetadata::default();
    assert_eq!(metadata.description, "evaluation");
    assert!(metadata.memory_limit.is_none());
    assert!(metadata.time_limit.is_none());
    assert!(!metadata.speculative);
}

#[test]
fn test_transaction_metadata_custom() {
    let metadata = TransactionMetadata {
        description: "test transaction".to_string(),
        memory_limit: Some(1024 * 1024),
        time_limit: Some(Duration::from_secs(5)),
        speculative: true,
    };
    assert_eq!(metadata.description, "test transaction");
    assert_eq!(metadata.memory_limit, Some(1024 * 1024));
    assert_eq!(metadata.time_limit, Some(Duration::from_secs(5)));
    assert!(metadata.speculative);
}

// ============================================================================
// TransactionalState Tests
// ============================================================================

#[test]
fn test_transactional_state_new() {
    let state = TransactionalState::new(1024 * 1024);
    assert_eq!(state.depth(), 0);
    assert!(state.bindings().is_empty());
    assert_eq!(state.memory_used(), 0);
}

#[test]
fn test_begin_transaction() {
    let mut state = TransactionalState::new(1024 * 1024);
    let metadata = TransactionMetadata::default();
    
    let tx_id = state.begin_transaction(metadata).unwrap();
    assert_eq!(tx_id, TransactionId(1));
    assert_eq!(state.depth(), 1);
}

#[test]
fn test_begin_multiple_transactions() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let tx1 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    let tx2 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    
    assert_eq!(tx1, TransactionId(1));
    assert_eq!(tx2, TransactionId(2));
    assert_eq!(state.depth(), 2);
}

#[test]
fn test_commit_transaction() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let tx_id = state.begin_transaction(TransactionMetadata::default()).unwrap();
    state.insert_binding("x".to_string(), Value::Int(42), false);
    
    assert_eq!(state.depth(), 1);
    state.commit_transaction(tx_id).unwrap();
    assert_eq!(state.depth(), 0);
    
    // Binding should be preserved after commit
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(42)));
}

#[test]
fn test_rollback_transaction() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Initial binding
    state.insert_binding("x".to_string(), Value::Int(10), false);
    
    // Begin transaction and modify
    let tx_id = state.begin_transaction(TransactionMetadata::default()).unwrap();
    state.insert_binding("x".to_string(), Value::Int(20), false);
    state.insert_binding("y".to_string(), Value::Int(30), false);
    
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(20)));
    assert_eq!(state.bindings().get("y"), Some(&Value::Int(30)));
    
    // Rollback
    state.rollback_transaction(tx_id).unwrap();
    
    // Should restore original state
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(10)));
    assert!(state.bindings().get("y").is_none());
    assert_eq!(state.depth(), 0);
}

#[test]
fn test_nested_transactions() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(1), false);
    
    let tx1 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    state.insert_binding("x".to_string(), Value::Int(2), false);
    
    let tx2 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    state.insert_binding("x".to_string(), Value::Int(3), false);
    
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(3)));
    
    // Rollback inner transaction
    state.rollback_transaction(tx2).unwrap();
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(2)));
    
    // Commit outer transaction
    state.commit_transaction(tx1).unwrap();
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(2)));
}

#[test]
fn test_transaction_depth_limit() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Fill up to max depth (100)
    for _ in 0..100 {
        state.begin_transaction(TransactionMetadata::default()).unwrap();
    }
    
    assert_eq!(state.depth(), 100);
    
    // Next one should fail
    let result = state.begin_transaction(TransactionMetadata::default());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("depth limit"));
}

#[test]
fn test_commit_no_transaction() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let result = state.commit_transaction(TransactionId(1));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No active transaction"));
}

#[test]
fn test_commit_wrong_transaction_id() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let _tx1 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    let result = state.commit_transaction(TransactionId(999));
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("ID mismatch"));
}

#[test]
fn test_rollback_no_transaction() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let result = state.rollback_transaction(TransactionId(1));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No active transaction"));
}

#[test]
fn test_rollback_wrong_transaction_id() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let _tx1 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    let result = state.rollback_transaction(TransactionId(999));
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("ID mismatch"));
}

// ============================================================================
// Binding Management Tests
// ============================================================================

#[test]
fn test_insert_binding() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(42), false);
    state.insert_binding("y".to_string(), Value::String("hello".to_string()), true);
    
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(42)));
    assert_eq!(state.bindings().get("y"), Some(&Value::String("hello".to_string())));
    assert!(!state.is_mutable("x"));
    assert!(state.is_mutable("y"));
}

#[test]
fn test_binding_mutability() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("immut".to_string(), Value::Int(1), false);
    state.insert_binding("mut".to_string(), Value::Int(2), true);
    
    assert!(!state.is_mutable("immut"));
    assert!(state.is_mutable("mut"));
    assert!(!state.is_mutable("nonexistent"));
}

#[test]
fn test_bindings_mut() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(1), false);
    
    // Direct mutation through bindings_mut
    state.bindings_mut().insert("y".to_string(), Value::Int(2));
    
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(1)));
    assert_eq!(state.bindings().get("y"), Some(&Value::Int(2)));
}

#[test]
fn test_clear_state() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(1), false);
    state.insert_binding("y".to_string(), Value::Int(2), true);
    let _tx = state.begin_transaction(TransactionMetadata::default());
    
    state.clear();
    
    assert!(state.bindings().is_empty());
    assert!(!state.is_mutable("x"));
    assert!(!state.is_mutable("y"));
    assert_eq!(state.depth(), 0);
}

// ============================================================================
// Transaction Limits Tests
// ============================================================================

#[test]
fn test_check_transaction_limits_not_found() {
    let state = TransactionalState::new(1024 * 1024);
    
    let result = state.check_transaction_limits(TransactionId(999));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_check_transaction_limits_no_limits() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let tx_id = state.begin_transaction(TransactionMetadata::default()).unwrap();
    let result = state.check_transaction_limits(tx_id);
    assert!(result.is_ok());
}

#[test]
fn test_check_memory_limit() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let metadata = TransactionMetadata {
        description: "memory limited".to_string(),
        memory_limit: Some(100), // Very small limit
        time_limit: None,
        speculative: false,
    };
    
    let tx_id = state.begin_transaction(metadata).unwrap();
    
    // Initially should be OK
    let result = state.check_transaction_limits(tx_id);
    assert!(result.is_ok());
    
    // Note: Actually exceeding memory limit would require allocating in the arena
    // which is more complex to test here
}

#[test]
fn test_check_time_limit() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    let metadata = TransactionMetadata {
        description: "time limited".to_string(),
        memory_limit: None,
        time_limit: Some(Duration::from_millis(1)), // Very short limit
        speculative: false,
    };
    
    let tx_id = state.begin_transaction(metadata).unwrap();
    
    // Sleep to exceed time limit
    std::thread::sleep(Duration::from_millis(2));
    
    let result = state.check_transaction_limits(tx_id);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("time limit exceeded"));
}

// ============================================================================
// Arena Integration Tests
// ============================================================================

#[test]
fn test_arena_access() {
    let state = TransactionalState::new(1024 * 1024);
    let arena = state.arena();
    assert_eq!(arena.used(), 0);
}

#[test]
fn test_memory_used() {
    let state = TransactionalState::new(1024 * 1024);
    assert_eq!(state.memory_used(), 0);
}

// ============================================================================
// Transaction Event Tests
// ============================================================================

#[test]
fn test_transaction_event_begin() {
    let event = TransactionEvent::Begin {
        id: TransactionId(1),
        metadata: TransactionMetadata::default(),
    };
    
    match event {
        TransactionEvent::Begin { id, metadata } => {
            assert_eq!(id, TransactionId(1));
            assert_eq!(metadata.description, "evaluation");
        }
        _ => panic!("Expected Begin event"),
    }
}

#[test]
fn test_transaction_event_commit() {
    let event = TransactionEvent::Commit {
        id: TransactionId(1),
        duration: Duration::from_secs(1),
        memory_used: 1024,
    };
    
    match event {
        TransactionEvent::Commit { id, duration, memory_used } => {
            assert_eq!(id, TransactionId(1));
            assert_eq!(duration, Duration::from_secs(1));
            assert_eq!(memory_used, 1024);
        }
        _ => panic!("Expected Commit event"),
    }
}

#[test]
fn test_transaction_event_rollback() {
    let event = TransactionEvent::Rollback {
        id: TransactionId(1),
        reason: "test rollback".to_string(),
    };
    
    match event {
        TransactionEvent::Rollback { id, reason } => {
            assert_eq!(id, TransactionId(1));
            assert_eq!(reason, "test rollback");
        }
        _ => panic!("Expected Rollback event"),
    }
}

#[test]
fn test_transaction_event_binding_added() {
    let event = TransactionEvent::BindingAdded {
        name: "x".to_string(),
        value_type: "Int".to_string(),
    };
    
    match event {
        TransactionEvent::BindingAdded { name, value_type } => {
            assert_eq!(name, "x");
            assert_eq!(value_type, "Int");
        }
        _ => panic!("Expected BindingAdded event"),
    }
}

#[test]
fn test_transaction_event_binding_modified() {
    let event = TransactionEvent::BindingModified {
        name: "x".to_string(),
        old_type: "Int".to_string(),
        new_type: "String".to_string(),
    };
    
    match event {
        TransactionEvent::BindingModified { name, old_type, new_type } => {
            assert_eq!(name, "x");
            assert_eq!(old_type, "Int");
            assert_eq!(new_type, "String");
        }
        _ => panic!("Expected BindingModified event"),
    }
}

// ============================================================================
// Transaction Log Tests
// ============================================================================

#[test]
fn test_transaction_log_new() {
    let log = TransactionLog::new(100);
    assert_eq!(log.recent_events(10).len(), 0);
}

#[test]
fn test_transaction_log_log_event() {
    let mut log = TransactionLog::new(100);
    
    log.log(TransactionEvent::Begin {
        id: TransactionId(1),
        metadata: TransactionMetadata::default(),
    });
    
    log.log(TransactionEvent::Commit {
        id: TransactionId(1),
        duration: Duration::from_millis(100),
        memory_used: 512,
    });
    
    let recent = log.recent_events(10);
    assert_eq!(recent.len(), 2);
}

#[test]
fn test_transaction_log_max_entries() {
    let mut log = TransactionLog::new(2);
    
    log.log(TransactionEvent::Begin {
        id: TransactionId(1),
        metadata: TransactionMetadata::default(),
    });
    
    log.log(TransactionEvent::Commit {
        id: TransactionId(1),
        duration: Duration::from_millis(100),
        memory_used: 512,
    });
    
    log.log(TransactionEvent::Begin {
        id: TransactionId(2),
        metadata: TransactionMetadata::default(),
    });
    
    // Should only keep last 2 entries
    let recent = log.recent_events(10);
    assert_eq!(recent.len(), 2);
}

#[test]
fn test_transaction_log_clear() {
    let mut log = TransactionLog::new(100);
    
    log.log(TransactionEvent::Begin {
        id: TransactionId(1),
        metadata: TransactionMetadata::default(),
    });
    
    assert!(!log.recent_events(10).is_empty());
    
    log.clear();
    assert!(log.recent_events(10).is_empty());
}

#[test]
fn test_transaction_log_recent_events() {
    let mut log = TransactionLog::new(100);
    
    for i in 0..5 {
        log.log(TransactionEvent::Begin {
            id: TransactionId(i),
            metadata: TransactionMetadata::default(),
        });
    }
    
    assert_eq!(log.recent_events(3).len(), 3);
    assert_eq!(log.recent_events(10).len(), 5);
    assert_eq!(log.recent_events(0).len(), 0);
}

// ============================================================================
// MVCC Tests
// ============================================================================

#[test]
fn test_mvcc_new() {
    let mvcc = MVCC::new();
    // Can't test begin_read directly as Version constructor is private
    // Just test that we can create an MVCC instance
    let _ = mvcc.begin_read();
}

#[test]
fn test_mvcc_begin_write() {
    let mut mvcc = MVCC::new();
    let v1 = mvcc.begin_write();
    let v2 = mvcc.begin_write();
    // Versions should be different
    assert_ne!(v1, v2);
}

#[test]
fn test_mvcc_write_and_read() {
    let mut mvcc = MVCC::new();
    
    let write_version = mvcc.begin_write();
    mvcc.write("x".to_string(), Value::Int(42), write_version);
    
    let value = mvcc.read("x", write_version);
    assert_eq!(value, Some(&Value::Int(42)));
}

#[test]
fn test_mvcc_read_nonexistent() {
    let mvcc = MVCC::new();
    let read_version = mvcc.begin_read();
    let value = mvcc.read("nonexistent", read_version);
    assert!(value.is_none());
}

#[test]
fn test_mvcc_multiple_versions() {
    let mut mvcc = MVCC::new();
    
    let v1 = mvcc.begin_write();
    mvcc.write("x".to_string(), Value::Int(1), v1);
    
    let v2 = mvcc.begin_write();
    mvcc.write("x".to_string(), Value::Int(2), v2);
    
    // Should be able to read both versions
    assert_eq!(mvcc.read("x", v1), Some(&Value::Int(1)));
    assert_eq!(mvcc.read("x", v2), Some(&Value::Int(2)));
}

#[test]
fn test_mvcc_garbage_collection() {
    let mut mvcc = MVCC::new();
    
    // Create many versions
    let mut versions = Vec::new();
    for i in 0..15 {
        let v = mvcc.begin_write();
        mvcc.write("x".to_string(), Value::Int(i), v);
        versions.push(v);
    }
    
    // Garbage collect old versions
    if versions.len() > 5 {
        mvcc.gc(versions[5]);
        
        // Old versions should be gone
        assert!(mvcc.read("x", versions[0]).is_none());
        assert!(mvcc.read("x", versions[4]).is_none());
        
        // Recent versions should still exist
        assert!(mvcc.read("x", versions[6]).is_some());
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_transactional_workflow() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Initial state
    state.insert_binding("counter".to_string(), Value::Int(0), true);
    
    // Transaction 1: Increment counter
    let tx1 = state.begin_transaction(TransactionMetadata {
        description: "increment".to_string(),
        ..Default::default()
    }).unwrap();
    
    state.insert_binding("counter".to_string(), Value::Int(1), true);
    state.commit_transaction(tx1).unwrap();
    
    assert_eq!(state.bindings().get("counter"), Some(&Value::Int(1)));
    
    // Transaction 2: Try increment but rollback
    let tx2 = state.begin_transaction(TransactionMetadata {
        description: "failed increment".to_string(),
        ..Default::default()
    }).unwrap();
    
    state.insert_binding("counter".to_string(), Value::Int(2), true);
    state.insert_binding("temp".to_string(), Value::String("test".to_string()), false);
    
    state.rollback_transaction(tx2).unwrap();
    
    // Should be back to committed state
    assert_eq!(state.bindings().get("counter"), Some(&Value::Int(1)));
    assert!(state.bindings().get("temp").is_none());
}

#[test]
fn test_speculative_evaluation() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(10), false);
    
    // Begin speculative transaction
    let tx = state.begin_transaction(TransactionMetadata {
        description: "speculative".to_string(),
        speculative: true,
        ..Default::default()
    }).unwrap();
    
    // Speculative changes
    state.insert_binding("x".to_string(), Value::Int(20), false);
    state.insert_binding("y".to_string(), Value::Int(30), false);
    
    // Check speculative state
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(20)));
    assert_eq!(state.bindings().get("y"), Some(&Value::Int(30)));
    
    // Rollback speculation
    state.rollback_transaction(tx).unwrap();
    
    // Back to original
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(10)));
    assert!(state.bindings().get("y").is_none());
}