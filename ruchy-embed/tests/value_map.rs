#![allow(missing_docs)]
//! EMBED-007 (map portion): `Value::Map` marshaling.
//!
//! Verifies that embed-side `Value::Map` round-trips through the interpreter
//! as `RuchyValue::Object`. Complements `value_containers.rs`.

use ruchy_embed::{Engine, Value};

fn get_key<'a>(map: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

#[test]
fn test_embed_007_map_host_roundtrip() {
    let mut engine = Engine::new();
    let original = Value::Map(vec![
        ("a".to_string(), Value::Integer(1)),
        ("b".to_string(), Value::String("two".to_string())),
        ("c".to_string(), Value::Bool(true)),
    ]);
    engine.set("m", original);

    let echoed = engine.get("m").expect("map global must be present");
    match echoed {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 3);
            assert!(matches!(get_key(entries, "a"), Some(Value::Integer(1))));
            assert!(matches!(get_key(entries, "b"), Some(Value::String(s)) if s == "two"));
            assert!(matches!(get_key(entries, "c"), Some(Value::Bool(true))));
        }
        other => panic!("expected Value::Map, got {other:?}"),
    }
}

#[test]
fn test_embed_007_map_empty() {
    let mut engine = Engine::new();
    engine.set("m", Value::Map(vec![]));
    let echoed = engine.get("m").expect("map global must be present");
    match echoed {
        Value::Map(entries) => assert!(entries.is_empty()),
        other => panic!("expected empty Value::Map, got {other:?}"),
    }
}

#[test]
fn test_embed_007_map_nested_list() {
    let mut engine = Engine::new();
    let original = Value::Map(vec![(
        "xs".to_string(),
        Value::List(vec![Value::Integer(10), Value::Integer(20)]),
    )]);
    engine.set("m", original);

    let echoed = engine.get("m").expect("map global must be present");
    match echoed {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 1);
            match get_key(entries, "xs") {
                Some(Value::List(xs)) => {
                    assert_eq!(xs.len(), 2);
                    assert!(matches!(xs[0], Value::Integer(10)));
                    assert!(matches!(xs[1], Value::Integer(20)));
                }
                other => panic!("expected nested Value::List, got {other:?}"),
            }
        }
        other => panic!("expected Value::Map, got {other:?}"),
    }
}
