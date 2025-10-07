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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_literal() {
        let lit = Literal::Integer(42);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_i64(42));
    }

    #[test]
    fn test_float_literal() {
        let lit = Literal::Float(3.14);
        let value = eval_literal(&lit);
        assert_eq!(value, Value::from_f64(3.14));
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_integer_roundtrip(i in any::<i64>()) {
            let lit = Literal::Integer(i);
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
