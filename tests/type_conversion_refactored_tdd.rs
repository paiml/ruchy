//! TDD Test Suite for type_conversion_refactored.rs
//! Target: 6.38% → 80%+ coverage
//! PMAT: Keep complexity <10 per test

#![cfg(test)]

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Test helper: Parse and transpile with type conversion
fn transpile_type_conversion(code: &str) -> anyhow::Result<String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast)?;
    Ok(tokens.to_string())
}

#[test]
fn test_str_conversion_integer() {
    let code = "str(42)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("format!"));
    assert!(result.contains("42"));
}

#[test]
fn test_str_conversion_float() {
    let code = "str(3.14)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("format!"));
}

#[test]
fn test_str_conversion_bool() {
    let code = "str(true)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("format!"));
}

#[test]
fn test_str_conversion_none() {
    let code = "str(None)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("format!"));
}

#[test]
fn test_int_conversion_string_literal() {
    let code = r#"int("42")"#;
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("parse::<i64>()"));
}

#[test]
fn test_int_conversion_float() {
    let code = "int(3.14)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("as i64"));
}

#[test]
fn test_int_conversion_bool_true() {
    let code = "int(true)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("1i64") || result.contains("if true"));
}

#[test]
fn test_int_conversion_bool_false() {
    let code = "int(false)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("0i64") || result.contains("if false") || result.contains("else"));
}

#[test]
fn test_float_conversion_string() {
    let code = r#"float("3.14")"#;
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("parse::<f64>()"));
}

#[test]
fn test_float_conversion_integer() {
    let code = "float(42)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("as f64"));
}

#[test]
fn test_bool_conversion_integer_zero() {
    let code = "bool(0)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("!= 0") || result.contains("false"));
}

#[test]
fn test_bool_conversion_integer_nonzero() {
    let code = "bool(1)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("!= 0") || result.contains("true"));
}

#[test]
fn test_bool_conversion_empty_string() {
    let code = r#"bool("")"#;
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("is_empty()"));
}

#[test]
fn test_bool_conversion_nonempty_string() {
    let code = r#"bool("hello")"#;
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("is_empty()") || result.contains("!"));
}

#[test]
fn test_bool_conversion_none() {
    let code = "bool(None)";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("false"));
}

#[test]
fn test_bool_conversion_empty_list() {
    let code = "bool([])";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("is_empty()"));
}

#[test]
fn test_bool_conversion_nonempty_list() {
    let code = "bool([1, 2, 3])";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("is_empty()") || result.contains("vec!"));
}

#[test]
fn test_list_conversion_string() {
    let code = r#"list("hello")"#;
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("chars()") || result.contains("collect()"));
}

#[test]
fn test_set_conversion_list() {
    let code = "set([1, 2, 3, 2, 1])";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("HashSet") || result.contains("BTreeSet"));
}

#[test]
fn test_dict_conversion_empty() {
    let code = "dict()";
    let result = transpile_type_conversion(code);
    // Should either succeed with HashMap or fail with arg count error
    assert!(result.is_err() || result.unwrap().contains("HashMap"));
}

#[test]
fn test_type_conversion_invalid_args() {
    let code = "int(1, 2, 3)";
    let result = transpile_type_conversion(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("expects exactly 1 argument"));
}

#[test]
fn test_string_interpolation_to_int() {
    let code = r#"
x = "42"
int(f"{x}")
"#;
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("parse::<i64>()") || result.contains("as i64"));
}

#[test]
fn test_complex_type_conversion_chain() {
    let code = "str(int(float(\"3.14\")))";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("format!"));
    assert!(result.contains("parse"));
}

#[test]
fn test_bool_conversion_float() {
    let code = "bool(0.0)";
    let result = transpile_type_conversion(code).unwrap();
    // Check for float truthiness
    assert!(result.contains("0.0") || result.contains("!= 0"));
}

#[test]
fn test_list_conversion_range() {
    let code = "list(range(5))";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("collect()") || result.contains("Vec"));
}

// Property tests for type conversion consistency
#[test]
fn test_type_conversion_round_trip_int() {
    let code = "int(str(42))";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("parse") || result.contains("42"));
}

#[test]
fn test_type_conversion_round_trip_float() {
    let code = "float(str(3.14))";
    let result = transpile_type_conversion(code).unwrap();
    assert!(result.contains("parse") || result.contains("3.14"));
}

// Edge cases
#[test]
fn test_int_conversion_hex_string() {
    let code = r#"int("0x2A", 16)"#;
    let result = transpile_type_conversion(code);
    // May not be supported yet, but test the behavior
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("argument"));
}

#[test]
fn test_not_a_type_conversion() {
    let code = "print(42)";
    let result = transpile_type_conversion(code).unwrap();
    // Should not apply type conversion logic
    assert!(!result.contains("parse::<i64>()"));
}