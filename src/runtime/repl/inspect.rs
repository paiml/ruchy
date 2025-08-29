//! Inspect trait implementation for Value types
//!
//! [OBJ-INSPECT-003] Standardize display formats across all value types

use crate::runtime::inspect::{Inspect, Inspector};
use crate::runtime::repl::Value;
use std::fmt::{self, Write};

impl Inspect for Value {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            Value::Int(n) => n.inspect(inspector),
            Value::Float(f) => f.inspect(inspector),
            Value::Bool(b) => b.inspect(inspector),
            Value::String(s) => s.inspect(inspector),
            Value::Char(c) => write!(inspector, "'{c}'"),
            
            Value::List(items) => {
                if inspector.at_max_depth() {
                    return write!(inspector, "[{} items]", items.len());
                }
                
                if !inspector.enter(self) {
                    return write!(inspector, "[<circular>]");
                }
                
                write!(inspector, "[")?;
                
                let display_count = items.len().min(inspector.style.max_elements);
                for (i, item) in items.iter().take(display_count).enumerate() {
                    if i > 0 {
                        write!(inspector, ", ")?;
                    }
                    item.inspect(inspector)?;
                    
                    if !inspector.has_budget() {
                        write!(inspector, ", ...")?;
                        break;
                    }
                }
                
                if items.len() > display_count {
                    write!(inspector, ", ...{} more", items.len() - display_count)?;
                }
                
                inspector.exit();
                write!(inspector, "]")
            }
            
            Value::Tuple(items) => {
                if inspector.at_max_depth() {
                    return write!(inspector, "({} items)", items.len());
                }
                
                if !inspector.enter(self) {
                    return write!(inspector, "(<circular>)");
                }
                
                write!(inspector, "(")?;
                
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(inspector, ", ")?;
                    }
                    item.inspect(inspector)?;
                    
                    if !inspector.has_budget() {
                        write!(inspector, ", ...")?;
                        break;
                    }
                }
                
                inspector.exit();
                write!(inspector, ")")
            }
            
            Value::Object(map) => {
                if inspector.at_max_depth() {
                    return write!(inspector, "{{{} fields}}", map.len());
                }
                
                if !inspector.enter(self) {
                    return write!(inspector, "{{<circular>}}");
                }
                
                write!(inspector, "{{")?;
                
                let display_count = map.len().min(inspector.style.max_elements);
                for (i, (key, value)) in map.iter().take(display_count).enumerate() {
                    if i > 0 {
                        write!(inspector, ", ")?;
                    }
                    write!(inspector, "\"{key}\": ")?;
                    value.inspect(inspector)?;
                    
                    if !inspector.has_budget() {
                        write!(inspector, ", ...")?;
                        break;
                    }
                }
                
                if map.len() > display_count {
                    write!(inspector, ", ...{} more", map.len() - display_count)?;
                }
                
                inspector.exit();
                write!(inspector, "}}")
            }
            
            Value::HashMap(map) => {
                if inspector.at_max_depth() {
                    return write!(inspector, "HashMap{{{} entries}}", map.len());
                }
                
                if !inspector.enter(self) {
                    return write!(inspector, "HashMap{{<circular>}}");
                }
                
                write!(inspector, "HashMap{{")?;
                
                let display_count = map.len().min(inspector.style.max_elements);
                for (i, (key, value)) in map.iter().take(display_count).enumerate() {
                    if i > 0 {
                        write!(inspector, ", ")?;
                    }
                    key.inspect(inspector)?;
                    write!(inspector, ": ")?;
                    value.inspect(inspector)?;
                    
                    if !inspector.has_budget() {
                        write!(inspector, ", ...")?;
                        break;
                    }
                }
                
                if map.len() > display_count {
                    write!(inspector, ", ...{} more", map.len() - display_count)?;
                }
                
                inspector.exit();
                write!(inspector, "}}")
            }
            
            Value::HashSet(set) => {
                if inspector.at_max_depth() {
                    return write!(inspector, "HashSet{{{} items}}", set.len());
                }
                
                if !inspector.enter(self) {
                    return write!(inspector, "HashSet{{<circular>}}");
                }
                
                write!(inspector, "HashSet{{")?;
                
                let display_count = set.len().min(inspector.style.max_elements);
                for (i, value) in set.iter().take(display_count).enumerate() {
                    if i > 0 {
                        write!(inspector, ", ")?;
                    }
                    value.inspect(inspector)?;
                    
                    if !inspector.has_budget() {
                        write!(inspector, ", ...")?;
                        break;
                    }
                }
                
                if set.len() > display_count {
                    write!(inspector, ", ...{} more", set.len() - display_count)?;
                }
                
                inspector.exit();
                write!(inspector, "}}")
            }
            
            Value::Function { name, params, .. } => {
                write!(inspector, "fn {}({})", name, params.join(", "))
            }
            
            Value::Lambda { params, .. } => {
                write!(inspector, "|{}| <closure>", params.join(", "))
            }
            
            Value::Range { start, end, inclusive } => {
                if *inclusive {
                    write!(inspector, "{start}..={end}")
                } else {
                    write!(inspector, "{start}..{end}")
                }
            }
            
            Value::EnumVariant { enum_name, variant_name, data } => {
                write!(inspector, "{enum_name}::{variant_name}")?;
                if let Some(values) = data {
                    if !values.is_empty() {
                        write!(inspector, "(")?;
                        for (i, value) in values.iter().enumerate() {
                            if i > 0 {
                                write!(inspector, ", ")?;
                            }
                            value.inspect(inspector)?;
                        }
                        write!(inspector, ")")?;
                    }
                }
                Ok(())
            }
            
            Value::DataFrame { columns } => {
                write!(inspector, "DataFrame[{} columns]", columns.len())
            }
            
            Value::Unit => write!(inspector, "()"),
            Value::Nil => write!(inspector, "null"),
        }
    }
    
    fn inspect_depth(&self) -> usize {
        match self {
            Value::List(items) => {
                1 + items.iter().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0)
            }
            Value::Tuple(items) => {
                1 + items.iter().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0)
            }
            Value::Object(map) => {
                1 + map.values().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0)
            }
            Value::HashMap(map) => {
                let key_depth = map.keys().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0);
                let val_depth = map.values().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0);
                1 + key_depth.max(val_depth)
            }
            Value::HashSet(set) => {
                1 + set.iter().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0)
            }
            Value::EnumVariant { data, .. } => {
                if let Some(values) = data {
                    1 + values.iter().map(super::super::inspect::Inspect::inspect_depth).max().unwrap_or(0)
                } else {
                    1
                }
            }
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::inspect::InspectStyle;
    use std::collections::HashMap;
    
    #[test]
    fn test_value_inspection() {
        let mut inspector = Inspector::with_style(InspectStyle::default());
        
        // Test integer
        let val = Value::Int(42);
        val.inspect(&mut inspector).unwrap();
        
        // Test list
        let list = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
        list.inspect(&mut inspector).unwrap();
        
        // Test object
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Int(10));
        obj.insert("y".to_string(), Value::String("hello".to_string()));
        let obj_val = Value::Object(obj);
        obj_val.inspect(&mut inspector).unwrap();
    }
    
    #[test]
    fn test_enum_variant_inspection() {
        let mut inspector = Inspector::new();
        
        // Test Option::Some
        let some = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Int(42)]),
        };
        some.inspect(&mut inspector).unwrap();
        
        // Test Option::None
        let none = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };
        none.inspect(&mut inspector).unwrap();
    }
    
    #[test]
    fn test_depth_calculation() {
        // Simple value has depth 1
        assert_eq!(Value::Int(42).inspect_depth(), 1);
        
        // Nested list [[1]] has depth 3 (outer list + inner list + int)
        let nested = Value::List(vec![
            Value::List(vec![Value::Int(1)]),
        ]);
        assert_eq!(nested.inspect_depth(), 3);
        
        // Deeper nesting [[[1]]] has depth 4
        let deep = Value::List(vec![
            Value::List(vec![
                Value::List(vec![Value::Int(1)]),
            ]),
        ]);
        assert_eq!(deep.inspect_depth(), 4);
        
        // Empty list has depth 1
        assert_eq!(Value::List(vec![]).inspect_depth(), 1);
    }
}