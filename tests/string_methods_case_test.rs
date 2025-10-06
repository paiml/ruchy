//! String case conversion method tests - HYBRID-C-1
//!
//! EXTREME TDD: Tests written FIRST, implementation follows
//! Ensures to_uppercase() and to_lowercase() methods work

use ruchy::runtime::eval_string_methods::eval_string_method;
use ruchy::runtime::Value;
use std::rc::Rc;

#[test]
fn test_to_uppercase_method() {
    let s = Rc::from("hello");
    let result = eval_string_method(&s, "to_uppercase", &[]).unwrap();

    match result {
        Value::String(upper) => assert_eq!(&*upper, "HELLO"),
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_to_lowercase_method() {
    let s = Rc::from("WORLD");
    let result = eval_string_method(&s, "to_lowercase", &[]).unwrap();

    match result {
        Value::String(lower) => assert_eq!(&*lower, "world"),
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_to_uppercase_empty_string() {
    let s = Rc::from("");
    let result = eval_string_method(&s, "to_uppercase", &[]).unwrap();

    match result {
        Value::String(upper) => assert_eq!(&*upper, ""),
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_to_lowercase_mixed_case() {
    let s = Rc::from("HeLLo WoRLd");
    let result = eval_string_method(&s, "to_lowercase", &[]).unwrap();

    match result {
        Value::String(lower) => assert_eq!(&*lower, "hello world"),
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_to_uppercase_unicode() {
    let s = Rc::from("café");
    let result = eval_string_method(&s, "to_uppercase", &[]).unwrap();

    match result {
        Value::String(upper) => assert_eq!(&*upper, "CAFÉ"),
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_case_conversion_idempotent() {
    let s = Rc::from("hello");
    let upper1 = eval_string_method(&s, "to_uppercase", &[]).unwrap();

    if let Value::String(upper_str) = upper1 {
        let upper2 = eval_string_method(&upper_str, "to_uppercase", &[]).unwrap();

        if let Value::String(upper2_str) = upper2 {
            assert_eq!(
                &*upper_str, &*upper2_str,
                "to_uppercase should be idempotent"
            );
        } else {
            panic!("Second uppercase failed");
        }
    } else {
        panic!("First uppercase failed");
    }
}

// Property tests with 10K cases (EXTREME TDD requirement)
#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn uppercase_idempotent(s in ".*") {
            let rc_s = Rc::from(s.as_str());
            let upper1 = eval_string_method(&rc_s, "to_uppercase", &[]).unwrap();

            if let Value::String(upper_str) = upper1 {
                let upper2 = eval_string_method(&upper_str, "to_uppercase", &[]).unwrap();

                if let Value::String(upper2_str) = upper2 {
                    prop_assert_eq!(&*upper_str, &*upper2_str);
                }
            }
        }

        #[test]
        fn lowercase_idempotent(s in ".*") {
            let rc_s = Rc::from(s.as_str());
            let lower1 = eval_string_method(&rc_s, "to_lowercase", &[]).unwrap();

            if let Value::String(lower_str) = lower1 {
                let lower2 = eval_string_method(&lower_str, "to_lowercase", &[]).unwrap();

                if let Value::String(lower2_str) = lower2 {
                    prop_assert_eq!(&*lower_str, &*lower2_str);
                }
            }
        }

        #[test]
        fn case_conversion_never_panics(s in ".*") {
            let rc_s = Rc::from(s.as_str());
            let _ = eval_string_method(&rc_s, "to_uppercase", &[]);
            let _ = eval_string_method(&rc_s, "to_lowercase", &[]);
        }
    }
}
