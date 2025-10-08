//! Complete mutation coverage for eval_string_methods.rs
//! TOYOTA WAY: Stop the line - fix ALL 20 MISSED mutations
//!
//! This test file catches all mutations found by cargo-mutants

use ruchy::runtime::eval_string_methods::eval_string_method;
use ruchy::runtime::Value;
use std::rc::Rc;

// MISSED: delete match arm "to_string"
#[test]
fn test_to_string_method() {
    let s = Rc::from("hello");
    let result = eval_string_method(&s, "to_string", &[]).unwrap();
    assert!(matches!(result, Value::String(_)));
}

// MISSED: delete match arm "chars"
#[test]
fn test_chars_method() {
    let s = Rc::from("hi");
    let result = eval_string_method(&s, "chars", &[]).unwrap();
    assert!(matches!(result, Value::Array(_)));
}

// MISSED: delete match arm "trim"
#[test]
fn test_trim_method() {
    let s = Rc::from("  hello  ");
    let result = eval_string_method(&s, "trim", &[]).unwrap();
    if let Value::String(trimmed) = result {
        assert_eq!(&*trimmed, "hello");
    } else {
        panic!("Expected String");
    }
}

// MISSED: delete match arm "trim_start"
#[test]
fn test_trim_start_method() {
    let s = Rc::from("  hello");
    let result = eval_string_method(&s, "trim_start", &[]).unwrap();
    if let Value::String(trimmed) = result {
        assert_eq!(&*trimmed, "hello");
    } else {
        panic!("Expected String");
    }
}

// MISSED: delete match arm "trim_end"
#[test]
fn test_trim_end_method() {
    let s = Rc::from("hello  ");
    let result = eval_string_method(&s, "trim_end", &[]).unwrap();
    if let Value::String(trimmed) = result {
        assert_eq!(&*trimmed, "hello");
    } else {
        panic!("Expected String");
    }
}

// MISSED: delete match arm "lines"
#[test]
fn test_lines_method() {
    let s = Rc::from("line1\nline2");
    let result = eval_string_method(&s, "lines", &[]).unwrap();

    // Must verify actual content, not just type
    if let Value::Array(lines) = result {
        assert_eq!(lines.len(), 2, "Should have 2 lines");
        if let Value::String(line1) = &lines[0] {
            assert_eq!(&**line1, "line1");
        } else {
            panic!("First line should be String");
        }
        if let Value::String(line2) = &lines[1] {
            assert_eq!(&**line2, "line2");
        } else {
            panic!("Second line should be String");
        }
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

// MISSED: delete match arm "is_empty"
#[test]
fn test_is_empty_method() {
    let s = Rc::from("");
    let result = eval_string_method(&s, "is_empty", &[]).unwrap();
    assert_eq!(result, Value::Bool(true));

    let s2 = Rc::from("not empty");
    let result2 = eval_string_method(&s2, "is_empty", &[]).unwrap();
    assert_eq!(result2, Value::Bool(false));
}

// MISSED: delete match arm "starts_with"
#[test]
fn test_starts_with_method() {
    let s = Rc::from("hello world");
    let arg = Value::from_string("hello".to_string());
    let result = eval_string_method(&s, "starts_with", &[arg]).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// MISSED: delete match arm "ends_with"
#[test]
fn test_ends_with_method() {
    let s = Rc::from("hello world");
    let arg = Value::from_string("world".to_string());
    let result = eval_string_method(&s, "ends_with", &[arg]).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// MISSED: delete match arm "char_at"
#[test]
fn test_char_at_method() {
    let s = Rc::from("hello");
    let arg = Value::Integer(1);
    let result = eval_string_method(&s, "char_at", &[arg]).unwrap();
    if let Value::String(ch) = result {
        assert_eq!(&*ch, "e");
    } else {
        panic!("Expected String");
    }
}

// MISSED: delete match arm "replace"
#[test]
fn test_replace_method() {
    let s = Rc::from("hello world");
    let from = Value::from_string("world".to_string());
    let to = Value::from_string("rust".to_string());
    let result = eval_string_method(&s, "replace", &[from, to]).unwrap();
    if let Value::String(replaced) = result {
        assert_eq!(&*replaced, "hello rust");
    } else {
        panic!("Expected String");
    }
}

// MISSED: replace >= with < in char_at (boundary check)
#[test]
fn test_char_at_boundary() {
    let s = Rc::from("hi");

    // Valid index 0 - must verify it works
    let result = eval_string_method(&s, "char_at", &[Value::Integer(0)]).unwrap();
    if let Value::String(ch) = result {
        assert_eq!(&*ch, "h", "char_at(0) should return 'h'");
    } else {
        panic!("Expected String, got {:?}", result);
    }

    // Valid index 1 - must verify it works
    let result2 = eval_string_method(&s, "char_at", &[Value::Integer(1)]).unwrap();
    if let Value::String(ch) = result2 {
        assert_eq!(&*ch, "i", "char_at(1) should return 'i'");
    } else {
        panic!("Expected String, got {:?}", result2);
    }

    // Invalid index (should return Nil or error, NOT panic)
    let result_invalid = eval_string_method(&s, "char_at", &[Value::Integer(10)]).unwrap();
    assert_eq!(
        result_invalid,
        Value::Nil,
        "Out-of-bounds should return Nil"
    );
}

// MISSED: replace && with || in substring (line 206)
#[test]
fn test_substring_logic() {
    let s = Rc::from("hello");

    // Valid substring (catches correct && logic)
    let result =
        eval_string_method(&s, "substring", &[Value::Integer(1), Value::Integer(3)]).unwrap();
    if let Value::String(substr) = result {
        assert_eq!(&*substr, "el", "substring(1, 3) should return 'el'");
    } else {
        panic!("Expected String, got {:?}", result);
    }

    // Invalid: negative start (should error with &&, but might succeed with ||)
    let result_neg = eval_string_method(&s, "substring", &[Value::Integer(-1), Value::Integer(2)]);
    assert!(
        result_neg.is_err(),
        "Negative start_idx should error (catches && mutation)"
    );

    // Invalid: end < start (should error with &&, but might succeed with ||)
    let result_backwards =
        eval_string_method(&s, "substring", &[Value::Integer(3), Value::Integer(1)]);
    assert!(
        result_backwards.is_err(),
        "end < start should error (catches && mutation)"
    );
}

// MISSED: delete match arm Value::Integer(n) in eval_primitive_method (line 259)
#[test]
fn test_integer_to_string() {
    use ruchy::runtime::eval_string_methods::eval_primitive_method;

    // This calls eval_primitive_method → eval_integer_method (line 259)
    let num = Value::Integer(42);
    let result = eval_primitive_method(&num, "to_string", &[], true).unwrap();

    if let Value::String(s) = result {
        assert_eq!(&*s, "42", "Integer.to_string() should work");
    } else {
        panic!("Expected String, got {:?}", result);
    }
}
