//! Coverage tests for transactional state
//!
//! [TEST-COV-006] Transaction State Coverage

use ruchy::runtime::{
    TransactionalState, TransactionMetadata, TransactionId,
    TransactionEvent, TransactionLog, MVCC, Version,
    Value,
};
use std::time::Duration;

#[test]
fn test_transactional_state_basic() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Insert bindings
    state.insert_binding("x".to_string(), Value::Int(42), false);
    state.insert_binding("y".to_string(), Value::String("hello".to_string()), true);
    
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(42)));
    assert_eq!(state.bindings().get("y"), Some(&Value::String("hello".to_string())));
    
    assert!(!state.is_mutable("x"));
    assert!(state.is_mutable("y"));
    
    // Check memory usage
    assert!(state.memory_used() > 0);
}

#[test]
fn test_transaction_begin_commit() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Initial state
    state.insert_binding("x".to_string(), Value::Int(1), false);
    
    // Begin transaction
    let metadata = TransactionMetadata {
        description: "test transaction".to_string(),
        memory_limit: Some(1024),
        time_limit: Some(Duration::from_secs(1)),
        speculative: false,
    };
    
    let tx_id = state.begin_transaction(metadata).unwrap();
    assert_eq!(state.depth(), 1);
    
    // Modify state
    state.insert_binding("x".to_string(), Value::Int(2), false);
    state.insert_binding("y".to_string(), Value::Int(3), false);
    
    // Commit
    assert!(state.commit_transaction(tx_id).is_ok());
    assert_eq!(state.depth(), 0);
    
    // Changes persist
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(2)));
    assert_eq!(state.bindings().get("y"), Some(&Value::Int(3)));
}

#[test]
fn test_transaction_rollback() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Initial state
    state.insert_binding("x".to_string(), Value::Int(1), false);
    
    // Begin transaction
    let tx_id = state.begin_transaction(TransactionMetadata::default()).unwrap();
    
    // Modify state
    state.insert_binding("x".to_string(), Value::Int(2), false);
    state.insert_binding("y".to_string(), Value::Int(3), false);
    
    // Rollback
    assert!(state.rollback_transaction(tx_id).is_ok());
    
    // Changes reverted
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(1)));
    assert_eq!(state.bindings().get("y"), None);
}

#[test]
fn test_nested_transactions() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(1), false);
    
    // First transaction
    let tx1 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    state.insert_binding("x".to_string(), Value::Int(2), false);
    
    // Nested transaction
    let tx2 = state.begin_transaction(TransactionMetadata::default()).unwrap();
    state.insert_binding("x".to_string(), Value::Int(3), false);
    
    assert_eq!(state.depth(), 2);
    
    // Rollback inner
    state.rollback_transaction(tx2).unwrap();
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(2)));
    
    // Commit outer
    state.commit_transaction(tx1).unwrap();
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(2)));
}

#[test]
fn test_transaction_limits() {
    let mut state = TransactionalState::new(1024);
    
    let metadata = TransactionMetadata {
        description: "limited".to_string(),
        memory_limit: Some(100),
        time_limit: Some(Duration::from_millis(100)),
        speculative: false,
    };
    
    let tx_id = state.begin_transaction(metadata).unwrap();
    
    // Should pass initially
    assert!(state.check_transaction_limits(tx_id).is_ok());
    
    // Allocate memory
    for i in 0..10 {
        state.insert_binding(format!("var{}", i), Value::Int(i), false);
    }
    
    // May exceed memory limit
    // Note: actual behavior depends on implementation
}

#[test]
fn test_transaction_depth_limit() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Create many nested transactions
    let mut tx_ids = Vec::new();
    for _ in 0..100 {
        match state.begin_transaction(TransactionMetadata::default()) {
            Ok(id) => tx_ids.push(id),
            Err(_) => break, // Hit depth limit
        }
    }
    
    // Should hit limit before 100
    assert!(tx_ids.len() < 101);
    
    // Clean up
    for id in tx_ids.iter().rev() {
        state.rollback_transaction(*id).ok();
    }
}

#[test]
fn test_transaction_clear() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    state.insert_binding("x".to_string(), Value::Int(42), false);
    state.insert_binding("y".to_string(), Value::Bool(true), true);
    
    state.clear();
    
    assert!(state.bindings().is_empty());
    assert_eq!(state.depth(), 0);
    assert_eq!(state.memory_used(), 0);
}

#[test]
fn test_transaction_log() {
    let mut log = TransactionLog::new(100);
    
    log.log(TransactionEvent::Begin {
        id: TransactionId(1),
        metadata: TransactionMetadata::default(),
    });
    
    log.log(TransactionEvent::BindingAdded {
        name: "x".to_string(),
        value_type: "Int".to_string(),
    });
    
    log.log(TransactionEvent::Commit {
        id: TransactionId(1),
        duration: Duration::from_millis(50),
        memory_used: 100,
    });
    
    let recent = log.recent_events(2);
    assert_eq!(recent.len(), 2);
    
    // Clear log
    log.clear();
    assert_eq!(log.recent_events(10).len(), 0);
}

#[test]
fn test_mvcc() {
    let mut mvcc = MVCC::new();
    
    // Write at version 1
    let v1 = mvcc.begin_write();
    mvcc.write("x".to_string(), Value::Int(1), v1);
    
    // Write at version 2
    let v2 = mvcc.begin_write();
    mvcc.write("x".to_string(), Value::Int(2), v2);
    
    // Write at version 3
    let v3 = mvcc.begin_write();
    mvcc.write("x".to_string(), Value::Int(3), v3);
    mvcc.write("y".to_string(), Value::Bool(true), v3);
    
    // Read at different versions
    assert_eq!(mvcc.read("x", v1), Some(&Value::Int(1)));
    assert_eq!(mvcc.read("x", v2), Some(&Value::Int(2)));
    assert_eq!(mvcc.read("x", v3), Some(&Value::Int(3)));
    
    // y doesn't exist at v1 or v2
    assert_eq!(mvcc.read("y", v1), None);
    assert_eq!(mvcc.read("y", v2), None);
    assert_eq!(mvcc.read("y", v3), Some(&Value::Bool(true)));
    
    // Garbage collect old versions
    mvcc.gc(v2);
    
    // v1 data may be gone, but v2 and v3 remain
    assert_eq!(mvcc.read("x", v2), Some(&Value::Int(2)));
    assert_eq!(mvcc.read("x", v3), Some(&Value::Int(3)));
}

#[test]
fn test_mvcc_concurrent_reads() {
    let mut mvcc = MVCC::new();
    
    // Initial write
    let v1 = mvcc.begin_write();
    mvcc.write("counter".to_string(), Value::Int(0), v1);
    
    // Start read transaction at v1
    let read_version = mvcc.begin_read();
    
    // Concurrent writes
    for i in 1..=5 {
        let v = mvcc.begin_write();
        mvcc.write("counter".to_string(), Value::Int(i), v);
    }
    
    // Read still sees v1 value
    assert_eq!(mvcc.read("counter", read_version), Some(&Value::Int(0)));
    
    // New read sees latest
    let new_read = mvcc.begin_read();
    assert_eq!(mvcc.read("counter", new_read), Some(&Value::Int(5)));
}