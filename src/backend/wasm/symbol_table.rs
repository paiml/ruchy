//! Symbol Table for WASM Code Generation
//!
//! Tracks variable types and local indices across scopes.

use super::types::WasmType;
use std::collections::HashMap;

/// Symbol table for tracking variable types and local indices across scopes
/// Complexity: <10 per method (Toyota Way)
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, (WasmType, u32)>>,
    next_local_index: u32,
}

impl SymbolTable {
    /// Create a new symbol table with a single global scope
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            next_local_index: 0,
        }
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope from the stack
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Insert a variable into the current scope
    pub fn insert(&mut self, name: String, ty: WasmType) {
        let index = self.next_local_index;
        self.next_local_index += 1;
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, (ty, index));
        }
    }

    /// Lookup a variable by name, searching from innermost to outermost scope
    pub fn lookup(&self, name: &str) -> Option<(WasmType, u32)> {
        for scope in self.scopes.iter().rev() {
            if let Some(&(ty, index)) = scope.get(name) {
                return Some((ty, index));
            }
        }
        None
    }

    /// Lookup just the type of a variable
    pub fn lookup_type(&self, name: &str) -> Option<WasmType> {
        self.lookup(name).map(|(ty, _)| ty)
    }

    /// Lookup just the local index of a variable
    pub fn lookup_index(&self, name: &str) -> Option<u32> {
        self.lookup(name).map(|(_, index)| index)
    }

    /// Get the total number of local variables allocated
    pub fn local_count(&self) -> u32 {
        self.next_local_index
    }

    /// Get the current scope depth
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }

    /// Check if a variable exists in the current scope only
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .is_some_and(|scope| scope.contains_key(name))
    }

    /// Get all local variables across all scopes as (type, index) pairs
    pub fn all_locals(&self) -> Vec<(WasmType, u32)> {
        let mut locals: Vec<(WasmType, u32)> = Vec::new();
        for scope in &self.scopes {
            for &(ty, index) in scope.values() {
                locals.push((ty, index));
            }
        }
        locals
    }

    /// Clear all scopes and reset
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scopes.push(HashMap::new());
        self.next_local_index = 0;
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_new() {
        let table = SymbolTable::new();
        assert_eq!(table.scope_depth(), 1);
        assert_eq!(table.local_count(), 0);
    }

    #[test]
    fn test_symbol_table_default() {
        let table = SymbolTable::default();
        assert_eq!(table.scope_depth(), 1);
    }

    #[test]
    fn test_push_scope() {
        let mut table = SymbolTable::new();
        assert_eq!(table.scope_depth(), 1);
        table.push_scope();
        assert_eq!(table.scope_depth(), 2);
        table.push_scope();
        assert_eq!(table.scope_depth(), 3);
    }

    #[test]
    fn test_pop_scope() {
        let mut table = SymbolTable::new();
        table.push_scope();
        table.push_scope();
        assert_eq!(table.scope_depth(), 3);
        table.pop_scope();
        assert_eq!(table.scope_depth(), 2);
        table.pop_scope();
        assert_eq!(table.scope_depth(), 1);
    }

    #[test]
    fn test_pop_scope_doesnt_remove_global() {
        let mut table = SymbolTable::new();
        assert_eq!(table.scope_depth(), 1);
        table.pop_scope();
        assert_eq!(table.scope_depth(), 1); // Can't pop global scope
    }

    #[test]
    fn test_insert_and_lookup() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);

        let result = table.lookup("x");
        assert!(result.is_some());
        let (ty, index) = result.unwrap();
        assert_eq!(ty, WasmType::I32);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_insert_multiple() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);
        table.insert("y".to_string(), WasmType::F32);
        table.insert("z".to_string(), WasmType::I64);

        assert_eq!(table.lookup_index("x"), Some(0));
        assert_eq!(table.lookup_index("y"), Some(1));
        assert_eq!(table.lookup_index("z"), Some(2));
        assert_eq!(table.local_count(), 3);
    }

    #[test]
    fn test_lookup_type() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::F64);

        assert_eq!(table.lookup_type("x"), Some(WasmType::F64));
        assert_eq!(table.lookup_type("nonexistent"), None);
    }

    #[test]
    fn test_lookup_index() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);

        assert_eq!(table.lookup_index("x"), Some(0));
        assert_eq!(table.lookup_index("nonexistent"), None);
    }

    #[test]
    fn test_scope_shadowing() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);
        assert_eq!(table.lookup_type("x"), Some(WasmType::I32));

        table.push_scope();
        table.insert("x".to_string(), WasmType::F32);
        assert_eq!(table.lookup_type("x"), Some(WasmType::F32));

        table.pop_scope();
        assert_eq!(table.lookup_type("x"), Some(WasmType::I32));
    }

    #[test]
    fn test_inner_scope_sees_outer_vars() {
        let mut table = SymbolTable::new();
        table.insert("outer".to_string(), WasmType::I32);

        table.push_scope();
        assert_eq!(table.lookup_type("outer"), Some(WasmType::I32));
    }

    #[test]
    fn test_exists_in_current_scope() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);
        assert!(table.exists_in_current_scope("x"));

        table.push_scope();
        assert!(!table.exists_in_current_scope("x")); // Not in new scope

        table.insert("y".to_string(), WasmType::F32);
        assert!(table.exists_in_current_scope("y"));
    }

    #[test]
    fn test_clear() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);
        table.push_scope();
        table.insert("y".to_string(), WasmType::F32);

        table.clear();

        assert_eq!(table.scope_depth(), 1);
        assert_eq!(table.local_count(), 0);
        assert!(table.lookup("x").is_none());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_debug() {
        let table = SymbolTable::new();
        let debug = format!("{:?}", table);
        assert!(debug.contains("SymbolTable"));
    }

    #[test]
    fn test_clone() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);

        let cloned = table.clone();
        assert_eq!(cloned.lookup_type("x"), Some(WasmType::I32));
    }

    #[test]
    fn test_all_locals() {
        let mut table = SymbolTable::new();
        table.insert("x".to_string(), WasmType::I32);
        table.insert("y".to_string(), WasmType::F64);
        table.push_scope();
        table.insert("z".to_string(), WasmType::F32);

        let locals = table.all_locals();
        assert_eq!(locals.len(), 3);
        // All three locals should be present (order may vary due to HashMap)
        assert!(locals.contains(&(WasmType::I32, 0)));
        assert!(locals.contains(&(WasmType::F64, 1)));
        assert!(locals.contains(&(WasmType::F32, 2)));
    }

    #[test]
    fn test_all_locals_empty() {
        let table = SymbolTable::new();
        let locals = table.all_locals();
        assert!(locals.is_empty());
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_insert_increments_local_count(name in "[a-z]{1,10}") {
                let mut table = SymbolTable::new();
                let before = table.local_count();
                table.insert(name, WasmType::I32);
                prop_assert_eq!(table.local_count(), before + 1);
            }

            #[test]
            fn prop_push_pop_preserves_depth(pushes in 1usize..10) {
                let mut table = SymbolTable::new();
                for _ in 0..pushes {
                    table.push_scope();
                }
                prop_assert_eq!(table.scope_depth(), 1 + pushes);

                for _ in 0..pushes {
                    table.pop_scope();
                }
                prop_assert_eq!(table.scope_depth(), 1);
            }

            #[test]
            fn prop_lookup_after_insert(name in "[a-z]{1,10}") {
                let mut table = SymbolTable::new();
                table.insert(name.clone(), WasmType::F64);
                prop_assert!(table.lookup(&name).is_some());
            }

            #[test]
            fn prop_clear_resets_state(inserts in 1usize..10) {
                let mut table = SymbolTable::new();
                for i in 0..inserts {
                    table.insert(format!("var{i}"), WasmType::I32);
                }
                table.push_scope();

                table.clear();

                prop_assert_eq!(table.scope_depth(), 1);
                prop_assert_eq!(table.local_count(), 0);
            }
        }
    }
}
