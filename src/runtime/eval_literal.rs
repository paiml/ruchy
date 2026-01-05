//! Literal expression evaluation module
//!
//! This module handles evaluation of literal values in the interpreter.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::frontend::ast::Literal;
use crate::runtime::Value;

/// Evaluate a literal expression to its corresponding runtime value
///
/// # Complexity
/// Cyclomatic complexity: 9 (within limit of 10)
pub fn eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Integer(i, _) => Value::from_i64(*i),
        Literal::Float(f) => Value::from_f64(*f),
        Literal::String(s) => Value::from_string(s.clone()),
        Literal::Bool(b) => Value::from_bool(*b),
        Literal::Char(c) => Value::from_string(c.to_string()),
        Literal::Byte(b) => Value::Byte(*b),
        Literal::Unit => Value::nil(),
        Literal::Null => Value::nil(),
        Literal::Atom(s) => Value::Atom(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_literal() {
        let lit = Literal::Integer(42, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(42));
    }

    #[test]
    fn test_float_literal() {
        let lit = Literal::Float(3.15);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(3.15));
    }

    #[test]
    fn test_string_literal() {
        let lit = Literal::String("hello".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_bool_literal() {
        let lit = Literal::Bool(true);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_bool(true));
    }

    #[test]
    fn test_char_literal() {
        let lit = Literal::Char('a');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("a".to_string()));
    }

    #[test]
    fn test_unit_literal() {
        let lit = Literal::Unit;
        let value = eval_literal(&lit);
        assert_eq!(value, Value::nil());
    }

    #[test]
    fn test_null_literal() {
        let lit = Literal::Null;
        let value = eval_literal(&lit);
        assert_eq!(value, Value::nil());
    }

    #[test]
    fn test_atom_literal() {
        let lit = Literal::Atom("status".into());
        let value = eval_literal(&lit);
        // We can't easily construct Value::Atom directly here because it might not be exported or have a constructor helper
        // But we can check it via matching or if PartialEq is implemented
        match value {
            Value::Atom(s) => assert_eq!(s, "status"),
            _ => panic!("Expected Value::Atom"),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_integer_roundtrip(i in any::<i64>()) {
            let lit = Literal::Integer(i, None);
            let value = eval_literal(&lit);
            prop_assert_eq!(value, Value::from_i64(i));
        }

        #[test]
        fn test_float_roundtrip(f in any::<f64>().prop_filter("not NaN", |f| !f.is_nan())) {
            let lit = Literal::Float(f);
            let value = eval_literal(&lit);
            // Note: Value comparison handles float equality properly
            prop_assert_eq!(value, Value::from_f64(f));
        }

        #[test]
        fn test_string_roundtrip(s in ".*") {
            let lit = Literal::String(s.clone());
            let value = eval_literal(&lit);
            prop_assert_eq!(value, Value::from_string(s));
        }

        #[test]
        fn test_bool_roundtrip(b in any::<bool>()) {
            let lit = Literal::Bool(b);
            let value = eval_literal(&lit);
            prop_assert_eq!(value, Value::from_bool(b));
        }
    }
}

// === EXTREME TDD Round 27 - Coverage Push Tests ===

#[cfg(test)]
mod coverage_push_tests {
    use super::*;

    #[test]
    fn test_integer_negative() {
        let lit = Literal::Integer(-999, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(-999));
    }

    #[test]
    fn test_integer_zero() {
        let lit = Literal::Integer(0, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(0));
    }

    #[test]
    fn test_integer_max() {
        let lit = Literal::Integer(i64::MAX, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(i64::MAX));
    }

    #[test]
    fn test_integer_min() {
        let lit = Literal::Integer(i64::MIN, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(i64::MIN));
    }

    #[test]
    fn test_integer_with_suffix() {
        let lit = Literal::Integer(42, Some("u32".to_string()));
        let value = eval_literal(&lit);
        // Suffix is ignored in runtime value
        assert_eq!(value, Value::from_i64(42));
    }

    #[test]
    fn test_float_negative() {
        let lit = Literal::Float(-3.14159);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(-3.14159));
    }

    #[test]
    fn test_float_zero() {
        let lit = Literal::Float(0.0);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(0.0));
    }

    #[test]
    fn test_float_very_small() {
        let lit = Literal::Float(0.000001);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(0.000001));
    }

    #[test]
    fn test_float_very_large() {
        let lit = Literal::Float(1e100);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(1e100));
    }

    #[test]
    fn test_string_empty() {
        let lit = Literal::String("".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("".to_string()));
    }

    #[test]
    fn test_string_unicode() {
        let lit = Literal::String("日本語 🎉".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("日本語 🎉".to_string()));
    }

    #[test]
    fn test_string_whitespace() {
        let lit = Literal::String("  \t\n  ".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("  \t\n  ".to_string()));
    }

    #[test]
    fn test_bool_false() {
        let lit = Literal::Bool(false);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_bool(false));
    }

    #[test]
    fn test_char_unicode() {
        let lit = Literal::Char('日');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("日".to_string()));
    }

    #[test]
    fn test_char_newline() {
        let lit = Literal::Char('\n');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("\n".to_string()));
    }

    #[test]
    fn test_byte_zero() {
        let lit = Literal::Byte(0);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(0));
    }

    #[test]
    fn test_byte_max() {
        let lit = Literal::Byte(255);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(255));
    }

    #[test]
    fn test_byte_ascii_letter() {
        let lit = Literal::Byte(b'A');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(65));
    }

    #[test]
    fn test_atom_simple() {
        let lit = Literal::Atom("ok".into());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert_eq!(s, "ok"),
            _ => panic!("Expected Atom"),
        }
    }

    #[test]
    fn test_atom_with_underscore() {
        let lit = Literal::Atom("some_status".into());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert_eq!(s, "some_status"),
            _ => panic!("Expected Atom"),
        }
    }

    #[test]
    fn test_atom_empty() {
        let lit = Literal::Atom("".into());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert_eq!(s, ""),
            _ => panic!("Expected Atom"),
        }
    }

    #[test]
    fn test_unit_is_nil() {
        let lit = Literal::Unit;
        let value = eval_literal(&lit);
        assert_eq!(value, Value::nil());
    }

    #[test]
    fn test_null_is_nil() {
        let lit = Literal::Null;
        let value = eval_literal(&lit);
        assert_eq!(value, Value::nil());
    }

    #[test]
    fn test_unit_and_null_are_equal() {
        let unit = eval_literal(&Literal::Unit);
        let null = eval_literal(&Literal::Null);
        assert_eq!(unit, null);
    }
}
