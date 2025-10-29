//! Property Tests for Arc Refactoring (DEFECT-001-A-TICKET-1)
//!
//! Purpose: Prove that Arc-based Values behave identically to Rc-based Values
//! Ticket: DEFECT-001-A-TICKET-1
//! Estimate: 2 hours
//! Target: 10,000+ property test iterations pass
//!
//! These tests verify the semantics of Value operations remain unchanged
//! when refactoring from Rc<T> to Arc<T> for thread-safety.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use ruchy::runtime::interpreter::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Generate arbitrary Value instances for property testing
fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        any::<i64>().prop_map(Value::Integer),
        any::<f64>().prop_map(|f| {
            if f.is_finite() {
                Value::Float(f)
            } else {
                Value::Float(0.0) // Replace NaN/Inf with 0.0
            }
        }),
        any::<bool>().prop_map(Value::Bool),
        any::<u8>().prop_map(Value::Byte),
        Just(Value::Nil),
        "[a-zA-Z0-9]{0,20}".prop_map(|s| Value::String(Arc::from(s.as_str()))),
        "[a-zA-Z_][a-zA-Z0-9_]{0,20}".prop_map(Value::BuiltinFunction),
    ];

    leaf.prop_recursive(
        2,  // levels deep
        10, // max size
        5,  // items per collection
        |inner| {
            prop_oneof![
                // Arrays
                prop::collection::vec(inner.clone(), 0..3)
                    .prop_map(|v| { Value::Array(Arc::from(v.into_boxed_slice())) }),
                // Tuples
                prop::collection::vec(inner.clone(), 0..3)
                    .prop_map(|v| { Value::Tuple(Arc::from(v.into_boxed_slice())) }),
                // Objects
                prop::collection::hash_map("[a-zA-Z][a-zA-Z0-9]{0,10}", inner.clone(), 0..3)
                    .prop_map(|m| Value::Object(Arc::new(m))),
                // EnumVariant
                (
                    "[A-Z][a-zA-Z0-9]{0,10}", // enum_name
                    "[A-Z][a-zA-Z0-9]{0,10}", // variant_name
                    prop::option::of(prop::collection::vec(inner.clone(), 0..2))
                )
                    .prop_map(|(enum_name, variant_name, data)| Value::EnumVariant {
                        enum_name,
                        variant_name,
                        data,
                    }),
                // Range
                (inner.clone(), inner.clone(), any::<bool>()).prop_map(|(start, end, incl)| {
                    Value::Range {
                        start: Box::new(start),
                        end: Box::new(end),
                        inclusive: incl,
                    }
                }),
            ]
        },
    )
}

/// Generate arbitrary HashMap<String, Value> for environment testing
fn arb_env() -> impl Strategy<Value = HashMap<String, Value>> {
    prop::collection::hash_map("[a-z][a-z0-9_]{0,10}", arb_value(), 0..5)
}

proptest! {
    /// Test that Value cloning preserves equality
    ///
    /// Property: For all values v, v.clone() == v
    ///
    /// This is the CORE property - Arc must behave like Rc for cloning.
    #[test]
    fn test_value_clone_equivalence(value in arb_value()) {
        let cloned = value.clone();

        // This property MUST hold after Rc → Arc refactoring
        match (&value, &cloned) {
            (Value::Float(a), Value::Float(b)) => {
                // Handle NaN specially (NaN != NaN)
                if a.is_nan() && b.is_nan() {
                    prop_assert!(true);
                } else {
                    prop_assert_eq!(a, b, "Float values should be equal");
                }
            }
            _ => {
                // For all other types, use PartialEq
                // Note: Value doesn't implement Eq due to Float
                prop_assert!(
                    value_eq(&value, &cloned),
                    "Cloned value should equal original"
                );
            }
        }
    }

    /// Test that Value is Send after Arc refactoring
    ///
    /// Property: All Value instances can cross thread boundaries
    ///
    /// This property will FAIL with Rc, PASS with Arc.
    /// This is the smoking gun test for the refactoring.
    ///
    /// CURRENTLY COMMENTED OUT: Cannot compile with Rc-based Values
    /// Will uncomment after TICKET-2 (Arc refactoring) is complete.
    /*
    #[test]
    fn test_value_thread_safety(value in arb_value()) {
        // Try to send value across thread boundary
        let handle = std::thread::spawn(move || {
            // Just clone and drop it - proves Send works
            let _ = value.clone();
            42
        });

        let result = handle.join().expect("Thread should not panic");
        prop_assert_eq!(result, 42);
    }
    */

    /// Test that Value cloning is idempotent
    ///
    /// Property: For all values v, v.clone().clone() == v.clone()
    #[test]
    fn test_value_clone_idempotent(value in arb_value()) {
        let clone1 = value.clone();
        let clone2 = clone1.clone();

        prop_assert!(
            value_eq(&clone1, &clone2),
            "Double clone should equal single clone"
        );
    }

    /// Test that Array operations preserve semantics
    ///
    /// Property: Array cloning creates independent copies (Rc semantics)
    #[test]
    fn test_array_clone_semantics(values in prop::collection::vec(arb_value(), 0..5)) {
        let array = Value::Array(Arc::from(values.clone().into_boxed_slice()));
        let cloned = array.clone();

        // After cloning, both should have same values
        if let (Value::Array(a), Value::Array(b)) = (&array, &cloned) {
            prop_assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b.iter()) {
                prop_assert!(value_eq(x, y));
            }
        } else {
            panic!("Expected Array values");
        }
    }

    /// Test that Object operations preserve semantics
    ///
    /// Property: Object cloning creates independent copies (Rc semantics)
    #[test]
    fn test_object_clone_semantics(
        map in prop::collection::hash_map("[a-z][a-z0-9]{0,5}", arb_value(), 0..5)
    ) {
        let object = Value::Object(Arc::new(map.clone()));
        let cloned = object.clone();

        // After cloning, both should have same key-value pairs
        if let (Value::Object(a), Value::Object(b)) = (&object, &cloned) {
            prop_assert_eq!(a.len(), b.len());
            for (key, val_a) in a.iter() {
                if let Some(val_b) = b.get(key) {
                    prop_assert!(value_eq(val_a, val_b));
                } else {
                    panic!("Key {:?} missing in cloned object", key);
                }
            }
        } else {
            panic!("Expected Object values");
        }
    }

    /// Test that String operations preserve semantics
    ///
    /// Property: String cloning preserves content (Rc semantics)
    #[test]
    fn test_string_clone_semantics(s in "[a-zA-Z0-9 ]{0,50}") {
        let value = Value::String(Arc::from(s.as_str()));
        let cloned = value.clone();

        if let (Value::String(a), Value::String(b)) = (&value, &cloned) {
            prop_assert_eq!(a.as_ref(), b.as_ref());
        } else {
            panic!("Expected String values");
        }
    }

    /// Test that Tuple operations preserve semantics
    ///
    /// Property: Tuple cloning preserves all elements (Rc semantics)
    #[test]
    fn test_tuple_clone_semantics(values in prop::collection::vec(arb_value(), 0..5)) {
        let tuple = Value::Tuple(Arc::from(values.clone().into_boxed_slice()));
        let cloned = tuple.clone();

        if let (Value::Tuple(a), Value::Tuple(b)) = (&tuple, &cloned) {
            prop_assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b.iter()) {
                prop_assert!(value_eq(x, y));
            }
        } else {
            panic!("Expected Tuple values");
        }
    }

    /// Test that EnumVariant operations preserve semantics
    ///
    /// Property: EnumVariant cloning preserves enum_name, variant_name and data
    #[test]
    fn test_enum_variant_semantics(
        enum_name in "[A-Z][a-zA-Z0-9]{0,10}",
        variant_name in "[A-Z][a-zA-Z0-9]{0,10}",
        data in prop::option::of(prop::collection::vec(arb_value(), 0..3))
    ) {
        let variant = Value::EnumVariant {
            enum_name: enum_name.clone(),
            variant_name: variant_name.clone(),
            data: data.clone(),
        };
        let cloned = variant.clone();

        if let (Value::EnumVariant { enum_name: e1, variant_name: n1, data: d1 },
                Value::EnumVariant { enum_name: e2, variant_name: n2, data: d2 }) = (&variant, &cloned) {
            prop_assert_eq!(e1, e2);
            prop_assert_eq!(n1, n2);
            match (d1, d2) {
                (None, None) => {},
                (Some(v1), Some(v2)) => {
                    prop_assert_eq!(v1.len(), v2.len());
                    for (a, b) in v1.iter().zip(v2.iter()) {
                        prop_assert!(value_eq(a, b));
                    }
                }
                _ => panic!("EnumVariant data mismatch"),
            }
        } else {
            panic!("Expected EnumVariant values");
        }
    }
}

/// Helper: Value equality check (handles Float NaN)
fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => {
            if x.is_nan() && y.is_nan() {
                true
            } else {
                x == y
            }
        }
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Byte(x), Value::Byte(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        (Value::String(x), Value::String(y)) => x.as_ref() == y.as_ref(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| value_eq(a, b))
        }
        (Value::Tuple(x), Value::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| value_eq(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v1)| y.get(k).map_or(false, |v2| value_eq(v1, v2)))
        }
        (Value::BuiltinFunction(x), Value::BuiltinFunction(y)) => x == y,
        (
            Value::EnumVariant {
                enum_name: e1,
                variant_name: n1,
                data: d1,
            },
            Value::EnumVariant {
                enum_name: e2,
                variant_name: n2,
                data: d2,
            },
        ) => {
            e1 == e2
                && n1 == n2
                && match (d1, d2) {
                    (None, None) => true,
                    (Some(v1), Some(v2)) => {
                        v1.len() == v2.len()
                            && v1.iter().zip(v2.iter()).all(|(a, b)| value_eq(a, b))
                    }
                    _ => false,
                }
        }
        (
            Value::Range {
                start: s1,
                end: e1,
                inclusive: i1,
            },
            Value::Range {
                start: s2,
                end: e2,
                inclusive: i2,
            },
        ) => i1 == i2 && value_eq(s1, s2) && value_eq(e1, e2),
        _ => false,
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Sanity check: Simple values clone correctly
    #[test]
    fn test_simple_value_cloning() {
        let int = Value::Integer(42);
        let cloned = int.clone();
        assert!(value_eq(&int, &cloned));

        let string = Value::String(Arc::from("hello"));
        let cloned = string.clone();
        assert!(value_eq(&string, &cloned));

        let nil = Value::Nil;
        let cloned = nil.clone();
        assert!(value_eq(&nil, &cloned));
    }

    /// Sanity check: Arrays clone correctly
    #[test]
    fn test_array_cloning() {
        let array = Value::Array(Arc::from(
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into_boxed_slice(),
        ));

        let cloned = array.clone();
        assert!(value_eq(&array, &cloned));
    }

    /// Sanity check: Objects clone correctly
    #[test]
    fn test_object_cloning() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), Value::Integer(10));
        map.insert("y".to_string(), Value::Integer(20));

        let object = Value::Object(Arc::new(map));
        let cloned = object.clone();
        assert!(value_eq(&object, &cloned));
    }
}
