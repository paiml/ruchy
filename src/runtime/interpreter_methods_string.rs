//! String and array method dispatch
//!
//! Extracted from interpreter_methods.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::rc_buffer)]

use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

impl Interpreter {
    /// Evaluate string methods
    pub(crate) fn eval_string_method(
        &mut self,
        s: &Arc<str>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        super::eval_string_methods::eval_string_method(s, method, args)
    }

    /// Evaluate array methods
    #[allow(clippy::rc_buffer)]
    pub(crate) fn eval_array_method(
        &mut self,
        arr: &Arc<[Value]>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Delegate to extracted array module with function call capability
        crate::runtime::eval_array::eval_array_method(arr, method, args, |func, args| {
            self.eval_function_call_value(func, args)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // String method tests
    #[test]
    fn test_string_method_len() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("hello");
        let result = interp.eval_string_method(&s, "len", &[]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_string_method_to_uppercase() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("hello");
        let result = interp.eval_string_method(&s, "to_uppercase", &[]).unwrap();
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_string_method_to_lowercase() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("HELLO");
        let result = interp.eval_string_method(&s, "to_lowercase", &[]).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_method_trim() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("  hello  ");
        let result = interp.eval_string_method(&s, "trim", &[]).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_string_method_starts_with() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("hello world");
        let result = interp
            .eval_string_method(
                &s,
                "starts_with",
                &[Value::from_string("hello".to_string())],
            )
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_method_ends_with() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("hello world");
        let result = interp
            .eval_string_method(&s, "ends_with", &[Value::from_string("world".to_string())])
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_method_contains() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("hello world");
        let result = interp
            .eval_string_method(&s, "contains", &[Value::from_string("lo wo".to_string())])
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_method_replace() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("hello world");
        let result = interp
            .eval_string_method(
                &s,
                "replace",
                &[
                    Value::from_string("world".to_string()),
                    Value::from_string("rust".to_string()),
                ],
            )
            .unwrap();
        assert_eq!(result, Value::from_string("hello rust".to_string()));
    }

    #[test]
    fn test_string_method_split() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("a,b,c");
        let result = interp
            .eval_string_method(&s, "split", &[Value::from_string(",".to_string())])
            .unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
            assert_eq!(arr[1], Value::from_string("b".to_string()));
            assert_eq!(arr[2], Value::from_string("c".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_method_chars() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("abc");
        let result = interp.eval_string_method(&s, "chars", &[]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
            assert_eq!(arr[1], Value::from_string("b".to_string()));
            assert_eq!(arr[2], Value::from_string("c".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_string_method_is_empty() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("");
        let result = interp.eval_string_method(&s, "is_empty", &[]).unwrap();
        assert_eq!(result, Value::Bool(true));

        let s2: Arc<str> = Arc::from("hello");
        let result2 = interp.eval_string_method(&s2, "is_empty", &[]).unwrap();
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_string_method_parse_int() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("42");
        let result = interp.eval_string_method(&s, "parse", &[]).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_string_method_repeat() {
        let mut interp = make_interpreter();
        let s: Arc<str> = Arc::from("ab");
        let result = interp
            .eval_string_method(&s, "repeat", &[Value::Integer(3)])
            .unwrap();
        assert_eq!(result, Value::from_string("ababab".to_string()));
    }

    // Array method tests
    #[test]
    fn test_array_method_len() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = interp.eval_array_method(&arr, "len", &[]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_array_method_first() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let result = interp.eval_array_method(&arr, "first", &[]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_array_method_last() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let result = interp.eval_array_method(&arr, "last", &[]).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_array_method_is_empty() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![]);
        let result = interp.eval_array_method(&arr, "is_empty", &[]).unwrap();
        assert_eq!(result, Value::Bool(true));

        let arr2: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let result2 = interp.eval_array_method(&arr2, "is_empty", &[]).unwrap();
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_array_method_contains() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = interp
            .eval_array_method(&arr, "contains", &[Value::Integer(2)])
            .unwrap();
        assert_eq!(result, Value::Bool(true));

        let result2 = interp
            .eval_array_method(&arr, "contains", &[Value::Integer(5)])
            .unwrap();
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_array_method_join() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
            Value::from_string("c".to_string()),
        ]);
        let result = interp
            .eval_array_method(&arr, "join", &[Value::from_string(",".to_string())])
            .unwrap();
        assert_eq!(result, Value::from_string("a,b,c".to_string()));
    }

    #[test]
    fn test_array_method_reverse() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = interp.eval_array_method(&arr, "reverse", &[]).unwrap();
        if let Value::Array(rev) = result {
            assert_eq!(rev[0], Value::Integer(3));
            assert_eq!(rev[1], Value::Integer(2));
            assert_eq!(rev[2], Value::Integer(1));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_array_method_sum() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = interp.eval_array_method(&arr, "sum", &[]).unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_array_method_max() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(5),
            Value::Integer(3),
        ]);
        let result = interp.eval_array_method(&arr, "max", &[]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_array_method_min() {
        let mut interp = make_interpreter();
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(5),
            Value::Integer(3),
        ]);
        let result = interp.eval_array_method(&arr, "min", &[]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }
}
