use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use crate::vm::bytecode::Value;

/// Global state that persists across cell executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalState {
    /// Global variables with the `global` keyword
    pub globals: HashMap<String, GlobalVariable>,
    /// Import registry
    pub imports: HashMap<String, ImportInfo>,
    /// Function registry
    pub functions: HashMap<String, FunctionInfo>,
    /// Type definitions
    pub types: HashMap<String, TypeInfo>,
}

/// Information about a global variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalVariable {
    pub name: String,
    pub value: GlobalValue,
    pub mutable: bool,
    pub cell_id: Option<String>,
    pub declaration_order: usize,
}

/// Serializable representation of values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<GlobalValue>),
    Map(Vec<(String, GlobalValue)>),
    /// Reference to a value in the slab allocator
    SlabRef(String), // ID for slab lookup
}

/// Information about imports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module_name: String,
    pub imported_names: Vec<String>,
    pub alias: Option<String>,
    pub cell_id: String,
}

/// Information about functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameter_names: Vec<String>,
    pub return_type: Option<String>,
    pub cell_id: String,
    pub is_async: bool,
}

/// Information about types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
    pub cell_id: String,
}

/// Types of type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Struct { fields: Vec<(String, String)> },
    Enum { variants: Vec<String> },
    Alias { target: String },
}

/// Thread-safe state manager
pub struct StateManager {
    state: Arc<RwLock<GlobalState>>,
    next_declaration_order: Arc<RwLock<usize>>,
}

impl GlobalState {
    /// Create a new global state
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            imports: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
        }
    }
    
    /// Declare a global variable
    pub fn declare_global(
        &mut self,
        name: String,
        value: GlobalValue,
        mutable: bool,
        cell_id: Option<String>,
        order: usize,
    ) -> Result<(), String> {
        if self.globals.contains_key(&name) {
            return Err(format!("Global variable '{}' already exists", name));
        }
        
        self.globals.insert(name.clone(), GlobalVariable {
            name,
            value,
            mutable,
            cell_id,
            declaration_order: order,
        });
        
        Ok(())
    }
    
    /// Update a global variable
    pub fn update_global(&mut self, name: &str, value: GlobalValue) -> Result<(), String> {
        if let Some(global) = self.globals.get_mut(name) {
            if !global.mutable {
                return Err(format!("Cannot modify immutable global '{}'", name));
            }
            global.value = value;
            Ok(())
        } else {
            Err(format!("Global variable '{}' does not exist", name))
        }
    }
    
    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<&GlobalVariable> {
        self.globals.get(name)
    }
    
    /// Add an import
    pub fn add_import(&mut self, import: ImportInfo) {
        self.imports.insert(import.module_name.clone(), import);
    }
    
    /// Register a function
    pub fn register_function(&mut self, function: FunctionInfo) {
        self.functions.insert(function.name.clone(), function);
    }
    
    /// Register a type
    pub fn register_type(&mut self, type_info: TypeInfo) {
        self.types.insert(type_info.name.clone(), type_info);
    }
    
    /// Clear state from a specific cell
    pub fn clear_cell_state(&mut self, cell_id: &str) {
        self.globals.retain(|_, v| v.cell_id.as_ref() != Some(&cell_id.to_string()));
        self.imports.retain(|_, v| v.cell_id != cell_id);
        self.functions.retain(|_, v| v.cell_id != cell_id);
        self.types.retain(|_, v| v.cell_id != cell_id);
    }
    
    /// Get all globals in declaration order
    pub fn globals_ordered(&self) -> Vec<&GlobalVariable> {
        let mut globals: Vec<_> = self.globals.values().collect();
        globals.sort_by_key(|g| g.declaration_order);
        globals
    }
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(GlobalState::new())),
            next_declaration_order: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Execute a closure with read access to state
    pub fn with_state<T>(&self, f: impl FnOnce(&GlobalState) -> T) -> T {
        let state = self.state.read().unwrap();
        f(&*state)
    }
    
    /// Execute a closure with write access to state
    pub fn with_state_mut<T>(&self, f: impl FnOnce(&mut GlobalState) -> T) -> T {
        let mut state = self.state.write().unwrap();
        f(&mut *state)
    }
    
    /// Declare a global variable (thread-safe)
    pub fn declare_global(
        &self,
        name: String,
        value: GlobalValue,
        mutable: bool,
        cell_id: Option<String>,
    ) -> Result<(), String> {
        let order = {
            let mut next_order = self.next_declaration_order.write().unwrap();
            let current = *next_order;
            *next_order += 1;
            current
        };
        
        self.with_state_mut(|state| {
            state.declare_global(name, value, mutable, cell_id, order)
        })
    }
    
    /// Update a global variable (thread-safe)
    pub fn update_global(&self, name: &str, value: GlobalValue) -> Result<(), String> {
        self.with_state_mut(|state| state.update_global(name, value))
    }
    
    /// Get a global variable (thread-safe)
    pub fn get_global(&self, name: &str) -> Option<GlobalVariable> {
        self.with_state(|state| state.get_global(name).cloned())
    }
    
    /// Clear all state
    pub fn clear(&self) {
        self.with_state_mut(|state| {
            *state = GlobalState::new();
        });
        *self.next_declaration_order.write().unwrap() = 0;
    }
    
    /// Export state to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        self.with_state(|state| serde_json::to_string_pretty(state))
    }
    
    /// Import state from JSON
    pub fn import_json(&self, json: &str) -> Result<(), serde_json::Error> {
        let imported_state: GlobalState = serde_json::from_str(json)?;
        self.with_state_mut(|state| *state = imported_state);
        Ok(())
    }
}

impl GlobalValue {
    /// Convert from VM Value (lossy conversion for serialization)
    pub fn from_vm_value(value: &Value) -> Self {
        match value {
            Value::Null => GlobalValue::Null,
            Value::Bool(b) => GlobalValue::Bool(*b),
            Value::Int(i) => GlobalValue::Int(*i),
            Value::Float(f) => GlobalValue::Float(*f),
            Value::String(s) => GlobalValue::String(s.clone()),
            Value::List(items) => {
                GlobalValue::List(items.iter().map(Self::from_vm_value).collect())
            }
            Value::Map(pairs) => {
                GlobalValue::Map(pairs.iter().map(|(k, v)| (k.clone(), Self::from_vm_value(v))).collect())
            }
        }
    }
    
    /// Convert to VM Value (may require slab lookup for references)
    pub fn to_vm_value(&self) -> Value {
        match self {
            GlobalValue::Null => Value::Null,
            GlobalValue::Bool(b) => Value::Bool(*b),
            GlobalValue::Int(i) => Value::Int(*i),
            GlobalValue::Float(f) => Value::Float(*f),
            GlobalValue::String(s) => Value::String(s.clone()),
            GlobalValue::List(items) => {
                Value::List(items.iter().map(|item| item.to_vm_value()).collect())
            }
            GlobalValue::Map(pairs) => {
                Value::Map(pairs.iter().map(|(k, v)| (k.clone(), v.to_vm_value())).collect())
            }
            GlobalValue::SlabRef(_) => {
                // Would need slab allocator to resolve
                Value::Null
            }
        }
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_global_state() {
        let mut state = GlobalState::new();
        
        let result = state.declare_global(
            "my_var".to_string(),
            GlobalValue::Int(42),
            false,
            Some("cell_1".to_string()),
            0,
        );
        assert!(result.is_ok());
        
        let global = state.get_global("my_var").unwrap();
        assert_eq!(global.name, "my_var");
        assert!(!global.mutable);
        
        // Try to redeclare - should fail
        let result = state.declare_global(
            "my_var".to_string(),
            GlobalValue::Int(100),
            true,
            Some("cell_2".to_string()),
            1,
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_mutable_globals() {
        let mut state = GlobalState::new();
        
        state.declare_global(
            "counter".to_string(),
            GlobalValue::Int(0),
            true,
            None,
            0,
        ).unwrap();
        
        let result = state.update_global("counter", GlobalValue::Int(1));
        assert!(result.is_ok());
        
        let global = state.get_global("counter").unwrap();
        if let GlobalValue::Int(val) = &global.value {
            assert_eq!(*val, 1);
        } else {
            panic!("Expected Int value");
        }
    }
    
    #[test]
    fn test_state_manager() {
        let manager = StateManager::new();
        
        manager.declare_global(
            "shared_var".to_string(),
            GlobalValue::String("test".to_string()),
            false,
            Some("cell_1".to_string()),
        ).unwrap();
        
        let global = manager.get_global("shared_var").unwrap();
        assert_eq!(global.name, "shared_var");
        
        // Test JSON export/import
        let json = manager.export_json().unwrap();
        
        manager.clear();
        assert!(manager.get_global("shared_var").is_none());
        
        manager.import_json(&json).unwrap();
        let restored = manager.get_global("shared_var").unwrap();
        assert_eq!(restored.name, "shared_var");
    }
}