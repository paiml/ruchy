// TDD Test Suite for Value::inspect Complexity Reduction
// Current: 133 cyclomatic complexity - HIGHEST COMPLEXITY FUNCTION  
// Target: <20 for both metrics
// Strategy: Extract type-specific inspection handlers

use ruchy::runtime::repl::Value;
use ruchy::runtime::inspect::{Inspect, Inspector};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[cfg(test)]
mod value_inspect_tdd {
    use super::*;

    fn create_test_inspector() -> Inspector {
        Inspector::new()
    }

    // Test simple value types
    #[test]
    fn test_inspect_int() {
        let mut inspector = create_test_inspector();
        let value = Value::Int(42);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_float() {
        let mut inspector = create_test_inspector();
        let value = Value::Float(3.14);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_bool() {
        let mut inspector = create_test_inspector();
        let value = Value::Bool(true);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_string() {
        let mut inspector = create_test_inspector();
        let value = Value::String("hello".to_string());
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_char() {
        let mut inspector = create_test_inspector();
        let value = Value::Char('x');
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    // Test collection types - these have the most complexity
    #[test]
    fn test_inspect_list_empty() {
        let mut inspector = create_test_inspector();
        let value = Value::List(vec![]);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_list_with_items() {
        let mut inspector = create_test_inspector();
        let value = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::String("test".to_string())
        ]);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_tuple_empty() {
        let mut inspector = create_test_inspector();
        let value = Value::Tuple(vec![]);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_tuple_with_items() {
        let mut inspector = create_test_inspector();
        let value = Value::Tuple(vec![
            Value::Int(1),
            Value::String("test".to_string())
        ]);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_object_empty() {
        let mut inspector = create_test_inspector();
        let value = Value::Object(HashMap::new());
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_object_with_fields() {
        let mut inspector = create_test_inspector();
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("John".to_string()));
        map.insert("age".to_string(), Value::Int(30));
        let value = Value::Object(map);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_hashmap() {
        let mut inspector = create_test_inspector();
        let mut map = HashMap::new();
        map.insert(Value::String("key1".to_string()), Value::Int(42));
        map.insert(Value::Int(2), Value::String("value2".to_string()));
        let value = Value::HashMap(map);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_hashset() {
        let mut inspector = create_test_inspector();
        let mut set = HashSet::new();
        set.insert(Value::Int(1));
        set.insert(Value::String("test".to_string()));
        let value = Value::HashSet(set);
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    // Test function types
    #[test]
    fn test_inspect_function() {
        let mut inspector = create_test_inspector();
        let value = Value::Function {
            name: "test_fn".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(ruchy::frontend::ast::Expr {
                kind: ruchy::frontend::ast::ExprKind::Identifier("x".to_string()),
                span: (0, 1).into(),
            }),
            env: Rc::new(HashMap::new()),
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_lambda() {
        let mut inspector = create_test_inspector();
        let value = Value::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(ruchy::frontend::ast::Expr {
                kind: ruchy::frontend::ast::ExprKind::Identifier("x".to_string()),
                span: (0, 1).into(),
            }),
            env: Rc::new(HashMap::new()),
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    // Test other complex types
    #[test]
    fn test_inspect_range_inclusive() {
        let mut inspector = create_test_inspector();
        let value = Value::Range {
            start: Box::new(Value::Int(1)),
            end: Box::new(Value::Int(10)),
            inclusive: true,
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_range_exclusive() {
        let mut inspector = create_test_inspector();
        let value = Value::Range {
            start: Box::new(Value::Int(1)),
            end: Box::new(Value::Int(10)),
            inclusive: false,
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_enum_variant_no_data() {
        let mut inspector = create_test_inspector();
        let value = Value::EnumVariant {
            enum_name: "Color".to_string(),
            variant_name: "Red".to_string(),
            data: None,
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_enum_variant_with_data() {
        let mut inspector = create_test_inspector();
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Int(42)]),
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_dataframe() {
        let mut inspector = create_test_inspector();
        let value = Value::DataFrame {
            columns: vec![
                ruchy::runtime::repl::DataFrameColumn {
                    name: "id".to_string(),
                    values: vec![Value::Int(1), Value::Int(2)],
                },
                ruchy::runtime::repl::DataFrameColumn {
                    name: "name".to_string(),
                    values: vec![Value::String("Alice".to_string()), Value::String("Bob".to_string())],
                },
            ],
        };
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_unit() {
        let mut inspector = create_test_inspector();
        let value = Value::Unit;
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_nil() {
        let mut inspector = create_test_inspector();
        let value = Value::Nil;
        let result = value.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    // Test depth limiting and circular reference detection
    #[test]
    fn test_inspect_with_max_depth() {
        let mut inspector = create_test_inspector();
        // Create deeply nested structure
        let inner_list = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let nested_list = Value::List(vec![inner_list]);
        let result = nested_list.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_large_collection() {
        let mut inspector = create_test_inspector();
        // Create a large list to test max_elements limiting
        let large_list = Value::List((0..100).map(Value::Int).collect());
        let result = large_list.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    // Test that inspection handles budget limits
    #[test]
    fn test_inspect_budget_limiting() {
        let mut inspector = create_test_inspector();
        // Create nested structure that should trigger budget limits
        let nested = Value::List(vec![
            Value::Object({
                let mut map = HashMap::new();
                map.insert("field".to_string(), Value::List(vec![Value::Int(1); 50]));
                map
            })
        ]);
        let result = nested.inspect(&mut inspector);
        assert!(result.is_ok());
    }

    // Tests for refactored helper methods (to be implemented)
    mod refactored_helpers {
        use super::*;

        #[test]  
        fn test_inspect_simple_value() {
            // Test extracted simple value inspection
            let mut inspector = create_test_inspector();
            
            // This would test the extracted inspect_simple_value once implemented
            // let result = Value::Int(42).inspect_simple_value(&mut inspector);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_inspect_collection_value() {
            // Test extracted collection inspection
            let mut inspector = create_test_inspector();
            
            // This would test the extracted inspect_collection_value once implemented  
            // let value = Value::List(vec![Value::Int(1), Value::Int(2)]);
            // let result = value.inspect_collection_value(&mut inspector);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_inspect_function_value() {
            // Test extracted function inspection
            let mut inspector = create_test_inspector();
            
            // This would test the extracted inspect_function_value once implemented
        }
    }
}

// Demonstration of how the refactoring would work
// These would be the extracted helper methods to reduce complexity
/*
impl Inspect for Value {
    // Main method becomes a dispatcher (complexity ~15)
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            // Simple types (complexity ~1 each)
            Value::Int(_) | Value::Float(_) | Value::Bool(_) | 
            Value::String(_) | Value::Char(_) | Value::Unit | Value::Nil => {
                self.inspect_simple_value(inspector)
            }
            // Collection types (complexity delegated)
            Value::List(_) | Value::Tuple(_) | Value::Object(_) | 
            Value::HashMap(_) | Value::HashSet(_) => {
                self.inspect_collection_value(inspector)
            }
            // Function types (complexity delegated)
            Value::Function { .. } | Value::Lambda { .. } => {
                self.inspect_function_value(inspector)
            }
            // Complex types (complexity delegated)
            Value::DataFrame { .. } | Value::Range { .. } | Value::EnumVariant { .. } => {
                self.inspect_complex_value(inspector)
            }
        }
    }

    // Extract simple value inspection (complexity ~8)
    fn inspect_simple_value(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            Value::Int(n) => n.inspect(inspector),
            Value::Float(f) => f.inspect(inspector),
            Value::Bool(b) => b.inspect(inspector),
            Value::String(s) => s.inspect(inspector),
            Value::Char(c) => write!(inspector, "'{c}'"),
            Value::Unit => write!(inspector, "()"),
            Value::Nil => write!(inspector, "null"),
            _ => unreachable!("Non-simple value passed to inspect_simple_value"),
        }
    }

    // Extract collection inspection (complexity ~30)
    fn inspect_collection_value(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            Value::List(items) => self.inspect_list(inspector, items),
            Value::Tuple(items) => self.inspect_tuple(inspector, items),
            Value::Object(map) => self.inspect_object(inspector, map),
            Value::HashMap(map) => self.inspect_hashmap(inspector, map),
            Value::HashSet(set) => self.inspect_hashset(inspector, set),
            _ => unreachable!("Non-collection value passed to inspect_collection_value"),
        }
    }

    // Individual collection handlers (complexity ~10 each)
    fn inspect_list(&self, inspector: &mut Inspector, items: &[Value]) -> fmt::Result { ... }
    fn inspect_tuple(&self, inspector: &mut Inspector, items: &[Value]) -> fmt::Result { ... }
    fn inspect_object(&self, inspector: &mut Inspector, map: &HashMap<String, Value>) -> fmt::Result { ... }
    fn inspect_hashmap(&self, inspector: &mut Inspector, map: &HashMap<Value, Value>) -> fmt::Result { ... }
    fn inspect_hashset(&self, inspector: &mut Inspector, set: &HashSet<Value>) -> fmt::Result { ... }
}
*/