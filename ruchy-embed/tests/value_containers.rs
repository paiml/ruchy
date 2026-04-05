#![allow(missing_docs)]
//! EMBED-007: Value container marshaling (List/Tuple).
//!
//! Verifies the embed-side `Value::List` and `Value::Tuple` variants
//! round-trip through the Ruchy interpreter as `RuchyValue::Array` and
//! `RuchyValue::Tuple` respectively, and that nested containers work.
//!
//! Ticket: EMBED-007 (Pillar 9 sub-spec Section 7).

use ruchy_embed::{Engine, Value};

#[test]
fn test_embed_007_list_literal_evaluates_to_list() {
    let mut engine = Engine::new();
    let result = engine.eval("[1, 2, 3]").expect("eval [1,2,3] must succeed");
    match result {
        Value::List(xs) => {
            assert_eq!(xs.len(), 3, "list must have 3 elements: {xs:?}");
            assert!(matches!(xs[0], Value::Integer(1)));
            assert!(matches!(xs[1], Value::Integer(2)));
            assert!(matches!(xs[2], Value::Integer(3)));
        }
        other => panic!("expected Value::List, got {other:?}"),
    }
}

#[test]
fn test_embed_007_empty_list_evaluates() {
    let mut engine = Engine::new();
    let result = engine.eval("[]").expect("eval [] must succeed");
    match result {
        Value::List(xs) => assert!(xs.is_empty(), "empty list must be empty: {xs:?}"),
        other => panic!("expected empty Value::List, got {other:?}"),
    }
}

#[test]
fn test_embed_007_heterogeneous_list() {
    let mut engine = Engine::new();
    let result = engine
        .eval("[1, \"two\", true]")
        .expect("eval heterogeneous list must succeed");
    match result {
        Value::List(xs) => {
            assert_eq!(xs.len(), 3);
            assert!(matches!(xs[0], Value::Integer(1)));
            assert!(matches!(&xs[1], Value::String(s) if s == "two"));
            assert!(matches!(xs[2], Value::Bool(true)));
        }
        other => panic!("expected Value::List, got {other:?}"),
    }
}

#[test]
fn test_embed_007_nested_list() {
    let mut engine = Engine::new();
    let result = engine
        .eval("[[1, 2], [3, 4]]")
        .expect("eval nested list must succeed");
    match result {
        Value::List(outer) => {
            assert_eq!(outer.len(), 2);
            match &outer[0] {
                Value::List(inner) => {
                    assert_eq!(inner.len(), 2);
                    assert!(matches!(inner[0], Value::Integer(1)));
                    assert!(matches!(inner[1], Value::Integer(2)));
                }
                other => panic!("expected nested Value::List, got {other:?}"),
            }
        }
        other => panic!("expected outer Value::List, got {other:?}"),
    }
}

#[test]
fn test_embed_007_list_passed_as_global_roundtrips() {
    // Set a List global, read it back -- host-side round-trip only.
    let mut engine = Engine::new();
    let original = Value::List(vec![
        Value::Integer(10),
        Value::Integer(20),
        Value::Integer(30),
    ]);
    engine.set("xs", original);
    let echoed = engine.get("xs").expect("global must be present");
    match echoed {
        Value::List(xs) => {
            assert_eq!(xs.len(), 3);
            assert!(matches!(xs[0], Value::Integer(10)));
            assert!(matches!(xs[1], Value::Integer(20)));
            assert!(matches!(xs[2], Value::Integer(30)));
        }
        other => panic!("expected Value::List, got {other:?}"),
    }
}

#[test]
fn test_embed_007_tuple_literal_evaluates_to_tuple() {
    let mut engine = Engine::new();
    let result = engine
        .eval("(1, \"two\", true)")
        .expect("eval (1,'two',true) must succeed");
    match result {
        Value::Tuple(xs) => {
            assert_eq!(xs.len(), 3);
            assert!(matches!(xs[0], Value::Integer(1)));
            assert!(matches!(&xs[1], Value::String(s) if s == "two"));
            assert!(matches!(xs[2], Value::Bool(true)));
        }
        other => panic!("expected Value::Tuple, got {other:?}"),
    }
}
