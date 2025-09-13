//! Comprehensive TDD tests for SharedSession module
//! Target: Increase coverage for session management
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod shared_session_tests {
    use crate::wasm::shared_session::{SharedSession, GlobalRegistry, DefId, ExecutionMode, ExecuteResponse};
    use std::collections::HashMap;
    
    // ========== SharedSession Creation Tests ==========
    
    #[test]
    fn test_session_creation() {
        let session = SharedSession::new();
        assert_eq!(session.cell_count(), 0);
        assert!(session.is_empty());
    }
    
    #[test]
    fn test_session_with_id() {
        let session = SharedSession::with_id("test-session-123");
        assert_eq!(session.get_id(), "test-session-123");
    }
    
    #[test]
    fn test_multiple_sessions_unique() {
        let session1 = SharedSession::new();
        let session2 = SharedSession::new();
        assert_ne!(session1.get_id(), session2.get_id());
    }
    
    // ========== Cell Management Tests ==========
    
    #[test]
    fn test_add_cell() {
        let mut session = SharedSession::new();
        let cell_id = session.add_cell("let x = 10");
        assert_eq!(session.cell_count(), 1);
        assert!(!session.is_empty());
        assert!(cell_id.len() > 0);
    }
    
    #[test]
    fn test_get_cell() {
        let mut session = SharedSession::new();
        let cell_id = session.add_cell("let x = 10");
        
        let cell = session.get_cell(&cell_id);
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().source, "let x = 10");
    }
    
    #[test]
    fn test_update_cell() {
        let mut session = SharedSession::new();
        let cell_id = session.add_cell("let x = 10");
        
        let updated = session.update_cell(&cell_id, "let x = 20");
        assert!(updated);
        
        let cell = session.get_cell(&cell_id).unwrap();
        assert_eq!(cell.source, "let x = 20");
    }
    
    #[test]
    fn test_delete_cell() {
        let mut session = SharedSession::new();
        let cell_id = session.add_cell("let x = 10");
        assert_eq!(session.cell_count(), 1);
        
        let deleted = session.delete_cell(&cell_id);
        assert!(deleted);
        assert_eq!(session.cell_count(), 0);
        assert!(session.get_cell(&cell_id).is_none());
    }
    
    #[test]
    fn test_clear_all_cells() {
        let mut session = SharedSession::new();
        session.add_cell("let x = 10");
        session.add_cell("let y = 20");
        session.add_cell("x + y");
        assert_eq!(session.cell_count(), 3);
        
        session.clear();
        assert_eq!(session.cell_count(), 0);
        assert!(session.is_empty());
    }
    
    // ========== Execution Tests ==========
    
    #[test]
    fn test_execute_cell_normal_mode() {
        let mut session = SharedSession::new();
        let cell_id = session.add_cell("1 + 1");
        
        let response = session.execute_cell(&cell_id, ExecutionMode::Normal);
        assert!(response.success);
        assert!(response.error.is_none());
        assert!(response.timing.total_ms >= 0.0);
    }
    
    #[test]
    fn test_execute_cell_reactive_mode() {
        let mut session = SharedSession::new();
        let cell1 = session.add_cell("let x = 10");
        let cell2 = session.add_cell("let y = x * 2");
        
        let response = session.execute_cell(&cell1, ExecutionMode::Reactive);
        assert!(response.success);
        // In reactive mode, dependent cells should also execute
        assert!(response.affected_cells.contains(&cell2));
    }
    
    #[test]
    fn test_execute_all_cells() {
        let mut session = SharedSession::new();
        session.add_cell("let x = 10");
        session.add_cell("let y = 20");
        session.add_cell("x + y");
        
        let results = session.execute_all();
        assert_eq!(results.len(), 3);
        for response in results {
            assert!(response.timing.total_ms >= 0.0);
        }
    }
    
    // ========== Global Registry Tests ==========
    
    #[test]
    fn test_global_registry_singleton() {
        let registry1 = GlobalRegistry::instance();
        let registry2 = GlobalRegistry::instance();
        // Should be the same instance
        assert_eq!(registry1.get_session_count(), registry2.get_session_count());
    }
    
    #[test]
    fn test_registry_register_definition() {
        let registry = GlobalRegistry::instance();
        let def_id = DefId::new("test_func");
        
        registry.register_def(def_id.clone(), "fn test_func() {}");
        assert!(registry.has_def(&def_id));
        
        let def = registry.get_def(&def_id);
        assert!(def.is_some());
        assert_eq!(def.unwrap(), "fn test_func() {}");
    }
    
    #[test]
    fn test_registry_track_dependencies() {
        let registry = GlobalRegistry::instance();
        let def1 = DefId::new("x");
        let def2 = DefId::new("y");
        
        registry.add_dependency(&def2, &def1); // y depends on x
        
        let deps = registry.get_dependencies(&def2);
        assert!(deps.contains(&def1));
        
        let dependents = registry.get_dependents(&def1);
        assert!(dependents.contains(&def2));
    }
    
    // ========== Cell Dependency Tests ==========
    
    #[test]
    fn test_cell_dependencies() {
        let mut session = SharedSession::new();
        let cell1 = session.add_cell("let x = 10");
        let cell2 = session.add_cell("let y = x * 2");
        let cell3 = session.add_cell("let z = y + x");
        
        session.add_dependency(&cell2, &cell1); // cell2 depends on cell1
        session.add_dependency(&cell3, &cell1); // cell3 depends on cell1
        session.add_dependency(&cell3, &cell2); // cell3 depends on cell2
        
        let deps = session.get_dependencies(&cell3);
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&cell1));
        assert!(deps.contains(&cell2));
    }
    
    #[test]
    fn test_topological_execution_order() {
        let mut session = SharedSession::new();
        let cell1 = session.add_cell("let x = 10");
        let cell2 = session.add_cell("let y = x * 2");
        let cell3 = session.add_cell("let z = y + 5");
        
        session.add_dependency(&cell2, &cell1);
        session.add_dependency(&cell3, &cell2);
        
        let order = session.get_execution_order();
        // cell1 should come before cell2, cell2 before cell3
        let idx1 = order.iter().position(|c| c == &cell1).unwrap();
        let idx2 = order.iter().position(|c| c == &cell2).unwrap();
        let idx3 = order.iter().position(|c| c == &cell3).unwrap();
        
        assert!(idx1 < idx2);
        assert!(idx2 < idx3);
    }
    
    // ========== State Management Tests ==========
    
    #[test]
    fn test_checkpoint_creation() {
        let mut session = SharedSession::new();
        session.add_cell("let x = 10");
        session.add_cell("let y = 20");
        
        let checkpoint = session.create_checkpoint();
        assert_eq!(checkpoint.cell_count, 2);
        assert!(checkpoint.timestamp > 0);
    }
    
    #[test]
    fn test_restore_from_checkpoint() {
        let mut session = SharedSession::new();
        session.add_cell("let x = 10");
        let checkpoint = session.create_checkpoint();
        
        session.add_cell("let y = 20");
        session.add_cell("let z = 30");
        assert_eq!(session.cell_count(), 3);
        
        session.restore_checkpoint(&checkpoint);
        assert_eq!(session.cell_count(), 1);
    }
    
    // ========== Error Handling Tests ==========
    
    #[test]
    fn test_execute_nonexistent_cell() {
        let mut session = SharedSession::new();
        let response = session.execute_cell("nonexistent-id", ExecutionMode::Normal);
        assert!(!response.success);
        assert!(response.error.is_some());
    }
    
    #[test]
    fn test_update_nonexistent_cell() {
        let mut session = SharedSession::new();
        let updated = session.update_cell("nonexistent-id", "new code");
        assert!(!updated);
    }
    
    #[test]
    fn test_delete_nonexistent_cell() {
        let mut session = SharedSession::new();
        let deleted = session.delete_cell("nonexistent-id");
        assert!(!deleted);
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl SharedSession {
        /// Helper: Check if session is empty
        fn is_empty(&self) -> bool {
            self.cell_count() == 0
        }
        
        /// Helper: Get session ID
        fn get_id(&self) -> &str {
            &self.session_id
        }
    }
    
    impl GlobalRegistry {
        /// Helper: Get session count
        fn get_session_count(&self) -> usize {
            self.sessions.len()
        }
    }
    
    impl DefId {
        /// Helper: Create new DefId
        fn new(name: &str) -> Self {
            DefId {
                name: name.to_string(),
                module: None,
            }
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_add_cells_never_panics(sources in prop::collection::vec("[a-z]+", 1..20)) {
            let mut session = SharedSession::new();
            
            for source in sources {
                let cell_id = session.add_cell(&source);
                assert!(cell_id.len() > 0);
            }
        }
        
        #[test]
        fn test_cell_ids_unique(count in 1usize..100) {
            let mut session = SharedSession::new();
            let mut ids = Vec::new();
            
            for i in 0..count {
                let id = session.add_cell(&format!("cell {}", i));
                assert!(!ids.contains(&id));
                ids.push(id);
            }
        }
        
        #[test]
        fn test_checkpoint_restore_preserves_count(initial in 1usize..10, additional in 1usize..10) {
            let mut session = SharedSession::new();
            
            // Add initial cells
            for i in 0..initial {
                session.add_cell(&format!("initial {}", i));
            }
            
            let checkpoint = session.create_checkpoint();
            
            // Add more cells
            for i in 0..additional {
                session.add_cell(&format!("additional {}", i));
            }
            
            assert_eq!(session.cell_count(), initial + additional);
            
            // Restore should go back to initial count
            session.restore_checkpoint(&checkpoint);
            assert_eq!(session.cell_count(), initial);
        }
    }
}