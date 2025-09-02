//! Refactored inspect implementation with reduced complexity
//! Original complexity: 133, Target: <20 per function

use crate::runtime::inspect::{Inspect, Inspector};
use crate::runtime::repl::Value;
use std::fmt::{self, Write};
use std::collections::{HashMap, HashSet};

impl Value {
    /// Main inspect dispatch function (complexity: ~10)
    pub fn inspect_refactored(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            // Simple values (complexity: 1 each)
            Value::Int(n) => n.inspect(inspector),
            Value::Float(f) => f.inspect(inspector),
            Value::Bool(b) => b.inspect(inspector),
            Value::String(s) => s.inspect(inspector),
            Value::Char(c) => write!(inspector, "'{c}'"),
            Value::Unit => write!(inspector, "()"),
            Value::Nil => write!(inspector, "null"),
            
            // Collection types (delegated)
            Value::List(items) => inspect_list(items, inspector),
            Value::Tuple(items) => inspect_tuple(items, inspector),
            Value::Object(map) => inspect_object(map, inspector),
            Value::HashMap(map) => inspect_hashmap(map, inspector),
            Value::HashSet(set) => inspect_hashset(set, inspector),
            
            // Complex types (delegated)
            Value::Function { name, params, .. } => inspect_function(name, params, inspector),
            Value::Lambda { params, .. } => inspect_lambda(params, inspector),
            Value::Range { start, end, inclusive } => inspect_range(start, end, *inclusive, inspector),
            Value::EnumVariant { enum_name, variant_name, data } => 
                inspect_enum_variant(enum_name, variant_name, data, inspector),
            Value::DataFrame { columns } => inspect_dataframe(columns, inspector),
        }
    }
    
    /// Calculate depth for recursive structures (complexity: ~10)
    pub fn inspect_depth_refactored(&self) -> usize {
        match self {
            Value::List(items) => calc_list_depth(items),
            Value::Tuple(items) => calc_tuple_depth(items),
            Value::Object(map) => calc_object_depth(map),
            Value::HashMap(map) => calc_hashmap_depth(map),
            Value::HashSet(set) => calc_hashset_depth(set),
            Value::EnumVariant { data, .. } => calc_enum_variant_depth(data),
            _ => 1,
        }
    }
}

// Helper functions for list inspection (complexity: ~8)
fn inspect_list(items: &[Value], inspector: &mut Inspector) -> fmt::Result {
    if inspector.at_max_depth() {
        return write!(inspector, "[{} items]", items.len());
    }
    
    if !inspector.enter(items as *const _ as *const ()) {
        return write!(inspector, "[<circular>]");
    }
    
    write!(inspector, "[")?;
    inspect_collection_items(items.iter(), inspector)?;
    inspector.exit();
    write!(inspector, "]")
}

// Helper for tuple inspection (complexity: ~8)
fn inspect_tuple(items: &[Value], inspector: &mut Inspector) -> fmt::Result {
    if inspector.at_max_depth() {
        return write!(inspector, "({} items)", items.len());
    }
    
    if !inspector.enter(items as *const _ as *const ()) {
        return write!(inspector, "(<circular>)");
    }
    
    write!(inspector, "(")?;
    inspect_collection_items(items.iter(), inspector)?;
    inspector.exit();
    write!(inspector, ")")
}

// Helper for object inspection (complexity: ~10)
fn inspect_object(map: &HashMap<String, Value>, inspector: &mut Inspector) -> fmt::Result {
    if inspector.at_max_depth() {
        return write!(inspector, "{{{} fields}}", map.len());
    }
    
    if !inspector.enter(map as *const _ as *const ()) {
        return write!(inspector, "{{<circular>}}");
    }
    
    write!(inspector, "{{")?;
    inspect_map_items(map.iter(), inspector, true)?;
    inspector.exit();
    write!(inspector, "}}")
}

// Helper for HashMap inspection (complexity: ~10)
fn inspect_hashmap(map: &HashMap<Value, Value>, inspector: &mut Inspector) -> fmt::Result {
    if inspector.at_max_depth() {
        return write!(inspector, "HashMap{{{} entries}}", map.len());
    }
    
    if !inspector.enter(map as *const _ as *const ()) {
        return write!(inspector, "HashMap{{<circular>}}");
    }
    
    write!(inspector, "HashMap{{")?;
    inspect_hashmap_items(map.iter(), inspector)?;
    inspector.exit();
    write!(inspector, "}}")
}

// Helper for HashSet inspection (complexity: ~10)
fn inspect_hashset(set: &HashSet<Value>, inspector: &mut Inspector) -> fmt::Result {
    if inspector.at_max_depth() {
        return write!(inspector, "HashSet{{{} items}}", set.len());
    }
    
    if !inspector.enter(set as *const _ as *const ()) {
        return write!(inspector, "HashSet{{<circular>}}");
    }
    
    write!(inspector, "HashSet{{")?;
    inspect_collection_items(set.iter(), inspector)?;
    inspector.exit();
    write!(inspector, "}}")
}

// Generic collection item inspector (complexity: ~8)
fn inspect_collection_items<'a, I>(items: I, inspector: &mut Inspector) -> fmt::Result
where
    I: Iterator<Item = &'a Value> + ExactSizeIterator,
{
    let total = items.len();
    let display_count = total.min(inspector.style.max_elements);
    
    for (i, item) in items.take(display_count).enumerate() {
        if i > 0 {
            write!(inspector, ", ")?;
        }
        item.inspect(inspector)?;
        
        if !inspector.has_budget() {
            write!(inspector, ", ...")?;
            break;
        }
    }
    
    if total > display_count {
        write!(inspector, ", ...{} more", total - display_count)?;
    }
    
    Ok(())
}

// Map item inspector (complexity: ~10)
fn inspect_map_items<'a, I>(items: I, inspector: &mut Inspector, string_keys: bool) -> fmt::Result
where
    I: Iterator<Item = (&'a String, &'a Value)> + ExactSizeIterator,
{
    let total = items.len();
    let display_count = total.min(inspector.style.max_elements);
    
    for (i, (key, value)) in items.take(display_count).enumerate() {
        if i > 0 {
            write!(inspector, ", ")?;
        }
        if string_keys {
            write!(inspector, "\"{key}\": ")?;
        } else {
            write!(inspector, "{key}: ")?;
        }
        value.inspect(inspector)?;
        
        if !inspector.has_budget() {
            write!(inspector, ", ...")?;
            break;
        }
    }
    
    if total > display_count {
        write!(inspector, ", ...{} more", total - display_count)?;
    }
    
    Ok(())
}

// HashMap item inspector (complexity: ~10)
fn inspect_hashmap_items<'a, I>(items: I, inspector: &mut Inspector) -> fmt::Result
where
    I: Iterator<Item = (&'a Value, &'a Value)> + ExactSizeIterator,
{
    let total = items.len();
    let display_count = total.min(inspector.style.max_elements);
    
    for (i, (key, value)) in items.take(display_count).enumerate() {
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
    
    if total > display_count {
        write!(inspector, ", ...{} more", total - display_count)?;
    }
    
    Ok(())
}

// Simple type inspectors (complexity: 1-3 each)
fn inspect_function(name: &str, params: &[String], inspector: &mut Inspector) -> fmt::Result {
    write!(inspector, "fn {}({})", name, params.join(", "))
}

fn inspect_lambda(params: &[String], inspector: &mut Inspector) -> fmt::Result {
    write!(inspector, "|{}| <closure>", params.join(", "))
}

fn inspect_range(start: &i64, end: &i64, inclusive: bool, inspector: &mut Inspector) -> fmt::Result {
    if inclusive {
        write!(inspector, "{start}..={end}")
    } else {
        write!(inspector, "{start}..{end}")
    }
}

fn inspect_enum_variant(
    enum_name: &str, 
    variant_name: &str, 
    data: &Option<Vec<Value>>, 
    inspector: &mut Inspector
) -> fmt::Result {
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

fn inspect_dataframe(columns: &HashMap<String, Vec<Value>>, inspector: &mut Inspector) -> fmt::Result {
    write!(inspector, "DataFrame[{} columns]", columns.len())
}

// Depth calculation helpers (complexity: 3-5 each)
fn calc_list_depth(items: &[Value]) -> usize {
    1 + items.iter()
        .map(|v| v.inspect_depth_refactored())
        .max()
        .unwrap_or(0)
}

fn calc_tuple_depth(items: &[Value]) -> usize {
    1 + items.iter()
        .map(|v| v.inspect_depth_refactored())
        .max()
        .unwrap_or(0)
}

fn calc_object_depth(map: &HashMap<String, Value>) -> usize {
    1 + map.values()
        .map(|v| v.inspect_depth_refactored())
        .max()
        .unwrap_or(0)
}

fn calc_hashmap_depth(map: &HashMap<Value, Value>) -> usize {
    let key_depth = map.keys()
        .map(|v| v.inspect_depth_refactored())
        .max()
        .unwrap_or(0);
    let val_depth = map.values()
        .map(|v| v.inspect_depth_refactored())
        .max()
        .unwrap_or(0);
    1 + key_depth.max(val_depth)
}

fn calc_hashset_depth(set: &HashSet<Value>) -> usize {
    1 + set.iter()
        .map(|v| v.inspect_depth_refactored())
        .max()
        .unwrap_or(0)
}

fn calc_enum_variant_depth(data: &Option<Vec<Value>>) -> usize {
    if let Some(values) = data {
        1 + values.iter()
            .map(|v| v.inspect_depth_refactored())
            .max()
            .unwrap_or(0)
    } else {
        1
    }
}