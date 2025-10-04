// EXTREME TDD: Runtime Value Tests
// Sprint 80: Final push to 80% coverage
// Testing runtime::Value comprehensively

use ruchy::runtime::Value;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod value_tests {
    use super::*;

    #[test]
    fn test_value_integer() {
        let val1 = Value::Integer(42);
        let val2 = Value::Integer(42);
        let val3 = Value::Integer(100);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
        assert_eq!(format!("{}", val1), "42");
    }

    #[test]
    fn test_value_float() {
        let val1 = Value::Float(3.14);
        let val2 = Value::Float(3.14);
        let val3 = Value::Float(2.71);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
        assert_eq!(format!("{}", val1), "3.14");
    }

    #[test]
    fn test_value_bool() {
        let val1 = Value::Bool(true);
        let val2 = Value::Bool(true);
        let val3 = Value::Bool(false);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
        assert_eq!(format!("{}", val1), "true");
        assert_eq!(format!("{}", val3), "false");
    }

    #[test]
    fn test_value_nil() {
        let val1 = Value::Nil;
        let val2 = Value::Nil;

        assert_eq!(val1, val2);
        assert_eq!(format!("{}", val1), "nil");
    }

    #[test]
    fn test_value_string() {
        let val1 = Value::String(Rc::from("hello"));
        let val2 = Value::String(Rc::from("hello"));
        let val3 = Value::String(Rc::from("world"));

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
        assert_eq!(format!("{}", val1), "hello");
    }

    #[test]
    fn test_value_array() {
        let val1 = Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        let val2 = Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        let val3 = Value::Array(vec![Value::Integer(4), Value::Integer(5)].into());

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_value_tuple() {
        let val1 = Value::Tuple(vec![Value::Integer(42), Value::Bool(true)].into());

        let val2 = Value::Tuple(vec![Value::Integer(42), Value::Bool(true)].into());

        let val3 = Value::Tuple(vec![Value::Integer(42), Value::Bool(false)].into());

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_value_object() {
        let mut map1 = HashMap::new();
        map1.insert("x".to_string(), Value::Integer(10));
        map1.insert("y".to_string(), Value::Integer(20));

        let mut map2 = HashMap::new();
        map2.insert("x".to_string(), Value::Integer(10));
        map2.insert("y".to_string(), Value::Integer(20));

        let mut map3 = HashMap::new();
        map3.insert("x".to_string(), Value::Integer(30));

        let val1 = Value::Object(Rc::new(map1));
        let val2 = Value::Object(Rc::new(map2));
        let val3 = Value::Object(Rc::new(map3));

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_value_clone() {
        let values = vec![
            Value::Integer(42),
            Value::Float(3.14),
            Value::Bool(true),
            Value::Nil,
            Value::String(Rc::from("test")),
            Value::Array(vec![Value::Integer(1)].into()),
            Value::Tuple(vec![Value::Bool(true)].into()),
        ];

        for val in values {
            let cloned = val.clone();
            assert_eq!(val, cloned);
        }
    }

    #[test]
    fn test_value_debug() {
        let val = Value::Integer(42);
        let debug = format!("{:?}", val);
        assert!(debug.contains("Integer"));

        let val = Value::String(Rc::from("test"));
        let debug = format!("{:?}", val);
        assert!(debug.contains("String"));
    }

    #[test]
    fn test_nested_arrays() {
        let inner1 = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
        let inner2 = Value::Array(vec![Value::Integer(3), Value::Integer(4)].into());

        let nested = Value::Array(vec![inner1, inner2].into());

        if let Value::Array(outer) = &nested {
            assert_eq!(outer.len(), 2);
            if let Value::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
                assert_eq!(inner[0], Value::Integer(1));
            }
        }
    }

    #[test]
    fn test_mixed_nested_structures() {
        let array = Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::String(Rc::from("nested")),
        ]));

        let tuple = Value::Tuple(vec![Value::Bool(true), array].into());

        if let Value::Tuple(tup) = &tuple {
            assert_eq!(tup.len(), 2);
            assert_eq!(tup[0], Value::Bool(true));
            if let Value::Array(arr) = &tup[1] {
                assert_eq!(arr.len(), 2);
            }
        }
    }

    #[test]
    fn test_large_array() {
        let mut elements = vec![];
        for i in 0..1000 {
            elements.push(Value::Integer(i));
        }

        let large_array = Value::Array(Rc::new(elements));

        if let Value::Array(arr) = &large_array {
            assert_eq!(arr.len(), 1000);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[999], Value::Integer(999));
        }
    }

    #[test]
    fn test_large_object() {
        let mut map = HashMap::new();
        for i in 0..100 {
            map.insert(format!("key_{}", i), Value::Integer(i));
        }

        let large_obj = Value::Object(Rc::new(map));

        if let Value::Object(obj) = &large_obj {
            assert_eq!(obj.len(), 100);
            assert_eq!(obj.get("key_0"), Some(&Value::Integer(0)));
            assert_eq!(obj.get("key_99"), Some(&Value::Integer(99)));
        }
    }

    #[test]
    fn test_rc_memory_sharing() {
        let s = Rc::new("shared".to_string());
        let val1 = Value::String(Rc::clone(&s));
        let val2 = Value::String(Rc::clone(&s));

        // Both values share the same underlying string
        if let (Value::String(s1), Value::String(s2)) = (&val1, &val2) {
            assert!(Rc::ptr_eq(s1, s2));
        }
    }

    #[test]
    fn test_value_type_checking() {
        let val = Value::Integer(42);
        assert!(matches!(val, Value::Integer(_)));
        assert!(!matches!(val, Value::Float(_)));

        let val = Value::String(Rc::from("test"));
        assert!(matches!(val, Value::String(_)));
        assert!(!matches!(val, Value::Integer(_)));
    }

    #[test]
    fn test_empty_collections() {
        let empty_array = Value::Array(vec![].into());
        if let Value::Array(arr) = &empty_array {
            assert_eq!(arr.len(), 0);
        }

        let empty_tuple = Value::Tuple(vec![].into());
        if let Value::Tuple(tup) = &empty_tuple {
            assert_eq!(tup.len(), 0);
        }

        let empty_object = Value::Object(Rc::new(HashMap::new()));
        if let Value::Object(obj) = &empty_object {
            assert_eq!(obj.len(), 0);
        }
    }
}

// Property-based tests
#[cfg(test)]
mod value_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_integer_equality(n: i64) {
            let val1 = Value::Integer(n);
            let val2 = Value::Integer(n);
            prop_assert_eq!(val1, val2);
        }

        #[test]
        fn test_float_display(f: f64) {
            prop_assume!(f.is_finite());
            let val = Value::Float(f);
            let displayed = format!("{}", val);
            prop_assert!(!displayed.is_empty());
        }

        #[test]
        fn test_string_equality(s: String) {
            let val1 = Value::String(Rc::new(s.clone()));
            let val2 = Value::String(Rc::new(s.clone()));
            prop_assert_eq!(val1, val2);
        }

        #[test]
        fn test_array_length(len: usize) {
            prop_assume!(len < 1000);
            let mut elements = vec![];
            for i in 0..len {
                elements.push(Value::Integer(i as i64));
            }

            let array = Value::Array(Rc::new(elements));

            if let Value::Array(arr) = array {
                prop_assert_eq!(arr.len(), len);
            }
        }

        #[test]
        fn test_value_clone_equality(n: i64) {
            let val = Value::Integer(n);
            let cloned = val.clone();
            prop_assert_eq!(val, cloned);
        }
    }
}

// Stress tests
#[cfg(test)]
mod value_stress_tests {
    use super::*;

    #[test]
    #[ignore] // Expensive
    fn test_deeply_nested_arrays() {
        let mut current = Value::Integer(0);

        for i in 1..100 {
            current = Value::Array(vec![current, Value::Integer(i)].into());
        }

        // Should not stack overflow
        let _ = format!("{:?}", current);
    }

    #[test]
    #[ignore] // Expensive
    fn test_massive_object() {
        let mut map = HashMap::new();

        for i in 0..10_000 {
            map.insert(
                format!("very_long_key_name_number_{}", i),
                Value::String(Rc::new(format!("value_{}", i))),
            );
        }

        let huge_obj = Value::Object(Rc::new(map));

        if let Value::Object(obj) = &huge_obj {
            assert_eq!(obj.len(), 10_000);
        }
    }
}
