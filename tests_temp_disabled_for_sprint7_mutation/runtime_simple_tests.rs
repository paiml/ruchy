//! Simple tests for runtime modules
//!
//! [TEST-COV-012] Basic runtime test coverage

use ruchy::frontend::ast::{Expr, ExprKind, Literal};
use ruchy::runtime::Value;
use std::rc::Rc;
use std::rc::Rc;

#[test]
fn test_value_int() {
    let val = Value::Integer(42);
    assert_eq!(val.to_string(), "42");
    assert_eq!(val, Value::Integer(42));
    assert_ne!(val, Value::Integer(43));
}

#[test]
fn test_value_float() {
    let val = Value::Float(3.14);
    assert_eq!(val.to_string(), "3.14");
    assert_eq!(val, Value::Float(3.14));
    assert_ne!(val, Value::Float(2.71));
}

#[test]
fn test_value_bool() {
    let val_true = Value::Bool(true);
    let val_false = Value::Bool(false);

    assert_eq!(val_true.to_string(), "true");
    assert_eq!(val_false.to_string(), "false");
    assert_eq!(val_true, Value::Bool(true));
    assert_ne!(val_true, val_false);
}

#[test]
fn test_value_string() {
    let val = Value::String(Rc::from("hello"));
    assert_eq!(val.to_string(), "hello");
    assert_eq!(val, Value::String(Rc::from("hello")));
    assert_ne!(val, Value::String(Rc::from("world")));
}

#[test]
fn test_value_nil() {
    let val = Value::Nil;
    assert_eq!(val.to_string(), "null");
    assert_eq!(val, Value::Nil);
    assert_ne!(val, Value::Unit);
}

#[test]
fn test_value_unit() {
    let val = Value::Unit;
    assert_eq!(val.to_string(), "()");
    assert_eq!(val, Value::Unit);
    assert_ne!(val, Value::Nil);
}

#[test]
fn test_value_list() {
    let val = Value::Array(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    assert_eq!(val.to_string(), "[1, 2, 3]");

    let val2 = Value::Array(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    assert_eq!(val, val2);

    let val3 = Value::Array(vec![Value::Integer(1)]);
    assert_ne!(val, val3);
}

#[test]
fn test_value_tuple() {
    let val = Value::Tuple(vec![Value::Integer(10), Value::String(Rc::from("test"))]);
    assert_eq!(val.to_string(), "(10, \"test\")");

    let val2 = Value::Tuple(vec![Value::Integer(10), Value::String(Rc::from("test"))]);
    assert_eq!(val, val2);
}

#[test]
fn test_value_char() {
    let val = Value::Char('A');
    assert_eq!(val.to_string(), "'A'");
    assert_eq!(val, Value::Char('A'));
    assert_ne!(val, Value::Char('B'));
}

#[test]
fn test_value_range() {
    let val = Value::Range {
        start: 1,
        end: 5,
        inclusive: false,
    };
    assert_eq!(val.to_string(), "1..5");

    let val2 = Value::Range {
        start: 1,
        end: 5,
        inclusive: true,
    };
    assert_eq!(val2.to_string(), "1..=5");
    assert_ne!(val, val2);
}

#[test]
fn test_value_object() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String(Rc::from("Alice")));
    map.insert("age".to_string(), Value::Integer(30));

    let val = Value::Object(map);
    let display = val.to_string();
    assert!(display.contains("name"));
    assert!(display.contains("Alice"));
    assert!(display.contains("age"));
    assert!(display.contains("30"));
}

#[test]
fn test_value_hashmap() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(Value::String(Rc::from("key1")), Value::Integer(100));
    map.insert(Value::String(Rc::from("key2")), Value::Integer(200));

    let val = Value::HashMap(map);
    let display = val.to_string();
    assert!(display.contains("HashMap"));
    assert!(display.contains("key1"));
    assert!(display.contains("100"));
}

#[test]
fn test_value_hashset() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Value::Integer(1));
    set.insert(Value::Integer(2));
    set.insert(Value::Integer(3));

    let val = Value::HashSet(set);
    let display = val.to_string();
    assert!(display.contains("HashSet"));
}

#[test]
fn test_value_function() {
    let val = Value::Function {
        name: "test_func".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        body: Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Default::default(),
        )),
    };

    assert_eq!(val.to_string(), "fn test_func(a, b)");
}

#[test]
fn test_value_lambda() {
    let val = Value::Lambda {
        params: vec!["x".to_string()],
        body: Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Default::default(),
        )),
    };

    assert_eq!(val.to_string(), "|x| <closure>");
}

#[test]
fn test_value_enum_variant() {
    let val = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };
    assert_eq!(val.to_string(), "Option::Some(42)");

    let val2 = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "None".to_string(),
        data: None,
    };
    assert_eq!(val2.to_string(), "Option::None");
}

// DataFrame test removed - complex internal structure

#[test]
fn test_value_clone() {
    let val1 = Value::Integer(42);
    let val2 = val1.clone();
    assert_eq!(val1, val2);

    let val3 = Value::String(Rc::from("test"));
    let val4 = val3.clone();
    assert_eq!(val3, val4);

    let val5 = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
    let val6 = val5.clone();
    assert_eq!(val5, val6);
}

#[test]
fn test_value_debug() {
    let val = Value::Integer(42);
    let debug = format!("{val:?}");
    assert!(debug.contains("Int"));
    assert!(debug.contains("42"));

    let val2 = Value::String(Rc::from("hello"));
    let debug2 = format!("{val2:?}");
    assert!(debug2.contains("String"));
    assert!(debug2.contains("hello"));
}
