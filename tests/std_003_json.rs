//! STD-003: JSON Module Tests (ruchy/std/json)
//!
//! Test suite for JSON operations module.
//! Thin wrappers around serde_json with Ruchy-friendly API.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

#[test]
fn test_std_003_parse_object() {
    // STD-003: Test parsing JSON object

    let json_str = r#"{"name": "Alice", "age": 30}"#;

    // Call ruchy::stdlib::json::parse
    let result = ruchy::stdlib::json::parse(json_str);

    assert!(result.is_ok(), "Valid JSON should parse successfully");
    let value = result.unwrap();

    // Verify it's an object
    assert!(value.is_object(), "Parsed value should be an object");
    assert!(!value.is_null(), "Parsed value must not be null");
    assert!(!value.is_array(), "Parsed value must not be array");

    // Verify fields exist
    let name = ruchy::stdlib::json::get(&value, "name");
    assert!(name.is_some(), "Must have 'name' field");
    let age = ruchy::stdlib::json::get(&value, "age");
    assert!(age.is_some(), "Must have 'age' field");
}

#[test]
fn test_std_003_parse_array() {
    // STD-003: Test parsing JSON array

    let json_str = r#"[1, 2, 3, 4, 5]"#;

    let result = ruchy::stdlib::json::parse(json_str);

    assert!(result.is_ok(), "Valid JSON array should parse");
    let value = result.unwrap();
    assert!(value.is_array(), "Parsed value should be an array");
    assert!(!value.is_null(), "Parsed value must not be null");
    assert!(!value.is_object(), "Parsed value must not be object");

    // Verify array has elements
    let first = ruchy::stdlib::json::get_index(&value, 0);
    assert!(first.is_some(), "Array must have first element");
    let last = ruchy::stdlib::json::get_index(&value, 4);
    assert!(last.is_some(), "Array must have last element");
}

#[test]
fn test_std_003_parse_primitives() {
    // STD-003: Test parsing JSON primitives

    // String
    let result = ruchy::stdlib::json::parse(r#""hello""#);
    assert!(result.is_ok());
    assert!(result.unwrap().is_string());

    // Number
    let result = ruchy::stdlib::json::parse("42");
    assert!(result.is_ok());
    assert!(result.unwrap().is_number());

    // Boolean
    let result = ruchy::stdlib::json::parse("true");
    assert!(result.is_ok());
    assert!(result.unwrap().is_boolean());

    // Null
    let result = ruchy::stdlib::json::parse("null");
    assert!(result.is_ok());
    assert!(result.unwrap().is_null());
}

#[test]
fn test_std_003_parse_invalid_json() {
    // STD-003: Test parsing invalid JSON returns error

    let invalid_cases = vec![
        "",                    // Empty
        "{",                   // Incomplete object
        "[1, 2,",              // Incomplete array
        r#"{"key": }"#,        // Missing value
        "undefined",           // Invalid literal
        r#"{'key': 'value'}"#, // Single quotes
    ];

    for invalid_json in invalid_cases {
        let result = ruchy::stdlib::json::parse(invalid_json);
        assert!(
            result.is_err(),
            "Invalid JSON should return error: {}",
            invalid_json
        );
    }
}

#[test]
fn test_std_003_stringify_object() {
    // STD-003: Test converting value to JSON string

    let json_str = r#"{"name":"Alice","age":30}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::stringify
    let result = ruchy::stdlib::json::stringify(&value);

    assert!(result.is_ok(), "Stringify should succeed");
    let output = result.unwrap();

    // Validate output
    assert!(!output.is_empty(), "Stringified output must not be empty");
    assert!(output.contains("name"), "Output must contain 'name' field");
    assert!(output.contains("Alice"), "Output must contain 'Alice' value");
    assert!(output.contains("age"), "Output must contain 'age' field");
    assert!(output.len() > 10, "Output must have reasonable length");

    // Should be valid JSON (parse it back)
    let reparsed = ruchy::stdlib::json::parse(&output);
    assert!(reparsed.is_ok(), "Stringified output should be valid JSON");
}

#[test]
fn test_std_003_stringify_array() {
    // STD-003: Test stringifying array

    let json_str = r#"[1,2,3,4,5]"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    let result = ruchy::stdlib::json::stringify(&value);

    assert!(result.is_ok(), "Stringify should succeed");
    let output = result.unwrap();
    assert!(!output.is_empty(), "Output must not be empty");
    assert!(output.contains('['), "Output must contain opening bracket");
    assert!(output.contains(']'), "Output must contain closing bracket");
    assert!(output.contains('1'), "Output must contain array elements");
    assert!(output.len() >= 11, "Output must have reasonable length");
}

#[test]
fn test_std_003_pretty_print() {
    // STD-003: Test pretty printing JSON

    let json_str = r#"{"name":"Alice","age":30,"active":true}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::pretty
    let result = ruchy::stdlib::json::pretty(&value);

    assert!(result.is_ok(), "Pretty print should succeed");
    let output = result.unwrap();

    // Validate output
    assert!(!output.is_empty(), "Pretty output must not be empty");
    assert!(output.contains('\n'), "Pretty output should have newlines");
    assert!(output.contains("name"), "Output must contain 'name' field");
    assert!(output.contains("Alice"), "Output must contain 'Alice' value");
    assert!(output.len() > json_str.len(), "Pretty output should be longer than compact");

    // Should still be valid JSON
    let reparsed = ruchy::stdlib::json::parse(&output);
    assert!(reparsed.is_ok(), "Pretty output should be valid JSON");
}

#[test]
fn test_std_003_get_field() {
    // STD-003: Test getting field from JSON object

    let json_str = r#"{"name": "Alice", "age": 30}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::get
    let name = ruchy::stdlib::json::get(&value, "name");
    assert!(name.is_some(), "Field 'name' should exist");
    assert!(name.unwrap().is_string());

    let age = ruchy::stdlib::json::get(&value, "age");
    assert!(age.is_some(), "Field 'age' should exist");
    assert!(age.unwrap().is_number());
}

#[test]
fn test_std_003_get_field_missing() {
    // STD-003: Test getting missing field returns None

    let json_str = r#"{"name": "Alice"}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    let missing = ruchy::stdlib::json::get(&value, "missing");
    assert!(missing.is_none(), "Missing field should return None");
}

#[test]
fn test_std_003_get_nested_field() {
    // STD-003: Test getting nested field with path

    let json_str = r#"{"user": {"name": "Alice", "address": {"city": "NYC"}}}"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::get_path
    let city = ruchy::stdlib::json::get_path(&value, &["user", "address", "city"]);
    assert!(city.is_some(), "Nested path should resolve");
    assert!(city.unwrap().is_string());
}

#[test]
fn test_std_003_get_array_index() {
    // STD-003: Test getting array element by index

    let json_str = r#"[10, 20, 30, 40]"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::get_index
    let elem = ruchy::stdlib::json::get_index(&value, 2);
    assert!(elem.is_some(), "Index 2 should exist");
    assert!(elem.unwrap().is_number());
}

#[test]
fn test_std_003_get_array_index_out_of_bounds() {
    // STD-003: Test getting out of bounds index returns None

    let json_str = r#"[10, 20, 30]"#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    let elem = ruchy::stdlib::json::get_index(&value, 10);
    assert!(elem.is_none(), "Out of bounds index should return None");
}

#[test]
fn test_std_003_as_string() {
    // STD-003: Test converting JSON value to Rust string

    let json_str = r#""hello world""#;
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::as_string
    let result = ruchy::stdlib::json::as_string(&value);
    assert!(result.is_some(), "String value should convert");
    let string = result.unwrap();
    assert_eq!(string, "hello world", "String must match exactly");
    assert_eq!(string.len(), 11, "String length must be 11");
    assert!(string.contains("hello"), "String must contain 'hello'");
    assert!(!string.is_empty(), "String must not be empty");
}

#[test]
fn test_std_003_as_i64() {
    // STD-003: Test converting JSON value to i64

    let json_str = "42";
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::as_i64
    let result = ruchy::stdlib::json::as_i64(&value);
    assert!(result.is_some(), "Number value should convert");
    let num = result.unwrap();
    assert_eq!(num, 42, "Number must be exactly 42");
    assert_ne!(num, 0, "Number must not be 0");
    assert_ne!(num, 1, "Number must not be 1");
    assert!(num > 0, "Number must be positive");
    assert!(num < 100, "Number must be less than 100");
}

#[test]
fn test_std_003_as_bool() {
    // STD-003: Test converting JSON value to bool

    let json_str = "true";
    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Call ruchy::stdlib::json::as_bool
    let result = ruchy::stdlib::json::as_bool(&value);
    assert!(result.is_some(), "Boolean value should convert");
    let bool_val = result.unwrap();
    assert_eq!(bool_val, true, "Boolean must be true");
    assert_ne!(bool_val, false, "Boolean must not be false");
    assert!(bool_val, "Boolean must be truthy");
}

#[test]
fn test_std_003_complex_nested_structure() {
    // STD-003: Test complex nested JSON structure

    let json_str = r#"{
        "users": [
            {"name": "Alice", "active": true},
            {"name": "Bob", "active": false}
        ],
        "count": 2
    }"#;

    let value = ruchy::stdlib::json::parse(json_str).unwrap();

    // Navigate to nested array
    let users = ruchy::stdlib::json::get(&value, "users");
    assert!(users.is_some());
    assert!(users.unwrap().is_array());

    // Get count field
    let count = ruchy::stdlib::json::get(&value, "count");
    assert!(count.is_some());
    assert_eq!(ruchy::stdlib::json::as_i64(count.unwrap()), Some(2));
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_003_parse_stringify_roundtrip(s in "[a-z]{1,20}") {
            // Property: Parsing then stringifying should preserve structure

            let json_str = format!(r#"{{"key":"{}"}}"#, s);
            let parsed = ruchy::stdlib::json::parse(&json_str);

            if let Ok(value) = parsed {
                let stringified = ruchy::stdlib::json::stringify(&value);
                if let Ok(output) = stringified {
                    // Re-parse to verify structure preserved
                    let reparsed = ruchy::stdlib::json::parse(&output);
                    assert!(reparsed.is_ok(), "Roundtrip should produce valid JSON");
                }
            }
        }

        #[test]
        fn test_std_003_parse_never_panics(s in "\\PC{0,100}") {
            // Property: Parse should never panic on any input

            let _ = ruchy::stdlib::json::parse(&s);
            // Should not panic
        }

        #[test]
        fn test_std_003_number_roundtrip(n in -1000i64..1000i64) {
            // Property: Numbers should roundtrip through JSON

            let json_str = n.to_string();
            let parsed = ruchy::stdlib::json::parse(&json_str);

            if let Ok(value) = parsed {
                let extracted = ruchy::stdlib::json::as_i64(&value);
                assert_eq!(extracted, Some(n), "Number should roundtrip");
            }
        }
    }
}
