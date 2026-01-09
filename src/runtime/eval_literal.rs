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

// ============================================================================
// EXTREME TDD Round 133: Additional comprehensive tests
// Target: 32 → 50+ tests
// ============================================================================
#[cfg(test)]
mod round_133_tests {
    use super::*;

    // --- Integer edge cases ---
    #[test]
    fn test_integer_one() {
        let lit = Literal::Integer(1, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(1));
    }

    #[test]
    fn test_integer_minus_one() {
        let lit = Literal::Integer(-1, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(-1));
    }

    #[test]
    fn test_integer_large_positive() {
        let lit = Literal::Integer(999_999_999_999, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(999_999_999_999));
    }

    #[test]
    fn test_integer_large_negative() {
        let lit = Literal::Integer(-999_999_999_999, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(-999_999_999_999));
    }

    #[test]
    fn test_integer_with_i64_suffix() {
        let lit = Literal::Integer(42, Some("i64".to_string()));
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(42));
    }

    // --- Float edge cases ---
    #[test]
    fn test_float_one() {
        let lit = Literal::Float(1.0);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(1.0));
    }

    #[test]
    fn test_float_minus_one() {
        let lit = Literal::Float(-1.0);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(-1.0));
    }

    #[test]
    fn test_float_positive_infinity() {
        let lit = Literal::Float(f64::INFINITY);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(f64::INFINITY));
    }

    #[test]
    fn test_float_negative_infinity() {
        let lit = Literal::Float(f64::NEG_INFINITY);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(f64::NEG_INFINITY));
    }

    #[test]
    fn test_float_scientific_notation_large() {
        let lit = Literal::Float(1.5e10);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(1.5e10));
    }

    #[test]
    fn test_float_scientific_notation_small() {
        let lit = Literal::Float(1.5e-10);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(1.5e-10));
    }

    // --- String edge cases ---
    #[test]
    fn test_string_single_char() {
        let lit = Literal::String("x".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("x".to_string()));
    }

    #[test]
    fn test_string_very_long() {
        let long_str = "a".repeat(10000);
        let lit = Literal::String(long_str.clone());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string(long_str));
    }

    #[test]
    fn test_string_special_chars() {
        let lit = Literal::String("!@#$%^&*()".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("!@#$%^&*()".to_string()));
    }

    #[test]
    fn test_string_escape_sequences() {
        let lit = Literal::String("line1\\nline2\\ttab".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("line1\\nline2\\ttab".to_string()));
    }

    // --- Char edge cases ---
    #[test]
    fn test_char_space() {
        let lit = Literal::Char(' ');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string(" ".to_string()));
    }

    #[test]
    fn test_char_tab() {
        let lit = Literal::Char('\t');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("\t".to_string()));
    }

    #[test]
    fn test_char_carriage_return() {
        let lit = Literal::Char('\r');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("\r".to_string()));
    }

    #[test]
    fn test_char_emoji() {
        let lit = Literal::Char('🎉');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("🎉".to_string()));
    }

    #[test]
    fn test_char_zero() {
        let lit = Literal::Char('0');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("0".to_string()));
    }

    // --- Byte edge cases ---
    #[test]
    fn test_byte_one() {
        let lit = Literal::Byte(1);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(1));
    }

    #[test]
    fn test_byte_middle() {
        let lit = Literal::Byte(127);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(127));
    }

    #[test]
    fn test_byte_ascii_digit() {
        let lit = Literal::Byte(b'5');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(53));
    }

    #[test]
    fn test_byte_ascii_lowercase() {
        let lit = Literal::Byte(b'z');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(122));
    }

    // --- Atom edge cases ---
    #[test]
    fn test_atom_uppercase() {
        let lit = Literal::Atom("OK".into());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert_eq!(s, "OK"),
            _ => panic!("Expected Atom"),
        }
    }

    #[test]
    fn test_atom_mixed_case() {
        let lit = Literal::Atom("MyAtom".into());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert_eq!(s, "MyAtom"),
            _ => panic!("Expected Atom"),
        }
    }

    #[test]
    fn test_atom_with_numbers() {
        let lit = Literal::Atom("status_123".into());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert_eq!(s, "status_123"),
            _ => panic!("Expected Atom"),
        }
    }

    // === EXTREME TDD Round 138 tests ===

    #[test]
    fn test_integer_min_value() {
        let lit = Literal::Integer(i64::MIN, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(i64::MIN));
    }

    #[test]
    fn test_integer_max_value() {
        let lit = Literal::Integer(i64::MAX, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(i64::MAX));
    }

    #[test]
    fn test_float_negative() {
        let lit = Literal::Float(-2.718);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(-2.718));
    }

    #[test]
    fn test_float_zero() {
        let lit = Literal::Float(0.0);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(0.0));
    }

    #[test]
    fn test_float_very_small() {
        let lit = Literal::Float(1e-10);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(1e-10));
    }

    #[test]
    fn test_string_empty() {
        let lit = Literal::String(String::new());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string(String::new()));
    }

    #[test]
    fn test_string_unicode() {
        let lit = Literal::String("日本語".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("日本語".to_string()));
    }

    #[test]
    fn test_string_with_newline() {
        let lit = Literal::String("line1\nline2".into());
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("line1\nline2".to_string()));
    }

    #[test]
    fn test_bool_false() {
        let lit = Literal::Bool(false);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_bool(false));
    }

    #[test]
    fn test_char_unicode() {
        let lit = Literal::Char('漢');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("漢".to_string()));
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
    fn test_atom_empty() {
        let lit = Literal::Atom(String::new());
        let value = eval_literal(&lit);
        match value {
            Value::Atom(s) => assert!(s.is_empty()),
            _ => panic!("Expected Atom"),
        }
    }

    // === EXTREME TDD Round 160 - Coverage Push Tests ===

    #[test]
    fn test_integer_zero_r160() {
        let lit = Literal::Integer(0, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Integer(0));
    }

    #[test]
    fn test_integer_negative_r160() {
        let lit = Literal::Integer(-42, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Integer(-42));
    }

    #[test]
    fn test_integer_max_r160() {
        let lit = Literal::Integer(i64::MAX, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Integer(i64::MAX));
    }

    #[test]
    fn test_integer_min_r160() {
        let lit = Literal::Integer(i64::MIN, None);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Integer(i64::MIN));
    }

    #[test]
    fn test_float_zero_r160() {
        let lit = Literal::Float(0.0);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Float(0.0));
    }

    #[test]
    fn test_float_negative_r160() {
        let lit = Literal::Float(-3.14);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Float(-3.14));
    }

    #[test]
    fn test_float_very_small_r160() {
        let lit = Literal::Float(0.000001);
        let value = eval_literal(&lit);
        if let Value::Float(f) = value {
            assert!(f < 0.0001);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_float_very_large_r160() {
        let lit = Literal::Float(1e100);
        let value = eval_literal(&lit);
        if let Value::Float(f) = value {
            assert!(f > 1e99);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_string_empty_r160() {
        let lit = Literal::String(String::new());
        let value = eval_literal(&lit);
        if let Value::String(s) = value {
            assert!(s.is_empty());
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_string_with_spaces_r160() {
        let lit = Literal::String("  spaces  ".to_string());
        let value = eval_literal(&lit);
        if let Value::String(s) = value {
            assert_eq!(s.as_ref(), "  spaces  ");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_string_unicode_emoji_r160() {
        let lit = Literal::String("hello 🌍".to_string());
        let value = eval_literal(&lit);
        if let Value::String(s) = value {
            assert!(s.contains("🌍"));
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_bool_true_r160() {
        let lit = Literal::Bool(true);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn test_bool_false_r160() {
        let lit = Literal::Bool(false);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Bool(false));
    }

    #[test]
    fn test_char_space_r160() {
        let lit = Literal::Char(' ');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string(" ".to_string()));
    }

    #[test]
    fn test_char_tab_r160() {
        let lit = Literal::Char('\t');
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_string("\t".to_string()));
    }

    #[test]
    fn test_byte_mid_range_r160() {
        let lit = Literal::Byte(128);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Byte(128));
    }

    #[test]
    fn test_unit_r160() {
        let lit = Literal::Unit;
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Nil);
    }

    #[test]
    fn test_null_r160() {
        let lit = Literal::Null;
        let value = eval_literal(&lit);
        assert_eq!(value, Value::Nil);
    }

    #[test]
    fn test_atom_with_special_chars_r160() {
        let lit = Literal::Atom("ok_value".to_string());
        let value = eval_literal(&lit);
        if let Value::Atom(s) = value {
            assert_eq!(s, "ok_value");
        } else {
            panic!("Expected Atom");
        }
    }

    #[test]
    fn test_atom_unicode_r160() {
        let lit = Literal::Atom("日本語".to_string());
        let value = eval_literal(&lit);
        if let Value::Atom(s) = value {
            assert_eq!(s, "日本語");
        } else {
            panic!("Expected Atom");
        }
    }
}
